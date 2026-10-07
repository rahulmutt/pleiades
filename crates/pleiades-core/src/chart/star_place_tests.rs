use pleiades_apparent::motion::HALF_SPAN_DAYS;
use pleiades_backend::{Apparentness, EphemerisErrorKind};
use pleiades_data::packaged_backend;
use pleiades_types::{
    Ayanamsa, CelestialBody, HouseSystem, Instant, JulianDay, Latitude, Longitude,
    ObserverLocation, SiderealStarPlace, TimeScale, ZodiacMode,
};

use super::test_support::{AbsurdDistanceReleaseGradeBackend, NativeSiderealChartBackend};
use crate::apparent_star_ayanamsa_correction;
use crate::chart::{ChartEngine, ChartRequest, ChartSnapshot};

const JD: f64 = 2_460_000.5;

fn instant() -> Instant {
    Instant::new(JulianDay::from_days(JD), TimeScale::Tt)
}

fn request(ayanamsa: Ayanamsa, star_place: SiderealStarPlace) -> ChartRequest {
    ChartRequest::new(instant())
        .with_bodies(vec![
            CelestialBody::Sun,
            CelestialBody::Moon,
            CelestialBody::Mars,
            CelestialBody::MeanNode,
            CelestialBody::TrueNode,
        ])
        .with_zodiac_mode(ZodiacMode::Sidereal { ayanamsa })
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(13.0827),
            Longitude::from_degrees(80.2707),
            None,
        ))
        .with_house_system(HouseSystem::Placidus)
        .with_sidereal_star_place(star_place)
}

fn chart(request: &ChartRequest) -> ChartSnapshot {
    ChartEngine::new(packaged_backend())
        .chart(request)
        .expect("chart")
}

fn correction_deg(ayanamsa: &Ayanamsa, jd: f64) -> f64 {
    apparent_star_ayanamsa_correction(
        ayanamsa,
        Instant::new(JulianDay::from_days(jd), TimeScale::Tt),
    )
    .map_or(0.0, |a| a.degrees())
}

fn wrap(d: f64) -> f64 {
    (d + 540.0).rem_euclid(360.0) - 180.0
}

#[test]
fn the_default_star_place_is_mean() {
    assert_eq!(
        ChartRequest::new(instant()).sidereal_star_place,
        SiderealStarPlace::Mean
    );
    let snapshot = chart(&request(Ayanamsa::TrueCitra, SiderealStarPlace::Mean));
    assert_eq!(snapshot.sidereal_star_place, SiderealStarPlace::Mean);
}

// Every placement of an apparent chart, the lunar points included (spec
// amendment 6), moves by exactly −correction; so do the cusps and angles
// (amendment 5).
#[test]
fn the_apparent_star_place_moves_placements_and_cusps_by_the_correction() {
    let ayanamsa = Ayanamsa::TrueCitra;
    let mean = chart(&request(ayanamsa.clone(), SiderealStarPlace::Mean));
    let apparent = chart(&request(ayanamsa.clone(), SiderealStarPlace::Apparent));
    let shift = -correction_deg(&ayanamsa, JD);
    assert!(
        shift.abs() > 1.0 / 3600.0,
        "the test epoch must have a sizeable correction"
    );
    for (m, a) in mean.placements.iter().zip(&apparent.placements) {
        let dm = m.position.ecliptic.unwrap().longitude.degrees();
        let da = a.position.ecliptic.unwrap().longitude.degrees();
        assert!((wrap(da - dm) - shift).abs() < 1e-9, "{:?}", m.body);
        assert_eq!(
            m.position.ecliptic.unwrap().latitude,
            a.position.ecliptic.unwrap().latitude
        );
    }
    let (hm, ha) = (mean.houses.unwrap(), apparent.houses.unwrap());
    assert!(
        (wrap(ha.angles.ascendant.degrees() - hm.angles.ascendant.degrees()) - shift).abs() < 1e-9
    );
    for (cm, ca) in hm.cusps.iter().zip(&ha.cusps) {
        assert!((wrap(ca.degrees() - cm.degrees()) - shift).abs() < 1e-9);
    }
}

