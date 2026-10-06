//! A sidereal placement the chart leaves mean is on the mean ecliptic and
//! equinox of date, less the mean ayanamsa (issue #164 (b)).
//!
//! The backends report a mean place on the J2000 equinox. An ayanamsa is the
//! distance from the sidereal zero point to the equinox of date, so before
//! this fix a sidereal mean longitude kept the precession accumulated since
//! J2000: 1.2 degrees at 1913.

use pleiades_apparent::precess_ecliptic_j2000_to_date;
use pleiades_backend::{
    Apparentness, BackendMetadata, EphemerisBackend, EphemerisError, EphemerisRequest,
    EphemerisResult,
};
use pleiades_data::packaged_backend;
use pleiades_types::{CelestialBody, EclipticCoordinates, Latitude, Longitude, Motion};

use super::sidereal::{mean_motion_of_date, mean_place_of_date};
use super::sidereal_tests::{
    composite_backend, lahiri, lahiri_deg, rate, speed_deg_per_day, tt, wrap_arcsec,
};
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

/// [`AbsurdDistanceReleaseGradeBackend`] with a speed, which that backend
/// does not report, so a fallback's speed has something to go wrong.
struct MovingAbsurdDistanceBackend;

impl EphemerisBackend for MovingAbsurdDistanceBackend {
    fn metadata(&self) -> BackendMetadata {
        AbsurdDistanceReleaseGradeBackend.metadata()
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        AbsurdDistanceReleaseGradeBackend.supports_body(body)
    }

    fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        let mut result = AbsurdDistanceReleaseGradeBackend.position(request)?;
        result.motion = Some(Motion::new(Some(0.5), Some(0.01), None));
        Ok(result)
    }
}

