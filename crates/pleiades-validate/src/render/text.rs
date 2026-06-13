//! Report and summary text rendering for the validation tool.

use std::collections::{BTreeMap, BTreeSet};

use super::summary::*;
use crate::*;

pub(crate) fn comparison_tolerance_policy_coverage(
    comparison: &ComparisonReport,
) -> Vec<ComparisonToleranceScopeCoverageSummary> {
    let entries = comparison_tolerance_policy_entries(&comparison.candidate_backend.family);
    let tolerance_summaries = comparison.tolerance_summaries();

    entries
        .into_iter()
        .map(|entry| {
            let mut bodies = Vec::new();
            let mut sample_count = 0;

            for summary in &tolerance_summaries {
                if comparison_tolerance_scope_for_body(&summary.body) == entry.scope {
                    bodies.push(summary.body.clone());
                    sample_count += summary.sample_count;
                }
            }

            ComparisonToleranceScopeCoverageSummary {
                entry,
                body_count: bodies.len(),
                bodies,
                sample_count,
            }
        })
        .collect()
}

pub(crate) fn write_tolerance_policy(
    f: &mut fmt::Formatter<'_>,
    comparison: &ComparisonReport,
) -> fmt::Result {
    let family_label = tolerance_backend_family_label(&comparison.candidate_backend.family);
    let summary = match validated_comparison_tolerance_policy_summary_for_report(comparison) {
        Ok(summary) => summary,
        Err(error) => {
            writeln!(f, "Tolerance policy catalog")?;
            writeln!(f, "  unavailable ({error})")?;
            return Ok(());
        }
    };
    let coordinate_frames = format_frames(&summary.coordinate_frames);
    writeln!(f, "Tolerance policy catalog")?;
    writeln!(f, "  candidate backend family: {}", family_label)?;
    writeln!(
        f,
        "  comparison evidence: {} bodies, {} samples",
        summary.comparison_body_count, summary.comparison_sample_count
    )?;
    writeln!(
        f,
        "  comparison window: {}",
        summary.comparison_window.summary_line()
    )?;
    writeln!(f, "  coordinate frames: {}", coordinate_frames)?;
    for scope_coverage in summary.coverage {
        writeln!(f, "  {}", scope_coverage.summary_line())?;
    }
    Ok(())
}

pub(crate) fn write_tolerance_policy_text(text: &mut String, comparison: &ComparisonReport) {
    use std::fmt::Write as _;

    let family_label = tolerance_backend_family_label(&comparison.candidate_backend.family);
    let summary = match validated_comparison_tolerance_policy_summary_for_report(comparison) {
        Ok(summary) => summary,
        Err(error) => {
            let _ = writeln!(text, "Tolerance policy catalog");
            let _ = writeln!(text, "  unavailable ({error})");
            return;
        }
    };
    let coordinate_frames = format_frames(&summary.coordinate_frames);
    let _ = writeln!(text, "Tolerance policy catalog");
    let _ = writeln!(text, "  candidate backend family: {}", family_label);
    let _ = writeln!(
        text,
        "  comparison evidence: {} bodies, {} samples",
        summary.comparison_body_count, summary.comparison_sample_count
    );
    let _ = writeln!(
        text,
        "  comparison window: {}",
        summary.comparison_window.summary_line()
    );
    let _ = writeln!(text, "  coordinate frames: {}", coordinate_frames);
    for scope_coverage in summary.coverage {
        let _ = writeln!(text, "  {}", scope_coverage.summary_line());
    }
}

/// Per-body comparison status against the expected tolerance table.
#[derive(Clone, Debug, PartialEq)]
pub struct BodyToleranceSummary {
    /// Body queried for this tolerance summary.
    pub body: CelestialBody,
    /// Expected tolerance for the body.
    pub tolerance: ComparisonTolerance,
    /// Number of samples compared for this body.
    pub sample_count: usize,
    /// Whether all measured deltas are within the expected tolerance.
    pub within_tolerance: bool,
    /// Maximum absolute longitude delta measured for this body.
    pub max_longitude_delta_deg: f64,
    /// Signed margin between the longitude limit and measured maximum.
    pub longitude_margin_deg: f64,
    /// Maximum absolute latitude delta measured for this body.
    pub max_latitude_delta_deg: f64,
    /// Signed margin between the latitude limit and measured maximum.
    pub latitude_margin_deg: f64,
    /// Maximum absolute distance delta measured for this body.
    pub max_distance_delta_au: Option<f64>,
    /// Signed margin between the distance limit and measured maximum.
    pub distance_margin_au: Option<f64>,
}

impl BodyToleranceSummary {
    /// Returns `Ok(())` when the tolerance status is internally consistent.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        validate_comparison_tolerance(&self.tolerance)?;

        if self.sample_count == 0 {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} has no samples to compare",
                    self.body
                ),
            ));
        }

        for (label, value) in [
            ("longitude", self.max_longitude_delta_deg),
            ("latitude", self.max_latitude_delta_deg),
            ("longitude margin", self.longitude_margin_deg),
            ("latitude margin", self.latitude_margin_deg),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body tolerance summary for {} has invalid {} {}",
                        self.body, label, value
                    ),
                ));
            }
        }

        if let Some(value) = self.max_distance_delta_au {
            if !value.is_finite() || value < 0.0 {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body tolerance summary for {} has invalid distance delta {}",
                        self.body, value
                    ),
                ));
            }
        }

        if let Some(value) = self.distance_margin_au {
            if !value.is_finite() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body tolerance summary for {} has invalid distance margin {}",
                        self.body, value
                    ),
                ));
            }
        }

        let tolerance = &self.tolerance;
        let distance_margin = self.distance_margin_au;
        let has_distance_limit = tolerance.max_distance_delta_au.is_some();
        let has_distance_measurement = self.max_distance_delta_au.is_some();
        if distance_margin.is_some() != (has_distance_limit && has_distance_measurement) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} distance-margin presence does not match the measured values and tolerance limit",
                    self.body
                ),
            ));
        }

        let expected_longitude_margin =
            tolerance.max_longitude_delta_deg - self.max_longitude_delta_deg;
        if self.longitude_margin_deg != expected_longitude_margin {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} longitude margin drifted from the declared tolerance limit",
                    self.body
                ),
            ));
        }

        let expected_latitude_margin =
            tolerance.max_latitude_delta_deg - self.max_latitude_delta_deg;
        if self.latitude_margin_deg != expected_latitude_margin {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} latitude margin drifted from the declared tolerance limit",
                    self.body
                ),
            ));
        }

        if let (Some(measured), Some(limit), Some(margin)) = (
            self.max_distance_delta_au,
            tolerance.max_distance_delta_au,
            self.distance_margin_au,
        ) {
            if margin != limit - measured {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "body tolerance summary for {} distance margin drifted from the declared tolerance limit",
                        self.body
                    ),
                ));
            }
        }

        let within_tolerance = self.longitude_margin_deg >= 0.0
            && self.latitude_margin_deg >= 0.0
            && self
                .distance_margin_au
                .map(|value| value >= 0.0)
                .unwrap_or(true);
        if self.within_tolerance != within_tolerance {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "body tolerance summary for {} status disagrees with the measured margins",
                    self.body
                ),
            ));
        }

        Ok(())
    }

    /// Renders the compact report wording after validating the summary fields.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Renders the compact report wording for this tolerance status.
    pub fn summary_line(&self) -> String {
        format!(
            "{}: backend family={}, profile={}, samples={}, status={}, limit Δlon≤{:.6}°, margin Δlon={:+.12}°, limit Δlat≤{:.6}°, margin Δlat={:+.12}°, limit Δdist={}, margin Δdist={}",
            self.body,
            tolerance_backend_family_label(&self.tolerance.backend_family),
            self.tolerance.profile,
            self.sample_count,
            if self.within_tolerance { "within" } else { "exceeded" },
            self.tolerance.max_longitude_delta_deg,
            self.longitude_margin_deg,
            self.tolerance.max_latitude_delta_deg,
            self.latitude_margin_deg,
            self.tolerance
                .max_distance_delta_au
                .map(|value| format!("{value:.6} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.distance_margin_au
                .map(|value| format!("{value:+.12} AU"))
                .unwrap_or_else(|| "n/a".to_string())
        )
    }
}

impl fmt::Display for BodyToleranceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Renders a compact workspace audit summary used by the CLI and release bundle.
pub fn render_workspace_audit_summary() -> Result<String, std::io::Error> {
    let report = workspace_audit_report()?;
    Ok(render_workspace_audit_summary_text(&report))
}

/// Renders the compact native-dependency audit summary used by release bundling.
///
/// This stays explicit even though it currently shares the same underlying report,
/// so release-bundle bookkeeping can keep the native-dependency path separate.
pub fn render_native_dependency_audit_summary() -> Result<String, std::io::Error> {
    render_workspace_audit_summary()
}

/// Benchmarks a backend against a validation corpus.
pub fn benchmark_backend(
    backend: &dyn EphemerisBackend,
    corpus: &ValidationCorpus,
    rounds: usize,
) -> Result<BenchmarkReport, EphemerisError> {
    let single_start = StdInstant::now();
    for _ in 0..rounds {
        for request in &corpus.requests {
            std::hint::black_box(backend.position(request)?);
        }
    }
    let elapsed = single_start.elapsed();

    let batch_start = StdInstant::now();
    for _ in 0..rounds {
        std::hint::black_box(backend.positions(&corpus.requests)?);
    }
    let batch_elapsed = batch_start.elapsed();

    let report = BenchmarkReport {
        backend: backend.metadata(),
        corpus_name: corpus.name.clone(),
        apparentness: corpus.apparentness,
        rounds,
        sample_count: corpus.requests.len(),
        elapsed,
        batch_elapsed,
        estimated_corpus_heap_bytes: corpus.estimated_heap_bytes(),
    };
    report.validate()?;
    Ok(report)
}

/// Computes a deterministic 64-bit checksum for bundle text.
pub(crate) fn checksum64(text: &str) -> u64 {
    checksum64_bytes(text.as_bytes())
}

/// Computes a deterministic 64-bit checksum for arbitrary bytes.
pub(crate) fn checksum64_bytes(bytes: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0001_0000_01b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Renders the compact compatibility-profile summary used by release tooling.
pub fn render_compatibility_profile_summary() -> String {
    render_compatibility_profile_summary_text()
}

/// Renders the compact compatibility-caveats summary used by release tooling.
pub fn render_compatibility_caveats_summary() -> String {
    render_compatibility_caveats_summary_text()
}

/// Renders the compact latitude-sensitive house failure modes summary used by release tooling.
pub fn render_house_latitude_sensitive_failure_modes_summary() -> String {
    format_latitude_sensitive_house_failure_modes_for_report()
}

/// Renders the compact known-gaps summary used by release tooling.
pub fn render_known_gaps_summary() -> String {
    render_known_gaps_summary_text()
}

/// Renders the compact compatibility catalog inventory summary used by release tooling.
pub fn render_catalog_inventory_summary() -> String {
    render_catalog_inventory_summary_text()
}

pub(crate) fn render_catalog_inventory_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(|| match validated_catalog_inventory_summary_for_report() {
            Ok(summary) => summary,
            Err(error) => format!("Compatibility catalog inventory unavailable ({error})"),
        })
        .clone()
}

pub(crate) fn render_known_gaps_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(|| match validated_known_gaps_summary_for_report() {
            Ok(summary) => format!("Known gaps: {summary}"),
            Err(error) => format!("Known gaps unavailable ({error})"),
        })
        .clone()
}

/// Renders the compact compatibility catalog posture summary used by release tooling.
pub fn render_catalog_posture_summary() -> String {
    render_catalog_posture_summary_text()
}

pub(crate) fn render_catalog_posture_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(
            || match core_validated_catalog_posture_summary_for_report() {
                Ok(summary) => summary,
                Err(error) => format!("Compatibility catalog posture unavailable ({error})"),
            },
        )
        .clone()
}

/// Renders the compact custom-definition ayanamsa label summary used by release tooling.
pub fn render_custom_definition_ayanamsa_labels_summary() -> String {
    format_custom_definition_ayanamsa_labels_for_report()
}

/// Renders the compact release-specific house-system canonical-name summary used by release tooling.
pub fn render_release_house_system_canonical_names_summary() -> String {
    format_release_house_system_canonical_names_for_report()
}

/// Renders the compact release-specific ayanamsa canonical-name summary used by release tooling.
pub fn render_release_ayanamsa_canonical_names_summary() -> String {
    format_release_ayanamsa_canonical_names_for_report()
}

/// Renders the compact ayanamsa audit summary used by release tooling.
pub fn render_ayanamsa_audit_summary() -> String {
    format_ayanamsa_audit_for_report()
}

/// Renders the compact target house-system scope summary used by release tooling.
pub fn render_target_house_scope_summary() -> String {
    match core_validated_target_house_scope_summary_for_report() {
        Ok(summary) => summary,
        Err(error) => format!("Compatibility profile target house scope unavailable ({error})"),
    }
}

/// Renders the compact target ayanamsa scope summary used by release tooling.
pub fn render_target_ayanamsa_scope_summary() -> String {
    match core_validated_target_ayanamsa_scope_summary_for_report() {
        Ok(summary) => summary,
        Err(error) => format!("Compatibility profile target ayanamsa scope unavailable ({error})"),
    }
}

/// Renders the release notes used by release tooling.
pub fn render_release_notes() -> String {
    render_release_notes_text()
}

/// Renders the compact release notes summary used by release tooling.
pub fn render_release_notes_summary() -> String {
    render_release_notes_summary_text()
}

/// Renders the release checklist used by release tooling.
pub fn render_release_checklist() -> String {
    render_release_checklist_text()
}

/// Renders the compact release checklist summary used by release tooling.
pub fn render_release_checklist_summary() -> String {
    render_release_checklist_summary_text()
}

/// Renders the compact release summary used by release tooling.
pub fn render_release_summary() -> String {
    render_release_summary_text()
}

/// Renders the compact Delta T policy summary used by validation and release tooling.
pub fn render_delta_t_policy_summary() -> String {
    render_delta_t_policy_summary_text()
}