// Issue #226: Whole Sign and Equal (1=Aries) cusps are rebuilt from the
// corrected ascendant on the sidereal sign boundaries (#180), so they do not
// take the −correction shift; the ascendant does.
#[test]
fn sign_anchored_cusps_stay_on_sign_boundaries_under_the_apparent_star_place() {
    let ayanamsa = Ayanamsa::TrueCitra;
    let shift = -correction_deg(&ayanamsa, JD);
    for system in [HouseSystem::WholeSign, HouseSystem::EqualAries] {
        let houses = |place| {
            chart(&request(ayanamsa.clone(), place).with_house_system(system.clone()))
                .houses
                .expect("houses")
        };
        let (mean, apparent) = (
            houses(SiderealStarPlace::Mean),
            houses(SiderealStarPlace::Apparent),
        );
        let (asc_mean, asc_apparent) = (
            mean.angles.ascendant.degrees(),
            apparent.angles.ascendant.degrees(),
        );
        assert!(
            (wrap(asc_apparent - asc_mean) - shift).abs() < 1e-9,
            "{system:?}"
        );
        // The correction must not carry the ascendant into another sign, or
        // Whole Sign's cusps would legitimately move by 30°.
        assert_eq!(
            (asc_mean / 30.0).floor(),
            (asc_apparent / 30.0).floor(),
            "{system:?}: the test epoch's ascendant sits on a sign boundary"
        );
        for (cm, ca) in mean.cusps.iter().zip(&apparent.cusps) {
            let degrees = ca.degrees();
            let off_boundary = degrees - 30.0 * (degrees / 30.0).round();
            assert!(off_boundary.abs() < 1e-9, "{system:?} {degrees}");
            assert!(wrap(degrees - cm.degrees()).abs() < 1e-9, "{system:?}");
        }
    }
}

// Issue #226: the non-angle `ascmc` points take the correction like the angles
// (spec amendment 5); ARMC is a right ascension and does not.
#[test]
fn non_angle_ascmc_points_move_by_the_correction() {
    let ayanamsa = Ayanamsa::TrueCitra;
    let shift = -correction_deg(&ayanamsa, JD);
    let mean = chart(&request(ayanamsa.clone(), SiderealStarPlace::Mean));
    let apparent = chart(&request(ayanamsa.clone(), SiderealStarPlace::Apparent));
    let (pm, pa) = (mean.asc_mc().unwrap(), apparent.asc_mc().unwrap());
    for (name, m, a) in [
        ("vertex", pm.vertex, pa.vertex),
        ("antivertex", pm.antivertex, pa.antivertex),
        (
            "equatorial ascendant",
            pm.equatorial_ascendant,
            pa.equatorial_ascendant,
        ),
        (
            "coascendant (Koch)",
            pm.coascendant_koch,
            pa.coascendant_koch,
        ),
        (
            "coascendant (Munkasey)",
            pm.coascendant_munkasey,
            pa.coascendant_munkasey,
        ),
        ("polar ascendant", pm.polar_ascendant, pa.polar_ascendant),
    ] {
        assert!(
            (wrap(a.degrees() - m.degrees()) - shift).abs() < 1e-9,
            "{name}"
        );
    }
    assert_eq!(pm.armc, pa.armc);
}

