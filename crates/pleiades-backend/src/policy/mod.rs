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
    "direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model";

/// Canonical current policy summary text for the shared Delta T posture.
pub const CURRENT_DELTA_T_POLICY_SUMMARY_TEXT: &str =
    "built-in Delta T modeling remains out of scope; UTC/UT1 inputs require caller-supplied conversion helpers";

/// Canonical current policy summary text for the shared UTC-convenience posture.
pub const CURRENT_UTC_CONVENIENCE_POLICY_SUMMARY_TEXT: &str =
    "built-in UTC convenience conversion remains out of scope; callers must supply TT/TDB offsets explicitly";

/// Canonical current policy summary text for the shared observer posture.
pub const CURRENT_OBSERVER_POLICY_SUMMARY_TEXT: &str =
    "chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported";

/// Canonical current policy summary text for the shared apparentness posture.
pub const CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT: &str =
    "current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support";

/// Canonical current policy summary text for the shared frame posture.
pub const CURRENT_FRAME_POLICY_SUMMARY_TEXT: &str =
    "ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; supported equatorial precision is bounded by the shared mean-obliquity frame round-trip envelope; native sidereal backend output remains unsupported unless a backend explicitly advertises it";

/// Canonical current policy summary text for the shared native sidereal posture.
pub const CURRENT_NATIVE_SIDEREAL_POLICY_SUMMARY_TEXT: &str =
    "native sidereal backend output remains unsupported unless a backend explicitly advertises it";

/// Canonical current unsupported-modes summary text used by release reporting.
pub const CURRENT_UNSUPPORTED_MODES_SUMMARY_TEXT: &str =
    "built-in UTC convenience remains out of scope; built-in Delta T remains out of scope; topocentric body positions remain unsupported; apparent-place corrections are rejected unless a backend explicitly advertises support; native sidereal backend output remains unsupported unless a backend explicitly advertises it";

/// Canonical current policy summary text for the shared zodiac posture.
pub const CURRENT_ZODIAC_POLICY_SUMMARY_TEXT: &str = "tropical only";

/// Canonical current policy summary text for the Pluto fallback posture.
pub const CURRENT_PLUTO_FALLBACK_POLICY_SUMMARY_TEXT: &str =
    "Pluto remains an explicitly approximate fallback; release-grade major-body claims exclude Pluto";

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
