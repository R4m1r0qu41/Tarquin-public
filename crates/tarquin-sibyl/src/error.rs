use thiserror::Error;

/// Errors raised by the Sibyl layer. Validation *findings* (rule violations) are returned as
/// data (`Vec<Finding>`), not errors; `SibylError` is reserved for operational failures.
#[derive(Debug, Error)]
pub enum SibylError {
    /// A registry or LBRY document could not be parsed.
    #[error("parse error: {0}")]
    Parse(String),

    /// A referenced Sibyl DID is not present in the registry.
    #[error("unknown sibyl DID: {0}")]
    UnknownSibyl(String),

    /// Signature material could not be decoded.
    #[error("signature decode error: {0}")]
    SignatureDecode(String),

    /// An identity-layer (DID resolution / verification) failure.
    #[error("identity error: {0}")]
    Identity(String),
}