// The speed takes the correction's rate, through the chart's own central
// difference over ±HALF_SPAN_DAYS (spec amendment 8).
#[test]
fn speeds_take_the_correction_rate() {
    let ayanamsa = Ayanamsa::TruePushya;
    let mean = chart(&request(ayanamsa.clone(), SiderealStarPlace::Mean));
    let apparent = chart(&request(ayanamsa.clone(), SiderealStarPlace::Apparent));
    let rate = (correction_deg(&ayanamsa, JD + HALF_SPAN_DAYS)
        - correction_deg(&ayanamsa, JD - HALF_SPAN_DAYS))
        / (2.0 * HALF_SPAN_DAYS);
    for (m, a) in mean.placements.iter().zip(&apparent.placements) {
        let sm = m.position.motion.unwrap().longitude_deg_per_day.unwrap();
        let sa = a.position.motion.unwrap().longitude_deg_per_day.unwrap();
        assert!((sa - sm + rate).abs() < 1e-9, "{:?}", m.body);
    }
}

#[test]
fn unanchored_ayanamsas_and_the_tropical_zodiac_are_unchanged() {
    for zodiac in [
        ZodiacMode::Tropical,
        ZodiacMode::Sidereal {
            ayanamsa: Ayanamsa::Lahiri,
        },
    ] {
        let base =
            request(Ayanamsa::Lahiri, SiderealStarPlace::Mean).with_zodiac_mode(zodiac.clone());
        let with = base
            .clone()
            .with_sidereal_star_place(SiderealStarPlace::Apparent);
        assert_eq!(
            chart(&base).placements,
            chart(&with).placements,
            "{zodiac:?}"
        );
        assert_eq!(chart(&base).houses, chart(&with).houses, "{zodiac:?}");
    }
}

// Review Focus 1: Swiss Ephemeris's geometric flags keep the mean ayanamsa.
#[test]
fn mean_chart_cusps_ignore_the_star_place() {
    let base =
        request(Ayanamsa::TrueCitra, SiderealStarPlace::Mean).with_apparentness(Apparentness::Mean);
    let with = base
        .clone()
        .with_sidereal_star_place(SiderealStarPlace::Apparent);
    assert_eq!(chart(&base).placements, chart(&with).placements);
    assert_eq!(chart(&base).houses, chart(&with).houses);
}

// Review Focus 2: on the backend `apparentness_applied_tests.rs` uses, Mars's
// distance trips the light-time cap and Mars falls back to its mean place
// while the Sun is reduced. The fallback keeps the mean ayanamsa; the Sun
// moves by −correction.
#[test]
fn a_mean_fallback_placement_keeps_the_mean_ayanamsa() {
    let jd = 2_451_545.0;
    let fallback_chart = |place| {
        ChartEngine::new(AbsurdDistanceReleaseGradeBackend)
            .chart(
                &ChartRequest::new(Instant::new(JulianDay::from_days(jd), TimeScale::Tt))
                    .with_bodies(vec![CelestialBody::Sun, CelestialBody::Mars])
                    .with_zodiac_mode(ZodiacMode::Sidereal {
                        ayanamsa: Ayanamsa::TrueCitra,
                    })
                    .with_sidereal_star_place(place),
            )
            .expect("chart succeeds")
    };
    let mean = fallback_chart(SiderealStarPlace::Mean);
    let apparent = fallback_chart(SiderealStarPlace::Apparent);
    let fallback: Vec<_> = apparent
        .mean_fallback_placements()
        .map(|p| p.body.clone())
        .collect();
    assert_eq!(fallback, vec![CelestialBody::Mars]);
    let lon = |s: &ChartSnapshot, i: usize| {
        s.placements[i]
            .position
            .ecliptic
            .unwrap()
            .longitude
            .degrees()
    };
    // Mars (index 1): bit-identical.
    assert_eq!(lon(&mean, 1), lon(&apparent, 1));
    assert_eq!(
        mean.placements[1].position.motion,
        apparent.placements[1].position.motion
    );
    // Sun (index 0): moved by the correction.
    let shift = -correction_deg(&Ayanamsa::TrueCitra, jd);
    assert!((wrap(lon(&apparent, 0) - lon(&mean, 0)) - shift).abs() < 1e-9);
}

