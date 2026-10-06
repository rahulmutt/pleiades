//! A sidereal placement the chart leaves mean is on the mean ecliptic and
//! equinox of date, less the mean ayanamsa (issue #164 (b)).
//!
//! The backends report a mean place on the J2000 equinox. An ayanamsa is the
//! distance from the sidereal zero point to the equinox of date, so before
//! this fix a sidereal mean longitude kept the precession accumulated since
//! J2000: 1.2 degrees at 1913.

use pleiades_apparent::precess_ecliptic_j2000_to_date;
use pleiades_backend::Apparentness;
use pleiades_types::{CelestialBody, EclipticCoordinates, Latitude, Longitude};

use super::sidereal::mean_place_of_date;
use super::sidereal_tests::{composite_backend, lahiri, lahiri_deg, tt, wrap_arcsec};
use super::test_support::AbsurdDistanceReleaseGradeBackend;
use crate::chart::{ChartEngine, ChartRequest, ChartSnapshot};

/// 1913, J2000 and 2077: near both ends of the supported window, and the
/// epoch at which the two equinoxes coincide.
const EPOCHS_JD_TT: [f64; 3] = [2_420_000.5, 2_451_545.0, 2_480_000.5];

/// A body on the ecliptic, the Moon at 5 degrees of latitude, a slow planet,
/// Pluto at 17 degrees, and a lunar point.
fn bodies() -> Vec<CelestialBody> {
    vec![
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Saturn,
        CelestialBody::Pluto,
        CelestialBody::MeanNode,
    ]
}

/// The tropical and the Lahiri mean chart of `bodies` at `jd_tt`.
fn mean_charts(jd_tt: f64, bodies: Vec<CelestialBody>) -> (ChartSnapshot, ChartSnapshot) {
    let engine = ChartEngine::new(composite_backend());
    let request = ChartRequest::new(tt(jd_tt))
        .with_bodies(bodies)
        .with_apparentness(Apparentness::Mean);
    let tropical = engine.chart(&request).expect("tropical mean chart");
    let sidereal = engine
        .chart(&request.with_zodiac_mode(lahiri()))
        .expect("sidereal mean chart");
    (tropical, sidereal)
}

fn ecliptic(snapshot: &ChartSnapshot, body: &CelestialBody) -> EclipticCoordinates {
    snapshot
        .placement_for(body)
        .expect("body is placed")
        .position
        .ecliptic
        .expect("ecliptic coordinates")
}

/// General precession in longitude accumulated since J2000, in arcseconds:
/// Lieske et al. 1977 (IAU 1976), `p_A = 5029.0966″ T + 1.11113″ T² −
/// 0.000006″ T³`, with `T` in Julian centuries of TT from J2000.
fn general_precession_arcsec(jd_tt: f64) -> f64 {
    let t = (jd_tt - 2_451_545.0) / 36_525.0;
    5029.0966 * t + 1.11113 * t * t - 0.000_006 * t * t * t
}

#[test]
fn a_sidereal_mean_place_is_the_precessed_j2000_place_less_the_ayanamsa() {
    for jd in EPOCHS_JD_TT {
        let (tropical, sidereal) = mean_charts(jd, bodies());
        for body in bodies() {
            // A tropical mean chart reports the backend's J2000 place.
            let j2000 = ecliptic(&tropical, &body);
            let of_date = precess_ecliptic_j2000_to_date(
                j2000.longitude.degrees(),
                j2000.latitude.degrees(),
                jd,
            )
            .expect("precession");
            let got = ecliptic(&sidereal, &body);
            let longitude_off =
                wrap_arcsec(got.longitude.degrees() - (of_date.longitude_deg - lahiri_deg(jd)));
            let latitude_off = (got.latitude.degrees() - of_date.latitude_deg) * 3600.0;
            assert!(
                longitude_off.abs() < 1e-6,
                "{body:?} at {jd}: longitude is {longitude_off}″ off"
            );
            assert!(
                latitude_off.abs() < 1e-6,
                "{body:?} at {jd}: latitude is {latitude_off}″ off"
            );
            assert_eq!(got.distance_au, j2000.distance_au, "{body:?} at {jd}");
        }
    }
}

