//! Relocated production-generation renderers copied from
//! `pleiades-jpl::reference_summary::production_generation` (report-surface
//! relocation program, Slice D). Rendering only — the functional crate keeps
//! the structured evidence structs, their `*_details()`/`*_summary()`
//! constructors, `validate()`/`label()` methods, and all release-gate data;
//! jpl's own rendering stays in place until the Task 14 contract sweep.
//!
//! This module covers both of `pleiades-jpl`'s `production_generation`
//! source files: `reference_summary/production_generation.rs` (Slice D Task
//! 10a — 17 free `*_for_report` renderers) and the top-level
//! `pleiades-jpl::production_generation` module (Slice D Task 10b — the
//! boundary-overlay structs' 7 free `*_for_report` renderers; the
//! request-corpus builders and every struct's `validate()`/`label()` stay in
//! jpl permanently, called via `pleiades_jpl::`). Task 10a's calls into the
//! top-level file were written before Task 10b copied it, so they were
//! initially left as `pleiades_jpl::<name>()` even though the target already
//! had a local copy here; every such residual, plus every rendering call
//! into an already-copied sibling file (`holdout.rs`,
//! `reference_snapshot/core/evidence.rs`, `reference_snapshot/core/general_b.rs`),
//! is now repointed to its local copy (Slice D Task 13b) — only data
//! accessors, `validate()`/`label()` gates, and jpl-only structs/constants
//! stay `pleiades_jpl::`.

use std::sync::OnceLock;

use pleiades_jpl::{
    IndependentHoldoutSourceSummary, ProductionGenerationBoundaryBodyClassCoverageSummary,
    ProductionGenerationBoundaryRequestCorpusSummary, ProductionGenerationBoundarySummary,
    ProductionGenerationBoundaryWindow, ProductionGenerationBoundaryWindowSummary,
    ProductionGenerationCorpusShapeSummary, ProductionGenerationManifestSummary,
    ProductionGenerationSnapshotBodyClassCoverageSummary, ProductionGenerationSnapshotSummary,
    ProductionGenerationSnapshotWindow, ProductionGenerationSnapshotWindowSummary,
    ProductionGenerationSourceRevisionSummary,
    ProductionGenerationSourceRevisionSummaryValidationError, ProductionGenerationSourceSummary,
    ProductionGenerationSourceSummaryValidationError,
};
use pleiades_types::{CoordinateFrame, Instant};

use crate::posture::jpl::holdout::independent_holdout_snapshot_source_window_summary_for_report;
use crate::posture::jpl::reference_snapshot::core::evidence::reference_snapshot_exact_j2000_evidence_summary_for_report;
use crate::posture::jpl::reference_snapshot::core::general_a::{
    format_bodies, strip_report_prefix,
};
use crate::posture::jpl::reference_snapshot::core::general_b::reference_snapshot_source_summary_line;

