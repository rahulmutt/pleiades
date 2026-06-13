use core::fmt;

use pleiades_backend::Apparentness;
use pleiades_types::{CoordinateFrame, TimeScale, ZodiacMode};

/// Structured request policy for the current lunar-theory selection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarTheoryRequestPolicy {
    /// Coordinate frames the current baseline exposes.
    pub supported_frames: &'static [CoordinateFrame],
    /// Time scales accepted by the current baseline.
    pub supported_time_scales: &'static [TimeScale],
    /// Zodiac modes accepted by the current baseline.
    pub supported_zodiac_modes: &'static [ZodiacMode],
    /// Apparentness modes accepted by the current baseline.
    pub supported_apparentness: &'static [Apparentness],
    /// Whether the current baseline accepts topocentric observer requests.
    pub supports_topocentric_observer: bool,
}

/// Validation error for a lunar-theory request policy that drifted from the current selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LunarTheoryRequestPolicyValidationError {
    /// A rendered policy field no longer matches the current lunar-theory selection.
    FieldOutOfSync {
        /// Field that drifted out of sync.
        field: &'static str,
    },
}

impl fmt::Display for LunarTheoryRequestPolicyValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the lunar theory request policy field `{field}` is out of sync with the current selection"
            ),
        }
    }
}

impl std::error::Error for LunarTheoryRequestPolicyValidationError {}

impl LunarTheoryRequestPolicy {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "frames={}; time scales={}; zodiac modes={}; apparentness={}; topocentric observer={}",
            crate::format_frames(self.supported_frames),
            crate::format_time_scales(self.supported_time_scales),
            crate::format_zodiac_modes(self.supported_zodiac_modes),
            crate::format_apparentness_modes(self.supported_apparentness),
            self.supports_topocentric_observer,
        )
    }

    /// Returns `Ok(())` when the policy still matches the current lunar-theory selection.
    pub fn validate(&self) -> Result<(), LunarTheoryRequestPolicyValidationError> {
        let theory = crate::specification::lunar_theory_specification();

        if self.supported_frames != theory.request_policy.supported_frames {
            return Err(LunarTheoryRequestPolicyValidationError::FieldOutOfSync {
                field: "supported_frames",
            });
        }
        if self.supported_time_scales != theory.request_policy.supported_time_scales {
            return Err(LunarTheoryRequestPolicyValidationError::FieldOutOfSync {
                field: "supported_time_scales",
            });
        }
        if self.supported_zodiac_modes != theory.request_policy.supported_zodiac_modes {
            return Err(LunarTheoryRequestPolicyValidationError::FieldOutOfSync {
                field: "supported_zodiac_modes",
            });
        }
        if self.supported_apparentness != theory.request_policy.supported_apparentness {
            return Err(LunarTheoryRequestPolicyValidationError::FieldOutOfSync {
                field: "supported_apparentness",
            });
        }
        if self.supports_topocentric_observer != theory.request_policy.supports_topocentric_observer
        {
            return Err(LunarTheoryRequestPolicyValidationError::FieldOutOfSync {
                field: "supports_topocentric_observer",
            });
        }

        Ok(())
    }
}

impl fmt::Display for LunarTheoryRequestPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the current lunar-theory request policy.
pub const fn lunar_theory_request_policy() -> LunarTheoryRequestPolicy {
    crate::specification::LUNAR_THEORY_REQUEST_POLICY
}
