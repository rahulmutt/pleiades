//! Packaged compressed ephemeris backend for the common 1500-2500 range.
//!
//! This crate now ships a small stage-5 prototype artifact backed by the
//! `pleiades-compression` codec. The bundled data covers the comparison-body
//! planetary set plus the source-backed custom asteroid `asteroid:433-Eros`
//! via quantized linear segments fitted to checked-in reference epochs, and the
//! backend falls back to other providers when callers request bodies outside
//! that packaged slice.

#![forbid(unsafe_code)]

use std::cmp::Ordering;
use std::sync::OnceLock;

use pleiades_backend::{
    validate_observer_policy, validate_request_policy, AccuracyClass, BackendCapabilities,
    BackendFamily, BackendId, BackendMetadata, BackendProvenance, CelestialBody, CoordinateFrame,
    CustomBodyId, EclipticCoordinates, EphemerisBackend, EphemerisError, EphemerisErrorKind,
    EphemerisRequest, EphemerisResult, Instant, QualityAnnotation, TimeRange, TimeScale,
    ZodiacMode,
};
use pleiades_compression::{
    ArtifactHeader, BodyArtifact, ChannelKind, CompressedArtifact, PolynomialChannel, Segment,
};
use pleiades_jpl::{reference_snapshot, SnapshotEntry};

const PACKAGE_NAME: &str = "pleiades-data";
const ARTIFACT_LABEL: &str = "stage-5 packaged-data prototype";
const ARTIFACT_SOURCE: &str = "Quantized linear segments fitted to JPL Horizons reference epochs (1800, 2000, 2500 CE) for the comparison-body planetary set plus asteroid:433-Eros, with J2000 point segments for the outer planets, Pluto, and the asteroid coverage.";
const PACKAGED_BASE_BODIES: [CelestialBody; 10] = [
    CelestialBody::Sun,
    CelestialBody::Moon,
    CelestialBody::Mercury,
    CelestialBody::Venus,
    CelestialBody::Mars,
    CelestialBody::Jupiter,
    CelestialBody::Saturn,
    CelestialBody::Uranus,
    CelestialBody::Neptune,
    CelestialBody::Pluto,
];

fn packaged_bodies() -> &'static [CelestialBody] {
    static BODIES: OnceLock<Vec<CelestialBody>> = OnceLock::new();
    BODIES.get_or_init(|| {
        let mut bodies = PACKAGED_BASE_BODIES.to_vec();
        bodies.push(CelestialBody::Custom(CustomBodyId::new(
            "asteroid", "433-Eros",
        )));
        bodies
    })
}
const AU_IN_KM: f64 = 149_597_870.7;

/// Returns the canonical package name for this crate.
pub const fn package_name() -> &'static str {
    PACKAGE_NAME
}

/// Returns the bundled packed artifact.
pub fn packaged_artifact() -> &'static CompressedArtifact {
    static ARTIFACT: OnceLock<CompressedArtifact> = OnceLock::new();
    ARTIFACT.get_or_init(build_packaged_artifact)
}

/// Returns a packaged lookup for a body and instant.
pub fn packaged_lookup(
    body: &CelestialBody,
    instant: Instant,
) -> Result<EclipticCoordinates, pleiades_compression::CompressionError> {
    packaged_artifact().lookup_ecliptic(body, normalize_lookup_instant(instant))
}

/// Returns a packaged-data backend instance.
pub fn packaged_backend() -> PackagedDataBackend {
    PackagedDataBackend::new()
}

/// A packaged compressed-data backend.
#[derive(Debug, Default, Clone, Copy)]
pub struct PackagedDataBackend;

impl PackagedDataBackend {
    /// Creates a new packaged-data backend.
    pub const fn new() -> Self {
        Self
    }
}

impl EphemerisBackend for PackagedDataBackend {
    fn metadata(&self) -> BackendMetadata {
        let artifact = packaged_artifact();
        let bodies = artifact
            .bodies
            .iter()
            .map(|series| series.body.clone())
            .collect::<Vec<_>>();
        let range = artifact_time_range(artifact);

        BackendMetadata {
            id: BackendId::new(PACKAGE_NAME),
            version: format!(
                "{} checksum:{:016x}",
                artifact.header.version, artifact.checksum
            ),
            family: BackendFamily::CompressedData,
            provenance: BackendProvenance {
                summary: artifact.header.source.clone(),
                data_sources: vec![
                    "Checked-in JPL Horizons reference epochs (Sun, Moon, and asteroid:433-Eros)"
                        .to_string(),
                    "Quantized linear segments stored in pleiades-compression artifact format"
                        .to_string(),
                ],
            },
            nominal_range: range,
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
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        packaged_artifact()
            .bodies
            .iter()
            .any(|series| series.body == body)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        if !matches!(req.instant.scale, TimeScale::Tt | TimeScale::Tdb) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedTimeScale,
                "packaged data only supports TT or TDB requests",
            ));
        }

        validate_request_policy(
            req,
            "packaged data",
            &[TimeScale::Tt, TimeScale::Tdb],
            &[CoordinateFrame::Ecliptic],
            false,
        )?;

        if req.zodiac_mode != ZodiacMode::Tropical {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "sidereal conversion is handled above the packaged-data backend",
            ));
        }

        validate_observer_policy(req, "packaged data", false)?;

        let ecliptic = packaged_artifact()
            .lookup_ecliptic(&req.body, normalize_lookup_instant(req.instant))
            .map_err(map_artifact_error)?;

        let mut result = EphemerisResult::new(
            BackendId::new(PACKAGE_NAME),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        result.ecliptic = Some(ecliptic);
        result.quality = QualityAnnotation::Interpolated;
        Ok(result)
    }
}

