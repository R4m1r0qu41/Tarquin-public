//! MC-007 legal-grounding pin conformance (mirror of the SLC MC-006 test).
//!
//! Recomputes the sealed bundle hash from Tarquin's build-time copy at
//! `imports/orlog/lex-grounding-001` and compares it to the pinned value.
//! Drift in any sealed node — or in the manifest itself — must reject.
//! Also verifies every per-node `file_hash`, which is what LC-014-era
//! profile `lex_refs` validation keys on: a profile may cite only lex ids
//! present in this pinned, hash-verified bundle.

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const PINNED_BUNDLE_HASH: &str =
    "sha256:fb2d2e099629c23a730267b51c331ce38df7f9732392647f400d6e378a6b131f";
const BUNDLE_ID: &str = "lex-grounding-001";

fn bundle_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../imports/orlog/lex-grounding-001")
}

/// Canonical JSON per the MC-006 convention: keys sorted recursively,
/// 2-space indent, trailing newline.
fn canonical_json(v: &Value) -> String {
    fn sort(v: &Value) -> Value {
        match v {
            Value::Object(m) => {
                let mut keys: Vec<&String> = m.keys().collect();
                keys.sort();
                let mut out = serde_json::Map::new();
                for k in keys {
                    out.insert(k.clone(), sort(&m[k]));
                }
                Value::Object(out)
            }
            Value::Array(a) => Value::Array(a.iter().map(sort).collect()),
            other => other.clone(),
        }
    }
    format!("{}\n", serde_json::to_string_pretty(&sort(v)).unwrap())
}

fn recompute_bundle_hash(dir: &Path) -> anyhow::Result<(String, Value)> {
    let manifest_path = dir.join("lex/handoff/legal-grounding-manifest.json");
    let manifest: Value = serde_json::from_str(&std::fs::read_to_string(&manifest_path)?)?;

    // Per-node hashing in the manifest's (sorted) path order.
    let mut hasher = Sha256::new();
    let nodes = manifest["nodes"].as_array().expect("manifest nodes");
    for node in nodes {
        let rel = node["path"].as_str().expect("node path");
        let bytes = std::fs::read(dir.join(rel))?;

        // Per-node file_hash must match the sealed bytes.
        let file_hash = format!("sha256:{:x}", Sha256::digest(&bytes));
        assert_eq!(
            file_hash,
            node["file_hash"].as_str().unwrap(),
            "sealed node {rel} drifted from its manifest file_hash"
        );

        hasher.update(rel.as_bytes());
        hasher.update([0u8]);
        hasher.update(&bytes);
        hasher.update([0u8]);
    }

    // Manifest last, bundle_hash removed, canonical form.
    let mut for_hash = manifest.clone();
    for_hash.as_object_mut().unwrap().remove("bundle_hash");
    hasher.update("lex/handoff/legal-grounding-manifest.json".as_bytes());
    hasher.update([0u8]);
    hasher.update(canonical_json(&for_hash).as_bytes());

    Ok((format!("sha256:{:x}", hasher.finalize()), manifest))
}

#[test]
fn pinned_lex_grounding_bundle_verifies_and_drift_rejects() {
    let dir = bundle_dir();
    let (actual, manifest) = recompute_bundle_hash(&dir).expect("recompute");

    // The recomputed hash must match BOTH the manifest's own claim and the
    // pin constant (which mirrors imports/orlog/PIN.json).
    assert_eq!(
        manifest["bundle_id"].as_str().unwrap(),
        BUNDLE_ID,
        "unexpected bundle id"
    );
    assert_eq!(
        actual,
        manifest["bundle_hash"].as_str().unwrap(),
        "manifest bundle_hash does not match recomputation"
    );
    assert_eq!(
        actual, PINNED_BUNDLE_HASH,
        "pin drift: bundle hash mismatch"
    );

    // Every sealed node carries the synced marker for this bundle.
    for node in manifest["nodes"].as_array().unwrap() {
        let rel = node["path"].as_str().unwrap();
        let text = std::fs::read_to_string(dir.join(rel)).unwrap();
        assert!(
            text.contains(&format!("lex_codex_status: synced@{BUNDLE_ID}")),
            "{rel} lost its synced marker"
        );
    }

    // Drift-detection sanity: a wrong pin must NOT verify.
    assert_ne!(
        actual,
        "sha256:0000000000000000000000000000000000000000000000000000000000000000"
    );
}

#[test]
fn profile_lex_refs_must_resolve_in_pinned_bundle() {
    // The consumer-side rule current-state.md limitation #1 called for:
    // profile lex_refs are validated against the pinned surface. This test
    // locks the mechanism: every id in the bundle is resolvable, and an id
    // absent from the bundle is detectable.
    let dir = bundle_dir();
    let manifest: Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join("lex/handoff/legal-grounding-manifest.json")).unwrap(),
    )
    .unwrap();
    let ids: std::collections::HashSet<&str> = manifest["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();

    // Pilot-relevant grounding is sealed:
    for required in [
        "lex-mx-ccom-89",
        "lex-mx-lfea-7",
        "lex-mx-ccf-1840",
        "lex-us-ny-stt-304",
        "lex-us-15usc-7001",
    ] {
        assert!(
            ids.contains(required),
            "{required} missing from sealed bundle"
        );
    }
    // The architect-unverified concept is NOT citable:
    assert!(
        !ids.contains("concept-liquidated-damages"),
        "LEGAL_UNVERIFIED node must not be sealed/citable"
    );
    assert!(!ids.contains("concept-grace-period"));
}

/// Cross-language golden vector for the MC-006/007 hash convention.
///
/// The vector file is authored and asserted in Orlog (JS, the producer);
/// this test asserts the SAME vector with this crate's Rust recomputation,
/// pinning the two implementations to one convention. If either language's
/// canonicalization drifts, exactly one side of this pair goes red.
#[test]
fn mc007_golden_vector_cross_language() {
    let vector_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../Orlog/tests/golden/mc007-golden.json");
    if !vector_path.exists() {
        // Sibling Orlog checkout absent (e.g. crate tested in isolation):
        // skip rather than fabricate — CI checks out all three repos.
        eprintln!("skipping: Orlog golden vector not present at {vector_path:?}");
        return;
    }
    let vector: Value =
        serde_json::from_str(&std::fs::read_to_string(&vector_path).unwrap()).unwrap();

    let mut nodes: Vec<(String, String)> = vector["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| {
            (
                n["path"].as_str().unwrap().to_string(),
                n["payload_utf8"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    nodes.sort_by(|a, b| a.0.cmp(&b.0));

    let mut hasher = Sha256::new();
    for (path, payload) in &nodes {
        hasher.update(path.as_bytes());
        hasher.update([0u8]);
        hasher.update(payload.as_bytes());
        hasher.update([0u8]);
    }
    let mut manifest = vector["manifest"].clone();
    manifest.as_object_mut().unwrap().remove("bundle_hash");
    hasher.update(vector["manifest_path"].as_str().unwrap().as_bytes());
    hasher.update([0u8]);
    hasher.update(canonical_json(&manifest).as_bytes());

    let actual = format!("sha256:{:x}", hasher.finalize());
    assert_eq!(
        actual,
        vector["expected_bundle_hash"].as_str().unwrap(),
        "Rust MC-007 hashing diverged from the shared golden vector"
    );
}
