//! Overclaim audit: catalog claim tiers must match SE numeric-gate evidence.
#![forbid(unsafe_code)]

use std::fmt;

use pleiades_ayanamsa::built_in_ayanamsas;
use pleiades_houses::built_in_house_systems;
use pleiades_types::CompatibilityClaimTier;

use crate::{validate_ayanamsa_corpus, validate_house_corpus};

/// A violation found by the compatibility overclaim audit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CompatClaimAuditError {
    /// An entry is marked `ReleaseGradeNumeric` but the SE corpus does not
    /// validate it.
    ReleaseGradeWithoutCorpusEvidence {
        catalog: &'static str,
        entry: String,
    },
    /// An entry is marked `DescriptorOnly` but the SE corpus validates it.
    DescriptorOnlyHasEvidence {
        catalog: &'static str,
        entry: String,
    },
    /// The compatibility profile's release-grade-numeric count disagrees with
    /// the descriptor-derived count.
    ProfileCountMismatch {
        catalog: &'static str,
        profile: usize,
        descriptors: usize,
    },
    /// A prose/CLI surface disagrees with the descriptor-derived counts.
    SurfaceDisagrees { surface: &'static str },
}

impl fmt::Display for CompatClaimAuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReleaseGradeWithoutCorpusEvidence { catalog, entry } => write!(
                f,
                "{catalog} entry `{entry}` is ReleaseGradeNumeric but has no corpus evidence"
            ),
            Self::DescriptorOnlyHasEvidence { catalog, entry } => write!(
                f,
                "{catalog} entry `{entry}` is DescriptorOnly but the corpus validates it"
            ),
            Self::ProfileCountMismatch {
                catalog,
                profile,
                descriptors,
            } => write!(
                f,
                "{catalog} profile release-grade count {profile} != descriptor count {descriptors}"
            ),
            Self::SurfaceDisagrees { surface } => {
                write!(
                    f,
                    "surface `{surface}` disagrees with descriptor-derived counts"
                )
            }
        }
    }
}

impl std::error::Error for CompatClaimAuditError {}

/// Classifies one entry's tier against whether the corpus validated it.
/// Returns the violation, if any. Pure: enables exhaustive testing of both directions.
fn classify_tier_evidence(
    catalog: &'static str,
    entry: &str,
    tier: CompatibilityClaimTier,
    has_evidence: bool,
) -> Option<CompatClaimAuditError> {
    match tier {
        CompatibilityClaimTier::ReleaseGradeNumeric if !has_evidence => {
            Some(CompatClaimAuditError::ReleaseGradeWithoutCorpusEvidence {
                catalog,
                entry: entry.to_string(),
            })
        }
        CompatibilityClaimTier::DescriptorOnly if has_evidence => {
            Some(CompatClaimAuditError::DescriptorOnlyHasEvidence {
                catalog,
                entry: entry.to_string(),
            })
        }
        _ => None,
    }
}

/// Check A: bidirectional tier ↔ corpus-evidence agreement for both catalogs.
fn check_tier_evidence(errors: &mut Vec<CompatClaimAuditError>) {
    let house_report = match validate_house_corpus() {
        Ok(r) => r,
        Err(_) => {
            errors.push(CompatClaimAuditError::SurfaceDisagrees {
                surface: "house-corpus-gate",
            });
            return;
        }
    };
    for d in built_in_house_systems() {
        let has_evidence = house_report.validated_systems().contains(&d.system);
        if let Some(e) =
            classify_tier_evidence("house", d.canonical_name, d.claim_tier, has_evidence)
        {
            errors.push(e);
        }
    }

    let aya_report = match validate_ayanamsa_corpus() {
        Ok(r) => r,
        Err(_) => {
            errors.push(CompatClaimAuditError::SurfaceDisagrees {
                surface: "ayanamsa-corpus-gate",
            });
            return;
        }
    };
    for d in built_in_ayanamsas() {
        let has_evidence = aya_report.validated_modes().contains(&d.ayanamsa);
        if let Some(e) =
            classify_tier_evidence("ayanamsa", d.canonical_name, d.claim_tier, has_evidence)
        {
            errors.push(e);
        }
    }
}

