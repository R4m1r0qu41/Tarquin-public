//! `sibyl` — the runnable Tier-1 Sibyl driver (Phase 3 vertical slice as a CLI).
//!
//! The library crates stay deterministic and clock-free; this binary is the I/O shell that
//! supplies them with bytes, keys, and time. Subcommands mirror the trust chain of
//! `drafts/sibyl/sibyl-legal-oracle.md` §4:
//!
//! ```text
//! sibyl keygen   --out <key.json> [--seed <64-hex>]
//! sibyl attest   --source <docket.json> --key <key.json>
//!                [--registry <registry.json>] [--sibyl-did <did>] [--lbry-id <id>]
//!                [--now <iso8601>] [--out <lbry.json>]
//! sibyl emit     --lbry <lbry.json> --registry <registry.json>
//!                [--now <iso8601>] [--out <event.json>]
//! sibyl slice    --source <docket.json> [--out-dir <dir>] [--now <iso8601>]
//! ```
//!
//! `slice` runs the whole Phase-3 vertical — source → EXTRACT → ANCHOR → SIGN → LBRY →
//! OA-* preflight → LEG event → signed envelope → fail-closed consumer → HoldForReview —
//! and writes every artifact to the out dir. Nothing executes autonomously (ADR-0001).

use std::path::Path;
use std::process::ExitCode;

use base64::Engine as _;
use chrono::{SecondsFormat, Utc};

use arrow_adapter::ConsumerExecutionMode;
use arrow_identity::crypto::ed25519::{generate_keypair_from_seed, generate_test_keypair};
use arrow_identity::resolver::did_key::did_key_from_public;
use arrow_origin_leg::build_signed_envelope;

use tarquin_lex_exe::consume::{InMemoryReplayStore, InMemoryReviewQueue};
use tarquin_lex_exe::{consume_envelope, emit_leg_event, preflight, ConsumeConfig};
use tarquin_sibyl::pipeline::{attest, extract, AttestParams};
use tarquin_sibyl::registry::{
    AttestationScope, AuthorityProfile, AuthorityType, Confidence, Key, KeyAlgorithm, KeyPurpose,
    KeyStatus, RegistryMetadata, SibylEntry, SibylStatus, SourceAuthorityBasis, VerificationRegime,
};
use tarquin_sibyl::validate::{has_fatal, Finding, Severity};
use tarquin_sibyl::{LbryRecord, SibylRegistry};

const B64: base64::engine::general_purpose::GeneralPurpose =
    base64::engine::general_purpose::STANDARD_NO_PAD;

/// Consumer identity for the demo slice (the envelope's `receiver_id`).
const SLICE_RECEIVER: &str = "did:web:consumer.tarquin.example";

const USAGE: &str = "\
sibyl — Tier-1 Sibyl driver (Tarquin Phase 3 vertical slice)

USAGE:
  sibyl keygen   --out <key.json> [--seed <64-hex>]
  sibyl attest   --source <docket.json> --key <key.json>
                 [--registry <registry.json>] [--sibyl-did <did>] [--lbry-id <id>]
                 [--now <iso8601>] [--out <lbry.json>]
  sibyl emit     --lbry <lbry.json> --registry <registry.json>
                 [--now <iso8601>] [--out <event.json>]
  sibyl slice    --source <docket.json> [--out-dir <dir>] [--now <iso8601>]

`--now` defaults to the current UTC time; pass it explicitly for reproducible runs.
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    };
    let result = match cmd.as_str() {
        "keygen" => cmd_keygen(&args[1..]),
        "attest" => cmd_attest(&args[1..]),
        "emit" => cmd_emit(&args[1..]),
        "slice" => cmd_slice(&args[1..]),
        "-h" | "--help" | "help" => {
            print!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        other => Err(anyhow::anyhow!("unknown subcommand {other:?}\n{USAGE}")),
    };
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::from(2)
        }
    }
}

// ── subcommands ────────────────────────────────────────────────────────────────