/// Reproduced from jpl's private `format_instant` (`lib.rs:66`), which is
/// crate-private and not callable cross-crate. Per-module duplicate accepted
/// (Slice D expand) — already reproduced identically in the sibling
/// `comparison.rs`, `holdout.rs`, `jpl_posture.rs`, and `reference_asteroid.rs`
/// posture modules.
fn format_instant(instant: Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

/// Reproduced from jpl's private `checksum64`
/// (`reference_summary/reference_snapshot/core/general_a.rs:452`), which is
/// crate-private and not callable cross-crate; that file's own doc comment
/// intentionally excludes it from its reusable-helper cluster ("not a
/// rendering helper"), so it is reproduced here rather than reused. Same
/// deterministic FNV-1a algorithm.
fn checksum64(text: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0001_0000_01b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Reproduced from jpl's `pub(crate)` `PRODUCTION_GENERATION_BOUNDARY_COVERAGE`
/// (the top-level `production_generation.rs:9`, a *different* file with its
/// own Slice D task, 10b), which is crate-private and not callable
/// cross-crate. Copied verbatim as a literal rather than promoted, so this
/// task does not modify a file outside its scope.
const PRODUCTION_GENERATION_BOUNDARY_COVERAGE: &str = "major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Mercury and Venus at 2451545, 2451915.25, and 2451915.75, plus Jupiter, Saturn, Uranus, Neptune, and Pluto at 2451545, plus Mars at 2451545, plus Sun at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Moon at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 66 rows across 16 bodies and 12 epochs.";

/// Compact release-facing summary line for the production-generation
/// coverage. Verbatim copy of `ProductionGenerationSnapshotSummary::summary_line`
/// (reference_summary/production_generation.rs:222).
pub(crate) fn production_generation_snapshot_summary_line(
    s: &ProductionGenerationSnapshotSummary,
) -> String {
    format!(
        "Production generation coverage: {} rows across {} bodies and {} epochs ({}..{}); bodies: {}; boundary overlay ({PRODUCTION_GENERATION_BOUNDARY_COVERAGE}): {} rows across {} bodies and {} epochs ({}..{}); boundary bodies: {}; quarter-day boundary samples: {} rows across {} bodies and {} epochs (JD 2451915.25 (TDB)..JD 2451915.75 (TDB)); quarter-day bodies: {}",
        s.row_count,
        s.body_count,
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        format_bodies(s.bodies),
        s.boundary_row_count,
        s.boundary_body_count,
        s.boundary_epoch_count,
        format_instant(s.boundary_earliest_epoch),
        format_instant(s.boundary_latest_epoch),
        format_bodies(s.boundary_bodies),
        s.quarter_day_row_count,
        s.quarter_day_body_count,
        s.quarter_day_epoch_count,
        format_bodies(s.quarter_day_bodies),
    )
}

/// Compact release-facing summary line for the production-generation source
/// revision. Verbatim copy of
/// `ProductionGenerationSourceRevisionSummary::summary_line`
/// (reference_summary/production_generation.rs:416).
pub(crate) fn production_generation_source_revision_summary_line(
    s: &ProductionGenerationSourceRevisionSummary,
) -> String {
    format!(
        "source revision=reference_snapshot.csv checksum=0x{reference_snapshot_checksum:016x}; independent_holdout_snapshot.csv checksum=0x{independent_holdout_snapshot_checksum:016x}",
        reference_snapshot_checksum = s.reference_snapshot_checksum,
        independent_holdout_snapshot_checksum = s.independent_holdout_snapshot_checksum,
    )
}

/// Verbatim copy of jpl's
/// `production_generation_source_cadence_fragment_from_counts`
/// (reference_summary/production_generation.rs:475), a same-file rendering
/// fragment helper consumed by `production_generation_source_summary_line`
/// below.
pub(crate) fn production_generation_source_cadence_fragment_from_counts(
    source_window_epoch_count: usize,
    boundary_epoch_count_ecliptic: usize,
    boundary_epoch_count_equatorial: usize,
) -> Result<String, ProductionGenerationSourceSummaryValidationError> {
    if boundary_epoch_count_ecliptic != boundary_epoch_count_equatorial {
        return Err(
            ProductionGenerationSourceSummaryValidationError::BoundaryRequestCorpusEpochCountMismatch {
                ecliptic_epoch_count: boundary_epoch_count_ecliptic,
                equatorial_epoch_count: boundary_epoch_count_equatorial,
            },
        );
    }

    Ok(format!(
        "cadence={} reference epochs and {} boundary epochs",
        source_window_epoch_count, boundary_epoch_count_ecliptic
    ))
}

/// Verbatim copy of jpl's `production_generation_source_cadence_fragment`
/// (reference_summary/production_generation.rs:495), with the
/// `production_generation_boundary_request_corpus_summary` constructor call
/// left as `pleiades_jpl::` (top-level `production_generation.rs`, Slice D
/// Task 10b, not yet copied).
pub(crate) fn production_generation_source_cadence_fragment(
    summary: &ProductionGenerationSourceSummary,
) -> Result<String, ProductionGenerationSourceSummaryValidationError> {
    let boundary_request_corpus_ecliptic =
        pleiades_jpl::production_generation_boundary_request_corpus_summary(
            CoordinateFrame::Ecliptic,
        )
        .ok_or(ProductionGenerationSourceSummaryValidationError::SourceWindowsMismatch)?;
    let boundary_request_corpus_equatorial =
        pleiades_jpl::production_generation_boundary_request_corpus_summary(
            CoordinateFrame::Equatorial,
        )
        .ok_or(ProductionGenerationSourceSummaryValidationError::SourceWindowsMismatch)?;

    production_generation_source_cadence_fragment_from_counts(
        summary.source_windows.epoch_count,
        boundary_request_corpus_ecliptic.epoch_count,
        boundary_request_corpus_equatorial.epoch_count,
    )
}

/// Verbatim copy of jpl's
/// `production_generation_source_body_class_cadence_fragment`
/// (reference_summary/production_generation.rs:512), with the same-file
/// `production_generation_snapshot_body_class_coverage_summary` constructor
/// and the top-level `production_generation_boundary_body_class_coverage_summary`
/// constructor (Slice D Task 10b, not yet copied) both called via
/// `pleiades_jpl::`.
pub(crate) fn production_generation_source_body_class_cadence_fragment(
) -> Result<String, ProductionGenerationSourceSummaryValidationError> {
    let snapshot = pleiades_jpl::production_generation_snapshot_body_class_coverage_summary()
        .ok_or(ProductionGenerationSourceSummaryValidationError::BodyClassCadenceMismatch)?;
    let boundary = pleiades_jpl::production_generation_boundary_body_class_coverage_summary()
        .ok_or(ProductionGenerationSourceSummaryValidationError::BodyClassCadenceMismatch)?;

    Ok(format!(
        "body-class cadence=reference major bodies: {} epochs; reference selected asteroids: {} epochs; boundary major bodies: {} epochs; boundary selected asteroids: {} epochs",
        snapshot.major_epoch_count,
        snapshot.asteroid_epoch_count,
        boundary.major_epoch_count,
        boundary.asteroid_epoch_count,
    ))
}

/// Compact release-facing summary line for the production-generation source
/// provenance. Verbatim copy of `ProductionGenerationSourceSummary::summary_line`
/// (reference_summary/production_generation.rs:655). `self.reference_summary`
/// is a `ReferenceSnapshotSourceSummary` whose rendering lives in
/// `reference_snapshot/core/general_b.rs` (Slice D Task 8, already copied) —
/// its `.summary_line()` call is rewired to the local
/// `reference_snapshot_source_summary_line` (Slice D Task 13b).
/// `self.source_windows` and `self.source_revision` are this file's own
/// structs, so their nested `.summary_line()` calls are rewired to the local
/// `production_generation_snapshot_window_summary_line` and
/// `production_generation_source_revision_summary_line`.
/// `format_production_generation_boundary_source_summary` (this file's own
/// top-level `production_generation.rs` copy, Task 10b) and
/// `reference_snapshot_exact_j2000_evidence_summary_for_report`
/// (`reference_snapshot/core/evidence.rs`, Slice D Task 8, already copied)
/// are both rewired to their local copies too (Slice D Task 13b).
pub(crate) fn production_generation_source_summary_line(
    s: &ProductionGenerationSourceSummary,
) -> String {
    let cadence_fragment = production_generation_source_cadence_fragment(s)
        .unwrap_or_else(|error| format!("cadence unavailable ({error})"));
    let body_class_cadence_fragment = production_generation_source_body_class_cadence_fragment()
        .unwrap_or_else(|error| format!("body-class cadence unavailable ({error})"));
    let source_density_fragment = production_generation_source_density_summary_for_report()
        .unwrap_or_else(|error| format!("source density floors unavailable ({error})"));

    format!(
        "Production generation source: strategy=documented hybrid fixture corpus; {}; {}; source windows={}; reference snapshot exact J2000 evidence={}; evidence classes=reference, hold-out, boundary overlay, provenance-only; input path=checked-in CSV fixtures via include_str! reference_snapshot.csv and independent_holdout_snapshot.csv; license posture=public-source provenance only; checked-in fixtures remain repository-local regression data; {}; generation command=generate-packaged-artifact --check (consuming the checked-in CSV fixtures); file format=comma-separated values; schema=epoch_jd, body, x_km, y_km, z_km; columns=epoch_jd, body, x_km, y_km, z_km; frame=geocentric ecliptic J2000; time scale=TDB; apparentness=Mean; parser=pure-Rust and deterministic; checksum expectation=byte-identical fixture contents; {}; {}; {}; reference and hold-out rows remain separate; redistribution posture=repository-checked regression fixtures, not a broad public corpus",
        reference_snapshot_source_summary_line(&s.reference_summary),
        format_production_generation_boundary_source_summary(&s.boundary_summary),
        strip_report_prefix(
            &production_generation_snapshot_window_summary_line(&s.source_windows),
            "Production generation source windows: ",
        ),
        strip_report_prefix(
            &reference_snapshot_exact_j2000_evidence_summary_for_report(),
            "Reference snapshot exact J2000 evidence: ",
        ),
        production_generation_source_revision_summary_line(&s.source_revision),
        cadence_fragment,
        body_class_cadence_fragment,
        source_density_fragment,
    )
}

/// Compact release-facing summary line for the production-generation corpus
/// shape. Verbatim copy of
/// `ProductionGenerationCorpusShapeSummary::summary_line`
/// (reference_summary/production_generation.rs:936). The two boundary
/// request corpus fields are `ProductionGenerationBoundaryRequestCorpusSummary`
/// values (this file's own top-level `production_generation.rs` copy, Slice
/// D Task 10b) — their `.summary_line()` calls are rewired to the local
/// `production_generation_boundary_request_corpus_summary_line` (Slice D
/// Task 13b).
pub(crate) fn production_generation_corpus_shape_summary_line(
    s: &ProductionGenerationCorpusShapeSummary,
) -> String {
    format!(
        "Production generation corpus shape: source={}; boundary request corpora: ecliptic={}; equatorial={}; validated fields=body order, epochs, frame, time scale, columns, apparentness, checksums",
        strip_report_prefix(
            &production_generation_source_summary_line(&s.source_summary),
            "Production generation source: ",
        ),
        strip_report_prefix(
            &production_generation_boundary_request_corpus_summary_line(
                &s.boundary_request_corpus_ecliptic
            ),
            "Production generation boundary request corpus: ",
        ),
        strip_report_prefix(
            &production_generation_boundary_request_corpus_summary_line(
                &s.boundary_request_corpus_equatorial
            ),
            "Production generation boundary request corpus: ",
        ),
    )
}

/// Compact release-facing summary line for the production-generation
/// manifest. Verbatim copy of
/// `ProductionGenerationManifestSummary::summary_line`
/// (reference_summary/production_generation.rs:1145). The boundary overlay,
/// boundary-window, and boundary-request-corpus fields are this file's own
/// top-level `production_generation.rs` types (Slice D Task 10b) — their
/// `.summary_line()` calls are rewired to the local
/// `production_generation_boundary_summary_line`,
/// `production_generation_boundary_window_summary_line`, and
/// `production_generation_boundary_request_corpus_summary_line` (Slice D
/// Task 13b).
pub(crate) fn production_generation_manifest_summary_line(
    s: &ProductionGenerationManifestSummary,
) -> String {
    format!(
        "Production generation manifest: coverage={}; source={}; body-class coverage={}; boundary overlay={}; boundary windows={}; boundary request corpus={}",
        strip_report_prefix(
            &production_generation_snapshot_summary_line(&s.coverage_summary),
            "Production generation coverage: ",
        ),
        strip_report_prefix(
            &production_generation_source_summary_line(&s.source_summary),
            "Production generation source: ",
        ),
        strip_report_prefix(
            &production_generation_snapshot_body_class_coverage_summary_line(
                &s.body_class_coverage_summary
            ),
            "Production generation body-class coverage: ",
        ),
        strip_report_prefix(
            &production_generation_boundary_summary_line(&s.boundary_summary),
            "Production generation boundary overlay: "
        ),
        strip_report_prefix(
            &production_generation_boundary_window_summary_line(&s.boundary_window_summary),
            "Production generation boundary windows: ",
        ),
        strip_report_prefix(
            &production_generation_boundary_request_corpus_summary_line(
                &s.boundary_request_corpus_summary
            ),
            "Production generation boundary request corpus: ",
        ),
    )
}

/// Compact release-facing summary line for a single production-generation
/// body window. Verbatim copy of
/// `ProductionGenerationSnapshotWindow::summary_line`
/// (reference_summary/production_generation.rs:1353).
pub(crate) fn production_generation_snapshot_window_line(
    s: &ProductionGenerationSnapshotWindow,
) -> String {
    let time_span = if s.earliest_epoch == s.latest_epoch {
        format_instant(s.earliest_epoch)
    } else {
        format!(
            "{}..{}",
            format_instant(s.earliest_epoch),
            format_instant(s.latest_epoch)
        )
    };

    format!(
        "{}: {} samples across {} epochs at {}",
        s.body, s.sample_count, s.epoch_count, time_span
    )
}

/// Compact release-facing summary line for the merged production-generation
/// source windows. Verbatim copy of
/// `ProductionGenerationSnapshotWindowSummary::summary_line`
/// (reference_summary/production_generation.rs:1503), with the nested
/// `self.windows.iter().map(ToString::to_string)` (jpl's `Display` for
/// `ProductionGenerationSnapshotWindow`, same file) rewritten to the local
/// `production_generation_snapshot_window_line`.
pub(crate) fn production_generation_snapshot_window_summary_line(
    s: &ProductionGenerationSnapshotWindowSummary,
) -> String {
    format!(
        "Production generation source windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
        s.sample_count,
        s.sample_bodies.len(),
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        s.windows
            .iter()
            .map(production_generation_snapshot_window_line)
            .collect::<Vec<_>>()
            .join("; ")
    )
}

/// Compact release-facing summary line for the production-generation
/// body-class coverage. Verbatim copy of
/// `ProductionGenerationSnapshotBodyClassCoverageSummary::summary_line`
/// (reference_summary/production_generation.rs:1770), with the nested
/// `.map(ProductionGenerationSnapshotWindow::summary_line)` (same file)
/// rewritten to the local `production_generation_snapshot_window_line`.
pub(crate) fn production_generation_snapshot_body_class_coverage_summary_line(
    s: &ProductionGenerationSnapshotBodyClassCoverageSummary,
) -> String {
    let major_windows = s
        .major_windows
        .iter()
        .map(production_generation_snapshot_window_line)
        .collect::<Vec<_>>()
        .join("; ");
    let asteroid_windows = s
        .asteroid_windows
        .iter()
        .map(production_generation_snapshot_window_line)
        .collect::<Vec<_>>()
        .join("; ");

    format!(
        "Production generation body-class coverage: major bodies: {} rows across {} bodies and {} epochs; major windows: {}; selected asteroids: {} rows across {} bodies and {} epochs; asteroid windows: {}",
        s.major_body_row_count,
        s.major_bodies.len(),
        s.major_epoch_count,
        major_windows,
        s.asteroid_row_count,
        s.asteroid_bodies.len(),
        s.asteroid_epoch_count,
        asteroid_windows,
    )
}

/// Compact release-facing summary line for the production-generation
/// boundary overlay. Verbatim copy of
/// `ProductionGenerationBoundarySummary::summary_line` (top-level
/// `production_generation.rs:358`, Slice D Task 10b).
pub(crate) fn production_generation_boundary_summary_line(
    s: &ProductionGenerationBoundarySummary,
) -> String {
    format!(
        "Production generation boundary overlay: {} rows across {} bodies and {} epochs ({}..{}); bodies: {}",
        s.row_count,
        s.body_count,
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        format_bodies(s.bodies),
    )
}

/// Verbatim copy of jpl's `format_production_generation_boundary_source_summary`
/// (top-level `production_generation.rs:441`, Slice D Task 10b), with the
/// checksum call rewired to the local
/// `production_generation_boundary_source_checksum` (below) in place of
/// jpl's private `independent_holdout_snapshot_checksum`.
pub(crate) fn format_production_generation_boundary_source_summary(
    summary: &IndependentHoldoutSourceSummary,
) -> String {
    format!(
        "Production generation boundary overlay source: {}; evidence class={}; coverage={}; columns={}; redistribution={}; checksum=0x{:016x}; {}; time scale={}",
        summary.source,
        summary.evidence_class,
        summary.coverage,
        summary.columns,
        summary.redistribution,
        production_generation_boundary_source_checksum(),
        summary.frame_treatment,
        summary.time_scale,
    )
}

/// Verbatim copy of jpl's private `independent_holdout_snapshot_checksum`
/// (top-level `production_generation.rs:457`, Slice D Task 10b). Note this
/// is a *different* FNV-1a constant pairing than this file's `checksum64`
/// (Task 10a, reproduced from the unrelated
/// `reference_summary/reference_snapshot/core/general_a.rs:452` helper of
/// the same name) — that helper's prime literal
/// (`0x0000_0001_0000_01b3` = `0x1000001b3`) differs from this file's
/// top-level jpl source's `fnv1a64` prime (`0x100000001b3`, the standard
/// 64-bit FNV prime), so it is **not** reused here; `fnv1a64` is reproduced
/// separately below to preserve byte-identity with jpl's actual checksum.
/// The `include_str!` reaches one directory over to jpl's checked-in copy of
/// the CSV (jpl's own `env!("CARGO_MANIFEST_DIR")` resolves against jpl's
/// manifest dir, not validate's); the bytes read are identical either way
/// (established precedent — `holdout.rs`/`comparison.rs` do the same).
fn production_generation_boundary_source_checksum() -> u64 {
    static CHECKSUM: OnceLock<u64> = OnceLock::new();
    *CHECKSUM.get_or_init(|| {
        fnv1a64(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../pleiades-jpl/data/independent_holdout_snapshot.csv"
            ))
            .as_bytes(),
        )
    })
}