#[test]
fn display_names_the_apparent_star_place_only_when_it_applies() {
    let mean = chart(&request(Ayanamsa::TrueCitra, SiderealStarPlace::Mean)).to_string();
    let apparent = chart(&request(Ayanamsa::TrueCitra, SiderealStarPlace::Apparent)).to_string();
    assert!(!mean.contains("Sidereal star place"));
    assert!(
        apparent.contains("Sidereal star place: apparent"),
        "{apparent}"
    );
    // Neither a mean chart nor an unanchored ayanamsa applies the correction,
    // so neither announces it.
    let mean_chart = chart(
        &request(Ayanamsa::TrueCitra, SiderealStarPlace::Apparent)
            .with_apparentness(Apparentness::Mean),
    )
    .to_string();
    assert!(!mean_chart.contains("Sidereal star place"), "{mean_chart}");
    let unanchored = chart(&request(Ayanamsa::Lahiri, SiderealStarPlace::Apparent)).to_string();
    assert!(!unanchored.contains("Sidereal star place"), "{unanchored}");
}

#[test]
fn the_request_summary_names_the_apparent_star_place_only_when_it_applies() {
    let suffix = "sidereal star place=apparent";
    let applies = request(Ayanamsa::TrueCitra, SiderealStarPlace::Apparent);
    assert!(applies.summary_line().contains(suffix));
    assert!(!request(Ayanamsa::TrueCitra, SiderealStarPlace::Mean)
        .summary_line()
        .contains(suffix));
    assert!(!applies
        .clone()
        .with_apparentness(Apparentness::Mean)
        .summary_line()
        .contains(suffix));
    assert!(!request(Ayanamsa::Lahiri, SiderealStarPlace::Apparent)
        .summary_line()
        .contains(suffix));
}

fn native_sidereal_request(ayanamsa: Ayanamsa, star_place: SiderealStarPlace) -> ChartRequest {
    ChartRequest::new(instant())
        .with_bodies(vec![CelestialBody::Sun])
        .with_zodiac_mode(ZodiacMode::Sidereal { ayanamsa })
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(13.0827),
            Longitude::from_degrees(80.2707),
            None,
        ))
        .with_house_system(HouseSystem::Placidus)
        .with_sidereal_star_place(star_place)
}

// Final review Important 1: a native-sidereal backend serves its placements on
// its own (mean) ayanamsa while the chart layer would correct the cusps, so
// the chart refuses rather than mixing the two zodiacs.
#[test]
fn a_native_sidereal_backend_refuses_the_apparent_star_place() {
    let engine = ChartEngine::new(NativeSiderealChartBackend);
    let error = engine
        .chart(&native_sidereal_request(
            Ayanamsa::TrueCitra,
            SiderealStarPlace::Apparent,
        ))
        .expect_err("native sidereal + apparent star place must fail closed");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedZodiacMode);
    assert!(error.message.contains("native-sidereal-chart"), "{error}");
    assert!(
        error.message.contains("anchor star's apparent place"),
        "{error}"
    );
}

#[test]
fn a_native_sidereal_backend_serves_the_mean_star_place_and_unanchored_ayanamsas() {
    let engine = ChartEngine::new(NativeSiderealChartBackend);
    for request in [
        native_sidereal_request(Ayanamsa::TrueCitra, SiderealStarPlace::Mean),
        native_sidereal_request(Ayanamsa::Lahiri, SiderealStarPlace::Apparent),
        // Issue #224: a mean chart never applies the star-place correction,
        // so the option is a no-op there and nothing mixes two zodiacs.
        native_sidereal_request(Ayanamsa::TrueCitra, SiderealStarPlace::Apparent)
            .with_apparentness(Apparentness::Mean),
    ] {
        let snapshot = engine.chart(&request).expect("chart succeeds");
        assert_eq!(snapshot.placements.len(), 1, "{request}");
        assert!(!snapshot.to_string().contains("Sidereal star place"));
    }
}
