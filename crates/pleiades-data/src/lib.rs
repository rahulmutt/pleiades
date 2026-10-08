//! Packaged compressed ephemeris backend for the default 1900-2100 range.
//!
//! Wider coverage is available as an opt-in: regenerate the artifact over a
//! custom window with the `generate-artifact <kernel> --asteroid-kernel <kernel>
//! --out <path> [--start --end]` CLI subcommand.
//!
//! This crate ships a packaged artifact backed by the `pleiades-compression`
//! codec, validated against a deterministic binary fixture. The backend serves
//! the Sun, the Moon, Mercury through Pluto, and five asteroids (Ceres, Pallas,
//! Juno, Vesta and `asteroid:433-Eros`), and falls back to other providers when
//! callers request bodies outside that packaged slice.
//!
//! The asteroids are densely fitted (heliocentric) from the JPL `sb441-n373s`
//! small-body kernel and served on every date in 1900-2100 (issue #201); their
//! accuracy is gated against the `sb441-n373s` rows of `asteroid_reference.csv`.
//!
//! The packaged artifact stores J2000 ecliptic coordinates directly,
//! reconstructs equatorial coordinates from the stored channels and
//! J2000 mean-obliquity transform when requested. The checked-in artifact
//! carries no residual correction segments. A maintainer-facing regeneration
//! helper rebuilds the checked-in fixture from the de440 and `sb441-n373s`
//! kernels without introducing any native tooling. When the
//! `packaged-artifact-path` feature is enabled, callers can also load an
//! explicit artifact file for larger or externally distributed packaged
//! datasets. See `docs/time-observer-policy.md`
//! for the explicit packaged request/lookup-epoch policy, and
//! `spec/data-compression.md` for the stored-vs-derived artifact contract.
//!
//! # Examples
//!
//! ```
//! use pleiades_backend::{CelestialBody, Instant, JulianDay, TimeScale};
//! use pleiades_data::{packaged_backend, packaged_lookup};
//!
//! let _backend = packaged_backend();
//! let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
//! let sun = packaged_lookup(&CelestialBody::Sun, instant)
//!     .expect("Sun should be in the packaged artifact");
//!
//! assert!(sun.distance_au.is_some());
//! ```

#![forbid(unsafe_code)]

use std::sync::OnceLock;

use pleiades_backend::{CelestialBody, CustomBodyId};
use pleiades_jpl::SnapshotEntry;

mod accuracy_baseline;
mod backend;
mod coverage;
mod data;
mod lookup;
mod regenerate;
pub mod thresholds;

pub use accuracy_baseline::{
    accuracy_baseline_against, packaged_artifact_accuracy_baseline, BodyChannelError,
};
pub use backend::*;
pub use coverage::*;
pub use data::*;
pub use lookup::*;
pub use regenerate::*;

// Test-only re-exports: bring pub(crate) items into lib.rs scope so that
// `use super::*` in the tests module can pick them up.
#[cfg(test)]
pub(crate) use coverage::{
    packaged_artifact_body_cadence, packaged_artifact_fit_outlier_sample_fractions,
    packaged_artifact_fit_sample_fractions, packaged_artifact_fit_sample_fractions_for_body,
    PackagedArtifactBodyCadence,
};
#[cfg(test)]
pub(crate) use data::PACKAGED_ARTIFACT_FIXTURE;
#[cfg(test)]
pub(crate) use lookup::{
    validate_packaged_artifact_access_summary_line, validate_packaged_artifact_storage_profile,
    validate_packaged_artifact_storage_summary_line,
    validate_packaged_frame_treatment_summary_line,
};
#[cfg(test)]
pub(crate) use regenerate::{
    coordinates, packaged_artifact_segment_validation_fractions_for_body,
    PACKAGED_ARTIFACT_DENSE_VALIDATION_SAMPLE_FRACTIONS,
    PACKAGED_ARTIFACT_MEDIUM_VALIDATION_SAMPLE_FRACTIONS,
};
// External types needed by tests via `use super::*`
#[cfg(test)]
pub(crate) use pleiades_backend::{
    Apparentness, BackendFamily, CoordinateFrame, EclipticCoordinates, EphemerisBackend,
    EphemerisErrorKind, EphemerisRequest, Instant, JulianDay, QualityAnnotation, TimeRange,
    TimeScale, ZodiacMode,
};
#[cfg(test)]
pub(crate) use pleiades_compression::{
    ArtifactOutput, ArtifactProfile, ChannelKind, CompressedArtifact, EndianPolicy, Segment,
    SpeedPolicy,
};
#[cfg(test)]
pub(crate) use pleiades_jpl::{
    production_generation_source_summary_for_report, production_holdout_corpus, reference_snapshot,
};