fn cmd_keygen(args: &[String]) -> anyhow::Result<ExitCode> {
    let out = require_flag(args, "--out")?;
    let (private, public) = match flag(args, "--seed") {
        Some(hex) => generate_keypair_from_seed(parse_seed(&hex)?),
        None => generate_test_keypair(),
    };
    let key = key_file_json(&private, &public);
    write_json(Path::new(&out), &key)?;
    println!("did:      {}", key["did"].as_str().unwrap());
    println!("key_ref:  {}", key["key_ref"].as_str().unwrap());
    println!("key file: {out}");
    Ok(ExitCode::SUCCESS)
}

fn cmd_attest(args: &[String]) -> anyhow::Result<ExitCode> {
    let source_path = require_flag(args, "--source")?;
    let key_path = require_flag(args, "--key")?;
    let now = flag(args, "--now").unwrap_or_else(now_utc);

    let source_bytes = std::fs::read(&source_path)?;
    let key = read_key_file(Path::new(&key_path))?;
    let params = AttestParams {
        sibyl_did: flag(args, "--sibyl-did").unwrap_or_else(|| key.did.clone()),
        key_ref: key.key_ref.clone(),
        attested_at: now.clone(),
        lbry_id: flag(args, "--lbry-id"),
    };
    let record = attest(&source_bytes, &params, &key.private)?;
    println!(
        "LBRY record {} attested_by {}",
        record.lbry_id, record.attested_by
    );
    println!("  source_ref:   {}", record.source_ref);
    println!("  source_hash:  {}", record.source_hash);
    println!("  content_hash: {}", record.content_hash);

    let mut code = ExitCode::SUCCESS;
    if let Some(reg_path) = flag(args, "--registry") {
        let registry = SibylRegistry::from_json(&std::fs::read_to_string(reg_path)?)?;
        let findings = preflight(&record, &registry, &now);
        print_findings(&findings);
        if has_fatal(&findings) {
            eprintln!("attestation is NOT emittable (fatal findings above)");
            code = ExitCode::FAILURE;
        }
    }
    write_json_or_stdout(flag(args, "--out"), &record)?;
    Ok(code)
}

fn cmd_emit(args: &[String]) -> anyhow::Result<ExitCode> {
    let record = LbryRecord::from_json(&std::fs::read_to_string(require_flag(args, "--lbry")?)?)?;
    let registry =
        SibylRegistry::from_json(&std::fs::read_to_string(require_flag(args, "--registry")?)?)?;
    let now = flag(args, "--now").unwrap_or_else(now_utc);

    match emit_leg_event(&record, &registry, &now) {
        Ok(event) => {
            println!("LEG event: {}", event.arrow_code);
            write_json_or_stdout(flag(args, "--out"), &event)?;
            Ok(ExitCode::SUCCESS)
        }
        Err(blocked) => {
            print_findings(&blocked.findings);
            eprintln!("emission blocked");
            Ok(ExitCode::FAILURE)
        }
    }
}

