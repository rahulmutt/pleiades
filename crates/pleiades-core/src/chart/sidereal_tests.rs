//! Sidereal placements, cusps and angles of a chart (issues #120, #141, #157).
//!
//! A sidereal longitude is the longitude on the **mean** equinox of date minus
//! the mean ayanamsa: the Swiss Ephemeris `SEFLG_SIDEREAL` convention, and the
//! one `pleiades-events` reads crossings in. An apparent longitude is on the
//! true equinox, so nutation in longitude (Δψ, up to about 17″) must come off
//! before the ayanamsa. Before this fix the chart subtracted the ayanamsa from
//! the true-equinox longitude and every sidereal placement carried Δψ.

use pleiades_apparent::nutation::nutation;
use pleiades_ayanamsa::sidereal_offset;
use pleiades_backend::{Apparentness, CompositeBackend, ZodiacMode};
use pleiades_elp::ElpBackend;
use pleiades_types::{
    Ayanamsa, CelestialBody, HouseSystem, Instant, JulianDay, Latitude, Longitude,
    ObserverLocation, TimeScale,
};
use pleiades_vsop87::Vsop87Backend;

use crate::chart::{ChartEngine, ChartRequest, ChartSnapshot};

/// 2000-01-01 12:00 UTC as TT, the instant of the issue's reproduction.
const ISSUE_120_JD_TT: f64 = 2_451_545.000_739;

/// Swiss Ephemeris 2.10 (Moshier) `SEFLG_SIDEREAL` Lahiri longitudes at
/// [`ISSUE_120_JD_TT`], quoted in issue #120.
const SWISS_EPHEMERIS_LAHIRI_DEG: [(CelestialBody, f64); 4] = [
    (CelestialBody::Sun, 256.515_697),
    (CelestialBody::Moon, 199.470_553),
    (CelestialBody::Mars, 304.110_091),
    (CelestialBody::Saturn, 16.542_416),
];

fn composite_backend() -> CompositeBackend<Vsop87Backend, ElpBackend> {
    CompositeBackend::new(Vsop87Backend::new(), ElpBackend::new())
}

fn issue_instant() -> Instant {
    Instant::new(JulianDay::from_days(ISSUE_120_JD_TT), TimeScale::Tt)
}

fn lahiri() -> ZodiacMode {
    ZodiacMode::Sidereal {
        ayanamsa: Ayanamsa::Lahiri,
    }
}

fn bodies() -> Vec<CelestialBody> {
    SWISS_EPHEMERIS_LAHIRI_DEG
        .iter()
        .map(|(body, _)| body.clone())
        .collect()
}

fn longitude_deg(snapshot: &ChartSnapshot, body: &CelestialBody) -> f64 {
    snapshot
        .placement_for(body)
        .expect("body is placed")
        .position
        .ecliptic
        .expect("ecliptic coordinates")
        .longitude
        .degrees()
}

fn wrap_arcsec(deg: f64) -> f64 {
    ((deg + 180.0).rem_euclid(360.0) - 180.0) * 3600.0
}

#[test]
fn issue_120_sidereal_apparent_chart_matches_swiss_ephemeris_lahiri() {
    // The issue measured every chart placement 13.7″ below Swiss Ephemeris,
    // which is Δψ at the instant (−13.92″), while the events engine agreed
    // with Swiss Ephemeris to 0.1-0.7″.
    let snapshot = ChartEngine::new(composite_backend())
        .chart(
            &ChartRequest::new(issue_instant())
                .with_bodies(bodies())
                .with_zodiac_mode(lahiri()),
        )
        .expect("sidereal apparent chart succeeds");
    for (body, expected) in &SWISS_EPHEMERIS_LAHIRI_DEG {
        let residual = wrap_arcsec(longitude_deg(&snapshot, body) - expected);
        assert!(
            residual.abs() < 1.0,
            "{body:?} sidereal longitude is {residual:+.3}\" from Swiss Ephemeris Lahiri"
        );
    }
}

