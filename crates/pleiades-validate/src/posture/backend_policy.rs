//! Backend policy/report prose relocated from `pleiades-backend`
//! (report-surface relocation program). This module hosts the
//! policy-summary structs and the report-rendering functions; the
//! backend crate keeps the request/observer/zodiac contract validators.

// This is a verbatim relocation of a report-prose API surface: some summary
// constructors and report wrappers are exercised only by this module's own
// tests or are part of the surface without a current in-crate caller.
#![allow(dead_code)]

use core::fmt;

use pleiades_backend::FrameTreatmentSummary;
use pleiades_types::ZodiacMode;

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

fn format_display_list<T: fmt::Display>(values: &[T]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Returns the current shared time-scale policy used by validation and reports.
pub const fn current_time_scale_policy_summary() -> TimeScalePolicySummary {
    TimeScalePolicySummary::new(CURRENT_TIME_SCALE_POLICY_SUMMARY_TEXT)
}

/// Returns the current shared Delta T policy used by validation and reports.
pub const fn current_delta_t_policy_summary() -> DeltaTPolicySummary {
    DeltaTPolicySummary::new(CURRENT_DELTA_T_POLICY_SUMMARY_TEXT)
}

/// Returns the current shared UTC-convenience policy used by validation and reports.
pub const fn current_utc_convenience_policy_summary() -> UtcConveniencePolicySummary {
    UtcConveniencePolicySummary::new(CURRENT_UTC_CONVENIENCE_POLICY_SUMMARY_TEXT)
}

/// Returns the UTC-convenience policy posture used by validation and release reporting.
pub const fn utc_convenience_policy_summary_for_report() -> UtcConveniencePolicySummary {
    current_utc_convenience_policy_summary()
}

/// Returns the validated UTC-convenience policy summary line used by validation and release reporting.
pub fn validated_utc_convenience_policy_summary_for_report() -> String {
    match current_utc_convenience_policy_summary().validated_summary_line() {
        Ok(summary) => summary.to_string(),
        Err(error) => format!("UTC convenience policy unavailable ({error})"),
    }
}

/// Returns the current shared observer policy used by validation and reports.
pub const fn current_observer_policy_summary() -> ObserverPolicySummary {
    ObserverPolicySummary::new(CURRENT_OBSERVER_POLICY_SUMMARY_TEXT)
}

/// Returns the current shared apparentness policy used by validation and reports.
pub const fn current_apparentness_policy_summary() -> ApparentnessPolicySummary {
    ApparentnessPolicySummary::new(CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT)
}

/// Returns the current shared request-policy posture used by validation and reports.
pub const fn current_request_policy_summary() -> RequestPolicySummary {
    RequestPolicySummary {
        time_scale: CURRENT_TIME_SCALE_POLICY_SUMMARY_TEXT,
        observer: CURRENT_OBSERVER_POLICY_SUMMARY_TEXT,
        apparentness: CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT,
        frame: CURRENT_FRAME_POLICY_SUMMARY_TEXT,
    }
}

/// Returns the current shared frame-policy posture used by validation and reports.
pub const fn current_frame_policy_summary() -> FramePolicySummary {
    FramePolicySummary::new(CURRENT_FRAME_POLICY_SUMMARY_TEXT)
}

/// Returns the current native sidereal policy used by validation and reports.
pub const fn current_native_sidereal_policy_summary() -> NativeSiderealPolicySummary {
    NativeSiderealPolicySummary::new(CURRENT_NATIVE_SIDEREAL_POLICY_SUMMARY_TEXT)
}

/// Returns the native sidereal policy posture used by validation and release reporting.
pub const fn native_sidereal_policy_summary_for_report() -> NativeSiderealPolicySummary {
    current_native_sidereal_policy_summary()
}

/// Returns the validated native sidereal policy summary line used by validation and release reporting.
pub fn validated_native_sidereal_policy_summary_for_report() -> String {
    match current_native_sidereal_policy_summary().validated_summary_line() {
        Ok(summary) => summary.to_string(),
        Err(error) => format!("native sidereal policy unavailable ({error})"),
    }
}

/// Returns the current zodiac posture used by validation and reports.
pub const fn current_zodiac_policy_summary() -> ZodiacPolicySummary {
    ZodiacPolicySummary::new(CURRENT_ZODIAC_POLICY_SUMMARY_TEXT)
}

/// Returns the validated zodiac policy summary line used by validation and release reporting.
pub fn validated_zodiac_policy_summary_for_report() -> String {
    match current_zodiac_policy_summary().validated_summary_line() {
        Ok(summary) => summary.to_string(),
        Err(error) => format!("zodiac policy unavailable ({error})"),
    }
}

/// Returns the current Pluto fallback posture used by validation and reports.
pub const fn current_pluto_fallback_summary() -> PlutoFallbackSummary {
    PlutoFallbackSummary::new(CURRENT_PLUTO_FALLBACK_POLICY_SUMMARY_TEXT)
}

/// Returns the Pluto fallback posture used by validation and release reporting.
pub const fn pluto_fallback_summary_for_report() -> PlutoFallbackSummary {
    current_pluto_fallback_summary()
}

/// Returns the validated Pluto fallback summary line used by validation and release reporting.
pub fn validated_pluto_fallback_summary_line_for_report(
) -> Result<&'static str, PlutoFallbackSummaryValidationError> {
    current_pluto_fallback_summary().validated_summary_line()
}

/// Returns the request-policy posture used by validation and release reporting.
pub const fn request_policy_summary_for_report() -> RequestPolicySummary {
    current_request_policy_summary()
}

/// Returns the validated request-policy summary line used by validation and release reporting.
pub fn validated_request_policy_summary_for_report() -> String {
    match current_request_policy_summary().validated_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("request policy unavailable ({error})"),
    }
}