/// The whole Phase-3 vertical slice in one run, artifacts written per stage.
fn cmd_slice(args: &[String]) -> anyhow::Result<ExitCode> {
    let source_path = require_flag(args, "--source")?;
    let out_dir = flag(args, "--out-dir").unwrap_or_else(|| "exports/sibyl-demo".to_string());
    let out = Path::new(&out_dir);
    std::fs::create_dir_all(out)?;
    let now = flag(args, "--now").unwrap_or_else(now_utc);
    let source_bytes = std::fs::read(&source_path)?;

    // 1. Identity — one attestation key for the Sibyl, one transport key for Lex.exe.
    let (sibyl_priv, sibyl_pub) = generate_test_keypair();
    let sibyl_key = key_file_json(&sibyl_priv, &sibyl_pub);
    let sibyl_did = sibyl_key["did"].as_str().unwrap().to_string();
    let sibyl_key_ref = sibyl_key["key_ref"].as_str().unwrap().to_string();
    write_json(&out.join("sibyl-key.json"), &sibyl_key)?;

    let (_transport_priv, transport_pub) = {
        let (p, q) = generate_test_keypair();
        let f = key_file_json(&p, &q);
        write_json(&out.join("transport-key.json"), &f)?;
        (p, q)
    };
    let transport_did = did_key_from_public(&transport_pub);
    println!("[1/6] keys      sibyl {sibyl_did}");
    println!("                lexexe {transport_did}");

    // 2. Registry — one active Tier-1 entry scoped to exactly what the source covers.
    let (doc, _fact) = extract(&source_bytes)?;
    let registry = SibylRegistry {
        registry_metadata: RegistryMetadata {
            schema_version: "0.1".to_string(),
            registry_name: "sibyl_registry".to_string(),
            updated_at: now.clone(),
            notes: Some("Generated by `sibyl slice` — demo data, not production authority.".into()),
        },
        entries: vec![SibylEntry {
            sibyl_id: "SBYL-SLICE-001".to_string(),
            did: sibyl_did.clone(),
            name: format!("Slice demo monitor for {}", doc.court),
            authority_profile: AuthorityProfile {
                authority_type: AuthorityType::CourtMonitor,
                recognized_authority_ref: "RFC007B-SLICE-DEMO-001".to_string(),
                jurisdictions: std::iter::once(doc.jurisdiction_country.clone())
                    .chain(doc.jurisdiction_supra.clone())
                    .collect(),
                legal_system_refs: doc.legal_system_refs.clone(),
                source_authority_basis: vec![
                    SourceAuthorityBasis::OfficialDocket,
                    SourceAuthorityBasis::CourtOrder,
                ],
            },
            attestation_scope: AttestationScope {
                leg_subtypes: vec![doc.order.kind],
                domains: vec![],
                max_confidence: Confidence::C,
                verification_regime: VerificationRegime::StructuredLegalAttestation,
            },
            keys: vec![Key {
                key_id: sibyl_key_ref.clone(),
                purpose: KeyPurpose::Attestation,
                algorithm: KeyAlgorithm::Ed25519,
                status: KeyStatus::Active,
                valid_from: now.clone(),
                valid_to: None,
            }],
            status: SibylStatus::Active,
            added_at: now.clone(),
            updated_at: None,
            notes: None,
        }],
    };
    write_json(&out.join("sibyl-registry.json"), &registry)?;
    println!(
        "[2/6] registry  SBYL-SLICE-001 active ({})",
        doc.jurisdiction_country
    );

    // 3. Sibyl pipeline: EXTRACT → ANCHOR → SIGN → EMIT (the LBRY record).
    let params = AttestParams {
        sibyl_did: sibyl_did.clone(),
        key_ref: sibyl_key_ref,
        attested_at: now.clone(),
        lbry_id: None,
    };
    let record = attest(&source_bytes, &params, &sibyl_priv)?;
    anyhow::ensure!(
        record.verify_signature_raw(&sibyl_pub)?,
        "freshly signed LBRY record must verify"
    );
    write_json(&out.join("lbry-record.json"), &record)?;
    println!(
        "[3/6] lbry      {} ({})",
        record.lbry_id, record.source_hash
    );

    // 4. Lex.exe origin: OA-* preflight, then a protocol-valid LEG event.
    let findings = preflight(&record, &registry, &now);
    print_findings(&findings);
    let event = match emit_leg_event(&record, &registry, &now) {
        Ok(e) => e,
        Err(blocked) => {
            print_findings(&blocked.findings);
            eprintln!("emission blocked — slice stops here (fail closed)");
            return Ok(ExitCode::FAILURE);
        }
    };
    write_json(&out.join("leg-event.json"), &event)?;
    println!("[4/6] emit      {}", event.arrow_code);

    // 5. Transport: sign the envelope with the Lex.exe transport key.
    let envelope = build_signed_envelope(
        event.clone(),
        &transport_did,
        SLICE_RECEIVER,
        Some(&out.join("transport-key.json")),
    )?;
    write_json(&out.join("envelope.json"), &envelope)?;
    println!(
        "[5/6] envelope  {} -> {SLICE_RECEIVER}",
        envelope.envelope_id
    );

    // 6. Fail-closed consumer: verify, reduce, hold for a human. The queue IS the action.
    let cfg = ConsumeConfig {
        allowed_senders: vec![transport_did],
        receiver_id: SLICE_RECEIVER.to_string(),
    };
    let replay = InMemoryReplayStore::new(chrono::Duration::hours(24));
    let queue = InMemoryReviewQueue::new();
    let outcome = consume_envelope(
        &envelope,
        &cfg,
        &replay,
        &queue,
        ConsumerExecutionMode::Execute,
    )?;
    println!(
        "[6/6] consume   decision {:?} (queued: {})",
        outcome.decision, outcome.queued
    );
    if let Some(item) = &outcome.review_item {
        write_json(&out.join("review-item.json"), item)?;
    }

    println!("\nartifacts in {out_dir}/");
    println!("slice complete: legal fact is held for human review; nothing executed.");
    Ok(ExitCode::SUCCESS)
}

