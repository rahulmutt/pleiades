//! JPL Horizons reference snapshot backend for validation, comparison, and
//! selected asteroid support.
//!
//! This crate provides a narrow, source-backed backend based on a checked-in
//! JPL Horizons vector snapshot. The backend is intentionally limited to a
//! small set of canonical epochs so the stage-4 validation workflow can
//! compare the algorithmic backends against a reproducible reference corpus
//! with a broader time span than the original single-epoch snapshot.
//!
//! The checked-in snapshot now also includes a small set of named asteroids so
//! the shared body taxonomy can exercise source-backed asteroid support without
//! changing the comparison corpus used by validation reports.

#![forbid(unsafe_code)]

use core::fmt;
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

/// The instants covered by the checked-in reference snapshot.
pub fn reference_epochs() -> &'static [Instant] {
    snapshot_instants()
}

/// Returns the parsed reference snapshot entries.
pub fn reference_snapshot() -> &'static [SnapshotEntry] {
    snapshot_entries().unwrap_or(&[])
}

/// Returns the source-backed asteroid subset present in the reference snapshot.
pub fn reference_asteroids() -> &'static [pleiades_backend::CelestialBody] {
    reference_asteroid_list()
}

/// Returns the comparison-only subset used by the stage-4 validation corpus.
pub fn comparison_snapshot() -> &'static [SnapshotEntry] {
    comparison_snapshot_entries()
}

/// Returns the comparison-only body coverage used by validation tooling.
pub fn comparison_bodies() -> &'static [pleiades_backend::CelestialBody] {
    comparison_body_list()
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
        let bodies = reference_bodies().to_vec();
        let epochs = reference_epochs();
        let dataset_missing = snapshot_error().is_some();
        BackendMetadata {
            id: BackendId::new("jpl-snapshot"),
            version: "0.1.0".to_string(),
            family: BackendFamily::ReferenceData,
            provenance: BackendProvenance {
                summary: "NASA/JPL Horizons DE441 geocentric ecliptic snapshot across a small set of reference epochs"
                    .to_string(),
                data_sources: vec![
                    "NASA/JPL Horizons API vector tables (DE441)".to_string(),
                    "Checked-in reference snapshot at canonical comparison epochs".to_string(),
                ],
            },
            nominal_range: if dataset_missing {
                TimeRange::new(None, None)
            } else {
                TimeRange::new(epochs.first().copied(), epochs.last().copied())
            },
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
        snapshot_entries()
            .map(|entries| entries.iter().any(|entry| entry.body == body))
            .unwrap_or(false)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        if !matches!(req.instant.scale, TimeScale::Tt | TimeScale::Tdb) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedTimeScale,
                "the JPL snapshot backend only serves TT or TDB requests",
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

        if let Some(error) = snapshot_error() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::MissingDataset,
                format!("the JPL snapshot corpus could not be loaded: {error}"),
            ));
        }

        let entry = snapshot_entries()
            .and_then(|entries| {
                entries.iter().find(|entry| {
                    entry.body == req.body
                        && entry.epoch.julian_day.days() == req.instant.julian_day.days()
                })
            })
            .ok_or_else(|| {
                EphemerisError::new(
                    EphemerisErrorKind::OutOfRangeInstant,
                    "the requested body is not present in the JPL snapshot corpus at that instant",
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
    /// The epoch covered by the entry.
    pub epoch: Instant,
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

enum SnapshotState {
    Loaded(Vec<SnapshotEntry>),
    Failed(SnapshotLoadError),
}

impl SnapshotState {
    fn entries(&self) -> Option<&[SnapshotEntry]> {
        match self {
            Self::Loaded(entries) => Some(entries.as_slice()),
            Self::Failed(_) => None,
        }
    }

    fn error(&self) -> Option<&SnapshotLoadError> {
        match self {
            Self::Loaded(_) => None,
            Self::Failed(error) => Some(error),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SnapshotLoadError {
    line_number: usize,
    kind: SnapshotLoadErrorKind,
}

impl SnapshotLoadError {
    fn new(line_number: usize, kind: SnapshotLoadErrorKind) -> Self {
        Self { line_number, kind }
    }
}

impl fmt::Display for SnapshotLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line_number, self.kind)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SnapshotLoadErrorKind {
    MissingColumn { column: &'static str },
    UnexpectedExtraColumns,
    UnsupportedBody { body: String },
    InvalidNumber { column: &'static str, value: String },
}

impl fmt::Display for SnapshotLoadErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingColumn { column } => write!(f, "missing {column} column"),
            Self::UnexpectedExtraColumns => f.write_str("unexpected extra columns"),
            Self::UnsupportedBody { body } => write!(f, "unsupported body '{body}'"),
            Self::InvalidNumber { column, value } => {
                write!(f, "invalid {column} value '{value}'")
            }
        }
    }
}

fn snapshot_state() -> &'static SnapshotState {
    static STATE: OnceLock<SnapshotState> = OnceLock::new();
    STATE.get_or_init(|| match load_snapshot() {
        Ok(entries) => SnapshotState::Loaded(entries),
        Err(error) => SnapshotState::Failed(error),
    })
}

fn snapshot_entries() -> Option<&'static [SnapshotEntry]> {
    snapshot_state().entries()
}

fn snapshot_error() -> Option<&'static SnapshotLoadError> {
    snapshot_state().error()
}

fn snapshot_bodies() -> &'static [pleiades_backend::CelestialBody] {
    static BODIES: OnceLock<Vec<pleiades_backend::CelestialBody>> = OnceLock::new();
    BODIES
        .get_or_init(|| {
            let mut bodies = Vec::new();
            if let Some(entries) = snapshot_entries() {
                for entry in entries {
                    if !bodies.contains(&entry.body) {
                        bodies.push(entry.body.clone());
                    }
                }
            }
            bodies
        })
        .as_slice()
}

