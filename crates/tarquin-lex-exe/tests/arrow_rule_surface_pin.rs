//! Arrow rule-surface pin conformance.
//!
//! Tarquin consumes Arrow rule IDs and registry entries by pinning Arrow's
//! producer-owned `exports/rule-surface.v1.6.0.json`. This test makes the pin
//! loud: if Arrow changes the surface without an intentional Tarquin pin bump,
//! the workspace fails before prose can drift.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

fn tarquin_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn arrow_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("Arrow")
}

fn git_show(repo: &Path, revision_path: &str) -> Vec<u8> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("show")
        .arg(revision_path)
        .output()
        .expect("run git show for pinned Arrow artifact");
    assert!(
        output.status.success(),
        "git show {revision_path} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

#[test]
fn arrow_rule_surface_pin_matches_export_hash() {
    if !arrow_root().join(".git").exists() {
        eprintln!("skipping: sibling Arrow checkout not present at {}", arrow_root().display());
        return;
    }
    let compatibility_path = tarquin_root().join("arrow-compatibility.json");
    let compatibility_json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&compatibility_path).expect("read arrow-compatibility.json"),
    )
    .expect("arrow-compatibility.json parses");
    assert_eq!(compatibility_json["repository"], "Tarquin");
    assert_eq!(
        compatibility_json["arrow_compatibility"]["baseline"], "arrow-v1.6.0",
        "Tarquin must declare Arrow compatibility separately from Tarquin's own versions"
    );
    assert_eq!(
        compatibility_json["arrow_compatibility"]["pin_manifest"],
        "imports/arrow/PIN.json"
    );

    let pin_path = tarquin_root().join("imports/arrow/PIN.json");
    let pin_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&pin_path).expect("read imports/arrow/PIN.json"))
            .expect("PIN.json parses");

    assert_eq!(
        pin_json["arrow_compatibility"]["baseline"], "arrow-v1.6.0",
        "Tarquin must declare Arrow compatibility separately from Tarquin's own versions"
    );

    let pin = pin_json["pins"]
        .as_array()
        .and_then(|pins| {
            pins.iter()
                .find(|pin| pin["artifact"] == "arrow-rule-surface")
        })
        .expect("arrow-rule-surface pin exists");
    assert_eq!(pin["version"], "1.6.0");
    assert_eq!(pin["baseline"], "arrow-v1.6.0");

    let source_path = pin["source_path"].as_str().expect("pin source_path");
    let source_commit = pin["source_commit"].as_str().expect("pin source_commit");
    let expected = pin["hash"]
        .as_str()
        .expect("pin hash")
        .strip_prefix("sha256:")
        .expect("sha256-prefixed hash");
    let bytes = fs::read(arrow_root().join(source_path)).expect("read Arrow rule surface export");
    let actual = format!("{:x}", Sha256::digest(&bytes));

    assert_eq!(
        actual, expected,
        "Arrow rule-surface hash drifted; update imports/arrow/PIN.json intentionally"
    );

    let pinned_revision_path = format!("{source_commit}:{source_path}");
    let pinned_bytes = git_show(&arrow_root(), &pinned_revision_path);
    let pinned_actual = format!("{:x}", Sha256::digest(&pinned_bytes));

    assert_eq!(
        pinned_actual, expected,
        "Arrow rule-surface source_commit does not contain the pinned hash"
    );
}
