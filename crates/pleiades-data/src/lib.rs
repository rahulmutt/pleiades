//! Packaged compressed ephemeris backend for the common 1500-2500 range.
//!
//! This crate now ships a small stage-5 prototype artifact backed by the
//! `pleiades-compression` codec. The bundled data currently covers Sun and Moon
//! positions via quantized linear segments fitted to checked-in reference
//! epochs, and the backend falls back to other providers when callers request
//! bodies outside that packaged slice.

#![forbid(unsafe_code)]

use std::sync::OnceLock;

use pleiades_backend::{
    AccuracyClass, Apparentness, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
    BackendProvenance, CelestialBody, CoordinateFrame, EclipticCoordinates, EphemerisBackend,
    EphemerisError, EphemerisErrorKind, EphemerisRequest, EphemerisResult, Instant, JulianDay,
    QualityAnnotation, TimeRange, TimeScale, ZodiacMode,
};
use pleiades_compression::{
    ArtifactHeader, BodyArtifact, ChannelKind, CompressedArtifact, PolynomialChannel, Segment,
};

const PACKAGE_NAME: &str = "pleiades-data";
const ARTIFACT_LABEL: &str = "stage-5 packaged-data prototype";
const ARTIFACT_SOURCE: &str = "Quantized linear segments fitted to JPL Horizons reference epochs (1800, 2000, 2500 CE) for the Sun and Moon.";
const SUN_1800_JD: f64 = 2_378_499.0;
const SUN_2000_JD: f64 = 2_451_545.0;
const SUN_2500_JD: f64 = 2_634_167.0;
const SUN_1800_LONGITUDE: f64 = 285.778_468_393_866_8;
const SUN_1800_LATITUDE: f64 = -0.024_401_746_407_338_283;
const SUN_1800_DISTANCE: f64 = 0.983_232_199_627_031_7;
const SUN_2000_LONGITUDE: f64 = 280.377_822_768_143_5;
const SUN_2000_LATITUDE: f64 = 0.000_238_038_039_435_946_87;
const SUN_2000_DISTANCE: f64 = 0.983_327_678_869_055_9;
const SUN_2500_LONGITUDE: f64 = 274.030_996_829_848_1;
const SUN_2500_LATITUDE: f64 = 0.064_094_000_674_339_67;
const SUN_2500_DISTANCE: f64 = 0.983_818_643_245_76;
const MOON_1800_LONGITUDE: f64 = 21.707_800_882_436_008;
const MOON_1800_LATITUDE: f64 = -1.326_602_070_741_392_1;
const MOON_1800_DISTANCE: f64 = 0.002_690_214_220_051_901_4;
const MOON_2000_LONGITUDE: f64 = 223.318_924_130_496_44;
const MOON_2000_LATITUDE: f64 = 5.170_871_614_272_694;
const MOON_2000_DISTANCE: f64 = 0.002_690_202_995_704_765;
const MOON_2500_LONGITUDE: f64 = 272.026_168_351_137_6;
const MOON_2500_LATITUDE: f64 = 4.909_877_340_766_432;
const MOON_2500_DISTANCE: f64 = 0.002_387_075_486_659_366_2;

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
    packaged_artifact().lookup_ecliptic(body, instant)
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
                    "Checked-in JPL Horizons reference epochs (Sun and Moon)".to_string(),
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

        if req.frame != CoordinateFrame::Ecliptic {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedCoordinateFrame,
                "packaged data currently serves only ecliptic coordinates",
            ));
        }

        if req.apparent != Apparentness::Mean {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "packaged data stores mean-state lookup values only",
            ));
        }

        if req.zodiac_mode != ZodiacMode::Tropical {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "sidereal conversion is handled above the packaged-data backend",
            ));
        }

        if req.observer.is_some() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidObserver,
                "packaged data is geocentric only",
            ));
        }

        let ecliptic = packaged_artifact()
            .lookup_ecliptic(&req.body, req.instant)
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
        vec![
            BodyArtifact::new(
                CelestialBody::Sun,
                vec![
                    segment(SegmentSpec {
                        start_jd: SUN_1800_JD,
                        end_jd: SUN_2000_JD,
                        start_longitude: SUN_1800_LONGITUDE,
                        end_longitude: SUN_2000_LONGITUDE,
                        start_latitude: SUN_1800_LATITUDE,
                        end_latitude: SUN_2000_LATITUDE,
                        start_distance: SUN_1800_DISTANCE,
                        end_distance: SUN_2000_DISTANCE,
                    }),
                    segment(SegmentSpec {
                        start_jd: SUN_2000_JD,
                        end_jd: SUN_2500_JD,
                        start_longitude: SUN_2000_LONGITUDE,
                        end_longitude: SUN_2500_LONGITUDE,
                        start_latitude: SUN_2000_LATITUDE,
                        end_latitude: SUN_2500_LATITUDE,
                        start_distance: SUN_2000_DISTANCE,
                        end_distance: SUN_2500_DISTANCE,
                    }),
                ],
            ),
            BodyArtifact::new(
                CelestialBody::Moon,
                vec![
                    segment(SegmentSpec {
                        start_jd: SUN_1800_JD,
                        end_jd: SUN_2000_JD,
                        start_longitude: MOON_1800_LONGITUDE,
                        end_longitude: MOON_2000_LONGITUDE,
                        start_latitude: MOON_1800_LATITUDE,
                        end_latitude: MOON_2000_LATITUDE,
                        start_distance: MOON_1800_DISTANCE,
                        end_distance: MOON_2000_DISTANCE,
                    }),
                    segment(SegmentSpec {
                        start_jd: SUN_2000_JD,
                        end_jd: SUN_2500_JD,
                        start_longitude: MOON_2000_LONGITUDE,
                        end_longitude: MOON_2500_LONGITUDE,
                        start_latitude: MOON_2000_LATITUDE,
                        end_latitude: MOON_2500_LATITUDE,
                        start_distance: MOON_2000_DISTANCE,
                        end_distance: MOON_2500_DISTANCE,
                    }),
                ],
            ),
        ],
    );
    artifact.checksum = artifact
        .checksum()
        .expect("packaged artifact checksum should be reproducible");
    artifact
}