fn comparison_snapshot_entries() -> &'static [SnapshotEntry] {
    static SNAPSHOT: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    SNAPSHOT
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| is_comparison_body(&entry.body))
                .cloned()
                .collect()
        })
        .as_slice()
}

fn comparison_body_list() -> &'static [pleiades_backend::CelestialBody] {
    static BODIES: OnceLock<Vec<pleiades_backend::CelestialBody>> = OnceLock::new();
    BODIES
        .get_or_init(|| {
            let mut bodies = Vec::new();
            for entry in comparison_snapshot_entries() {
                if !bodies.contains(&entry.body) {
                    bodies.push(entry.body.clone());
                }
            }
            bodies
        })
        .as_slice()
}

fn reference_asteroid_list() -> &'static [pleiades_backend::CelestialBody] {
    static BODIES: OnceLock<Vec<pleiades_backend::CelestialBody>> = OnceLock::new();
    BODIES
        .get_or_init(|| {
            let mut bodies = Vec::new();
            for entry in snapshot_entries().into_iter().flatten() {
                if is_reference_asteroid(&entry.body) && !bodies.contains(&entry.body) {
                    bodies.push(entry.body.clone());
                }
            }
            bodies
        })
        .as_slice()
}

fn snapshot_instants() -> &'static [Instant] {
    static INSTANTS: OnceLock<Vec<Instant>> = OnceLock::new();
    INSTANTS
        .get_or_init(|| {
            let mut instants = Vec::new();
            if let Some(entries) = snapshot_entries() {
                for entry in entries {
                    if !instants.contains(&entry.epoch) {
                        instants.push(entry.epoch);
                    }
                }
            }
            instants
        })
        .as_slice()
}

fn load_snapshot() -> Result<Vec<SnapshotEntry>, SnapshotLoadError> {
    load_snapshot_from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/reference_snapshot.csv"
    )))
}

