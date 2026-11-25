//! CLI process abstraction. Tests inject `FakeRunner`; production uses `SystemRunner`.
//!
//! External commands are invoked directly via `std::process::Command` — never via a shell.
//! stdout/stderr are captured and bounded to 64 KiB each. Missing tools (`ENOENT`) produce
//! `CliResult::ToolMissing`; the caller records the absence in the evidence manifest and keeps
//! the affected dimension at `not_determined`. Missing tools must never fabricate a finding.

use std::path::Path;

/// Outcome of a single bounded CLI invocation.
#[derive(Debug, Clone)]
pub enum CliResult {
    /// Command exited 0; stdout is the relevant output.
    Ok { stdout: String },
    /// Binary not found (`ENOENT`). Caller should record as unavailable.
    ToolMissing,
    /// Command exited non-zero. Captured for evidence; dimension stays `not_determined`.
    Failed {
        stdout: String,
        stderr: String,
        code: i32,
    },
}

impl CliResult {
    pub fn stdout(&self) -> Option<&str> {
        match self {
            CliResult::Ok { stdout } | CliResult::Failed { stdout, .. } => Some(stdout),
            CliResult::ToolMissing => None,
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, CliResult::Ok { .. })
    }
}

/// Abstraction over external CLI execution. Implemented by `SystemRunner` (production) and
/// `FakeRunner` (tests). All implementations must: invoke the program directly (no shell), never
/// interpolate arguments into a shell string, and cap output to 64 KiB.
pub trait CliRunner {
    fn run(&self, program: &str, args: &[&str]) -> CliResult;
}

// ── Production runner ─────────────────────────────────────────────────────────

pub struct SystemRunner;

const MAX_OUTPUT: usize = 64 * 1024;

impl CliRunner for SystemRunner {
    fn run(&self, program: &str, args: &[&str]) -> CliResult {
        match std::process::Command::new(program).args(args).output() {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => CliResult::ToolMissing,
            Err(_) => CliResult::ToolMissing,
            Ok(out) => {
                let stdout = lossy_truncate(&out.stdout, MAX_OUTPUT);
                let stderr = lossy_truncate(&out.stderr, MAX_OUTPUT);
                if out.status.success() {
                    CliResult::Ok { stdout }
                } else {
                    CliResult::Failed {
                        stdout,
                        stderr,
                        code: out.status.code().unwrap_or(-1),
                    }
                }
            }
        }
    }
}

fn lossy_truncate(bytes: &[u8], limit: usize) -> String {
    let truncated = if bytes.len() > limit {
        &bytes[..limit]
    } else {
        bytes
    };
    String::from_utf8_lossy(truncated).into_owned()
}

// ── Test runner ───────────────────────────────────────────────────────────────

/// Pre-programmed responses keyed by program name. For use in tests only.
#[cfg(test)]
#[derive(Default)]
pub struct FakeRunner {
    pub responses: std::collections::HashMap<String, CliResult>,
}

#[cfg(test)]
impl FakeRunner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a canned `Ok` response for `program`.
    pub fn with_ok(mut self, program: &str, stdout: impl Into<String>) -> Self {
        self.responses.insert(
            program.to_string(),
            CliResult::Ok {
                stdout: stdout.into(),
            },
        );
        self
    }

    /// Register a `ToolMissing` response for `program` (simulates absent binary).
    pub fn with_missing(mut self, program: &str) -> Self {
        self.responses
            .insert(program.to_string(), CliResult::ToolMissing);
        self
    }
}

#[cfg(test)]
impl CliRunner for FakeRunner {
    fn run(&self, program: &str, _args: &[&str]) -> CliResult {
        self.responses
            .get(program)
            .cloned()
            .unwrap_or(CliResult::ToolMissing)
    }
}

/// Convenience: a runner that reports every tool as missing. Default for unit tests that don't
/// need any CLI output.
#[cfg(test)]
pub struct AllMissingRunner;

#[cfg(test)]
impl CliRunner for AllMissingRunner {
    fn run(&self, _program: &str, _args: &[&str]) -> CliResult {
        CliResult::ToolMissing
    }
}

// ── Path helper (unused at runtime but useful for callers) ────────────────────

/// Convert a `Path` to a `&str` argument, replacing non-UTF-8 sequences lossily.
/// Callers pass the result to `runner.run()` — never interpolate it into a shell string.
pub fn path_arg(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}
