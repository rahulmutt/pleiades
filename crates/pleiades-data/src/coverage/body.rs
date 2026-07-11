use super::*;

/// Structured coverage summary for the bodies bundled into the packaged artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagedBodyCoverageSummary {
    /// Number of bodies bundled into the packaged artifact.
    pub body_count: usize,
    /// Bodies bundled into the packaged artifact.
    pub bodies: Vec<CelestialBody>,
}

/// Validation error for a packaged-body coverage summary that drifted from the bundled body set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PackagedBodyCoverageSummaryValidationError {
    /// A rendered summary field no longer matches the current packaged body set.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedBodyCoverageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the packaged body coverage summary field `{field}` is out of sync with the current bundled body set"
            ),
        }
    }
}

impl std::error::Error for PackagedBodyCoverageSummaryValidationError {}

impl PackagedBodyCoverageSummary {
    /// Returns the bundled body set as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "Packaged body set: {} bundled bodies ({})",
            self.body_count,
            join_display(&self.bodies)
        )
    }

    /// Returns `Ok(())` when the summary still matches the bundled body set.
    pub fn validate(&self) -> Result<(), PackagedBodyCoverageSummaryValidationError> {
        let expected_bodies = packaged_bodies();

        if self.body_count != expected_bodies.len() {
            return Err(PackagedBodyCoverageSummaryValidationError::FieldOutOfSync {
                field: "body_count",
            });
        }
        if self.bodies.as_slice() != expected_bodies {
            return Err(PackagedBodyCoverageSummaryValidationError::FieldOutOfSync {
                field: "bodies",
            });
        }

        Ok(())
    }

    /// Returns the bundled body set as a validated compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedBodyCoverageSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedBodyCoverageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the structured packaged body coverage summary.
pub fn packaged_body_coverage_summary_details() -> PackagedBodyCoverageSummary {
    let bodies = packaged_bodies().to_vec();
    PackagedBodyCoverageSummary {
        body_count: bodies.len(),
        bodies,
    }
}
