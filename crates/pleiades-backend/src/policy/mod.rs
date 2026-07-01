pub(crate) mod apparentness;
pub(crate) mod current;
pub(crate) mod delta_t;
pub(crate) mod frame;
pub(crate) mod native_sidereal;
pub(crate) mod observer;
pub(crate) mod pluto_fallback;
pub(crate) mod request;
pub(crate) mod time_scale;
pub(crate) mod utc;
pub(crate) mod zodiac;

/// Canonical current policy summary text for direct backend time-scale requests.
pub const CURRENT_TIME_SCALE_POLICY_SUMMARY_TEXT: &str =
    "direct backend requests accept TT/TDB; civil UTC/UT1 inputs convert via the pleiades-time crate or caller-supplied offsets; the ephemeris backends carry no internal Delta T or UTC convenience model";

/// Canonical current policy summary text for the shared Delta T posture.
pub const CURRENT_DELTA_T_POLICY_SUMMARY_TEXT: &str =
    "built-in Delta T modeling is now provided by the pleiades-time crate for civil UTC/UT1 inputs over 1900-2100, tagged observed/predicted; direct backend requests still accept TT/TDB";

/// Canonical current policy summary text for the shared UTC-convenience posture.
pub const CURRENT_UTC_CONVENIENCE_POLICY_SUMMARY_TEXT: &str =
    "built-in UTC convenience conversion is now provided by the pleiades-time crate (civil UTC/UT1 to TT/TDB, leap-second-exact UTC, tiered exact/observed/predicted, 1900-2100); direct backends still consume TT/TDB";

/// Canonical current policy summary text for the shared observer posture.
pub const CURRENT_OBSERVER_POLICY_SUMMARY_TEXT: &str =
    "chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; chart-layer topocentric body positions are supported as an opt-in correction (diurnal parallax + diurnal aberration); native-backend topocentric remains unsupported";

/// Canonical current policy summary text for the shared apparentness posture.
pub const CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT: &str =
    "backends remain mean-only and J2000 at the backend boundary; apparent place of date (chart layer, default): light-time + precession-to-date + annual aberration + nutation-in-longitude, release-grade bodies; gravitational light-deflection omitted";

/// Canonical current policy summary text for the shared frame posture.
pub const CURRENT_FRAME_POLICY_SUMMARY_TEXT: &str =
    "ecliptic body positions are the default request shape; at the backend boundary equatorial output is derived via mean-obliquity transforms when supported, while the chart layer reports apparent equatorial of date (true obliquity = mean obliquity + nutation-in-obliquity) for release-grade bodies; supported equatorial precision is bounded by the shared mean-obliquity frame round-trip envelope; native sidereal backend output remains unsupported unless a backend explicitly advertises it";

/// Canonical current policy summary text for the shared native sidereal posture.
pub const CURRENT_NATIVE_SIDEREAL_POLICY_SUMMARY_TEXT: &str =
    "native sidereal backend output remains unsupported unless a backend explicitly advertises it";

/// Canonical current unsupported-modes summary text used by release reporting.
pub const CURRENT_UNSUPPORTED_MODES_SUMMARY_TEXT: &str =
    "built-in UTC convenience remains out of scope; built-in Delta T remains out of scope; native sidereal backend output remains unsupported unless a backend explicitly advertises it";

/// Canonical current policy summary text for the shared zodiac posture.
pub const CURRENT_ZODIAC_POLICY_SUMMARY_TEXT: &str = "tropical only";

/// Canonical current policy summary text for the Pluto fallback posture.
///
/// This line is per-backend: the algorithmic (VSOP87) path treats Pluto as an
/// explicitly approximate fallback, while the packaged-data artifact ships
/// Pluto as release-grade ([`ClaimEvidence::ArtifactValidated`]). The prose is
/// scoped to the fallback path so it cannot be read as a global posture that
/// excludes Pluto from every backend's release-grade claims.
///
/// [`ClaimEvidence::ArtifactValidated`]: crate::ClaimEvidence::ArtifactValidated
pub const CURRENT_PLUTO_FALLBACK_POLICY_SUMMARY_TEXT: &str =
    "Pluto remains an explicitly approximate fallback on the algorithmic (VSOP87) path; the packaged-data artifact ships Pluto as release-grade";

// Re-exports of all public types from submodules:
pub use apparentness::{ApparentnessPolicySummary, ApparentnessPolicySummaryValidationError};
pub use delta_t::{DeltaTPolicySummary, DeltaTPolicySummaryValidationError};
pub use frame::{
    FramePolicySummary, FramePolicySummaryValidationError, FrameTreatmentSummary,
    FrameTreatmentSummaryValidationError,
};
pub use native_sidereal::{
    NativeSiderealPolicySummary, NativeSiderealPolicySummaryValidationError,
};
pub use observer::{ObserverPolicySummary, ObserverPolicySummaryValidationError};
pub use pluto_fallback::{PlutoFallbackSummary, PlutoFallbackSummaryValidationError};
pub use request::{RequestPolicySummary, RequestPolicySummaryValidationError};
pub use time_scale::{TimeScalePolicySummary, TimeScalePolicySummaryValidationError};
pub use utc::{UtcConveniencePolicySummary, UtcConveniencePolicySummaryValidationError};
pub use zodiac::{ZodiacPolicySummary, ZodiacPolicySummaryValidationError};

#[cfg(test)]
#[path = "../policy_tests.rs"]
mod tests;
