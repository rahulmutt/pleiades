use core::fmt;

use pleiades_compression::{CompressedArtifact, CompressionError};
use pleiades_core::{Angle, CelestialBody, EclipticCoordinates, Instant};
use pleiades_data::packaged_artifact;

/// A report describing the bundled compressed artifact and its boundary checks.
#[derive(Clone, Debug)]
pub struct ArtifactInspectionReport {
    /// Human-readable label from the artifact header.
    pub generation_label: String,
    /// Source/provenance summary from the artifact header.
    pub source: String,
    /// Artifact format version.
    pub version: u16,
    /// The encoded artifact checksum.
    pub checksum: u64,
    /// Size of the encoded artifact in bytes.
    pub encoded_bytes: usize,
    /// Whether the codec roundtrip preserved the artifact structure.
    pub roundtrip_ok: bool,
    /// Whether the decoded checksum matches the encoded checksum.
    pub checksum_ok: bool,
    /// Number of bodies in the artifact.
    pub body_count: usize,
    /// Number of segments across all bodies.
    pub segment_count: usize,
    /// Earliest covered instant.
    pub earliest: Instant,
    /// Latest covered instant.
    pub latest: Instant,
    /// Per-body validation summaries.
    pub bodies: Vec<ArtifactBodyInspection>,
}

/// Validation summary for a single body in the packaged artifact.
#[derive(Clone, Debug)]
pub struct ArtifactBodyInspection {
    /// Body identifier.
    pub body: CelestialBody,
    /// Number of segments for the body.
    pub segment_count: usize,
    /// Earliest segment start.
    pub earliest: Instant,
    /// Latest segment end.
    pub latest: Instant,
    /// Number of sample lookups exercised for this body.
    pub sample_count: usize,
    /// Number of shared segment boundaries checked for continuity.
    pub boundary_checks: usize,
    /// Maximum longitude delta observed at any checked boundary.
    pub max_boundary_longitude_delta_deg: f64,
    /// Maximum latitude delta observed at any checked boundary.
    pub max_boundary_latitude_delta_deg: f64,
    /// Maximum distance delta observed at any checked boundary.
    pub max_boundary_distance_delta_au: Option<f64>,
}

impl ArtifactInspectionReport {
    fn from_artifact(
        artifact: &CompressedArtifact,
        encoded_bytes: usize,
    ) -> Result<Self, CompressionError> {
        let decoded = CompressedArtifact::decode(&artifact.encode()?)?;
        let mut bodies = Vec::with_capacity(decoded.bodies.len());
        let mut segment_count = 0usize;
        let mut earliest: Option<Instant> = None;
        let mut latest: Option<Instant> = None;

        for body in &decoded.bodies {
            let inspection = inspect_body(&decoded, body)?;
            segment_count += inspection.segment_count;
            earliest = Some(match earliest {
                Some(current)
                    if current.julian_day.days() <= inspection.earliest.julian_day.days() =>
                {
                    current
                }
                Some(_) => inspection.earliest,
                None => inspection.earliest,
            });
            latest = Some(match latest {
                Some(current)
                    if current.julian_day.days() >= inspection.latest.julian_day.days() =>
                {
                    current
                }
                Some(_) => inspection.latest,
                None => inspection.latest,
            });
            bodies.push(inspection);
        }

        Ok(Self {
            generation_label: decoded.header.generation_label,
            source: decoded.header.source,
            version: decoded.header.version,
            checksum: decoded.checksum,
            encoded_bytes,
            roundtrip_ok: true,
            checksum_ok: decoded.checksum == artifact.checksum,
            body_count: decoded.bodies.len(),
            segment_count,
            earliest: earliest.unwrap_or_else(|| artifact_first_instant(artifact)),
            latest: latest.unwrap_or_else(|| artifact_first_instant(artifact)),
            bodies,
        })
    }
}

/// Renders the bundled artifact validation report.
pub fn render_artifact_report() -> Result<String, CompressionError> {
    let artifact = packaged_artifact();
    let encoded = artifact.encode()?;
    let report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())?;
    Ok(report.to_string())
}

