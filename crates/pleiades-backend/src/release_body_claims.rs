use crate::errors::format_display_list;
use crate::policy::pluto_fallback::PlutoFallbackSummaryValidationError;
use core::fmt;
use pleiades_types::{CelestialBody, CustomBodyId};
use std::sync::OnceLock;

fn release_body_claims_lunar_validation_bodies() -> &'static [CelestialBody] {
    &[
        CelestialBody::MeanNode,
        CelestialBody::TrueNode,
        CelestialBody::MeanApogee,
        CelestialBody::MeanPerigee,
    ]
}

fn release_body_claims_major_bodies() -> &'static [CelestialBody] {
    &[
        CelestialBody::Sun,
        CelestialBody::Mercury,
        CelestialBody::Venus,
        CelestialBody::Mars,
        CelestialBody::Jupiter,
        CelestialBody::Saturn,
        CelestialBody::Uranus,
        CelestialBody::Neptune,
    ]
}

fn release_body_claims_selected_asteroids() -> &'static [CelestialBody] {
    static BODIES: OnceLock<Vec<CelestialBody>> = OnceLock::new();
    BODIES
        .get_or_init(|| {
            vec![
                CelestialBody::Ceres,
                CelestialBody::Pallas,
                CelestialBody::Juno,
                CelestialBody::Vesta,
                CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
                CelestialBody::Custom(CustomBodyId::new("asteroid", "99942-Apophis")),
            ]
        })
        .as_slice()
}

pub(crate) fn release_body_claims_summary_text() -> &'static str {
    debug_assert_eq!(
        release_body_claims_major_bodies(),
        &[
            CelestialBody::Sun,
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
        ]
    );

    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            format!(
                "Moon and supported lunar points ({}) remain source-backed validation bodies; True Apogee and True Perigee remain unsupported; Sun through Neptune are release-grade major-body claims; Pluto remains an explicitly approximate fallback; selected asteroids ({}) remain source-backed validation bodies",
                format_display_list(release_body_claims_lunar_validation_bodies()),
                format_display_list(release_body_claims_selected_asteroids()),
            )
        })
        .as_str()
}

/// Compact summary of the current release-grade body claims.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ReleaseBodyClaimsSummary {
    summary: &'static str,
}

/// Validation error for the current release-grade body claims summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReleaseBodyClaimsSummaryValidationError {
    /// The summary text is blank or whitespace-only.
    BlankSummary,
    /// The summary text has surrounding whitespace.
    WhitespacePaddedSummary,
    /// The summary text contains an embedded line break.
    EmbeddedLineBreak,
    /// The summary text no longer matches the current release-grade body claims posture.
    CurrentPolicyOutOfSync,
}

impl fmt::Display for ReleaseBodyClaimsSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("release-grade body claims summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("release-grade body claims summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => {
                f.write_str("release-grade body claims summary contains a line break")
            }
            Self::CurrentPolicyOutOfSync => f.write_str(
                "release-grade body claims summary is out of sync with the current posture",
            ),
        }
    }
}

impl std::error::Error for ReleaseBodyClaimsSummaryValidationError {}

impl ReleaseBodyClaimsSummary {
    /// Creates a new release-grade body claims summary from backend-owned prose.
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    /// Returns the compact one-line rendering of the release-grade body claims posture.
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    /// Returns the current release-grade body claims posture.
    pub fn current() -> Self {
        crate::policy::current::current_release_body_claims_summary()
    }

