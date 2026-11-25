//! Consumer lifecycle discipline — LC-013, LC-015, LC-016
//! (`specs/consumer-lifecycle-scope-rules.md`; Arrow RFC-010 Part 4).
//!
//! Pure, deterministic checks over an explicit [`LifecycleView`]: the caller
//! supplies the lifecycle's recorded events, the ratification watermark, and
//! the invocation time. Nothing here reads a clock, a store, or a network —
//! the runtime stays AI-free and ambient-state-free (ADR-0001). The router
//! consults these checks before dispatch; the reduction path may call
//! [`closure_staleness`] with the same inputs.

use arrow_core::types::event::ArrowEvent;

/// The caller-supplied view of one lifecycle at a decision point.
#[derive(Debug, Clone)]
pub struct LifecycleView<'a> {
    /// Events recorded for this lifecycle, in append order.
    pub events: &'a [ArrowEvent],
    /// How many of `events` were already recorded when the human ratified.
    /// Events at indices `>= ratified_at_len` arrived after ratification.
    pub ratified_at_len: usize,
    /// Invocation-supplied current time (unix seconds). Explicit input,
    /// like every time value in this runtime.
    pub now_unix: u64,
}

impl<'a> LifecycleView<'a> {
    /// A lifecycle with no recorded history (nothing to check against).
    pub fn empty(now_unix: u64) -> LifecycleView<'static> {
        LifecycleView {
            events: &[],
            ratified_at_len: 0,
            now_unix,
        }
    }
}

/// Why the lifecycle discipline refused execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleRefusal {
    /// LC-013: a reversal or supersession landed after ratification — the
    /// ratified world-state is superseded; back to review.
    SupersededSinceRatification,
    /// LC-015: the lifecycle holds a Class 6 closure — terminal; nothing
    /// executes for it (reopenings route to human review, not here).
    LifecycleClosed,
    /// LC-016: the lifecycle's declared closure horizon has passed without
    /// a closure — stale, and possibly under attack; refuse and review.
    ClosureOverdue,
}

// Reopening/closure shape predicates are SHARED with the Arrow gate
// (arrow_core::validation::reopening) so the consumer and LFC-007 cannot
// drift when SEM-021 promotes the dormant subtype clause. Each predicate
// parses the code once.
use arrow_core::validation::reopening::{is_closure, is_reversal, is_supersession};

/// LC-013 — lifecycle-head check. True if a reversal or supersession landed
/// at or after the ratification watermark.
pub fn superseded_since_ratification(view: &LifecycleView) -> bool {
    view.events[view.ratified_at_len.min(view.events.len())..]
        .iter()
        .any(|e| is_reversal(e) || is_supersession(e))
}

/// LC-015 — terminal check. True if the lifecycle holds a Class 6 closure.
pub fn lifecycle_closed(view: &LifecycleView) -> bool {
    view.events.iter().any(is_closure)
}

/// LC-016 — closure staleness. True if a settled event declared a dated
/// closure horizon (`closure_eta` with `closure_eta_type = "unix_ts"`), that
/// horizon precedes `now_unix`, and no closure has landed.
///
/// Scope notes (deterministic by construction):
/// - Only `unix_ts` horizons are comparable to the invocation time.
///   `block_count` horizons need a chain-height input and are the
///   registry-derived fallback's job when a height is supplied (future
///   extension; absent input means no claim, fail-open to the OTHER checks,
///   never a fabricated verdict).
/// - `legal_exhaustion` lifecycles carry no `closure_eta` (Arrow LFC-008
///   exempts them), so they are never presumed overdue by wall-clock —
///   exactly the architect's weeks-order-legal ruling.
pub fn closure_staleness(view: &LifecycleView) -> bool {
    if lifecycle_closed(view) {
        return false;
    }
    view.events.iter().any(|e| {
        e.closure_eta_type.as_deref() == Some("unix_ts")
            && e.closure_eta
                .map(|eta| eta < view.now_unix)
                .unwrap_or(false)
    })
}

