//! Assembly of a `ClaimedFinding` from the evidence collected by the extraction engine.
//!
//! At MVP (detection only) the engine can produce:
//! - `document_integrity`: `PartiallyConfirmed` backed by `ArtifactHash` evidence.
//! - All other dimensions: `not_determined` (no validation-class evidence available yet).
//!
//! This is correct: the reducer will confirm that `ArtifactHash` alone caps integrity at
//! `partially_confirmed`, and all derived dimensions remain `not_determined`. `requires_human_review`
//! will always be `true` at this phase.

use tarquin_sibyl::docverify::{
    ClaimedFinding, DimensionClaim, EvidenceKind, EvidenceRef, SemanticState,
};

use crate::{
    evidence::{EvidenceEntry, EvidenceManifest},
    ingest::IngestedArtifact,
    profiles::Profile,
};

/// Assemble a `ClaimedFinding` from the evidence collected in this run.
///
/// `checked_at` must be an explicit ISO 8601 timestamp supplied by the caller — never sourced
/// from the system clock inside this function.
pub fn build_claim(
    artifact: &IngestedArtifact,
    profile: &Profile,
    manifest: &EvidenceManifest,
    manifest_hash: &str,
    checked_at: &str,
) -> ClaimedFinding {
    // Find the artifact-hash entry to use as the EvidenceRef for document_integrity.
    let hash_entry = manifest.entries.iter().find(|e| e.id == "ev-artifact-hash");

    let document_integrity = build_integrity_claim(artifact, profile, hash_entry);

    ClaimedFinding {
        artifact_sha256: artifact.sha256.clone(),
        profile_id: profile.id.to_string(),
        profile_hash: profile.profile_hash.to_string(),
        evidence_manifest_hash: manifest_hash.to_string(),
        checked_at: checked_at.to_string(),
        document_integrity,
        issuer_identity: DimensionClaim::default(),
        signer_identity: DimensionClaim::default(),
        signer_capacity: DimensionClaim::default(),
        registry_status: DimensionClaim::default(),
        filing_status: DimensionClaim::default(),
        document_authenticity: DimensionClaim::default(),
        legal_effect: DimensionClaim::default(),
    }
}