/// Renders the compact request-policy summary used by validation and release tooling.
pub fn render_request_policy_summary() -> String {
    render_request_policy_summary_text()
}

/// Renders the compact request-surface inventory used by validation and release tooling.
pub fn render_request_surface_summary() -> String {
    render_request_surface_summary_text()
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AyanamsaReferenceOffsetExample {
    pub(crate) canonical_name: &'static str,
    pub(crate) epoch: JulianDay,
    pub(crate) offset_degrees: Angle,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AyanamsaReferenceOffsetsSummary {
    pub(crate) examples: Vec<AyanamsaReferenceOffsetExample>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AyanamsaProvenanceExample {
    pub(crate) canonical_name: &'static str,
    pub(crate) provenance_note: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AyanamsaProvenanceSummary {
    pub(crate) examples: Vec<AyanamsaProvenanceExample>,
}

impl AyanamsaReferenceOffsetsSummary {
    pub(crate) fn validate(&self) -> Result<(), EphemerisError> {
        validate_name_sequence(
            "ayanamsa reference offsets",
            self.examples.iter().map(|example| example.canonical_name),
        )?;

        Ok(())
    }

    pub(crate) fn summary_line(&self) -> String {
        match self.examples.as_slice() {
            [] => "representative zero-point examples: 0 (none)".to_string(),
            [single] => format!(
                "representative zero-point examples: 1 ({}: epoch={}; offset={})",
                single.canonical_name, single.epoch, single.offset_degrees
            ),
            _ => format!(
                "representative zero-point examples: {}",
                self.examples
                    .iter()
                    .map(|example| format!(
                        "{}: epoch={}; offset={}",
                        example.canonical_name, example.epoch, example.offset_degrees
                    ))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        }
    }
}

impl fmt::Display for AyanamsaReferenceOffsetsSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl AyanamsaProvenanceSummary {
    pub(crate) fn validate(&self) -> Result<(), EphemerisError> {
        validate_name_sequence(
            "ayanamsa provenance examples",
            self.examples.iter().map(|example| example.canonical_name),
        )?;

        for example in &self.examples {
            if example.provenance_note.trim().is_empty()
                || example.provenance_note.contains('\n')
                || example.provenance_note.contains('\r')
                || has_surrounding_whitespace(example.provenance_note)
            {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "ayanamsa provenance example `{}` has an unnormalized provenance note",
                        example.canonical_name
                    ),
                ));
            }
        }

        Ok(())
    }

    pub(crate) fn summary_line(&self) -> String {
        match self.examples.as_slice() {
            [] => "representative provenance examples: 0 (none)".to_string(),
            [single] => format!(
                "representative provenance examples: 1 ({} — {})",
                single.canonical_name, single.provenance_note
            ),
            _ => format!(
                "representative provenance examples: {}",
                self.examples
                    .iter()
                    .map(|example| format!(
                        "{} — {}",
                        example.canonical_name, example.provenance_note
                    ))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        }
    }

    pub(crate) fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for AyanamsaProvenanceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn summarize_ayanamsa_reference_offsets(
) -> Result<AyanamsaReferenceOffsetsSummary, EphemerisError> {
    let samples = pleiades_ayanamsa::reference_offset_sample_ayanamsas();

    let mut examples = Vec::with_capacity(samples.len());
    for sample in samples {
        let descriptor = descriptor(sample).ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("ayanamsa reference offsets sample `{sample}` is unavailable"),
            )
        })?;
        let epoch = descriptor.epoch.ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "ayanamsa reference offsets sample `{}` is missing its reference epoch",
                    descriptor.canonical_name
                ),
            )
        })?;
        let offset_degrees = descriptor.offset_degrees.ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "ayanamsa reference offsets sample `{}` is missing its reference offset",
                    descriptor.canonical_name
                ),
            )
        })?;

        examples.push(AyanamsaReferenceOffsetExample {
            canonical_name: descriptor.canonical_name,
            epoch,
            offset_degrees,
        });
    }

    let summary = AyanamsaReferenceOffsetsSummary { examples };
    summary.validate()?;
    Ok(summary)
}

pub(crate) fn validated_ayanamsa_reference_offsets_summary_for_report(
    summary: &AyanamsaReferenceOffsetsSummary,
) -> Result<String, EphemerisError> {
    summary.validate()?;
    Ok(summary.to_string())
}

pub(crate) fn format_ayanamsa_reference_offsets_for_report() -> String {
    match summarize_ayanamsa_reference_offsets() {
        Ok(summary) => match validated_ayanamsa_reference_offsets_summary_for_report(&summary) {
            Ok(summary) => format!("Ayanamsa reference offsets: {summary}"),
            Err(error) => format!("Ayanamsa reference offsets: unavailable ({error})"),
        },
        Err(error) => format!("Ayanamsa reference offsets: unavailable ({error})"),
    }
}

pub(crate) fn summarize_ayanamsa_provenance() -> Result<AyanamsaProvenanceSummary, EphemerisError> {
    let samples = pleiades_ayanamsa::provenance_sample_ayanamsas();

    let mut examples = Vec::with_capacity(samples.len());
    for sample in samples {
        let descriptor = descriptor(sample).ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("ayanamsa provenance sample `{sample}` is unavailable"),
            )
        })?;

        examples.push(AyanamsaProvenanceExample {
            canonical_name: descriptor.canonical_name,
            provenance_note: descriptor.notes,
        });
    }

    let summary = AyanamsaProvenanceSummary { examples };
    summary.validate()?;
    Ok(summary)
}

pub(crate) fn format_ayanamsa_catalog_validation_for_report() -> String {
    match ayanamsa_catalog_validation_summary().validated_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("ayanamsa catalog validation: unavailable ({error})"),
    }
}

pub(crate) fn format_ayanamsa_metadata_coverage_for_report() -> String {
    match metadata_coverage().validated_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("ayanamsa sidereal metadata: unavailable ({error})"),
    }
}

pub(crate) fn format_ayanamsa_provenance_for_report() -> String {
    match summarize_ayanamsa_provenance() {
        Ok(summary) => match summary.validated_summary_line() {
            Ok(summary) => format!("Ayanamsa provenance: {summary}"),
            Err(error) => format!("Ayanamsa provenance: unavailable ({error})"),
        },
        Err(error) => format!("Ayanamsa provenance: unavailable ({error})"),
    }
}

pub(crate) fn format_ayanamsa_audit_for_report() -> String {
    format!(
        "Ayanamsa audit: {}; {}; {}; {}",
        format_ayanamsa_catalog_validation_for_report(),
        format_ayanamsa_metadata_coverage_for_report(),
        format_ayanamsa_reference_offsets_for_report(),
        format_ayanamsa_provenance_for_report(),
    )
}

pub(crate) fn format_house_code_aliases_for_report() -> String {
    match pleiades_houses::validated_house_system_code_aliases_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("house-code aliases unavailable ({error})"),
    }
}

pub(crate) fn format_house_formula_families_for_report() -> String {
    match validated_house_formula_families_summary_for_report() {
        Ok(summary) => format!("House formula families: {summary}"),
        Err(error) => format!("house formula families unavailable ({error})"),
    }
}

pub(crate) fn format_latitude_sensitive_house_systems_for_report() -> String {
    match validated_latitude_sensitive_house_systems_summary_for_report() {
        Ok(summary) => format!("Latitude-sensitive house systems: {summary}"),
        Err(error) => format!("Latitude-sensitive house systems unavailable ({error})"),
    }
}

pub(crate) fn format_latitude_sensitive_house_constraints_for_report() -> String {
    match validated_latitude_sensitive_house_constraints_summary_for_report() {
        Ok(summary) => format!("Latitude-sensitive house constraints: {summary}"),
        Err(error) => format!("Latitude-sensitive house constraints unavailable ({error})"),
    }
}

pub(crate) fn format_latitude_sensitive_house_failure_modes_for_report() -> String {
    match validated_latitude_sensitive_house_failure_modes_summary_for_report() {
        Ok(summary) => format!("Latitude-sensitive house failure modes: {summary}"),
        Err(error) => format!("Latitude-sensitive house failure modes unavailable ({error})"),
    }
}

pub(crate) fn format_custom_definition_ayanamsa_labels_for_report() -> String {
    match validated_custom_definition_ayanamsa_labels_summary_for_report() {
        Ok(summary) => summary,
        Err(error) => format!("custom-definition ayanamsa labels unavailable ({error})"),
    }
}

pub(crate) fn validated_release_house_system_canonical_names_for_report() -> Result<String, String>
{
    core_validated_release_house_system_canonical_names_summary_for_report()
        .map_err(|error| error.to_string())
}

pub(crate) fn validated_release_ayanamsa_canonical_names_for_report() -> Result<String, String> {
    core_validated_release_ayanamsa_canonical_names_summary_for_report()
        .map_err(|error| error.to_string())
}

pub(crate) fn format_release_house_system_canonical_names_for_report() -> String {
    match validated_release_house_system_canonical_names_for_report() {
        Ok(summary) => format!("Release-specific house-system canonical names: {summary}"),
        Err(error) => {
            format!("Release-specific house-system canonical names unavailable ({error})")
        }
    }
}

pub(crate) fn format_release_ayanamsa_canonical_names_for_report() -> String {
    match validated_release_ayanamsa_canonical_names_for_report() {
        Ok(summary) => format!("Release-specific ayanamsa canonical names: {summary}"),
        Err(error) => {
            format!("Release-specific ayanamsa canonical names unavailable ({error})")
        }
    }
}

pub(crate) fn validate_name_sequence<'a, I>(
    section_label: &'static str,
    names: I,
) -> Result<(), EphemerisError>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut seen_names = BTreeSet::new();
    let mut seen_names_case_insensitive = BTreeMap::new();

    for name in names {
        if name.trim().is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("{section_label} contains a blank name"),
            ));
        }

        if has_surrounding_whitespace(name) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("{section_label} entry '{name}' contains surrounding whitespace"),
            ));
        }

        let normalized_name = name.trim().to_string();
        if !seen_names.insert(normalized_name.clone()) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("{section_label} contains a duplicate name '{name}'"),
            ));
        }

        let normalized_name_case_insensitive = normalized_name.to_ascii_lowercase();
        if let Some(existing_name) =
            seen_names_case_insensitive.get(&normalized_name_case_insensitive)
        {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "{section_label} contains a case-insensitive duplicate name '{name}' that conflicts with '{existing_name}'"
                ),
            ));
        }
        seen_names_case_insensitive.insert(normalized_name_case_insensitive, normalized_name);
    }

    Ok(())
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DescriptorNamesSummary {
    pub(crate) names: Vec<&'static str>,
}

#[cfg(test)]
impl DescriptorNamesSummary {
    pub(crate) fn validate(&self) -> Result<(), EphemerisError> {
        validate_name_sequence("descriptor-name summary", self.names.iter().copied())
    }

    pub(crate) fn summary_line(&self) -> String {
        match self.names.as_slice() {
            [] => "0 (none)".to_string(),
            [single] => format!("1 ({single})"),
            _ => format!("{} ({})", self.names.len(), self.names.join(", ")),
        }
    }
}

#[cfg(test)]
impl fmt::Display for DescriptorNamesSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

#[cfg(test)]
pub(crate) fn summarize_descriptor_names<T>(
    entries: &[T],
    canonical_name: impl Fn(&T) -> &'static str,
) -> DescriptorNamesSummary {
    DescriptorNamesSummary {
        names: entries.iter().map(canonical_name).collect::<Vec<_>>(),
    }
}

pub(crate) fn validate_compatibility_profile_summary_text(
    text: &str,
    profile: &CompatibilityProfile,
    release_profiles: &ReleaseProfileIdentifiers,
) -> Result<(), String> {
    let expected_house_line = format!(
        "House systems: {} total ({} baseline, {} release-specific)",
        profile.house_systems.len(),
        profile.baseline_house_systems.len(),
        profile.release_house_systems.len()
    );
    if !text.contains(&expected_house_line) {
        return Err(format!(
            "compatibility profile summary house-system baseline/release split mismatch: expected `{expected_house_line}`"
        ));
    }

    let expected_constraints_line = format!(
        "Latitude-sensitive house constraints: {}",
        profile.latitude_sensitive_house_constraints_summary_line()
    );
    if !text.contains(&expected_constraints_line) {
        return Err(format!(
            "compatibility profile summary latitude-sensitive house constraints mismatch: expected `{expected_constraints_line}`"
        ));
    }

    let expected_ayanamsa_line = format!(
        "Ayanamsas: {} total ({} baseline, {} release-specific)",
        profile.ayanamsas.len(),
        profile.baseline_ayanamsas.len(),
        profile.release_ayanamsas.len()
    );
    if !text.contains(&expected_ayanamsa_line) {
        return Err(format!(
            "compatibility profile summary ayanamsa baseline/release split mismatch: expected `{expected_ayanamsa_line}`"
        ));
    }

    let expected_profile_line = format!("Profile: {}", release_profiles.compatibility_profile_id);
    if !text.contains(&expected_profile_line) {
        return Err(format!(
            "compatibility profile summary profile id mismatch: expected `{expected_profile_line}`"
        ));
    }

    let expected_unsupported_modes_line = format!(
        "Unsupported modes: {}",
        unsupported_modes_summary_for_report()
    );
    if !text.contains(&expected_unsupported_modes_line) {
        return Err(format!(
            "compatibility profile summary unsupported-modes mismatch: expected `{expected_unsupported_modes_line}`"
        ));
    }

    Ok(())
}

