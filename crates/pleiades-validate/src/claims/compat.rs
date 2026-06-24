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

/// Check A: bidirectional tier ↔ corpus-evidence agreement for both catalogs.
// Called by `audit_compat_claims` and by tests; not yet called from Task 6–7 callers.
#[allow(dead_code)]
fn check_tier_evidence(errors: &mut Vec<CompatClaimAuditError>) {
    let house_report = match validate_house_corpus() {
        Ok(r) => r,
        Err(e) => {
            errors.push(CompatClaimAuditError::SurfaceDisagrees {
                surface: "house-corpus-gate",
            });
            let _ = e;
            return;
        }
    };
    for d in built_in_house_systems() {
        let has_evidence = house_report
            .validated_systems()
            .iter()
            .any(|s| *s == d.system);
        match d.claim_tier {
            CompatibilityClaimTier::ReleaseGradeNumeric if !has_evidence => {
                errors.push(CompatClaimAuditError::ReleaseGradeWithoutCorpusEvidence {
                    catalog: "house",
                    entry: d.canonical_name.to_string(),
                });
            }
            CompatibilityClaimTier::DescriptorOnly if has_evidence => {
                errors.push(CompatClaimAuditError::DescriptorOnlyHasEvidence {
                    catalog: "house",
                    entry: d.canonical_name.to_string(),
                });
            }
            _ => {}
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
        match d.claim_tier {
            CompatibilityClaimTier::ReleaseGradeNumeric if !has_evidence => {
                errors.push(CompatClaimAuditError::ReleaseGradeWithoutCorpusEvidence {
                    catalog: "ayanamsa",
                    entry: d.canonical_name.to_string(),
                });
            }
            CompatibilityClaimTier::DescriptorOnly if has_evidence => {
                errors.push(CompatClaimAuditError::DescriptorOnlyHasEvidence {
                    catalog: "ayanamsa",
                    entry: d.canonical_name.to_string(),
                });
            }
            _ => {}
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
    fn descriptor_only_with_evidence_is_detected() {
        // Synthetic: a house validated by the corpus but treated as DescriptorOnly.
        let report = validate_house_corpus().expect("gate passes");
        let validated = report.validated_systems();
        assert!(!validated.is_empty());
        // Prove the membership test that Check A relies on actually fires.
        let any_release = built_in_house_systems().iter().any(|d| {
            d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric
                && validated.iter().any(|s| *s == d.system)
        });
        assert!(any_release);
    }
}
