//! JPL Horizons reference snapshot backend for validation and comparison.
//!
//! This crate provides a narrow, source-backed backend based on a checked-in
//! JPL Horizons vector snapshot. The backend is intentionally limited to a
//! single canonical epoch so the stage-4 validation workflow can compare the
//! algorithmic backends against a reproducible reference corpus.

#![forbid(unsafe_code)]

use std::sync::OnceLock;

use pleiades_backend::{
    AccuracyClass, Apparentness, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
    BackendProvenance, EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest,
    EphemerisResult, QualityAnnotation,
};
use pleiades_types::{
    CoordinateFrame, EclipticCoordinates, Instant, JulianDay, Latitude, Longitude, Motion,
    TimeRange, TimeScale, ZodiacMode,
};

const REFERENCE_EPOCH_JD: f64 = 2_451_545.0;
const AU_IN_KM: f64 = 149_597_870.7;

/// Canonical JPL Horizons snapshot instant used by the reference backend.
pub const fn reference_instant() -> Instant {
    Instant::new(JulianDay::from_days(REFERENCE_EPOCH_JD), TimeScale::Tdb)
}

/// The narrow body set covered by the checked-in reference snapshot.
pub fn reference_bodies() -> &'static [pleiades_backend::CelestialBody] {
    snapshot_bodies()
}

/// Returns the parsed reference snapshot entries.
pub fn reference_snapshot() -> &'static [SnapshotEntry] {
    snapshot_entries()
}

/// A reference-backend implementation backed by JPL Horizons snapshot data.
#[derive(Debug, Default, Clone, Copy)]
pub struct JplSnapshotBackend;

impl JplSnapshotBackend {
    /// Creates a new snapshot backend.
    pub const fn new() -> Self {
        Self
    }
}

impl EphemerisBackend for JplSnapshotBackend {
    fn metadata(&self) -> BackendMetadata {
        let bodies = reference_snapshot()
            .iter()
            .map(|entry| entry.body.clone())
            .collect();
        BackendMetadata {
            id: BackendId::new("jpl-snapshot"),
            version: "0.1.0".to_string(),
            family: BackendFamily::ReferenceData,
            provenance: BackendProvenance {
                summary: "NASA/JPL Horizons DE441 geocentric ecliptic snapshot at J2000.0"
                    .to_string(),
                data_sources: vec![
                    "NASA/JPL Horizons API vector table (DE441)".to_string(),
                    "Checked-in reference snapshot at JDTDB 2451545.0".to_string(),
                ],
            },
            nominal_range: TimeRange::new(Some(reference_instant()), Some(reference_instant())),
            supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
            body_coverage: bodies,
            supported_frames: vec![CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities {
                geocentric: true,
                topocentric: false,
                apparent: false,
                mean: true,
                batch: true,
                native_sidereal: false,
            },
            accuracy: AccuracyClass::Exact,
            deterministic: true,
            offline: true,
        }
    }

    fn supports_body(&self, body: pleiades_backend::CelestialBody) -> bool {
        snapshot_entries().iter().any(|entry| entry.body == body)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        if !matches!(req.instant.scale, TimeScale::Tt | TimeScale::Tdb) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedTimeScale,
                "the JPL snapshot backend only serves TT or TDB requests",
            ));
        }

        if (req.instant.julian_day.days() - REFERENCE_EPOCH_JD).abs() > f64::EPSILON {
            return Err(EphemerisError::new(
                EphemerisErrorKind::OutOfRangeInstant,
                "the JPL snapshot backend only serves the J2000.0 reference instant",
            ));
        }

        if req.frame != CoordinateFrame::Ecliptic {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedCoordinateFrame,
                "the JPL snapshot backend only returns ecliptic coordinates",
            ));
        }

        if req.apparent != Apparentness::Mean {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "the JPL snapshot backend serves geometric mean-state vectors only",
            ));
        }

        if req.zodiac_mode != ZodiacMode::Tropical {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "sidereal conversion is handled above the backend layer",
            ));
        }

        if req.observer.is_some() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidObserver,
                "the JPL snapshot backend is geocentric only",
            ));
        }

        let entry = snapshot_entries()
            .iter()
            .find(|entry| entry.body == req.body)
            .ok_or_else(|| {
                EphemerisError::new(
                    EphemerisErrorKind::UnsupportedBody,
                    "the requested body is not present in the JPL snapshot corpus",
                )
            })?;

        let mut result = EphemerisResult::new(
            BackendId::new("jpl-snapshot"),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        result.ecliptic = Some(entry.ecliptic());
        result.motion = None::<Motion>;
        result.quality = QualityAnnotation::Exact;
        Ok(result)
    }
}

/// One parsed record from the reference snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotEntry {
    /// The body covered by the entry.
    pub body: pleiades_backend::CelestialBody,
    /// Cartesian X position in kilometers.
    pub x_km: f64,
    /// Cartesian Y position in kilometers.
    pub y_km: f64,
    /// Cartesian Z position in kilometers.
    pub z_km: f64,
}

