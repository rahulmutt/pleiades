//! Packaged compressed ephemeris backend for the common 1500-2500 range.
//!
//! This crate now ships a small stage-5 prototype artifact backed by the
//! `pleiades-compression` codec. The bundled data is loaded from a checked-in
//! deterministic binary fixture that covers the comparison-body planetary set
//! plus the source-backed custom asteroid `asteroid:433-Eros`, and the backend
//! falls back to other providers when callers request bodies outside that
//! packaged slice. The fixture is still regenerated from the checked-in JPL
//! reference snapshot in tests so the packaged data stays reproducible. A maintainer-facing
//! regeneration helper can rebuild the checked-in fixture from the bundled JPL reference
//! snapshot without introducing any native tooling.

#![forbid(unsafe_code)]

use std::cmp::Ordering;
use std::sync::OnceLock;

use pleiades_backend::{
    validate_observer_policy, validate_request_policy, validate_zodiac_policy, AccuracyClass,
    Apparentness, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
    BackendProvenance, CelestialBody, CoordinateFrame, CustomBodyId, EclipticCoordinates,
    EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest, EphemerisResult,
    Instant, QualityAnnotation, TimeRange, TimeScale, ZodiacMode,
};
use pleiades_compression::CompressedArtifact;
use pleiades_compression::{ArtifactHeader, BodyArtifact, ChannelKind, PolynomialChannel, Segment};
use pleiades_jpl::{reference_snapshot, reference_snapshot_summary_for_report, SnapshotEntry};

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

/// Returns the packaged body set as a human-readable provenance summary.
pub fn packaged_body_coverage_summary() -> String {
    let bodies = packaged_bodies();
    let body_list = bodies
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "Packaged body set: {} bundled bodies ({body_list})",
        bodies.len()
    )
}

/// Returns the packaged-artifact regeneration provenance summary.
pub fn packaged_artifact_regeneration_summary() -> String {
    format!(
        "Packaged artifact regeneration source: label={}; source={}; {}",
        ARTIFACT_LABEL,
        ARTIFACT_SOURCE,
        reference_snapshot_summary_for_report(),
    )
}

fn join_labels<T>(values: &[T], label: impl Fn(&T) -> &'static str) -> String {
    values.iter().map(label).collect::<Vec<_>>().join(", ")
}

/// Structured policy for how packaged-data lookup epochs are handled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackagedLookupEpochPolicy {
    /// TDB-tagged lookups are re-tagged onto the TT grid without relativistic correction.
    RetagToTtGridWithoutRelativisticCorrection,
}

impl PackagedLookupEpochPolicy {
    /// Returns the compact label used in release-facing summaries.
    pub const fn label(self) -> &'static str {
        match self {
            Self::RetagToTtGridWithoutRelativisticCorrection => {
                "TT-grid retag without relativistic correction"
            }
        }
    }

    /// Returns the explanatory note used in release-facing summaries.
    pub const fn note(self) -> &'static str {
        match self {
            Self::RetagToTtGridWithoutRelativisticCorrection => {
                "TDB lookup epochs are re-tagged onto the TT grid without applying a relativistic correction"
            }
        }
    }
}

/// Structured request-policy summary for the packaged-data backend.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PackagedRequestPolicySummary {
    /// Whether the backend is geocentric only.
    pub geocentric_only: bool,
    /// Coordinate frames supported by the packaged backend.
    pub supported_frames: &'static [CoordinateFrame],
    /// Time scales supported by the packaged backend.
    pub supported_time_scales: &'static [TimeScale],
    /// Zodiac modes supported by the packaged backend.
    pub supported_zodiac_modes: &'static [ZodiacMode],
    /// Apparentness modes supported by the packaged backend.
    pub supported_apparentness: &'static [Apparentness],
    /// Whether the packaged backend accepts topocentric observer requests.
    pub supports_topocentric_observer: bool,
    /// Policy describing how TDB-tagged lookups are handled.
    pub lookup_epoch_policy: PackagedLookupEpochPolicy,
}

