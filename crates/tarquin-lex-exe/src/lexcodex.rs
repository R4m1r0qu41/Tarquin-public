//! Lex.Codex stub — pinned Orlog SLC bundle verification and fixture recomputation.
//!
//! This is the **minimal** Lex.Codex surface needed for the first SLC handoff dry run
//! (Orlog MC-006 → Tarquin). It does three things, all deterministic, no I/O beyond
//! reading the pinned bundle, no clock, no AI (ADR-0001):
//!
//! 1. **Seal check** — recompute the MC-006 `artifact_hash` over a pinned bundle directory and
//!    refuse the bundle on any drift ([`compute_artifact_hash`] / [`verify_pinned_bundle`]).
//! 2. **Recompute** — re-run the `slc-late-payment-001` contract logic for a fixture case in
//!    Rust and confirm the result matches the bundle's own `expected` block
//!    ([`recompute_case`]). The TypeScript in `logic/logic.ts` remains the authoring-side
//!    source; this port is the curation-side check that the sealed fixture is reproducible.
//! 3. **Bind** — produce the [`SlcArtifactRef`] (`{artifact_id, version, artifact_hash,
//!    fixture_case, computed_result_hash}`) that an LBRY `structured_fact` carries, and
//!    confirm a presented reference against a fresh recomputation ([`confirm_artifact_ref`]).
//!
//! Boundary (ADR-0001 / MC-006): Orlog authors SLCs and never emits Arrow events;
//! Tarquin pins copies and never authors SLCs. A computed number is **not** a settled legal
//! claim — entitlement contests ride the dispute block and keep the Action proposal at `HL`.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use tarquin_sibyl::lbry::SlcArtifactRef;

/// Errors from bundle verification or fixture recomputation.
#[derive(Debug, thiserror::Error)]
pub enum LexCodexError {
    #[error("io error reading pinned bundle: {0}")]
    Io(#[from] std::io::Error),
    #[error("malformed bundle JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("artifact hash mismatch: manifest declares {declared}, recomputed {computed}")]
    ArtifactHashMismatch { declared: String, computed: String },
    #[error("pinned artifact hash mismatch: pin expects {expected}, recomputed {computed}")]
    PinMismatch { expected: String, computed: String },
    #[error("fixture case not found: {0}")]
    CaseNotFound(String),
    #[error("recomputed result diverges from sealed expected block for case {case}: {detail}")]
    ResultDrift { case: String, detail: String },
    #[error("invalid contract input: {0}")]
    InvalidInput(String),
    #[error("artifact reference does not confirm: {0}")]
    RefMismatch(String),
}

// ───────────────────────────── MC-006 artifact hash ─────────────────────────────

/// Recompute the MC-006 `artifact_hash` over a bundle directory.
///
/// Port of Orlog `tools/validate.mjs::artifactHash`: every file under the bundle in
/// sorted path order (directories descended in sorted name order); each entry contributes
/// `relative_posix_path NUL payload NUL` to one SHA-256 stream; `manifest.json` is included
/// after deleting its own `artifact_hash` field and serializing the remaining object with
/// sorted keys (2-space-indent JSON plus trailing newline).
pub fn compute_artifact_hash(bundle_dir: &Path) -> Result<String, LexCodexError> {
    let mut hasher = Sha256::new();
    let mut files = Vec::new();
    collect_sorted(bundle_dir, bundle_dir, &mut files)?;
    for (rel, full) in files {
        hasher.update(rel.as_bytes());
        hasher.update([0u8]);
        if rel == "manifest.json" {
            let raw = fs::read_to_string(&full)?;
            // serde_json's default Map is BTreeMap: parsing re-sorts keys recursively,
            // matching validate.mjs's stableJson.
            let mut v: serde_json::Value = serde_json::from_str(&raw)?;
            if let Some(obj) = v.as_object_mut() {
                obj.remove("artifact_hash");
            }
            let pretty = serde_json::to_string_pretty(&v)?;
            hasher.update(pretty.as_bytes());
            hasher.update(b"\n");
        } else {
            hasher.update(fs::read(&full)?);
        }
        hasher.update([0u8]);
    }
    Ok(format!("sha256:{}", hex_lower(&hasher.finalize())))
}

fn collect_sorted(
    base: &Path,
    dir: &Path,
    out: &mut Vec<(String, std::path::PathBuf)>,
) -> Result<(), LexCodexError> {
    let mut names: Vec<_> = fs::read_dir(dir)?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|e| e.path())
        .collect();
    names.sort_by_key(|p| p.file_name().map(|n| n.to_os_string()));
    for path in names {
        if path.is_dir() {
            collect_sorted(base, &path, out)?;
        } else {
            let rel = path
                .strip_prefix(base)
                .expect("path under base")
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            out.push((rel, path));
        }
    }
    Ok(())
}

