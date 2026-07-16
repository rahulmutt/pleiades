use std::fmt;
use std::sync::OnceLock;

use crate::*;

const SELECTED_ASTEROID_SOURCE_2451917_EPOCH: f64 = 2_451_917.5;

fn selected_asteroid_source_2451917_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_reference_asteroid(&entry.body)
                        && entry.epoch.julian_day.days() == SELECTED_ASTEROID_SOURCE_2451917_EPOCH
                })
                .cloned()
                .collect()
        })
        .as_slice();

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

/// Compact release-facing summary for the selected-asteroid 2001-01-08 source evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedAsteroidSource2451917Summary {
    /// Number of exact samples in the source slice.
    pub sample_count: usize,
    /// Bodies covered by the source slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the source slice.
    pub epoch: Instant,
}

/// Validation errors for a selected-asteroid 2001-01-08 source summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum SelectedAsteroidSource2451917SummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        sample_count: usize,
        derived_sample_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch { expected: Instant, found: Instant },
}

impl fmt::Display for SelectedAsteroidSource2451917SummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("selected asteroid 2001-01-08 source evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "selected asteroid 2001-01-08 source evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "selected asteroid 2001-01-08 source evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "selected asteroid 2001-01-08 source evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for SelectedAsteroidSource2451917SummaryValidationError {}

impl SelectedAsteroidSource2451917Summary {
    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), SelectedAsteroidSource2451917SummaryValidationError> {
        let evidence = selected_asteroid_source_2451917_entries()
            .ok_or(SelectedAsteroidSource2451917SummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                SelectedAsteroidSource2451917SummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        let mut expected_bodies = Vec::new();
        for entry in evidence {
            if !expected_bodies.contains(&entry.body) {
                expected_bodies.push(entry.body.clone());
            }
        }
        if self.sample_bodies.as_slice() != expected_bodies.as_slice() {
            for (index, (expected, found)) in expected_bodies
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        SelectedAsteroidSource2451917SummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                SelectedAsteroidSource2451917SummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                SelectedAsteroidSource2451917SummaryValidationError::EpochMismatch {
                    expected: evidence[0].epoch,
                    found: self.epoch,
                },
            );
        }

        Ok(())
    }
}

fn selected_asteroid_source_2451917_summary_details() -> Option<SelectedAsteroidSource2451917Summary>
{
    let evidence = selected_asteroid_source_2451917_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(SelectedAsteroidSource2451917Summary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the selected-asteroid 2001-01-08 source evidence.
pub fn selected_asteroid_source_2451917_summary() -> Option<SelectedAsteroidSource2451917Summary> {
    selected_asteroid_source_2451917_summary_details()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_asteroid_source_2451917_summary_reports_the_2001_source_slice() {
        let summary = selected_asteroid_source_2451917_summary()
            .expect("selected asteroid 2001-01-08 source summary should exist");
        assert!(summary.sample_count > 0);
        assert_eq!(summary.epoch.julian_day.days(), 2_451_917.5);
        assert_eq!(summary.validate(), Ok(()));
    }
}