// An anchor that does not come from the code under test: the Sun stays on
// the ecliptic, so the equinox of date differs from the J2000 one by the
// general precession in longitude and nothing else.
#[test]
fn the_suns_sidereal_mean_longitude_carries_the_general_precession() {
    let sun = CelestialBody::Sun;
    for jd in EPOCHS_JD_TT {
        let (tropical, sidereal) = mean_charts(jd, vec![sun.clone()]);
        let shift = wrap_arcsec(
            ecliptic(&sidereal, &sun).longitude.degrees() + lahiri_deg(jd)
                - ecliptic(&tropical, &sun).longitude.degrees(),
        );
        let want = general_precession_arcsec(jd);
        assert!(
            (shift - want).abs() < 0.05,
            "at {jd}: the equinox moved {shift}″, general precession is {want}″"
        );
    }
    // 86 years before J2000 that is well over a degree.
    assert!(general_precession_arcsec(EPOCHS_JD_TT[0]) < -4300.0);
}

// The ecliptic of date is tilted against the J2000 one by 47″ per century,
// so a latitude changes by at most that tilt, and by how much depends on
// where the body is along the ecliptic.
#[test]
fn a_sidereal_mean_latitude_is_on_the_ecliptic_of_date() {
    let jd = EPOCHS_JD_TT[0];
    let (tropical, sidereal) = mean_charts(jd, bodies());
    let tilt = 47.1 * (jd - 2_451_545.0).abs() / 36_525.0;
    let mut largest = 0.0_f64;
    for body in bodies() {
        let moved = (ecliptic(&sidereal, &body).latitude.degrees()
            - ecliptic(&tropical, &body).latitude.degrees())
            * 3600.0;
        assert!(moved.abs() < tilt, "{body:?}: latitude moved {moved}″");
        largest = largest.max(moved.abs());
    }
    // Five bodies spread around the ecliptic cannot all sit on the line the
    // two ecliptics cross along.
    assert!(largest > 5.0, "no latitude moved more than {largest}″");
}

#[test]
fn a_place_near_the_equinox_precesses_across_the_zero_of_longitude() {
    let j2000 = EclipticCoordinates::new(
        Longitude::from_degrees(0.2),
        Latitude::from_degrees(0.0),
        Some(1.0),
    );
    let of_date = mean_place_of_date(j2000, EPOCHS_JD_TT[0]).expect("precession");
    // 0.2° less 1.206° of precession.
    let longitude = of_date.longitude.degrees();
    assert!((358.9..359.1).contains(&longitude), "{longitude}");
    assert_eq!(of_date.distance_au, Some(1.0));
    // At J2000 the two equinoxes are the same.
    let unchanged = mean_place_of_date(j2000, 2_451_545.0).expect("precession");
    assert!((unchanged.longitude.degrees() - 0.2).abs() < 1e-9);
}

// Mars falls back to its mean place in the apparent chart (its distance
// trips the light-time sanity cap). The fallback and a requested mean chart
// go through one step, and must keep doing so.
#[test]
fn a_mean_fallback_in_a_sidereal_chart_reports_the_requested_mean_place() {
    let chart = |apparentness: Apparentness| {
        ChartEngine::new(AbsurdDistanceReleaseGradeBackend)
            .chart(
                &ChartRequest::new(tt(EPOCHS_JD_TT[0]))
                    .with_bodies(vec![CelestialBody::Sun, CelestialBody::Mars])
                    .with_apparentness(apparentness)
                    .with_zodiac_mode(lahiri()),
            )
            .expect("chart succeeds")
    };
    let apparent = chart(Apparentness::Apparent);
    let mean = chart(Apparentness::Mean);
    let mars = CelestialBody::Mars;
    assert!(
        apparent
            .mean_fallback_placements()
            .any(|placement| placement.body == mars),
        "Mars must fall back for this test to mean anything"
    );
    let (fallback, requested) = (ecliptic(&apparent, &mars), ecliptic(&mean, &mars));
    assert_eq!(fallback.longitude.degrees(), requested.longitude.degrees());
    assert_eq!(fallback.latitude.degrees(), requested.latitude.degrees());
}
