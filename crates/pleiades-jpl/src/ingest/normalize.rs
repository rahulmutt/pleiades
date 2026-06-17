//! The single fail-closed normalizer: IR -> SnapshotCorpus + provenance.

use pleiades_backend::CelestialBody;
use pleiades_types::{CoordinateFrame, Instant, JulianDay, TimeScale};

use crate::backend::{SnapshotCorpus, SnapshotEntry, SnapshotManifest};

use super::error::{Attribute, IngestError};
use super::ir::{RawCorpus, RawManifest};
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

/// Kilometers per astronomical unit (matches the crate-wide constant).
const AU_IN_KM: f64 = 149_597_870.7;

/// Maps a raw Horizons/CSV body label to a built-in body.
///
/// Accepts plain names ("Mars"), Horizons "Name (id)" forms ("Mars (499)"),
/// and is case-insensitive. Unknown labels fail closed.
fn body_from_label(label: &str) -> Option<CelestialBody> {
    // Strip a trailing " (id)" suffix if present.
    let name = label.split('(').next().unwrap_or(label).trim();
    let lower = name.to_ascii_lowercase();
    let body = match lower.as_str() {
        "sun" => CelestialBody::Sun,
        "moon" => CelestialBody::Moon,
        "mercury" => CelestialBody::Mercury,
        "venus" => CelestialBody::Venus,
        // Earth has no built-in variant (the chart domain is geocentric) and is
        // not a release-claimed corpus body; an Earth label fails closed here.
        "mars" => CelestialBody::Mars,
        "jupiter" => CelestialBody::Jupiter,
        "saturn" => CelestialBody::Saturn,
        "uranus" => CelestialBody::Uranus,
        "neptune" => CelestialBody::Neptune,
        "pluto" => CelestialBody::Pluto,
        "ceres" => CelestialBody::Ceres,
        "pallas" => CelestialBody::Pallas,
        "juno" => CelestialBody::Juno,
        "vesta" => CelestialBody::Vesta,
        _ => return None,
    };
    Some(body)
}

/// Normalizes a raw external corpus into the typed `SnapshotCorpus`.
// The in-crate caller (`read_public_corpus_as`) lands in Task 10; until then
// this public entry point is unreachable from the lib's public surface because
// `mod normalize` is `pub(crate)`. Marking only the entry point as allowed keeps
// the resolver/mapping chain it drives reachable, so no blanket module allow is
// needed.
#[allow(dead_code)]
pub fn normalize(
    raw: RawCorpus,
    expected: &ExpectedProfile,
) -> Result<(SnapshotCorpus, IngestProvenance), IngestError> {
    let resolved = resolve_attributes(&raw.declared, expected)?;
    let km_per_unit = match resolved.units {
        Units::Km => 1.0,
        Units::Au => AU_IN_KM,
    };

    let mut entries = Vec::with_capacity(raw.records.len());
    for record in &raw.records {
        let body = body_from_label(&record.body_label).ok_or_else(|| IngestError::UnknownBody {
            label: record.body_label.clone(),
        })?;

        if !record.epoch_jd.is_finite() || record.pos.iter().any(|c| !c.is_finite()) {
            return Err(IngestError::MalformedRow {
                epoch_jd: record.epoch_jd,
                detail: "non-finite epoch or position component".to_string(),
            });
        }

        entries.push(SnapshotEntry {
            body,
            epoch: Instant::new(JulianDay::from_days(record.epoch_jd), resolved.time_scale),
            x_km: record.pos[0] * km_per_unit,
            y_km: record.pos[1] * km_per_unit,
            z_km: record.pos[2] * km_per_unit,
        });
    }

    let manifest = SnapshotManifest {
        title: raw.declared.source_label.clone(),
        source: raw.declared.source_label.clone(),
        coverage: None,
        redistribution: None,
        columns: raw.declared.columns.clone(),
    };

    // Frame is carried in provenance, not in SnapshotEntry (entries are frame-agnostic km).
    let _ = (resolved.frame, resolved.center); // captured in provenance below
    Ok((SnapshotCorpus { manifest, entries }, resolved.provenance))
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

    use crate::ingest::ir::{RawCorpus, RawEphemerisRecord};

    fn one_record(body: &str, jd: f64, pos: [f64; 3]) -> RawCorpus {
        RawCorpus {
            declared: declared("Ecliptic", "TDB", "KM-S", "500@0"),
            records: vec![RawEphemerisRecord {
                body_label: body.to_string(),
                epoch_jd: jd,
                pos,
                vel: None,
            }],
        }
    }

    #[test]
    fn normalizes_km_record_into_entry() {
        let raw = one_record("Mars", 2_451_545.0, [1.0, 2.0, 3.0]);
        let (corpus, prov) = normalize(raw, &ExpectedProfile::default()).unwrap();
        assert_eq!(corpus.entries.len(), 1);
        let e = &corpus.entries[0];
        assert_eq!(e.body, pleiades_backend::CelestialBody::Mars);
        assert_eq!(e.epoch.julian_day.days(), 2_451_545.0);
        assert_eq!(e.epoch.scale, TimeScale::Tdb);
        assert_eq!((e.x_km, e.y_km, e.z_km), (1.0, 2.0, 3.0));
        assert_eq!(prov.frame, Provenance::Read);
    }

    #[test]
    fn converts_au_positions_to_km() {
        let mut raw = one_record("Mercury", 2_451_545.0, [1.0, 0.0, 0.0]);
        raw.declared.units = Some("AU-D".to_string());
        let (corpus, _) = normalize(raw, &ExpectedProfile::default()).unwrap();
        assert!((corpus.entries[0].x_km - 149_597_870.7).abs() < 1e-3);
    }

    #[test]
    fn maps_horizons_numeric_id_body() {
        let raw = one_record("Mars (499)", 2_451_545.0, [1.0, 2.0, 3.0]);
        let (corpus, _) = normalize(raw, &ExpectedProfile::default()).unwrap();
        assert_eq!(
            corpus.entries[0].body,
            pleiades_backend::CelestialBody::Mars
        );
    }

    #[test]
    fn rejects_unknown_body() {
        let raw = one_record("Wormwood", 2_451_545.0, [1.0, 2.0, 3.0]);
        let err = normalize(raw, &ExpectedProfile::default()).unwrap_err();
        assert!(matches!(err, IngestError::UnknownBody { .. }));
    }

    #[test]
    fn rejects_non_finite_row() {
        let raw = one_record("Mars", 2_451_545.0, [f64::NAN, 0.0, 0.0]);
        let err = normalize(raw, &ExpectedProfile::default()).unwrap_err();
        assert!(matches!(err, IngestError::MalformedRow { .. }));
    }

    #[test]
    fn carries_source_label_into_manifest() {
        let raw = one_record("Mars", 2_451_545.0, [1.0, 2.0, 3.0]);
        let (corpus, _) = normalize(raw, &ExpectedProfile::default()).unwrap();
        assert_eq!(corpus.manifest.source.as_deref(), Some("JPL Horizons"));
    }
}
