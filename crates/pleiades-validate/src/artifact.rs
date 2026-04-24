use core::fmt;

use crate::{
    compare_backends, default_candidate_backend, ComparisonReport, ComparisonSample,
    ValidationCorpus,
};
use pleiades_compression::{CompressedArtifact, CompressionError};
use pleiades_core::{
    Angle, Apparentness, CelestialBody, CoordinateFrame, EclipticCoordinates, EphemerisRequest,
    Instant, JulianDay, ZodiacMode,
};
use pleiades_data::{packaged_artifact, packaged_backend};

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
    /// Comparison report against the algorithmic baseline.
    pub model_comparison: ComparisonReport,
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
    ) -> Result<Self, ArtifactInspectionError> {
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

        let comparison_corpus = artifact_model_comparison_corpus(&decoded);
        let model_comparison = compare_backends(
            &default_candidate_backend(),
            &packaged_backend(),
            &comparison_corpus,
        )?;

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
            model_comparison,
            bodies,
        })
    }
}

/// Renders the bundled artifact validation report.
pub fn render_artifact_report() -> Result<String, ArtifactInspectionError> {
    let artifact = packaged_artifact();
    let encoded = artifact.encode()?;
    let report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())?;
    Ok(report.to_string())
}

/// Renders a compact summary of the bundled artifact validation report.
pub fn render_artifact_summary() -> Result<String, ArtifactInspectionError> {
    let artifact = packaged_artifact();
    let encoded = artifact.encode()?;
    let report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())?;
    Ok(render_artifact_summary_text(&report))
}

pub(crate) fn packaged_artifact_corpus() -> ValidationCorpus {
    artifact_comparison_corpus(packaged_artifact())
}

fn artifact_model_comparison_corpus(artifact: &CompressedArtifact) -> ValidationCorpus {
    artifact_comparison_corpus_filtered(artifact, |body| !matches!(body, CelestialBody::Custom(_)))
}

fn artifact_comparison_corpus(artifact: &CompressedArtifact) -> ValidationCorpus {
    artifact_comparison_corpus_filtered(artifact, |_| true)
}

fn artifact_comparison_corpus_filtered<F>(
    artifact: &CompressedArtifact,
    include_body: F,
) -> ValidationCorpus
where
    F: Fn(&CelestialBody) -> bool,
{
    let mut requests = Vec::new();

    for body in &artifact.bodies {
        if !include_body(&body.body) {
            continue;
        }

        for segment in &body.segments {
            let midpoint = midpoint(segment.start, segment.end);
            for instant in [segment.start, midpoint, segment.end] {
                requests.push(EphemerisRequest {
                    body: body.body.clone(),
                    instant,
                    observer: None,
                    frame: CoordinateFrame::Ecliptic,
                    zodiac_mode: ZodiacMode::Tropical,
                    apparent: Apparentness::Mean,
                });
            }
        }
    }

    ValidationCorpus {
        name: "Packaged artifact error envelope".to_string(),
        description: "Comparison corpus built from packaged artifact coverage at segment endpoints and midpoints so the bundled data can be measured against the algorithmic baseline.",
        apparentness: Apparentness::Mean,
        requests,
    }
}

