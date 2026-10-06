//! nod_aps integration over the production-style backend chain.
//!
//! Asteroid coverage bound: away from the snapshot's fixture rows no asteroid
//! in the offline chain supports `nod_aps`'s osculating sampling. Ceres,
//! Pallas, Juno, Vesta, asteroid:99942-Apophis and asteroid:433-Eros are
//! served only by `JplSnapshotBackend`, at or between closely spaced sample
//! rows; the packaged backend carries an Eros fit it does not serve. nod_aps
//! samples off the row, so at an isolated row it fails closed with the
//! backend's refusal (issue #158) — pinned below as the correct production
//! behavior. Inside the January 2001 cluster the rows are close enough and
//! it is served (issue #201).

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

#[test]
fn moon_mean_node_matches_swiss_ephemeris_a_century_from_j2000() {
    // Swiss Ephemeris 2.10.03 swe_nod_aps(Moon, mean, MOSEPH) at JD 2415100.5 TT
    // (row 12 of the committed nod-aps corpus): ascending node 254.924769366°,
    // true equinox of date. One century from J2000 a frame slip in the
    // analytic mean-element frame (of date read as J2000, or the reverse)
    // shows up as a ~1.4° error, far above the gate's sub-arcsecond MEAN_MOON
    // ceiling.
    let engine = engine();
    let r = engine
        .nod_aps(
            CelestialBody::Moon,
            tdb(2_415_100.5),
            NodApsMethod::Mean,
            ApsisConvention::Aphelion,
        )
        .unwrap();
    let residual_arcsec =
        pleiades_events_test_wrap(r.ascending.longitude_deg, 254.924_769_366).abs() * 3600.0;
    assert!(
        residual_arcsec < 5.0,
        "mean Moon ascending node residual vs SE: {residual_arcsec}″"
    );
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
/// which answers at its sample rows and refuses an instant its rows cannot
/// support (issue #158). nod_aps samples a fraction of a day either side of
/// the query, off the J2000 row, so the engine fails closed with the
/// backend's refusal. SE small-body parity here is a documented coverage
/// bound (issues #158 and #160).
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
        matches!(err, EventError::Backend { .. }),
        "expected the backend's refusal, got: {err:?}"
    );
    assert!(err.to_string().contains("SpkBackend"), "{err}");
    assert!(err.to_string().contains("OutOfRangeInstant"), "{err}");
}

/// Inside the January 2001 cluster the snapshot rows are a day or less apart,
/// so the stencil guard admits nod_aps's sampling a fraction of a day either
/// side of the query and Ceres is served (issue #201).
#[test]
fn a_snapshot_asteroid_is_served_inside_the_fixture_cluster() {
    let engine = engine();
    let result = engine
        .nod_aps(
            CelestialBody::Ceres,
            tdb(2_451_915.0),
            NodApsMethod::Osculating,
            ApsisConvention::Aphelion,
        )
        .expect("the cluster rows support nod_aps's sampling");
    assert!(result.perihelion.distance_au.is_finite() && result.perihelion.distance_au > 0.0);
    assert!(result.ascending.latitude_deg.abs() < 0.5);
}

/// asteroid:433-Eros routes past the packaged backend, which carries a fit it
/// does not serve, to `JplSnapshotBackend`. The J2000 row is exact, but
/// nod_aps samples a fraction of a day either side of it, where the snapshot
/// refuses (issue #158).
#[test]
fn eros_fails_closed_past_the_packaged_backend() {
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
        matches!(err, EventError::Backend { .. }),
        "expected the backend's refusal, got: {err:?}"
    );
    assert!(err.to_string().contains("OutOfRangeInstant"), "{err}");
}

/// Issue #90: the mean lunar node and apsides are analytic, so the Mean method
/// for the Moon must work on the packaged backend alone.
#[test]
fn moon_mean_points_work_on_the_packaged_backend_alone() {
    let engine = EventEngine::new(PackagedDataBackend::new());
    let r = engine
        .nod_aps(
            CelestialBody::Moon,
            tdb(2_460_000.5),
            NodApsMethod::Mean,
            ApsisConvention::Aphelion,
        )
        .expect("Moon mean nod_aps on the packaged backend");
    assert!(r.ascending.longitude_deg.is_finite());

    // Swiss Ephemeris swe_nod_aps, Moon, SE_NODBIT_MEAN at J2000 (nod-aps corpus).
    let j2000 = engine
        .nod_aps(
            CelestialBody::Moon,
            tdb(JD),
            NodApsMethod::Mean,
            ApsisConvention::Aphelion,
        )
        .unwrap();
    let arcsec =
        |got: f64, want: f64| ((got - want + 180.0).rem_euclid(360.0) - 180.0).abs() * 3600.0;
    assert!(arcsec(j2000.ascending.longitude_deg, 125.040_685_175) < 0.8);
    assert!(arcsec(j2000.aphelion.longitude_deg, 263.464_250_479) < 0.8);
    assert!(((j2000.aphelion.latitude_deg - 3.419_723_161) * 3600.0).abs() < 0.06);
}