impl PackagedRequestPolicySummary {
    /// Renders the packaged request policy into a release-facing summary line.
    pub fn summary_line(&self) -> String {
        format!(
            "Packaged request policy: {}frames={}; time scales={}; zodiac modes={}; apparentness={}; topocentric observer={}; lookup epoch policy={}; {}",
            if self.geocentric_only {
                "geocentric-only; "
            } else {
                ""
            },
            join_labels(self.supported_frames, |value| match value {
                CoordinateFrame::Ecliptic => "Ecliptic",
                CoordinateFrame::Equatorial => "Equatorial",
                _ => "<unknown>",
            }),
            join_labels(self.supported_time_scales, |value| match value {
                TimeScale::Tt => "TT",
                TimeScale::Tdb => "TDB",
                _ => "<unknown>",
            }),
            join_labels(self.supported_zodiac_modes, |value| match value {
                ZodiacMode::Tropical => "Tropical",
                _ => "<unknown>",
            }),
            join_labels(self.supported_apparentness, |value| match value {
                Apparentness::Mean => "Mean",
                Apparentness::Apparent => "Apparent",
                _ => "<unknown>",
            }),
            self.supports_topocentric_observer,
            self.lookup_epoch_policy.label(),
            self.lookup_epoch_policy.note(),
        )
    }
}

const PACKAGED_REQUEST_POLICY_SUMMARY: PackagedRequestPolicySummary =
    PackagedRequestPolicySummary {
        geocentric_only: true,
        supported_frames: &[CoordinateFrame::Ecliptic],
        supported_time_scales: &[TimeScale::Tt, TimeScale::Tdb],
        supported_zodiac_modes: &[ZodiacMode::Tropical],
        supported_apparentness: &[Apparentness::Mean],
        supports_topocentric_observer: false,
        lookup_epoch_policy: PackagedLookupEpochPolicy::RetagToTtGridWithoutRelativisticCorrection,
    };

/// Returns the current packaged-data request-policy summary record.
pub fn packaged_request_policy_summary_details() -> PackagedRequestPolicySummary {
    PACKAGED_REQUEST_POLICY_SUMMARY
}

/// Returns the current packaged-data request policy summary.
pub fn packaged_request_policy_summary() -> &'static str {
    "Packaged request policy: geocentric-only; frames=Ecliptic; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false; lookup epoch policy=TT-grid retag without relativistic correction; TDB lookup epochs are re-tagged onto the TT grid without applying a relativistic correction"
}

const AU_IN_KM: f64 = 149_597_870.7;

/// Returns the canonical package name for this crate.
pub const fn package_name() -> &'static str {
    PACKAGE_NAME
}

const PACKAGED_ARTIFACT_FIXTURE: &[u8] = include_bytes!("../tests/fixtures/packaged-artifact.bin");

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
                    packaged_body_coverage_summary(),
                    packaged_request_policy_summary_details().summary_line(),
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

        validate_zodiac_policy(req, "packaged data", &[ZodiacMode::Tropical])?;

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
    CompressedArtifact::decode(PACKAGED_ARTIFACT_FIXTURE)
        .expect("packaged artifact fixture should decode")
}

