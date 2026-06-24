//! Overclaim audit: catalog claim tiers must match SE numeric-gate evidence.
#![forbid(unsafe_code)]

use std::fmt;

use pleiades_ayanamsa::built_in_ayanamsas;
use pleiades_houses::built_in_house_systems;
use pleiades_types::CompatibilityClaimTier;

use crate::{validate_ayanamsa_corpus, validate_house_corpus};

/// A violation found by the compatibility overclaim audit.
// `ProfileCountMismatch` is constructed in Task 6; `SurfaceDisagrees` variants
// added in Tasks 6–7. Allow dead_code until all variants are wired.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CompatClaimAuditError {
    /// An entry is marked `ReleaseGradeNumeric` but the SE corpus does not
    /// validate it.
    ReleaseGradeWithoutCorpusEvidence { catalog: &'static str, entry: String },
    /// An entry is marked `DescriptorOnly` but the SE corpus validates it.
    DescriptorOnlyHasEvidence { catalog: &'static str, entry: String },
    /// The compatibility profile's release-grade-numeric count disagrees with
    /// the descriptor-derived count.
    ProfileCountMismatch { catalog: &'static str, profile: usize, descriptors: usize },
    /// A prose/CLI surface disagrees with the descriptor-derived counts.
    SurfaceDisagrees { surface: &'static str },
}

impl fmt::Display for CompatClaimAuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReleaseGradeWithoutCorpusEvidence { catalog, entry } => write!(
                f, "{catalog} entry `{entry}` is ReleaseGradeNumeric but has no corpus evidence"
            ),
            Self::DescriptorOnlyHasEvidence { catalog, entry } => write!(
                f, "{catalog} entry `{entry}` is DescriptorOnly but the corpus validates it"
            ),
            Self::ProfileCountMismatch { catalog, profile, descriptors } => write!(
                f, "{catalog} profile release-grade count {profile} != descriptor count {descriptors}"
            ),
            Self::SurfaceDisagrees { surface } => {
                write!(f, "surface `{surface}` disagrees with descriptor-derived counts")
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
// Called by `audit_compat_claims` and by tests; not yet called from Task 6–7 callers.
#[allow(dead_code)]
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
        let has_evidence = house_report
            .validated_systems()
            .iter()
            .any(|s| *s == d.system);
        if let Some(e) = classify_tier_evidence("house", d.canonical_name, d.claim_tier, has_evidence) {
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
        let has_evidence = aya_report.validated_modes().iter().any(|m| *m == d.ayanamsa);
        if let Some(e) = classify_tier_evidence("ayanamsa", d.canonical_name, d.claim_tier, has_evidence) {
            errors.push(e);
        }
    }
}

/// Runs the full compatibility overclaim audit (Checks A–C).
// Called by the gate runner (Tasks 6–7); not yet wired to an external caller.
#[allow(dead_code)]
pub(crate) fn audit_compat_claims() -> Result<(), Vec<CompatClaimAuditError>> {
    let mut errors = Vec::new();
    check_tier_evidence(&mut errors);
    // Check B (Task 6) and Check C (Task 7) append here.
    if errors.is_empty() { Ok(()) } else { Err(errors) }
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
        assert!(n > 0, "release-grade-numeric house set is empty — audit would be vacuous");
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
                && validated.iter().any(|s| *s == d.system)
        });
        assert!(any_release);
    }

    #[test]
    fn descriptor_only_entry_with_corpus_evidence_is_flagged() {
        // DescriptorOnly + has evidence -> DescriptorOnlyHasEvidence
        assert!(matches!(
            classify_tier_evidence("house", "Synthetic", CompatibilityClaimTier::DescriptorOnly, true),
            Some(CompatClaimAuditError::DescriptorOnlyHasEvidence { .. })
        ));
        // ReleaseGradeNumeric + no evidence -> ReleaseGradeWithoutCorpusEvidence
        assert!(matches!(
            classify_tier_evidence("ayanamsa", "Synthetic", CompatibilityClaimTier::ReleaseGradeNumeric, false),
            Some(CompatClaimAuditError::ReleaseGradeWithoutCorpusEvidence { .. })
        ));
        // the two consistent cases -> None
        assert!(classify_tier_evidence("house", "X", CompatibilityClaimTier::ReleaseGradeNumeric, true).is_none());
        assert!(classify_tier_evidence("house", "X", CompatibilityClaimTier::DescriptorOnly, false).is_none());
    }
}