fn inspect_body(
    artifact: &CompressedArtifact,
    body: &pleiades_compression::BodyArtifact,
) -> Result<ArtifactBodyInspection, ArtifactInspectionError> {
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

fn render_artifact_summary_text(report: &ArtifactInspectionReport) -> String {
    let mut text = String::new();

    text.push_str("Artifact summary\n");
    text.push_str("  label: ");
    text.push_str(&report.generation_label);
    text.push('\n');
    text.push_str("  source: ");
    text.push_str(&report.source);
    text.push('\n');
    text.push_str("  version: ");
    text.push_str(&report.version.to_string());
    text.push('\n');
    text.push_str(&format!("  checksum: 0x{:016x}\n", report.checksum));
    text.push_str("  encoded bytes: ");
    text.push_str(&report.encoded_bytes.to_string());
    text.push('\n');
    text.push_str("  coverage: ");
    text.push_str(&report.earliest.julian_day.to_string());
    text.push_str(" → ");
    text.push_str(&report.latest.julian_day.to_string());
    text.push('\n');
    text.push_str("  bodies: ");
    text.push_str(&report.body_count.to_string());
    text.push_str(" total\n");
    text.push_str("  segments: ");
    text.push_str(&report.segment_count.to_string());
    text.push_str(" total\n");
    text.push_str("  roundtrip decode: ");
    text.push_str(yes_no(report.roundtrip_ok));
    text.push('\n');
    text.push_str("  checksum verified: ");
    text.push_str(yes_no(report.checksum_ok));
    text.push('\n');
    if report
        .bodies
        .iter()
        .any(|body| matches!(body.body, CelestialBody::Custom(_)))
    {
        text.push_str(
            "  note: custom bodies are included in decode and boundary checks, but omitted from the algorithmic comparison corpus.\n",
        );
    }
    text.push('\n');
    text.push_str("Model error envelope\n");
    text.push_str("  baseline backend: ");
    text.push_str(&report.model_comparison.reference_backend.id.to_string());
    text.push('\n');
    text.push_str("  candidate backend: ");
    text.push_str(&report.model_comparison.candidate_backend.id.to_string());
    text.push('\n');
    text.push_str("  corpus: ");
    text.push_str(&report.model_comparison.corpus_name);
    text.push('\n');
    text.push_str("  samples: ");
    text.push_str(&report.model_comparison.summary.sample_count.to_string());
    text.push('\n');
    text.push_str(&format!(
        "  max longitude delta: {:.12}°\n",
        report.model_comparison.summary.max_longitude_delta_deg
    ));
    text.push_str(&format!(
        "  max latitude delta: {:.12}°\n",
        report.model_comparison.summary.max_latitude_delta_deg
    ));
    if let Some(value) = report.model_comparison.summary.max_distance_delta_au {
        text.push_str(&format!("  max distance delta: {:.12} AU\n", value));
    }
    text.push_str("\nRelease summary: release-summary\n");
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str(
        "See validate-artifact for the full body-class envelopes and regression details.\nSee release-summary for the compact one-screen release overview.\n",
    );

    text
}

fn midpoint(start: Instant, end: Instant) -> Instant {
    let start_days = start.julian_day.days();
    let end_days = end.julian_day.days();
    Instant::new(
        JulianDay::from_days((start_days + end_days) / 2.0),
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BodyClass {
    Luminary,
    MajorPlanet,
    LunarPoint,
    Asteroid,
    Custom,
}

impl BodyClass {
    const ALL: [Self; 5] = [
        Self::Luminary,
        Self::MajorPlanet,
        Self::LunarPoint,
        Self::Asteroid,
        Self::Custom,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Luminary => "Luminaries",
            Self::MajorPlanet => "Major planets",
            Self::LunarPoint => "Lunar points",
            Self::Asteroid => "Asteroids",
            Self::Custom => "Custom bodies",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Luminary => 0,
            Self::MajorPlanet => 1,
            Self::LunarPoint => 2,
            Self::Asteroid => 3,
            Self::Custom => 4,
        }
    }
}

fn body_class(body: &CelestialBody) -> BodyClass {
    match body {
        CelestialBody::Sun | CelestialBody::Moon => BodyClass::Luminary,
        CelestialBody::Mercury
        | CelestialBody::Venus
        | CelestialBody::Mars
        | CelestialBody::Jupiter
        | CelestialBody::Saturn
        | CelestialBody::Uranus
        | CelestialBody::Neptune
        | CelestialBody::Pluto => BodyClass::MajorPlanet,
        CelestialBody::MeanNode
        | CelestialBody::TrueNode
        | CelestialBody::MeanApogee
        | CelestialBody::TrueApogee
        | CelestialBody::MeanPerigee
        | CelestialBody::TruePerigee => BodyClass::LunarPoint,
        CelestialBody::Ceres
        | CelestialBody::Pallas
        | CelestialBody::Juno
        | CelestialBody::Vesta => BodyClass::Asteroid,
        CelestialBody::Custom(_) => BodyClass::Custom,
        _ => BodyClass::Custom,
    }
}

#[derive(Clone, Debug)]
struct BodyClassSummary {
    class: BodyClass,
    sample_count: usize,
    max_longitude_delta_deg: f64,
    sum_longitude_delta_deg: f64,
    max_latitude_delta_deg: f64,
    sum_latitude_delta_deg: f64,
    max_distance_delta_au: Option<f64>,
    sum_distance_delta_au: f64,
    distance_count: usize,
}

impl BodyClassSummary {
    const fn new(class: BodyClass) -> Self {
        Self {
            class,
            sample_count: 0,
            max_longitude_delta_deg: 0.0,
            sum_longitude_delta_deg: 0.0,
            max_latitude_delta_deg: 0.0,
            sum_latitude_delta_deg: 0.0,
            max_distance_delta_au: None,
            sum_distance_delta_au: 0.0,
            distance_count: 0,
        }
    }

    fn update(&mut self, sample: &ComparisonSample) {
        self.sample_count += 1;
        self.max_longitude_delta_deg = self.max_longitude_delta_deg.max(sample.longitude_delta_deg);
        self.sum_longitude_delta_deg += sample.longitude_delta_deg;
        self.max_latitude_delta_deg = self.max_latitude_delta_deg.max(sample.latitude_delta_deg);
        self.sum_latitude_delta_deg += sample.latitude_delta_deg;

        if let Some(distance_delta_au) = sample.distance_delta_au {
            self.max_distance_delta_au = Some(
                self.max_distance_delta_au
                    .map_or(distance_delta_au, |current| current.max(distance_delta_au)),
            );
            self.sum_distance_delta_au += distance_delta_au;
            self.distance_count += 1;
        }
    }

    fn mean_longitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            self.sum_longitude_delta_deg / self.sample_count as f64
        }
    }

    fn mean_latitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            self.sum_latitude_delta_deg / self.sample_count as f64
        }
    }

    fn mean_distance_delta_au(&self) -> Option<f64> {
        if self.distance_count == 0 {
            None
        } else {
            Some(self.sum_distance_delta_au / self.distance_count as f64)
        }
    }

    fn render(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.sample_count == 0 {
            return Ok(());
        }

        writeln!(f, "  {}", self.class.label())?;
        writeln!(f, "    samples: {}", self.sample_count)?;
        writeln!(
            f,
            "    max longitude delta: {:.12}°",
            self.max_longitude_delta_deg
        )?;
        writeln!(
            f,
            "    mean longitude delta: {:.12}°",
            self.mean_longitude_delta_deg()
        )?;
        writeln!(
            f,
            "    max latitude delta: {:.12}°",
            self.max_latitude_delta_deg
        )?;
        writeln!(
            f,
            "    mean latitude delta: {:.12}°",
            self.mean_latitude_delta_deg()
        )?;
        if let Some(value) = self.max_distance_delta_au {
            writeln!(f, "    max distance delta: {:.12} AU", value)?;
        }
        if let Some(value) = self.mean_distance_delta_au() {
            writeln!(f, "    mean distance delta: {:.12} AU", value)?;
        }

        Ok(())
    }
}