/// Verbatim copy of jpl's private `fnv1a64` (top-level
/// `production_generation.rs:470`, Slice D Task 10b). Deliberately *not*
/// unified with this file's `checksum64` (Task 10a) — see the doc comment
/// on `production_generation_boundary_source_checksum` above for why the two
/// constant pairings differ.
fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET_BASIS;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

/// Compact release-facing summary line for a single production-generation
/// boundary-overlay body window. Verbatim copy of
/// `ProductionGenerationBoundaryWindow::summary_line` (top-level
/// `production_generation.rs:509`, Slice D Task 10b).
pub(crate) fn production_generation_boundary_window_line(
    s: &ProductionGenerationBoundaryWindow,
) -> String {
    let time_span = if s.earliest_epoch == s.latest_epoch {
        format_instant(s.earliest_epoch)
    } else {
        format!(
            "{}..{}",
            format_instant(s.earliest_epoch),
            format_instant(s.latest_epoch)
        )
    };

    format!(
        "{}: {} samples across {} epochs at {}",
        s.body, s.sample_count, s.epoch_count, time_span
    )
}

/// Compact release-facing summary line for the production-generation
/// boundary windows. Verbatim copy of
/// `ProductionGenerationBoundaryWindowSummary::summary_line` (top-level
/// `production_generation.rs:566`, Slice D Task 10b), with the nested
/// `self.windows.iter().map(ProductionGenerationBoundaryWindow::summary_line)`
/// (same file) rewritten to the local
/// `production_generation_boundary_window_line`.
pub(crate) fn production_generation_boundary_window_summary_line(
    s: &ProductionGenerationBoundaryWindowSummary,
) -> String {
    let window_summary = s
        .windows
        .iter()
        .map(production_generation_boundary_window_line)
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "Production generation boundary windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
        s.sample_count,
        s.sample_bodies.len(),
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        window_summary,
    )
}

/// Compact release-facing summary line for the production-generation
/// boundary body-class coverage. Verbatim copy of
/// `ProductionGenerationBoundaryBodyClassCoverageSummary::summary_line`
/// (top-level `production_generation.rs:753`, Slice D Task 10b), with the
/// nested `.map(ProductionGenerationBoundaryWindow::summary_line)` calls
/// (same file) rewritten to the local
/// `production_generation_boundary_window_line`.
pub(crate) fn production_generation_boundary_body_class_coverage_summary_line(
    s: &ProductionGenerationBoundaryBodyClassCoverageSummary,
) -> String {
    let major_windows = s
        .major_windows
        .iter()
        .map(production_generation_boundary_window_line)
        .collect::<Vec<_>>()
        .join("; ");
    let asteroid_windows = s
        .asteroid_windows
        .iter()
        .map(production_generation_boundary_window_line)
        .collect::<Vec<_>>()
        .join("; ");

    format!(
        "Production generation boundary body-class coverage: major bodies: {} rows across {} bodies and {} epochs; major windows: {}; selected asteroids: {} rows across {} bodies and {} epochs; asteroid windows: {}",
        s.major_body_row_count,
        s.major_bodies.len(),
        s.major_epoch_count,
        major_windows,
        s.asteroid_row_count,
        s.asteroid_bodies.len(),
        s.asteroid_epoch_count,
        asteroid_windows,
    )
}

/// Compact release-facing summary line for the production-generation
/// boundary request corpus. Verbatim copy of
/// `ProductionGenerationBoundaryRequestCorpusSummary::summary_line`
/// (top-level `production_generation.rs:989`, Slice D Task 10b).
pub(crate) fn production_generation_boundary_request_corpus_summary_line(
    s: &ProductionGenerationBoundaryRequestCorpusSummary,
) -> String {
    format!(
        "Production generation boundary request corpus: {} requests (frame={}; time scale={}; zodiac mode={}; apparentness={}; observerless) across {} bodies and {} epochs ({}..{}); bodies: {}",
        s.request_count,
        s.frame,
        s.time_scale,
        s.zodiac_mode,
        s.apparentness,
        s.body_count,
        s.epoch_count,
        format_instant(s.earliest_epoch),
        format_instant(s.latest_epoch),
        format_bodies(&s.bodies),
    )
}

/// Returns the release-facing production-generation coverage summary string.
/// Verbatim copy of jpl's `production_generation_snapshot_summary_for_report`
/// (reference_summary/production_generation.rs:353), with
/// `summary.validated_summary_line()` rewired to
/// `match summary.validate() { Ok(()) => <local render>, ... }` (`validate()`
/// stays on the jpl struct; rendering is local).
pub fn production_generation_snapshot_summary_for_report() -> String {
    match pleiades_jpl::production_generation_snapshot_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => production_generation_snapshot_summary_line(&summary),
            Err(error) => format!("Production generation coverage: unavailable ({error})"),
        },
        None => "Production generation coverage: unavailable".to_string(),
    }
}

/// Returns the compact quarter-day boundary sample summary for release-facing
/// reporting. Verbatim copy of jpl's
/// `production_generation_quarter_day_boundary_summary_for_report`
/// (reference_summary/production_generation.rs:364).
pub fn production_generation_quarter_day_boundary_summary_for_report() -> String {
    match pleiades_jpl::production_generation_snapshot_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => format!(
                "Production generation quarter-day boundary samples: {} rows across {} bodies and {} epochs (JD 2451915.25 (TDB)..JD 2451915.75 (TDB)); bodies: {}",
                summary.quarter_day_row_count,
                summary.quarter_day_body_count,
                summary.quarter_day_epoch_count,
                format_bodies(summary.quarter_day_bodies),
            ),
            Err(error) => {
                format!("Production generation quarter-day boundary samples: unavailable ({error})")
            }
        },
        None => "Production generation quarter-day boundary samples: unavailable".to_string(),
    }
}

/// Returns the release-facing production-generation source revision summary
/// string. Verbatim copy of jpl's
/// `production_generation_source_revision_summary_for_report`
/// (reference_summary/production_generation.rs:448), with
/// `production_generation_source_revision_summary` promoted to `pub` (Slice D
/// Task 10a) and `.validated_summary_line()` rewired to
/// `match s.validate() { Ok(()) => <local render>, ... }`.
#[doc(alias = "production_generation_source_revision_summary")]
pub fn production_generation_source_revision_summary_for_report() -> String {
    let s = pleiades_jpl::production_generation_source_revision_summary();
    match s.validate() {
        Ok(()) => production_generation_source_revision_summary_line(&s),
        Err(error) => format!("source revision=unavailable ({error})"),
    }
}

/// Returns the validated release-facing production-generation source
/// revision summary string. Verbatim copy of jpl's
/// `validated_production_generation_source_revision_summary_for_report`
/// (reference_summary/production_generation.rs:457).
#[doc(alias = "production_generation_source_revision_summary")]
pub(crate) fn validated_production_generation_source_revision_summary_for_report(
) -> Result<String, ProductionGenerationSourceRevisionSummaryValidationError> {
    let s = pleiades_jpl::production_generation_source_revision_summary();
    s.validate()?;
    Ok(production_generation_source_revision_summary_line(&s))
}

/// Returns a compact source-density summary for the production-generation
/// corpus. Verbatim copy of jpl's
/// `production_generation_source_density_summary_for_report`
/// (reference_summary/production_generation.rs:529).
pub(crate) fn production_generation_source_density_summary_for_report(
) -> Result<String, ProductionGenerationSourceSummaryValidationError> {
    let snapshot = pleiades_jpl::production_generation_snapshot_body_class_coverage_summary()
        .ok_or(ProductionGenerationSourceSummaryValidationError::SourceDensityMismatch)?;
    let boundary = pleiades_jpl::production_generation_boundary_body_class_coverage_summary()
        .ok_or(ProductionGenerationSourceSummaryValidationError::SourceDensityMismatch)?;

    Ok(format!(
        "source density floors=reference major bodies: {} epochs minimum; reference selected asteroids: {} epochs minimum; boundary major bodies: {} epochs minimum; boundary selected asteroids: {} epochs minimum",
        snapshot.major_epoch_count,
        snapshot.asteroid_epoch_count,
        boundary.major_epoch_count,
        boundary.asteroid_epoch_count,
    ))
}