pub(crate) fn render_compatibility_profile_summary_text() -> String {
    let profile = match validated_compatibility_profile_for_report() {
        Ok(profile) => profile,
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    };
    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    };
    let coverage = metadata_coverage();
    let mut text = String::new();

    text.push_str("Compatibility profile summary\n");
    text.push_str("Profile: ");
    text.push_str(release_profiles.compatibility_profile_id);
    text.push('\n');
    text.push_str("House systems: ");
    text.push_str(&profile.house_systems.len().to_string());
    text.push_str(" total (");
    text.push_str(&profile.baseline_house_systems.len().to_string());
    text.push_str(" baseline, ");
    text.push_str(&profile.release_house_systems.len().to_string());
    text.push_str(" release-specific)\n");
    text.push_str(&format_latitude_sensitive_house_systems_for_report());
    text.push('\n');
    text.push_str(&format_latitude_sensitive_house_constraints_for_report());
    text.push('\n');
    text.push_str(&format_house_formula_families_for_report());
    text.push('\n');
    text.push_str("House code aliases: ");
    match profile.validated_house_code_aliases_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Ayanamsas: ");
    text.push_str(&profile.ayanamsas.len().to_string());
    text.push_str(" total (");
    text.push_str(&profile.baseline_ayanamsas.len().to_string());
    text.push_str(" baseline, ");
    text.push_str(&profile.release_ayanamsas.len().to_string());
    text.push_str(" release-specific)\n");
    match profile.validated_target_house_scope_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    }
    text.push('\n');
    match profile.validated_target_ayanamsa_scope_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    }
    text.push('\n');
    match coverage.validated_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str(&format_ayanamsa_catalog_validation_for_report());
    text.push('\n');
    text.push_str(&format_ayanamsa_metadata_coverage_for_report());
    text.push('\n');
    text.push_str(&format_ayanamsa_reference_offsets_for_report());
    text.push('\n');
    text.push_str(&format_ayanamsa_provenance_for_report());
    text.push('\n');
    text.push_str("Release-specific house-system canonical names: ");
    match profile.validated_release_house_system_canonical_names_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Release-specific ayanamsa canonical names: ");
    match profile.validated_release_ayanamsa_canonical_names_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Compatibility profile summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Custom-definition labels: ");
    text.push_str(&profile.custom_definition_labels.len().to_string());
    text.push('\n');
    text.push_str("Custom-definition label names: ");
    if profile.custom_definition_labels.is_empty() {
        text.push_str("none");
    } else {
        text.push_str(&profile.custom_definition_labels.join(", "));
    }
    text.push('\n');
    text.push_str("Validation reference points: ");
    text.push_str(&summarize_validation_reference_points(
        profile.validation_reference_points,
    ));
    text.push('\n');
    text.push_str("Compatibility caveats: ");
    text.push_str(&profile.known_gaps.len().to_string());
    text.push('\n');
    text.push_str("Compatibility caveats documented: ");
    if profile.known_gaps.is_empty() {
        text.push_str("none");
    } else {
        text.push_str(&profile.known_gaps.join("; "));
    }
    text.push('\n');
    text.push_str("Unsupported modes: ");
    text.push_str(unsupported_modes_summary_for_report());
    text.push('\n');
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push_str("Compact summary views: backend-matrix-summary, api-stability-summary, workspace-audit-summary, validation-report-summary / validation-summary / report-summary, artifact-summary / artifact-posture-summary, release-checklist-summary\n");
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Release summary: release-summary\n");
    text.push_str("Release checklist summary: release-checklist-summary\n");
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("See release-summary for the compact one-screen release overview.\n");

    if let Err(error) =
        validate_compatibility_profile_summary_text(&text, &profile, &release_profiles)
    {
        return format!("Compatibility profile summary unavailable ({error})");
    }

    text
}

pub(crate) fn render_compatibility_caveats_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let profile = match validated_compatibility_profile_for_report() {
                Ok(profile) => profile,
                Err(error) => {
                    return format!("Compatibility caveats summary unavailable ({error})")
                }
            };
            let release_profiles = match validated_release_profile_identifiers_for_report() {
                Ok(release_profiles) => release_profiles,
                Err(error) => {
                    return format!("Compatibility caveats summary unavailable ({error})")
                }
            };
            core_compatibility_caveats_summary_for_report(&profile, &release_profiles)
        })
        .clone()
}

pub(crate) fn validate_packaged_artifact_fit_posture() -> Result<(), EphemerisError> {
    let fit_envelope = packaged_artifact_fit_envelope_summary_details();
    let thresholds = packaged_artifact_fit_threshold_summary_details();
    let target_threshold = packaged_artifact_target_threshold_summary_details();
    validate_packaged_artifact_fit_posture_with(&fit_envelope, &thresholds, &target_threshold)
}

pub(crate) fn validate_packaged_artifact_fit_posture_with(
    fit_envelope: &pleiades_data::PackagedArtifactFitEnvelopeSummary,
    thresholds: &pleiades_data::PackagedArtifactFitThresholdSummary,
    target_threshold: &pleiades_data::PackagedArtifactTargetThresholdSummary,
) -> Result<(), EphemerisError> {
    fit_envelope.validate().map_err(|error| {
        EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("validation report packaged-artifact fit envelope is invalid: {error}"),
        )
    })?;
    fit_envelope
        .validate_against_thresholds(thresholds)
        .map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "validation report packaged-artifact fit envelope exceeds calibrated thresholds: {error}; measured fit envelope: {}; fit thresholds: {}",
                    fit_envelope.summary_line(),
                    thresholds.summary_line(),
                ),
            )
        })?;
    target_threshold.validate().map_err(|error| {
        EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "validation report packaged-artifact target-threshold summary is invalid: {error}"
            ),
        )
    })?;

    Ok(())
}

pub(crate) fn build_validation_report(rounds: usize) -> Result<ValidationReport, EphemerisError> {
    static CACHE: OnceLock<Mutex<HashMap<usize, ValidationReport>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache
        .lock()
        .expect("validation report cache should be lockable");

    if let Some(report) = cache.get(&rounds).cloned() {
        return Ok(report);
    }

    let report = build_validation_report_uncached(rounds)?;
    cache.insert(rounds, report.clone());
    Ok(report)
}

pub(crate) fn build_validation_report_uncached(
    rounds: usize,
) -> Result<ValidationReport, EphemerisError> {
    validate_packaged_artifact_fit_posture()?;
    let comparison_corpus = release_grade_corpus();
    let benchmark_corpus = benchmark_timing_corpus();
    let packaged_benchmark_corpus = artifact::packaged_artifact_corpus();
    let chart_benchmark_corpus = chart_benchmark_corpus_summary();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let packaged = PackagedDataBackend::new();
    let comparison = compare_backends(&reference, &candidate, &comparison_corpus)?;
    let reference_benchmark = benchmark_backend(&reference, &comparison_corpus, rounds)?;
    let candidate_benchmark = benchmark_backend(&candidate, &benchmark_corpus, rounds)?;
    let packaged_benchmark = benchmark_backend(&packaged, &packaged_benchmark_corpus, rounds)?;
    let artifact_decode_benchmark =
        artifact::benchmark_packaged_artifact_decode(rounds).map_err(|error| {
            EphemerisError::new(EphemerisErrorKind::MissingDataset, error.to_string())
        })?;
    let chart_benchmark = benchmark_chart_backend(default_candidate_backend(), rounds)?;
    let archived_regressions = comparison.regression_archive();

    let report = ValidationReport {
        comparison_corpus: comparison_corpus.summary(),
        benchmark_corpus: benchmark_corpus.summary(),
        packaged_benchmark_corpus: packaged_benchmark_corpus.summary(),
        chart_benchmark_corpus,
        artifact_decode_benchmark,
        house_validation: house_validation_report(),
        comparison,
        archived_regressions,
        reference_benchmark,
        candidate_benchmark,
        packaged_benchmark,
        chart_benchmark,
    };
    report.validate()?;
    Ok(report)
}

/// Renders the validation report used by the CLI.
pub fn render_validation_report(rounds: usize) -> Result<String, EphemerisError> {
    static CACHE: OnceLock<Mutex<HashMap<usize, String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache
        .lock()
        .expect("validation report cache should be lockable");

    if let Some(report) = cache.get(&rounds).cloned() {
        return Ok(report);
    }

    let report = render_validation_report_uncached(rounds)?;
    cache.insert(rounds, report.clone());
    Ok(report)
}

pub(crate) fn render_validation_report_uncached(rounds: usize) -> Result<String, EphemerisError> {
    Ok(build_validation_report(rounds)?.to_string())
}

/// Renders a compact validation-report summary used by the CLI.
pub fn render_validation_report_summary(rounds: usize) -> Result<String, EphemerisError> {
    static CACHE: OnceLock<Mutex<HashMap<usize, String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache
        .lock()
        .expect("validation report summary cache should be lockable");

    if let Some(report) = cache.get(&rounds).cloned() {
        return Ok(report);
    }

    let report = render_validation_report_summary_uncached(rounds)?;
    cache.insert(rounds, report.clone());
    Ok(report)
}

pub(crate) fn render_validation_report_summary_uncached(
    rounds: usize,
) -> Result<String, EphemerisError> {
    let report = build_validation_report(rounds)?;
    Ok(render_validation_report_summary_text(&report))
}

pub(crate) fn validated_packaged_artifact_fit_sample_classes_summary_for_report(
    boundary: &ArtifactBoundaryEnvelopeSummary,
) -> Result<String, String> {
    let boundary = boundary
        .validated_summary_line()
        .map_err(|error| error.to_string())?;
    let interior = packaged_artifact_fit_envelope_summary_for_report();

    Ok(format!(
        "fit sample classes: boundary continuity={}; interior fit={}",
        boundary, interior,
    ))
}

/// Returns the combined packaged-artifact boundary and interior fit sample summary for reports.
pub fn packaged_artifact_fit_sample_classes_summary_for_report() -> String {
    let boundary = match artifact_boundary_envelope_summary_for_report() {
        Ok(boundary) => boundary,
        Err(error) => return format!("fit sample classes: unavailable ({error})"),
    };

    match validated_packaged_artifact_fit_sample_classes_summary_for_report(&boundary) {
        Ok(summary) => summary,
        Err(error) => format!("fit sample classes: unavailable ({error})"),
    }
}

/// Renders the comparison report used by the CLI.
pub fn render_comparison_report() -> Result<String, EphemerisError> {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    Ok(compare_backends(&reference, &candidate, &corpus)?.to_string())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComparisonMedianEnvelope {
    pub(crate) longitude_delta_deg: f64,
    pub(crate) latitude_delta_deg: f64,
    pub(crate) distance_delta_au: Option<f64>,
}

impl ComparisonMedianEnvelope {
    /// Validates the stored median comparison envelope.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        for (label, value) in [
            ("longitude_delta_deg", self.longitude_delta_deg),
            ("latitude_delta_deg", self.latitude_delta_deg),
        ] {
            if !value.is_finite() || value.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison median envelope field `{label}` must be a finite non-negative value"
                    ),
                ));
            }
        }

        if let Some(distance_delta_au) = self.distance_delta_au {
            if !distance_delta_au.is_finite() || distance_delta_au.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "comparison median envelope field `distance_delta_au` must be a finite non-negative value",
                ));
            }
        }

        Ok(())
    }

    /// Returns the compact median comparison envelope line.
    pub fn summary_line(&self) -> String {
        let distance = self
            .distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string());

        format!(
            "median longitude delta: {:.12}°, median latitude delta: {:.12}°, median distance delta: {}",
            self.longitude_delta_deg, self.latitude_delta_deg, distance,
        )
    }
}

impl fmt::Display for ComparisonMedianEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the median comparison envelope used by the compact report.
pub fn comparison_median_envelope(
    samples: &[ComparisonSample],
) -> Result<ComparisonMedianEnvelope, EphemerisError> {
    validate_comparison_samples_for_report(samples)?;

    let envelope = comparison_median_envelope_for_samples(samples);
    envelope.validate()?;
    Ok(envelope)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComparisonPercentileEnvelope {
    pub(crate) longitude_delta_deg: f64,
    pub(crate) latitude_delta_deg: f64,
    pub(crate) distance_delta_au: Option<f64>,
}

impl ComparisonPercentileEnvelope {
    /// Validates the stored 95th-percentile comparison envelope.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        for (label, value) in [
            ("longitude_delta_deg", self.longitude_delta_deg),
            ("latitude_delta_deg", self.latitude_delta_deg),
        ] {
            if !value.is_finite() || value.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison percentile envelope field `{label}` must be a finite non-negative value"
                    ),
                ));
            }
        }

        if let Some(distance_delta_au) = self.distance_delta_au {
            if !distance_delta_au.is_finite() || distance_delta_au.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "comparison percentile envelope field `distance_delta_au` must be a finite non-negative value",
                ));
            }
        }

        Ok(())
    }

    /// Returns the compact 95th-percentile comparison envelope line.
    pub fn summary_line(&self) -> String {
        let distance = self
            .distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string());

        format!(
            "95th percentile absolute deltas: longitude {:.12}°, latitude {:.12}°, distance {}",
            self.longitude_delta_deg, self.latitude_delta_deg, distance,
        )
    }
}

impl fmt::Display for ComparisonPercentileEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the 95th-percentile comparison envelope used by the compact tail report.
pub fn comparison_tail_envelope(
    samples: &[ComparisonSample],
) -> Result<ComparisonPercentileEnvelope, EphemerisError> {
    validate_comparison_samples_for_report(samples)?;

    let envelope = comparison_percentile_envelope(samples, 0.95);
    envelope.validate()?;
    Ok(envelope)
}

/// Combined comparison envelope summary used by the compact report.
///
/// The summary keeps the aggregate comparison record, the median deltas, and
/// the 95th-percentile tail together so downstream tooling can reuse the same
/// validated envelope that the report formatter renders.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonEnvelopeSummary {
    pub(crate) summary: ComparisonSummary,
    pub(crate) median: ComparisonMedianEnvelope,
    pub(crate) percentile: ComparisonPercentileEnvelope,
}

impl ComparisonEnvelopeSummary {
    /// Returns the compact comparison summary line with the median envelope.
    pub fn summary_line(&self) -> String {
        let summary = self
            .summary
            .validated_summary_line()
            .unwrap_or_else(|error| format!("comparison summary unavailable ({error})"));
        format!("{}; {}", summary, self.median)
    }

    /// Returns the compact comparison summary line after validating against samples.
    pub fn validated_summary_line(
        &self,
        samples: &[ComparisonSample],
    ) -> Result<String, EphemerisError> {
        self.validate_against_samples(samples)?;
        Ok(self.summary_line())
    }

    /// Returns the compact 95th-percentile tail line.
    pub fn percentile_line(&self) -> String {
        self.percentile.summary_line()
    }

