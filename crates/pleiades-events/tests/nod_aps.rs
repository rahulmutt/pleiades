//! nod_aps integration over the production-style backend chain.
//!
//! Asteroid coverage bound: no asteroid in the offline chain supports
//! `nod_aps`'s osculating sampling today. Snapshot-only bodies (Ceres, Pallas,
//! Juno, Vesta, asteroid:99942-Apophis) sit on sparse regression fixtures;
//! asteroid:433-Eros has a continuous packaged fit whose positions are exact
//! at corpus epochs but whose time-derivative is non-physical at nod_aps's
//! sub-day sampling scale (the 180-day corpus cadence undersamples the
//! ~643-day orbit). Both classes fail closed with a typed error — pinned
//! below as the correct production behavior.

use pleiades_backend::{CompositeBackend, RoutingBackend};
use pleiades_data::PackagedDataBackend;
use pleiades_elp::ElpBackend;
use pleiades_events::{ApsisConvention, EventEngine, EventError, NodApsMethod};
use pleiades_fict::FictitiousBackend;
use pleiades_jpl::JplSnapshotBackend;
use pleiades_types::{CelestialBody, CustomBodyId, Instant, JulianDay, TimeScale};
use pleiades_vsop87::Vsop87Backend;

fn engine() -> EventEngine<RoutingBackend> {
    EventEngine::new(RoutingBackend::new(vec![
        Box::new(PackagedDataBackend::new()),
        Box::new(CompositeBackend::new(
            Vsop87Backend::new(),
            ElpBackend::new(),
        )),
        Box::new(JplSnapshotBackend::new()),
        Box::new(FictitiousBackend::new(PackagedDataBackend::new())),
    ]))
}

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

const JD: f64 = 2_451_545.0; // J2000

#[test]
fn planet_nodes_lie_near_the_ecliptic_plane() {
    let engine = engine();
    for body in [
        CelestialBody::Mercury,
        CelestialBody::Mars,
        CelestialBody::Saturn,
    ] {
        for method in [NodApsMethod::Mean, NodApsMethod::Osculating] {
            let r = engine
                .nod_aps(body.clone(), tdb(JD), method, ApsisConvention::Aphelion)
                .unwrap();
            // Node points sit in the ecliptic plane; the geocentric direction
            // picks up only Earth's tiny ecliptic latitude.
            assert!(r.ascending.latitude_deg.abs() < 0.1, "{body:?} {method:?}");
            assert!(r.descending.latitude_deg.abs() < 0.1, "{body:?} {method:?}");
            assert!(r.perihelion.distance_au.is_finite() && r.perihelion.distance_au > 0.0);
        }
    }
}

#[test]
fn moon_mean_node_matches_the_backend_mean_node_longitude() {
    let engine = engine();
    let r = engine
        .nod_aps(
            CelestialBody::Moon,
            tdb(JD),
            NodApsMethod::Mean,
            ApsisConvention::Aphelion,
        )
        .unwrap();
    // Mean lunar node at J2000 ≈ 125.04° (Meeus); allow frame/nutation slack.
    assert!(
        (r.ascending.longitude_deg - 125.04).abs() < 1.0,
        "{}",
        r.ascending.longitude_deg
    );
    // Mean node regresses ≈ −0.0529 deg/day.
    assert!((r.ascending.longitude_speed_deg_per_day + 0.0529).abs() < 0.01);
    // Perigee distance ≈ a(1−e) with SE's mean scalars.
    assert!((r.perihelion.distance_au - 0.00256955 * (1.0 - 0.054900489)).abs() < 1e-5);
}

#[test]
fn moon_osculating_node_is_near_the_true_node() {
    let engine = engine();
    let r = engine
        .nod_aps(
            CelestialBody::Moon,
            tdb(JD),
            NodApsMethod::Osculating,
            ApsisConvention::Aphelion,
        )
        .unwrap();
    // The osculating node oscillates around the mean node within ~±2°.
    assert!((pleiades_events_test_wrap(r.ascending.longitude_deg, 125.0)).abs() < 3.0);
}

fn pleiades_events_test_wrap(a: f64, b: f64) -> f64 {
    let mut d = (a - b).rem_euclid(360.0);
    if d > 180.0 {
        d -= 360.0;
    }
    d
}

#[test]
fn second_focus_scales_the_moon_apogee_distance() {
    let engine = engine();
    let apo = engine
        .nod_aps(
            CelestialBody::Moon,
            tdb(JD),
            NodApsMethod::Mean,
            ApsisConvention::Aphelion,
        )
        .unwrap()
        .aphelion;
    let foc = engine
        .nod_aps(
            CelestialBody::Moon,
            tdb(JD),
            NodApsMethod::Mean,
            ApsisConvention::SecondFocus,
        )
        .unwrap()
        .aphelion;
    let e = 0.054900489_f64;
    assert!((foc.distance_au / apo.distance_au - 2.0 * e / (1.0 + e)).abs() < 1e-9);
    assert!((foc.longitude_deg - apo.longitude_deg).abs() < 1e-9);
}

