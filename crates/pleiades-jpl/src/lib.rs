//! JPL Horizons reference fixture backend for validation, comparison, and
//! selected asteroid support.
//!
//! This crate provides a narrow, source-backed backend based on a checked-in
//! JPL Horizons vector fixture. The backend serves exact states at fixture
//! epochs and uses quadratic interpolation on three-sample windows when it can,
//! falling back to linear interpolation between adjacent samples when only two
//! bracketing epochs exist. This intentionally small derivative format proves
//! the pure-Rust reader/interpolator path before larger public JPL-derived
//! corpora are added.
//!
//! The checked-in fixture also includes a small set of named asteroids and a
//! custom `catalog:designation` example so the shared body taxonomy can exercise
//! source-backed asteroid support without changing the comparison corpus used by
//! validation reports.

#![forbid(unsafe_code)]

use core::fmt;
use std::sync::OnceLock;

use pleiades_backend::{
    AccuracyClass, Apparentness, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
    BackendProvenance, EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest,
    EphemerisResult, QualityAnnotation,
};
use pleiades_types::{
    CoordinateFrame, CustomBodyId, EclipticCoordinates, Instant, JulianDay, Latitude, Longitude,
    Motion, TimeRange, TimeScale, ZodiacMode,
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

/// Returns the parsed reference fixture entries.
pub fn reference_snapshot() -> &'static [SnapshotEntry] {
    snapshot_entries().unwrap_or(&[])
}

/// Returns the source-backed asteroid subset present in the reference snapshot.
pub fn reference_asteroids() -> &'static [pleiades_backend::CelestialBody] {
    reference_asteroid_list()
}

/// Exact J2000 asteroid reference samples from the checked-in snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceAsteroidEvidence {
    /// Asteroid body covered by the exact snapshot row.
    pub body: pleiades_backend::CelestialBody,
    /// Exact epoch used for the reference sample.
    pub epoch: Instant,
    /// Ecliptic longitude in degrees.
    pub longitude_deg: f64,
    /// Ecliptic latitude in degrees.
    pub latitude_deg: f64,
    /// Geocentric distance in astronomical units.
    pub distance_au: f64,
}

/// Returns the exact J2000 asteroid evidence samples present in the reference snapshot.
pub fn reference_asteroid_evidence() -> &'static [ReferenceAsteroidEvidence] {
    reference_asteroid_evidence_list()
}

/// Returns the comparison-only subset used by the stage-4 validation corpus.
pub fn comparison_snapshot() -> &'static [SnapshotEntry] {
    comparison_snapshot_entries()
}

/// Returns the comparison-only body coverage used by validation tooling.
pub fn comparison_bodies() -> &'static [pleiades_backend::CelestialBody] {
    comparison_body_list()
}

/// Returns coarse leave-one-out interpolation checks derived from the checked-in
/// fixture.
///
/// Each sample treats a middle exact fixture epoch as a held-out point and
/// linearly interpolates from the nearest earlier and later same-body fixture
/// entries. The current fixture is intentionally sparse, so these values are
/// evidence for report transparency rather than production interpolation
/// tolerances.
///
/// Note that these samples intentionally remain linear counterfactuals even if
/// the runtime backend later uses a higher-order interpolation fallback.
pub fn interpolation_quality_samples() -> &'static [InterpolationQualitySample] {
    interpolation_quality_sample_list()
}

/// A coarse hold-out check for the snapshot backend's linear counterfactual path.
#[derive(Clone, Debug, PartialEq)]
pub struct InterpolationQualitySample {
    /// Body evaluated by this check.
    pub body: pleiades_backend::CelestialBody,
    /// Held-out exact epoch used for comparison.
    pub epoch: Instant,
    /// Span between the bracketing fixture entries in days.
    pub bracket_span_days: f64,
    /// Absolute wrapped longitude error in degrees.
    pub longitude_error_deg: f64,
    /// Absolute latitude error in degrees.
    pub latitude_error_deg: f64,
    /// Absolute distance error in astronomical units.
    pub distance_error_au: f64,
}