fn hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Verify a pinned bundle before any use: the recomputed hash MUST match both the bundle's
/// own `manifest.json` declaration and the pin's expected hash. Drift rejects the bundle.
pub fn verify_pinned_bundle(bundle_dir: &Path, pinned_hash: &str) -> Result<String, LexCodexError> {
    let computed = compute_artifact_hash(bundle_dir)?;
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(bundle_dir.join("manifest.json"))?)?;
    let declared = manifest
        .get("artifact_hash")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    if computed != declared {
        return Err(LexCodexError::ArtifactHashMismatch { declared, computed });
    }
    if computed != pinned_hash {
        return Err(LexCodexError::PinMismatch {
            expected: pinned_hash.to_string(),
            computed,
        });
    }
    Ok(computed)
}

// ──────────────────── slc-late-payment-001 deterministic logic ────────────────────
//
// Faithful Rust port of the pinned `logic/logic.ts` (pure functions; ACT/365F; round
// half-up to 2 decimals once per computed amount). The fixture recomputation below proves
// the port against the sealed `expected` blocks.

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatePaymentTerms {
    pub principal: f64,
    pub currency: String,
    pub due_date: String,
    pub grace_days: i64,
    pub accrual_basis: String,
    pub charge_kind: String,
    pub rate_or_amount: f64,
    pub cap_kind: String,
    pub cap_value: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentEvent {
    pub paid_date: String,
    pub amount_paid: f64,
}

/// Result keys mirror the fixture `expected` block (camelCase) so the recomputed object and
/// the sealed expectation compare and hash over the same shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatePaymentResult {
    pub days_late: i64,
    pub chargeable_days: i64,
    pub late_charge: f64,
    pub capped: bool,
    pub total_due: f64,
    pub obligation: String,
}

/// UTC day number for a strict `YYYY-MM-DD` date (port of `dayNumber`).
fn day_number(iso: &str) -> Result<i64, LexCodexError> {
    let parts: Vec<&str> = iso.split('-').collect();
    let bad = || LexCodexError::InvalidInput(format!("invalid ISO date: {iso}"));
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return Err(bad());
    }
    let y: i32 = parts[0].parse().map_err(|_| bad())?;
    let m: u32 = parts[1].parse().map_err(|_| bad())?;
    let d: u32 = parts[2].parse().map_err(|_| bad())?;
    let date = chrono::NaiveDate::from_ymd_opt(y, m, d)
        .ok_or_else(|| LexCodexError::InvalidInput(format!("impossible calendar date: {iso}")))?;
    let epoch = chrono::NaiveDate::from_ymd_opt(1970, 1, 1).expect("epoch");
    Ok((date - epoch).num_days())
}

/// Round half-up to 2 decimals, once, at the end of a computed amount (port of `round2`).
fn round2(x: f64) -> f64 {
    ((x + f64::EPSILON) * 100.0).round() / 100.0
}

fn non_negative(name: &str, v: f64) -> Result<(), LexCodexError> {
    if !v.is_finite() || v < 0.0 {
        return Err(LexCodexError::InvalidInput(format!(
            "{name} must be a finite non-negative number, got {v}"
        )));
    }
    Ok(())
}