/// Returns the validated request-semantics summary line used by validation and release reporting.
///
/// This is a backend-layer alias for [`validated_request_policy_summary_for_report()`]
/// so callers that use the request-semantics vocabulary can share the same
/// guarded compact line without reinterpreting the compact report wording.
pub fn validated_request_semantics_summary_for_report() -> String {
    validated_request_policy_summary_for_report()
}

/// Returns the request-semantics posture used by validation and release reporting.
///
/// This is a backend-layer alias for [`request_policy_summary_for_report()`]
/// so callers that use the request-semantics vocabulary can share the same
/// typed summary without reinterpreting the compact report wording.
pub const fn request_semantics_summary_for_report() -> RequestPolicySummary {
    request_policy_summary_for_report()
}

/// Returns the observer-policy posture used by validation and release reporting.
pub const fn observer_policy_summary_for_report() -> ObserverPolicySummary {
    current_observer_policy_summary()
}

/// Returns the validated observer-policy summary line used by validation and release reporting.
pub fn validated_observer_policy_summary_for_report() -> String {
    match current_observer_policy_summary().validated_summary_line() {
        Ok(summary) => summary.to_string(),
        Err(error) => format!("observer policy unavailable ({error})"),
    }
}

/// Returns the apparentness-policy posture used by validation and release reporting.
pub const fn apparentness_policy_summary_for_report() -> ApparentnessPolicySummary {
    current_apparentness_policy_summary()
}

/// Returns the validated apparentness-policy summary line used by validation and release reporting.
pub fn validated_apparentness_policy_summary_for_report() -> String {
    match current_apparentness_policy_summary().validated_summary_line() {
        Ok(summary) => summary.to_string(),
        Err(error) => format!("apparentness policy unavailable ({error})"),
    }
}

/// Returns the compact report wording for the current time-scale policy.
pub const fn time_scale_policy_summary_for_report() -> TimeScalePolicySummary {
    current_time_scale_policy_summary()
}

/// Returns the validated compact time-scale policy summary line used by validation and release reporting.
pub fn validated_time_scale_policy_summary_for_report() -> String {
    match current_time_scale_policy_summary().validated_summary_line() {
        Ok(summary) => summary.to_string(),
        Err(error) => format!("time-scale policy unavailable ({error})"),
    }
}

/// Returns the compact report wording for the current Delta T policy.
pub const fn delta_t_policy_summary_for_report() -> DeltaTPolicySummary {
    current_delta_t_policy_summary()
}

/// Returns the validated compact Delta T policy summary line used by validation and release reporting.
pub fn validated_delta_t_policy_summary_for_report() -> String {
    match current_delta_t_policy_summary().validated_summary_line() {
        Ok(summary) => summary.to_string(),
        Err(error) => format!("Delta T policy unavailable ({error})"),
    }
}

/// Returns the compact report wording for the current frame policy.
pub const fn frame_policy_summary_for_report() -> &'static str {
    current_frame_policy_summary().summary_line()
}

/// Returns the compact report wording for the current frame policy after validation.
pub fn validated_frame_policy_summary_for_report() -> String {
    match current_frame_policy_summary().validated_summary_line() {
        Ok(summary) => summary.to_string(),
        Err(error) => format!("frame policy unavailable ({error})"),
    }
}

/// Returns the compact typed frame-policy posture for reporting.
pub const fn frame_policy_summary_details() -> FramePolicySummary {
    current_frame_policy_summary()
}

/// Returns the compact typed frame-treatment posture for reporting.
pub const fn frame_treatment_summary_for_report() -> FrameTreatmentSummary {
    FrameTreatmentSummary::new(current_request_policy_summary().frame)
}

/// Returns the current frame-treatment posture after validation.
pub fn validated_frame_treatment_summary_for_report() -> String {
    match frame_treatment_summary_for_report().validated_summary_line() {
        Ok(summary) => summary.to_string(),
        Err(error) => format!("frame treatment unavailable ({error})"),
    }
}

/// Formats the zodiac-mode policy shared by the current first-party backends.
pub fn zodiac_policy_summary_for_report(supported_zodiac_modes: &[ZodiacMode]) -> String {
    if supported_zodiac_modes.len() == 1 && supported_zodiac_modes[0] == ZodiacMode::Tropical {
        "tropical only".to_string()
    } else {
        format!(
            "zodiac modes=[{}]",
            format_display_list(supported_zodiac_modes)
        )
    }
}

/// Compact summary of the current shared apparentness policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ApparentnessPolicySummary {
    summary: &'static str,
}

/// Validation error for the shared apparentness-policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ApparentnessPolicySummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
    /// The summary text no longer matches the current canonical posture.
    CurrentPolicyOutOfSync,
}

impl fmt::Display for ApparentnessPolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("apparentness policy summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("apparentness policy summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => {
                f.write_str("apparentness policy summary contains a line break")
            }
            Self::CurrentPolicyOutOfSync => {
                f.write_str("apparentness policy summary is out of sync with the current posture")
            }
        }
    }
}

impl std::error::Error for ApparentnessPolicySummaryValidationError {}

impl ApparentnessPolicySummary {
    /// Creates a new apparentness policy summary from a backend-owned note.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the apparentness policy posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns the current shared apparentness policy posture.
    pub const fn current() -> Self {
        Self::new(CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT)
    }

