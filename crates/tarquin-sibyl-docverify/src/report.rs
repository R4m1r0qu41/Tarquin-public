//! Canonical JSON and Markdown report generation from a `DocumentVerificationFinding`.
//!
//! Canonical JSON: serialized with alphabetically ordered keys (via `serde_json::Value` +
//! `BTreeMap` conversion). Repeated calls with identical inputs produce byte-identical output.
//! Run timestamps are NOT embedded — the caller controls what context appears around the report.

use std::collections::BTreeMap;

use tarquin_sibyl::docverify::{DocumentVerificationFinding, SemanticState};

/// Canonical JSON representation of a `DocumentVerificationFinding`. Key order is stable
/// (alphabetical) so repeated identical inputs produce byte-identical output.
pub fn to_canonical_json(f: &DocumentVerificationFinding) -> Result<String, String> {
    let mut map = BTreeMap::new();
    map.insert(
        "artifact_sha256",
        serde_json::Value::String(f.artifact_sha256.clone()),
    );
    map.insert(
        "profile_id",
        serde_json::Value::String(f.profile_id.clone()),
    );
    map.insert(
        "profile_hash",
        serde_json::Value::String(f.profile_hash.clone()),
    );
    map.insert(
        "evidence_manifest_hash",
        serde_json::Value::String(f.evidence_manifest_hash.clone()),
    );
    map.insert(
        "checked_at",
        serde_json::Value::String(f.checked_at.clone()),
    );
    map.insert(
        "document_integrity",
        serde_json::Value::String(state_str(f.document_integrity)),
    );
    map.insert(
        "issuer_identity",
        serde_json::Value::String(state_str(f.issuer_identity)),
    );
    map.insert(
        "signer_identity",
        serde_json::Value::String(state_str(f.signer_identity)),
    );
    map.insert(
        "signer_capacity",
        serde_json::Value::String(state_str(f.signer_capacity)),
    );
    map.insert(
        "registry_status",
        serde_json::Value::String(state_str(f.registry_status)),
    );
    map.insert(
        "filing_status",
        serde_json::Value::String(state_str(f.filing_status)),
    );
    map.insert(
        "document_authenticity",
        serde_json::Value::String(state_str(f.document_authenticity)),
    );
    map.insert(
        "legal_effect",
        serde_json::Value::String(state_str(f.legal_effect)),
    );
    map.insert(
        "requires_human_review",
        serde_json::Value::Bool(f.requires_human_review()),
    );
    let warnings: Vec<serde_json::Value> = f
        .warnings
        .iter()
        .map(|w| serde_json::Value::String(w.clone()))
        .collect();
    map.insert("warnings", serde_json::Value::Array(warnings));

    serde_json::to_string(&map).map_err(|e| e.to_string())
}

/// Human-readable Markdown summary of a `DocumentVerificationFinding`.
pub fn to_markdown(f: &DocumentVerificationFinding) -> String {
    let review_flag = if f.requires_human_review() {
        "**Yes — human legal review required**"
    } else {
        "No"
    };

    let warnings_section = if f.warnings.is_empty() {
        String::from("None")
    } else {
        f.warnings
            .iter()
            .map(|w| format!("- {w}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "# Document Verification Finding\n\
         \n\
         | Field | Value |\n\
         |---|---|\n\
         | Artifact SHA-256 | `{sha256}` |\n\
         | Profile | `{profile_id}` |\n\
         | Checked at | `{checked_at}` |\n\
         | Requires human review | {review_flag} |\n\
         \n\
         ## Dimensions\n\
         \n\
         | Dimension | State |\n\
         |---|---|\n\
         | document_integrity | `{di}` |\n\
         | issuer_identity | `{ii}` |\n\
         | signer_identity | `{si}` |\n\
         | signer_capacity | `{sc}` |\n\
         | registry_status | `{rs}` |\n\
         | filing_status | `{fs}` |\n\
         | document_authenticity | `{da}` |\n\
         | legal_effect | `{le}` |\n\
         \n\
         ## Warnings\n\
         \n\
         {warnings}\n",
        sha256 = f.artifact_sha256,
        profile_id = f.profile_id,
        checked_at = f.checked_at,
        di = state_label(f.document_integrity),
        ii = state_label(f.issuer_identity),
        si = state_label(f.signer_identity),
        sc = state_label(f.signer_capacity),
        rs = state_label(f.registry_status),
        fs = state_label(f.filing_status),
        da = state_label(f.document_authenticity),
        le = state_label(f.legal_effect),
        warnings = warnings_section,
    )
}

fn state_str(s: SemanticState) -> String {
    state_label(s).to_string()
}

fn state_label(s: SemanticState) -> &'static str {
    match s {
        SemanticState::NotDetermined => "not_determined",
        SemanticState::Refuted => "refuted",
        SemanticState::PartiallyConfirmed => "partially_confirmed",
        SemanticState::Confirmed => "confirmed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tarquin_sibyl::docverify::SemanticState;

    fn digest(nibble: char) -> String {
        format!("sha256:{}", nibble.to_string().repeat(64))
    }

    fn synthetic_finding() -> DocumentVerificationFinding {
        DocumentVerificationFinding {
            artifact_sha256: digest('a'),
            profile_id: "MX-JUZGADO-OFICIO-01".to_string(),
            profile_hash: digest('b'),
            evidence_manifest_hash: digest('c'),
            checked_at: "2026-06-27T00:00:00Z".to_string(),
            document_integrity: SemanticState::PartiallyConfirmed,
            issuer_identity: SemanticState::NotDetermined,
            signer_identity: SemanticState::NotDetermined,
            signer_capacity: SemanticState::NotDetermined,
            registry_status: SemanticState::NotDetermined,
            filing_status: SemanticState::NotDetermined,
            document_authenticity: SemanticState::NotDetermined,
            legal_effect: SemanticState::NotDetermined,
            warnings: vec!["document_integrity capped to partially_confirmed: artifact hash alone is not tamper-evidence (needs signature validation)".to_string()],
        }
    }

    #[test]
    fn canonical_json_is_stable() {
        let f = synthetic_finding();
        let j1 = to_canonical_json(&f).unwrap();
        let j2 = to_canonical_json(&f).unwrap();
        assert_eq!(j1, j2);
    }

    #[test]
    fn canonical_json_contains_all_dimensions() {
        let json = to_canonical_json(&synthetic_finding()).unwrap();
        for dim in &[
            "document_integrity",
            "issuer_identity",
            "signer_identity",
            "signer_capacity",
            "registry_status",
            "filing_status",
            "document_authenticity",
            "legal_effect",
        ] {
            assert!(json.contains(dim), "missing dimension {dim} in JSON");
        }
    }

    #[test]
    fn canonical_json_keys_are_sorted() {
        let json = to_canonical_json(&synthetic_finding()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let keys: Vec<_> = v.as_object().unwrap().keys().collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted, "JSON keys must be in alphabetical order");
    }

    #[test]
    fn requires_human_review_is_true_for_not_determined_legal_effect() {
        let json = to_canonical_json(&synthetic_finding()).unwrap();
        assert!(json.contains(r#""requires_human_review":true"#));
    }

    #[test]
    fn markdown_contains_profile_and_artifact() {
        let md = to_markdown(&synthetic_finding());
        assert!(md.contains("MX-JUZGADO-OFICIO-01"));
        assert!(md.contains(&digest('a')));
        assert!(md.contains("partially_confirmed"));
    }

    #[test]
    fn markdown_flags_human_review() {
        let md = to_markdown(&synthetic_finding());
        assert!(md.contains("human legal review required"));
    }
}