const PACKAGE_NAME: &str = "pleiades-data";
const ARTIFACT_LABEL: &str = "stage-5 packaged-data draft";
const ARTIFACT_PROFILE_ID: &str = "pleiades-packaged-artifact-profile/stage-5-draft";
/// How every packaged body is fit (`regenerate::fit_dense_body_artifact`),
/// shared by the artifact header and the generation-policy note. Every
/// packaged body uses the same degree and sample count (pinned by a test).
pub(crate) fn packaged_artifact_dense_fit_method_text() -> &'static str {
    static METHOD: OnceLock<String> = OnceLock::new();
    METHOD.get_or_init(|| {
        format!(
            "one quantized degree-{} least-squares polynomial per channel (unwrapped longitude, latitude, distance) on each fixed per-body fitting span, from {} evenly spaced kernel samples, with the planets and asteroids stored heliocentric and recombined with the geocentric Sun at lookup, and no point segments, residual correction channels or adaptive subdivision",
            coverage::fitting_degree(&CelestialBody::Sun),
            coverage::fitting_within_span_sample_count(&CelestialBody::Sun),
        )
    })
    .as_str()
}

pub(crate) fn packaged_artifact_generation_policy_note_text() -> &'static str {
    static NOTE: OnceLock<String> = OnceLock::new();
    NOTE.get_or_init(|| {
        format!(
            "every packaged body is fit densely over the coverage window: {}",
            packaged_artifact_dense_fit_method_text()
        )
    })
    .as_str()
}

pub(crate) fn packaged_artifact_source_text() -> &'static str {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE.get_or_init(|| {
        format!(
            "Dense per-body polynomial fits over the default 1900-2100 coverage window: the Sun, Moon and Mercury through Pluto from the JPL de440 kernel, and Ceres, Pallas, Juno, Vesta and asteroid:433-Eros from the JPL sb441-n373s small-body kernel; {}.",
            packaged_artifact_dense_fit_method_text()
        )
    })
    .as_str()
}

pub(crate) const PACKAGED_BASE_BODIES: [CelestialBody; 10] = [
    CelestialBody::Sun,
    CelestialBody::Moon,
    CelestialBody::Mercury,
    CelestialBody::Venus,
    CelestialBody::Mars,
    CelestialBody::Jupiter,
    CelestialBody::Saturn,
    CelestialBody::Uranus,
    CelestialBody::Neptune,
    CelestialBody::Pluto,
];

const PACKAGED_REFERENCE_EPOCH_JD: f64 = 2_451_545.0;

pub(crate) fn packaged_bodies() -> &'static [CelestialBody] {
    static BODIES: OnceLock<Vec<CelestialBody>> = OnceLock::new();
    BODIES.get_or_init(|| {
        let mut bodies = PACKAGED_BASE_BODIES.to_vec();
        bodies.extend(packaged_asteroids().iter().cloned());
        bodies
    })
}

/// The asteroids the artifact fits densely from the JPL `sb441-n373s`
/// kernel, in artifact order (issue #201).
pub(crate) fn packaged_asteroids() -> &'static [CelestialBody] {
    static BODIES: OnceLock<Vec<CelestialBody>> = OnceLock::new();
    BODIES.get_or_init(|| {
        vec![
            CelestialBody::Ceres,
            CelestialBody::Pallas,
            CelestialBody::Juno,
            CelestialBody::Vesta,
            packaged_eros().clone(),
        ]
    })
}

/// `asteroid:433-Eros`, the one packaged asteroid with no [`CelestialBody`]
/// variant of its own.
pub(crate) fn packaged_eros() -> &'static CelestialBody {
    static EROS: OnceLock<CelestialBody> = OnceLock::new();
    EROS.get_or_init(|| CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")))
}