#[test]
fn barycentric_falls_back_heliocentric_inside_six_au() {
    let engine = engine();
    let oscu = engine
        .nod_aps(
            CelestialBody::Mars,
            tdb(JD),
            NodApsMethod::Osculating,
            ApsisConvention::Aphelion,
        )
        .unwrap();
    let bar = engine
        .nod_aps(
            CelestialBody::Mars,
            tdb(JD),
            NodApsMethod::OsculatingBarycentric,
            ApsisConvention::Aphelion,
        )
        .unwrap();
    assert!((oscu.ascending.longitude_deg - bar.ascending.longitude_deg).abs() < 1e-9);
    // …and diverges beyond it.
    let n_oscu = engine
        .nod_aps(
            CelestialBody::Neptune,
            tdb(JD),
            NodApsMethod::Osculating,
            ApsisConvention::Aphelion,
        )
        .unwrap();
    let n_bar = engine
        .nod_aps(
            CelestialBody::Neptune,
            tdb(JD),
            NodApsMethod::OsculatingBarycentric,
            ApsisConvention::Aphelion,
        )
        .unwrap();
    assert!((n_oscu.ascending.longitude_deg - n_bar.ascending.longitude_deg).abs() > 1e-6);
}

#[test]
fn default_method_matches_se_semantics() {
    let engine = engine();
    let venus = engine
        .nod_aps_default(CelestialBody::Venus, tdb(JD), ApsisConvention::Aphelion)
        .unwrap();
    assert_eq!(venus.method, NodApsMethod::Mean);
    let pluto = engine
        .nod_aps_default(CelestialBody::Pluto, tdb(JD), ApsisConvention::Aphelion)
        .unwrap();
    assert_eq!(pluto.method, NodApsMethod::Osculating);
}

#[test]
fn fictitious_bodies_compose_through_the_chain() {
    let engine = engine();
    let body = CelestialBody::Cupido;
    let r = engine
        .nod_aps(
            body.clone(),
            tdb(JD),
            NodApsMethod::Osculating,
            ApsisConvention::Aphelion,
        )
        .unwrap_or_else(|e| panic!("{body:?}: {e}"));
    assert!(r.perihelion.distance_au.is_finite() && r.perihelion.distance_au > 0.0);
    assert!(r.ascending.latitude_deg.abs() < 0.5, "{body:?}");
}

/// Ceres (and the other JPL-snapshot-only selected asteroids) is served by no
/// continuous backend in the production chain — only `JplSnapshotBackend`,
/// whose sparse regression fixtures are linearly interpolated between epochs
/// that can lie centuries apart. That interpolation cannot support nod_aps's
/// sub-day finite-difference sampling: the velocity estimate straddles an
/// epoch-bracket seam and comes out non-physical, so the engine fails closed
/// with a typed error instead of returning garbage orbital points. SE
/// small-body parity here is a documented coverage bound (follow-up filed in
/// docs/follow-ups.md by a later task).
#[test]
fn snapshot_only_asteroids_fail_closed() {
    let engine = engine();
    let err = engine
        .nod_aps(
            CelestialBody::Ceres,
            tdb(JD),
            NodApsMethod::Osculating,
            ApsisConvention::Aphelion,
        )
        .unwrap_err();
    assert!(
        matches!(err, EventError::DegenerateNodAps { .. }),
        "expected a fail-closed DegenerateNodAps error, got: {err:?}"
    );
}

/// asteroid:433-Eros is the one custom asteroid the packaged artifact serves
/// continuously, but its compressed fit is built from a 180-day-cadence corpus
/// against a ~643-day orbit: positions are exact at corpus epochs (verified to
/// ~1e-9 deg against the JPL J2000 snapshot row) while the fit's
/// time-derivative is non-physical at any sampling scale (measured
/// heliocentric |v| ≈ 0.060 AU/day, stable across dt from 1e-4 to 1e-1 days,
/// vs. an escape velocity of 0.021 AU/day at r = 1.345 AU). nod_aps's
/// osculating elements need that derivative, so the engine fails closed with
/// the same typed error rather than emitting garbage orbital points. Same
/// coverage-bound follow-up as the snapshot-only asteroids above.
#[test]
fn packaged_fit_asteroids_fail_closed() {
    let engine = engine();
    let err = engine
        .nod_aps(
            CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
            tdb(JD),
            NodApsMethod::Osculating,
            ApsisConvention::Aphelion,
        )
        .unwrap_err();
    assert!(
        matches!(err, EventError::DegenerateNodAps { .. }),
        "expected a fail-closed DegenerateNodAps error, got: {err:?}"
    );
}