    /// Returns `Ok(())` when the summary still contains the current canonical line.
    pub fn validate(&self) -> Result<(), ReleaseBodyClaimsSummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(ReleaseBodyClaimsSummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(ReleaseBodyClaimsSummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(ReleaseBodyClaimsSummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != release_body_claims_summary_text() {
            Err(ReleaseBodyClaimsSummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    /// Returns the compact summary line after validating the cached prose.
    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, ReleaseBodyClaimsSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReleaseBodyClaimsSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

/// Validation error for the combined release-grade body-claims posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReleaseBodyClaimsPostureValidationError {
    /// The release-grade body claims summary itself is invalid.
    ReleaseBodyClaimsSummary(ReleaseBodyClaimsSummaryValidationError),
    /// The Pluto fallback summary itself is invalid.
    PlutoFallbackSummary(PlutoFallbackSummaryValidationError),
    /// The release-grade body claims summary no longer mentions the current Pluto fallback phrase.
    MissingPlutoFallbackPhrase,
    /// The release-grade body claims summary no longer keeps the lunar validation bodies explicit.
    MissingLunarValidationPhrase,
    /// The release-grade body claims summary no longer keeps the selected asteroid validation bodies explicit.
    MissingSelectedAsteroidsPhrase,
    /// The Pluto fallback summary no longer states that Pluto is excluded from release-grade claims.
    MissingPlutoExclusionPhrase,
}

impl fmt::Display for ReleaseBodyClaimsPostureValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReleaseBodyClaimsSummary(error) => {
                write!(f, "release-grade body claims summary is invalid: {error}")
            }
            Self::PlutoFallbackSummary(error) => {
                write!(f, "Pluto fallback summary is invalid: {error}")
            }
            Self::MissingPlutoFallbackPhrase => f.write_str(
                "release-grade body claims summary no longer references the current Pluto fallback phrase",
            ),
            Self::MissingLunarValidationPhrase => f.write_str(
                "release-grade body claims summary no longer keeps the lunar validation bodies explicit",
            ),
            Self::MissingSelectedAsteroidsPhrase => f.write_str(
                "release-grade body claims summary no longer keeps the selected asteroid validation bodies explicit",
            ),
            Self::MissingPlutoExclusionPhrase => f.write_str(
                "Pluto fallback summary no longer states that Pluto is excluded from release-grade claims",
            ),
        }
    }
}

impl std::error::Error for ReleaseBodyClaimsPostureValidationError {}

/// Validates the combined release-grade body claims and Pluto fallback posture.
pub fn validate_release_body_claims_posture(
    release_body_claims_summary: &str,
    pluto_fallback_summary: &str,
) -> Result<(), ReleaseBodyClaimsPostureValidationError> {
    if release_body_claims_summary.trim().is_empty() {
        return Err(
            ReleaseBodyClaimsPostureValidationError::ReleaseBodyClaimsSummary(
                ReleaseBodyClaimsSummaryValidationError::BlankSummary,
            ),
        );
    }
    if release_body_claims_summary.trim() != release_body_claims_summary {
        return Err(
            ReleaseBodyClaimsPostureValidationError::ReleaseBodyClaimsSummary(
                ReleaseBodyClaimsSummaryValidationError::WhitespacePaddedSummary,
            ),
        );
    }
    if release_body_claims_summary.contains('\n') || release_body_claims_summary.contains('\r') {
        return Err(
            ReleaseBodyClaimsPostureValidationError::ReleaseBodyClaimsSummary(
                ReleaseBodyClaimsSummaryValidationError::EmbeddedLineBreak,
            ),
        );
    }
    if pluto_fallback_summary.trim().is_empty() {
        return Err(
            ReleaseBodyClaimsPostureValidationError::PlutoFallbackSummary(
                PlutoFallbackSummaryValidationError::BlankSummary,
            ),
        );
    }
    if pluto_fallback_summary.trim() != pluto_fallback_summary {
        return Err(
            ReleaseBodyClaimsPostureValidationError::PlutoFallbackSummary(
                PlutoFallbackSummaryValidationError::WhitespacePaddedSummary,
            ),
        );
    }
    if pluto_fallback_summary.contains('\n') || pluto_fallback_summary.contains('\r') {
        return Err(
            ReleaseBodyClaimsPostureValidationError::PlutoFallbackSummary(
                PlutoFallbackSummaryValidationError::EmbeddedLineBreak,
            ),
        );
    }

    const LUNAR_VALIDATION_PHRASE: &str = "Moon and supported lunar points (Mean Node, True Node, Mean Apogee, Mean Perigee) remain source-backed validation bodies; True Apogee and True Perigee remain unsupported";
    const PLUTO_FALLBACK_PHRASE: &str = "Pluto remains an explicitly approximate fallback";
    const PLUTO_EXCLUSION_PHRASE: &str = "release-grade major-body claims exclude Pluto";
    const SELECTED_ASTEROIDS_PHRASE: &str = "selected asteroids (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis) remain source-backed validation bodies";

    if !release_body_claims_summary.contains(LUNAR_VALIDATION_PHRASE) {
        return Err(ReleaseBodyClaimsPostureValidationError::MissingLunarValidationPhrase);
    }
    if !release_body_claims_summary.contains(PLUTO_FALLBACK_PHRASE) {
        return Err(ReleaseBodyClaimsPostureValidationError::MissingPlutoFallbackPhrase);
    }
    if !release_body_claims_summary.contains(SELECTED_ASTEROIDS_PHRASE) {
        return Err(ReleaseBodyClaimsPostureValidationError::MissingSelectedAsteroidsPhrase);
    }
    if !pluto_fallback_summary.contains(PLUTO_EXCLUSION_PHRASE) {
        return Err(ReleaseBodyClaimsPostureValidationError::MissingPlutoExclusionPhrase);
    }

    Ok(())
}