/// Build the `document_integrity` claim from artifact-hash evidence. `ArtifactHash` grounds
/// integrity at `PartiallyConfirmed` — the reducer will cap it there.
fn build_integrity_claim(
    _artifact: &IngestedArtifact,
    profile: &Profile,
    hash_entry: Option<&EvidenceEntry>,
) -> DimensionClaim {
    let Some(entry) = hash_entry else {
        return DimensionClaim::default();
    };

    let evidence_ref = EvidenceRef {
        kind: EvidenceKind::ArtifactHash,
        lex_refs: profile.lex_refs.iter().map(|s| s.to_string()).collect(),
        evidence_id: entry.id.clone(),
    };

    DimensionClaim {
        state: SemanticState::PartiallyConfirmed,
        evidence: vec![evidence_ref],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{evidence::EvidenceManifest, ingest::MediaType, profiles::all_profiles};
    use tarquin_sibyl::docverify::reduce;

    fn digest(nibble: char) -> String {
        format!("sha256:{}", nibble.to_string().repeat(64))
    }

    fn fake_artifact(sha: &str) -> IngestedArtifact {
        IngestedArtifact {
            sha256: sha.to_string(),
            media_type: MediaType::Pdf,
            byte_len: 1024,
        }
    }

    fn fake_manifest(artifact_sha: &str, profile_id: &str) -> EvidenceManifest {
        EvidenceManifest {
            artifact_sha256: artifact_sha.to_string(),
            profile_id: profile_id.to_string(),
            entries: vec![crate::evidence::EvidenceEntry {
                id: "ev-artifact-hash".to_string(),
                kind: "artifact_hash".to_string(),
                description: "test".to_string(),
                tool_identity: "sha2::Sha256".to_string(),
            }],
        }
    }

    #[test]
    fn claim_has_correct_artifact_sha_and_profile() {
        let artifact_sha = digest('a');
        let artifact = fake_artifact(&artifact_sha);
        let profile = &all_profiles()[3]; // MX-JUZGADO-OFICIO-01
        let manifest = fake_manifest(&artifact_sha, profile.id);
        let manifest_hash = manifest.canonical_hash();

        let claim = build_claim(
            &artifact,
            profile,
            &manifest,
            &manifest_hash,
            "2026-06-27T00:00:00Z",
        );

        assert_eq!(claim.artifact_sha256, artifact_sha);
        assert_eq!(claim.profile_id, "MX-JUZGADO-OFICIO-01");
        assert_eq!(claim.checked_at, "2026-06-27T00:00:00Z");
        assert_eq!(claim.profile_hash, profile.profile_hash);
        assert!(!claim.evidence_manifest_hash.is_empty());
    }

    #[test]
    fn document_integrity_is_partially_confirmed_from_hash() {
        let artifact_sha = digest('a');
        let artifact = fake_artifact(&artifact_sha);
        let profile = &all_profiles()[3];
        let manifest = fake_manifest(&artifact_sha, profile.id);
        let manifest_hash = manifest.canonical_hash();

        let claim = build_claim(
            &artifact,
            profile,
            &manifest,
            &manifest_hash,
            "2026-06-27T00:00:00Z",
        );

        assert_eq!(
            claim.document_integrity.state,
            SemanticState::PartiallyConfirmed
        );
        assert_eq!(claim.document_integrity.evidence.len(), 1);
        assert_eq!(
            claim.document_integrity.evidence[0].kind,
            EvidenceKind::ArtifactHash
        );
    }

    #[test]
    fn all_other_dimensions_are_not_determined() {
        let artifact_sha = digest('a');
        let artifact = fake_artifact(&artifact_sha);
        let profile = &all_profiles()[3];
        let manifest = fake_manifest(&artifact_sha, profile.id);
        let manifest_hash = manifest.canonical_hash();

        let claim = build_claim(
            &artifact,
            profile,
            &manifest,
            &manifest_hash,
            "2026-06-27T00:00:00Z",
        );

        assert_eq!(claim.issuer_identity.state, SemanticState::NotDetermined);
        assert_eq!(claim.signer_identity.state, SemanticState::NotDetermined);
        assert_eq!(claim.signer_capacity.state, SemanticState::NotDetermined);
        assert_eq!(claim.registry_status.state, SemanticState::NotDetermined);
        assert_eq!(claim.filing_status.state, SemanticState::NotDetermined);
        assert_eq!(
            claim.document_authenticity.state,
            SemanticState::NotDetermined
        );
        assert_eq!(claim.legal_effect.state, SemanticState::NotDetermined);
    }

    #[test]
    fn reduce_over_engine_claim_gives_partial_integrity_and_human_review() {
        let artifact_sha = digest('a');
        let artifact = fake_artifact(&artifact_sha);
        let profile = &all_profiles()[3];
        let manifest = fake_manifest(&artifact_sha, profile.id);
        let manifest_hash = manifest.canonical_hash();

        let claim = build_claim(
            &artifact,
            profile,
            &manifest,
            &manifest_hash,
            "2026-06-27T00:00:00Z",
        );
        let finding = reduce(&claim);

        assert_eq!(
            finding.document_integrity,
            SemanticState::PartiallyConfirmed
        );
        assert_eq!(finding.document_authenticity, SemanticState::NotDetermined);
        assert_eq!(finding.legal_effect, SemanticState::NotDetermined);
        assert!(finding.requires_human_review());
    }

    #[test]
    fn checked_at_is_passed_through_verbatim() {
        let artifact_sha = digest('c');
        let artifact = fake_artifact(&artifact_sha);
        let profile = &all_profiles()[1]; // MX-SAT-RFC-01
        let manifest = fake_manifest(&artifact_sha, profile.id);
        let manifest_hash = manifest.canonical_hash();

        let explicit_time = "2026-06-27T12:00:00Z";
        let claim = build_claim(&artifact, profile, &manifest, &manifest_hash, explicit_time);
        assert_eq!(claim.checked_at, explicit_time);
    }

    #[test]
    fn identical_inputs_produce_identical_claim() {
        let artifact_sha = digest('d');
        let artifact = fake_artifact(&artifact_sha);
        let profile = &all_profiles()[0];
        let manifest = fake_manifest(&artifact_sha, profile.id);
        let mh = manifest.canonical_hash();

        let c1 = build_claim(&artifact, profile, &manifest, &mh, "2026-06-27T00:00:00Z");
        let c2 = build_claim(&artifact, profile, &manifest, &mh, "2026-06-27T00:00:00Z");

        assert_eq!(c1.artifact_sha256, c2.artifact_sha256);
        assert_eq!(c1.profile_hash, c2.profile_hash);
        assert_eq!(c1.evidence_manifest_hash, c2.evidence_manifest_hash);
        assert_eq!(c1.document_integrity.state, c2.document_integrity.state);
    }
}
