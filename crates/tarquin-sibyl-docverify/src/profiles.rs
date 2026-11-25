//! Synthetic MX document profiles. Each profile describes an expected document class for
//! the MX jurisdiction. Profile matching determines which lex_refs are cited in evidence
//! and which features are expected. Profiles are embedded at compile time.

use crate::ingest::MediaType;
use crate::qr::QrClass;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct Profile {
    pub id: &'static str,
    pub display_name: &'static str,
    /// Expected media type for matching. `None` means any type matches.
    pub expected_media_type: Option<MediaType>,
    /// Whether the profile expects a QR code to be present.
    pub requires_qr: bool,
    /// Expected QR content class, or `None` if QR class is not a matching criterion.
    pub expected_qr_class: Option<QrClass>,
    /// Lex references to cite for evidence produced against this profile.
    pub lex_refs: &'static [&'static str],
    /// Stable synthetic hash identifying this profile definition.
    pub profile_hash: &'static str,
}

impl Profile {
    /// Hash the runtime-relevant profile definition using sorted-key JSON.
    pub fn computed_hash(&self) -> String {
        let canonical = serde_json::json!({
            "display_name": self.display_name,
            "expected_media_type": media_type_name(self.expected_media_type),
            "expected_qr_class": self.expected_qr_class.as_ref().map(qr_class_name),
            "id": self.id,
            "lex_refs": self.lex_refs,
            "requires_qr": self.requires_qr,
        });
        let bytes = serde_json::to_vec(&canonical).expect("profile JSON value must serialize");
        let digest = Sha256::digest(bytes);
        format!("sha256:{}", hex_encode(&digest))
    }
}

fn media_type_name(media_type: Option<MediaType>) -> &'static str {
    match media_type {
        None => "any",
        Some(MediaType::Pdf) => "pdf",
        Some(MediaType::Jpeg | MediaType::Png) => "image",
        Some(MediaType::Unknown) => "unknown",
    }
}

fn qr_class_name(class: &QrClass) -> &'static str {
    match class {
        QrClass::MexicoCurp => "MexicoCurp",
        QrClass::MexicoCfdiUrl => "MexicoCfdiUrl",
        QrClass::MexicoRfc => "MexicoRfc",
        QrClass::Unclassified => "Unclassified",
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

static PROFILES: &[Profile] = &[
    Profile {
        id: "MX-RENAPO-CURP-01",
        display_name: "RENAPO — Constancia CURP",
        expected_media_type: Some(MediaType::Jpeg),
        requires_qr: true,
        expected_qr_class: Some(QrClass::MexicoCurp),
        lex_refs: &["lex-mx-ccom-89", "lex-mx-lfea-7"],
        profile_hash: "sha256:ae48d53c4a41e2955e803901f21eb60cc20acfb089db2e74923bdc982576e4b5",
    },
    Profile {
        id: "MX-SAT-RFC-01",
        display_name: "SAT — Cédula de Identificación Fiscal (RFC)",
        expected_media_type: None,
        requires_qr: false,
        expected_qr_class: None,
        lex_refs: &["lex-mx-ccom-89", "lex-mx-lfea-7"],
        profile_hash: "sha256:4c8243dab16527a04bb5291e0c6a45fde3c02e03d0336429b576f54172e23507",
    },
    Profile {
        id: "MX-INE-VOTORID-01",
        display_name: "INE — Credencial para Votar",
        expected_media_type: Some(MediaType::Jpeg),
        requires_qr: false,
        expected_qr_class: None,
        lex_refs: &["lex-mx-ccom-89"],
        profile_hash: "sha256:75f17d393301d75a004cb596c4003c4a2bba1053b8f89477a213a4c2b3d976ce",
    },
    Profile {
        id: "MX-JUZGADO-OFICIO-01",
        display_name: "Juzgado — Oficio Judicial",
        expected_media_type: Some(MediaType::Pdf),
        requires_qr: false,
        expected_qr_class: None,
        lex_refs: &["lex-mx-ccom-89"],
        profile_hash: "sha256:c8b64ef3cbb19cbac33ed402f6e0f80efcc1a664f6c9b44e88c759de51e23991",
    },
    Profile {
        id: "MX-SAT-CFDI-01",
        display_name: "SAT — Comprobante Fiscal Digital por Internet (CFDI)",
        expected_media_type: Some(MediaType::Pdf),
        requires_qr: true,
        expected_qr_class: Some(QrClass::MexicoCfdiUrl),
        lex_refs: &["lex-mx-ccom-89", "lex-mx-lfea-7"],
        profile_hash: "sha256:88efc1f5eb0b94598adff9dd7a442f6f97cb80633aa377e6a42613b12d3f5675",
    },
];

/// Match the most specific profile for the given artifact characteristics. Returns the first
/// profile whose constraints are all satisfied; specificity is ordered by how many constraints
/// a profile asserts (more constraints = more specific = tried first).
pub fn match_profile(media_type: MediaType, qr_texts: &[String]) -> Option<&'static Profile> {
    let qr_classes: Vec<QrClass> = qr_texts.iter().map(|t| crate::qr::classify_qr(t)).collect();

    // Most-specific first: QR+media > media-only > unconstrained (RFC = any type, any QR).
    let order = [0usize, 4, 2, 3, 1]; // CURP, CFDI, INE, JUZGADO, RFC

    for &i in &order {
        let p = &PROFILES[i];
        if !media_type_matches(p.expected_media_type, media_type) {
            continue;
        }
        if p.requires_qr && qr_texts.is_empty() {
            continue;
        }
        if let Some(ref expected_class) = p.expected_qr_class {
            if !qr_classes.contains(expected_class) {
                continue;
            }
        }
        return Some(p);
    }
    None
}