    /// Returns the compact 95th-percentile tail line after validating against samples.
    pub fn validated_percentile_line(
        &self,
        samples: &[ComparisonSample],
    ) -> Result<String, EphemerisError> {
        self.validate_against_samples(samples)?;
        Ok(self.percentile_line())
    }

    /// Validates the stored envelope against the provided comparison samples.
    pub fn validate_against_samples(
        &self,
        samples: &[ComparisonSample],
    ) -> Result<(), EphemerisError> {
        self.summary.validate()?;

        if self.summary.sample_count != samples.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "comparison envelope summary sample-count mismatch: expected {}, found {}",
                    self.summary.sample_count,
                    samples.len()
                ),
            ));
        }

        if samples.is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "comparison envelope summary has no samples",
            ));
        }

        for (index, sample) in samples.iter().enumerate() {
            for (label, value) in [
                ("longitude_delta_deg", sample.longitude_delta_deg),
                ("latitude_delta_deg", sample.latitude_delta_deg),
            ] {
                if !value.is_finite() || value.is_sign_negative() {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!(
                            "comparison sample {} field `{label}` must be a finite non-negative value",
                            index + 1
                        ),
                    ));
                }
            }

            if let Some(distance_delta_au) = sample.distance_delta_au {
                if !distance_delta_au.is_finite() || distance_delta_au.is_sign_negative() {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!(
                            "comparison sample {} field `distance_delta_au` must be a finite non-negative value",
                            index + 1
                        ),
                    ));
                }
            }
        }

        validate_comparison_sample_distance_channels(samples)?;
        self.median.validate()?;
        self.percentile.validate()?;

        let expected_median = comparison_median_envelope_for_samples(samples);
        if self.median != expected_median {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "comparison envelope summary median drifted from the sampled comparison values",
            ));
        }

        let expected_percentile = comparison_percentile_envelope(samples, 0.95);
        if self.percentile != expected_percentile {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "comparison envelope summary percentile drifted from the sampled comparison values",
            ));
        }

        Ok(())
    }
}

impl fmt::Display for ComparisonEnvelopeSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the combined comparison envelope summary used by the compact report.
pub fn comparison_envelope_summary(
    summary: &ComparisonSummary,
    samples: &[ComparisonSample],
) -> ComparisonEnvelopeSummary {
    ComparisonEnvelopeSummary {
        summary: summary.clone(),
        median: comparison_median_envelope_for_samples(samples),
        percentile: comparison_percentile_envelope(samples, 0.95),
    }
}

pub(crate) fn median_value(values: &mut [f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    values.sort_by(|left, right| left.total_cmp(right));
    let middle = values.len() / 2;
    if values.len().is_multiple_of(2) {
        Some((values[middle - 1] + values[middle]) / 2.0)
    } else {
        Some(values[middle])
    }
}

pub(crate) fn percentile_value(values: &mut [f64], percentile: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    values.sort_by(|left, right| left.total_cmp(right));
    let percentile = percentile.clamp(0.0, 1.0);
    let position = percentile * (values.len().saturating_sub(1)) as f64;
    let lower_index = position.floor() as usize;
    let upper_index = position.ceil() as usize;
    if lower_index == upper_index {
        Some(values[lower_index])
    } else {
        let weight = position - lower_index as f64;
        Some(values[lower_index] + (values[upper_index] - values[lower_index]) * weight)
    }
}

pub(crate) fn comparison_median_envelope_for_samples(
    samples: &[ComparisonSample],
) -> ComparisonMedianEnvelope {
    let mut longitude_values = samples
        .iter()
        .map(|sample| sample.longitude_delta_deg)
        .collect::<Vec<_>>();
    let mut latitude_values = samples
        .iter()
        .map(|sample| sample.latitude_delta_deg)
        .collect::<Vec<_>>();
    let mut distance_values = samples
        .iter()
        .filter_map(|sample| sample.distance_delta_au)
        .collect::<Vec<_>>();

    ComparisonMedianEnvelope {
        longitude_delta_deg: median_value(&mut longitude_values).unwrap_or_default(),
        latitude_delta_deg: median_value(&mut latitude_values).unwrap_or_default(),
        distance_delta_au: median_value(&mut distance_values),
    }
}

pub(crate) fn comparison_percentile_envelope(
    samples: &[ComparisonSample],
    percentile: f64,
) -> ComparisonPercentileEnvelope {
    let mut longitude_values = samples
        .iter()
        .map(|sample| sample.longitude_delta_deg)
        .collect::<Vec<_>>();
    let mut latitude_values = samples
        .iter()
        .map(|sample| sample.latitude_delta_deg)
        .collect::<Vec<_>>();
    let mut distance_values = samples
        .iter()
        .filter_map(|sample| sample.distance_delta_au)
        .collect::<Vec<_>>();

    ComparisonPercentileEnvelope {
        longitude_delta_deg: percentile_value(&mut longitude_values, percentile)
            .unwrap_or_default(),
        latitude_delta_deg: percentile_value(&mut latitude_values, percentile).unwrap_or_default(),
        distance_delta_au: percentile_value(&mut distance_values, percentile),
    }
}

pub(crate) fn format_comparison_percentile_envelope_for_report(
    samples: &[ComparisonSample],
) -> String {
    match comparison_tail_envelope(samples) {
        Ok(envelope) => envelope.summary_line(),
        Err(error) => format!("comparison percentile envelope unavailable ({error})"),
    }
}

pub(crate) fn format_comparison_envelope_for_report(
    summary: &ComparisonSummary,
    samples: &[ComparisonSample],
) -> String {
    let envelope = comparison_envelope_summary(summary, samples);
    match envelope.validated_summary_line(samples) {
        Ok(rendered) => rendered,
        Err(error) => format!("comparison envelope unavailable ({error})"),
    }
}

pub(crate) fn format_body_class_comparison_envelope_for_report(
    summary: &BodyClassSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("body-class error envelope unavailable ({error})"),
    }
}

pub(crate) fn comparison_body_class_error_envelope_summaries_for_report(
) -> Result<Vec<BodyClassSummary>, String> {
    let report = comparison_report_for_default_render()?;
    let summaries = report.body_class_summaries();

    if summaries.is_empty() {
        return Err("comparison report did not produce any body-class error envelopes".to_string());
    }

    for summary in &summaries {
        summary.validate().map_err(|error| error.to_string())?;
    }

    Ok(summaries)
}

pub(crate) fn comparison_body_class_error_envelope_summary_for_report() -> String {
    match comparison_body_class_error_envelope_summaries_for_report() {
        Ok(summaries) => format!("{} classes checked", summaries.len()),
        Err(error) => format!("body-class error envelopes unavailable ({error})"),
    }
}

pub(crate) fn render_comparison_body_class_error_envelope_summary_text_from_summaries(
    summaries: Result<Vec<BodyClassSummary>, String>,
) -> String {
    use std::fmt::Write as _;

    let summaries = match summaries {
        Ok(summaries) => summaries,
        Err(error) => {
            return format!(
                "Comparison body-class error envelope summary\nComparison body-class error envelope unavailable ({error})\n"
            );
        }
    };

    if summaries.is_empty() {
        return "Comparison body-class error envelope summary\nComparison body-class error envelope unavailable (comparison report did not produce any body-class error envelopes)\n".to_string();
    }

    for summary in &summaries {
        if let Err(error) = summary.validate() {
            return format!(
                "Comparison body-class error envelope summary\nComparison body-class error envelope unavailable ({error})\n"
            );
        }
    }

    let mut text = String::from("Comparison body-class error envelope summary\n");
    let _ = writeln!(text, "Body-class error envelopes: {}", summaries.len());
    for summary in summaries {
        let _ = writeln!(
            text,
            "  {}: {}",
            summary.class.label(),
            summary.summary_line()
        );
    }
    text
}

pub(crate) fn render_comparison_body_class_error_envelope_summary_text() -> String {
    render_comparison_body_class_error_envelope_summary_text_from_summaries(
        comparison_body_class_error_envelope_summaries_for_report(),
    )
}

pub(crate) fn format_body_class_tolerance_envelope_for_report(
    summary: &BodyClassToleranceSummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("body-class tolerance envelope unavailable ({error})"),
    }
}

pub(crate) fn comparison_report_for_default_render() -> Result<ComparisonReport, String> {
    compare_backends(
        &default_reference_backend(),
        &default_candidate_backend(),
        &default_corpus(),
    )
    .map_err(|error| error.to_string())
}

pub(crate) fn validated_comparison_body_class_tolerance_posture_line(
    report: &ComparisonReport,
) -> Result<String, String> {
    use std::fmt::Write as _;

    let summaries = report.body_class_tolerance_summaries();
    if summaries.is_empty() {
        return Err(
            "comparison report did not produce any body-class tolerance summaries".to_string(),
        );
    }

    let outlier_class_count = summaries
        .iter()
        .filter(|summary| summary.outside_tolerance_body_count > 0)
        .count();
    let outlier_bodies = summaries
        .iter()
        .flat_map(|summary| summary.outside_bodies.iter().cloned())
        .collect::<Vec<_>>();

    let mut text = String::new();
    let _ = write!(
        text,
        "body-class tolerance posture: {} classes checked, {} classes with outlier bodies, outlier bodies: {}",
        summaries.len(),
        outlier_class_count,
        if outlier_bodies.is_empty() {
            "none".to_string()
        } else {
            format_bodies(&outlier_bodies)
        }
    );
    Ok(text)
}

pub(crate) fn validated_comparison_body_class_tolerance_posture_for_report(
) -> Result<String, String> {
    let report = comparison_report_for_default_render()?;
    validated_comparison_body_class_tolerance_posture_line(&report)
}

pub(crate) fn format_body_class_tolerance_posture_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();

    SUMMARY
        .get_or_init(|| {
            validated_comparison_body_class_tolerance_posture_for_report().unwrap_or_else(|error| {
                format!("body-class tolerance posture unavailable ({error})")
            })
        })
        .clone()
}

pub(crate) fn validate_comparison_sample_distance_channels(
    samples: &[ComparisonSample],
) -> Result<(), EphemerisError> {
    let has_distance = samples
        .iter()
        .any(|sample| sample.distance_delta_au.is_some());
    let has_missing_distance = samples
        .iter()
        .any(|sample| sample.distance_delta_au.is_none());

    if has_distance && has_missing_distance {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            "comparison sample slice must either provide distance deltas for every sample or for none of them",
        ));
    }

    Ok(())
}

pub(crate) fn validate_comparison_samples_for_report(
    samples: &[ComparisonSample],
) -> Result<(), EphemerisError> {
    if samples.is_empty() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            "comparison sample slice is empty",
        ));
    }

    for (index, sample) in samples.iter().enumerate() {
        for (label, value) in [
            ("longitude_delta_deg", sample.longitude_delta_deg),
            ("latitude_delta_deg", sample.latitude_delta_deg),
        ] {
            if !value.is_finite() || value.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison sample {} field `{label}` must be a finite non-negative value",
                        index + 1
                    ),
                ));
            }
        }

        if let Some(distance_delta_au) = sample.distance_delta_au {
            if !distance_delta_au.is_finite() || distance_delta_au.is_sign_negative() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "comparison sample {} field `distance_delta_au` must be a finite non-negative value",
                        index + 1
                    ),
                ));
            }
        }
    }

    validate_comparison_sample_distance_channels(samples)
}

pub(crate) fn comparison_tolerance_policy_summary_details(
    comparison: &ComparisonReport,
) -> ComparisonTolerancePolicySummary {
    let entries = comparison_tolerance_policy_entries(&comparison.candidate_backend.family);
    let coverage = comparison_tolerance_policy_coverage(comparison);
    let comparison_window = TimeRange::new(
        comparison.corpus_summary.epochs.first().copied(),
        comparison.corpus_summary.epochs.last().copied(),
    );

    ComparisonTolerancePolicySummary {
        backend_family: comparison.candidate_backend.family.clone(),
        entries,
        coverage,
        comparison_body_count: comparison.body_summaries().len(),
        comparison_sample_count: comparison.summary.sample_count,
        comparison_window,
        coordinate_frames: comparison_coordinate_frames(comparison).to_vec(),
    }
}

pub(crate) fn validated_comparison_tolerance_policy_summary_for_report(
    comparison: &ComparisonReport,
) -> Result<ComparisonTolerancePolicySummary, String> {
    let summary = comparison_tolerance_policy_summary_details(comparison);
    summary.validate().map_err(|error| error.to_string())?;
    Ok(summary)
}

pub(crate) fn format_comparison_tolerance_policy_for_report(
    comparison: &ComparisonReport,
) -> String {
    let summary = comparison_tolerance_policy_summary_details(comparison);
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("comparison tolerance policy unavailable ({error})"),
    }
}

pub(crate) fn format_comparison_tolerance_limits_for_report(
    entries: &[ComparisonToleranceEntry],
) -> String {
    entries
        .iter()
        .map(format_comparison_tolerance_limit_for_report)
        .collect::<Vec<_>>()
        .join("; ")
}

pub(crate) fn format_comparison_tolerance_limit_for_report(
    entry: &ComparisonToleranceEntry,
) -> String {
    match entry.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("{} unavailable ({error})", entry.scope.label()),
    }
}

pub(crate) fn comparison_coordinate_frames(comparison: &ComparisonReport) -> &[CoordinateFrame] {
    &comparison.candidate_backend.supported_frames
}

/// Renders a release-grade comparison tolerance audit used by the CLI.
pub fn render_comparison_audit_report() -> Result<String, String> {
    let corpus = release_grade_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let comparison =
        compare_backends(&reference, &candidate, &corpus).map_err(|error| error.to_string())?;
    let (_, _, _, regression_count) = comparison_audit_totals(&comparison);
    let rendered = render_comparison_audit_report_text(&comparison);

    if regression_count == 0 {
        Ok(rendered)
    } else {
        Err(format!("comparison audit failed:\n{rendered}"))
    }
}

/// Renders the compact release-grade comparison-audit summary used by the CLI.
pub fn render_comparison_audit_summary() -> Result<String, String> {
    let corpus = release_grade_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let comparison =
        compare_backends(&reference, &candidate, &corpus).map_err(|error| error.to_string())?;

    Ok(comparison_audit_summary_for_report(&comparison))
}

