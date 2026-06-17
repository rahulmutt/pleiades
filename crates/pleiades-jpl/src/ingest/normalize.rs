//! The single fail-closed normalizer: IR -> SnapshotCorpus + provenance.
// Attribute resolution lands here in Task 5; its callers (row mapping and
// corpus assembly) arrive in Task 6, so these items are not yet wired in.
#![allow(dead_code)]

use pleiades_types::{CoordinateFrame, TimeScale};

use super::error::{Attribute, IngestError};
use super::ir::RawManifest;
use super::profile::{Center, ExpectedProfile, IngestProvenance, Provenance, Units};

/// The four resolved header attributes plus their provenance.
#[derive(Debug)]
pub(crate) struct ResolvedAttributes {
    pub frame: CoordinateFrame,
    pub time_scale: TimeScale,
    pub units: Units,
    pub center: Center,
    pub provenance: IngestProvenance,
}

/// Resolves one attribute from a declared source string and a caller assertion.
///
/// `parse` maps a non-empty declared string to `Some(value)` for supported
/// values, `None` for unsupported ones. Rules:
/// - declared supported + asserted equal      -> Read
/// - declared supported + asserted different   -> Contradiction
/// - declared present but unsupported           -> Unsupported
/// - declared silent + asserted                 -> Asserted
/// - declared silent + unasserted               -> Undetermined
fn resolve<T: Copy + PartialEq>(
    attribute: Attribute,
    declared: Option<&str>,
    asserted: Option<T>,
    parse: impl Fn(&str) -> Option<T>,
    render: impl Fn(&T) -> String,
) -> Result<(T, Provenance), IngestError> {
    match declared.map(str::trim).filter(|s| !s.is_empty()) {
        Some(raw) => match parse(raw) {
            Some(value) => {
                if let Some(want) = asserted {
                    if want != value {
                        return Err(IngestError::Contradiction {
                            attribute,
                            declared: render(&value),
                            expected: render(&want),
                        });
                    }
                }
                Ok((value, Provenance::Read))
            }
            None => Err(IngestError::Unsupported {
                attribute,
                value: raw.to_string(),
            }),
        },
        None => match asserted {
            Some(value) => Ok((value, Provenance::Asserted)),
            None => Err(IngestError::Undetermined { attribute }),
        },
    }
}

fn parse_frame(raw: &str) -> Option<CoordinateFrame> {
    let lower = raw.to_ascii_lowercase();
    if lower.contains("apparent") || lower.contains("topocentric") {
        return None; // unsupported observed-place markers
    }
    if lower.contains("ecliptic") {
        Some(CoordinateFrame::Ecliptic)
    } else if lower.contains("icrf")
        || lower.contains("equatorial")
        || lower.contains("frame: j2000")
    {
        Some(CoordinateFrame::Equatorial)
    } else {
        None
    }
}

fn parse_time_scale(raw: &str) -> Option<TimeScale> {
    match raw.to_ascii_uppercase().as_str() {
        "TDB" | "CT" | "BARYCENTRIC DYNAMICAL TIME" => Some(TimeScale::Tdb),
        "TT" | "TERRESTRIAL TIME" | "TDT" => Some(TimeScale::Tt),
        // UTC/UT1 are intentionally unsupported at this boundary.
        _ => None,
    }
}

fn parse_units(raw: &str) -> Option<Units> {
    let upper = raw.to_ascii_uppercase();
    if upper.starts_with("KM") {
        Some(Units::Km)
    } else if upper.starts_with("AU") {
        Some(Units::Au)
    } else {
        None
    }
}

fn parse_center(raw: &str) -> Option<Center> {
    let norm: String = raw.to_ascii_lowercase().split_whitespace().collect();
    if norm.contains("500@0") || norm == "@0" || norm.contains("barycenter") || norm.contains("ssb")
    {
        Some(Center::SolarSystemBarycenter)
    } else {
        None
    }
}

