use core::fmt;

/// Capability flags for a backend.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BackendCapabilities {
    /// Whether geocentric calculations are supported.
    pub geocentric: bool,
    /// Whether topocentric calculations are supported.
    pub topocentric: bool,
    /// Whether apparent values are supported.
    pub apparent: bool,
    /// Whether mean values are supported.
    pub mean: bool,
    /// Whether the backend can serve batch requests.
    pub batch: bool,
    /// Whether sidereal outputs are computed natively rather than derived above the backend.
    pub native_sidereal: bool,
}

impl Default for BackendCapabilities {
    fn default() -> Self {
        Self {
            geocentric: true,
            topocentric: false,
            apparent: true,
            mean: true,
            batch: true,
            native_sidereal: false,
        }
    }
}

impl BackendCapabilities {
    /// Returns a compact one-line rendering of the declared capability flags.
    pub fn summary_line(&self) -> String {
        format!(
            "geocentric={}; topocentric={}; apparent={}; mean={}; batch={}; native_sidereal={}",
            self.geocentric,
            self.topocentric,
            self.apparent,
            self.mean,
            self.batch,
            self.native_sidereal
        )
    }

    /// Returns the compact capability summary after validating the flag set.
    pub fn validated_summary_line(&self) -> Result<String, BackendCapabilitiesValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns `Ok(())` when the capability flags describe at least one usable
    /// position mode and one usable value mode.
    pub fn validate(&self) -> Result<(), BackendCapabilitiesValidationError> {
        if !self.geocentric && !self.topocentric {
            return Err(BackendCapabilitiesValidationError::MissingPositionMode);
        }

        if !self.apparent && !self.mean {
            return Err(BackendCapabilitiesValidationError::MissingValueMode);
        }

        Ok(())
    }
}

/// Errors returned when the declared backend capabilities cannot describe a usable request shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BackendCapabilitiesValidationError {
    /// Neither geocentric nor topocentric position support was declared.
    MissingPositionMode,
    /// Neither apparent nor mean output support was declared.
    MissingValueMode,
}

impl BackendCapabilitiesValidationError {
    /// Returns a compact validation summary string.
    pub fn summary_line(&self) -> &'static str {
        match self {
            Self::MissingPositionMode => {
                "backend capabilities must support geocentric or topocentric positions"
            }
            Self::MissingValueMode => "backend capabilities must support mean or apparent output",
        }
    }
}

impl fmt::Display for BackendCapabilitiesValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

impl std::error::Error for BackendCapabilitiesValidationError {}

impl fmt::Display for BackendCapabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}