fn media_type_matches(expected: Option<MediaType>, actual: MediaType) -> bool {
    match expected {
        None => true,
        Some(e) => e == actual,
    }
}

/// Return all profiles (for introspection in tests and reports).
pub fn all_profiles() -> &'static [Profile] {
    PROFILES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jpeg_with_curp_qr_matches_renapo() {
        let p = match_profile(MediaType::Jpeg, &["BAAB260101HMCRNS09".to_string()])
            .expect("should match");
        assert_eq!(p.id, "MX-RENAPO-CURP-01");
    }

    #[test]
    fn pdf_with_cfdi_url_matches_cfdi() {
        let url = "https://verificacfdi.facturaelectronica.sat.gob.mx/main.aspx?id=x".to_string();
        let p = match_profile(MediaType::Pdf, &[url]).expect("should match");
        assert_eq!(p.id, "MX-SAT-CFDI-01");
    }

    #[test]
    fn pdf_without_qr_matches_juzgado() {
        let p = match_profile(MediaType::Pdf, &[]).expect("should match");
        assert_eq!(p.id, "MX-JUZGADO-OFICIO-01");
    }

    #[test]
    fn jpeg_without_qr_matches_ine() {
        let p = match_profile(MediaType::Jpeg, &[]).expect("should match");
        assert_eq!(p.id, "MX-INE-VOTORID-01");
    }

    #[test]
    fn unknown_media_type_without_qr_matches_rfc() {
        // RFC has no media-type or QR constraint (most permissive).
        let p = match_profile(MediaType::Unknown, &[]).expect("should match");
        assert_eq!(p.id, "MX-SAT-RFC-01");
    }

    #[test]
    fn all_five_profiles_are_present() {
        assert_eq!(all_profiles().len(), 5);
    }

    #[test]
    fn all_profiles_have_stable_hash() {
        for p in all_profiles() {
            assert_eq!(
                p.profile_hash,
                p.computed_hash(),
                "profile {} drifted",
                p.id
            );
        }
    }
}