pub(crate) fn comparison_audit_result_label(regression_count: usize) -> &'static str {
    if regression_count == 0 {
        "clean"
    } else {
        "regressions found"
    }
}

pub(crate) fn render_comparison_audit_report_text(report: &ComparisonReport) -> String {
    use std::fmt::Write as _;

    let (body_count, within_tolerance_body_count, outside_tolerance_body_count, regression_count) =
        comparison_audit_totals(report);
    let mut text = String::new();

    let _ = writeln!(text, "Comparison tolerance audit");
    let _ = writeln!(text, "  corpus: {}", report.corpus_name);
    let _ = writeln!(
        text,
        "  reference backend: {} ({})",
        report.reference_backend.id,
        report
            .reference_backend
            .provenance
            .validated_summary_line()
            .unwrap_or_else(|error| format!("unavailable ({error})"))
    );
    let _ = writeln!(
        text,
        "  candidate backend: {} ({})",
        report.candidate_backend.id,
        report
            .candidate_backend
            .provenance
            .validated_summary_line()
            .unwrap_or_else(|error| format!("unavailable ({error})"))
    );
    let _ = writeln!(text, "  comparison corpus");
    write_corpus_summary_text(&mut text, &report.corpus_summary);
    let _ = writeln!(text, "  bodies checked: {}", body_count);
    let _ = writeln!(
        text,
        "  within tolerance bodies: {}",
        within_tolerance_body_count
    );
    let _ = writeln!(
        text,
        "  outside tolerance bodies: {}",
        outside_tolerance_body_count
    );
    let _ = writeln!(text, "  notable regressions: {}", regression_count);
    let _ = writeln!(
        text,
        "  regression bodies: {}",
        format_regression_bodies(&report.notable_regressions())
    );
    let body_class_tolerance_posture =
        match validated_comparison_body_class_tolerance_posture_line(report) {
            Ok(line) => line,
            Err(error) => format!("body-class tolerance posture unavailable ({error})"),
        };
    let _ = writeln!(text, "  {}", body_class_tolerance_posture);
    let _ = writeln!(
        text,
        "  result: {}",
        comparison_audit_result_label(regression_count)
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Comparison summary");
    let _ = writeln!(text, "  samples: {}", report.summary.sample_count);
    let _ = writeln!(
        text,
        "  max longitude delta: {:.12}°{}",
        report.summary.max_longitude_delta_deg,
        format_summary_body(&report.summary.max_longitude_delta_body)
    );
    let _ = writeln!(
        text,
        "  max latitude delta: {:.12}°{}",
        report.summary.max_latitude_delta_deg,
        format_summary_body(&report.summary.max_latitude_delta_body)
    );
    let _ = writeln!(
        text,
        "  max distance delta: {}{}",
        report
            .summary
            .max_distance_delta_au
            .map(|value| format!("{value:.12} AU"))
            .unwrap_or_else(|| "n/a".to_string()),
        format_summary_body(&report.summary.max_distance_delta_body)
    );
    let _ = writeln!(
        text,
        "  {}",
        format_comparison_percentile_envelope_for_report(&report.samples)
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Body-class error envelopes");
    for summary in report.body_class_summaries() {
        let _ = writeln!(text, "  {}", summary.class.label());
        let _ = writeln!(text, "    samples: {}", summary.sample_count);
        let _ = writeln!(
            text,
            "    max longitude delta: {:.12}°{}",
            summary.max_longitude_delta_deg,
            summary
                .max_longitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default()
        );
        let _ = writeln!(
            text,
            "    mean longitude delta: {:.12}°",
            if summary.sample_count == 0 {
                0.0
            } else {
                summary.sum_longitude_delta_deg / summary.sample_count as f64
            }
        );
        let _ = writeln!(
            text,
            "    median longitude delta: {:.12}°",
            summary.median_longitude_delta_deg
        );
        let _ = writeln!(
            text,
            "    rms longitude delta: {:.12}°",
            if summary.sample_count == 0 {
                0.0
            } else {
                (summary.sum_longitude_delta_sq_deg / summary.sample_count as f64).sqrt()
            }
        );
        let _ = writeln!(
            text,
            "    max latitude delta: {:.12}°{}",
            summary.max_latitude_delta_deg,
            summary
                .max_latitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default()
        );
        let _ = writeln!(
            text,
            "    mean latitude delta: {:.12}°",
            if summary.sample_count == 0 {
                0.0
            } else {
                summary.sum_latitude_delta_deg / summary.sample_count as f64
            }
        );
        let _ = writeln!(
            text,
            "    median latitude delta: {:.12}°",
            summary.median_latitude_delta_deg
        );
        let _ = writeln!(
            text,
            "    rms latitude delta: {:.12}°",
            if summary.sample_count == 0 {
                0.0
            } else {
                (summary.sum_latitude_delta_sq_deg / summary.sample_count as f64).sqrt()
            }
        );
        if let Some(value) = summary.max_distance_delta_au {
            let _ = writeln!(text, "    max distance delta: {:.12} AU", value);
        }
        if summary.distance_count > 0 {
            let mean_distance = summary.sum_distance_delta_au / summary.distance_count as f64;
            let median_distance = summary.median_distance_delta_au.unwrap_or(mean_distance);
            let rms_distance =
                (summary.sum_distance_delta_sq_au / summary.distance_count as f64).sqrt();
            let _ = writeln!(text, "    mean distance delta: {:.12} AU", mean_distance);
            let _ = writeln!(
                text,
                "    median distance delta: {:.12} AU",
                median_distance
            );
            let _ = writeln!(text, "    rms distance delta: {:.12} AU", rms_distance);
        }
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Body-class tolerance posture");
    for summary in report.body_class_tolerance_summaries() {
        let _ = writeln!(text, "  {}", summary.class.label());
        let _ = writeln!(text, "    bodies: {}", summary.body_count);
        let _ = writeln!(text, "    samples: {}", summary.sample_count);
        let _ = writeln!(
            text,
            "    within tolerance bodies: {}",
            summary.within_tolerance_body_count
        );
        let _ = writeln!(
            text,
            "    outside tolerance bodies: {}",
            summary.outside_tolerance_body_count
        );
        if !summary.outside_bodies.is_empty() {
            let _ = writeln!(
                text,
                "    outside bodies: {}",
                format_bodies(&summary.outside_bodies)
            );
        }
        let _ = writeln!(
            text,
            "    mean longitude delta: {:.12}°",
            summary.mean_longitude_delta_deg()
        );
        let _ = writeln!(
            text,
            "    median longitude delta: {:.12}°",
            summary.median_longitude_delta_deg
        );
        let _ = writeln!(
            text,
            "    rms longitude delta: {:.12}°",
            summary.rms_longitude_delta_deg()
        );
        let _ = writeln!(
            text,
            "    mean latitude delta: {:.12}°",
            summary.mean_latitude_delta_deg()
        );
        let _ = writeln!(
            text,
            "    median latitude delta: {:.12}°",
            summary.median_latitude_delta_deg
        );
        let _ = writeln!(
            text,
            "    rms latitude delta: {:.12}°",
            summary.rms_latitude_delta_deg()
        );
        if let Some(value) = summary.mean_distance_delta_au() {
            let _ = writeln!(text, "    mean distance delta: {:.12} AU", value);
        }
        if let Some(value) = summary.median_distance_delta_au {
            let _ = writeln!(text, "    median distance delta: {:.12} AU", value);
        }
        if let Some(value) = summary.rms_distance_delta_au() {
            let _ = writeln!(text, "    rms distance delta: {:.12} AU", value);
        }
        if let (Some(body), Some(value)) = (
            summary.max_longitude_delta_body.as_ref(),
            summary.max_longitude_delta_deg,
        ) {
            let _ = writeln!(text, "    max longitude delta: {:.12}° ({})", value, body);
        }
        if let (Some(body), Some(value)) = (
            summary.max_latitude_delta_body.as_ref(),
            summary.max_latitude_delta_deg,
        ) {
            let _ = writeln!(text, "    max latitude delta: {:.12}° ({})", value, body);
        }
        if let (Some(body), Some(value)) = (
            summary.max_distance_delta_body.as_ref(),
            summary.max_distance_delta_au,
        ) {
            let _ = writeln!(text, "    max distance delta: {:.12} AU ({})", value, body);
        }
    }
    let _ = writeln!(text);
    let _ = writeln!(text, "Tolerance policy");
    write_tolerance_policy_text(&mut text, report);
    let _ = writeln!(text);
    let _ = writeln!(text, "Notable regressions");
    let regressions = report.notable_regressions();
    if regressions.is_empty() {
        let _ = writeln!(text, "  none");
    } else {
        for finding in regressions {
            let _ = writeln!(
                text,
                "  {}: Δlon={:.12}°, Δlat={:.12}°, Δdist={}, {}",
                finding.body,
                finding.longitude_delta_deg,
                finding.latitude_delta_deg,
                finding
                    .distance_delta_au
                    .map(|value| format!("{value:.12} AU"))
                    .unwrap_or_else(|| "n/a".to_string()),
                finding.note
            );
        }
    }

    text
}

/// Renders a benchmark report used by the CLI.
pub fn render_benchmark_report(rounds: usize) -> Result<String, EphemerisError> {
    static CACHE: OnceLock<Mutex<HashMap<usize, String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache
        .lock()
        .expect("benchmark report cache should be lockable");

    if let Some(report) = cache.get(&rounds).cloned() {
        return Ok(report);
    }

    let report = render_benchmark_report_uncached(rounds)?;
    cache.insert(rounds, report.clone());
    Ok(report)
}

pub(crate) fn render_benchmark_report_uncached(rounds: usize) -> Result<String, EphemerisError> {
    let corpus = benchmark_timing_corpus();
    let candidate = default_candidate_backend();
    let backend_report = benchmark_backend(&candidate, &corpus, rounds)?;
    let artifact_lookup_report =
        artifact::benchmark_packaged_artifact_lookup(rounds).map_err(|error| {
            EphemerisError::new(EphemerisErrorKind::MissingDataset, error.to_string())
        })?;
    let artifact_decode_report =
        artifact::benchmark_packaged_artifact_decode(rounds).map_err(|error| {
            EphemerisError::new(EphemerisErrorKind::MissingDataset, error.to_string())
        })?;
    let chart_report = benchmark_chart_backend(default_candidate_backend(), rounds)?;
    Ok(format!(
        "{}\n\n{}\n\n{}\n\n{}\n\n{}",
        benchmark_provenance_text(),
        backend_report,
        artifact_lookup_report,
        artifact_decode_report,
        chart_report
    ))
}

/// Renders a compact benchmark matrix summary used by the CLI.
pub fn render_benchmark_matrix_summary(rounds: usize) -> Result<String, EphemerisError> {
    let report = build_validation_report(rounds)?;
    Ok(render_benchmark_matrix_summary_text(&report))
}

pub(crate) fn report_summary_payload(summary: String, prefix: &str) -> String {
    summary
        .strip_prefix(prefix)
        .unwrap_or(summary.as_str())
        .to_string()
}

pub(crate) fn render_benchmark_matrix_summary_text(report: &ValidationReport) -> String {
    use std::fmt::Write as _;

    let mut text = String::from("Benchmark matrix summary\n");
    let _ = writeln!(text, "{}", benchmark_provenance_text());
    let _ = writeln!(text, "Benchmark corpora");
    let _ = writeln!(
        text,
        "  comparison corpus: {}",
        report.comparison_corpus.summary_line()
    );
    let _ = writeln!(
        text,
        "  benchmark corpus: {}",
        report.benchmark_corpus.summary_line()
    );
    let _ = writeln!(
        text,
        "  packaged-data benchmark corpus: {}",
        report.packaged_benchmark_corpus.summary_line()
    );
    let _ = writeln!(
        text,
        "  chart benchmark corpus: {}",
        report.chart_benchmark_corpus.summary_line()
    );
    let _ = writeln!(text);
    let _ = writeln!(text, "Benchmark rows");
    let _ = writeln!(
        text,
        "  reference benchmark: {}",
        report.reference_benchmark.summary_line()
    );
    let _ = writeln!(
        text,
        "  candidate benchmark: {}",
        report.candidate_benchmark.summary_line()
    );
    let _ = writeln!(
        text,
        "  packaged-data benchmark: {}",
        report.packaged_benchmark.summary_line()
    );
    let _ = writeln!(
        text,
        "  chart benchmark: {}",
        report.chart_benchmark.summary_line()
    );
    let _ = writeln!(
        text,
        "  artifact decode benchmark: {}",
        report.artifact_decode_benchmark.summary_line()
    );
    let _ = writeln!(
        text,
        "  packaged-artifact size: {} bytes",
        report.artifact_decode_benchmark.encoded_bytes
    );
    let fit_envelope_summary = packaged_artifact_fit_envelope_summary_for_report();
    let fit_sample_classes_summary = packaged_artifact_fit_sample_classes_summary_for_report();
    let fit_outlier_summary = packaged_artifact_fit_outlier_summary_for_report();
    let fit_thresholds_summary = packaged_artifact_fit_threshold_summary_for_report();
    let target_threshold_summary =
        validated_packaged_artifact_target_threshold_summary_for_report();
    let target_threshold_state_summary =
        validated_packaged_artifact_target_threshold_state_for_report();
    let target_threshold_scope_envelopes_summary =
        validated_packaged_artifact_target_threshold_scope_envelopes_summary_for_report();
    let fit_margin_summary = report_summary_payload(
        packaged_artifact_fit_margin_summary_for_report(),
        "fit margins: ",
    );
    let fit_threshold_violation_count_summary = report_summary_payload(
        packaged_artifact_fit_threshold_violation_count_for_report(),
        "fit threshold violations: ",
    );
    let fit_threshold_violation_summary = report_summary_payload(
        packaged_artifact_fit_threshold_violation_summary_for_report(),
        "fit threshold violations: ",
    );
    let fit_envelope = fit_envelope_summary
        .strip_prefix("fit envelope: ")
        .unwrap_or(&fit_envelope_summary);
    let fit_sample_classes = fit_sample_classes_summary
        .strip_prefix("fit sample classes: ")
        .unwrap_or(&fit_sample_classes_summary);
    let fit_outliers = fit_outlier_summary
        .strip_prefix("fit outliers: ")
        .unwrap_or(&fit_outlier_summary);
    let fit_thresholds = fit_thresholds_summary
        .strip_prefix("fit thresholds: ")
        .unwrap_or(&fit_thresholds_summary);
    let target_threshold_scope_envelopes = target_threshold_scope_envelopes_summary
        .strip_prefix("scope envelopes: ")
        .unwrap_or(&target_threshold_scope_envelopes_summary);

    let _ = writeln!(text);
    let _ = writeln!(text, "Packaged-artifact fit posture");
    let _ = writeln!(text, "  fit envelope: {}", fit_envelope);
    let _ = writeln!(text, "  fit margins: {}", fit_margin_summary);
    let _ = writeln!(
        text,
        "  fit threshold violation count: {}",
        fit_threshold_violation_count_summary
    );
    let _ = writeln!(
        text,
        "  fit threshold violations: {}",
        fit_threshold_violation_summary
    );
    let _ = writeln!(text, "  fit sample classes: {}", fit_sample_classes);
    let _ = writeln!(text, "  fit outliers: {}", fit_outliers);
    let _ = writeln!(text, "  fit thresholds: {}", fit_thresholds);
    let _ = writeln!(text, "  target thresholds: {}", target_threshold_summary);
    let _ = writeln!(
        text,
        "  target-threshold state: {}",
        target_threshold_state_summary
    );
    let _ = writeln!(
        text,
        "  target-threshold scope envelopes: {}",
        target_threshold_scope_envelopes
    );
    text
}

pub(crate) fn vsop87_canonical_body_evidence(
) -> Option<Vec<pleiades_vsop87::Vsop87CanonicalBodyEvidence>> {
    pleiades_vsop87::canonical_epoch_body_evidence()
}

pub(crate) fn format_vsop87_canonical_evidence_summary() -> String {
    canonical_epoch_evidence_summary_for_report()
}

pub(crate) fn format_vsop87_equatorial_evidence_summary() -> String {
    canonical_epoch_equatorial_evidence_summary_for_report()
}

pub(crate) fn format_vsop87_j2000_batch_summary() -> String {
    canonical_j2000_batch_parity_summary_for_report()
}

pub(crate) fn format_vsop87_supported_body_j2000_ecliptic_batch_summary() -> String {
    supported_body_j2000_ecliptic_batch_parity_summary_for_report()
}

pub(crate) fn format_vsop87_supported_body_j2000_equatorial_batch_summary() -> String {
    supported_body_j2000_equatorial_batch_parity_summary_for_report()
}

pub(crate) fn format_vsop87_supported_body_j1900_ecliptic_batch_summary() -> String {
    supported_body_j1900_ecliptic_batch_parity_summary_for_report()
}

pub(crate) fn format_vsop87_supported_body_j1900_equatorial_batch_summary() -> String {
    supported_body_j1900_equatorial_batch_parity_summary_for_report()
}

pub(crate) fn format_vsop87_mixed_batch_summary() -> String {
    canonical_mixed_time_scale_batch_parity_summary_for_report()
}

pub(crate) fn format_vsop87_j1900_batch_summary() -> String {
    canonical_j1900_batch_parity_summary_for_report()
}

pub(crate) fn format_vsop87_body_evidence_summary() -> String {
    source_body_evidence_summary_for_report()
}

pub(crate) fn format_vsop87_source_body_class_evidence_summary() -> String {
    source_body_class_evidence_summary_for_report()
}

pub(crate) fn format_vsop87_equatorial_body_class_evidence_summary() -> String {
    canonical_epoch_equatorial_body_class_evidence_summary_for_report()
}

pub(crate) fn format_vsop87_canonical_outlier_note_summary() -> String {
    canonical_epoch_outlier_note_for_report()
}

pub(crate) fn format_validated_vsop87_source_documentation_summary_for_report(
    summary: &pleiades_vsop87::Vsop87SourceDocumentationSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("VSOP87 source documentation: unavailable ({error})"),
    }
}

pub(crate) fn format_validated_vsop87_source_documentation_health_summary_for_report(
    summary: &pleiades_vsop87::Vsop87SourceDocumentationHealthSummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("VSOP87 source documentation health: unavailable ({error})"),
    }
}

pub(crate) fn format_vsop87_source_documentation_summary() -> String {
    format_validated_vsop87_source_documentation_summary_for_report(&source_documentation_summary())
}

pub(crate) fn format_vsop87_source_documentation_health_summary() -> String {
    format_validated_vsop87_source_documentation_health_summary_for_report(
        &source_documentation_health_summary(),
    )
}

pub(crate) fn format_vsop87_frame_treatment_summary() -> String {
    frame_treatment_summary_for_report()
}

pub(crate) fn format_jpl_frame_treatment_summary() -> String {
    jpl_frame_treatment_summary_for_report()
}

/// Compact validation evidence for the shared mean-obliquity frame round-trip samples.
#[derive(Clone, Debug, PartialEq)]
pub struct MeanObliquityFrameRoundTripSummary {
    pub(crate) sample_count: usize,
    pub(crate) max_longitude_delta_deg: f64,
    pub(crate) max_latitude_delta_deg: f64,
    pub(crate) max_distance_delta_au: f64,
    pub(crate) mean_longitude_delta_deg: f64,
    pub(crate) mean_latitude_delta_deg: f64,
    pub(crate) mean_distance_delta_au: f64,
    pub(crate) percentile_longitude_delta_deg: f64,
    pub(crate) percentile_latitude_delta_deg: f64,
    pub(crate) percentile_distance_delta_au: f64,
}

impl MeanObliquityFrameRoundTripSummary {
    /// Validates the stored round-trip envelope.
    pub fn validate(&self) -> Result<(), String> {
        if self.sample_count == 0 {
            return Err("mean-obliquity frame round-trip summary has no samples".to_string());
        }

        for (label, value) in [
            ("max_longitude_delta_deg", self.max_longitude_delta_deg),
            ("max_latitude_delta_deg", self.max_latitude_delta_deg),
            ("max_distance_delta_au", self.max_distance_delta_au),
            ("mean_longitude_delta_deg", self.mean_longitude_delta_deg),
            ("mean_latitude_delta_deg", self.mean_latitude_delta_deg),
            ("mean_distance_delta_au", self.mean_distance_delta_au),
            (
                "percentile_longitude_delta_deg",
                self.percentile_longitude_delta_deg,
            ),
            (
                "percentile_latitude_delta_deg",
                self.percentile_latitude_delta_deg,
            ),
            (
                "percentile_distance_delta_au",
                self.percentile_distance_delta_au,
            ),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(format!(
                    "mean-obliquity frame round-trip summary field `{label}` must be a finite non-negative value"
                ));
            }
        }

        let expected = expected_mean_obliquity_frame_round_trip_summary()?;
        if *self != expected {
            return Err(
                "mean-obliquity frame round-trip summary drifted from the canonical sample set"
                    .to_string(),
            );
        }

        Ok(())
    }

    pub(crate) fn summary_line(&self) -> String {
        format!(
            "{} samples, max |Δlon|={:.12}°, mean |Δlon|={:.12}°, p95 |Δlon|={:.12}°, max |Δlat|={:.12}°, mean |Δlat|={:.12}°, p95 |Δlat|={:.12}°, max |Δdist|={:.12} AU, mean |Δdist|={:.12} AU, p95 |Δdist|={:.12} AU",
            self.sample_count,
            self.max_longitude_delta_deg,
            self.mean_longitude_delta_deg,
            self.percentile_longitude_delta_deg,
            self.max_latitude_delta_deg,
            self.mean_latitude_delta_deg,
            self.percentile_latitude_delta_deg,
            self.max_distance_delta_au,
            self.mean_distance_delta_au,
            self.percentile_distance_delta_au,
        )
    }

    /// Returns the compact round-trip summary line after validating the canonical sample set.
    pub fn validated_summary_line(&self) -> Result<String, String> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for MeanObliquityFrameRoundTripSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the canonical sample corpus used to validate the shared mean-obliquity frame round-trip envelope.
///
/// Downstream tooling can reuse this exact input set instead of reconstructing it from report text.
/// The corpus intentionally covers a near-polar wraparound case so the report evidence exercises the
/// same precision edge that the frame regression tests pin.
pub fn mean_obliquity_frame_round_trip_sample_corpus() -> [(EclipticCoordinates, Instant); 7] {
    [
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(123.45),
                pleiades_core::Latitude::from_degrees(-6.75),
                Some(0.123),
            ),
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(90.0),
                pleiades_core::Latitude::from_degrees(0.0),
                Some(1.0),
            ),
            Instant::new(JulianDay::from_days(2_459_000.5), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(27.5),
                pleiades_core::Latitude::from_degrees(-33.25),
                Some(2.5),
            ),
            Instant::new(JulianDay::from_days(2_415_020.5), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(315.0),
                pleiades_core::Latitude::from_degrees(18.0),
                Some(4.25),
            ),
            Instant::new(JulianDay::from_days(2_440_587.5), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(5.0),
                pleiades_core::Latitude::from_degrees(66.0),
                Some(0.75),
            ),
            Instant::new(JulianDay::from_days(2_500_000.5), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(359.875),
                pleiades_core::Latitude::from_degrees(89.25),
                Some(0.5),
            ),
            Instant::new(JulianDay::from_days(2_450_000.5), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(180.0),
                pleiades_core::Latitude::from_degrees(-89.25),
                Some(0.5),
            ),
            Instant::new(JulianDay::from_days(2_450_000.5), TimeScale::Tt),
        ),
    ]
}

