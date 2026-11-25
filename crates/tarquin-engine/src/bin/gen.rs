//! `tarquin-engine-gen` — generate (or drift-check) the backend artifacts from the Rule IR.
//!
//! Writes, relative to the workspace root:
//! - `primitives/src/generated/RuleEngineV01.sol`        (EN-O2, Solidity backend)
//! - `primitives/test/generated/RuleEngineParity.t.sol`  (EN-O4, corpus replay on-chain)
//! - `exports/engine/ruleEngine.ts`                      (EN-O3, AP-path TS backend)
//! - `exports/engine/ruleset.json`                       (EN-O6, the IR artifact itself)
//!
//! `--check` regenerates in memory and exits non-zero if any committed artifact differs —
//! the PR-003 "divergence = build failure" hook. Before generating anything, the golden
//! corpus is validated against the Rust evaluator, so a wrong expectation can never be
//! compiled into a target.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use tarquin_engine::codegen::{parse_corpus, sol, ts};
use tarquin_engine::{canonical_ruleset, evaluate, ruleset_hash};

fn workspace_root() -> PathBuf {
    // crates/tarquin-engine -> workspace root.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .to_path_buf()
}

fn main() -> anyhow::Result<ExitCode> {
    let check = std::env::args().any(|a| a == "--check");
    let root = workspace_root();
    let rules = canonical_ruleset();

    // The corpus must be true against the reference evaluator before it is embedded anywhere.
    let corpus_path = root.join("tests/parity/corpus.json");
    let corpus = parse_corpus(&std::fs::read_to_string(&corpus_path)?)?;
    for case in &corpus {
        let got = evaluate(&rules, &case.inputs);
        anyhow::ensure!(
            got == case.expected,
            "corpus case {:?} disagrees with the Rust evaluator: expected {:?}, engine says {:?}",
            case.name,
            case.expected,
            got
        );
    }

    let artifacts: Vec<(PathBuf, String)> = vec![
        (
            root.join("primitives/src/generated/RuleEngineV01.sol"),
            sol::generate(&rules)?,
        ),
        (
            root.join("primitives/test/generated/RuleEngineParity.t.sol"),
            sol::generate_parity_test(&rules, &corpus)?,
        ),
        (
            root.join("exports/engine/ruleEngine.ts"),
            ts::generate(&rules)?,
        ),
        (
            root.join("exports/engine/ruleset.json"),
            serde_json::to_string_pretty(&rules)? + "\n",
        ),
    ];

    let mut drift = false;
    for (path, content) in &artifacts {
        if check {
            match std::fs::read_to_string(path) {
                Ok(on_disk) if &on_disk == content => {
                    println!("ok      {}", rel(&root, path));
                }
                Ok(_) => {
                    eprintln!(
                        "DRIFT   {} (regenerate with tarquin-engine-gen)",
                        rel(&root, path)
                    );
                    drift = true;
                }
                Err(_) => {
                    eprintln!(
                        "MISSING {} (regenerate with tarquin-engine-gen)",
                        rel(&root, path)
                    );
                    drift = true;
                }
            }
        } else {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            std::fs::write(path, content)?;
            println!("wrote   {}", rel(&root, path));
        }
    }

    println!("ruleset_sha256: {}", ruleset_hash(&rules));
    if drift {
        eprintln!("generated artifacts have drifted from the Rule IR — build failure (PR-003)");
        return Ok(ExitCode::FAILURE);
    }
    Ok(ExitCode::SUCCESS)
}

fn rel<'a>(root: &Path, path: &'a Path) -> std::borrow::Cow<'a, str> {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy()
}