fn load_snapshot_from_str(source: &str) -> Result<Vec<SnapshotEntry>, SnapshotLoadError> {
    source
        .lines()
        .enumerate()
        .map(|(index, line)| parse_snapshot_line(index + 1, line))
        .try_fold(Vec::new(), |mut entries, record| {
            if let Some(entry) = record? {
                entries.push(entry);
            }
            Ok(entries)
        })
}

fn parse_snapshot_line(
    line_number: usize,
    line: &str,
) -> Result<Option<SnapshotEntry>, SnapshotLoadError> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(None);
    }

    let mut parts = trimmed.split(',').map(str::trim);
    let epoch_jd = next_part(&mut parts, line_number, "epoch")?;
    let body = next_part(&mut parts, line_number, "body")?;
    let x_km = next_part(&mut parts, line_number, "x")?;
    let y_km = next_part(&mut parts, line_number, "y")?;
    let z_km = next_part(&mut parts, line_number, "z")?;

    if parts.next().is_some() {
        return Err(SnapshotLoadError::new(
            line_number,
            SnapshotLoadErrorKind::UnexpectedExtraColumns,
        ));
    }

    Ok(Some(SnapshotEntry {
        body: parse_body(body, line_number)?,
        epoch: Instant::new(
            JulianDay::from_days(parse_f64(epoch_jd, line_number, "epoch_jd")?),
            TimeScale::Tdb,
        ),
        x_km: parse_f64(x_km, line_number, "x_km")?,
        y_km: parse_f64(y_km, line_number, "y_km")?,
        z_km: parse_f64(z_km, line_number, "z_km")?,
    }))
}

fn next_part<'a>(
    parts: &mut impl Iterator<Item = &'a str>,
    line_number: usize,
    column: &'static str,
) -> Result<&'a str, SnapshotLoadError> {
    parts.next().ok_or_else(|| {
        SnapshotLoadError::new(line_number, SnapshotLoadErrorKind::MissingColumn { column })
    })
}

fn parse_body(
    body: &str,
    line_number: usize,
) -> Result<pleiades_backend::CelestialBody, SnapshotLoadError> {
    let body = match body {
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
        "Ceres" => pleiades_backend::CelestialBody::Ceres,
        "Pallas" => pleiades_backend::CelestialBody::Pallas,
        "Juno" => pleiades_backend::CelestialBody::Juno,
        "Vesta" => pleiades_backend::CelestialBody::Vesta,
        other => {
            return Err(SnapshotLoadError::new(
                line_number,
                SnapshotLoadErrorKind::UnsupportedBody {
                    body: other.to_string(),
                },
            ))
        }
    };

    Ok(body)
}

fn is_comparison_body(body: &pleiades_backend::CelestialBody) -> bool {
    matches!(
        body,
        pleiades_backend::CelestialBody::Sun
            | pleiades_backend::CelestialBody::Moon
            | pleiades_backend::CelestialBody::Mercury
            | pleiades_backend::CelestialBody::Venus
            | pleiades_backend::CelestialBody::Mars
            | pleiades_backend::CelestialBody::Jupiter
            | pleiades_backend::CelestialBody::Saturn
            | pleiades_backend::CelestialBody::Uranus
            | pleiades_backend::CelestialBody::Neptune
            | pleiades_backend::CelestialBody::Pluto
    )
}

fn is_reference_asteroid(body: &pleiades_backend::CelestialBody) -> bool {
    matches!(
        body,
        pleiades_backend::CelestialBody::Ceres
            | pleiades_backend::CelestialBody::Pallas
            | pleiades_backend::CelestialBody::Juno
            | pleiades_backend::CelestialBody::Vesta
    )
}

