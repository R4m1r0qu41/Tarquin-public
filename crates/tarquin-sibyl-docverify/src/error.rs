use thiserror::Error;

#[derive(Debug, Error)]
pub enum DocVerifyError {
    #[error("I/O error reading artifact: {0}")]
    Io(#[from] std::io::Error),
    #[error("no profile matched the artifact")]
    NoProfileMatch,
    #[error("evidence manifest serialization failed: {0}")]
    ManifestSerialize(String),
    #[error("report serialization failed: {0}")]
    ReportSerialize(String),
}