/// Returns the validated production-generation source-class breakdown line
/// for release reports. Verbatim copy of jpl's
/// `production_generation_source_class_breakdown_summary_for_report`
/// (reference_summary/production_generation.rs:546), with the same-file
/// `production_generation_snapshot_window_summary_for_report` call local, and
/// `independent_holdout_snapshot_source_window_summary_for_report`
/// (`holdout.rs`, already copied) and
/// `production_generation_boundary_summary_for_report` (this file's own
/// top-level `production_generation.rs` copy, Slice D Task 10b) both rewired
/// to their local copies (Slice D Task 13b).
pub(crate) fn production_generation_source_class_breakdown_summary_for_report() -> String {
    format!(
        "Production generation source class breakdown: reference source windows={}; hold-out source windows={}; boundary overlay={}; provenance-only source and manifest summaries remain separate",
        strip_report_prefix(
            &production_generation_snapshot_window_summary_for_report(),
            "Production generation source windows: ",
        ),
        strip_report_prefix(
            &independent_holdout_snapshot_source_window_summary_for_report(),
            "Independent hold-out source windows: ",
        ),
        strip_report_prefix(
            &production_generation_boundary_summary_for_report(),
            "Production generation boundary overlay: ",
        ),
    )
}

/// Returns the release-facing production-generation source summary string.
/// Verbatim copy of jpl's `production_generation_source_summary_for_report`
/// (reference_summary/production_generation.rs:801).
pub fn production_generation_source_summary_for_report() -> String {
    let summary = pleiades_jpl::production_generation_source_summary();
    match summary.validate() {
        Ok(()) => production_generation_source_summary_line(&summary),
        Err(error) => format!("Production generation source: unavailable ({error})"),
    }
}

/// Returns the validated release-facing production-generation source summary
/// string. Verbatim copy of jpl's
/// `validated_production_generation_source_summary_for_report`
/// (reference_summary/production_generation.rs:810).
pub(crate) fn validated_production_generation_source_summary_for_report() -> Result<String, String>
{
    let summary = pleiades_jpl::production_generation_source_summary();
    summary.validate().map_err(|error| error.to_string())?;
    Ok(production_generation_source_summary_line(&summary))
}

/// Returns the release-facing production-generation corpus-shape summary
/// string. Verbatim copy of jpl's
/// `production_generation_corpus_shape_summary_for_report`
/// (reference_summary/production_generation.rs:1043).
pub fn production_generation_corpus_shape_summary_for_report() -> String {
    match pleiades_jpl::production_generation_corpus_shape_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => production_generation_corpus_shape_summary_line(&summary),
            Err(error) => format!("Production generation corpus shape: unavailable ({error})"),
        },
        None => "Production generation corpus shape: unavailable".to_string(),
    }
}

/// Returns the validated release-facing production-generation corpus-shape
/// summary string. Verbatim copy of jpl's
/// `validated_production_generation_corpus_shape_summary_for_report`
/// (reference_summary/production_generation.rs:1054).
pub(crate) fn validated_production_generation_corpus_shape_summary_for_report(
) -> Result<String, String> {
    let summary = pleiades_jpl::production_generation_corpus_shape_summary()
        .ok_or_else(|| "production generation corpus shape unavailable".to_string())?;
    summary.validate().map_err(|error| error.to_string())?;
    Ok(production_generation_corpus_shape_summary_line(&summary))
}

/// Returns the release-facing production-generation manifest summary string.
/// Verbatim copy of jpl's `production_generation_manifest_summary_for_report`
/// (reference_summary/production_generation.rs:1300).
pub(crate) fn production_generation_manifest_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(
            || match pleiades_jpl::production_generation_manifest_summary() {
                Some(summary) => match summary.validate() {
                    Ok(()) => production_generation_manifest_summary_line(&summary),
                    Err(error) => format!("Production generation manifest: unavailable ({error})"),
                },
                None => "Production generation manifest: unavailable".to_string(),
            },
        )
        .clone()
}

/// Returns the validated release-facing production-generation manifest
/// summary string. Verbatim copy of jpl's
/// `validated_production_generation_manifest_summary_for_report`
/// (reference_summary/production_generation.rs:1314).
pub(crate) fn validated_production_generation_manifest_summary_for_report() -> Result<String, String>
{
    let summary = pleiades_jpl::production_generation_manifest_summary()
        .ok_or_else(|| "production generation manifest unavailable".to_string())?;
    summary.validate().map_err(|error| error.to_string())?;
    Ok(production_generation_manifest_summary_line(&summary))
}

/// Returns the release-facing production-generation manifest checksum
/// summary string. Verbatim copy of jpl's
/// `production_generation_manifest_checksum_for_report`
/// (reference_summary/production_generation.rs:1323), with `checksum64`
/// reproduced locally (see above).
pub(crate) fn production_generation_manifest_checksum_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = production_generation_manifest_summary_for_report();
            format!(
                "Production generation manifest checksum: 0x{:016x}",
                checksum64(&summary)
            )
        })
        .clone()
}

/// Returns the release-facing production-generation source window summary
/// string. Verbatim copy of jpl's
/// `production_generation_snapshot_window_summary_for_report`
/// (reference_summary/production_generation.rs:1702).
pub fn production_generation_snapshot_window_summary_for_report() -> String {
    match pleiades_jpl::production_generation_snapshot_window_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => production_generation_snapshot_window_summary_line(&summary),
            Err(error) => format!("Production generation source windows: unavailable ({error})"),
        },
        None => "Production generation source windows: unavailable".to_string(),
    }
}

/// Returns the validated release-facing production-generation source window
/// summary string. Verbatim copy of jpl's
/// `validated_production_generation_snapshot_window_summary_for_report`
/// (reference_summary/production_generation.rs:1713).
pub(crate) fn validated_production_generation_snapshot_window_summary_for_report(
) -> Result<String, String> {
    let summary = pleiades_jpl::production_generation_snapshot_window_summary()
        .ok_or_else(|| "production generation source windows unavailable".to_string())?;
    summary.validate().map_err(|error| error.to_string())?;
    Ok(production_generation_snapshot_window_summary_line(&summary))
}

/// Returns the release-facing body-class coverage summary string for the
/// merged production-generation corpus. Verbatim copy of jpl's
/// `production_generation_snapshot_body_class_coverage_summary_for_report`
/// (reference_summary/production_generation.rs:1955).
pub(crate) fn production_generation_snapshot_body_class_coverage_summary_for_report() -> String {
    match pleiades_jpl::production_generation_snapshot_body_class_coverage_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => production_generation_snapshot_body_class_coverage_summary_line(&summary),
            Err(error) => {
                format!("Production generation body-class coverage: unavailable ({error})")
            }
        },
        None => "Production generation body-class coverage: unavailable".to_string(),
    }
}

/// Returns the validated release-facing body-class coverage summary string
/// for the merged production-generation corpus. Verbatim copy of jpl's
/// `validated_production_generation_snapshot_body_class_coverage_summary_for_report`
/// (reference_summary/production_generation.rs:1968).
pub(crate) fn validated_production_generation_snapshot_body_class_coverage_summary_for_report(
) -> Result<String, String> {
    let summary = pleiades_jpl::production_generation_snapshot_body_class_coverage_summary()
        .ok_or_else(|| {
            pleiades_jpl::ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                field: "row_count",
            }
            .to_string()
        })?;
    summary.validate().map_err(|error| error.to_string())?;
    Ok(production_generation_snapshot_body_class_coverage_summary_line(&summary))
}

/// Returns the release-facing production-generation boundary overlay
/// summary string. Verbatim copy of jpl's
/// `production_generation_boundary_summary_for_report` (top-level
/// `production_generation.rs:424`, Slice D Task 10b), with
/// `summary.validated_summary_line()` rewired to
/// `match summary.validate() { Ok(()) => <local render>, ... }` (`validate()`
/// stays on the jpl struct; rendering is local).
pub fn production_generation_boundary_summary_for_report() -> String {
    match pleiades_jpl::production_generation_boundary_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => production_generation_boundary_summary_line(&summary),
            Err(error) => format!("Production generation boundary overlay: unavailable ({error})"),
        },
        None => "Production generation boundary overlay: unavailable".to_string(),
    }
}

/// Returns the release-facing provenance summary string for the
/// production-generation boundary overlay. Verbatim copy of jpl's
/// `production_generation_boundary_source_summary_for_report` (top-level
/// `production_generation.rs:482`, Slice D Task 10b).
pub fn production_generation_boundary_source_summary_for_report() -> String {
    let summary = pleiades_jpl::production_generation_boundary_source_summary();
    match summary.validate() {
        Ok(()) => format_production_generation_boundary_source_summary(&summary),
        Err(error) => {
            format!("Production generation boundary overlay source: unavailable ({error})")
        }
    }
}

/// Returns the release-facing production-generation boundary window summary
/// string. Verbatim copy of jpl's
/// `production_generation_boundary_window_summary_for_report` (top-level
/// `production_generation.rs:695`, Slice D Task 10b).
pub fn production_generation_boundary_window_summary_for_report() -> String {
    match pleiades_jpl::production_generation_boundary_window_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => production_generation_boundary_window_summary_line(&summary),
            Err(error) => format!("Production generation boundary windows: unavailable ({error})"),
        },
        None => "Production generation boundary windows: unavailable".to_string(),
    }
}