impl SnapshotEntry {
    fn ecliptic(&self) -> EclipticCoordinates {
        let radius_km =
            (self.x_km * self.x_km + self.y_km * self.y_km + self.z_km * self.z_km).sqrt();
        let longitude = Longitude::from_degrees(self.y_km.atan2(self.x_km).to_degrees());
        let latitude =
            Latitude::from_degrees((self.z_km / radius_km).clamp(-1.0, 1.0).asin().to_degrees());
        EclipticCoordinates::new(longitude, latitude, Some(radius_km / AU_IN_KM))
    }
}

fn snapshot_entries() -> &'static [SnapshotEntry] {
    static SNAPSHOT: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    SNAPSHOT.get_or_init(load_snapshot).as_slice()
}

fn snapshot_bodies() -> &'static [pleiades_backend::CelestialBody] {
    static BODIES: OnceLock<Vec<pleiades_backend::CelestialBody>> = OnceLock::new();
    BODIES
        .get_or_init(|| {
            snapshot_entries()
                .iter()
                .map(|entry| entry.body.clone())
                .collect()
        })
        .as_slice()
}

fn load_snapshot() -> Vec<SnapshotEntry> {
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/j2000_snapshot.csv"
    ))
    .lines()
    .enumerate()
    .filter_map(|(index, line)| parse_snapshot_line(index + 1, line))
    .collect()
}

fn parse_snapshot_line(line_number: usize, line: &str) -> Option<SnapshotEntry> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    let mut parts = trimmed.split(',').map(str::trim);
    let body = parts
        .next()
        .unwrap_or_else(|| panic!("missing body on line {line_number}"));
    let x_km = parts
        .next()
        .unwrap_or_else(|| panic!("missing x coordinate on line {line_number}"));
    let y_km = parts
        .next()
        .unwrap_or_else(|| panic!("missing y coordinate on line {line_number}"));
    let z_km = parts
        .next()
        .unwrap_or_else(|| panic!("missing z coordinate on line {line_number}"));

    if parts.next().is_some() {
        panic!("unexpected extra columns on line {line_number}");
    }

    Some(SnapshotEntry {
        body: parse_body(body, line_number),
        x_km: parse_f64(x_km, line_number, "x_km"),
        y_km: parse_f64(y_km, line_number, "y_km"),
        z_km: parse_f64(z_km, line_number, "z_km"),
    })
}

fn parse_body(body: &str, line_number: usize) -> pleiades_backend::CelestialBody {
    match body {
        "Sun" => pleiades_backend::CelestialBody::Sun,
        "Moon" => pleiades_backend::CelestialBody::Moon,
        "Mercury" => pleiades_backend::CelestialBody::Mercury,
        "Venus" => pleiades_backend::CelestialBody::Venus,
        "Mars" => pleiades_backend::CelestialBody::Mars,
        "Jupiter" => pleiades_backend::CelestialBody::Jupiter,
        "Saturn" => pleiades_backend::CelestialBody::Saturn,
        "Uranus" => pleiades_backend::CelestialBody::Uranus,
        "Neptune" => pleiades_backend::CelestialBody::Neptune,
        "Pluto" => pleiades_backend::CelestialBody::Pluto,
        other => panic!("unsupported body '{other}' on line {line_number}"),
    }
}

fn parse_f64(value: &str, line_number: usize, column: &str) -> f64 {
    value
        .parse::<f64>()
        .unwrap_or_else(|error| panic!("invalid {column} value on line {line_number}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::{Apparentness, EphemerisRequest};

    #[test]
    fn reference_snapshot_covers_the_expected_bodies() {
        let metadata = JplSnapshotBackend::new().metadata();
        assert!(metadata
            .body_coverage
            .contains(&pleiades_backend::CelestialBody::Sun));
        assert!(metadata
            .body_coverage
            .contains(&pleiades_backend::CelestialBody::Moon));
        assert_eq!(metadata.nominal_range.start, metadata.nominal_range.end);
    }

    #[test]
    fn j2000_sun_position_is_finite() {
        let backend = JplSnapshotBackend;
        let request = EphemerisRequest {
            body: pleiades_backend::CelestialBody::Sun,
            instant: reference_instant(),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        let result = backend
            .position(&request)
            .expect("reference snapshot should resolve");
        let ecliptic = result
            .ecliptic
            .expect("reference snapshot should include ecliptic coordinates");
        assert!(ecliptic.longitude.degrees().is_finite());
        assert!(ecliptic.latitude.degrees().is_finite());
        assert!(ecliptic
            .distance_au
            .expect("distance should be present")
            .is_finite());
    }

    #[test]
    fn snapshot_data_matches_the_known_j2000_sun_longitude() {
        let entry = reference_snapshot()
            .iter()
            .find(|entry| entry.body == pleiades_backend::CelestialBody::Sun)
            .expect("sun entry should exist");

        let longitude = entry.ecliptic().longitude.degrees();
        assert!((longitude - 280.3778227681435).abs() < 1e-9);
    }
}