/// Returns the per-body release claims for the packaged artifact. The planets,
/// Sun and Moon are validated inside the artifact build against the hold-out
/// corpus; the asteroids against the JPL `sb441-n373s` rows of
/// `asteroid_reference.csv` (issue #201).
pub fn packaged_body_claims() -> Vec<pleiades_backend::BodyClaim> {
    use pleiades_backend::{AccuracyClass, BodyClaim, ClaimEvidence};
    let asteroids = packaged_asteroids();
    packaged_bodies()
        .iter()
        .cloned()
        .map(|body| {
            let evidence = if asteroids.contains(&body) {
                ClaimEvidence::CorpusValidated {
                    source: "sb441-n373s".to_string(),
                }
            } else {
                ClaimEvidence::ArtifactValidated
            };
            BodyClaim::release_grade(body, AccuracyClass::High, evidence)
        })
        .collect()
}

/// Release claims for the derived osculating lunar apsides (True Apogee /
/// Perigee). These are computed from the packaged Moon state at lookup and
/// validated against the Swiss Ephemeris `SE_OSCU_APOG` corpus by the
/// `validate-lilith` gate, so their evidence is `CorpusValidated`.
pub fn apsis_body_claims() -> Vec<pleiades_backend::BodyClaim> {
    use pleiades_backend::{AccuracyClass, BodyClaim, ClaimEvidence};
    let source = "Swiss Ephemeris 2.10.03 SE_OSCU_APOG (validate-lilith)".to_string();
    vec![
        BodyClaim::release_grade(
            CelestialBody::TrueApogee,
            AccuracyClass::High,
            ClaimEvidence::CorpusValidated {
                source: source.clone(),
            },
        ),
        BodyClaim::release_grade(
            CelestialBody::TruePerigee,
            AccuracyClass::High,
            ClaimEvidence::CorpusValidated { source },
        ),
    ]
}

/// Release claim for the derived osculating lunar ascending node
/// (`TrueNode`). Computed from the packaged Moon state at lookup (formed in
/// the mean ecliptic of date, emitted in J2000) and validated against the
/// Swiss Ephemeris `SE_TRUE_NODE` corpus by the `validate-true-node` gate, so
/// its evidence is `CorpusValidated`. Supersedes, in the routed chart chain,
/// the `pleiades-elp` Meeus periodic-term approximation (issue #58).
pub fn true_node_body_claims() -> Vec<pleiades_backend::BodyClaim> {
    use pleiades_backend::{AccuracyClass, BodyClaim, ClaimEvidence};
    vec![BodyClaim::release_grade(
        CelestialBody::TrueNode,
        AccuracyClass::High,
        ClaimEvidence::CorpusValidated {
            source: "Swiss Ephemeris 2.10.03 SE_TRUE_NODE (validate-true-node)".to_string(),
        },
    )]
}

/// Release claims for the mean lunar points (`MeanNode`, `MeanApogee`,
/// `MeanPerigee`). Formed at lookup from the mean lunar elements in the mean
/// ecliptic of date (the apsides on the inclined mean orbit, as Swiss
/// Ephemeris' `SE_MEAN_APOG` is), emitted in J2000, and validated against the
/// Swiss Ephemeris `SE_MEAN_NODE` / `SE_MEAN_APOG` corpus by the
/// `validate-mean-lunar-points` gate. Supersede, in the routed chart chain,
/// the `pleiades-elp` mean-element channels (issue #90).
pub fn mean_lunar_point_body_claims() -> Vec<pleiades_backend::BodyClaim> {
    use pleiades_backend::{AccuracyClass, BodyClaim, ClaimEvidence};
    let source = "Swiss Ephemeris 2.10.03 SE_MEAN_NODE / SE_MEAN_APOG (validate-mean-lunar-points)";
    [
        CelestialBody::MeanNode,
        CelestialBody::MeanApogee,
        CelestialBody::MeanPerigee,
    ]
    .into_iter()
    .map(|body| {
        BodyClaim::release_grade(
            body,
            AccuracyClass::High,
            ClaimEvidence::CorpusValidated {
                source: source.to_string(),
            },
        )
    })
    .collect()
}

pub(crate) fn packaged_reference_entry_for_body(
    snapshot: &[SnapshotEntry],
    body: &CelestialBody,
) -> Option<SnapshotEntry> {
    snapshot
        .iter()
        .find(|entry| {
            entry.body == *body
                && (entry.epoch.julian_day.days() - PACKAGED_REFERENCE_EPOCH_JD).abs()
                    < f64::EPSILON
        })
        .cloned()
        .or_else(|| snapshot.iter().find(|entry| entry.body == *body).cloned())
}

pub(crate) const AU_IN_KM: f64 = 149_597_870.7;

/// Returns the canonical package name for this crate.
pub const fn package_name() -> &'static str {
    PACKAGE_NAME
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
