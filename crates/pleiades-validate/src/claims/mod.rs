//! Derived, cross-backend release posture.
//!
//! This module is the cutover from the retired global string-matched claim layer
//! (`pleiades-backend::release_body_claims`) to a structural posture derived from
//! the canonical first-party backends' per-body claims. The renderers and the
//! release bundle gate consume [`posture::derived_release_posture`] rather than
//! hand-maintained prose.

pub(crate) mod audit;
pub(crate) mod drift;
pub(crate) mod posture;

pub(crate) use posture::{
    canonical_release_metadata, derived_release_posture, validate_release_posture,
    validated_release_body_claims_summary_line_for_report,
};

pub(crate) use drift::check_claim_drift;

/// Builds a minimal, structurally valid [`pleiades_backend::EphemerisRequest`]
/// for the given body.
///
/// The request uses TT time scale, a J2000 instant (JD 2 451 545.0), and the
/// default geocentric ecliptic tropical mean-geometric shape so that the only
/// reason a backend would reject it is the body itself.  This is used by the
/// structural claim audit to probe whether backends correctly reject bodies
/// they declare as `Unsupported`.
// Called by `audit::audit_structural`; exposed `pub(crate)` for Task 11.
#[allow(dead_code)]
pub(crate) fn sample_request_for(
    body: &pleiades_backend::CelestialBody,
) -> pleiades_backend::EphemerisRequest {
    pleiades_backend::EphemerisRequest::new(
        body.clone(),
        pleiades_backend::Instant::new(
            pleiades_backend::JulianDay::from_days(2_451_545.0),
            pleiades_backend::TimeScale::Tt,
        ),
    )
}

/// Builds the JPL/SPK release backend from environment-provided kernels.
///
/// The only kernel source available to `pleiades-validate` is the environment:
/// `PLEIADES_DE_KERNEL` (planetary DE kernel) and `PLEIADES_AST_KERNEL` (small-body
/// kernel). When neither is set — as in the default build/test environment — an
/// empty backend is built: its `covered_bodies()` is empty and it contributes no
/// claims, so the derived posture stays well-defined (packaged-data still carries
/// the release-grade Pluto/Moon/Eros claims). Kernel-absence for a
/// `ReleaseGrade`-intended body is reported by the claims audit, not here.
pub(crate) fn spk_release_backend() -> pleiades_jpl::SpkBackend {
    // Collect the kernel paths declared via the environment, then fold them into
    // the builder. `add_kernel` consumes the builder by value, so any kernel that
    // fails to load is simply skipped while the successfully-loaded ones are kept.
    let paths: Vec<String> = ["PLEIADES_DE_KERNEL", "PLEIADES_AST_KERNEL"]
        .into_iter()
        .filter_map(|var| std::env::var(var).ok())
        .collect();

    let mut builder = pleiades_jpl::SpkBackend::builder();
    for path in paths {
        match builder.add_kernel(&path) {
            Ok(updated) => builder = updated,
            // `add_kernel` consumes the builder even on error; restart from an
            // empty builder so an unreadable kernel cannot poison construction.
            Err(_) => builder = pleiades_jpl::SpkBackend::builder(),
        }
    }
    builder.build()
}
