//! The parity harness, Rust side (PR-003 / EN-O4).
//!
//! 1. The golden corpus must hold against the reference evaluator.
//! 2. The committed generated artifacts must match a fresh regeneration from the IR — so
//!    `cargo test` alone fails on drift, before forge or node ever run.

use tarquin_engine::codegen::{parse_corpus, sol, ts};
use tarquin_engine::{canonical_ruleset, evaluate};

const CORPUS: &str = include_str!("../../../tests/parity/corpus.json");
const GEN_SOL: &str = include_str!("../../../primitives/src/generated/RuleEngineV01.sol");
const GEN_SOL_TEST: &str =
    include_str!("../../../primitives/test/generated/RuleEngineParity.t.sol");
const GEN_TS: &str = include_str!("../../../exports/engine/ruleEngine.ts");
const GEN_IR: &str = include_str!("../../../exports/engine/ruleset.json");

#[test]
fn golden_corpus_holds_on_reference_evaluator() {
    let rules = canonical_ruleset();
    let corpus = parse_corpus(CORPUS).expect("corpus parses");
    assert!(corpus.len() >= 10, "corpus is suspiciously small");
    for case in &corpus {
        let got = evaluate(&rules, &case.inputs);
        assert_eq!(
            got, case.expected,
            "corpus case {:?} diverges on the Rust evaluator",
            case.name
        );
    }
}

#[test]
fn committed_artifacts_match_regenerated_ir() {
    let rules = canonical_ruleset();
    let corpus = parse_corpus(CORPUS).unwrap();
    assert_eq!(
        GEN_SOL,
        sol::generate(&rules).unwrap(),
        "RuleEngineV01.sol drifted — run tarquin-engine-gen"
    );
    assert_eq!(
        GEN_SOL_TEST,
        sol::generate_parity_test(&rules, &corpus).unwrap(),
        "RuleEngineParity.t.sol drifted — run tarquin-engine-gen"
    );
    assert_eq!(
        GEN_TS,
        ts::generate(&rules).unwrap(),
        "ruleEngine.ts drifted — run tarquin-engine-gen"
    );
    assert_eq!(
        GEN_IR,
        serde_json::to_string_pretty(&rules).unwrap() + "\n",
        "ruleset.json drifted — run tarquin-engine-gen"
    );
}
