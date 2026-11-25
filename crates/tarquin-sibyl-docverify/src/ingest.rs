//! Artifact ingestion: SHA-256 hash and media-type detection from magic bytes.

use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

use crate::error::DocVerifyError;

/// Media type detected from magic bytes (no file-extension heuristics).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    Pdf,
    Jpeg,
    Png,
    /// Could not identify from the first bytes read.
    Unknown,
}

impl MediaType {
    pub fn as_str(self) -> &'static str {
        match self {
            MediaType::Pdf => "application/pdf",
            MediaType::Jpeg => "image/jpeg",
            MediaType::Png => "image/png",
            MediaType::Unknown => "application/octet-stream",
        }
    }
}

/// Result of ingesting an artifact file.
#[derive(Debug, Clone)]
pub struct IngestedArtifact {
    /// `sha256:<hex>` of the full artifact bytes.
    pub sha256: String,
    pub media_type: MediaType,
    pub byte_len: u64,
}

/// Hash the artifact and detect its media type. Reads the file once with streaming SHA-256.
pub fn ingest(path: &Path) -> Result<IngestedArtifact, DocVerifyError> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut header = [0u8; 8];
    let header_read = file.read(&mut header)?;

    hasher.update(&header[..header_read]);

    let mut buf = [0u8; 8192];
    let mut byte_len = header_read as u64;
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        byte_len += n as u64;
    }

    let digest = hasher.finalize();
    let sha256 = format!("sha256:{}", hex_encode(&digest));
    let media_type = detect_media_type(&header[..header_read]);

    Ok(IngestedArtifact {
        sha256,
        media_type,
        byte_len,
    })
}

/// Hash a byte slice directly (for tests and synthetic fixtures).
pub fn ingest_bytes(bytes: &[u8]) -> IngestedArtifact {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let sha256 = format!("sha256:{}", hex_encode(&digest));
    let media_type = detect_media_type(bytes);
    IngestedArtifact {
        sha256,
        media_type,
        byte_len: bytes.len() as u64,
    }
}

fn detect_media_type(header: &[u8]) -> MediaType {
    if header.len() >= 4 && &header[..4] == b"%PDF" {
        MediaType::Pdf
    } else if header.len() >= 2 && header[0] == 0xFF && header[1] == 0xD8 {
        MediaType::Jpeg
    } else if header.len() >= 4 && &header[..4] == b"\x89PNG" {
        MediaType::Png
    } else {
        MediaType::Unknown
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdf_magic_detected() {
        let bytes = b"%PDF-1.4 fake document bytes";
        let a = ingest_bytes(bytes);
        assert_eq!(a.media_type, MediaType::Pdf);
        assert!(a.sha256.starts_with("sha256:"));
        assert_eq!(a.byte_len, bytes.len() as u64);
    }

    #[test]
    fn jpeg_magic_detected() {
        let bytes = [0xFF, 0xD8, 0xFF, 0xE0, 0x00];
        let a = ingest_bytes(&bytes);
        assert_eq!(a.media_type, MediaType::Jpeg);
    }

    #[test]
    fn png_magic_detected() {
        let bytes = b"\x89PNG\r\n\x1a\n fake png";
        let a = ingest_bytes(bytes);
        assert_eq!(a.media_type, MediaType::Png);
    }

    #[test]
    fn unknown_bytes_give_unknown_type() {
        let a = ingest_bytes(b"random data");
        assert_eq!(a.media_type, MediaType::Unknown);
    }

    #[test]
    fn hash_is_deterministic() {
        let a1 = ingest_bytes(b"hello");
        let a2 = ingest_bytes(b"hello");
        assert_eq!(a1.sha256, a2.sha256);
    }

    #[test]
    fn different_bytes_give_different_hash() {
        let a1 = ingest_bytes(b"hello");
        let a2 = ingest_bytes(b"world");
        assert_ne!(a1.sha256, a2.sha256);
    }
}