#[test]
fn sidereal_apparent_shift_is_the_ayanamsa_plus_nutation_in_longitude() {
    // Independent of any reference ephemeris: tropical apparent minus sidereal
    // apparent must be the mean ayanamsa plus Δψ, not the ayanamsa alone.
    let engine = ChartEngine::new(composite_backend());
    let tropical = engine
        .chart(&ChartRequest::new(issue_instant()).with_bodies(bodies()))
        .expect("tropical chart succeeds");
    let sidereal = engine
        .chart(
            &ChartRequest::new(issue_instant())
                .with_bodies(bodies())
                .with_zodiac_mode(lahiri()),
        )
        .expect("sidereal chart succeeds");
    let ayanamsa_deg = sidereal_offset(&Ayanamsa::Lahiri, issue_instant())
        .expect("Lahiri offset")
        .degrees();
    let delta_psi_deg = nutation(ISSUE_120_JD_TT)
        .expect("nutation")
        .delta_psi_arcsec
        / 3600.0;
    for body in bodies() {
        let shift = longitude_deg(&tropical, &body) - longitude_deg(&sidereal, &body);
        let residual = wrap_arcsec(shift - (ayanamsa_deg + delta_psi_deg));
        assert!(
            residual.abs() < 1e-6,
            "{body:?}: tropical − sidereal differs from ayanamsa + Δψ by {residual:+.6}\""
        );
    }
}

// Issue #141: a sidereal chart reported the tropical speed. The speed is the
// rate of the longitude the placement reports, so it drops by the rate of
// whatever the sidereal step subtracts.

/// J2000.0 TT, the instant of issue #141's reproduction.
const ISSUE_141_JD_TT: f64 = 2_451_545.0;

/// Half-span the chart differences its speed corrections over, in days.
const HALF_SPAN_DAYS: f64 = 0.5;

fn tt(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tt)
}

fn speed_deg_per_day(snapshot: &ChartSnapshot, body: &CelestialBody) -> f64 {
    snapshot
        .placement_for(body)
        .expect("body is placed")
        .position
        .motion
        .expect("motion")
        .longitude_deg_per_day
        .expect("longitude speed")
}

fn lahiri_deg(jd: f64) -> f64 {
    sidereal_offset(&Ayanamsa::Lahiri, tt(jd))
        .expect("Lahiri offset")
        .degrees()
}

fn delta_psi_deg(jd: f64) -> f64 {
    nutation(jd).expect("nutation").delta_psi_arcsec / 3600.0
}

/// Central-difference rate of `value` at `jd`, per day.
fn rate(value: impl Fn(f64) -> f64, jd: f64) -> f64 {
    (value(jd + HALF_SPAN_DAYS) - value(jd - HALF_SPAN_DAYS)) / (2.0 * HALF_SPAN_DAYS)
}

fn issue_141_charts(apparentness: Apparentness) -> (ChartSnapshot, ChartSnapshot) {
    let engine = ChartEngine::new(composite_backend());
    let request = ChartRequest::new(tt(ISSUE_141_JD_TT))
        .with_bodies(issue_141_bodies())
        .with_apparentness(apparentness);
    let tropical = engine.chart(&request).expect("tropical chart succeeds");
    let sidereal = engine
        .chart(&request.with_zodiac_mode(lahiri()))
        .expect("sidereal chart succeeds");
    (tropical, sidereal)
}

fn issue_141_bodies() -> Vec<CelestialBody> {
    vec![
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Saturn,
        CelestialBody::Pluto,
    ]
}

#[test]
fn issue_141_sidereal_apparent_speed_matches_swiss_ephemeris_lahiri() {
    // Swiss Ephemeris 2.10.03 (Moshier) `SEFLG_SIDEREAL | SEFLG_SPEED` Lahiri
    // speeds at J2000.0 TT, quoted in issue #141. The tropical speeds the
    // chart used to report sit 4.0e-5 to 4.2e-5 deg/day above them.
    let (_, sidereal) = issue_141_charts(Apparentness::Apparent);
    for (body, expected) in [
        (CelestialBody::Sun, 1.019_391_809),
        (CelestialBody::Saturn, -0.019_986_138),
        (CelestialBody::Pluto, 0.035_113_209),
    ] {
        let residual = speed_deg_per_day(&sidereal, &body) - expected;
        assert!(
            residual.abs() < 3e-6,
            "{body:?} sidereal speed is {residual:+.3e} deg/day from Swiss Ephemeris Lahiri"
        );
    }
}