    /// Returns `Ok(())` when the summary still contains the current canonical line.
    pub fn validate(&self) -> Result<(), ApparentnessPolicySummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(ApparentnessPolicySummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(ApparentnessPolicySummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(ApparentnessPolicySummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT {
            Err(ApparentnessPolicySummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    /// Returns the compact summary line after validating the cached prose.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, ApparentnessPolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ApparentnessPolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

/// Compact summary of the current shared Delta T policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DeltaTPolicySummary {
    summary: &'static str,
}

/// Validation error for the shared Delta T policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeltaTPolicySummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
    /// The summary text no longer matches the current canonical posture.
    CurrentPolicyOutOfSync,
}

impl fmt::Display for DeltaTPolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("Delta T policy summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("Delta T policy summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => f.write_str("Delta T policy summary contains a line break"),
            Self::CurrentPolicyOutOfSync => {
                f.write_str("Delta T policy summary is out of sync with the current posture")
            }
        }
    }
}

impl std::error::Error for DeltaTPolicySummaryValidationError {}

impl DeltaTPolicySummary {
    /// Creates a new Delta T policy summary from a backend-owned note.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the Delta T policy posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns the current shared Delta T policy posture.
    pub const fn current() -> Self {
        Self::new(CURRENT_DELTA_T_POLICY_SUMMARY_TEXT)
    }

    /// Returns `Ok(())` when the summary still contains the current canonical line.
    pub fn validate(&self) -> Result<(), DeltaTPolicySummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(DeltaTPolicySummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(DeltaTPolicySummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(DeltaTPolicySummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != CURRENT_DELTA_T_POLICY_SUMMARY_TEXT {
            Err(DeltaTPolicySummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    /// Returns the compact summary line after validating the cached prose.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, DeltaTPolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for DeltaTPolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

/// Compact summary of the current native sidereal policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NativeSiderealPolicySummary {
    summary: &'static str,
}

/// Validation error for the current native sidereal policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NativeSiderealPolicySummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
    /// The summary text no longer matches the current native sidereal posture.
    CurrentPolicyOutOfSync,
}

impl fmt::Display for NativeSiderealPolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("native sidereal policy summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("native sidereal policy summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => {
                f.write_str("native sidereal policy summary contains a line break")
            }
            Self::CurrentPolicyOutOfSync => f.write_str(
                "native sidereal policy summary is out of sync with the current posture",
            ),
        }
    }
}

impl std::error::Error for NativeSiderealPolicySummaryValidationError {}

impl NativeSiderealPolicySummary {
    /// Creates a new native sidereal policy summary from a backend-owned note.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the native sidereal policy posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns the current native sidereal policy posture.
    pub const fn current() -> Self {
        Self::new(CURRENT_NATIVE_SIDEREAL_POLICY_SUMMARY_TEXT)
    }

    /// Returns `Ok(())` when the summary still contains the current canonical line.
    pub fn validate(&self) -> Result<(), NativeSiderealPolicySummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(NativeSiderealPolicySummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(NativeSiderealPolicySummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(NativeSiderealPolicySummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != CURRENT_NATIVE_SIDEREAL_POLICY_SUMMARY_TEXT {
            Err(NativeSiderealPolicySummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    /// Returns the compact summary line after validating the cached prose.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, NativeSiderealPolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for NativeSiderealPolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

/// Compact summary of the current shared observer policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObserverPolicySummary {
    summary: &'static str,
}

/// Validation error for the shared observer-policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ObserverPolicySummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
    /// The summary text no longer matches the current canonical posture.
    CurrentPolicyOutOfSync,
}

impl fmt::Display for ObserverPolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("observer policy summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("observer policy summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => f.write_str("observer policy summary contains a line break"),
            Self::CurrentPolicyOutOfSync => {
                f.write_str("observer policy summary is out of sync with the current posture")
            }
        }
    }
}

impl std::error::Error for ObserverPolicySummaryValidationError {}

impl ObserverPolicySummary {
    /// Creates a new observer policy summary from a backend-owned note.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the observer policy posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns the current shared observer policy posture.
    pub const fn current() -> Self {
        Self::new(CURRENT_OBSERVER_POLICY_SUMMARY_TEXT)
    }

    /// Returns `Ok(())` when the summary still contains the current canonical line.
    pub fn validate(&self) -> Result<(), ObserverPolicySummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(ObserverPolicySummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(ObserverPolicySummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(ObserverPolicySummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != CURRENT_OBSERVER_POLICY_SUMMARY_TEXT {
            Err(ObserverPolicySummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    /// Returns the compact summary line after validating the cached prose.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, ObserverPolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ObserverPolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

/// Compact summary of the current Pluto fallback posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlutoFallbackSummary {
    summary: &'static str,
}

/// Validation error for the current Pluto fallback summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlutoFallbackSummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
    /// The summary text no longer matches the current Pluto fallback posture.
    CurrentPolicyOutOfSync,
}

impl fmt::Display for PlutoFallbackSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("Pluto fallback summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("Pluto fallback summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => f.write_str("Pluto fallback summary contains a line break"),
            Self::CurrentPolicyOutOfSync => {
                f.write_str("Pluto fallback summary is out of sync with the current posture")
            }
        }
    }
}

impl std::error::Error for PlutoFallbackSummaryValidationError {}

impl PlutoFallbackSummary {
    /// Creates a new Pluto fallback summary from backend-owned prose.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the Pluto fallback posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns the current Pluto fallback posture.
    pub const fn current() -> Self {
        Self::new(CURRENT_PLUTO_FALLBACK_POLICY_SUMMARY_TEXT)
    }

    /// Returns `Ok(())` when the summary still contains the current canonical line.
    pub fn validate(&self) -> Result<(), PlutoFallbackSummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(PlutoFallbackSummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(PlutoFallbackSummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(PlutoFallbackSummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != CURRENT_PLUTO_FALLBACK_POLICY_SUMMARY_TEXT {
            Err(PlutoFallbackSummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    /// Returns the compact summary line after validating the cached prose.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, PlutoFallbackSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PlutoFallbackSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

/// Compact summary of the current shared request-policy posture.
///
/// # Example
///
/// ```ignore
/// use pleiades_backend::RequestPolicySummary;
///
/// let summary = RequestPolicySummary::current();
/// assert_eq!(summary.to_string(), summary.summary_line());
/// assert!(summary.summary_line().contains("time-scale="));
/// assert!(summary.summary_line().contains("observer="));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RequestPolicySummary {
    /// Time-scale policy wording.
    pub time_scale: &'static str,
    /// Observer policy wording.
    pub observer: &'static str,
    /// Apparentness policy wording.
    pub apparentness: &'static str,
    /// Frame policy wording.
    pub frame: &'static str,
}

impl RequestPolicySummary {
    /// Returns the current shared request-policy posture.
    pub const fn current() -> Self {
        Self {
            time_scale: CURRENT_TIME_SCALE_POLICY_SUMMARY_TEXT,
            observer: CURRENT_OBSERVER_POLICY_SUMMARY_TEXT,
            apparentness: CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT,
            frame: CURRENT_FRAME_POLICY_SUMMARY_TEXT,
        }
    }

    /// Returns a compact one-line rendering of the shared request-policy posture.
    pub fn summary_line(&self) -> String {
        format!(
            "time-scale={}; observer={}; apparentness={}; frame={}",
            self.time_scale, self.observer, self.apparentness, self.frame
        )
    }

    /// Returns the compact summary line after validating the cached prose.
    pub fn validated_summary_line(&self) -> Result<String, RequestPolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for RequestPolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error for the shared request-policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RequestPolicySummaryValidationError {
    /// A summary field was blank or whitespace-only.
    BlankField {
        /// Name of the offending summary field.
        field: &'static str,
    },
    /// A summary field had surrounding whitespace.
    WhitespacePaddedField {
        /// Name of the offending summary field.
        field: &'static str,
    },
    /// A summary field contained an embedded line break.
    EmbeddedLineBreak {
        /// Name of the offending summary field.
        field: &'static str,
    },
    /// A summary field is out of sync with the current request-policy posture.
    FieldOutOfSync {
        /// Name of the summary field that drifted from the current posture.
        field: &'static str,
    },
}

impl fmt::Display for RequestPolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankField { field } => {
                write!(f, "the request-policy summary field `{field}` is blank")
            }
            Self::WhitespacePaddedField { field } => write!(
                f,
                "the request-policy summary field `{field}` has surrounding whitespace"
            ),
            Self::EmbeddedLineBreak { field } => write!(
                f,
                "the request-policy summary field `{field}` contains a line break"
            ),
            Self::FieldOutOfSync { field } => write!(
                f,
                "the request-policy summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for RequestPolicySummaryValidationError {}

impl RequestPolicySummary {
    /// Returns `Ok(())` when the shared request-policy wording still matches the current posture.
    pub fn validate(&self) -> Result<(), RequestPolicySummaryValidationError> {
        let current = Self::current();
        for (field, value, expected) in [
            ("time_scale", self.time_scale, current.time_scale),
            ("observer", self.observer, current.observer),
            ("apparentness", self.apparentness, current.apparentness),
            ("frame", self.frame, current.frame),
        ] {
            if value.trim().is_empty() {
                return Err(RequestPolicySummaryValidationError::BlankField { field });
            }
            if value.trim() != value {
                return Err(RequestPolicySummaryValidationError::WhitespacePaddedField { field });
            }
            if value.contains('\n') || value.contains('\r') {
                return Err(RequestPolicySummaryValidationError::EmbeddedLineBreak { field });
            }
            if value != expected {
                return Err(RequestPolicySummaryValidationError::FieldOutOfSync { field });
            }
        }

        Ok(())
    }
}

/// Compact summary of the current shared time-scale policy.
///
/// # Example
///
/// ```ignore
/// use pleiades_backend::TimeScalePolicySummary;
///
/// let summary = TimeScalePolicySummary::current();
/// assert_eq!(summary.to_string(), summary.summary_line());
/// assert!(summary.summary_line().contains("TT/TDB"));
/// assert!(summary.validate().is_ok());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TimeScalePolicySummary {
    summary: &'static str,
}

/// Validation error for a time-scale policy summary that drifted away from a compact release-facing line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TimeScalePolicySummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
    /// The summary text no longer matches the current canonical posture.
    CurrentPolicyOutOfSync,
}

impl fmt::Display for TimeScalePolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("time-scale policy summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("time-scale policy summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => {
                f.write_str("time-scale policy summary contains a line break")
            }
            Self::CurrentPolicyOutOfSync => {
                f.write_str("time-scale policy summary is out of sync with the current posture")
            }
        }
    }
}

impl std::error::Error for TimeScalePolicySummaryValidationError {}

impl TimeScalePolicySummary {
    /// Creates a new time-scale policy summary from a backend-owned note.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the time-scale policy posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns the current shared time-scale policy posture.
    pub const fn current() -> Self {
        Self::new(CURRENT_TIME_SCALE_POLICY_SUMMARY_TEXT)
    }

    /// Returns `Ok(())` when the summary still contains the current canonical line.
    pub fn validate(&self) -> Result<(), TimeScalePolicySummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(TimeScalePolicySummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(TimeScalePolicySummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(TimeScalePolicySummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != CURRENT_TIME_SCALE_POLICY_SUMMARY_TEXT {
            Err(TimeScalePolicySummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    /// Returns the compact summary line after validating the cached prose.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, TimeScalePolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for TimeScalePolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

/// Compact summary of the current shared UTC-convenience policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UtcConveniencePolicySummary {
    summary: &'static str,
}

/// Validation error for the shared UTC-convenience policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UtcConveniencePolicySummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
    /// The summary text no longer matches the current canonical posture.
    CurrentPolicyOutOfSync,
}

impl fmt::Display for UtcConveniencePolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("UTC convenience policy summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("UTC convenience policy summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => {
                f.write_str("UTC convenience policy summary contains a line break")
            }
            Self::CurrentPolicyOutOfSync => f.write_str(
                "UTC convenience policy summary is out of sync with the current posture",
            ),
        }
    }
}

impl std::error::Error for UtcConveniencePolicySummaryValidationError {}

impl UtcConveniencePolicySummary {
    /// Creates a new UTC-convenience policy summary from a backend-owned note.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the UTC-convenience policy posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns the current shared UTC-convenience policy posture.
    pub const fn current() -> Self {
        Self::new(CURRENT_UTC_CONVENIENCE_POLICY_SUMMARY_TEXT)
    }

    /// Returns `Ok(())` when the summary still contains the current canonical line.
    pub fn validate(&self) -> Result<(), UtcConveniencePolicySummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(UtcConveniencePolicySummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(UtcConveniencePolicySummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(UtcConveniencePolicySummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != CURRENT_UTC_CONVENIENCE_POLICY_SUMMARY_TEXT {
            Err(UtcConveniencePolicySummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    /// Returns the compact summary line after validating the cached prose.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, UtcConveniencePolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for UtcConveniencePolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

/// Compact summary of the current shared zodiac policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ZodiacPolicySummary {
    summary: &'static str,
}

/// Validation error for the shared zodiac-policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ZodiacPolicySummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
    /// The summary text no longer matches the current zodiac posture.
    CurrentPolicyOutOfSync,
}

impl fmt::Display for ZodiacPolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("zodiac policy summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("zodiac policy summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => f.write_str("zodiac policy summary contains a line break"),
            Self::CurrentPolicyOutOfSync => {
                f.write_str("zodiac policy summary is out of sync with the current posture")
            }
        }
    }
}

impl std::error::Error for ZodiacPolicySummaryValidationError {}

impl ZodiacPolicySummary {
    /// Creates a new zodiac policy summary from a backend-owned note.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the zodiac policy posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns the current shared zodiac policy posture.
    pub const fn current() -> Self {
        Self::new(CURRENT_ZODIAC_POLICY_SUMMARY_TEXT)
    }

    /// Returns `Ok(())` when the summary still contains the current canonical line.
    pub fn validate(&self) -> Result<(), ZodiacPolicySummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(ZodiacPolicySummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(ZodiacPolicySummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(ZodiacPolicySummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != CURRENT_ZODIAC_POLICY_SUMMARY_TEXT {
            Err(ZodiacPolicySummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    /// Returns the compact summary line after validating the cached prose.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, ZodiacPolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ZodiacPolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

/// Compact summary of the current frame policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FramePolicySummary {
    summary: &'static str,
}

/// Validation error for the current frame-policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FramePolicySummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
    /// The summary text no longer matches the current frame-policy posture.
    CurrentPolicyOutOfSync,
}

impl fmt::Display for FramePolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("frame-policy summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("frame-policy summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => f.write_str("frame-policy summary contains a line break"),
            Self::CurrentPolicyOutOfSync => {
                f.write_str("frame-policy summary is out of sync with the current frame policy")
            }
        }
    }
}

impl std::error::Error for FramePolicySummaryValidationError {}

impl FramePolicySummary {
    /// Creates a new frame-policy summary from a backend-owned note.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the frame-policy posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns `Ok(())` when the summary still matches the current frame-policy posture.
    pub fn validate(&self) -> Result<(), FramePolicySummaryValidationError> {
        let current = CURRENT_FRAME_POLICY_SUMMARY_TEXT;

        if self.summary.trim().is_empty() {
            Err(FramePolicySummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(FramePolicySummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(FramePolicySummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != current {
            Err(FramePolicySummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    /// Returns the compact one-line rendering after validation.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, FramePolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for FramePolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::{
        validate_observer_policy, EphemerisErrorKind, EphemerisRequest, FrameTreatmentSummary,
        FrameTreatmentSummaryValidationError,
    };
    use pleiades_types::{
        CelestialBody, Instant, Latitude, Longitude, ObserverLocation, TimeScale, ZodiacMode,
    };

    #[test]
    fn time_scale_policy_summary_has_a_compact_display() {
        let summary = TimeScalePolicySummary::current();

        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            summary.summary_line(),
            CURRENT_TIME_SCALE_POLICY_SUMMARY_TEXT
        );
        assert!(summary.summary_line().contains("TT/TDB"));
        assert!(summary.validate().is_ok());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            time_scale_policy_summary_for_report().summary_line(),
            summary.summary_line()
        );
    }

    #[test]
    fn time_scale_policy_summary_validate_rejects_blank_fields() {
        let summary = TimeScalePolicySummary::new(" ");

        let error = summary
            .validate()
            .expect_err("blank policy prose should fail validation");
        assert_eq!(error.to_string(), "time-scale policy summary is blank");
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn time_scale_policy_summary_validate_rejects_policy_drift() {
        let summary = TimeScalePolicySummary::new(
            "direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; built-in Delta T model",
        );

        let error = summary
            .validate()
            .expect_err("drifted policy prose should fail validation");
        assert_eq!(
            error.to_string(),
            "time-scale policy summary is out of sync with the current posture"
        );
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn delta_t_policy_summary_has_a_compact_display() {
        let summary = DeltaTPolicySummary::current();

        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.summary_line(), CURRENT_DELTA_T_POLICY_SUMMARY_TEXT);
        assert!(summary.summary_line().contains("Delta T"));
        assert!(summary.validate().is_ok());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            delta_t_policy_summary_for_report().summary_line(),
            summary.summary_line()
        );
    }

    #[test]
    fn delta_t_policy_summary_validate_rejects_blank_fields() {
        let summary = DeltaTPolicySummary::new(" ");

        let error = summary
            .validate()
            .expect_err("blank Delta T policy prose should fail validation");
        assert_eq!(error.to_string(), "Delta T policy summary is blank");
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn delta_t_policy_summary_validate_rejects_policy_drift() {
        let summary = DeltaTPolicySummary::new("built-in Delta T modeling is documented elsewhere");

        let error = summary
            .validate()
            .expect_err("drifted Delta T policy prose should fail validation");
        assert_eq!(
            error.to_string(),
            "Delta T policy summary is out of sync with the current posture"
        );
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn utc_convenience_policy_summary_has_a_compact_display() {
        let summary = UtcConveniencePolicySummary::current();

        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            summary.summary_line(),
            CURRENT_UTC_CONVENIENCE_POLICY_SUMMARY_TEXT
        );
        assert!(summary.summary_line().contains("UTC convenience"));
        assert!(summary.validate().is_ok());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            utc_convenience_policy_summary_for_report().summary_line(),
            summary.summary_line()
        );
    }

    #[test]
    fn utc_convenience_policy_summary_validate_rejects_blank_fields() {
        let summary = UtcConveniencePolicySummary::new(" ");

        let error = summary
            .validate()
            .expect_err("blank UTC convenience prose should fail validation");
        assert_eq!(error.to_string(), "UTC convenience policy summary is blank");
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn utc_convenience_policy_summary_validate_rejects_policy_drift() {
        let summary = UtcConveniencePolicySummary::new(
            "built-in UTC convenience conversion is documented elsewhere",
        );

        let error = summary
            .validate()
            .expect_err("drifted UTC convenience policy prose should fail validation");
        assert_eq!(
            error.to_string(),
            "UTC convenience policy summary is out of sync with the current posture"
        );
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn validated_utc_convenience_policy_summary_for_report_tracks_the_current_posture() {
        assert_eq!(
            validated_utc_convenience_policy_summary_for_report(),
            CURRENT_UTC_CONVENIENCE_POLICY_SUMMARY_TEXT
        );
    }

    #[test]
    fn validated_request_policy_component_summaries_track_the_current_posture() {
        assert_eq!(
            validated_time_scale_policy_summary_for_report(),
            CURRENT_TIME_SCALE_POLICY_SUMMARY_TEXT
        );
        assert_eq!(
            validated_delta_t_policy_summary_for_report(),
            CURRENT_DELTA_T_POLICY_SUMMARY_TEXT
        );
        assert_eq!(
            validated_request_policy_summary_for_report(),
            current_request_policy_summary()
                .validated_summary_line()
                .unwrap()
        );
        assert_eq!(
            validated_observer_policy_summary_for_report(),
            CURRENT_OBSERVER_POLICY_SUMMARY_TEXT
        );
        assert_eq!(
            validated_apparentness_policy_summary_for_report(),
            CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT
        );
    }

    #[test]
    fn request_policy_summary_has_a_compact_display() {
        let summary = RequestPolicySummary::current();

        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            summary.summary_line(),
            RequestPolicySummary {
                time_scale: CURRENT_TIME_SCALE_POLICY_SUMMARY_TEXT,
                observer: CURRENT_OBSERVER_POLICY_SUMMARY_TEXT,
                apparentness: CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT,
                frame: CURRENT_FRAME_POLICY_SUMMARY_TEXT,
            }
            .summary_line()
        );
        assert!(summary.summary_line().contains("time-scale="));
        assert!(summary.summary_line().contains("observer="));
        assert!(summary.summary_line().contains("apparentness="));
        assert!(summary.summary_line().contains("frame="));
        assert!(summary.validate().is_ok());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            validated_request_semantics_summary_for_report(),
            validated_request_policy_summary_for_report()
        );
        assert_eq!(
            request_semantics_summary_for_report(),
            request_policy_summary_for_report()
        );
        assert_eq!(
            request_semantics_summary_for_report().summary_line(),
            request_policy_summary_for_report().summary_line()
        );
        assert_eq!(
            request_semantics_summary_for_report().validated_summary_line(),
            request_policy_summary_for_report().validated_summary_line()
        );
    }

    #[test]
    fn request_policy_summary_validate_rejects_blank_fields() {
        let mut summary = RequestPolicySummary::current();
        summary.frame = " ";

        let error = summary
            .validate()
            .expect_err("blank policy prose should fail validation");
        assert_eq!(
            error,
            RequestPolicySummaryValidationError::BlankField { field: "frame" }
        );
        assert_eq!(
            error.to_string(),
            "the request-policy summary field `frame` is blank"
        );
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn request_policy_summary_validate_rejects_whitespace_padded_fields() {
        let mut summary = RequestPolicySummary::current();
        summary.observer = " chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; chart-layer topocentric body positions are supported as an opt-in correction (diurnal parallax + diurnal aberration); native-backend topocentric remains unsupported ";

        let error = summary
            .validate()
            .expect_err("whitespace-padded policy prose should fail validation");
        assert_eq!(
            error,
            RequestPolicySummaryValidationError::WhitespacePaddedField { field: "observer" }
        );
        assert_eq!(
            error.to_string(),
            "the request-policy summary field `observer` has surrounding whitespace"
        );
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn request_policy_summary_validate_rejects_line_breaks() {
        let mut summary = RequestPolicySummary::current();
        summary.observer = "chart houses use observer locations\nbody requests stay geocentric";

        let error = summary
            .validate()
            .expect_err("multi-line policy prose should fail validation");
        assert_eq!(
            error,
            RequestPolicySummaryValidationError::EmbeddedLineBreak { field: "observer" }
        );
        assert_eq!(
            error.to_string(),
            "the request-policy summary field `observer` contains a line break"
        );
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn frame_treatment_summary_has_a_compact_display() {
        let summary = FrameTreatmentSummary::new(
            "geocentric ecliptic inputs; equatorial coordinates are derived with a mean-obliquity transform",
        );

        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.summary_line(), "geocentric ecliptic inputs; equatorial coordinates are derived with a mean-obliquity transform");
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            validated_frame_treatment_summary_for_report(),
            current_request_policy_summary().frame
        );
        assert!(summary.summary_line().contains("mean-obliquity"));
    }

    #[test]
    fn frame_treatment_summary_rejects_blank_summary_text() {
        let summary = FrameTreatmentSummary::new("   ");

        assert_eq!(
            summary.validate(),
            Err(FrameTreatmentSummaryValidationError::BlankSummary)
        );
    }

    #[test]
    fn frame_treatment_summary_rejects_whitespace_padded_summary_text() {
        let summary = FrameTreatmentSummary::new(
            " geocentric ecliptic inputs; equatorial coordinates are derived with a mean-obliquity transform ",
        );

        assert_eq!(
            summary.validate(),
            Err(FrameTreatmentSummaryValidationError::WhitespacePaddedSummary)
        );
    }

    #[test]
    fn frame_treatment_summary_rejects_embedded_line_breaks() {
        let summary = FrameTreatmentSummary::new(
            "geocentric ecliptic inputs;\nequatorial coordinates are derived with a mean-obliquity transform",
        );

        assert_eq!(
            summary.validate(),
            Err(FrameTreatmentSummaryValidationError::EmbeddedLineBreak)
        );
    }

    #[test]
    fn frame_policy_summary_tracks_the_current_posture() {
        let summary = FramePolicySummary::new(current_request_policy_summary().frame);

        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            summary.summary_line(),
            current_request_policy_summary().frame
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert!(summary.summary_line().contains("mean-obliquity"));
    }

    #[test]
    fn frame_policy_summary_rejects_policy_drift() {
        let summary = FramePolicySummary::new(
            "geocentric ecliptic inputs; equatorial coordinates are derived with a mean-obliquity transform",
        );

        assert_eq!(
            summary.validate(),
            Err(FramePolicySummaryValidationError::CurrentPolicyOutOfSync)
        );
    }

    #[test]
    fn frame_policy_summary_details_reuse_the_current_posture() {
        let summary = frame_policy_summary_details();

        assert_eq!(
            summary.summary_line(),
            current_request_policy_summary().frame
        );
        assert_eq!(
            frame_policy_summary_for_report(),
            current_request_policy_summary().frame
        );
        assert_eq!(
            validated_frame_policy_summary_for_report(),
            current_request_policy_summary().frame
        );
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    }

    #[test]
    fn native_sidereal_policy_summary_tracks_the_current_posture() {
        let summary = native_sidereal_policy_summary_for_report();

        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            summary.summary_line(),
            current_native_sidereal_policy_summary().summary_line()
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert!(summary
            .summary_line()
            .contains("native sidereal backend output"));
    }

    #[test]
    fn native_sidereal_policy_summary_rejects_policy_drift() {
        let summary = NativeSiderealPolicySummary::new(
            "native sidereal backend output is documented elsewhere",
        );

        assert_eq!(
            summary.validate(),
            Err(NativeSiderealPolicySummaryValidationError::CurrentPolicyOutOfSync)
        );
    }

    #[test]
    fn validated_native_sidereal_policy_summary_for_report_tracks_the_current_posture() {
        assert_eq!(
            validated_native_sidereal_policy_summary_for_report(),
            CURRENT_NATIVE_SIDEREAL_POLICY_SUMMARY_TEXT
        );
    }

    #[test]
    fn zodiac_policy_summary_tracks_the_current_posture() {
        let summary = current_zodiac_policy_summary();

        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            summary.summary_line(),
            current_zodiac_policy_summary().summary_line()
        );
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(summary.summary_line(), CURRENT_ZODIAC_POLICY_SUMMARY_TEXT);
    }

    #[test]
    fn zodiac_policy_summary_rejects_invalid_cached_prose() {
        assert_eq!(
            ZodiacPolicySummary::new("   ").validate(),
            Err(ZodiacPolicySummaryValidationError::BlankSummary)
        );
        assert_eq!(
            ZodiacPolicySummary::new(" tropical only ").validate(),
            Err(ZodiacPolicySummaryValidationError::WhitespacePaddedSummary)
        );
        assert_eq!(
            ZodiacPolicySummary::new("tropical\nonly").validate(),
            Err(ZodiacPolicySummaryValidationError::EmbeddedLineBreak)
        );
    }

    #[test]
    fn zodiac_policy_summary_rejects_policy_drift() {
        let summary = ZodiacPolicySummary::new("sidereal zodiac output is documented elsewhere");

        assert_eq!(
            summary.validate(),
            Err(ZodiacPolicySummaryValidationError::CurrentPolicyOutOfSync)
        );
    }

    #[test]
    fn validated_zodiac_policy_summary_for_report_tracks_the_current_posture() {
        assert_eq!(
            validated_zodiac_policy_summary_for_report(),
            CURRENT_ZODIAC_POLICY_SUMMARY_TEXT
        );
    }

    #[test]
    fn observer_policy_summary_validates_the_current_report_prose() {
        let summary = observer_policy_summary_for_report();
        assert_eq!(summary.summary_line(), summary.to_string());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    }

    #[test]
    fn apparentness_policy_summary_validates_the_current_report_prose() {
        let summary = apparentness_policy_summary_for_report();
        assert_eq!(summary.summary_line(), summary.to_string());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    }

    #[test]
    fn observer_policy_summary_rejects_invalid_cached_prose() {
        assert!(matches!(
            ObserverPolicySummary::new("").validate(),
            Err(ObserverPolicySummaryValidationError::BlankSummary)
        ));
        assert!(matches!(
            ObserverPolicySummary::new(" observer ").validate(),
            Err(ObserverPolicySummaryValidationError::WhitespacePaddedSummary)
        ));
        assert!(matches!(
            ObserverPolicySummary::new("observer\npolicy").validate(),
            Err(ObserverPolicySummaryValidationError::EmbeddedLineBreak)
        ));
        assert!(matches!(
            ObserverPolicySummary::new("observer policy drift").validate(),
            Err(ObserverPolicySummaryValidationError::CurrentPolicyOutOfSync)
        ));
    }

    #[test]
    fn apparentness_policy_summary_rejects_invalid_cached_prose() {
        assert!(matches!(
            ApparentnessPolicySummary::new("").validate(),
            Err(ApparentnessPolicySummaryValidationError::BlankSummary)
        ));
        assert!(matches!(
            ApparentnessPolicySummary::new(" apparent ").validate(),
            Err(ApparentnessPolicySummaryValidationError::WhitespacePaddedSummary)
        ));
        assert!(matches!(
            ApparentnessPolicySummary::new("apparent\npolicy").validate(),
            Err(ApparentnessPolicySummaryValidationError::EmbeddedLineBreak)
        ));
        assert!(matches!(
            ApparentnessPolicySummary::new("apparentness policy drift").validate(),
            Err(ApparentnessPolicySummaryValidationError::CurrentPolicyOutOfSync)
        ));
    }

    #[test]
    fn validate_observer_policy_rejects_invalid_observer_locations_even_when_supported() {
        let mut observer_request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(
                pleiades_types::JulianDay::from_days(2_451_545.0),
                TimeScale::Tt,
            ),
        );
        observer_request.observer = Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        ));
        let error = validate_observer_policy(&observer_request, "toy backend", true).expect_err(
            "invalid observer locations should fail even when topocentric support is available",
        );
        assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
        assert!(error
            .message
            .contains("observer latitude must stay within [-90, 90]"));
        assert!(error.message.contains("received invalid observer location"));
    }

    #[test]
    fn request_policy_summary_validation_rejects_stale_field_text() {
        fn assert_field_out_of_sync(
            mut summary: RequestPolicySummary,
            field: &'static str,
            mutate: impl FnOnce(&mut RequestPolicySummary),
        ) {
            mutate(&mut summary);

            let error = summary
                .validate()
                .expect_err("stale request-policy wording should fail validation");

            assert_eq!(
                error,
                RequestPolicySummaryValidationError::FieldOutOfSync { field }
            );
            assert_eq!(
                error.to_string(),
                format!(
                    "the request-policy summary field `{field}` is out of sync with the current posture"
                )
            );
        }

        let current = current_request_policy_summary();

        assert_field_out_of_sync(current, "time_scale", |summary| {
            summary.time_scale =
                "direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers";
        });
        assert_field_out_of_sync(current, "observer", |summary| {
            summary.observer = "chart houses use observer locations; chart body observers stay separate; body requests stay geocentric";
        });
        assert_field_out_of_sync(current, "apparentness", |summary| {
            summary.apparentness = "current first-party backends accept mean geometric output only";
        });
        assert_field_out_of_sync(current, "frame", |summary| {
            summary.frame = "ecliptic body positions are the default request shape";
        });
    }

    #[test]
    fn request_policy_component_report_summaries_match_the_current_posture() {
        let request_policy = current_request_policy_summary();
        assert_eq!(
            request_policy.time_scale,
            "direct backend requests accept TT/TDB; civil UTC/UT1 inputs convert via the pleiades-time crate or caller-supplied offsets; the ephemeris backends carry no internal Delta T or UTC convenience model"
        );
        assert_eq!(
            request_policy.observer,
            "chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; chart-layer topocentric body positions are supported as an opt-in correction (diurnal parallax + diurnal aberration); native-backend topocentric remains unsupported"
        );
        assert_eq!(
            request_policy.apparentness,
            "backends remain mean-only and J2000 at the backend boundary; apparent place of date (chart layer, default): light-time + precession-to-date + annual aberration + nutation-in-longitude, release-grade bodies; gravitational light-deflection omitted"
        );
        assert_eq!(
            request_policy.frame,
            "ecliptic body positions are the default request shape; at the backend boundary equatorial output is derived via mean-obliquity transforms when supported, while the chart layer reports apparent equatorial of date (true obliquity = mean obliquity + nutation-in-obliquity) for release-grade bodies; supported equatorial precision is bounded by the shared mean-obliquity frame round-trip envelope; native sidereal backend output remains unsupported unless a backend explicitly advertises it"
        );
        assert_eq!(
            time_scale_policy_summary_for_report().summary_line(),
            request_policy.time_scale
        );
        assert_eq!(
            observer_policy_summary_for_report().summary_line(),
            request_policy.observer
        );
        assert_eq!(
            apparentness_policy_summary_for_report().summary_line(),
            request_policy.apparentness
        );
        assert_eq!(frame_policy_summary_for_report(), request_policy.frame);
        assert_eq!(
            zodiac_policy_summary_for_report(&[ZodiacMode::Tropical]),
            "tropical only"
        );
    }
}