/// Resolves all four header attributes, failing closed per the design rules.
pub(crate) fn resolve_attributes(
    declared: &RawManifest,
    expected: &ExpectedProfile,
) -> Result<ResolvedAttributes, IngestError> {
    let (frame, frame_prov) = resolve(
        Attribute::Frame,
        declared.frame.as_deref(),
        expected.frame,
        parse_frame,
        |v| v.to_string(),
    )?;
    let (time_scale, ts_prov) = resolve(
        Attribute::TimeScale,
        declared.time_scale.as_deref(),
        expected.time_scale,
        parse_time_scale,
        |v| v.to_string(),
    )?;
    let (units, units_prov) = resolve(
        Attribute::Units,
        declared.units.as_deref(),
        expected.units,
        parse_units,
        |v| format!("{v:?}"),
    )?;
    let (center, center_prov) = resolve(
        Attribute::Center,
        declared.center.as_deref(),
        expected.center,
        parse_center,
        |v| format!("{v:?}"),
    )?;

    Ok(ResolvedAttributes {
        frame,
        time_scale,
        units,
        center,
        provenance: IngestProvenance {
            frame: frame_prov,
            time_scale: ts_prov,
            units: units_prov,
            center: center_prov,
            source_label: declared.source_label.clone(),
            request_url: None,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::{Attribute, Center, ExpectedProfile, IngestError, Provenance, Units};
    use pleiades_types::{CoordinateFrame, TimeScale};

    fn declared(frame: &str, ts: &str, units: &str, center: &str) -> RawManifest {
        RawManifest {
            source_label: Some("JPL Horizons".to_string()),
            center: Some(center.to_string()),
            frame: Some(frame.to_string()),
            time_scale: Some(ts.to_string()),
            units: Some(units.to_string()),
            columns: vec![],
        }
    }

    #[test]
    fn resolves_all_declared_as_read() {
        let m = declared("Ecliptic of J2000.0", "TDB", "KM-S", "500@0");
        let r = resolve_attributes(&m, &ExpectedProfile::default()).unwrap();
        assert_eq!(r.frame, CoordinateFrame::Ecliptic);
        assert_eq!(r.time_scale, TimeScale::Tdb);
        assert_eq!(r.units, Units::Km);
        assert_eq!(r.center, Center::SolarSystemBarycenter);
        assert_eq!(r.provenance.frame, Provenance::Read);
        assert_eq!(r.provenance.time_scale, Provenance::Read);
    }

    #[test]
    fn fills_silent_time_scale_from_expected_as_asserted() {
        let mut m = declared("Ecliptic", "", "KM", "500@0");
        m.time_scale = None;
        let expected = ExpectedProfile {
            time_scale: Some(TimeScale::Tt),
            ..Default::default()
        };
        let r = resolve_attributes(&m, &expected).unwrap();
        assert_eq!(r.time_scale, TimeScale::Tt);
        assert_eq!(r.provenance.time_scale, Provenance::Asserted);
    }

    #[test]
    fn rejects_unsupported_time_scale() {
        let m = declared("Ecliptic", "UTC", "KM", "500@0");
        let err = resolve_attributes(&m, &ExpectedProfile::default()).unwrap_err();
        assert!(matches!(
            err,
            IngestError::Unsupported {
                attribute: Attribute::TimeScale,
                ..
            }
        ));
    }

    #[test]
    fn rejects_contradiction_between_source_and_expected() {
        let m = declared("Ecliptic", "TDB", "KM", "500@0");
        let expected = ExpectedProfile {
            time_scale: Some(TimeScale::Tt),
            ..Default::default()
        };
        let err = resolve_attributes(&m, &expected).unwrap_err();
        assert!(matches!(
            err,
            IngestError::Contradiction {
                attribute: Attribute::TimeScale,
                ..
            }
        ));
    }

    #[test]
    fn rejects_undetermined_when_silent_and_unasserted() {
        let mut m = declared("Ecliptic", "TDB", "KM", "500@0");
        m.frame = None;
        let err = resolve_attributes(&m, &ExpectedProfile::default()).unwrap_err();
        assert!(matches!(
            err,
            IngestError::Undetermined {
                attribute: Attribute::Frame
            }
        ));
    }

    #[test]
    fn converts_au_units() {
        let m = declared("Ecliptic", "TDB", "AU-D", "500@0");
        let r = resolve_attributes(&m, &ExpectedProfile::default()).unwrap();
        assert_eq!(r.units, Units::Au);
    }
}