pub(crate) fn validate_mean_obliquity_frame_round_trip_sample_corpus(
    samples: &[(EclipticCoordinates, Instant)],
) -> Result<(), String> {
    if samples.len() != 7 {
        return Err(format!(
            "mean-obliquity frame round-trip sample corpus must contain 7 samples, found {}",
            samples.len()
        ));
    }

    if !samples
        .iter()
        .any(|(coordinates, _)| coordinates.latitude.degrees() > 80.0)
    {
        return Err(
            "mean-obliquity frame round-trip sample corpus must include a northern polar sample"
                .to_string(),
        );
    }

    if !samples
        .iter()
        .any(|(coordinates, _)| coordinates.latitude.degrees() < -80.0)
    {
        return Err(
            "mean-obliquity frame round-trip sample corpus must include a southern polar sample"
                .to_string(),
        );
    }

    if !samples
        .iter()
        .any(|(coordinates, _)| coordinates.longitude.degrees() > 350.0)
    {
        return Err(
            "mean-obliquity frame round-trip sample corpus must include a wraparound longitude sample"
                .to_string(),
        );
    }

    if !samples
        .iter()
        .any(|(coordinates, _)| coordinates.latitude.degrees().abs() < 1e-12)
    {
        return Err(
            "mean-obliquity frame round-trip sample corpus must include an equatorial sample"
                .to_string(),
        );
    }

    Ok(())
}

pub(crate) fn mean_obliquity_frame_round_trip_summary_from_samples(
    samples: &[(EclipticCoordinates, Instant)],
) -> Result<MeanObliquityFrameRoundTripSummary, String> {
    validate_mean_obliquity_frame_round_trip_sample_corpus(samples)?;

    let mut sample_count = 0usize;
    let mut max_longitude_delta_deg: f64 = 0.0;
    let mut max_latitude_delta_deg: f64 = 0.0;
    let mut max_distance_delta_au: f64 = 0.0;
    let mut longitude_deltas = Vec::with_capacity(samples.len());
    let mut latitude_deltas = Vec::with_capacity(samples.len());
    let mut distance_deltas = Vec::with_capacity(samples.len());

    for (ecliptic, instant) in samples.iter().copied() {
        let obliquity = instant.mean_obliquity();
        let round_trip = ecliptic.to_equatorial(obliquity).to_ecliptic(obliquity);
        let longitude_delta_deg =
            (round_trip.longitude.degrees() - ecliptic.longitude.degrees()).abs();
        let latitude_delta_deg =
            (round_trip.latitude.degrees() - ecliptic.latitude.degrees()).abs();
        let distance_delta_au = (round_trip.distance_au.unwrap_or_default()
            - ecliptic.distance_au.unwrap_or_default())
        .abs();

        if !longitude_delta_deg.is_finite()
            || !latitude_delta_deg.is_finite()
            || !distance_delta_au.is_finite()
        {
            return Err("non-finite round-trip delta".to_string());
        }

        max_longitude_delta_deg = max_longitude_delta_deg.max(longitude_delta_deg);
        max_latitude_delta_deg = max_latitude_delta_deg.max(latitude_delta_deg);
        max_distance_delta_au = max_distance_delta_au.max(distance_delta_au);
        longitude_deltas.push(longitude_delta_deg);
        latitude_deltas.push(latitude_delta_deg);
        distance_deltas.push(distance_delta_au);
        sample_count += 1;
    }

    Ok(MeanObliquityFrameRoundTripSummary {
        sample_count,
        max_longitude_delta_deg,
        max_latitude_delta_deg,
        max_distance_delta_au,
        mean_longitude_delta_deg: arithmetic_mean(&longitude_deltas),
        mean_latitude_delta_deg: arithmetic_mean(&latitude_deltas),
        mean_distance_delta_au: arithmetic_mean(&distance_deltas),
        percentile_longitude_delta_deg: percentile_linear_interpolation(&longitude_deltas, 0.95),
        percentile_latitude_delta_deg: percentile_linear_interpolation(&latitude_deltas, 0.95),
        percentile_distance_delta_au: percentile_linear_interpolation(&distance_deltas, 0.95),
    })
}

pub(crate) fn expected_mean_obliquity_frame_round_trip_summary(
) -> Result<MeanObliquityFrameRoundTripSummary, String> {
    mean_obliquity_frame_round_trip_summary_from_samples(
        &mean_obliquity_frame_round_trip_sample_corpus(),
    )
}

pub(crate) fn arithmetic_mean(values: &[f64]) -> f64 {
    values.iter().copied().sum::<f64>() / values.len() as f64
}

