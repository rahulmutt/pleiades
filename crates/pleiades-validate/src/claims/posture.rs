//! The derived release posture and its structural validation.

use std::fmt;

use pleiades_backend::{BackendMetadata, EphemerisBackend, ReleasePosture};

/// Metadata for the four release-facing backends.
///
/// `jpl-snapshot` is intentionally excluded — it is a checked-in DE441 validation
/// fixture, not a release-claimed runtime backend.
pub(crate) fn canonical_release_metadata() -> Vec<BackendMetadata> {
    vec![
        pleiades_data::PackagedDataBackend::default().metadata(),
        crate::claims::spk_release_backend().metadata(),
        pleiades_elp::ElpBackend.metadata(),
        pleiades_vsop87::Vsop87Backend.metadata(),
    ]
}

/// The derived, cross-backend release posture aggregated over the canonical set.
pub(crate) fn derived_release_posture() -> ReleasePosture {
    let metas = canonical_release_metadata();
    let refs: Vec<&BackendMetadata> = metas.iter().collect();
    ReleasePosture::from_backends(&refs)
}

/// Returns the derived release-grade body-claims summary line.
///
/// This is the cutover replacement for the retired
/// `pleiades-backend::validated_release_body_claims_summary_line_for_report`: the
/// line is now derived from the per-backend claim model rather than from frozen
/// prose. It is `Result` for signature compatibility with its many call sites
/// (bundle generation/verification, corpus report, body/date/channel summary);
/// the structural posture is always well-defined, so it currently never errors.
pub(crate) fn validated_release_body_claims_summary_line_for_report(
) -> Result<String, ClaimPostureError> {
    let posture = derived_release_posture();
    validate_release_posture(&posture)?;
    Ok(posture.summary_line())
}

/// Structured error from the structural release-posture validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ClaimPostureError {
    /// A single backend assigns two different tiers to the same body.
    ConflictingTier {
        /// The backend id holding the conflicting claims.
        backend: String,
        /// The body claimed at two different tiers by that backend.
        body: String,
    },
}

impl fmt::Display for ClaimPostureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConflictingTier { backend, body } => write!(
                f,
                "backend `{backend}` claims body `{body}` at more than one tier"
            ),
        }
    }
}

impl std::error::Error for ClaimPostureError {}

/// Validates the derived posture structurally.
///
/// The per-backend claim model makes most legacy invariants true by construction
/// (a body may legitimately hold different tiers via *different* backends, e.g.
/// Pluto is `ReleaseGrade@pleiades-data` and `Approximate@pleiades-vsop87`). The
/// remaining honesty invariant that is not guaranteed by the type is that a
/// *single* backend must not claim the same body at two different tiers.
pub(crate) fn validate_release_posture(posture: &ReleasePosture) -> Result<(), ClaimPostureError> {
    for (id, claim) in &posture.entries {
        for (id2, claim2) in &posture.entries {
            if id == id2 && claim.body == claim2.body && claim.tier != claim2.tier {
                return Err(ClaimPostureError::ConflictingTier {
                    backend: id.as_str().to_string(),
                    body: claim.body.to_string(),
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::CelestialBody;

    #[test]
    fn derived_posture_promotes_pluto_via_packaged_data() {
        let posture = derived_release_posture();
        let rg = posture.release_grade();
        // Pluto is release-grade via the packaged-data backend (id "pleiades-data").
        assert!(rg
            .iter()
            .any(|(id, b)| id.as_str() == "pleiades-data" && b == &CelestialBody::Pluto));
        // Pluto via vsop87 must NOT be release-grade.
        assert!(!rg
            .iter()
            .any(|(id, b)| id.as_str() == "pleiades-vsop87" && b == &CelestialBody::Pluto));
    }

    #[test]
    fn structural_validation_accepts_derived_posture() {
        assert_eq!(validate_release_posture(&derived_release_posture()), Ok(()));
    }
}