/// A reference-backend implementation backed by JPL Horizons fixture data.
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
                summary: "NASA/JPL Horizons DE441 geocentric ecliptic fixture with exact epoch lookup and quadratic interpolation on three-sample windows"
                    .to_string(),
                data_sources: vec![
                    "NASA/JPL Horizons API vector tables (DE441)".to_string(),
                    "Checked-in derivative CSV fixture: epoch_jd,body,x_km,y_km,z_km".to_string(),
                    "Quadratic interpolation on three-sample windows with linear fallback between adjacent same-body fixture samples".to_string(),
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

        let resolved = resolve_fixture_state(req.body.clone(), req.instant.julian_day.days())?;

        let mut result = EphemerisResult::new(
            BackendId::new("jpl-snapshot"),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        result.ecliptic = Some(resolved.entry.ecliptic());
        result.motion = None::<Motion>;
        result.quality = resolved.quality;
        Ok(result)
    }
}

/// One parsed record from the reference fixture.
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

    fn interpolate_linear(before: &Self, after: &Self, epoch_jd: f64) -> Self {
        let span_days = after.epoch.julian_day.days() - before.epoch.julian_day.days();
        let fraction = (epoch_jd - before.epoch.julian_day.days()) / span_days;
        Self {
            body: before.body.clone(),
            epoch: Instant::new(JulianDay::from_days(epoch_jd), TimeScale::Tdb),
            x_km: lerp(before.x_km, after.x_km, fraction),
            y_km: lerp(before.y_km, after.y_km, fraction),
            z_km: lerp(before.z_km, after.z_km, fraction),
        }
    }

    fn interpolate_quadratic(a: &Self, b: &Self, c: &Self, epoch_jd: f64) -> Self {
        let xs = [
            a.epoch.julian_day.days(),
            b.epoch.julian_day.days(),
            c.epoch.julian_day.days(),
        ];
        Self {
            body: a.body.clone(),
            epoch: Instant::new(JulianDay::from_days(epoch_jd), TimeScale::Tdb),
            x_km: lagrange_interpolate_3(epoch_jd, xs, [a.x_km, b.x_km, c.x_km]),
            y_km: lagrange_interpolate_3(epoch_jd, xs, [a.y_km, b.y_km, c.y_km]),
            z_km: lagrange_interpolate_3(epoch_jd, xs, [a.z_km, b.z_km, c.z_km]),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ResolvedFixtureState {
    entry: SnapshotEntry,
    quality: QualityAnnotation,
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

fn lerp(start: f64, end: f64, fraction: f64) -> f64 {
    start + (end - start) * fraction
}

fn lagrange_interpolate_3(x: f64, xs: [f64; 3], ys: [f64; 3]) -> f64 {
    let [x0, x1, x2] = xs;
    let [y0, y1, y2] = ys;

    let l0 = (x - x1) * (x - x2) / ((x0 - x1) * (x0 - x2));
    let l1 = (x - x0) * (x - x2) / ((x1 - x0) * (x1 - x2));
    let l2 = (x - x0) * (x - x1) / ((x2 - x0) * (x2 - x1));

    y0 * l0 + y1 * l1 + y2 * l2
}

fn interpolate_fixture_state(
    entries: &[SnapshotEntry],
    body: pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> Option<SnapshotEntry> {
    let mut body_entries = entries
        .iter()
        .filter(|entry| entry.body == body)
        .collect::<Vec<_>>();

    if body_entries.len() < 3 {
        return None;
    }

    body_entries.sort_by(|left, right| {
        left.epoch
            .julian_day
            .days()
            .partial_cmp(&right.epoch.julian_day.days())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut ranked = body_entries
        .into_iter()
        .map(|entry| ((entry.epoch.julian_day.days() - epoch_jd).abs(), entry))
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        left.0
            .partial_cmp(&right.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                left.1
                    .epoch
                    .julian_day
                    .days()
                    .partial_cmp(&right.1.epoch.julian_day.days())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    let mut selected = ranked
        .into_iter()
        .take(3)
        .map(|(_, entry)| entry)
        .collect::<Vec<_>>();

    if selected.len() == 3 {
        selected.sort_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .partial_cmp(&right.epoch.julian_day.days())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        return Some(SnapshotEntry::interpolate_quadratic(
            selected[0],
            selected[1],
            selected[2],
            epoch_jd,
        ));
    }

    None
}

fn angular_degrees_delta(left: f64, right: f64) -> f64 {
    let delta = (left - right + 180.0).rem_euclid(360.0) - 180.0;
    delta.abs()
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

fn reference_asteroid_evidence_list() -> &'static [ReferenceAsteroidEvidence] {
    static EVIDENCE: OnceLock<Vec<ReferenceAsteroidEvidence>> = OnceLock::new();
    EVIDENCE
        .get_or_init(|| {
            let mut evidence = Vec::new();
            let Some(entries) = snapshot_entries() else {
                return evidence;
            };

            for body in reference_asteroid_list() {
                if let Some(entry) = entries.iter().find(|entry| {
                    &entry.body == body && entry.epoch.julian_day.days() == REFERENCE_EPOCH_JD
                }) {
                    let ecliptic = entry.ecliptic();
                    evidence.push(ReferenceAsteroidEvidence {
                        body: body.clone(),
                        epoch: entry.epoch,
                        longitude_deg: ecliptic.longitude.degrees(),
                        latitude_deg: ecliptic.latitude.degrees(),
                        distance_au: ecliptic.distance_au.unwrap_or_default(),
                    });
                }
            }

            evidence
        })
        .as_slice()
}

fn interpolation_quality_sample_list() -> &'static [InterpolationQualitySample] {
    static SAMPLES: OnceLock<Vec<InterpolationQualitySample>> = OnceLock::new();
    SAMPLES
        .get_or_init(|| {
            let mut samples = Vec::new();
            let Some(entries) = snapshot_entries() else {
                return samples;
            };

            for body in comparison_body_list() {
                let mut body_entries = entries
                    .iter()
                    .filter(|entry| &entry.body == body)
                    .collect::<Vec<_>>();
                body_entries.sort_by(|left, right| {
                    left.epoch
                        .julian_day
                        .days()
                        .partial_cmp(&right.epoch.julian_day.days())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                for window in body_entries.windows(3) {
                    let before = window[0];
                    let exact = window[1];
                    let after = window[2];
                    let epoch_jd = exact.epoch.julian_day.days();
                    let interpolated = SnapshotEntry::interpolate_linear(before, after, epoch_jd);
                    let exact_ecliptic = exact.ecliptic();
                    let interpolated_ecliptic = interpolated.ecliptic();
                    let exact_distance = exact_ecliptic.distance_au.unwrap_or_default();
                    let interpolated_distance =
                        interpolated_ecliptic.distance_au.unwrap_or_default();

                    samples.push(InterpolationQualitySample {
                        body: exact.body.clone(),
                        epoch: exact.epoch,
                        bracket_span_days: after.epoch.julian_day.days()
                            - before.epoch.julian_day.days(),
                        longitude_error_deg: angular_degrees_delta(
                            exact_ecliptic.longitude.degrees(),
                            interpolated_ecliptic.longitude.degrees(),
                        ),
                        latitude_error_deg: (exact_ecliptic.latitude.degrees()
                            - interpolated_ecliptic.latitude.degrees())
                        .abs(),
                        distance_error_au: (exact_distance - interpolated_distance).abs(),
                    });
                }
            }

            samples
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

fn resolve_fixture_state(
    body: pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> Result<ResolvedFixtureState, EphemerisError> {
    let Some(entries) = snapshot_entries() else {
        return Err(EphemerisError::new(
            EphemerisErrorKind::MissingDataset,
            "the JPL fixture corpus is unavailable",
        ));
    };

    let mut exact = None;
    let mut before = None;
    let mut after = None;
    let mut body_seen = false;

    for entry in entries.iter().filter(|entry| entry.body == body) {
        body_seen = true;
        let entry_jd = entry.epoch.julian_day.days();
        if entry_jd == epoch_jd {
            exact = Some(entry);
            break;
        }
        if entry_jd < epoch_jd
            && before.is_none_or(|candidate: &SnapshotEntry| {
                entry_jd > candidate.epoch.julian_day.days()
            })
        {
            before = Some(entry);
        }
        if entry_jd > epoch_jd
            && after.is_none_or(|candidate: &SnapshotEntry| {
                entry_jd < candidate.epoch.julian_day.days()
            })
        {
            after = Some(entry);
        }
    }

    if let Some(entry) = exact {
        return Ok(ResolvedFixtureState {
            entry: entry.clone(),
            quality: QualityAnnotation::Exact,
        });
    }

    if !body_seen {
        return Err(EphemerisError::new(
            EphemerisErrorKind::UnsupportedBody,
            format!("the JPL fixture corpus does not include {body}"),
        ));
    }

    if before.is_some() && after.is_some() {
        if let Some(entry) = interpolate_fixture_state(entries, body.clone(), epoch_jd) {
            return Ok(ResolvedFixtureState {
                entry,
                quality: QualityAnnotation::Interpolated,
            });
        }
    }

    match (before, after) {
        (Some(before), Some(after)) => Ok(ResolvedFixtureState {
            entry: SnapshotEntry::interpolate_linear(before, after, epoch_jd),
            quality: QualityAnnotation::Interpolated,
        }),
        _ => Err(EphemerisError::new(
            EphemerisErrorKind::OutOfRangeInstant,
            "the requested instant is outside adjacent JPL fixture samples for that body",
        )),
    }
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
            let Some((catalog, designation)) = other.split_once(':') else {
                return Err(SnapshotLoadError::new(
                    line_number,
                    SnapshotLoadErrorKind::UnsupportedBody {
                        body: other.to_string(),
                    },
                ));
            };

            let catalog = catalog.trim();
            let designation = designation.trim();
            if catalog.is_empty() || designation.is_empty() {
                return Err(SnapshotLoadError::new(
                    line_number,
                    SnapshotLoadErrorKind::UnsupportedBody {
                        body: other.to_string(),
                    },
                ));
            }

            pleiades_backend::CelestialBody::Custom(CustomBodyId::new(catalog, designation))
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
    match body {
        pleiades_backend::CelestialBody::Ceres
        | pleiades_backend::CelestialBody::Pallas
        | pleiades_backend::CelestialBody::Juno
        | pleiades_backend::CelestialBody::Vesta => true,
        pleiades_backend::CelestialBody::Custom(custom) if custom.catalog == "asteroid" => true,
        _ => false,
    }
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
    use pleiades_backend::{Apparentness, EphemerisErrorKind, EphemerisRequest};

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
        assert!(metadata
            .body_coverage
            .contains(&pleiades_backend::CelestialBody::Custom(CustomBodyId::new(
                "asteroid", "433-Eros"
            ))));
        assert_eq!(
            reference_asteroids(),
            [
                pleiades_backend::CelestialBody::Ceres,
                pleiades_backend::CelestialBody::Pallas,
                pleiades_backend::CelestialBody::Juno,
                pleiades_backend::CelestialBody::Vesta,
                pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
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
        assert_eq!(reference_epochs().len(), 4);
        assert_eq!(
            reference_snapshot()
                .iter()
                .filter(|entry| entry.epoch.julian_day.days() == 2_500_000.0)
                .count(),
            10
        );
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
    fn parser_accepts_custom_catalog_bodies() {
        let snapshot = load_snapshot_from_str("2451545.0,asteroid:433-Eros,-1.0,-2.0,-3.0\n")
            .expect("custom catalog bodies should parse");
        assert_eq!(snapshot.len(), 1);
        assert_eq!(
            snapshot[0].body,
            pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
        );
    }

    #[test]
    fn quadratic_interpolation_matches_a_known_parabola() {
        let a = SnapshotEntry {
            body: pleiades_backend::CelestialBody::Moon,
            epoch: Instant::new(JulianDay::from_days(0.0), TimeScale::Tdb),
            x_km: 0.0,
            y_km: 1.0,
            z_km: 2.0,
        };
        let b = SnapshotEntry {
            body: pleiades_backend::CelestialBody::Moon,
            epoch: Instant::new(JulianDay::from_days(1.0), TimeScale::Tdb),
            x_km: 1.0,
            y_km: 6.0,
            z_km: 5.0,
        };
        let c = SnapshotEntry {
            body: pleiades_backend::CelestialBody::Moon,
            epoch: Instant::new(JulianDay::from_days(2.0), TimeScale::Tdb),
            x_km: 4.0,
            y_km: 15.0,
            z_km: 10.0,
        };

        let interpolated = SnapshotEntry::interpolate_quadratic(&a, &b, &c, 1.5);
        assert!((interpolated.x_km - 2.25).abs() < 1e-12);
        assert!((interpolated.y_km - 10.0).abs() < 1e-12);
        assert!((interpolated.z_km - 7.25).abs() < 1e-12);
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
            .expect("reference fixture should resolve at the later epoch");
        assert_eq!(result.quality, QualityAnnotation::Exact);
        let ecliptic = result
            .ecliptic
            .expect("reference fixture should include ecliptic coordinates");
        assert!(ecliptic.longitude.degrees().is_finite());
        assert!(ecliptic.latitude.degrees().is_finite());
    }

    #[test]
    fn snapshot_backend_interpolates_between_fixture_epochs() {
        let backend = JplSnapshotBackend;
        let request = EphemerisRequest {
            body: pleiades_backend::CelestialBody::Mars,
            instant: Instant::new(JulianDay::from_days(2_415_022.0), TimeScale::Tdb),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        let result = backend
            .position(&request)
            .expect("reference fixture should interpolate between Mars samples");
        assert_eq!(result.quality, QualityAnnotation::Interpolated);
        let ecliptic = result
            .ecliptic
            .expect("interpolated fixture should include ecliptic coordinates");
        assert!(ecliptic.longitude.degrees().is_finite());
        assert!(ecliptic.latitude.degrees().is_finite());
        assert!(ecliptic
            .distance_au
            .expect("distance should exist")
            .is_finite());
    }

    #[test]
    fn interpolation_quality_samples_are_reportable() {
        let samples = interpolation_quality_samples();
        assert_eq!(samples.len(), 10);
        assert!(samples.iter().all(|sample| {
            let epoch = sample.epoch.julian_day.days();
            (epoch == REFERENCE_EPOCH_JD || epoch == 2_500_000.0)
                && sample.bracket_span_days > 0.0
                && sample.longitude_error_deg.is_finite()
                && sample.latitude_error_deg.is_finite()
                && sample.distance_error_au.is_finite()
        }));
        assert!(samples
            .iter()
            .any(|sample| sample.epoch.julian_day.days() == 2_500_000.0));
        assert!(samples
            .iter()
            .any(|sample| sample.body == pleiades_backend::CelestialBody::Mars));
    }

    #[test]
    fn snapshot_backend_distinguishes_unsupported_body_from_out_of_range() {
        let backend = JplSnapshotBackend;
        let unsupported = EphemerisRequest {
            body: pleiades_backend::CelestialBody::MeanNode,
            instant: reference_instant(),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };
        let error = backend
            .position(&unsupported)
            .expect_err("missing bodies should not be reported as date-range errors");
        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);

        let out_of_range = EphemerisRequest {
            body: pleiades_backend::CelestialBody::Ceres,
            instant: Instant::new(JulianDay::from_days(2_451_546.0), TimeScale::Tdb),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };
        let error = backend
            .position(&out_of_range)
            .expect_err("single-epoch bodies should report out-of-range requests");
        assert_eq!(error.kind, EphemerisErrorKind::OutOfRangeInstant);
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

    #[test]
    fn snapshot_backend_resolves_named_asteroids_at_j2000() {
        let backend = JplSnapshotBackend;
        let cases = [
            (
                pleiades_backend::CelestialBody::Pallas,
                134.04575066840783,
                -48.351081494304466,
                1.4371532489145409,
            ),
            (
                pleiades_backend::CelestialBody::Juno,
                278.008461932084,
                9.450859010610209,
                4.084400792647673,
            ),
            (
                pleiades_backend::CelestialBody::Vesta,
                245.98418908965346,
                4.251902812654469,
                2.898586893865609,
            ),
            (
                pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
                236.28757472178148,
                -7.734019866618642,
                1.854402724550437,
            ),
        ];

        for (body, expected_longitude_deg, expected_latitude_deg, expected_distance_au) in cases {
            let request = EphemerisRequest {
                body,
                instant: reference_instant(),
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            };

            let result = backend
                .position(&request)
                .expect("reference snapshot should resolve the asteroid entry");
            assert_eq!(result.quality, QualityAnnotation::Exact);

            let ecliptic = result
                .ecliptic
                .expect("reference snapshot should include ecliptic coordinates");
            assert!((ecliptic.longitude.degrees() - expected_longitude_deg).abs() < 1e-12);
            assert!((ecliptic.latitude.degrees() - expected_latitude_deg).abs() < 1e-12);
            assert!(
                (ecliptic.distance_au.expect("distance should exist") - expected_distance_au).abs()
                    < 1e-12
            );
        }
    }

    #[test]
    fn reference_asteroid_evidence_exposes_exact_j2000_samples() {
        let evidence = reference_asteroid_evidence();
        assert_eq!(evidence.len(), 5);
        assert_eq!(reference_asteroids().len(), evidence.len());
        assert!(evidence.iter().all(|sample| {
            sample.epoch.julian_day.days() == REFERENCE_EPOCH_JD
                && sample.longitude_deg.is_finite()
                && sample.latitude_deg.is_finite()
                && sample.distance_au.is_finite()
        }));
        assert_eq!(evidence[0].body, pleiades_backend::CelestialBody::Ceres);
        assert_eq!(evidence[1].body, pleiades_backend::CelestialBody::Pallas);
        assert_eq!(evidence[2].body, pleiades_backend::CelestialBody::Juno);
        assert_eq!(evidence[3].body, pleiades_backend::CelestialBody::Vesta);
        assert_eq!(
            evidence[4].body,
            pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
        );
        assert!((evidence[0].longitude_deg - 184.459642854516).abs() < 1e-12);
        assert!((evidence[4].distance_au - 1.854402724550437).abs() < 1e-12);
    }

    #[test]
    fn snapshot_backend_resolves_custom_asteroid_at_j2000() {
        let backend = JplSnapshotBackend;
        let request = EphemerisRequest {
            body: pleiades_backend::CelestialBody::Custom(CustomBodyId::new(
                "asteroid", "433-Eros",
            )),
            instant: reference_instant(),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        let result = backend
            .position(&request)
            .expect("reference snapshot should resolve the custom asteroid entry");
        assert_eq!(result.quality, QualityAnnotation::Exact);
        let ecliptic = result
            .ecliptic
            .expect("reference snapshot should include ecliptic coordinates");
        assert!((ecliptic.longitude.degrees() - 236.28757472178148).abs() < 1e-12);
        assert!((ecliptic.latitude.degrees() - (-7.734019866618642)).abs() < 1e-12);
        assert!(
            (ecliptic.distance_au.expect("distance should exist") - 1.854402724550437).abs()
                < 1e-12
        );
    }
}