pub(crate) fn percentile_linear_interpolation(values: &[f64], percentile: f64) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    if sorted.len() == 1 {
        return sorted[0];
    }

    let clamped = percentile.clamp(0.0, 1.0);
    let position = clamped * (sorted.len() - 1) as f64;
    let lower_index = position.floor() as usize;
    let upper_index = position.ceil() as usize;

    if lower_index == upper_index {
        sorted[lower_index]
    } else {
        let lower_value = sorted[lower_index];
        let upper_value = sorted[upper_index];
        let fraction = position - lower_index as f64;
        lower_value + (upper_value - lower_value) * fraction
    }
}

/// Computes the shared mean-obliquity frame round-trip validation summary.
pub fn mean_obliquity_frame_round_trip_summary(
) -> Result<MeanObliquityFrameRoundTripSummary, String> {
    let summary = mean_obliquity_frame_round_trip_summary_from_samples(
        &mean_obliquity_frame_round_trip_sample_corpus(),
    )?;
    summary.validate()?;
    Ok(summary)
}

pub(crate) fn format_mean_obliquity_frame_round_trip_summary_for_report(
    summary: &MeanObliquityFrameRoundTripSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("mean-obliquity frame round-trip unavailable ({error})"),
    }
}

pub(crate) fn mean_obliquity_frame_round_trip_summary_for_report() -> String {
    match mean_obliquity_frame_round_trip_summary() {
        Ok(summary) => format_mean_obliquity_frame_round_trip_summary_for_report(&summary),
        Err(error) => format!("mean-obliquity frame round-trip unavailable ({error})"),
    }
}

pub(crate) fn format_time_scale_policy_summary_for_report(
    summary: &pleiades_backend::TimeScalePolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line.to_string(),
        Err(error) => format!("time-scale policy unavailable ({error})"),
    }
}

pub(crate) fn format_delta_t_policy_summary_for_report(
    summary: &pleiades_backend::DeltaTPolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line.to_string(),
        Err(error) => format!("delta T policy unavailable ({error})"),
    }
}

pub(crate) fn format_observer_policy_summary_for_report(
    summary: &pleiades_backend::ObserverPolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line.to_string(),
        Err(error) => format!("observer policy unavailable ({error})"),
    }
}

pub(crate) fn format_apparentness_policy_summary_for_report(
    summary: &pleiades_backend::ApparentnessPolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line.to_string(),
        Err(error) => format!("apparentness policy unavailable ({error})"),
    }
}

pub(crate) fn format_request_policy_summary_for_report(
    summary: &pleiades_backend::RequestPolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("request policy unavailable ({error})"),
    }
}

pub(crate) fn validated_request_policy_summary_for_report(
) -> Result<pleiades_backend::RequestPolicySummary, String> {
    let summary = request_policy_summary_for_report();
    summary.validate().map_err(|error| error.to_string())?;
    Ok(summary)
}

pub(crate) fn validated_production_generation_body_class_coverage_summary_for_report() -> String {
    match validated_production_generation_snapshot_body_class_coverage_summary_for_report() {
        Ok(summary) => summary,
        Err(error) => format!("Production generation body-class coverage unavailable ({error})"),
    }
}

pub(crate) fn format_request_semantics_summary_for_report(
    time_scale_policy: &pleiades_backend::TimeScalePolicySummary,
) -> String {
    use std::fmt::Write as _;

    let mut text = String::new();
    let _ = writeln!(
        text,
        "Time-scale policy: {}",
        format_time_scale_policy_summary_for_report(time_scale_policy)
    );

    let utc_convenience_policy =
        pleiades_backend::validated_utc_convenience_policy_summary_for_report();
    let _ = writeln!(text, "UTC convenience policy: {}", utc_convenience_policy);

    let delta_t_policy = delta_t_policy_summary_for_report();
    let _ = writeln!(
        text,
        "Delta T policy: {}",
        format_delta_t_policy_summary_for_report(&delta_t_policy)
    );

    let native_sidereal_policy =
        pleiades_backend::validated_native_sidereal_policy_summary_for_report();
    let _ = writeln!(text, "Native sidereal policy: {}", native_sidereal_policy);

    let request_policy = match validated_request_policy_summary_for_report() {
        Ok(summary) => summary,
        Err(error) => {
            let _ = writeln!(text, "Observer policy unavailable ({error})");
            let _ = writeln!(text, "Apparentness policy unavailable ({error})");
            let _ = writeln!(text, "Request policy unavailable ({error})");
            return text;
        }
    };

    let observer_policy = pleiades_backend::observer_policy_summary_for_report();
    let apparentness_policy = pleiades_backend::apparentness_policy_summary_for_report();
    let _ = writeln!(
        text,
        "Observer policy: {}",
        format_observer_policy_summary_for_report(&observer_policy)
    );
    let _ = writeln!(
        text,
        "Apparentness policy: {}",
        format_apparentness_policy_summary_for_report(&apparentness_policy)
    );
    let _ = writeln!(
        text,
        "Request policy: {}",
        format_request_policy_summary_for_report(&request_policy)
    );
    text
}

pub(crate) fn render_time_scale_policy_summary_text() -> String {
    match time_scale_policy_summary_for_report().validated_summary_line() {
        Ok(summary) => format!(
            "Time-scale policy summary\nTime-scale policy: {}\n",
            summary
        ),
        Err(error) => {
            format!("Time-scale policy summary\nTime-scale policy unavailable ({error})\n")
        }
    }
}

pub(crate) fn render_delta_t_policy_summary_text() -> String {
    match delta_t_policy_summary_for_report().validated_summary_line() {
        Ok(summary) => format!("Delta T policy summary\nDelta T policy: {}\n", summary),
        Err(error) => format!("Delta T policy summary\nDelta T policy unavailable ({error})\n"),
    }
}

pub(crate) fn render_zodiac_policy_summary_text() -> String {
    format!(
        "Zodiac policy summary\nZodiac policy: {}\n",
        pleiades_backend::validated_zodiac_policy_summary_for_report()
    )
}

pub(crate) fn render_utc_convenience_policy_summary_text() -> String {
    format!(
        "UTC convenience policy summary\nUTC convenience policy: {}\n",
        pleiades_backend::validated_utc_convenience_policy_summary_for_report()
    )
}

pub(crate) fn render_observer_policy_summary_text() -> String {
    match pleiades_backend::observer_policy_summary_for_report().validated_summary_line() {
        Ok(summary) => format!("Observer policy summary\nObserver policy: {}\n", summary),
        Err(error) => format!("Observer policy summary\nObserver policy unavailable ({error})\n"),
    }
}

pub(crate) fn render_apparentness_policy_summary_text() -> String {
    match pleiades_backend::apparentness_policy_summary_for_report().validated_summary_line() {
        Ok(summary) => format!(
            "Apparentness policy summary\nApparentness policy: {}\n",
            summary
        ),
        Err(error) => {
            format!("Apparentness policy summary\nApparentness policy unavailable ({error})\n")
        }
    }
}

pub(crate) fn render_native_sidereal_policy_summary_text() -> String {
    format!(
        "Native sidereal policy summary\nNative sidereal policy: {}\n",
        pleiades_backend::validated_native_sidereal_policy_summary_for_report()
    )
}

pub(crate) fn render_interpolation_posture_summary_text() -> String {
    match jpl_interpolation_posture_summary() {
        Some(summary) => {
            match summary.validated_summary_line() {
                Ok(summary) => format!(
                    "Interpolation posture summary\nInterpolation posture: {}\n",
                    summary
                ),
                Err(error) => {
                    format!("Interpolation posture summary\nInterpolation posture unavailable ({error})\n")
                }
            }
        }
        None => "Interpolation posture summary\nInterpolation posture unavailable\n".to_string(),
    }
}

pub(crate) fn render_interpolation_quality_summary_text() -> String {
    format!(
        "Interpolation quality summary\n{}\n",
        format_jpl_interpolation_quality_summary_for_report()
    )
}

pub(crate) fn render_comparison_snapshot_summary_text() -> String {
    format!(
        "Comparison snapshot summary\n{}\n",
        comparison_snapshot_summary_for_report()
    )
}

pub(crate) fn comparison_corpus_release_guard_summary() -> &'static str {
    "Pluto excluded from tolerance evidence"
}

pub(crate) fn validated_comparison_corpus_release_guard_summary_for_report(
) -> Result<&'static str, String> {
    const EXPECTED: &str = "Pluto excluded from tolerance evidence";
    let summary = comparison_corpus_release_guard_summary();

    if summary == EXPECTED {
        Ok(summary)
    } else {
        Err(format!(
            "comparison corpus release-grade guard mismatch: expected {EXPECTED}, found {summary}"
        ))
    }
}

pub(crate) fn render_comparison_corpus_summary_text() -> String {
    use std::fmt::Write as _;

    let corpus = release_grade_corpus();
    let summary = corpus.summary();
    let mut text = String::from("Comparison corpus summary\n");
    write_corpus_summary_text(&mut text, &summary);
    let release_grade_guard = match validated_comparison_corpus_release_guard_summary_for_report() {
        Ok(guard) => guard,
        Err(error) => return format!("Comparison corpus summary unavailable ({error})"),
    };
    let _ = writeln!(text, "  release-grade guard: {release_grade_guard}");
    text.push('\n');
    text
}

pub(crate) fn ensure_comparison_corpus_summary_matches_current_rendering(
    comparison_corpus_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if comparison_corpus_summary_text != render_comparison_corpus_summary_text() {
        return Err(ReleaseBundleError::Verification(
            "comparison corpus summary no longer matches the current comparison-corpus posture"
                .to_string(),
        ));
    }

    Ok(())
}

pub(crate) fn required_summary_payload(
    summary: String,
    prefix: &str,
    field: &'static str,
) -> Result<String, String> {
    summary
        .strip_prefix(prefix)
        .map(str::to_string)
        .ok_or_else(|| {
            format!("source corpus summary field `{field}` is out of sync with the current posture")
        })
}

pub(crate) fn required_labelled_summary_payload(
    summary: String,
    prefix: &str,
    field: &'static str,
) -> Result<String, String> {
    let payload = required_summary_payload(summary, prefix, field)?;
    if payload.starts_with(prefix) {
        return Err(format!(
            "source corpus summary field `{field}` is out of sync with the current posture"
        ));
    }

    Ok(payload)
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SourceCorpusSummary {
    pub(crate) comparison_corpus_release_grade_guard: String,
    pub(crate) jpl_source_corpus_contract: String,
    pub(crate) jpl_evidence_classification: String,
    pub(crate) jpl_provenance_only: String,
    pub(crate) lunar_source_window: String,
    pub(crate) shared_schema: String,
    pub(crate) generation_command: String,
    pub(crate) production_generation_source: String,
    pub(crate) production_generation_source_revision: String,
    pub(crate) production_generation_coverage: String,
    pub(crate) production_generation_source_windows: String,
    pub(crate) production_generation_body_class_coverage: String,
    pub(crate) production_generation_date_range: String,
    pub(crate) production_generation_quarter_day_boundary_samples: String,
    pub(crate) coverage_posture: String,
    pub(crate) production_generation_boundary_window: String,
    pub(crate) production_generation_boundary_source: String,
    pub(crate) production_generation_boundary_request_corpus: String,
    pub(crate) production_generation_boundary_request_corpus_equatorial: String,
    pub(crate) reference_snapshot_sparse_boundary: String,
    pub(crate) reference_snapshot_exact_j2000_evidence: String,
    pub(crate) reference_snapshot_exact_j2000_body_class_coverage: String,
    pub(crate) reference_snapshot_equatorial_parity: String,
    pub(crate) reference_snapshot_body_class_coverage: String,
    pub(crate) reference_snapshot_manifest: String,
    pub(crate) comparison_snapshot_manifest: String,
    pub(crate) independent_holdout_body_class_coverage: String,
    pub(crate) independent_holdout_source_window: String,
    pub(crate) pluto_fallback: String,
    pub(crate) release_grade_body_claims: String,
    pub(crate) body_date_channel_claims: String,
    pub(crate) phase2_corpus_alignment: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SourceCorpusSummaryValidationError {
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for SourceCorpusSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the source corpus summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for SourceCorpusSummaryValidationError {}

impl SourceCorpusSummary {
    pub(crate) fn summary_line(&self) -> String {
        format!(
            "comparison corpus release-grade guard: {}; JPL source corpus contract: {}; evidence classification={}; provenance-only={}; lunar source windows={}; shared schema={}; generation command={}; production generation source={}; production generation source revision={}; production generation coverage={}; production generation source windows={}; production generation body-class coverage={}; production generation date range={}; production generation quarter-day boundary samples={}; coverage posture={}; production generation boundary window={}; production generation boundary source={}; production generation boundary request corpus={}; production generation boundary request corpus equatorial={}; reference snapshot sparse boundary={}; reference snapshot exact J2000 evidence={}; reference snapshot exact J2000 body-class coverage={}; reference snapshot equatorial parity={}; reference snapshot body-class coverage={}; reference snapshot manifest={}; comparison snapshot manifest={}; independent-holdout body-class coverage={}; independent-holdout source window={}; pluto fallback={}; release-grade body claims={}; body-date-channel claims={}; phase-2 corpus alignment: {}",
            self.comparison_corpus_release_grade_guard,
            self.jpl_source_corpus_contract,
            self.jpl_evidence_classification,
            self.jpl_provenance_only,
            self.lunar_source_window,
            self.shared_schema,
            self.generation_command,
            self.production_generation_source,
            self.production_generation_source_revision,
            self.production_generation_coverage,
            self.production_generation_source_windows,
            self.production_generation_body_class_coverage,
            self.production_generation_date_range,
            self.production_generation_quarter_day_boundary_samples,
            self.coverage_posture,
            self.production_generation_boundary_window,
            self.production_generation_boundary_source,
            self.production_generation_boundary_request_corpus,
            self.production_generation_boundary_request_corpus_equatorial,
            self.reference_snapshot_sparse_boundary,
            self.reference_snapshot_exact_j2000_evidence,
            self.reference_snapshot_exact_j2000_body_class_coverage,
            self.reference_snapshot_equatorial_parity,
            self.reference_snapshot_body_class_coverage,
            self.reference_snapshot_manifest,
            self.comparison_snapshot_manifest,
            self.independent_holdout_body_class_coverage,
            self.independent_holdout_source_window,
            self.pluto_fallback,
            self.release_grade_body_claims,
            self.body_date_channel_claims,
            self.phase2_corpus_alignment,
        )
    }

    pub(crate) fn validate(&self) -> Result<(), SourceCorpusSummaryValidationError> {
        let expected = source_corpus_summary_details().ok_or(
            SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "source_corpus_summary",
            },
        )?;

        if self.comparison_corpus_release_grade_guard
            != expected.comparison_corpus_release_grade_guard
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "comparison_corpus_release_grade_guard",
            });
        }
        if self.jpl_source_corpus_contract != expected.jpl_source_corpus_contract {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "jpl_source_corpus_contract",
            });
        }
        if self.jpl_evidence_classification != expected.jpl_evidence_classification {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "jpl_evidence_classification",
            });
        }
        if self.jpl_provenance_only != expected.jpl_provenance_only {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "jpl_provenance_only",
            });
        }
        if self.lunar_source_window != expected.lunar_source_window {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "lunar_source_window",
            });
        }
        if self.shared_schema != expected.shared_schema {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "shared_schema",
            });
        }
        if self.generation_command != expected.generation_command {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "generation_command",
            });
        }
        if self.production_generation_source != expected.production_generation_source {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_source",
            });
        }
        if self.production_generation_source_revision
            != expected.production_generation_source_revision
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_source_revision",
            });
        }
        if self.production_generation_coverage != expected.production_generation_coverage {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_coverage",
            });
        }
        if self.production_generation_source_windows
            != expected.production_generation_source_windows
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_source_windows",
            });
        }
        if self.production_generation_body_class_coverage
            != expected.production_generation_body_class_coverage
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_body_class_coverage",
            });
        }
        if self.production_generation_date_range != expected.production_generation_date_range {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_date_range",
            });
        }
        if self.production_generation_quarter_day_boundary_samples
            != expected.production_generation_quarter_day_boundary_samples
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_quarter_day_boundary_samples",
            });
        }
        if self.coverage_posture != expected.coverage_posture {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "coverage_posture",
            });
        }
        if self.production_generation_boundary_window
            != expected.production_generation_boundary_window
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_boundary_window",
            });
        }
        if self.production_generation_boundary_source
            != expected.production_generation_boundary_source
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_boundary_source",
            });
        }
        if self.production_generation_boundary_request_corpus
            != expected.production_generation_boundary_request_corpus
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_boundary_request_corpus",
            });
        }
        if self.production_generation_boundary_request_corpus_equatorial
            != expected.production_generation_boundary_request_corpus_equatorial
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_boundary_request_corpus_equatorial",
            });
        }
        if self.reference_snapshot_sparse_boundary != expected.reference_snapshot_sparse_boundary {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "reference_snapshot_sparse_boundary",
            });
        }
        if self.reference_snapshot_exact_j2000_evidence
            != expected.reference_snapshot_exact_j2000_evidence
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "reference_snapshot_exact_j2000_evidence",
            });
        }
        if self.reference_snapshot_exact_j2000_body_class_coverage
            != expected.reference_snapshot_exact_j2000_body_class_coverage
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "reference_snapshot_exact_j2000_body_class_coverage",
            });
        }
        if self.reference_snapshot_equatorial_parity
            != expected.reference_snapshot_equatorial_parity
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "reference_snapshot_equatorial_parity",
            });
        }
        if self.reference_snapshot_body_class_coverage
            != expected.reference_snapshot_body_class_coverage
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "reference_snapshot_body_class_coverage",
            });
        }
        if self.reference_snapshot_manifest != expected.reference_snapshot_manifest {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "reference_snapshot_manifest",
            });
        }
        if self.independent_holdout_body_class_coverage
            != expected.independent_holdout_body_class_coverage
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "independent_holdout_body_class_coverage",
            });
        }
        if self.independent_holdout_source_window != expected.independent_holdout_source_window {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "independent_holdout_source_window",
            });
        }
        if self.pluto_fallback != expected.pluto_fallback {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "pluto_fallback",
            });
        }
        if self.release_grade_body_claims != expected.release_grade_body_claims {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "release_grade_body_claims",
            });
        }
        if self.body_date_channel_claims != expected.body_date_channel_claims {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "body_date_channel_claims",
            });
        }
        if self.phase2_corpus_alignment != expected.phase2_corpus_alignment {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "phase2_corpus_alignment",
            });
        }

        Ok(())
    }

    pub(crate) fn validated_summary_line(
        &self,
    ) -> Result<String, SourceCorpusSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