/// Returns the release-facing body-class coverage summary string for the
/// production-generation boundary overlay. Verbatim copy of jpl's
/// `production_generation_boundary_body_class_coverage_summary_for_report`
/// (top-level `production_generation.rs:882`, Slice D Task 10b).
pub(crate) fn production_generation_boundary_body_class_coverage_summary_for_report() -> String {
    match pleiades_jpl::production_generation_boundary_body_class_coverage_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => production_generation_boundary_body_class_coverage_summary_line(&summary),
            Err(error) => {
                format!("Production generation boundary body-class coverage: unavailable ({error})")
            }
        },
        None => "Production generation boundary body-class coverage: unavailable".to_string(),
    }
}

/// Returns the validated release-facing production-generation boundary
/// request corpus summary string for the given frame. Verbatim copy of
/// jpl's private
/// `validated_production_generation_boundary_request_corpus_summary_for_frame`
/// (top-level `production_generation.rs:1189`, Slice D Task 10b), with
/// `summary.validated_summary_line()` rewired to
/// `{ summary.validate().map_err(...)?; Ok(<local render>) }` (`validate()`
/// stays on the jpl struct; rendering is local).
fn validated_production_generation_boundary_request_corpus_summary_for_frame(
    frame: CoordinateFrame,
) -> Result<String, String> {
    let summary = pleiades_jpl::production_generation_boundary_request_corpus_summary(frame)
        .ok_or_else(|| "production generation boundary request corpus unavailable".to_string())?;
    summary.validate().map_err(|error| error.to_string())?;
    Ok(production_generation_boundary_request_corpus_summary_line(
        &summary,
    ))
}

/// Returns the release-facing production-generation boundary request corpus
/// summary string. Verbatim copy of jpl's
/// `production_generation_boundary_request_corpus_summary_for_report`
/// (top-level `production_generation.rs:1200`, Slice D Task 10b).
pub fn production_generation_boundary_request_corpus_summary_for_report() -> String {
    validated_production_generation_boundary_request_corpus_summary_for_frame(
        CoordinateFrame::Ecliptic,
    )
    .unwrap_or_else(|error| {
        format!("Production generation boundary request corpus: unavailable ({error})")
    })
}

/// Returns the release-facing equatorial production-generation boundary
/// request corpus summary string. Verbatim copy of jpl's
/// `production_generation_boundary_request_corpus_equatorial_summary_for_report`
/// (top-level `production_generation.rs:1210`, Slice D Task 10b).
pub fn production_generation_boundary_request_corpus_equatorial_summary_for_report() -> String {
    validated_production_generation_boundary_request_corpus_summary_for_frame(
        CoordinateFrame::Equatorial,
    )
    .unwrap_or_else(|error| {
        format!("Production generation boundary request corpus: unavailable ({error})")
    })
}

/// Returns the validated release-facing equatorial production-generation
/// boundary request corpus summary string. Verbatim copy of jpl's
/// `validated_production_generation_boundary_request_corpus_equatorial_summary_for_report`
/// (top-level `production_generation.rs:1220`, Slice D Task 10b).
pub(crate) fn validated_production_generation_boundary_request_corpus_equatorial_summary_for_report(
) -> Result<String, String> {
    validated_production_generation_boundary_request_corpus_summary_for_frame(
        CoordinateFrame::Equatorial,
    )
}

#[cfg(test)]
mod golden {
    // jpl's own renderers were deleted in the Task 14 contract sweep. The
    // `EXPECTED_*` constants below are byte-exact captures of jpl's renderer
    // output taken immediately before deletion (Slice D Task 14a); this
    // still fails closed on any drift in the validate copy, just pinned to a
    // literal instead of a live jpl call.
    const EXPECTED_PRODUCTION_GENERATION_BOUNDARY_SUMMARY_FOR_REPORT: &str = r"Production generation boundary overlay: 66 rows across 16 bodies and 12 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); bodies: Mars, Jupiter, Mercury, Venus, Saturn, Uranus, Neptune, Sun, Pluto, Moon, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis";
    const EXPECTED_PRODUCTION_GENERATION_BOUNDARY_SOURCE_SUMMARY_FOR_REPORT: &str = r"Production generation boundary overlay source: NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.; evidence class=hold-out; coverage=major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Mercury and Venus at 2451545, 2451915.25, and 2451915.75, plus Jupiter, Saturn, Uranus, Neptune, and Pluto at 2451545, plus Mars at 2451545, plus Sun at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Moon at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 66 rows across 16 bodies and 12 epochs.; columns=epoch_jd, body, x_km, y_km, z_km; redistribution=repository-checked regression fixtures, not a broad public corpus.; checksum=0x7d16ee3690481698; geocentric ecliptic J2000; time scale=TDB";
    const EXPECTED_PRODUCTION_GENERATION_BOUNDARY_WINDOW_SUMMARY_FOR_REPORT: &str = r"Production generation boundary windows: 66 source-backed samples across 16 bodies and 12 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); windows: Mars: 5 samples across 5 epochs at JD 2451545.0 (TDB)..JD 2451915.5 (TDB); Jupiter: 5 samples across 5 epochs at JD 2451545.0 (TDB)..JD 2451915.5 (TDB); Mercury: 4 samples across 4 epochs at JD 2451545.0 (TDB)..JD 2451915.8 (TDB); Venus: 4 samples across 4 epochs at JD 2451545.0 (TDB)..JD 2451915.8 (TDB); Saturn: 2 samples across 2 epochs at JD 2451545.0 (TDB)..JD 2451915.5 (TDB); Uranus: 2 samples across 2 epochs at JD 2451545.0 (TDB)..JD 2451915.5 (TDB); Neptune: 2 samples across 2 epochs at JD 2451545.0 (TDB)..JD 2451915.5 (TDB); Sun: 4 samples across 4 epochs at JD 2451545.0 (TDB)..JD 2451915.8 (TDB); Pluto: 2 samples across 2 epochs at JD 2451545.0 (TDB)..JD 2451915.5 (TDB); Moon: 4 samples across 4 epochs at JD 2451545.0 (TDB)..JD 2451915.8 (TDB); Ceres: 5 samples across 5 epochs at JD 2378498.5 (TDB)..JD 2500000.0 (TDB); Pallas: 5 samples across 5 epochs at JD 2378498.5 (TDB)..JD 2500000.0 (TDB); Juno: 5 samples across 5 epochs at JD 2378498.5 (TDB)..JD 2500000.0 (TDB); Vesta: 5 samples across 5 epochs at JD 2378498.5 (TDB)..JD 2500000.0 (TDB); asteroid:433-Eros: 6 samples across 6 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); asteroid:99942-Apophis: 6 samples across 6 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB)";
    const EXPECTED_PRODUCTION_GENERATION_BOUNDARY_BODY_CLASS_COVERAGE_SUMMARY_FOR_REPORT: &str = r"Production generation boundary body-class coverage: major bodies: 34 rows across 10 bodies and 7 epochs; major windows: Mars: 5 samples across 5 epochs at JD 2451545.0 (TDB)..JD 2451915.5 (TDB); Jupiter: 5 samples across 5 epochs at JD 2451545.0 (TDB)..JD 2451915.5 (TDB); Mercury: 4 samples across 4 epochs at JD 2451545.0 (TDB)..JD 2451915.8 (TDB); Venus: 4 samples across 4 epochs at JD 2451545.0 (TDB)..JD 2451915.8 (TDB); Saturn: 2 samples across 2 epochs at JD 2451545.0 (TDB)..JD 2451915.5 (TDB); Uranus: 2 samples across 2 epochs at JD 2451545.0 (TDB)..JD 2451915.5 (TDB); Neptune: 2 samples across 2 epochs at JD 2451545.0 (TDB)..JD 2451915.5 (TDB); Sun: 4 samples across 4 epochs at JD 2451545.0 (TDB)..JD 2451915.8 (TDB); Pluto: 2 samples across 2 epochs at JD 2451545.0 (TDB)..JD 2451915.5 (TDB); Moon: 4 samples across 4 epochs at JD 2451545.0 (TDB)..JD 2451915.8 (TDB); selected asteroids: 32 rows across 6 bodies and 7 epochs; asteroid windows: Ceres: 5 samples across 5 epochs at JD 2378498.5 (TDB)..JD 2500000.0 (TDB); Pallas: 5 samples across 5 epochs at JD 2378498.5 (TDB)..JD 2500000.0 (TDB); Juno: 5 samples across 5 epochs at JD 2378498.5 (TDB)..JD 2500000.0 (TDB); Vesta: 5 samples across 5 epochs at JD 2378498.5 (TDB)..JD 2500000.0 (TDB); asteroid:433-Eros: 6 samples across 6 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); asteroid:99942-Apophis: 6 samples across 6 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB)";
    const EXPECTED_PRODUCTION_GENERATION_BOUNDARY_REQUEST_CORPUS_SUMMARY_FOR_REPORT: &str = r"Production generation boundary request corpus: 66 requests (frame=Ecliptic; time scale=TDB; zodiac mode=Tropical; apparentness=Mean; observerless) across 16 bodies and 12 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); bodies: Mars, Jupiter, Mercury, Venus, Saturn, Uranus, Neptune, Sun, Pluto, Moon, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis";
    const EXPECTED_PRODUCTION_GENERATION_BOUNDARY_REQUEST_CORPUS_EQUATORIAL_SUMMARY_FOR_REPORT:
        &str = r"Production generation boundary request corpus: 66 requests (frame=Equatorial; time scale=TDB; zodiac mode=Tropical; apparentness=Mean; observerless) across 16 bodies and 12 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); bodies: Mars, Jupiter, Mercury, Venus, Saturn, Uranus, Neptune, Sun, Pluto, Moon, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis";

    #[test]
    fn production_generation_boundary_summary_for_report_byte_identical() {
        assert_eq!(
            super::production_generation_boundary_summary_for_report(),
            EXPECTED_PRODUCTION_GENERATION_BOUNDARY_SUMMARY_FOR_REPORT
        );
    }