#[test]
fn sidereal_apparent_speed_drops_by_the_rate_of_the_ayanamsa_plus_nutation() {
    let (tropical, sidereal) = issue_141_charts(Apparentness::Apparent);
    let expected_drop = rate(|jd| lahiri_deg(jd) + delta_psi_deg(jd), ISSUE_141_JD_TT);
    assert!(expected_drop.abs() > 1e-5, "drop {expected_drop}");
    for body in issue_141_bodies() {
        let drop = speed_deg_per_day(&tropical, &body) - speed_deg_per_day(&sidereal, &body);
        assert!(
            (drop - expected_drop).abs() < 1e-10,
            "{body:?}: tropical − sidereal speed is {drop}, expected {expected_drop} deg/day"
        );
        // Latitude and distance are not touched by the sidereal step.
        let motion = |snapshot: &ChartSnapshot| {
            snapshot
                .placement_for(&body)
                .expect("body is placed")
                .position
                .motion
                .expect("motion")
        };
        assert_eq!(
            motion(&tropical).latitude_deg_per_day,
            motion(&sidereal).latitude_deg_per_day
        );
        assert_eq!(
            motion(&tropical).distance_au_per_day,
            motion(&sidereal).distance_au_per_day
        );
    }
}

#[test]
fn sidereal_mean_speed_drops_by_the_rate_of_the_ayanamsa() {
    // A mean chart subtracts the ayanamsa from the backend's J2000 longitude,
    // with no nutation to remove.
    let (tropical, sidereal) = issue_141_charts(Apparentness::Mean);
    let expected_drop = rate(lahiri_deg, ISSUE_141_JD_TT);
    assert!(expected_drop > 3.5e-5, "drop {expected_drop}");
    for body in issue_141_bodies() {
        let drop = speed_deg_per_day(&tropical, &body) - speed_deg_per_day(&sidereal, &body);
        assert!(
            (drop - expected_drop).abs() < 1e-12,
            "{body:?}: tropical − sidereal speed is {drop}, expected {expected_drop} deg/day"
        );
    }
}

// Issue #157: #120 moved body placements to the mean equinox before the
// ayanamsa, but house cusps and angles, which `pleiades-houses` computes on
// the true equinox, still had the ayanamsa alone subtracted and so kept Δψ.

/// J2000.0 TT, the first instant of issue #157's reproduction.
const ISSUE_157_JD_TT: f64 = 2_451_545.0;

/// Swiss Ephemeris (Moshier) `swe_houses_ex` Porphyry ascendant under
/// `SEFLG_SIDEREAL` / `SIDM_LAHIRI` at [`ISSUE_157_JD_TT`] for the issue's
/// observer: the 71.843421° the chart reported plus the 13.80″ the issue
/// measured it below Swiss Ephemeris.
const ISSUE_157_SWISS_EPHEMERIS_LAHIRI_ASCENDANT_DEG: f64 = 71.843_421 + 13.80 / 3600.0;

fn issue_157_request() -> ChartRequest {
    ChartRequest::new(tt(ISSUE_157_JD_TT))
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(13.0827),
            Longitude::from_degrees(80.2707),
            None,
        ))
        .with_house_system(HouseSystem::Porphyry)
        .with_bodies(vec![CelestialBody::Sun])
}

fn issue_157_charts(apparentness: Apparentness) -> (ChartSnapshot, ChartSnapshot) {
    let engine = ChartEngine::new(composite_backend());
    let request = issue_157_request().with_apparentness(apparentness);
    let tropical = engine.chart(&request).expect("tropical chart succeeds");
    let sidereal = engine
        .chart(&request.with_zodiac_mode(lahiri()))
        .expect("sidereal chart succeeds");
    (tropical, sidereal)
}

fn ascendant_deg(snapshot: &ChartSnapshot) -> f64 {
    snapshot
        .houses
        .as_ref()
        .expect("houses are computed")
        .angles
        .ascendant
        .degrees()
}

/// The cusps and the four angles of a chart's houses, labelled.
fn house_longitudes_deg(snapshot: &ChartSnapshot) -> Vec<(String, f64)> {
    let houses = snapshot.houses.as_ref().expect("houses are computed");
    let angles = [
        ("ascendant", houses.angles.ascendant),
        ("descendant", houses.angles.descendant),
        ("midheaven", houses.angles.midheaven),
        ("imum coeli", houses.angles.imum_coeli),
    ];
    houses
        .cusps
        .iter()
        .enumerate()
        .map(|(index, cusp)| (format!("cusp {}", index + 1), cusp.degrees()))
        .chain(
            angles
                .into_iter()
                .map(|(name, angle)| (name.to_owned(), angle.degrees())),
        )
        .collect()
}

