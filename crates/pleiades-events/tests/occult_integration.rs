//! SP-6 occultation integration invariants over the real routing chain.

use pleiades_apparent::Atmosphere;
use pleiades_data::packaged_backend;
use pleiades_events::{EventEngine, OccultTarget, OccultationType};
use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
// Note: `CelestialBody` is not imported here (unlike the task brief's draft) because
// none of these tests exercise `OccultTarget::Body`; an unused import would fail
// the crate's warning-free clippy gate.

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}
fn observer() -> ObserverLocation {
    ObserverLocation::new(Latitude::from_degrees(40.0), Longitude::from_degrees(-3.7), Some(650.0))
}

#[test]
fn next_planet_occultation_has_ordered_contacts() {
    let engine = EventEngine::new(packaged_backend());
    // Aldebaran is occulted by the Moon repeatedly during 1900-2100; find one.
    // Verified (via a temporary --nocapture debug print, since removed) that the
    // default start of tdb(2_451_545.0) (2000-01-01) is NOT vacuous: this search
    // lands on a real, locally-visible occultation at JD 2457270.720129775
    // (~2015-08-30 TDB), part of the well-known 2015-2018 Aldebaran occultation
    // series, so the `Some` branch below genuinely exercises the C1<=max<=C4 and
    // Total/magnitude invariants -- no epoch adjustment was needed.
    let out = engine
        .next_occultation(
            OccultTarget::Star("Aldebaran".into()),
            observer(),
            Atmosphere::default(),
            tdb(2_451_545.0),
        )
        .unwrap();
    assert!(
        out.is_some(),
        "expected a real Aldebaran occultation near 2015-08-30 (JD 2457270.72); if this now \
         returns None, investigate before assuming the test is still meaningful (see SP-6 Task 8)"
    );
    if let Some(o) = out {
        let c1 = o.first_contact.instant.julian_day.days();
        let mx = o.maximum.instant.julian_day.days();
        let c4 = o.fourth_contact.instant.julian_day.days();
        assert!(c1 <= mx + 1e-9 && mx <= c4 + 1e-9, "C1<=max<=C4: {c1} {mx} {c4}");
        assert!(!matches!(o.occultation_type, OccultationType::Miss));
        assert!(o.magnitude >= 0.0 && o.obscuration >= 0.0 && o.obscuration <= 1.0);
        // A point star that is occulted is Total with magnitude 1.
        if matches!(o.occultation_type, OccultationType::Total) {
            assert!((o.magnitude - 1.0).abs() < 1e-6);
            assert!(o.second_contact.is_none(), "a point star has no interior contact");
        }
    }
    // If None (no locally-visible Aldebaran occultation in span), the search still
    // terminated cleanly -- that is the invariant under test for un-found cases.
}

#[test]
fn global_occultation_reports_finite_sublunar_point() {
    let engine = EventEngine::new(packaged_backend());
    // Verified (via a temporary --nocapture debug print, since removed) that the
    // default start of tdb(2_451_545.0) finds a real global occultation at
    // JD 2451561.306545391 (~2000-01-17 TDB), roughly 16 days after the start --
    // the `Some` branch below genuinely exercises the sub-lunar-point invariants.
    let out = engine
        .next_global_occultation(OccultTarget::Star("Aldebaran".into()), tdb(2_451_545.0))
        .unwrap();
    assert!(
        out.is_some(),
        "expected a real global Aldebaran occultation near 2000-01-17 (JD 2451561.31); if this \
         now returns None, investigate before assuming the test is still meaningful (see SP-6 \
         Task 8)"
    );
    if let Some(g) = out {
        assert!(g.sublunar_latitude.degrees().is_finite() && g.sublunar_latitude.degrees().abs() <= 90.0);
        // `Longitude::from_degrees` normalizes into [0, 360) (see
        // `pleiades_types::Longitude::degrees`'s own doc) -- that is the
        // type's actual invariant, not the signed (-180, 180] range a prior
        // version of this assertion assumed (which only happened to hold for
        // this particular occultation's OLD, buggy sub-Moon-zenith
        // longitude; Task 15's central-observation-point fix legitimately
        // moves the reported point, including into the (180, 360) half).
        let lon = g.sublunar_longitude.degrees();
        assert!(lon.is_finite() && (0.0..360.0).contains(&lon));
        assert!(g.maximum.julian_day.days() > 2_451_545.0);
    }
}

#[test]
fn ingress_and_egress_are_symmetric_about_maximum() {
    let engine = EventEngine::new(packaged_backend());
    // Verified (via a temporary --nocapture debug print, since removed) that this
    // hits the same real occultation as `next_planet_occultation_has_ordered_contacts`
    // above (JD 2457270.720129775, ~2015-08-30 TDB), so the symmetry assertions
    // below genuinely run against real ingress/egress geometry.
    let out = engine
        .next_occultation(
            OccultTarget::Star("Aldebaran".into()),
            observer(),
            Atmosphere::default(),
            tdb(2_451_545.0),
        )
        .unwrap();
    assert!(
        out.is_some(),
        "expected a real Aldebaran occultation near 2015-08-30 (JD 2457270.72); if this now \
         returns None, investigate before assuming the test is still meaningful (see SP-6 Task 8)"
    );
    if let Some(o) = out {
        if !matches!(o.occultation_type, OccultationType::Miss) {
            let pre = o.maximum.instant.julian_day.days() - o.first_contact.instant.julian_day.days();
            let post = o.fourth_contact.instant.julian_day.days() - o.maximum.instant.julian_day.days();
            // Chord halves need not be exactly equal, but both are positive and
            // of the same order (within 3x) for a genuine occultation.
            assert!(pre > 0.0 && post > 0.0, "positive half-chords: {pre} {post}");
            assert!(pre < 3.0 * post + 1e-9 && post < 3.0 * pre + 1e-9);
        }
    }
}

#[test]
fn sirius_never_occulted_terminates_with_none() {
    let engine = EventEngine::new(packaged_backend());
    let out = engine
        .next_occultation(
            OccultTarget::Star("Sirius".into()),
            observer(),
            Atmosphere::default(),
            tdb(2_451_545.0),
        )
        .unwrap();
    assert!(out.is_none(), "Sirius (~39 deg S ecliptic latitude) is never occultable");
}