/// prim-grace-period: days that count toward the charge.
fn chargeable_days(
    due_date: &str,
    performed_date: &str,
    grace_days: i64,
    basis: &str,
) -> Result<i64, LexCodexError> {
    if grace_days < 0 {
        return Err(LexCodexError::InvalidInput(format!(
            "graceDays must be a non-negative integer, got {grace_days}"
        )));
    }
    let due = day_number(due_date)?;
    let performed = day_number(performed_date)?;
    if performed <= due + grace_days {
        return Ok(0);
    }
    Ok(if basis == "FROM_DUE_DATE" {
        performed - due
    } else {
        performed - (due + grace_days)
    })
}

/// prim-late-fee-interest: charge before cap. `ANNUAL_RATE` is simple ACT/365F.
fn late_charge_amount(
    base_amount: f64,
    days: i64,
    kind: &str,
    rate_or_amount: f64,
) -> Result<f64, LexCodexError> {
    non_negative("baseAmount", base_amount)?;
    non_negative("rateOrAmount", rate_or_amount)?;
    if days < 0 {
        return Err(LexCodexError::InvalidInput(format!(
            "days must be a non-negative integer, got {days}"
        )));
    }
    if days == 0 {
        return Ok(0.0);
    }
    if kind == "ANNUAL_RATE" {
        return Ok(round2(
            base_amount * (rate_or_amount / 100.0) * days as f64 / 365.0,
        ));
    }
    Ok(round2(rate_or_amount * days as f64))
}

/// prim-remedy-cap: cap never increases a charge.
fn apply_cap(
    computed_charge: f64,
    kind: &str,
    cap_value: f64,
    reference_amount: f64,
) -> Result<(f64, bool), LexCodexError> {
    non_negative("computedCharge", computed_charge)?;
    non_negative("capValue", cap_value)?;
    non_negative("referenceAmount", reference_amount)?;
    let limit = if kind == "ABSOLUTE_AMOUNT" {
        cap_value
    } else {
        round2(reference_amount * cap_value / 100.0)
    };
    if computed_charge > limit {
        Ok((limit, true))
    } else {
        Ok((computed_charge, false))
    }
}

/// Contract logic: process a payment event against the agreed terms
/// (port of `evaluateLatePayment`).
pub fn evaluate_late_payment(
    terms: &LatePaymentTerms,
    event: &PaymentEvent,
) -> Result<LatePaymentResult, LexCodexError> {
    non_negative("principal", terms.principal)?;
    non_negative("amountPaid", event.amount_paid)?;
    if round2(event.amount_paid) != round2(terms.principal) {
        return Err(LexCodexError::InvalidInput(
            "amountPaid must equal principal for this payment-in-full artifact".into(),
        ));
    }
    let due = day_number(&terms.due_date)?;
    let paid = day_number(&event.paid_date)?;
    let days_late = (paid - due).max(0);
    let days = chargeable_days(
        &terms.due_date,
        &event.paid_date,
        terms.grace_days,
        &terms.accrual_basis,
    )?;
    let raw = late_charge_amount(
        terms.principal,
        days,
        &terms.charge_kind,
        terms.rate_or_amount,
    )?;
    let (charge, capped) = apply_cap(raw, &terms.cap_kind, terms.cap_value, terms.principal)?;
    Ok(LatePaymentResult {
        days_late,
        chargeable_days: days,
        late_charge: charge,
        capped,
        total_due: round2(terms.principal + charge),
        obligation: if charge > 0.0 {
            "LATE_CHARGE_DUE".into()
        } else {
            "NONE".into()
        },
    })
}

// ───────────────────────── fixture recomputation and binding ─────────────────────────

#[derive(Debug, Clone, Deserialize)]
struct FixtureFile {
    #[allow(dead_code)]
    description: String,
    cases: Vec<FixtureCase>,
}