#[derive(Clone, Copy)]
struct SegmentSpec {
    start_jd: f64,
    end_jd: f64,
    start_longitude: f64,
    end_longitude: f64,
    start_latitude: f64,
    end_latitude: f64,
    start_distance: f64,
    end_distance: f64,
}

fn segment(spec: SegmentSpec) -> Segment {
    Segment::new(
        Instant::new(JulianDay::from_days(spec.start_jd), TimeScale::Tt),
        Instant::new(JulianDay::from_days(spec.end_jd), TimeScale::Tt),
        vec![
            PolynomialChannel::linear(
                ChannelKind::Longitude,
                9,
                spec.start_longitude,
                spec.end_longitude,
            ),
            PolynomialChannel::linear(
                ChannelKind::Latitude,
                9,
                spec.start_latitude,
                spec.end_latitude,
            ),
            PolynomialChannel::linear(
                ChannelKind::DistanceAu,
                12,
                spec.start_distance,
                spec.end_distance,
            ),
        ],
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
        assert_eq!(decoded.bodies.len(), 2);
        assert_eq!(decoded.checksum, artifact.checksum);
    }

    #[test]
    fn lookup_uses_packaged_segments() {
        let ecliptic = packaged_lookup(
            &CelestialBody::Sun,
            Instant::new(JulianDay::from_days(SUN_2000_JD), TimeScale::Tt),
        )
        .expect("packaged lookup should succeed");

        assert!((ecliptic.longitude.degrees() - SUN_2000_LONGITUDE).abs() < 1e-8);
        assert!((ecliptic.latitude.degrees() - SUN_2000_LATITUDE).abs() < 1e-8);
        assert!((ecliptic.distance_au.unwrap() - SUN_2000_DISTANCE).abs() < 1e-12);
    }

    #[test]
    fn backend_metadata_exposes_packaged_scope() {
        let metadata = packaged_backend().metadata();
        assert_eq!(metadata.id.as_str(), PACKAGE_NAME);
        assert_eq!(metadata.family, BackendFamily::CompressedData);
        assert!(metadata.body_coverage.contains(&CelestialBody::Sun));
        assert!(metadata.body_coverage.contains(&CelestialBody::Moon));
    }
}