fn write_body_class_envelopes(
    f: &mut fmt::Formatter<'_>,
    samples: &[ComparisonSample],
) -> fmt::Result {
    writeln!(f, "Body-class error envelopes")?;

    let mut summaries = BodyClass::ALL.map(BodyClassSummary::new);
    for sample in samples {
        summaries[body_class(&sample.body).index()].update(sample);
    }

    let mut has_entries = false;
    for summary in &summaries {
        if summary.sample_count > 0 {
            has_entries = true;
            summary.render(f)?;
        }
    }

    if !has_entries {
        writeln!(f, "  none")?;
    }

    Ok(())
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
                body.body,
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

        writeln!(f)?;
        if self
            .bodies
            .iter()
            .any(|body| matches!(body.body, CelestialBody::Custom(_)))
        {
            writeln!(f, "Note: custom bodies are included in decode and boundary checks, but omitted from the algorithmic comparison corpus.")?;
            writeln!(f)?;
        }
        writeln!(f, "Model error envelope")?;
        writeln!(
            f,
            "  baseline backend: {}",
            self.model_comparison.reference_backend.id
        )?;
        writeln!(
            f,
            "  candidate backend: {}",
            self.model_comparison.candidate_backend.id
        )?;
        writeln!(f, "  corpus: {}", self.model_comparison.corpus_name)?;
        writeln!(
            f,
            "  samples: {}",
            self.model_comparison.summary.sample_count
        )?;
        writeln!(
            f,
            "  max longitude delta: {:.12}°",
            self.model_comparison.summary.max_longitude_delta_deg
        )?;
        writeln!(
            f,
            "  mean longitude delta: {:.12}°",
            self.model_comparison.summary.mean_longitude_delta_deg
        )?;
        writeln!(
            f,
            "  max latitude delta: {:.12}°",
            self.model_comparison.summary.max_latitude_delta_deg
        )?;
        writeln!(
            f,
            "  mean latitude delta: {:.12}°",
            self.model_comparison.summary.mean_latitude_delta_deg
        )?;
        if let Some(value) = self.model_comparison.summary.max_distance_delta_au {
            writeln!(f, "  max distance delta: {:.12} AU", value)?;
        }
        if let Some(value) = self.model_comparison.summary.mean_distance_delta_au {
            writeln!(f, "  mean distance delta: {:.12} AU", value)?;
        }

        writeln!(f)?;
        write_body_class_envelopes(f, &self.model_comparison.samples)?;
        writeln!(f)?;

        let notable_regressions = self.model_comparison.notable_regressions();
        writeln!(f, "  notable regressions")?;
        if notable_regressions.is_empty() {
            writeln!(f, "    none")?;
        } else {
            for finding in notable_regressions {
                writeln!(
                    f,
                    "    {}: Δlon={:.12}°, Δlat={:.12}°, Δdist={}, {}",
                    finding.body,
                    finding.longitude_delta_deg,
                    finding.latitude_delta_deg,
                    finding
                        .distance_delta_au
                        .map(|value| format!("{value:.12} AU"))
                        .unwrap_or_else(|| "n/a".to_string()),
                    finding.note
                )?;
            }
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

/// Errors produced while building the artifact inspection report.
#[derive(Debug)]
pub enum ArtifactInspectionError {
    /// Compression or codec failure while decoding the packaged artifact.
    Compression(CompressionError),
    /// Validation failure while comparing the packaged artifact to the baseline backend.
    Validation(pleiades_core::EphemerisError),
}

impl core::fmt::Display for ArtifactInspectionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Compression(error) => write!(f, "{error}"),
            Self::Validation(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ArtifactInspectionError {}

impl From<CompressionError> for ArtifactInspectionError {
    fn from(error: CompressionError) -> Self {
        Self::Compression(error)
    }
}

impl From<pleiades_core::EphemerisError> for ArtifactInspectionError {
    fn from(error: pleiades_core::EphemerisError) -> Self {
        Self::Validation(error)
    }
}