fn build_packaged_artifact() -> CompressedArtifact {
    let mut artifact = CompressedArtifact::new(
        ArtifactHeader::new(ARTIFACT_LABEL, ARTIFACT_SOURCE),
        packaged_body_artifacts(),
    );
    artifact.checksum = artifact
        .checksum()
        .expect("packaged artifact checksum should be reproducible");
    artifact
}

fn packaged_body_artifacts() -> Vec<BodyArtifact> {
    let mut artifacts = Vec::new();
    let snapshot = reference_snapshot();

    for body in packaged_bodies().iter().cloned() {
        let mut entries: Vec<&SnapshotEntry> =
            snapshot.iter().filter(|entry| entry.body == body).collect();
        if entries.is_empty() {
            continue;
        }

        entries.sort_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .partial_cmp(&right.epoch.julian_day.days())
                .unwrap_or(Ordering::Equal)
        });

        let segments = if entries.len() == 1 {
            vec![segment_from_entries(entries[0], entries[0])]
        } else {
            entries
                .windows(2)
                .map(|pair| segment_from_entries(pair[0], pair[1]))
                .collect()
        };

        artifacts.push(BodyArtifact::new(body, segments));
    }

    artifacts
}

fn segment_from_entries(start: &SnapshotEntry, end: &SnapshotEntry) -> Segment {
    let start_coordinates = coordinates(start);
    let end_coordinates = coordinates(end);
    Segment::new(
        Instant::new(start.epoch.julian_day, TimeScale::Tt),
        Instant::new(end.epoch.julian_day, TimeScale::Tt),
        vec![
            PolynomialChannel::linear(
                ChannelKind::Longitude,
                9,
                start_coordinates.longitude.degrees(),
                end_coordinates.longitude.degrees(),
            ),
            PolynomialChannel::linear(
                ChannelKind::Latitude,
                9,
                start_coordinates.latitude.degrees(),
                end_coordinates.latitude.degrees(),
            ),
            PolynomialChannel::linear(
                ChannelKind::DistanceAu,
                12,
                start_coordinates.distance_au.unwrap_or_default(),
                end_coordinates.distance_au.unwrap_or_default(),
            ),
        ],
    )
}

fn coordinates(entry: &SnapshotEntry) -> EclipticCoordinates {
    let radius_km =
        (entry.x_km * entry.x_km + entry.y_km * entry.y_km + entry.z_km * entry.z_km).sqrt();
    let longitude = entry.y_km.atan2(entry.x_km).to_degrees();
    let latitude = (entry.z_km / radius_km)
        .clamp(-1.0, 1.0)
        .asin()
        .to_degrees();
    EclipticCoordinates::new(
        pleiades_backend::Longitude::from_degrees(longitude),
        pleiades_backend::Latitude::from_degrees(latitude),
        Some(radius_km / AU_IN_KM),
    )
}

fn artifact_time_range(artifact: &CompressedArtifact) -> TimeRange {
    let mut start: Option<Instant> = None;
    let mut end: Option<Instant> = None;
    for body in &artifact.bodies {
        for segment in &body.segments {
            start = Some(match start {
                Some(current) => {
                    if segment.start.julian_day.days() < current.julian_day.days() {
                        segment.start
                    } else {
                        current
                    }
                }
                None => segment.start,
            });
            end = Some(match end {
                Some(current) => {
                    if segment.end.julian_day.days() > current.julian_day.days() {
                        segment.end
                    } else {
                        current
                    }
                }
                None => segment.end,
            });
        }
    }
    TimeRange::new(start, end)
}

fn normalize_lookup_instant(instant: Instant) -> Instant {
    match instant.scale {
        TimeScale::Tt => instant,
        TimeScale::Tdb => Instant::new(instant.julian_day, TimeScale::Tt),
        _ => instant,
    }
}

