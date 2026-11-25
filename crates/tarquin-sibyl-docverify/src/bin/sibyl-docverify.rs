//! `sibyl-docverify` CLI — document-verification extraction engine.
//!
//! Usage:
//!   sibyl-docverify verify <artifact> --checked-at <ISO8601> [--output json|markdown|both]
//!                          [--naat-command <path>]
//!
//! `--checked-at` is required and must be supplied explicitly; the binary never calls the system
//! clock to populate canonical fields.

use std::path::PathBuf;
use std::process;

use tarquin_sibyl_docverify::{verify, NaatConfig, SystemRunner};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let parsed = match parse_args(&args[1..]) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            eprintln!("{}", usage());
            process::exit(2);
        }
    };

    let runner = SystemRunner;
    let naat_config = match parsed.naat_command {
        Some(ref cmd) => NaatConfig {
            command: Some(PathBuf::from(cmd)),
            interchange_version: "1.0".to_string(),
        },
        None => NaatConfig::disabled(),
    };

    match verify(&parsed.artifact, &parsed.checked_at, &runner, &naat_config) {
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
        Ok(output) => {
            match parsed.output_format.as_str() {
                "json" => println!("{}", output.canonical_json),
                "markdown" => print!("{}", output.markdown),
                _ => {
                    // "both"
                    println!("{}", output.canonical_json);
                    eprintln!("---");
                    eprint!("{}", output.markdown);
                }
            }
        }
    }
}

struct ParsedArgs {
    artifact: PathBuf,
    checked_at: String,
    output_format: String,
    naat_command: Option<String>,
}

fn parse_args(args: &[String]) -> Result<ParsedArgs, String> {
    let mut artifact: Option<PathBuf> = None;
    let mut checked_at: Option<String> = None;
    let mut output_format = "json".to_string();
    let mut naat_command: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "verify" => {
                i += 1;
                if i >= args.len() {
                    return Err("expected artifact path after 'verify'".into());
                }
                artifact = Some(PathBuf::from(&args[i]));
            }
            "--checked-at" => {
                i += 1;
                if i >= args.len() {
                    return Err("expected ISO 8601 timestamp after --checked-at".into());
                }
                checked_at = Some(args[i].clone());
            }
            "--output" => {
                i += 1;
                if i >= args.len() {
                    return Err("expected format after --output".into());
                }
                let fmt = args[i].as_str();
                if !matches!(fmt, "json" | "markdown" | "both") {
                    return Err(format!(
                        "unknown output format '{fmt}'; use json, markdown, or both"
                    ));
                }
                output_format = fmt.to_string();
            }
            "--naat-command" => {
                i += 1;
                if i >= args.len() {
                    return Err("expected path after --naat-command".into());
                }
                naat_command = Some(args[i].clone());
            }
            other => return Err(format!("unknown argument '{other}'")),
        }
        i += 1;
    }

    Ok(ParsedArgs {
        artifact: artifact.ok_or("'verify <artifact>' is required")?,
        checked_at: checked_at.ok_or("--checked-at <ISO8601> is required")?,
        output_format,
        naat_command,
    })
}

fn usage() -> &'static str {
    "Usage: sibyl-docverify verify <artifact> --checked-at <ISO8601> [--output json|markdown|both] [--naat-command <path>]"
}