fn parse_f64(
    value: &str,
    line_number: usize,
    column: &'static str,
) -> Result<f64, SnapshotLoadError> {
    value.parse::<f64>().map_err(|_error| {
        SnapshotLoadError::new(
            line_number,
            SnapshotLoadErrorKind::InvalidNumber {
                column,
                value: value.to_string(),
            },
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::{Apparentness, EphemerisRequest};

    #[test]
    fn reference_snapshot_covers_the_expected_bodies_and_epochs() {
        let metadata = JplSnapshotBackend::new().metadata();
        assert!(metadata
            .body_coverage
            .contains(&pleiades_backend::CelestialBody::Sun));
        assert!(metadata
            .body_coverage
            .contains(&pleiades_backend::CelestialBody::Moon));
        assert!(metadata
            .body_coverage
            .contains(&pleiades_backend::CelestialBody::Pluto));
        assert!(metadata
            .body_coverage
            .contains(&pleiades_backend::CelestialBody::Ceres));
        assert!(metadata
            .body_coverage
            .contains(&pleiades_backend::CelestialBody::Pallas));
        assert!(metadata
            .body_coverage
            .contains(&pleiades_backend::CelestialBody::Juno));
        assert!(metadata
            .body_coverage
            .contains(&pleiades_backend::CelestialBody::Vesta));
        assert_eq!(
            reference_asteroids(),
            [
                pleiades_backend::CelestialBody::Ceres,
                pleiades_backend::CelestialBody::Pallas,
                pleiades_backend::CelestialBody::Juno,
                pleiades_backend::CelestialBody::Vesta,
            ]
        );
        assert!(metadata.nominal_range.start.is_some());
        assert!(metadata.nominal_range.end.is_some());
        let start = metadata
            .nominal_range
            .start
            .expect("start epoch should exist");
        let end = metadata.nominal_range.end.expect("end epoch should exist");
        assert!(start.julian_day.days() < end.julian_day.days());
        assert_eq!(reference_epochs().len(), 3);
    }

    #[test]
    fn parser_reports_malformed_rows_without_panicking() {
        let error = load_snapshot_from_str("2451545.0,Sun,1.0,2.0\n")
            .expect_err("missing columns should be reported");
        assert!(format!("{error}").contains("missing z"));

        let error = load_snapshot_from_str("2451545.0,Comet,1.0,2.0,3.0\n")
            .expect_err("unsupported bodies should be reported");
        assert!(format!("{error}").contains("unsupported body 'Comet'"));
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
            .find(|entry| {
                entry.body == pleiades_backend::CelestialBody::Sun
                    && entry.epoch.julian_day.days() == REFERENCE_EPOCH_JD
            })
            .expect("sun entry should exist at J2000");

        let longitude = entry.ecliptic().longitude.degrees();
        assert!((longitude - 280.3778227681435).abs() < 1e-9);
    }

    #[test]
    fn snapshot_backend_resolves_a_later_epoch() {
        let backend = JplSnapshotBackend;
        let request = EphemerisRequest {
            body: pleiades_backend::CelestialBody::Mars,
            instant: Instant::new(JulianDay::from_days(2_634_167.0), TimeScale::Tt),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        let result = backend
            .position(&request)
            .expect("reference snapshot should resolve at the later epoch");
        let ecliptic = result
            .ecliptic
            .expect("reference snapshot should include ecliptic coordinates");
        assert!(ecliptic.longitude.degrees().is_finite());
        assert!(ecliptic.latitude.degrees().is_finite());
    }

    #[test]
    fn snapshot_backend_resolves_ceres_at_j2000() {
        let backend = JplSnapshotBackend;
        let request = EphemerisRequest {
            body: pleiades_backend::CelestialBody::Ceres,
            instant: reference_instant(),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        let result = backend
            .position(&request)
            .expect("reference snapshot should resolve the asteroid entry");
        let ecliptic = result
            .ecliptic
            .expect("reference snapshot should include ecliptic coordinates");
        assert!((ecliptic.longitude.degrees() - 184.459642854516).abs() < 1e-12);
        assert!((ecliptic.latitude.degrees() - 11.838531252961646).abs() < 1e-12);
        assert!(
            (ecliptic.distance_au.expect("distance should exist") - 2.2568850705531642).abs()
                < 1e-12
        );
    }
}
