//! Fast structural claim audit (no corpus access).
//!
//! This module audits the canonical release backends for two structural
//! invariants that can be checked without loading a kernel or touching the
//! comparison corpus:
//!
//! 1. **Tier/evidence consistency** — a `ReleaseGrade` claim must be backed
//!    by `ArtifactValidated` or `CorpusValidated{..}` evidence.
//! 2. **Unsupported-body rejection** — every body declared `Unsupported` must
//!    be rejected by the backend's own `validate_request` preflight.

use crate::claims::canonical_release_metadata;
use pleiades_backend::{BodyClaim, BodyClaimTier, CelestialBody, ClaimEvidence, EphemerisBackend};
use pleiades_data::thresholds::accuracy_ceiling;
use std::fmt;

/// Degrees-to-arcseconds conversion factor.
const DEG_TO_ARCSEC: f64 = 3600.0;
/// Astronomical-unit-to-kilometre conversion factor.
const AU_TO_KM: f64 = 149_597_870.7;

/// Errors produced by the structural claim audit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClaimAuditError {
    /// A release-grade body that should be computable and comparable against a
    /// reference corpus was not found in the comparison report, or the entire
    /// comparison failed so no body could be checked.
    DeclaredBodyNotComputable {
        /// The backend identifier.
        backend: String,
        /// The body that is declared but not computable.
        body: String,
    },
    /// A backend's preflight (`validate_request`) accepted a request for a
    /// body that is declared `Unsupported` in the metadata.
    UnsupportedBodyAccepted {
        /// The backend identifier.
        backend: String,
        /// The body that was incorrectly accepted.
        body: String,
    },
    /// A `ReleaseGrade` claim is backed by evidence that does not meet the
    /// release bar (`ArtifactValidated` or `CorpusValidated{..}`).
    TierEvidenceMismatch {
        /// The backend identifier.
        backend: String,
        /// The body whose claim fails the tier/evidence check.
        body: String,
    },
    /// A `ReleaseGrade` body's measured corpus delta exceeds the SP3 accuracy
    /// ceiling for that body on one of the comparison channels.
    ReleaseGradeAboveCeiling {
        /// The backend identifier.
        backend: String,
        /// The body whose measured delta exceeds its ceiling.
        body: String,
        /// The channel that exceeded its ceiling (`longitude`, `latitude`, or
        /// `distance`).
        channel: String,
    },
}

impl fmt::Display for ClaimAuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeclaredBodyNotComputable { backend, body } => write!(
                f,
                "backend `{backend}` declares body `{body}` but cannot compute it"
            ),
            Self::UnsupportedBodyAccepted { backend, body } => write!(
                f,
                "backend `{backend}` accepted a request for unsupported body `{body}`"
            ),
            Self::TierEvidenceMismatch { backend, body } => write!(
                f,
                "backend `{backend}` claims `{body}` as ReleaseGrade but evidence is not artifact- or corpus-validated"
            ),
            Self::ReleaseGradeAboveCeiling {
                backend,
                body,
                channel,
            } => write!(
                f,
                "backend `{backend}` ReleaseGrade body `{body}` exceeds its accuracy ceiling on the `{channel}` channel"
            ),
        }
    }
}

impl std::error::Error for ClaimAuditError {}

/// Returns `Ok(())` when the claim's tier and evidence are consistent.
///
/// A `ReleaseGrade` claim is consistent only when evidence is
/// `ArtifactValidated` or `CorpusValidated{..}`.  Any other evidence
/// on a `ReleaseGrade` claim returns `Err(())`.  Non-`ReleaseGrade`
/// claims are always consistent and return `Ok(())`.
// Called by `audit_structural` and directly by the unit tests; exposed
// `pub` for Task 11 / future corpus audit callers.
#[allow(dead_code)]
pub fn tier_evidence_consistent(claim: &BodyClaim) -> Result<(), ()> {
    match (claim.tier, &claim.evidence) {
        (BodyClaimTier::ReleaseGrade, ClaimEvidence::ArtifactValidated)
        | (BodyClaimTier::ReleaseGrade, ClaimEvidence::CorpusValidated { .. }) => Ok(()),
        (BodyClaimTier::ReleaseGrade, _) => Err(()),
        _ => Ok(()),
    }
}

