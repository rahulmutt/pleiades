//! corpus builder and summary tests (white-box; moved verbatim from the former `tests.rs`).

use super::*;
use pleiades_core::{Apparentness, CoordinateFrame, TimeScale};
use pleiades_jpl::comparison_bodies;

#[test]
fn default_corpus_covers_the_comparison_snapshot() {
    let corpus = default_corpus();
    let summary = corpus.summary();
    assert_eq!(corpus.requests.len(), 232);
    assert_eq!(summary.epoch_count, 28);
    assert_eq!(summary.epochs.len(), 28);
    assert!(summary
        .epochs
        .iter()
        .all(|epoch| epoch.scale == TimeScale::Tt));
    assert_eq!(summary.epochs[0].julian_day.days(), 2_268_932.5);
    assert_eq!(summary.body_count, comparison_bodies().len());
    assert!(corpus
        .requests
        .iter()
        .all(|request| request.instant.scale == TimeScale::Tt));
    assert!(corpus.requests.iter().all(|request| matches!(
        request.body,
        CelestialBody::Sun
            | CelestialBody::Moon
            | CelestialBody::Mercury
            | CelestialBody::Venus
            | CelestialBody::Mars
            | CelestialBody::Jupiter
            | CelestialBody::Saturn
            | CelestialBody::Uranus
            | CelestialBody::Neptune
            | CelestialBody::Pluto
    )));
    assert!(corpus
        .requests
        .iter()
        .any(|request| request.instant.julian_day.days() == 2_360_233.5));
    assert!(corpus
        .requests
        .iter()
        .any(|request| request.instant.julian_day.days() == 2_451_545.0));
    assert!(corpus
        .requests
        .iter()
        .any(|request| request.instant.julian_day.days() == 2_634_167.0));
    assert_eq!(corpus.requests[0].frame, CoordinateFrame::Ecliptic);
    assert_eq!(corpus.apparentness, Apparentness::Mean);
    assert_eq!(corpus.requests[0].apparent, Apparentness::Mean);
}

#[test]
fn release_grade_corpus_excludes_pluto_from_tolerance_evidence() {
    let corpus = release_grade_corpus();
    let summary = corpus.summary();
    assert!(corpus
        .requests
        .iter()
        .all(|request| request.body != CelestialBody::Pluto));
    assert!(summary.body_count < default_corpus().summary().body_count);
    assert!(summary.request_count < default_corpus().summary().request_count);
    assert!(corpus.name.contains("release-grade comparison window"));
    assert!(corpus
        .description
        .contains("Pluto excluded from tolerance evidence"));
    assert!(!corpus
        .summary()
        .epochs
        .iter()
        .any(|epoch| epoch.julian_day.days() == 2_451_913.5));
}

#[test]
fn benchmark_corpus_spans_the_target_window() {
    let corpus = benchmark_corpus();
    let summary = corpus.summary();
    assert_eq!(summary.epoch_count, 11);
    assert_eq!(summary.body_count, default_chart_bodies().len());
    assert_eq!(summary.request_count, 110);
    assert_eq!(summary.apparentness, Apparentness::Mean);
    assert!(summary.earliest_julian_day < summary.latest_julian_day);
}

#[test]
fn corpus_summary_has_a_displayable_summary_line() {
    let summary = benchmark_corpus().summary();
    assert_eq!(summary.summary_line(), summary.to_string());
    assert!(summary
        .summary_line()
        .contains("corpus name=Representative 1600-2600 window"));
    assert!(summary.summary_line().contains("requests=110"));
    assert!(summary.summary_line().contains("epochs=11"));
    assert!(summary.summary_line().contains("bodies="));
}

#[test]
fn corpus_summary_validate_accepts_the_reported_summary() {
    let summary = benchmark_corpus().summary();
    summary
        .validate()
        .expect("reported corpus summary should validate");
}

#[test]
fn corpus_summary_validate_rejects_epoch_order_drift() {
    let mut summary = benchmark_corpus().summary();
    summary.epochs.reverse();
    let error = summary
        .validate()
        .expect_err("mutated corpus summary should fail validation");
    assert!(
        error.to_string().contains("out of order") || error.to_string().contains("first epoch")
    );
}

#[test]
fn packaged_benchmark_corpus_uses_packaged_artifact_coverage() {
    let corpus = artifact::packaged_artifact_corpus();
    let summary = corpus.summary();
    assert!(summary.name.contains("Packaged artifact"));
    assert_eq!(summary.apparentness, Apparentness::Mean);
    assert_eq!(
        summary.body_count,
        pleiades_data::packaged_artifact().bodies.len()
    );
    assert!(summary.request_count > 0);
    assert!(summary.earliest_julian_day <= summary.latest_julian_day);
}