// ── key files ──────────────────────────────────────────────────────────────────

struct KeyFile {
    did: String,
    key_ref: String,
    private: [u8; 32],
}

/// Key-file JSON in the format `arrow-origin-leg::build_signed_envelope` consumes.
fn key_file_json(private: &[u8; 32], public: &[u8; 32]) -> serde_json::Value {
    let did = did_key_from_public(public);
    let frag = did.strip_prefix("did:key:").expect("did:key prefix");
    serde_json::json!({
        "private_key_b64": B64.encode(private),
        "key_ref": format!("{did}#{frag}"),
        "did": did,
    })
}

fn read_key_file(path: &Path) -> anyhow::Result<KeyFile> {
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    let field = |name: &str| -> anyhow::Result<String> {
        Ok(v.get(name)
            .and_then(|x| x.as_str())
            .ok_or_else(|| anyhow::anyhow!("key file {path:?} missing {name:?}"))?
            .to_string())
    };
    let private: [u8; 32] = B64
        .decode(field("private_key_b64")?)?
        .try_into()
        .map_err(|_| anyhow::anyhow!("private key must be 32 bytes"))?;
    Ok(KeyFile {
        did: field("did")?,
        key_ref: field("key_ref")?,
        private,
    })
}

fn parse_seed(hex: &str) -> anyhow::Result<[u8; 32]> {
    anyhow::ensure!(hex.len() == 64, "--seed must be 64 hex chars (32 bytes)");
    let mut seed = [0u8; 32];
    for (i, byte) in seed.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16)
            .map_err(|e| anyhow::anyhow!("--seed is not hex: {e}"))?;
    }
    Ok(seed)
}

// ── small CLI plumbing ─────────────────────────────────────────────────────────

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn require_flag(args: &[String], name: &str) -> anyhow::Result<String> {
    flag(args, name).ok_or_else(|| anyhow::anyhow!("missing required flag {name} <value>"))
}

fn now_utc() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn print_findings(findings: &[Finding]) {
    for f in findings {
        let sev = match f.severity {
            Severity::Fatal => "FATAL",
            Severity::Warning => "warn ",
        };
        eprintln!("  [{sev}] {} {}", f.rule, f.message);
    }
}

fn write_json(path: &Path, value: &impl serde::Serialize) -> anyhow::Result<()> {
    std::fs::write(path, serde_json::to_string_pretty(value)? + "\n")?;
    Ok(())
}

fn write_json_or_stdout(out: Option<String>, value: &impl serde::Serialize) -> anyhow::Result<()> {
    match out {
        Some(path) => write_json(Path::new(&path), value),
        None => {
            println!("{}", serde_json::to_string_pretty(value)?);
            Ok(())
        }
    }
}