/// Runs the structural audit over all canonical release backends.
///
/// Checks performed (no corpus or kernel access):
/// - Tier/evidence consistency: every `ReleaseGrade` claim must be backed by
///   `ArtifactValidated` or `CorpusValidated` evidence.
/// - Unsupported-body rejection: every body declared `Unsupported` must be
///   rejected by the backend's own `validate_request` preflight.
///
/// Returns `Ok(())` when all backends pass.  Returns `Err(errors)` with the
/// full list of violations when any check fails.
// Exposed `pub` for Task 11 / future corpus audit callers.
#[allow(dead_code)]
pub fn audit_structural() -> Result<(), Vec<ClaimAuditError>> {
    let mut errors = Vec::new();

    for meta in canonical_release_metadata() {
        for claim in &meta.body_claims {
            // Check 1: tier/evidence consistency.
            if tier_evidence_consistent(claim).is_err() {
                errors.push(ClaimAuditError::TierEvidenceMismatch {
                    backend: meta.id.as_str().to_string(),
                    body: claim.body.to_string(),
                });
            }

            // Check 2: Unsupported-tier bodies must be rejected by preflight.
            if claim.tier == BodyClaimTier::Unsupported {
                let req = crate::claims::sample_request_for(&claim.body);
                if meta.validate_request(&req).is_ok() {
                    errors.push(ClaimAuditError::UnsupportedBodyAccepted {
                        backend: meta.id.as_str().to_string(),
                        body: claim.body.to_string(),
                    });
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Audits that every `ReleaseGrade` body actually meets its SP3 accuracy
/// ceiling against the reference corpus.
///
/// Two backends are exercised:
///
/// - **packaged-data** (`pleiades-data`): the packaged artifact is compared
///   against a hold-out-backed reference over [`crate::corpus::holdout_corpus`].
///   The reference is a [`SnapshotCorpusBackend`] over the committed independent
///   hold-out rows (`production_holdout_corpus`), and the corpus requests one
///   sample per hold-out row, so both sides are evaluated at exactly matching
///   epochs. This is the SP3-grade, apples-to-apples comparison that backs the
///   published per-body accuracy ceilings (it mirrors the in-`pleiades-data`
///   accuracy baseline). The coarse `default_corpus` + narrow `JplSnapshotBackend`
///   reference is deliberately *not* used: that pairing extrapolates from a
///   J2000-only snapshot and is tuned for the VSOP/ELP comparison, which inflates
///   latitude/distance deltas by orders of magnitude and does not reflect the
///   packaged artifact's true accuracy. This path runs with real teeth in the
///   kernel-free environment. The packaged asteroids (Ceres, Pallas, Juno, Vesta,
///   Eros) have no hold-out rows; they are checked in a separate block against the
///   committed `sb441-n373s` rows of `asteroid_reference.csv`, and a claimed
///   asteroid with no rows or no comparison summary is an error, not a skip.
///   Afterwards every remaining packaged release-grade body (other than the lunar
///   points gated by their own named gates) with no evidence row is reported as
///   [`ClaimAuditError::DeclaredBodyNotComputable`].
/// - **jpl-spk** (`jpl-spk`): the sb441-n373s Tier-A asteroid reference (a
///   [`SnapshotCorpusBackend`] over [`crate::corpus::asteroid_corpus`]) is
///   compared against the SPK release backend. In the kernel-free environment
///   the SPK candidate declares no release-grade bodies, so this path performs
///   no checks; it activates only when a small-body kernel is provided.
///
/// Returns `Ok(())` when every checked body is within its ceiling on all
/// channels; otherwise returns the full list of [`ClaimAuditError::ReleaseGradeAboveCeiling`]
/// findings. This is a slow audit (it runs full corpus comparisons) and is
/// gated behind an `#[ignore]`'d test.
#[allow(dead_code)]
pub fn audit_release_grade_accuracy() -> Result<(), Vec<ClaimAuditError>> {
    let mut errors = Vec::new();

    let mut holdout_audited: Vec<CelestialBody> = Vec::new();

    // packaged-data ReleaseGrade bodies vs the independent hold-out reference,
    // sampled at matching epochs (SP3-grade, apples-to-apples with the published
    // per-body accuracy ceilings).
    {
        let reference = pleiades_jpl::SnapshotCorpusBackend::from_entries(
            pleiades_jpl::production_holdout_corpus().to_vec(),
        );
        let candidate = pleiades_data::PackagedDataBackend::default();
        let corpus = crate::corpus::holdout_corpus();

        // Derive the set of release-grade bodies that have a hold-out truth row.
        // The hold-out corpus has no asteroid rows, so the intersection with the
        // hold-out truth rows below keeps only bodies that have both a claim and a
        // row; the packaged asteroids are audited against their own rows below.
        let holdout_bodies: std::collections::HashSet<CelestialBody> =
            pleiades_jpl::production_holdout_corpus()
                .iter()
                .map(|entry| entry.body.clone())
                .collect();
        let expected_bodies: Vec<CelestialBody> = candidate
            .metadata()
            .release_grade_bodies()
            .into_iter()
            .filter(|b| holdout_bodies.contains(b))
            .collect();
        holdout_audited.clone_from(&expected_bodies);

        match crate::comparison::compare_backends(&reference, &candidate, &corpus) {
            Ok(report) => {
                // Guard against an empty/incomplete report: every release-grade body
                // with a hold-out row must appear in the comparison summaries.
                let summaries = report.body_summaries();
                let summary_bodies: std::collections::HashSet<&CelestialBody> =
                    summaries.iter().map(|s| &s.body).collect();
                for body in &expected_bodies {
                    if !summary_bodies.contains(body) {
                        errors.push(ClaimAuditError::DeclaredBodyNotComputable {
                            backend: "pleiades-data".into(),
                            body: body.to_string(),
                        });
                    }
                }
                check_report(&report, "pleiades-data", &expected_bodies, &mut errors);
            }
            Err(_) => {
                // compare_backends failed (fails fast on the first unservable
                // request): this is a hard failure — we checked nothing.
                errors.push(ClaimAuditError::DeclaredBodyNotComputable {
                    backend: "pleiades-data".into(),
                    body: "<all release-grade-with-holdout>".into(),
                });
            }
        }
    }

    // packaged-data asteroids vs the sb441-n373s rows (issue #201). The
    // hold-out corpus has no asteroid rows, so the block above skips them.
    let asteroids_audited: Vec<CelestialBody> = {
        let asteroids: Vec<CelestialBody> = pleiades_data::packaged_body_claims()
            .into_iter()
            .filter(|claim| {
                matches!(&claim.evidence, ClaimEvidence::CorpusValidated { source } if source == "sb441-n373s")
            })
            .map(|claim| claim.body)
            .collect();
        audit_packaged_asteroids(
            &asteroids,
            pleiades_jpl::asteroid_reference_corpus(),
            &mut errors,
        );
        asteroids
    };

    // Spec section 4: a release-grade body with no evidence row fails the audit.
    for body in unaudited_release_grade_bodies(
        &pleiades_data::PackagedDataBackend::default()
            .metadata()
            .body_claims,
        &holdout_audited,
        &asteroids_audited,
    ) {
        errors.push(ClaimAuditError::DeclaredBodyNotComputable {
            backend: "pleiades-data".into(),
            body: body.to_string(),
        });
    }

    // jpl-spk Tier-A asteroids vs the sb441-n373s asteroid reference.
    // NOTE: in the kernel-free environment the SPK backend declares no
    // release-grade bodies, so compare_backends returns Err and this block
    // is skipped. The `if let Ok` skip is intentional here — the asteroid
    // path is dormant until a small-body kernel (PLEIADES_AST_KERNEL) is
    // provided.
    {
        let reference = pleiades_jpl::SnapshotCorpusBackend::from_entries(
            pleiades_jpl::asteroid_reference_corpus().to_vec(),
        );
        let candidate = crate::claims::spk_release_backend();
        let corpus = crate::corpus::asteroid_corpus();
        if let Ok(report) = crate::comparison::compare_backends(&reference, &candidate, &corpus) {
            check_report(
                &report,
                "jpl-spk",
                &candidate.metadata().release_grade_bodies(),
                &mut errors,
            );
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// A dedicated gate that is release-grade evidence for the packaged lunar
/// points, which have no hold-out or `sb441-n373s` rows.
pub(crate) struct LunarPointGate {
    /// The gate's command name, which a claim's evidence source names in
    /// parentheses at its end, e.g. `"… SE_TRUE_NODE (validate-true-node)"`.
    /// In the release battery (`render::cli::NUMERIC_GATES`, run by
    /// `release-smoke` and `release-gate`) the gate reports under the name
    /// without its `validate-` prefix, e.g. `"true-node gate failed"`.
    pub(crate) name: &'static str,
    /// The bodies the gate's corpus compares.
    pub(crate) bodies: &'static [CelestialBody],
}

pub(crate) const LUNAR_POINT_GATES: [LunarPointGate; 3] = [
    LunarPointGate {
        name: "validate-lilith",
        bodies: &[CelestialBody::TrueApogee, CelestialBody::TruePerigee],
    },
    LunarPointGate {
        name: "validate-true-node",
        bodies: &[CelestialBody::TrueNode],
    },
    LunarPointGate {
        name: "validate-mean-lunar-points",
        bodies: &[
            CelestialBody::MeanNode,
            CelestialBody::MeanApogee,
            CelestialBody::MeanPerigee,
        ],
    },
];

/// Whether `claim`'s evidence is a [`LUNAR_POINT_GATES`] gate that compares
/// its body. The exemption follows the evidence, so a lunar point whose claim
/// loses its gate is audited like any other body (issue #236).
fn is_evidenced_by_a_lunar_point_gate(claim: &pleiades_backend::BodyClaim) -> bool {
    let ClaimEvidence::CorpusValidated { source } = &claim.evidence else {
        return false;
    };
    LUNAR_POINT_GATES.iter().any(|gate| {
        source.ends_with(&format!("({})", gate.name)) && gate.bodies.contains(&claim.body)
    })
}

/// Release-grade bodies in `claims` that no corpus block audited.
///
/// A body counts as covered when it was audited against the hold-out rows
/// (`holdout_audited`) or the asteroid rows (`asteroids_audited`), or its
/// evidence names a [`LUNAR_POINT_GATES`] gate that compares it. Anything left
/// has no evidence row and must fail the audit rather than pass silently.
fn unaudited_release_grade_bodies(
    claims: &[pleiades_backend::BodyClaim],
    holdout_audited: &[CelestialBody],
    asteroids_audited: &[CelestialBody],
) -> Vec<CelestialBody> {
    claims
        .iter()
        .filter(|claim| claim.tier == BodyClaimTier::ReleaseGrade)
        .filter(|claim| {
            !holdout_audited.contains(&claim.body)
                && !asteroids_audited.contains(&claim.body)
                && !is_evidenced_by_a_lunar_point_gate(claim)
        })
        .map(|claim| claim.body.clone())
        .collect()
}

/// Compares the packaged backend against `rows` for the claimed `asteroids`.
///
/// Every claimed asteroid must have at least one reference row and appear in the
/// comparison summaries; a missing body is reported as
/// [`ClaimAuditError::DeclaredBodyNotComputable`] rather than skipped, so a
/// release-grade claim can never pass without gate evidence.
fn audit_packaged_asteroids(
    asteroids: &[CelestialBody],
    all_rows: &[pleiades_jpl::SnapshotEntry],
    errors: &mut Vec<ClaimAuditError>,
) {
    let rows: Vec<pleiades_jpl::SnapshotEntry> = all_rows
        .iter()
        .filter(|row| asteroids.contains(&row.body))
        .cloned()
        .collect();
    let reference = pleiades_jpl::SnapshotCorpusBackend::from_entries(rows.clone());
    let candidate = pleiades_data::PackagedDataBackend::default();
    let corpus = crate::corpus::corpus_from_entries(&rows);
    match crate::comparison::compare_backends(&reference, &candidate, &corpus) {
        Ok(report) => {
            let summaries = report.body_summaries();
            for body in asteroids {
                if !summaries.iter().any(|summary| &summary.body == body) {
                    errors.push(ClaimAuditError::DeclaredBodyNotComputable {
                        backend: "pleiades-data".into(),
                        body: body.to_string(),
                    });
                }
            }
            check_report(&report, "pleiades-data", asteroids, errors);
        }
        Err(_) => errors.push(ClaimAuditError::DeclaredBodyNotComputable {
            backend: "pleiades-data".into(),
            body: "<sb441-n373s asteroids>".into(),
        }),
    }
}

/// Checks each release-grade body's measured deltas against its accuracy ceiling.
///
/// Bodies that are not in `release_bodies` are skipped. For each release-grade
/// body, the per-channel maximum corpus delta is converted to the ceiling's
/// units (arcseconds for longitude/latitude, kilometres for distance) and
/// compared against the body's [`accuracy_ceiling`]; any breach is pushed as a
/// [`ClaimAuditError::ReleaseGradeAboveCeiling`].
#[allow(dead_code)]
fn check_report(
    report: &crate::comparison::ComparisonReport,
    backend: &str,
    release_bodies: &[CelestialBody],
    errors: &mut Vec<ClaimAuditError>,
) {
    for summary in report.body_summaries() {
        if !release_bodies.contains(&summary.body) {
            continue;
        }
        let ceiling = accuracy_ceiling(&summary.body);
        if summary.max_longitude_delta_deg * DEG_TO_ARCSEC > ceiling.lon_arcsec {
            errors.push(ClaimAuditError::ReleaseGradeAboveCeiling {
                backend: backend.to_string(),
                body: summary.body.to_string(),
                channel: "longitude".to_string(),
            });
        }
        if summary.max_latitude_delta_deg * DEG_TO_ARCSEC > ceiling.lat_arcsec {
            errors.push(ClaimAuditError::ReleaseGradeAboveCeiling {
                backend: backend.to_string(),
                body: summary.body.to_string(),
                channel: "latitude".to_string(),
            });
        }
        if let Some(delta_au) = summary.max_distance_delta_au {
            if delta_au * AU_TO_KM > ceiling.dist_km {
                errors.push(ClaimAuditError::ReleaseGradeAboveCeiling {
                    backend: backend.to_string(),
                    body: summary.body.to_string(),
                    channel: "distance".to_string(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structural_audit_passes_for_canonical_backends() {
        assert!(audit_structural().is_ok());
    }

    #[test]
    #[ignore = "slow: runs corpus comparison"]
    fn release_grade_bodies_meet_accuracy_ceiling() {
        assert!(super::audit_release_grade_accuracy().is_ok());
    }

    /// Proves the expected-body guard in `audit_release_grade_accuracy` is not
    /// itself vacuous: the intersection of release-grade bodies with holdout rows
    /// must be non-empty, otherwise the packaged-data ceiling check would never
    /// run even on a successful compare.
    #[test]
    fn packaged_data_expected_body_set_is_non_empty() {
        use pleiades_backend::EphemerisBackend;
        use std::collections::HashSet;
        let holdout_bodies: HashSet<pleiades_backend::CelestialBody> =
            pleiades_jpl::production_holdout_corpus()
                .iter()
                .map(|entry| entry.body.clone())
                .collect();
        let expected: Vec<_> = pleiades_data::PackagedDataBackend::default()
            .metadata()
            .release_grade_bodies()
            .into_iter()
            .filter(|b| holdout_bodies.contains(b))
            .collect();
        assert!(
            !expected.is_empty(),
            "expected-body set is empty — the packaged-data ceiling guard would never fire"
        );
    }

    #[test]
    fn structural_audit_flags_release_grade_without_corpus_evidence() {
        use pleiades_backend::{AccuracyClass, BodyClaim, CelestialBody, ClaimEvidence};

        // A ReleaseGrade claim with ClaimEvidence::None must be rejected.
        let bad = BodyClaim::release_grade(
            CelestialBody::Mars,
            AccuracyClass::High,
            ClaimEvidence::None,
        );
        assert!(tier_evidence_consistent(&bad).is_err());

        // A ReleaseGrade claim with ClaimEvidence::AlgorithmicModel must be rejected.
        let bad2 = BodyClaim::release_grade(
            CelestialBody::Mars,
            AccuracyClass::High,
            ClaimEvidence::AlgorithmicModel,
        );
        assert!(tier_evidence_consistent(&bad2).is_err());
    }

    #[test]
    fn packaged_asteroid_claims_are_non_empty_and_have_rows() {
        let claimed: Vec<_> = pleiades_data::packaged_body_claims()
            .into_iter()
            .filter(|claim| {
                matches!(&claim.evidence, ClaimEvidence::CorpusValidated { source } if source == "sb441-n373s")
            })
            .map(|claim| claim.body)
            .collect();
        assert_eq!(claimed.len(), 5);
        for body in &claimed {
            assert!(
                pleiades_jpl::asteroid_reference_corpus()
                    .iter()
                    .any(|row| &row.body == body),
                "no sb441-n373s rows for {body}"
            );
        }
    }

    /// Every release-grade body the packaged backend claims is covered by the
    /// hold-out rows, the asteroid rows or a named dedicated gate (kernel-free).
    #[test]
    fn packaged_release_grade_bodies_all_have_an_evidence_row() {
        let holdout: Vec<CelestialBody> = pleiades_jpl::production_holdout_corpus()
            .iter()
            .map(|entry| entry.body.clone())
            .collect();
        let asteroids: Vec<CelestialBody> = pleiades_data::packaged_body_claims()
            .into_iter()
            .filter(|claim| {
                matches!(&claim.evidence, ClaimEvidence::CorpusValidated { source } if source == "sb441-n373s")
            })
            .map(|claim| claim.body)
            .collect();
        let claims = pleiades_data::PackagedDataBackend::default()
            .metadata()
            .body_claims;
        let missing = unaudited_release_grade_bodies(&claims, &holdout, &asteroids);
        assert!(missing.is_empty(), "no evidence row for {missing:?}");
    }

    fn corpus_claim(body: CelestialBody, source: &str) -> pleiades_backend::BodyClaim {
        pleiades_backend::BodyClaim::release_grade(
            body,
            pleiades_backend::AccuracyClass::High,
            ClaimEvidence::CorpusValidated {
                source: source.to_string(),
            },
        )
    }

    /// A release-grade body whose evidence no block recognises is reported,
    /// a lunar point included: its exemption follows its evidence, not its
    /// name (issue #236).
    #[test]
    fn release_grade_body_with_unrecognised_evidence_is_reported() {
        use pleiades_backend::{AccuracyClass, BodyClaim};
        let claims = vec![
            corpus_claim(CelestialBody::Mars, "some-unaudited-corpus"),
            BodyClaim::release_grade(
                CelestialBody::TrueNode,
                AccuracyClass::High,
                ClaimEvidence::ArtifactValidated,
            ),
        ];
        assert_eq!(
            unaudited_release_grade_bodies(&claims, &[], &[]),
            vec![CelestialBody::Mars, CelestialBody::TrueNode]
        );
    }

    /// A lunar point is exempt only through a gate that covers it: naming
    /// another lunar point's gate, or a gate as a mere substring, is not
    /// evidence.
    #[test]
    fn lunar_point_naming_a_gate_that_does_not_cover_it_is_reported() {
        let claims = vec![
            corpus_claim(
                CelestialBody::TrueNode,
                "Swiss Ephemeris 2.10.03 SE_OSCU_APOG (validate-lilith)",
            ),
            corpus_claim(
                CelestialBody::MeanNode,
                "validate-mean-lunar-points-retired",
            ),
            corpus_claim(
                CelestialBody::TrueApogee,
                "Swiss Ephemeris 2.10.03 SE_OSCU_APOG (validate-lilith)",
            ),
        ];
        assert_eq!(
            unaudited_release_grade_bodies(&claims, &[], &[]),
            vec![CelestialBody::TrueNode, CelestialBody::MeanNode]
        );
    }

    /// The packaged lunar points' own claims each name a gate that covers
    /// them, so none of them is reported.
    #[test]
    fn packaged_lunar_point_claims_name_their_covering_gates() {
        let claims: Vec<_> = pleiades_data::apsis_body_claims()
            .into_iter()
            .chain(pleiades_data::true_node_body_claims())
            .chain(pleiades_data::mean_lunar_point_body_claims())
            .collect();
        assert_eq!(claims.len(), 6);
        assert!(unaudited_release_grade_bodies(&claims, &[], &[]).is_empty());
    }

    /// With rows for Ceres only and a claim of `[Ceres, Pallas]`, the per-body
    /// branch reports Pallas (the empty-rows test only reaches the whole-compare
    /// failure branch).
    #[test]
    fn packaged_asteroid_missing_from_summaries_is_named() {
        let ceres_rows: Vec<_> = pleiades_jpl::asteroid_reference_corpus()
            .iter()
            .filter(|row| row.body == CelestialBody::Ceres)
            .cloned()
            .collect();
        assert!(!ceres_rows.is_empty());
        let mut errors = Vec::new();
        audit_packaged_asteroids(
            &[CelestialBody::Ceres, CelestialBody::Pallas],
            &ceres_rows,
            &mut errors,
        );
        assert!(
            errors.iter().any(|e| matches!(
                e,
                ClaimAuditError::DeclaredBodyNotComputable { body, .. } if body == "Pallas"
            )),
            "got {errors:?}"
        );
        assert!(
            !errors.iter().any(|e| matches!(
                e,
                ClaimAuditError::DeclaredBodyNotComputable { body, .. } if body == "Ceres"
            )),
            "got {errors:?}"
        );
    }

    /// A corpus-validated asteroid with no reference rows must be an error, not
    /// a silent skip.
    #[test]
    fn packaged_asteroid_without_rows_is_reported() {
        let mut errors = Vec::new();
        audit_packaged_asteroids(&[CelestialBody::Ceres], &[], &mut errors);
        assert!(
            errors.iter().any(|e| matches!(
                e,
                ClaimAuditError::DeclaredBodyNotComputable { backend, .. } if backend == "pleiades-data"
            )),
            "got {errors:?}"
        );
    }
}