fn inspect_body(
    artifact: &CompressedArtifact,
    body: &pleiades_compression::BodyArtifact,
) -> Result<ArtifactBodyInspection, CompressionError> {
    let mut sample_count = 0usize;
    let mut boundary_checks = 0usize;
    let mut max_boundary_longitude_delta_deg: f64 = 0.0;
    let mut max_boundary_latitude_delta_deg: f64 = 0.0;
    let mut max_boundary_distance_delta_au: Option<f64> = None;

    for segment in &body.segments {
        let midpoint = midpoint(segment.start, segment.end);
        for instant in [segment.start, midpoint, segment.end] {
            artifact.lookup_ecliptic(&body.body, instant)?;
            sample_count += 1;
        }
    }

    for pair in body.segments.windows(2) {
        let left = artifact.lookup_ecliptic(&body.body, pair[0].end)?;
        let right = artifact.lookup_ecliptic(&body.body, pair[1].start)?;
        let delta = boundary_delta(&left, &right);
        boundary_checks += 1;
        max_boundary_longitude_delta_deg =
            max_boundary_longitude_delta_deg.max(delta.longitude_delta_deg);
        max_boundary_latitude_delta_deg =
            max_boundary_latitude_delta_deg.max(delta.latitude_delta_deg);
        max_boundary_distance_delta_au =
            match (max_boundary_distance_delta_au, delta.distance_delta_au) {
                (Some(current), Some(next)) => Some(current.max(next)),
                (None, Some(next)) => Some(next),
                (current, None) => current,
            };
    }

    let earliest = body
        .segments
        .first()
        .map(|segment| segment.start)
        .unwrap_or_else(|| artifact_first_instant(artifact));
    let latest = body
        .segments
        .last()
        .map(|segment| segment.end)
        .unwrap_or_else(|| artifact_first_instant(artifact));

    Ok(ArtifactBodyInspection {
        body: body.body.clone(),
        segment_count: body.segments.len(),
        earliest,
        latest,
        sample_count,
        boundary_checks,
        max_boundary_longitude_delta_deg,
        max_boundary_latitude_delta_deg,
        max_boundary_distance_delta_au,
    })
}

fn midpoint(start: Instant, end: Instant) -> Instant {
    let start_days = start.julian_day.days();
    let end_days = end.julian_day.days();
    Instant::new(
        pleiades_core::JulianDay::from_days((start_days + end_days) / 2.0),
        start.scale,
    )
}

fn artifact_first_instant(artifact: &CompressedArtifact) -> Instant {
    artifact
        .bodies
        .iter()
        .flat_map(|body| body.segments.iter())
        .map(|segment| segment.start)
        .min_by(|left, right| {
            left.julian_day
                .days()
                .partial_cmp(&right.julian_day.days())
                .unwrap_or(core::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|| {
            Instant::new(
                pleiades_core::JulianDay::from_days(0.0),
                pleiades_core::TimeScale::Tt,
            )
        })
}

struct BoundaryDelta {
    longitude_delta_deg: f64,
    latitude_delta_deg: f64,
    distance_delta_au: Option<f64>,
}

fn boundary_delta(left: &EclipticCoordinates, right: &EclipticCoordinates) -> BoundaryDelta {
    let longitude_delta_deg =
        Angle::from_degrees(left.longitude.degrees() - right.longitude.degrees())
            .normalized_signed()
            .degrees()
            .abs();
    let latitude_delta_deg = (left.latitude.degrees() - right.latitude.degrees()).abs();
    let distance_delta_au = match (left.distance_au, right.distance_au) {
        (Some(left), Some(right)) => Some((left - right).abs()),
        _ => None,
    };

    BoundaryDelta {
        longitude_delta_deg,
        latitude_delta_deg,
        distance_delta_au,
    }
}

impl fmt::Display for ArtifactInspectionReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Artifact validation report")?;
        writeln!(f, "  label: {}", self.generation_label)?;
        writeln!(f, "  source: {}", self.source)?;
        writeln!(f, "  version: {}", self.version)?;
        writeln!(f, "  checksum: 0x{:016x}", self.checksum)?;
        writeln!(f, "  encoded bytes: {}", self.encoded_bytes)?;
        writeln!(f, "  roundtrip decode: {}", yes_no(self.roundtrip_ok))?;
        writeln!(f, "  checksum verified: {}", yes_no(self.checksum_ok))?;
        writeln!(f, "  bodies: {}", self.body_count)?;
        writeln!(f, "  segments: {}", self.segment_count)?;
        writeln!(
            f,
            "  coverage: {} → {}",
            self.earliest.julian_day, self.latest.julian_day
        )?;
        writeln!(f)?;
        writeln!(f, "Bodies")?;
        for body in &self.bodies {
            writeln!(
                f,
                "  {}: {} segments, {} → {}, {} samples, {} boundary checks, max boundary Δlon={:.12}°, Δlat={:.12}°, Δdist={}",
                body.body.built_in_name().unwrap_or("Custom"),
                body.segment_count,
                body.earliest.julian_day,
                body.latest.julian_day,
                body.sample_count,
                body.boundary_checks,
                body.max_boundary_longitude_delta_deg,
                body.max_boundary_latitude_delta_deg,
                body.max_boundary_distance_delta_au
                    .map(|value| format!("{value:.12} AU"))
                    .unwrap_or_else(|| "n/a".to_string())
            )?;
        }
        Ok(())
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "ok"
    } else {
        "failed"
    }
}