// Mars falls back to its mean place in the apparent chart (its distance
// trips the light-time sanity cap). The fallback and a requested mean chart
// go through one step, place and speed, and must keep doing so.
#[test]
fn a_mean_fallback_in_a_sidereal_chart_reports_the_requested_mean_place() {
    let chart = |apparentness: Apparentness| {
        ChartEngine::new(MovingAbsurdDistanceBackend)
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
    let motion = |snapshot: &ChartSnapshot| {
        snapshot
            .placement_for(&mars)
            .expect("Mars is placed")
            .position
            .motion
    };
    assert!(motion(&mean).is_some(), "the mean chart must carry a speed");
    assert_eq!(motion(&apparent), motion(&mean));
}

/// Bodies the composite backend reports a longitude speed for.
fn moving_bodies() -> Vec<CelestialBody> {
    vec![
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Saturn,
        CelestialBody::Pluto,
    ]
}

/// Rate of [`general_precession_arcsec`], in degrees per day.
fn general_precession_rate_deg_per_day(jd_tt: f64) -> f64 {
    let t = (jd_tt - 2_451_545.0) / 36_525.0;
    (5029.0966 + 2.0 * 1.11113 * t - 3.0 * 0.000_006 * t * t) / 3600.0 / 36_525.0
}

// The Sun stays on the ecliptic, so its longitude on the equinox of date
// moves faster than its J2000 longitude by the precession rate, 0.138″ a
// day, and the ayanamsa takes its own rate off. Before the fix only the
// ayanamsa's rate came off, and the speed was 3.8e-5 deg/day too low.
#[test]
fn the_suns_sidereal_mean_speed_gains_the_precession_rate_and_loses_the_ayanamsas() {
    let sun = CelestialBody::Sun;
    for jd in EPOCHS_JD_TT {
        let (tropical, sidereal) = mean_charts(jd, vec![sun.clone()]);
        let gain = speed_deg_per_day(&sidereal, &sun) - speed_deg_per_day(&tropical, &sun);
        let want = general_precession_rate_deg_per_day(jd) - rate(lahiri_deg, jd);
        assert!(
            (gain - want).abs() < 5e-7,
            "at {jd}: sidereal − tropical speed is {gain}, expected {want} deg/day"
        );
    }
    assert!(general_precession_rate_deg_per_day(2_451_545.0) > 3.8e-5);
}

fn latitude_speed_deg_per_day(snapshot: &ChartSnapshot, body: &CelestialBody) -> f64 {
    snapshot
        .placement_for(body)
        .expect("body is placed")
        .position
        .motion
        .expect("motion")
        .latitude_deg_per_day
        .expect("latitude speed")
}

// For every body the reported speeds are the rates of the reported place.
// Off the ecliptic the precession step also depends on where the body is, so
// the Moon's longitude speed changes by up to 0.8″ a day at 1913; a
// difference of the charts' own places a day apart sees that too. The
// tilting ecliptic moves a latitude speed as well, by several ″ a day for
// the Moon. The tolerance covers the truncation of that difference for the
// Moon (5e-7 deg/day).
#[test]
fn a_sidereal_mean_speed_is_the_rate_of_the_reported_place() {
    let jd = EPOCHS_JD_TT[0];
    // The step (sidereal less tropical place) of each body, in longitude and
    // latitude.
    let steps = |jd_tt: f64| {
        let (tropical, sidereal) = mean_charts(jd_tt, moving_bodies());
        moving_bodies()
            .into_iter()
            .map(|body| {
                let (to, from) = (ecliptic(&sidereal, &body), ecliptic(&tropical, &body));
                (
                    to.longitude.degrees() - from.longitude.degrees(),
                    to.latitude.degrees() - from.latitude.degrees(),
                )
            })
            .collect::<Vec<(f64, f64)>>()
    };
    let (earlier, later) = (steps(jd - 0.5), steps(jd + 0.5));
    let (tropical, sidereal) = mean_charts(jd, moving_bodies());
    for (index, body) in moving_bodies().into_iter().enumerate() {
        let gain = speed_deg_per_day(&sidereal, &body) - speed_deg_per_day(&tropical, &body);
        let want = wrap_arcsec(later[index].0 - earlier[index].0) / 3600.0;
        assert!(
            (gain - want).abs() < 2e-6,
            "{body:?}: sidereal − tropical speed is {gain}, the step's rate is {want} deg/day"
        );
        // A latitude difference does not wrap.
        let latitude_gain = latitude_speed_deg_per_day(&sidereal, &body)
            - latitude_speed_deg_per_day(&tropical, &body);
        let latitude_want = later[index].1 - earlier[index].1;
        assert!(
            (latitude_gain - latitude_want).abs() < 2e-6,
            "{body:?}: sidereal − tropical latitude speed is {latitude_gain}, the step's rate is {latitude_want} deg/day"
        );
    }
}

#[test]
fn a_missing_speed_channel_stays_missing() {
    let j2000 = EclipticCoordinates::new(
        Longitude::from_degrees(120.0),
        Latitude::from_degrees(4.0),
        Some(1.0),
    );
    let jd = EPOCHS_JD_TT[0];
    let none = mean_motion_of_date(j2000, Motion::new(None, None, None), jd).expect("motion");
    assert_eq!(none, Motion::new(None, None, None));
    let latitude_only =
        mean_motion_of_date(j2000, Motion::new(None, Some(0.01), None), jd).expect("motion");
    assert_eq!(latitude_only.longitude_deg_per_day, None);
    assert_eq!(latitude_only.distance_au_per_day, None);
    let latitude_speed = latitude_only.latitude_deg_per_day.expect("latitude speed");
    assert!((latitude_speed - 0.01).abs() < 1e-6, "{latitude_speed}");
    // A distance speed is not a matter of frame.
    let with_distance =
        mean_motion_of_date(j2000, Motion::new(Some(1.0), Some(0.0), Some(1e-4)), jd)
            .expect("motion");
    assert_eq!(with_distance.distance_au_per_day, Some(1e-4));
}

#[test]
fn a_speed_across_the_zero_of_longitude_is_continuous() {
    // The place precesses from 0.2° back past 360° at 1913
    // (`a_place_near_the_equinox_precesses_across_the_zero_of_longitude`).
    let j2000 = EclipticCoordinates::new(
        Longitude::from_degrees(0.2),
        Latitude::from_degrees(0.0),
        Some(1.0),
    );
    let jd = EPOCHS_JD_TT[0];
    let motion =
        mean_motion_of_date(j2000, Motion::new(Some(1.0), Some(0.0), None), jd).expect("motion");
    let speed = motion.longitude_deg_per_day.expect("longitude speed");
    let want = 1.0 + general_precession_rate_deg_per_day(jd);
    assert!((speed - want).abs() < 2e-6, "{speed} vs {want}");
}

// The speed needs no neighbouring backend read, so the first instant the
// packaged backend serves has one.
#[test]
fn a_sidereal_mean_chart_at_the_window_start_has_a_speed() {
    let request = ChartRequest::new(tt(2_415_020.5))
        .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon])
        .with_apparentness(Apparentness::Mean)
        .with_zodiac_mode(lahiri());
    let chart = ChartEngine::new(packaged_backend())
        .chart(&request)
        .expect("chart at the window start");
    for body in [CelestialBody::Sun, CelestialBody::Moon] {
        assert!(speed_deg_per_day(&chart, &body) > 0.9, "{body:?}");
    }
}