    #[test]
    fn production_generation_boundary_source_summary_for_report_byte_identical() {
        assert_eq!(
            super::production_generation_boundary_source_summary_for_report(),
            EXPECTED_PRODUCTION_GENERATION_BOUNDARY_SOURCE_SUMMARY_FOR_REPORT
        );
    }

    #[test]
    fn production_generation_boundary_window_summary_for_report_byte_identical() {
        assert_eq!(
            super::production_generation_boundary_window_summary_for_report(),
            EXPECTED_PRODUCTION_GENERATION_BOUNDARY_WINDOW_SUMMARY_FOR_REPORT
        );
    }

    #[test]
    fn production_generation_boundary_body_class_coverage_summary_for_report_byte_identical() {
        assert_eq!(
            super::production_generation_boundary_body_class_coverage_summary_for_report(),
            EXPECTED_PRODUCTION_GENERATION_BOUNDARY_BODY_CLASS_COVERAGE_SUMMARY_FOR_REPORT
        );
    }

    #[test]
    fn production_generation_boundary_request_corpus_summary_for_report_byte_identical() {
        assert_eq!(
            super::production_generation_boundary_request_corpus_summary_for_report(),
            EXPECTED_PRODUCTION_GENERATION_BOUNDARY_REQUEST_CORPUS_SUMMARY_FOR_REPORT
        );
    }

    #[test]
    fn production_generation_boundary_request_corpus_equatorial_summary_for_report_byte_identical()
    {
        assert_eq!(
            super::production_generation_boundary_request_corpus_equatorial_summary_for_report(),
            EXPECTED_PRODUCTION_GENERATION_BOUNDARY_REQUEST_CORPUS_EQUATORIAL_SUMMARY_FOR_REPORT
        );
    }