#[test]
fn issue_157_sidereal_ascendant_matches_swiss_ephemeris_lahiri() {
    let (_, sidereal) = issue_157_charts(Apparentness::Apparent);
    let residual =
        wrap_arcsec(ascendant_deg(&sidereal) - ISSUE_157_SWISS_EPHEMERIS_LAHIRI_ASCENDANT_DEG);
    assert!(
        residual.abs() < 0.5,
        "sidereal ascendant is {residual:+.3}\" from Swiss Ephemeris Lahiri"
    );
}

#[test]
fn sidereal_cusps_and_angles_shift_by_the_ayanamsa_plus_nutation_in_longitude() {
    // Houses sit on the true equinox whatever the bodies' apparentness, so
    // both chart kinds take Δψ off them.
    let expected_deg = lahiri_deg(ISSUE_157_JD_TT) + delta_psi_deg(ISSUE_157_JD_TT);
    assert!(delta_psi_deg(ISSUE_157_JD_TT).abs() * 3600.0 > 13.0);
    for apparentness in [Apparentness::Apparent, Apparentness::Mean] {
        let (tropical, sidereal) = issue_157_charts(apparentness);
        let tropical = house_longitudes_deg(&tropical);
        let sidereal = house_longitudes_deg(&sidereal);
        assert_eq!(tropical.len(), 16);
        for ((name, tropical_deg), (_, sidereal_deg)) in tropical.iter().zip(&sidereal) {
            let residual = wrap_arcsec(tropical_deg - sidereal_deg - expected_deg);
            assert!(
                residual.abs() < 1e-6,
                "{apparentness:?} {name}: tropical − sidereal differs from ayanamsa + Δψ by {residual:+.6}\""
            );
        }
    }
}

#[test]
fn sidereal_ascendant_and_sun_share_one_offset_from_the_tropical_chart() {
    // The issue's in-chart symptom: tropical − sidereal was 23.857056° for the
    // ascendant and 23.853188° for the Sun, 13.9″ (−Δψ) apart.
    let (tropical, sidereal) = issue_157_charts(Apparentness::Apparent);
    let ascendant_shift = ascendant_deg(&tropical) - ascendant_deg(&sidereal);
    let sun_shift = longitude_deg(&tropical, &CelestialBody::Sun)
        - longitude_deg(&sidereal, &CelestialBody::Sun);
    let residual = wrap_arcsec(ascendant_shift - sun_shift);
    assert!(
        residual.abs() < 1e-6,
        "ascendant and Sun sidereal offsets differ by {residual:+.6}\""
    );
}

// Issue #180: Whole Sign and Equal (1=Aries) cusps sit on the zodiac's sign
// boundaries, so in a sidereal chart they are the sidereal boundaries. A
// rigid shift of the tropical cusps, right for every ascendant-anchored
// system, left them about 6° into a sidereal sign.

/// 2026-03-21 05:00 UTC as TT, the instant of issue #180's reproduction.
const ISSUE_180_JD_TT: f64 = 2_461_120.709_134_04;

fn sidereal_chart(
    jd_tt: f64,
    latitude_deg: f64,
    system: HouseSystem,
    bodies: Vec<CelestialBody>,
    apparentness: Apparentness,
) -> ChartSnapshot {
    ChartEngine::new(composite_backend())
        .chart(
            &ChartRequest::new(tt(jd_tt))
                .with_observer(ObserverLocation::new(
                    Latitude::from_degrees(latitude_deg),
                    Longitude::from_degrees(80.2707),
                    None,
                ))
                .with_house_system(system)
                .with_bodies(bodies)
                .with_apparentness(apparentness)
                .with_zodiac_mode(lahiri()),
        )
        .expect("sidereal chart succeeds")
}

fn issue_180_chart(system: HouseSystem) -> ChartSnapshot {
    sidereal_chart(
        ISSUE_180_JD_TT,
        13.0827,
        system,
        vec![CelestialBody::Sun, CelestialBody::Moon],
        Apparentness::Apparent,
    )
}

fn cusps_deg(snapshot: &ChartSnapshot) -> Vec<f64> {
    let houses = snapshot.houses.as_ref().expect("houses are computed");
    houses.cusps.iter().map(|cusp| cusp.degrees()).collect()
}

/// Zero-based index of the sign holding `longitude_deg`.
fn sign_index(longitude_deg: f64) -> usize {
    (longitude_deg.rem_euclid(360.0) / 30.0).floor() as usize
}