/// Rebuilds the packaged artifact from the checked-in JPL reference snapshot.
///
/// This helper is deterministic and pure Rust so maintainers can regenerate the
/// checked-in fixture without relying on platform-specific tooling.
pub fn regenerate_packaged_artifact() -> CompressedArtifact {
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
        | pleiades_compression::CompressionErrorKind::UnsupportedEndianPolicy
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
        assert_eq!(encoded, PACKAGED_ARTIFACT_FIXTURE);
        let decoded =
            CompressedArtifact::decode(&encoded).expect("packaged artifact should decode");
        assert_eq!(decoded.header.generation_label, ARTIFACT_LABEL);
        assert_eq!(decoded.bodies.len(), packaged_bodies().len());
        assert_eq!(decoded.checksum, artifact.checksum);
    }

    #[test]
    fn packaged_artifact_decode_rejects_checksum_corruption() {
        let mut encoded = PACKAGED_ARTIFACT_FIXTURE.to_vec();
        let last_index = encoded.len() - 1;
        encoded[last_index] ^= 0x01;

        let error = CompressedArtifact::decode(&encoded)
            .expect_err("tampered packaged artifact should fail to decode");

        assert_eq!(
            error.kind,
            pleiades_compression::CompressionErrorKind::ChecksumMismatch
        );
    }

    #[test]
    fn packaged_artifact_fixture_matches_reference_snapshot_generation() {
        let generated = regenerate_packaged_artifact();
        let encoded = generated
            .encode()
            .expect("generated packaged artifact should encode");
        assert_eq!(encoded, PACKAGED_ARTIFACT_FIXTURE);
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
    fn observer_requests_are_rejected_explicitly() {
        let backend = packaged_backend();
        let request = EphemerisRequest {
            body: CelestialBody::Sun,
            instant: Instant::new(
                pleiades_backend::JulianDay::from_days(2_451_545.0),
                TimeScale::Tdb,
            ),
            observer: Some(pleiades_backend::ObserverLocation::new(
                pleiades_backend::Latitude::from_degrees(51.5),
                pleiades_backend::Longitude::from_degrees(0.0),
                None,
            )),
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: pleiades_backend::Apparentness::Mean,
        };

        let error = backend
            .position(&request)
            .expect_err("packaged data should reject topocentric requests");

        assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
    }

    #[test]
    fn apparent_requests_are_rejected_explicitly() {
        let backend = packaged_backend();
        let request = EphemerisRequest {
            body: CelestialBody::Sun,
            instant: Instant::new(
                pleiades_backend::JulianDay::from_days(2_451_545.0),
                TimeScale::Tdb,
            ),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: pleiades_backend::Apparentness::Apparent,
        };

        let error = backend
            .position(&request)
            .expect_err("packaged data should reject apparent-place requests");

        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
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
        assert!(metadata.provenance.data_sources[0].contains("11 bundled bodies"));
        assert!(metadata.provenance.data_sources[0].contains("asteroid:433-Eros"));
        assert_eq!(
            packaged_body_coverage_summary(),
            metadata.provenance.data_sources[0]
        );
        let request_policy = packaged_request_policy_summary_details();
        assert!(request_policy.geocentric_only);
        assert_eq!(
            request_policy.supported_frames,
            &[CoordinateFrame::Ecliptic]
        );
        assert_eq!(
            request_policy.supported_time_scales,
            &[TimeScale::Tt, TimeScale::Tdb]
        );
        assert_eq!(
            request_policy.supported_zodiac_modes,
            &[ZodiacMode::Tropical]
        );
        assert_eq!(request_policy.supported_apparentness, &[Apparentness::Mean]);
        assert!(!request_policy.supports_topocentric_observer);
        assert_eq!(
            request_policy.lookup_epoch_policy,
            PackagedLookupEpochPolicy::RetagToTtGridWithoutRelativisticCorrection
        );
        assert_eq!(
            request_policy.summary_line(),
            packaged_request_policy_summary().to_string()
        );
        assert_eq!(
            metadata.provenance.data_sources[1],
            request_policy.summary_line()
        );
    }

    #[test]
    fn packaged_artifact_regeneration_summary_includes_reference_snapshot_coverage() {
        let summary = packaged_artifact_regeneration_summary();
        assert!(summary.contains(
            "Packaged artifact regeneration source: label=stage-5 packaged-data prototype"
        ));
        assert!(summary.contains("Reference snapshot coverage:"));
        assert!(summary.contains("rows across"));
        assert!(summary.contains("asteroid rows"));
    }

    #[test]
    fn packaged_tdb_batch_requests_match_tt_grid_lookups() {
        let backend = packaged_backend();
        let tt_request = EphemerisRequest {
            body: CelestialBody::Sun,
            instant: Instant::new(
                pleiades_backend::JulianDay::from_days(2_451_545.0),
                TimeScale::Tt,
            ),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: pleiades_backend::Apparentness::Mean,
        };
        let tdb_request = EphemerisRequest {
            instant: Instant::new(tt_request.instant.julian_day, TimeScale::Tdb),
            ..tt_request.clone()
        };

        let tt_results = backend
            .positions(&[tt_request])
            .expect("TT requests should succeed through the batch path");
        let tdb_results = backend
            .positions(&[tdb_request])
            .expect("TDB requests should succeed through the batch path");

        assert_eq!(tt_results.len(), 1);
        assert_eq!(tdb_results.len(), 1);

        let tt_result = &tt_results[0];
        let tdb_result = &tdb_results[0];

        assert_eq!(tt_result.instant.scale, TimeScale::Tt);
        assert_eq!(tdb_result.instant.scale, TimeScale::Tdb);
        assert_eq!(tt_result.quality, QualityAnnotation::Interpolated);
        assert_eq!(tdb_result.quality, QualityAnnotation::Interpolated);
        assert_eq!(tt_result.ecliptic, tdb_result.ecliptic);
        assert_eq!(tt_result.backend_id, tdb_result.backend_id);
        assert_eq!(tt_result.body, tdb_result.body);
        assert_eq!(tt_result.apparent, tdb_result.apparent);
    }

    #[test]
    fn packaged_body_coverage_summary_matches_the_packaged_body_set() {
        let summary = packaged_body_coverage_summary();
        assert!(summary.contains("11 bundled bodies"));
        assert!(summary.contains("Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, asteroid:433-Eros"));
    }
}