    #[test]
    fn validated_production_generation_boundary_request_corpus_equatorial_summary_for_report_byte_identical(
    ) {
        assert_eq!(
            super::validated_production_generation_boundary_request_corpus_equatorial_summary_for_report(),
            Ok(EXPECTED_PRODUCTION_GENERATION_BOUNDARY_REQUEST_CORPUS_EQUATORIAL_SUMMARY_FOR_REPORT.to_string())
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_generation_snapshot_summary_reports_the_boundary_overlay() {
        let summary = pleiades_jpl::production_generation_snapshot_summary()
            .expect("production-generation snapshot summary should exist");
        summary
            .validate()
            .expect("production-generation snapshot summary should validate");
        assert_eq!(summary.row_count, 277);
        assert_eq!(summary.body_count, 16);
        assert_eq!(summary.bodies, pleiades_jpl::reference_bodies());
        assert_eq!(summary.epoch_count, 23);
        assert_eq!(summary.boundary_row_count, 66);
        assert_eq!(summary.boundary_body_count, 16);
        assert_eq!(
            summary.boundary_bodies,
            &[
                pleiades_backend::CelestialBody::Mars,
                pleiades_backend::CelestialBody::Jupiter,
                pleiades_backend::CelestialBody::Mercury,
                pleiades_backend::CelestialBody::Venus,
                pleiades_backend::CelestialBody::Saturn,
                pleiades_backend::CelestialBody::Uranus,
                pleiades_backend::CelestialBody::Neptune,
                pleiades_backend::CelestialBody::Sun,
                pleiades_backend::CelestialBody::Pluto,
                pleiades_backend::CelestialBody::Moon,
                pleiades_backend::CelestialBody::Ceres,
                pleiades_backend::CelestialBody::Pallas,
                pleiades_backend::CelestialBody::Juno,
                pleiades_backend::CelestialBody::Vesta,
                pleiades_backend::CelestialBody::Custom(pleiades_backend::CustomBodyId::new(
                    "asteroid", "433-Eros"
                )),
                pleiades_backend::CelestialBody::Custom(pleiades_backend::CustomBodyId::new(
                    "asteroid",
                    "99942-Apophis"
                )),
            ]
        );
        assert_eq!(summary.boundary_epoch_count, 12);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(
            summary.boundary_earliest_epoch.julian_day.days(),
            2_378_498.5
        );
        assert_eq!(summary.boundary_latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(summary.quarter_day_row_count, 8);
        assert_eq!(summary.quarter_day_body_count, 4);
        assert_eq!(
            summary.quarter_day_bodies,
            &[
                pleiades_backend::CelestialBody::Sun,
                pleiades_backend::CelestialBody::Moon,
                pleiades_backend::CelestialBody::Mercury,
                pleiades_backend::CelestialBody::Venus
            ]
        );
        assert_eq!(summary.quarter_day_epoch_count, 2);
        assert_eq!(
            summary.quarter_day_earliest_epoch.julian_day.days(),
            2_451_915.25
        );
        assert_eq!(
            summary.quarter_day_latest_epoch.julian_day.days(),
            2_451_915.75
        );
        let reference_bodies = format_bodies(pleiades_jpl::reference_bodies());
        let boundary_bodies = format_bodies(summary.boundary_bodies);
        let quarter_day_bodies = format_bodies(summary.quarter_day_bodies);
        assert_eq!(
            summary.summary_line(),
            format!(
                "Production generation coverage: 277 rows across 16 bodies and 23 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); bodies: {}; boundary overlay (major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Mercury and Venus at 2451545, 2451915.25, and 2451915.75, plus Jupiter, Saturn, Uranus, Neptune, and Pluto at 2451545, plus Mars at 2451545, plus Sun at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Moon at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 66 rows across 16 bodies and 12 epochs.): 66 rows across 16 bodies and 12 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); boundary bodies: {}; quarter-day boundary samples: 8 rows across 4 bodies and 2 epochs (JD 2451915.25 (TDB)..JD 2451915.75 (TDB)); quarter-day bodies: {}",
                reference_bodies,
                boundary_bodies,
                quarter_day_bodies
            )
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            production_generation_snapshot_summary_for_report(),
            summary.summary_line()
        );
        assert_eq!(
            production_generation_quarter_day_boundary_summary_for_report(),
            "Production generation quarter-day boundary samples: 8 rows across 4 bodies and 2 epochs (JD 2451915.25 (TDB)..JD 2451915.75 (TDB)); bodies: Sun, Moon, Mercury, Venus"
        );
        let production_generation_source_summary =
            production_generation_source_summary_for_report();
        assert!(production_generation_source_summary
            .contains("strategy=documented hybrid fixture corpus"));
        assert!(production_generation_source_summary.contains(
            "redistribution posture=repository-checked regression fixtures, not a broad public corpus"
        ));
        assert!(production_generation_source_summary
            .contains("source windows=277 source-backed samples across 16 bodies and 23 epochs"));
        assert!(production_generation_source_summary
            .contains("evidence classes=reference, hold-out, boundary overlay, provenance-only"));
        assert!(production_generation_source_summary
            .contains("generation command=generate-packaged-artifact --check"));
        assert!(production_generation_source_summary.contains("frame=geocentric ecliptic J2000"));
        assert!(production_generation_source_summary.contains("time scale=TDB"));
        assert!(production_generation_source_summary.contains("parser=pure-Rust and deterministic"));
    }

    #[test]
    fn production_generation_snapshot_summary_validation_rejects_quarter_day_drift() {
        let mut summary = pleiades_jpl::production_generation_snapshot_summary()
            .expect("production-generation snapshot summary should exist");
        summary.quarter_day_row_count += 1;

        let error = summary
            .validate()
            .expect_err("drifted quarter-day production-generation summary should fail validation");

        assert!(matches!(
            error,
            pleiades_jpl::ProductionGenerationSnapshotSummaryValidationError::DerivedSummaryMismatch
        ));
    }

    #[test]
    fn production_generation_snapshot_window_summary_reports_the_source_windows() {
        let summary = pleiades_jpl::production_generation_snapshot_window_summary()
            .expect("production-generation source window summary should exist");
        summary
            .validate()
            .expect("production-generation source window summary should validate");
        assert_eq!(summary.sample_count, 277);
        assert_eq!(summary.sample_bodies.len(), 16);
        assert_eq!(summary.windows.len(), summary.sample_bodies.len());
        assert_eq!(summary.sample_bodies, pleiades_jpl::reference_bodies());
        assert_eq!(summary.epoch_count, 23);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(
            summary.windows[0].body,
            pleiades_backend::CelestialBody::Ceres
        );
        assert!(summary.windows[0].sample_count >= 8);
        assert!(summary.windows[0].summary_line().starts_with("Ceres: "));
        assert!(summary.summary_line().starts_with(
            "Production generation source windows: 277 source-backed samples across 16 bodies and 23 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); windows: "
        ));
        assert!(summary.summary_line().contains("Mars:"));
        assert!(summary.summary_line().contains("Jupiter:"));
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            production_generation_snapshot_window_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn production_generation_snapshot_window_summary_validation_rejects_body_order_drift() {
        let mut summary = pleiades_jpl::production_generation_snapshot_window_summary()
            .expect("production-generation source window summary should exist");
        summary.sample_bodies.swap(0, 1);
        let error = summary
            .validate()
            .expect_err("body order drift should be rejected");
        assert!(matches!(
            error,
            pleiades_jpl::ProductionGenerationSnapshotWindowSummaryValidationError::BodyOrderMismatch { .. }
        ));
    }

    #[test]
    fn production_generation_snapshot_window_summary_validation_rejects_derived_summary_drift() {
        let mut summary = pleiades_jpl::production_generation_snapshot_window_summary()
            .expect("production-generation source window summary should exist");
        summary.sample_count += 1;
        let error = summary
            .validate()
            .expect_err("derived summary drift should be rejected");
        assert_eq!(
            error,
            pleiades_jpl::ProductionGenerationSnapshotWindowSummaryValidationError::DerivedSummaryMismatch
        );
    }

    #[test]
    fn production_generation_snapshot_body_class_coverage_summary_reports_the_split() {
        let summary = pleiades_jpl::production_generation_snapshot_body_class_coverage_summary()
            .expect("production-generation body-class coverage summary should exist");
        summary
            .validate()
            .expect("production-generation body-class coverage summary should validate");
        assert_eq!(summary.row_count, 277);
        assert_eq!(summary.major_bodies.len(), 10);
        assert_eq!(summary.asteroid_bodies.len(), 6);
        assert!(summary
            .summary_line()
            .starts_with("Production generation body-class coverage: major bodies: "));
        assert!(summary.summary_line().contains("selected asteroids: "));
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            production_generation_snapshot_body_class_coverage_summary_for_report(),
            summary.summary_line()
        );
        assert_eq!(
            validated_production_generation_snapshot_body_class_coverage_summary_for_report(),
            Ok(summary.summary_line())
        );
    }

    #[test]
    fn production_generation_snapshot_body_class_coverage_summary_validation_rejects_major_body_drift(
    ) {
        let mut summary =
            pleiades_jpl::production_generation_snapshot_body_class_coverage_summary()
                .expect("production-generation body-class coverage summary should exist");
        summary.major_bodies.pop();

        assert_eq!(
            summary.validate(),
            Err(
                pleiades_jpl::ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "major_bodies",
                }
            )
        );
    }

    #[test]
    fn production_generation_snapshot_summary_reports_the_expected_coverage() {
        let summary = pleiades_jpl::production_generation_snapshot_summary()
            .expect("production generation summary should exist");
        assert_eq!(summary.row_count, 277);
        assert_eq!(summary.body_count, 16);
        assert_eq!(summary.epoch_count, 23);
        assert_eq!(summary.boundary_row_count, 66);
        assert_eq!(summary.boundary_body_count, 16);
        assert_eq!(summary.boundary_epoch_count, 12);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            production_generation_snapshot_summary_for_report(),
            summary.summary_line()
        );
        assert!(summary.summary_line().contains("boundary overlay (major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Mercury and Venus at 2451545, 2451915.25, and 2451915.75, plus Jupiter, Saturn, Uranus, Neptune, and Pluto at 2451545, plus Mars at 2451545, plus Sun at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Moon at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 66 rows across 16 bodies and 12 epochs.): 66 rows across 16 bodies and 12 epochs"));
    }

    #[test]
    fn production_generation_source_summary_documents_the_checked_in_csv_path() {
        let summary = pleiades_jpl::production_generation_source_summary();
        let report = production_generation_source_summary_for_report();

        assert!(summary.validate().is_ok());
        assert!(report.contains("strategy=documented hybrid fixture corpus"));
        assert!(report.contains(
            "input path=checked-in CSV fixtures via include_str! reference_snapshot.csv and independent_holdout_snapshot.csv"
        ));
        assert!(report.contains("file format=comma-separated values"));
        assert!(report.contains("frame=geocentric ecliptic J2000"));
        assert!(report.contains("time scale=TDB"));
        assert!(report.contains("apparentness=Mean"));
        assert!(report.contains("parser=pure-Rust and deterministic"));
        assert!(report.contains("source revision=reference_snapshot.csv checksum=0x"));
        assert!(report.contains("evidence class=reference"));
        assert!(
            report.contains("reference snapshot exact J2000 evidence=16 exact J2000 samples at")
        );
        assert!(report
            .contains("evidence classes=reference, hold-out, boundary overlay, provenance-only"));
        assert!(report.contains("independent_holdout_snapshot.csv checksum=0x"));
        assert!(report
            .contains("source windows=277 source-backed samples across 16 bodies and 23 epochs"));
        assert!(report.contains("license posture=public-source provenance only; checked-in fixtures remain repository-local regression data"));
        assert!(report.contains("generation command=generate-packaged-artifact --check"));
        assert!(report.contains("checksum expectation=byte-identical fixture contents"));
        let expected_cadence = format!(
            "cadence={} reference epochs and {} boundary epochs",
            summary.source_windows.epoch_count,
            pleiades_jpl::production_generation_boundary_request_corpus_summary(
                CoordinateFrame::Ecliptic
            )
            .expect("production generation boundary request corpus should exist")
            .epoch_count,
        );
        assert!(report.contains(&expected_cadence));
        let body_class_coverage =
            pleiades_jpl::production_generation_snapshot_body_class_coverage_summary()
                .expect("production generation body-class coverage should exist");
        let boundary_body_class_coverage =
            pleiades_jpl::production_generation_boundary_body_class_coverage_summary()
                .expect("production generation boundary body-class coverage should exist");
        let expected_body_class_cadence = format!(
            "body-class cadence=reference major bodies: {} epochs; reference selected asteroids: {} epochs; boundary major bodies: {} epochs; boundary selected asteroids: {} epochs",
            body_class_coverage.major_epoch_count,
            body_class_coverage.asteroid_epoch_count,
            boundary_body_class_coverage.major_epoch_count,
            boundary_body_class_coverage.asteroid_epoch_count,
        );
        assert!(report.contains(&expected_body_class_cadence));
        assert!(report.contains("reference and hold-out rows remain separate"));
        assert!(report.contains("schema=epoch_jd, body, x_km, y_km, z_km"));
        assert!(report.contains("columns=epoch_jd, body, x_km, y_km, z_km"));
        assert!(report.contains(
            "redistribution posture=repository-checked regression fixtures, not a broad public corpus"
        ));
    }

    #[test]
    fn production_generation_source_summary_validated_report_matches_current_rendering() {
        let report = production_generation_source_summary_for_report();
        let validated = validated_production_generation_source_summary_for_report()
            .expect("validated production generation source summary should exist");

        assert_eq!(validated, report);
    }

    #[test]
    fn production_generation_source_cadence_fragment_rejects_boundary_epoch_count_drift() {
        let error = production_generation_source_cadence_fragment_from_counts(31, 13, 12)
            .expect_err("mismatched boundary epoch counts should be rejected");

        assert!(matches!(
            error,
            ProductionGenerationSourceSummaryValidationError::BoundaryRequestCorpusEpochCountMismatch {
                ecliptic_epoch_count: 13,
                equatorial_epoch_count: 12,
            }
        ));
        assert_eq!(
            error.to_string(),
            "boundary request corpus epoch counts differ: ecliptic=13, equatorial=12"
        );
    }

    #[test]
    fn production_generation_source_revision_summary_documents_fixture_checksums() {
        let summary = pleiades_jpl::production_generation_source_revision_summary();
        let report = production_generation_source_revision_summary_for_report();
        let validated = validated_production_generation_source_revision_summary_for_report()
            .expect("validated production generation source revision summary should exist");

        assert_eq!(report, summary.summary_line());
        assert_eq!(validated, report);
        assert!(report.contains("reference_snapshot.csv checksum=0x"));
        assert!(report.contains("independent_holdout_snapshot.csv checksum=0x"));
    }

    #[test]
    fn production_generation_source_revision_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::production_generation_source_revision_summary();
        summary.reference_snapshot_checksum ^= 1;

        assert!(matches!(
            summary.validate(),
            Err(
                pleiades_jpl::ProductionGenerationSourceRevisionSummaryValidationError::FieldOutOfSync {
                    field: "summary"
                }
            )
        ));
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn production_generation_manifest_summary_documents_the_current_contract() {
        let summary = pleiades_jpl::production_generation_manifest_summary()
            .expect("production generation manifest summary should exist");
        let report = production_generation_manifest_summary_for_report();

        assert!(summary.validate().is_ok());
        assert_eq!(summary.validated_summary_line().unwrap(), report);
        assert!(report.contains("Production generation manifest: coverage="));
        assert!(report.contains("source="));
        assert!(report.contains("body-class coverage="));
        assert!(report.contains("boundary overlay="));
        assert!(report.contains("boundary windows="));
        assert!(report.contains("boundary request corpus="));
    }

    #[test]
    fn production_generation_manifest_summary_validated_report_matches_current_rendering() {
        assert_eq!(
            validated_production_generation_manifest_summary_for_report()
                .expect("validated production generation manifest summary should exist"),
            production_generation_manifest_summary_for_report(),
        );
    }

    #[test]
    fn production_generation_manifest_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::production_generation_manifest_summary()
            .expect("production generation manifest summary should exist");
        summary.boundary_request_corpus_summary.epoch_count += 1;

        assert!(matches!(
            summary.validate(),
            Err(
                pleiades_jpl::ProductionGenerationManifestSummaryValidationError::BoundaryRequestCorpus(
                    pleiades_jpl::ProductionGenerationBoundaryRequestCorpusSummaryValidationError::FieldOutOfSync {
                        field: "epoch_count"
                    }
                )
            )
        ));
        assert!(summary.validated_summary_line().is_err());
    }

    #[test]
    fn production_generation_corpus_shape_summary_documents_the_current_contract() {
        let summary = pleiades_jpl::production_generation_corpus_shape_summary()
            .expect("production generation corpus shape summary should exist");
        let report = production_generation_corpus_shape_summary_for_report();

        assert!(summary.validate().is_ok());
        assert!(report.contains("Production generation corpus shape: source="));
        assert!(report.contains("boundary request corpora: ecliptic="));
        assert!(report.contains("equatorial="));
        assert!(report.contains(
            "validated fields=body order, epochs, frame, time scale, columns, apparentness, checksums"
        ));
        assert!(report.contains("columns=epoch_jd, body, x_km, y_km, z_km"));
        assert!(report.contains("frame=geocentric ecliptic J2000"));
        assert!(report.contains("time scale=TDB"));
        assert!(report.contains("apparentness=Mean"));
    }

    #[test]
    fn production_generation_corpus_shape_summary_validated_report_matches_current_rendering() {
        let report = production_generation_corpus_shape_summary_for_report();
        let validated = validated_production_generation_corpus_shape_summary_for_report()
            .expect("validated production generation corpus shape summary should exist");

        assert_eq!(validated, report);
    }