fn house_of(snapshot: &ChartSnapshot, body: &CelestialBody) -> usize {
    snapshot.house_for_body(body).expect("body has a house")
}

#[test]
fn issue_180_sidereal_whole_sign_cusps_match_swiss_ephemeris() {
    // Swiss Ephemeris 2.10 `swe_houses_ex(.., 'W', SEFLG_SIDEREAL)`, Lahiri:
    // ascendant 46.7632 (Taurus), cusps 30, 60, 90, ...; the chart reported
    // 35.7750, 65.7750, 95.7750.
    let chart = issue_180_chart(HouseSystem::WholeSign);
    assert!((ascendant_deg(&chart) - 46.7632).abs() < 1e-3);
    let expected: Vec<f64> = (0..12)
        .map(|house| (30.0 + 30.0 * f64::from(house)) % 360.0)
        .collect();
    assert_eq!(cusps_deg(&chart), expected);
    // The sidereal Moon is at 4.82° Aries, the twelfth sign from Taurus; the
    // shifted cusps put it in the eleventh house.
    assert_eq!(sign_index(longitude_deg(&chart, &CelestialBody::Moon)), 0);
    assert_eq!(house_of(&chart, &CelestialBody::Moon), 12);
}

#[test]
fn issue_180_sidereal_equal_aries_cusps_match_swiss_ephemeris() {
    // Swiss Ephemeris `'N'` under `SEFLG_SIDEREAL`: cusps 0, 30, 60, ...; the
    // chart reported 335.7750, 5.7750, 35.7750.
    let chart = issue_180_chart(HouseSystem::EqualAries);
    let expected: Vec<f64> = (0..12).map(|house| 30.0 * f64::from(house)).collect();
    assert_eq!(cusps_deg(&chart), expected);
    // The sidereal Sun is at 6.36° Pisces, the twelfth sign; the shifted
    // cusps put it in the first house.
    assert_eq!(sign_index(longitude_deg(&chart, &CelestialBody::Sun)), 11);
    assert_eq!(house_of(&chart, &CelestialBody::Sun), 12);
}

#[test]
fn sidereal_sign_anchored_houses_count_signs() {
    // Independent of any reference ephemeris: with cusps on sidereal sign
    // boundaries, a body's whole-sign house is its sign counted from the
    // ascendant's sign, and its Equal (1=Aries) house is its sign counted
    // from Aries.
    let chart = |jd_tt, latitude_deg, system, bodies| {
        sidereal_chart(jd_tt, latitude_deg, system, bodies, Apparentness::Apparent)
    };
    let bodies = vec![
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mars,
        CelestialBody::Saturn,
    ];
    for step in 0..12 {
        let jd_tt = 2_440_000.5 + 1_777.7 * f64::from(step);
        for latitude_deg in [-60.0, 13.0827, 51.5, 64.0] {
            let whole_sign = chart(jd_tt, latitude_deg, HouseSystem::WholeSign, bodies.clone());
            let equal_aries = chart(jd_tt, latitude_deg, HouseSystem::EqualAries, bodies.clone());
            let rising_sign = sign_index(ascendant_deg(&whole_sign));
            for body in &bodies {
                let sign = sign_index(longitude_deg(&whole_sign, body));
                assert_eq!(
                    house_of(&whole_sign, body),
                    (sign + 12 - rising_sign) % 12 + 1,
                    "whole sign: {body:?} at JD {jd_tt}, latitude {latitude_deg}"
                );
                assert_eq!(
                    house_of(&equal_aries, body),
                    sign + 1,
                    "equal Aries: {body:?} at JD {jd_tt}, latitude {latitude_deg}"
                );
            }
        }
    }
}

#[test]
fn tropical_sign_anchored_cusps_stay_on_tropical_sign_boundaries() {
    for system in [HouseSystem::WholeSign, HouseSystem::EqualAries] {
        let request = issue_157_request().with_house_system(system.clone());
        let chart = ChartEngine::new(composite_backend())
            .chart(&request)
            .expect("tropical chart succeeds");
        let first = match system {
            HouseSystem::WholeSign => 30.0 * sign_index(ascendant_deg(&chart)) as f64,
            _ => 0.0,
        };
        let expected: Vec<f64> = (0..12)
            .map(|house| (first + 30.0 * f64::from(house)) % 360.0)
            .collect();
        assert_eq!(cusps_deg(&chart), expected, "{system:?}");
    }
}