/// One sealed fixture case. The `dispute` block, when present, carries the
/// computation-vs-entitlement distinction the consumer must respect.
#[derive(Debug, Clone, Deserialize)]
pub struct FixtureCase {
    pub case_id: String,
    pub name: String,
    pub terms: LatePaymentTerms,
    pub event: PaymentEvent,
    pub expected: serde_json::Value,
    #[serde(default)]
    pub dispute: Option<DisputeBlock>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisputeBlock {
    pub status: String,
    pub contention: String,
    pub computation_contested: bool,
    pub entitlement_contested: bool,
    pub requires_human_review: bool,
}

/// A confirmed recomputation: the sealed case, the recomputed result, and its hash.
#[derive(Debug, Clone)]
pub struct Recomputation {
    pub case: FixtureCase,
    pub result: LatePaymentResult,
    pub computed_result_hash: String,
}

/// `sha256:<hex>` over the sorted-key compact JSON of a recomputed result. serde_json's
/// default map is key-sorted, so this is deterministic for a fixed result shape.
pub fn computed_result_hash(result: &LatePaymentResult) -> Result<String, LexCodexError> {
    let v = serde_json::to_value(result)?;
    let bytes = serde_json::to_vec(&v)?;
    Ok(format!("sha256:{}", hex_lower(&Sha256::digest(&bytes))))
}

/// Recompute one fixture case from a verified bundle and confirm it against the sealed
/// `expected` block. Any divergence is a hard error: a pinned bundle whose fixtures do not
/// reproduce MUST NOT back a legal-attestation reference.
pub fn recompute_case(
    bundle_dir: &Path,
    fixture_file: &str,
    case_id: &str,
) -> Result<Recomputation, LexCodexError> {
    let raw = fs::read_to_string(bundle_dir.join(fixture_file))?;
    let fixture: FixtureFile = serde_json::from_str(&raw)?;
    let case = fixture
        .cases
        .into_iter()
        .find(|c| c.case_id == case_id)
        .ok_or_else(|| LexCodexError::CaseNotFound(case_id.to_string()))?;
    let result = evaluate_late_payment(&case.terms, &case.event)?;
    let recomputed = serde_json::to_value(&result)?;
    if recomputed != case.expected {
        return Err(LexCodexError::ResultDrift {
            case: case_id.to_string(),
            detail: format!("recomputed {recomputed} vs sealed {}", case.expected),
        });
    }
    let hash = computed_result_hash(&result)?;
    Ok(Recomputation {
        case,
        result,
        computed_result_hash: hash,
    })
}

/// Build the `related_slc_artifacts` reference for a verified, recomputed fixture case.
pub fn build_artifact_ref(
    bundle_dir: &Path,
    pinned_hash: &str,
    fixture_file: &str,
    case_id: &str,
) -> Result<(SlcArtifactRef, Recomputation), LexCodexError> {
    let artifact_hash = verify_pinned_bundle(bundle_dir, pinned_hash)?;
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(bundle_dir.join("manifest.json"))?)?;
    let recomputation = recompute_case(bundle_dir, fixture_file, case_id)?;
    let r = SlcArtifactRef {
        artifact_id: manifest
            .get("artifact_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        version: manifest
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        artifact_hash,
        fixture_case: case_id.to_string(),
        computed_result_hash: recomputation.computed_result_hash.clone(),
    };
    Ok((r, recomputation))
}

/// Confirm a presented `related_slc_artifacts` reference against a fresh verification and
/// recomputation of the pinned bundle (the consumer-side mirror of [`build_artifact_ref`]).
/// Confirms that the computed charge the attestation record relies on matches what the
/// sealed bundle actually computes.
pub fn confirm_artifact_ref(
    bundle_dir: &Path,
    fixture_file: &str,
    presented: &SlcArtifactRef,
) -> Result<Recomputation, LexCodexError> {
    let (fresh, recomputation) = build_artifact_ref(
        bundle_dir,
        &presented.artifact_hash,
        fixture_file,
        &presented.fixture_case,
    )?;
    if fresh != *presented {
        return Err(LexCodexError::RefMismatch(format!(
            "presented {presented:?} vs recomputed {fresh:?}"
        )));
    }
    Ok(recomputation)
}
