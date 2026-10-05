//! A placement whose apparent reduction fails keeps its mean place, and the
//! snapshot says so (issue #170). `ChartSnapshot::apparentness` echoes the
//! request; before this, a caller reading only that field, `summary_line()` or
//! `Display` saw a chart that looked fully apparent.

use pleiades_backend::Apparentness;
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};

use super::test_support::AbsurdDistanceReleaseGradeBackend;
use crate::chart::{ChartEngine, ChartRequest, ChartSnapshot};

/// A chart from a backend whose Mars distance trips the light-time sanity cap,
/// so Mars falls back to its mean place while the Sun is reduced.
fn chart(bodies: Vec<CelestialBody>, apparentness: Apparentness) -> ChartSnapshot {
    ChartEngine::new(AbsurdDistanceReleaseGradeBackend)
        .chart(
            &ChartRequest::new(Instant::new(
                JulianDay::from_days(2_451_545.0),
                TimeScale::Tt,
            ))
            .with_bodies(bodies)
            .with_apparentness(apparentness),
        )
        .expect("chart succeeds")
}

fn fallback_bodies(snapshot: &ChartSnapshot) -> Vec<CelestialBody> {
    snapshot
        .mean_fallback_placements()
        .map(|placement| placement.body.clone())
        .collect()
}

#[test]
fn a_failed_reduction_is_visible_at_the_snapshot_level() {
    let snapshot = chart(
        vec![CelestialBody::Sun, CelestialBody::Mars],
        Apparentness::Apparent,
    );
    assert_eq!(snapshot.apparentness, Apparentness::Apparent);
    assert_eq!(fallback_bodies(&snapshot), vec![CelestialBody::Mars]);
    assert_eq!(snapshot.apparentness_applied(), Apparentness::Mean);

    let label = "Apparent (1 of 2 placements reduced)";
    let summary = snapshot.summary_line();
    assert!(
        summary.contains(&format!("apparentness={label};")),
        "{summary}"
    );
    let rendered = snapshot.to_string();
    assert!(
        rendered.contains(&format!("Apparentness: {label}\n")),
        "{rendered}"
    );
}

#[test]
fn a_fully_reduced_chart_reads_as_apparent() {
    let snapshot = chart(vec![CelestialBody::Sun], Apparentness::Apparent);
    assert_eq!(fallback_bodies(&snapshot), vec![]);
    assert_eq!(snapshot.apparentness_applied(), Apparentness::Apparent);
    assert!(snapshot.summary_line().contains("apparentness=Apparent;"));
    assert!(snapshot.to_string().contains("Apparentness: Apparent\n"));
}

#[test]
fn a_mean_chart_has_no_fallbacks() {
    // Every placement is mean because that was asked for, not because a
    // reduction failed.
    let snapshot = chart(
        vec![CelestialBody::Sun, CelestialBody::Mars],
        Apparentness::Mean,
    );
    assert_eq!(fallback_bodies(&snapshot), vec![]);
    assert_eq!(snapshot.apparentness_applied(), Apparentness::Mean);
    assert!(snapshot.summary_line().contains("apparentness=Mean;"));
    assert!(snapshot.to_string().contains("Apparentness: Mean\n"));
}

#[test]
fn an_apparent_chart_without_placements_reads_as_apparent() {
    let snapshot = chart(Vec::new(), Apparentness::Apparent);
    assert_eq!(snapshot.apparentness_applied(), Apparentness::Apparent);
    assert!(snapshot.summary_line().contains("apparentness=Apparent;"));
}