    #[test]
    fn production_generation_corpus_shape_summary_validation_rejects_drift() {
        let mut summary = pleiades_jpl::production_generation_corpus_shape_summary()
            .expect("production generation corpus shape summary should exist");
        summary.boundary_request_corpus_equatorial.apparentness =
            pleiades_backend::Apparentness::Apparent;

        assert!(matches!(
            summary.validate(),
            Err(
                pleiades_jpl::ProductionGenerationCorpusShapeSummaryValidationError::BoundaryRequestCorpusEquatorial(
                    _
                )
            )
        ));
    }

    // --- Task 10a fold-in: 10a left `production_generation_manifest_checksum_for_report`
    // without an equality test against jpl's still-present renderer. ---

    // jpl's `production_generation_manifest_checksum_for_report` was deleted
    // in the Task 14 contract sweep. `EXPECTED_PRODUCTION_GENERATION_MANIFEST_CHECKSUM_FOR_REPORT`
    // is a byte-exact capture of jpl's renderer output taken immediately
    // before deletion (Slice D Task 14a).
    const EXPECTED_PRODUCTION_GENERATION_MANIFEST_CHECKSUM_FOR_REPORT: &str =
        r"Production generation manifest checksum: 0xe60d112616ab2284";

    #[test]
    fn production_generation_manifest_checksum_for_report_byte_identical() {
        assert_eq!(
            super::production_generation_manifest_checksum_for_report(),
            EXPECTED_PRODUCTION_GENERATION_MANIFEST_CHECKSUM_FOR_REPORT
        );
    }

    // --- Task 10b: copied report tests from
    // `reference_summary/production_generation/tests.rs` targeting the 7
    // top-level `production_generation.rs` renderers. Verbatim, with bare
    // jpl symbol calls rewired to `pleiades_jpl::` and `_for_report` calls
    // rewired to `super::` (this file's local copies). ---

    #[test]
    fn production_generation_boundary_summary_reports_the_overlay() {
        let summary = pleiades_jpl::production_generation_boundary_summary()
            .expect("production-generation boundary summary should exist");
        summary
            .validate()
            .expect("production-generation boundary summary should validate");
        assert_eq!(summary.row_count, 66);
        assert_eq!(summary.body_count, 16);
        assert_eq!(
            summary.bodies,
            pleiades_jpl::production_generation_boundary_body_list()
        );
        assert_eq!(summary.epoch_count, 12);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(
            summary.summary_line(),
            "Production generation boundary overlay: 66 rows across 16 bodies and 12 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); bodies: Mars, Jupiter, Mercury, Venus, Saturn, Uranus, Neptune, Sun, Pluto, Moon, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis"
        );
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            production_generation_boundary_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn production_generation_boundary_source_summary_reports_the_overlay_provenance() {
        let boundary_summary = pleiades_jpl::production_generation_boundary_source_summary();
        let holdout_summary = pleiades_jpl::independent_holdout_source_summary();
        boundary_summary
            .validate()
            .expect("production-generation boundary source summary should validate");
        holdout_summary
            .validate()
            .expect("independent hold-out source summary should validate");
        assert_eq!(boundary_summary.source, holdout_summary.source);
        assert_eq!(
            boundary_summary.evidence_class,
            holdout_summary.evidence_class
        );
        assert_eq!(boundary_summary.coverage, holdout_summary.coverage);
        assert_eq!(boundary_summary.columns, holdout_summary.columns);
        assert_eq!(
            boundary_summary.redistribution,
            holdout_summary.redistribution
        );
        assert_eq!(
            boundary_summary.frame_treatment,
            holdout_summary.frame_treatment
        );
        assert_eq!(boundary_summary.time_scale, holdout_summary.time_scale);
        assert_eq!(
            format_production_generation_boundary_source_summary(&boundary_summary),
            production_generation_boundary_source_summary_for_report()
        );
        assert!(production_generation_boundary_source_summary_for_report().contains(
            "Production generation boundary overlay source: NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables."
        ));
        assert!(production_generation_boundary_source_summary_for_report().contains(
            "selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167"
        ));
        assert!(production_generation_boundary_source_summary_for_report()
            .contains("asteroid:99942-Apophis now also appears at 2378498.5"));
    }

    #[test]
    fn production_generation_boundary_window_summary_reports_the_overlay_windows() {
        let summary = pleiades_jpl::production_generation_boundary_window_summary()
            .expect("production-generation boundary window summary should exist");
        assert_eq!(summary.sample_count, 66);
        assert_eq!(summary.sample_bodies.len(), 16);
        assert_eq!(
            summary.sample_bodies,
            pleiades_jpl::production_generation_boundary_body_list().to_vec()
        );
        assert_eq!(summary.epoch_count, 12);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
        assert_eq!(
            summary.windows[0].body,
            pleiades_backend::CelestialBody::Mars
        );
        assert_eq!(summary.windows[0].sample_count, 5);
        assert_eq!(summary.windows[0].epoch_count, 5);
        assert_eq!(
            summary.windows[0].summary_line(),
            format!(
                "Mars: 5 samples across 5 epochs at {}..{}",
                format_instant(summary.windows[0].earliest_epoch),
                format_instant(summary.windows[0].latest_epoch)
            )
        );
        assert!(summary.summary_line().starts_with("Production generation boundary windows: 66 source-backed samples across 16 bodies and 12 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); windows: "));
        assert!(summary.summary_line().contains("Mars: 5 samples across 5 epochs at JD 2451545.0 (TDB)..JD 2451915.5 (TDB); Jupiter: 5 samples across 5 epochs at JD 2451545.0 (TDB)..JD 2451915.5 (TDB)"));
        assert_eq!(summary.summary_line(), summary.to_string());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            production_generation_boundary_window_summary_for_report(),
            summary.summary_line()
        );

        let mut drifted = summary.clone();
        drifted.sample_count += 1;
        assert!(drifted.validated_summary_line().is_err());
    }

    #[test]
    fn production_generation_boundary_body_class_coverage_summary_reports_the_overlay_body_classes()
    {
        let summary = pleiades_jpl::production_generation_boundary_body_class_coverage_summary()
            .expect("production-generation boundary body-class coverage summary should exist");
        summary
            .validate()
            .expect("production-generation boundary body-class coverage summary should validate");
        assert_eq!(summary.row_count, 66);
        assert_eq!(summary.major_body_row_count, 34);
        assert_eq!(summary.major_bodies.len(), 10);
        assert_eq!(
            summary.major_bodies,
            vec![
                pleiades_backend::CelestialBody::Mars,
                pleiades_backend::CelestialBody::Jupiter,
                pleiades_backend::CelestialBody::Mercury,
                pleiades_backend::CelestialBody::Venus,
                pleiades_backend::CelestialBody::Saturn,
                pleiades_backend::CelestialBody::Uranus,
                pleiades_backend::CelestialBody::Neptune,
                pleiades_backend::CelestialBody::Sun,
                pleiades_backend::CelestialBody::Pluto,
                pleiades_backend::CelestialBody::Moon,
            ]
        );
        assert_eq!(summary.major_epoch_count, 7);
        assert_eq!(summary.major_windows.len(), 10);
        assert_eq!(summary.asteroid_row_count, 32);
        assert_eq!(summary.asteroid_bodies.len(), 6);
        assert_eq!(summary.asteroid_epoch_count, 7);
        assert_eq!(summary.asteroid_windows.len(), 6);
        assert!(summary.summary_line().starts_with(
            "Production generation boundary body-class coverage: major bodies: 34 rows across 10 bodies and 7 epochs; major windows: "
        ));
        assert!(summary
            .summary_line()
            .contains(&summary.major_windows[0].summary_line()));
        assert!(summary
            .summary_line()
            .contains(&summary.major_windows[2].summary_line()));
        assert!(summary.summary_line().contains(
            "selected asteroids: 32 rows across 6 bodies and 7 epochs; asteroid windows: "
        ));
        assert_eq!(summary.summary_line(), summary.to_string());
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(
            production_generation_boundary_body_class_coverage_summary_for_report(),
            summary.summary_line()
        );

        let mut drifted = summary.clone();
        drifted.row_count += 1;
        assert!(drifted.validated_summary_line().is_err());
    }

    #[test]
    fn production_generation_boundary_request_corpus_summary_reports_the_expected_coverage() {
        let summary = pleiades_jpl::production_generation_boundary_request_corpus_summary(
            CoordinateFrame::Ecliptic,
        )
        .expect("production generation boundary request corpus summary should exist");
        assert_eq!(summary.request_count, 66);
        assert_eq!(summary.body_count, 16);
        assert_eq!(summary.epoch_count, 12);
        assert_eq!(summary.frame, CoordinateFrame::Ecliptic);
        assert_eq!(summary.time_scale, pleiades_types::TimeScale::Tdb);
        assert_eq!(summary.zodiac_mode, pleiades_types::ZodiacMode::Tropical);
        assert_eq!(summary.apparentness, pleiades_backend::Apparentness::Mean);
        assert_eq!(summary.validate(), Ok(()));
        assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            production_generation_boundary_request_corpus_summary_for_report(),
            summary.summary_line()
        );
        assert_eq!(
            production_generation_boundary_request_corpus_equatorial_summary_for_report(),
            pleiades_jpl::production_generation_boundary_request_corpus_summary(
                CoordinateFrame::Equatorial
            )
            .expect("production generation boundary request corpus equatorial summary should exist")
            .summary_line()
        );
        assert_eq!(
            validated_production_generation_boundary_request_corpus_equatorial_summary_for_report(),
            Ok(
                pleiades_jpl::production_generation_boundary_request_corpus_summary(
                    CoordinateFrame::Equatorial
                )
                .expect(
                    "production generation boundary request corpus equatorial summary should exist"
                )
                .summary_line()
            )
        );
        assert!(summary
            .summary_line()
            .contains("observerless) across 16 bodies and 12 epochs"));
    }
}