/// Run the full lifecycle discipline in rule order. `None` means clear to
/// proceed to the remaining gates (ratification, Sibyl, single-shot).
pub fn check(view: &LifecycleView) -> Option<LifecycleRefusal> {
    if lifecycle_closed(view) {
        return Some(LifecycleRefusal::LifecycleClosed);
    }
    if superseded_since_ratification(view) {
        return Some(LifecycleRefusal::SupersededSinceRatification);
    }
    if closure_staleness(view) {
        return Some(LifecycleRefusal::ClosureOverdue);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(code: &str, id: &str) -> ArrowEvent {
        ArrowEvent {
            arrow_code: code.to_string(),
            event_id: id.to_string(),
            lifecycle_id: "lfc-test".to_string(),
            timestamp: "2026-07-07T00:00:00Z".to_string(),
            object_type: "t".to_string(),
            object_id: "o".to_string(),
            ..Default::default()
        }
    }

    fn ev_with_eta(code: &str, id: &str, eta: u64) -> ArrowEvent {
        let mut e = ev(code, id);
        e.closure_eta = Some(eta);
        e.closure_eta_type = Some("unix_ts".to_string());
        e
    }

    const NOW: u64 = 1_800_000_000;

    #[test]
    fn lc013_reversal_after_ratification_refuses() {
        let events = vec![
            ev("2:LEG:FN:PR:1:C", "e1"),
            ev("1:ETH:RV:HL:1:A", "e2-reversal"),
        ];
        // Ratified when only e1 existed; the reversal landed after.
        let view = LifecycleView {
            events: &events,
            ratified_at_len: 1,
            now_unix: NOW,
        };
        assert_eq!(
            check(&view),
            Some(LifecycleRefusal::SupersededSinceRatification)
        );
    }

    #[test]
    fn lc013_reversal_before_ratification_is_the_humans_call() {
        // The reversal was already visible when the human ratified — LC-013
        // guards against post-ratification movement only.
        let events = vec![
            ev("2:LEG:FN:PR:1:C", "e1"),
            ev("1:ETH:RV:HL:1:A", "e2-reversal"),
        ];
        let view = LifecycleView {
            events: &events,
            ratified_at_len: 2,
            now_unix: NOW,
        };
        assert_eq!(check(&view), None);
    }

    #[test]
    fn lc015_closed_lifecycle_refuses_regardless_of_watermark() {
        let events = vec![
            ev("2:LEG:FN:PR:1:C", "e1"),
            ev("6:ETH:FN:CL:1:A", "e2-closure"),
        ];
        let view = LifecycleView {
            events: &events,
            ratified_at_len: 2,
            now_unix: NOW,
        };
        assert_eq!(check(&view), Some(LifecycleRefusal::LifecycleClosed));
    }

    #[test]
    fn lc016_overdue_unclosed_refuses() {
        // Settled event declared closure due at NOW-100; no closure landed.
        let events = vec![ev_with_eta("1:ETH:FN:PR:1:A", "e1", NOW - 100)];
        let view = LifecycleView {
            events: &events,
            ratified_at_len: 1,
            now_unix: NOW,
        };
        assert_eq!(check(&view), Some(LifecycleRefusal::ClosureOverdue));
    }

    #[test]
    fn lc016_within_horizon_is_clear() {
        let events = vec![ev_with_eta("1:ETH:FN:PR:1:A", "e1", NOW + 3600)];
        let view = LifecycleView {
            events: &events,
            ratified_at_len: 1,
            now_unix: NOW,
        };
        assert_eq!(check(&view), None);
    }

    #[test]
    fn lc016_closed_on_time_is_clear_even_past_eta() {
        // Horizon passed but the closure IS there — the obligation was met.
        // (check() reports LifecycleClosed for execution purposes; the
        // staleness predicate itself is false.)
        let events = vec![
            ev_with_eta("1:ETH:FN:PR:1:A", "e1", NOW - 100),
            ev("6:ETH:FN:CL:1:A", "e2-closure"),
        ];
        let view = LifecycleView {
            events: &events,
            ratified_at_len: 2,
            now_unix: NOW,
        };
        assert!(!closure_staleness(&view));
        assert_eq!(check(&view), Some(LifecycleRefusal::LifecycleClosed));
    }

    #[test]
    fn empty_lifecycle_is_clear() {
        assert_eq!(check(&LifecycleView::empty(NOW)), None);
    }
}