/// Check B: profile release-grade-numeric count must equal the descriptor-derived count.
fn check_profile(errors: &mut Vec<CompatClaimAuditError>) {
    let profile = pleiades_core::current_compatibility_profile();

    let house_profile = profile
        .baseline_house_systems
        .iter()
        .chain(profile.release_house_systems.iter())
        .filter(|d| d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric)
        .count();
    let house_descriptors = built_in_house_systems()
        .iter()
        .filter(|d| d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric)
        .count();
    if house_profile != house_descriptors {
        errors.push(CompatClaimAuditError::ProfileCountMismatch {
            catalog: "house",
            profile: house_profile,
            descriptors: house_descriptors,
        });
    }

    let aya_profile = profile
        .baseline_ayanamsas
        .iter()
        .chain(profile.release_ayanamsas.iter())
        .filter(|d| d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric)
        .count();
    let aya_descriptors = built_in_ayanamsas()
        .iter()
        .filter(|d| d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric)
        .count();
    if aya_profile != aya_descriptors {
        errors.push(CompatClaimAuditError::ProfileCountMismatch {
            catalog: "ayanamsa",
            profile: aya_profile,
            descriptors: aya_descriptors,
        });
    }
}

/// Check C: README prose must state the release-grade-numeric counts verbatim.
fn check_surfaces(errors: &mut Vec<CompatClaimAuditError>) {
    const README: &str = include_str!("../../../../README.md");

    let house_count = built_in_house_systems()
        .iter()
        .filter(|d| d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric)
        .count();
    let aya_count = built_in_ayanamsas()
        .iter()
        .filter(|d| d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric)
        .count();

    // Forward drift-guard: if a future commit increments a descriptor count
    // (e.g. 12→13) without updating the README, the new token won't be found
    // and the check fires.  It cannot catch a same-commit mistake where both
    // the README and the descriptor count are wrong together — Checks A and B
    // backstop that case.
    let house_token = format!(" {house_count} house systems pass");
    let aya_token = format!(" {aya_count} release-claimed");
    if !README.contains(&house_token) {
        errors.push(CompatClaimAuditError::SurfaceDisagrees {
            surface: "README:houses",
        });
    }
    if !README.contains(&aya_token) {
        errors.push(CompatClaimAuditError::SurfaceDisagrees {
            surface: "README:ayanamsa",
        });
    }
}

/// Runs the full compatibility overclaim audit (Checks A–C).
pub(crate) fn audit_compat_claims() -> Result<(), Vec<CompatClaimAuditError>> {
    let mut errors = Vec::new();
    check_tier_evidence(&mut errors);
    check_profile(&mut errors);
    check_surfaces(&mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_grade_numeric_set_is_non_empty() {
        let n = built_in_house_systems()
            .iter()
            .filter(|d| d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric)
            .count();
        assert!(
            n > 0,
            "release-grade-numeric house set is empty — audit would be vacuous"
        );
    }

    #[test]
    fn real_catalogs_pass_check_a() {
        let mut errors = Vec::new();
        check_tier_evidence(&mut errors);
        assert!(errors.is_empty(), "unexpected violations: {errors:?}");
    }

    #[test]
    fn release_grade_membership_operator_fires() {
        // Prove the membership test that Check A relies on actually fires.
        let report = validate_house_corpus().expect("gate passes");
        let validated = report.validated_systems();
        assert!(!validated.is_empty());
        let any_release = built_in_house_systems().iter().any(|d| {
            d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric
                && validated.contains(&d.system)
        });
        assert!(any_release);
    }

    #[test]
    fn profile_release_grade_counts_match_descriptors() {
        let mut errors = Vec::new();
        super::check_profile(&mut errors);
        assert!(errors.is_empty(), "profile disagreement: {errors:?}");
    }

    #[test]
    fn readme_counts_match_descriptors() {
        let mut errors = Vec::new();
        super::check_surfaces(&mut errors);
        assert!(errors.is_empty(), "surface drift: {errors:?}");
    }

    #[test]
    fn descriptor_only_entry_with_corpus_evidence_is_flagged() {
        // DescriptorOnly + has evidence -> DescriptorOnlyHasEvidence
        assert!(matches!(
            classify_tier_evidence(
                "house",
                "Synthetic",
                CompatibilityClaimTier::DescriptorOnly,
                true
            ),
            Some(CompatClaimAuditError::DescriptorOnlyHasEvidence { .. })
        ));
        // ReleaseGradeNumeric + no evidence -> ReleaseGradeWithoutCorpusEvidence
        assert!(matches!(
            classify_tier_evidence(
                "ayanamsa",
                "Synthetic",
                CompatibilityClaimTier::ReleaseGradeNumeric,
                false
            ),
            Some(CompatClaimAuditError::ReleaseGradeWithoutCorpusEvidence { .. })
        ));
        // the two consistent cases -> None
        assert!(classify_tier_evidence(
            "house",
            "X",
            CompatibilityClaimTier::ReleaseGradeNumeric,
            true
        )
        .is_none());
        assert!(classify_tier_evidence(
            "house",
            "X",
            CompatibilityClaimTier::DescriptorOnly,
            false
        )
        .is_none());
    }
}