fn map_artifact_error(error: pleiades_compression::CompressionError) -> EphemerisError {
    let kind = match error.kind {
        pleiades_compression::CompressionErrorKind::MissingBody => {
            EphemerisErrorKind::UnsupportedBody
        }
        pleiades_compression::CompressionErrorKind::OutOfRangeInstant => {
            EphemerisErrorKind::OutOfRangeInstant
        }
        pleiades_compression::CompressionErrorKind::UnsupportedTimeScale => {
            EphemerisErrorKind::UnsupportedTimeScale
        }
        pleiades_compression::CompressionErrorKind::MissingChannel => {
            EphemerisErrorKind::MissingDataset
        }
        pleiades_compression::CompressionErrorKind::QuantizationOverflow
        | pleiades_compression::CompressionErrorKind::InvalidFormat
        | pleiades_compression::CompressionErrorKind::InvalidMagic
        | pleiades_compression::CompressionErrorKind::UnsupportedVersion
        | pleiades_compression::CompressionErrorKind::ChecksumMismatch
        | pleiades_compression::CompressionErrorKind::Truncated
        | _ => EphemerisErrorKind::NumericalFailure,
    };

    EphemerisError::new(kind, error.message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packaged_artifact_roundtrips_through_codec() {
        let artifact = packaged_artifact();
        let encoded = artifact.encode().expect("packaged artifact should encode");
        let decoded =
            CompressedArtifact::decode(&encoded).expect("packaged artifact should decode");
        assert_eq!(decoded.header.generation_label, ARTIFACT_LABEL);
        assert_eq!(decoded.bodies.len(), packaged_bodies().len());
        assert_eq!(decoded.checksum, artifact.checksum);
    }

    #[test]
    fn lookup_uses_packaged_segments() {
        let reference = reference_snapshot()
            .iter()
            .find(|entry| {
                entry.body == CelestialBody::Sun
                    && (entry.epoch.julian_day.days() - 2_451_545.0).abs() < f64::EPSILON
            })
            .expect("reference snapshot should include the Sun at J2000");
        let ecliptic = packaged_lookup(&CelestialBody::Sun, reference.epoch)
            .expect("packaged lookup should succeed");
        let expected = coordinates(reference);

        assert!((ecliptic.longitude.degrees() - expected.longitude.degrees()).abs() < 1e-8);
        assert!((ecliptic.latitude.degrees() - expected.latitude.degrees()).abs() < 1e-8);
        assert!((ecliptic.distance_au.unwrap() - expected.distance_au.unwrap()).abs() < 1e-12);
    }

    #[test]
    fn lookup_uses_packaged_custom_asteroid_segments() {
        let reference = reference_snapshot()
            .iter()
            .find(|entry| {
                entry.body == CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
                    && (entry.epoch.julian_day.days() - 2_451_545.0).abs() < f64::EPSILON
            })
            .expect("reference snapshot should include asteroid:433-Eros at J2000");
        let body = CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"));
        let ecliptic = packaged_lookup(&body, reference.epoch)
            .expect("packaged lookup should succeed for the custom asteroid");
        let expected = coordinates(reference);

        assert!((ecliptic.longitude.degrees() - expected.longitude.degrees()).abs() < 1e-8);
        assert!((ecliptic.latitude.degrees() - expected.latitude.degrees()).abs() < 1e-8);
        assert!((ecliptic.distance_au.unwrap() - expected.distance_au.unwrap()).abs() < 1e-12);
    }

    #[test]
    fn backend_metadata_exposes_packaged_scope() {
        let metadata = packaged_backend().metadata();
        assert_eq!(metadata.id.as_str(), PACKAGE_NAME);
        assert_eq!(metadata.family, BackendFamily::CompressedData);
        assert_eq!(
            packaged_artifact().header.profile.stored_channels,
            vec![
                ChannelKind::Longitude,
                ChannelKind::Latitude,
                ChannelKind::DistanceAu
            ]
        );
        assert_eq!(
            packaged_artifact().header.profile.speed_policy,
            pleiades_compression::SpeedPolicy::Unsupported
        );
        assert!(packaged_artifact()
            .header
            .profile
            .unsupported_outputs
            .contains(&pleiades_compression::ArtifactOutput::Motion));
        assert!(metadata.body_coverage.contains(&CelestialBody::Sun));
        assert!(metadata.body_coverage.contains(&CelestialBody::Moon));
        assert!(metadata.body_coverage.contains(&CelestialBody::Jupiter));
        assert!(metadata.body_coverage.contains(&CelestialBody::Pluto));
        assert!(metadata
            .body_coverage
            .contains(&CelestialBody::Custom(CustomBodyId::new(
                "asteroid", "433-Eros",
            ))));
    }
}