pub(crate) fn source_corpus_summary_details() -> Option<SourceCorpusSummary> {
    let comparison_corpus_release_grade_guard =
        validated_comparison_corpus_release_guard_summary_for_report()
            .ok()?
            .to_string();
    let jpl_source_corpus_contract = required_labelled_summary_payload(
        jpl_source_corpus_contract_summary_for_report(),
        "JPL source corpus contract: ",
        "JPL source corpus contract",
    )
    .ok()?;
    let jpl_evidence_classification = required_labelled_summary_payload(
        jpl_snapshot_evidence_classification_summary_for_report(),
        "JPL evidence classification: ",
        "JPL evidence classification",
    )
    .ok()?;
    let jpl_provenance_only = required_labelled_summary_payload(
        jpl_provenance_only_summary_for_report(),
        "JPL provenance-only evidence: ",
        "JPL provenance-only evidence",
    )
    .ok()?;
    let release_grade_body_claims = validated_release_body_claims_summary_line_for_report()
        .ok()?
        .to_string();
    let lunar_source_window = required_summary_payload(
        lunar_source_window_summary_for_report(),
        "lunar source windows: ",
        "lunar source window",
    )
    .ok()?;
    let reference_snapshot_sparse_boundary = required_summary_payload(
        reference_snapshot_sparse_boundary_summary_for_report(),
        "Reference snapshot boundary day: ",
        "reference snapshot sparse boundary",
    )
    .ok()?;
    let reference_snapshot_exact_j2000_evidence = required_summary_payload(
        reference_snapshot_exact_j2000_evidence_summary_for_report(),
        "Reference snapshot exact J2000 evidence: ",
        "reference snapshot exact J2000 evidence",
    )
    .ok()?;
    let reference_snapshot_exact_j2000_body_class_coverage = required_summary_payload(
        pleiades_jpl::reference_snapshot_exact_j2000_body_class_coverage_summary_for_report(),
        "Reference snapshot exact J2000 body-class coverage: ",
        "reference snapshot exact J2000 body-class coverage",
    )
    .ok()?;
    let reference_snapshot_equatorial_parity = required_summary_payload(
        reference_snapshot_equatorial_parity_summary_for_report(),
        "JPL reference snapshot equatorial parity: ",
        "reference snapshot equatorial parity",
    )
    .ok()?;
    let reference_snapshot_body_class_coverage = required_summary_payload(
        reference_snapshot_body_class_coverage_summary_for_report(),
        "Reference snapshot body-class coverage: ",
        "reference snapshot body-class coverage",
    )
    .ok()?;
    let reference_snapshot_manifest = required_summary_payload(
        reference_snapshot_manifest_summary_for_report(),
        "Reference snapshot manifest: ",
        "reference snapshot manifest",
    )
    .ok()?;
    let comparison_snapshot_manifest = required_summary_payload(
        validated_comparison_snapshot_manifest_summary_for_report().ok()?,
        "Comparison snapshot manifest: ",
        "comparison snapshot manifest",
    )
    .ok()?;
    let independent_holdout_body_class_coverage = required_summary_payload(
        independent_holdout_snapshot_body_class_coverage_summary_for_report(),
        "Independent hold-out body-class coverage: ",
        "independent-holdout body-class coverage",
    )
    .ok()?;
    let independent_holdout_source_window = required_summary_payload(
        independent_holdout_snapshot_source_window_summary_for_report(),
        "Independent hold-out source windows: ",
        "independent-holdout source window",
    )
    .ok()?;
    let phase2_corpus_alignment =
        validated_packaged_artifact_phase2_corpus_alignment_summary_for_report();
    let pluto_fallback = required_summary_payload(
        format!(
            "Pluto fallback: {}",
            validated_pluto_fallback_summary_line_for_report().ok()?
        ),
        "Pluto fallback: ",
        "pluto fallback",
    )
    .ok()?;
    let production_generation_date_range = production_generation_date_range_for_report()?;
    let production_generation_quarter_day_boundary_samples = required_summary_payload(
        pleiades_jpl::production_generation_quarter_day_boundary_summary_for_report(),
        "Production generation quarter-day boundary samples: ",
        "production generation quarter-day boundary samples",
    )
    .ok()?;

    Some(SourceCorpusSummary {
        comparison_corpus_release_grade_guard,
        jpl_source_corpus_contract,
        jpl_evidence_classification,
        jpl_provenance_only,
        lunar_source_window,
        shared_schema: validated_checked_in_snapshot_schema_summary_for_report().ok()?,
        generation_command: "generate-packaged-artifact --check".to_string(),
        production_generation_source: required_summary_payload(
            validated_production_generation_source_summary_for_report().ok()?,
            "Production generation source: ",
            "production generation source",
        )
        .ok()?,
        production_generation_source_revision:
            validated_production_generation_source_revision_summary_for_report().ok()?,
        production_generation_coverage: required_summary_payload(
            production_generation_snapshot_summary_for_report(),
            "Production generation coverage: ",
            "production generation coverage",
        )
        .ok()?,
        production_generation_source_windows: required_summary_payload(
            production_generation_snapshot_window_summary_for_report(),
            "Production generation source windows: ",
            "production generation source windows",
        )
        .ok()?,
        production_generation_body_class_coverage: required_summary_payload(
            pleiades_jpl::production_generation_snapshot_body_class_coverage_summary_for_report(),
            "Production generation body-class coverage: ",
            "production generation body-class coverage",
        )
        .ok()?,
        production_generation_date_range,
        production_generation_quarter_day_boundary_samples,
        coverage_posture: production_generation_coverage_posture_for_report()?,
        production_generation_boundary_window: required_summary_payload(
            production_generation_boundary_window_summary_for_report(),
            "Production generation boundary windows: ",
            "production generation boundary window",
        )
        .ok()?,
        production_generation_boundary_source: required_summary_payload(
            production_generation_boundary_source_summary_for_report(),
            "Production generation boundary overlay source: ",
            "production generation boundary source",
        )
        .ok()?,
        production_generation_boundary_request_corpus: required_summary_payload(
            production_generation_boundary_request_corpus_summary_for_report(),
            "Production generation boundary request corpus: ",
            "production generation boundary request corpus",
        )
        .ok()?,
        production_generation_boundary_request_corpus_equatorial: required_summary_payload(
            production_generation_boundary_request_corpus_equatorial_summary_for_report(),
            "Production generation boundary request corpus: ",
            "production generation boundary request corpus equatorial",
        )
        .ok()?,
        reference_snapshot_sparse_boundary,
        reference_snapshot_exact_j2000_evidence,
        reference_snapshot_exact_j2000_body_class_coverage,
        reference_snapshot_equatorial_parity,
        reference_snapshot_body_class_coverage,
        reference_snapshot_manifest,
        comparison_snapshot_manifest,
        independent_holdout_body_class_coverage,
        independent_holdout_source_window,
        pluto_fallback,
        release_grade_body_claims,
        body_date_channel_claims: body_date_channel_claims_summary_details()?
            .validated_summary_line()
            .ok()?,
        phase2_corpus_alignment,
    })
}

pub(crate) fn validated_source_corpus_summary_for_report() -> Result<String, String> {
    let summary =
        source_corpus_summary_details().ok_or_else(|| "source corpus unavailable".to_string())?;
    summary
        .validated_summary_line()
        .map_err(|error| error.to_string())
}

pub(crate) fn source_corpus_summary_for_report() -> String {
    match validated_source_corpus_summary_for_report() {
        Ok(summary) => summary,
        Err(error) => format!("Source corpus unavailable ({error})"),
    }
}

pub(crate) fn source_corpus_posture_summary_for_report() -> String {
    source_corpus_summary_for_report()
}

pub(crate) fn render_comparison_corpus_release_guard_summary_text() -> String {
    let release_grade_guard = match validated_comparison_corpus_release_guard_summary_for_report() {
        Ok(guard) => guard,
        Err(error) => {
            return format!("Comparison corpus release-grade guard summary unavailable ({error})")
        }
    };
    format!(
        "Comparison corpus release-grade guard summary\nRelease-grade guard: {release_grade_guard}\n",
    )
}

pub(crate) fn ensure_comparison_corpus_release_guard_summary_matches_current_rendering(
    comparison_corpus_release_guard_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if comparison_corpus_release_guard_summary_text
        != render_comparison_corpus_release_guard_summary_text()
    {
        return Err(ReleaseBundleError::Verification(
            "comparison-corpus release-guard summary no longer matches the current comparison-corpus release-guard posture"
                .to_string(),
        ));
    }

    Ok(())
}

pub(crate) fn validated_benchmark_corpus_summary_for_report() -> Result<String, String> {
    let corpus = benchmark_corpus();
    let summary = corpus.summary();
    summary.validate().map_err(|error| error.to_string())?;

    let mut text = String::from("Benchmark corpus summary\n");
    write_corpus_summary_text(&mut text, &summary);
    text.push('\n');
    Ok(text)
}

pub(crate) fn validated_chart_benchmark_corpus_summary_for_report() -> Result<String, String> {
    let summary = chart_benchmark_corpus_summary();
    summary.validate().map_err(|error| error.to_string())?;

    let mut text = String::from("Chart benchmark corpus summary\n");
    write_corpus_summary_text(&mut text, &summary);
    text.push('\n');
    Ok(text)
}

pub(crate) fn render_benchmark_corpus_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();

    CACHE
        .get_or_init(|| match validated_benchmark_corpus_summary_for_report() {
            Ok(summary) => summary,
            Err(error) => format!("Benchmark corpus summary unavailable ({error})\n"),
        })
        .clone()
}

pub(crate) fn render_chart_benchmark_corpus_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();

    CACHE
        .get_or_init(
            || match validated_chart_benchmark_corpus_summary_for_report() {
                Ok(summary) => summary,
                Err(error) => format!("Chart benchmark corpus summary unavailable ({error})\n"),
            },
        )
        .clone()
}
