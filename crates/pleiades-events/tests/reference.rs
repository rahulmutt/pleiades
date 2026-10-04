//! Crossings, longitudes and positions in the mean place of date and in a
//! sidereal zodiac (issue #88).

use pleiades_apparent::nutation::nutation;
use pleiades_ayanamsa::sidereal_offset;
use pleiades_core::{ChartEngine, ChartRequest};
use pleiades_data::packaged_backend;
use pleiades_events::{
    CrossingFrame, CrossingReference, EventEngine, EventError, WINDOW_END_JD, WINDOW_START_JD,
};
use pleiades_types::{
    Apparentness, Ayanamsa, CelestialBody, CustomAyanamsa, Instant, JulianDay, Longitude,
    TimeScale, ZodiacMode,
};

const APPARENT: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;
const MEAN: CrossingFrame = CrossingFrame::GeocentricMeanOfDate;
const HELIO: CrossingFrame = CrossingFrame::Heliocentric;

fn lahiri(frame: CrossingFrame) -> CrossingReference {
    CrossingReference::sidereal(frame, Ayanamsa::Lahiri)
}

fn ayanamsa_deg(ayanamsa: &Ayanamsa, jd: f64) -> f64 {
    sidereal_offset(
        ayanamsa,
        Instant::new(JulianDay::from_days(jd), TimeScale::Tt),
    )
    .expect("offset")
    .degrees()
}

/// Every new reference, for tests that must hold in all of them.
fn new_references() -> Vec<CrossingReference> {
    vec![
        CrossingReference::tropical(MEAN),
        lahiri(APPARENT),
        lahiri(MEAN),
        CrossingReference::sidereal(APPARENT, Ayanamsa::TrueCitra),
        CrossingReference::sidereal(MEAN, Ayanamsa::GalacticCenter),
    ]
}

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn wrap(deg: f64) -> f64 {
    (deg + 180.0).rem_euclid(360.0) - 180.0
}

#[test]
fn sun_mean_of_date_differs_from_apparent_by_aberration_and_nutation() {
    // The Sun's apparent place is its geometric place moved by annual
    // aberration (-20.4898"/R, R in AU) and referred to the true equinox
    // (+Δψ). This checks the mean frame against two independently known
    // quantities, not against the engine's own formula.
    let engine = EventEngine::new(packaged_backend());
    for jd in [2_420_000.5, 2_451_545.0, 2_460_000.5, 2_480_000.5] {
        let apparent = engine
            .position_at(CelestialBody::Sun, APPARENT, tdb(jd))
            .unwrap();
        let mean = engine
            .longitude_at(CelestialBody::Sun, MEAN, tdb(jd))
            .unwrap();
        let got_arcsec = wrap(apparent.ecliptic.longitude.degrees() - mean.degrees()) * 3600.0;
        let delta_psi = nutation(jd).unwrap().delta_psi_arcsec;
        let aberration = -20.4898 / apparent.ecliptic.distance_au.unwrap();
        assert!(
            (got_arcsec - (delta_psi + aberration)).abs() < 0.2,
            "jd {jd}: apparent-mean {got_arcsec}\" vs Δψ {delta_psi}\" + aberration {aberration}\""
        );
    }
}

#[test]
fn mean_frame_reads_succeed_at_the_range_start() {
    // No light-time re-query, so the range-start failure of FU-17(c) does
    // not apply to the mean frame.
    let engine = EventEngine::new(packaged_backend());
    for body in [CelestialBody::Mars, CelestialBody::Moon] {
        engine
            .longitude_at(body.clone(), MEAN, tdb(WINDOW_START_JD))
            .unwrap_or_else(|e| panic!("{body:?}: {e}"));
        engine
            .position_at(body.clone(), MEAN, tdb(WINDOW_START_JD))
            .unwrap_or_else(|e| panic!("{body:?}: {e}"));
    }
}

#[test]
fn mean_position_longitude_is_bit_identical_to_longitude_at() {
    let engine = EventEngine::new(packaged_backend());
    for body in [
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mars,
        CelestialBody::Pluto,
    ] {
        for jd in [2_420_000.5, 2_451_545.0, 2_480_000.5] {
            let pos = engine.position_at(body.clone(), MEAN, tdb(jd)).unwrap();
            let lon = engine.longitude_at(body.clone(), MEAN, tdb(jd)).unwrap();
            assert_eq!(
                pos.ecliptic.longitude.degrees().to_bits(),
                lon.degrees().to_bits(),
                "{body:?} {jd}"
            );
            assert_eq!(pos.frame, MEAN);
        }
    }
}

#[test]
fn mean_speed_matches_a_central_difference_of_the_longitude() {
    let engine = EventEngine::new(packaged_backend());
    let h = 0.5;
    for body in [
        CelestialBody::Sun,
        CelestialBody::Mars,
        CelestialBody::Jupiter,
    ] {
        let jd = 2_455_000.5;
        let before = engine
            .longitude_at(body.clone(), MEAN, tdb(jd - h))
            .unwrap();
        let after = engine
            .longitude_at(body.clone(), MEAN, tdb(jd + h))
            .unwrap();
        let rate = wrap(after.degrees() - before.degrees()) / (2.0 * h);
        let speed = engine
            .position_at(body.clone(), MEAN, tdb(jd))
            .unwrap()
            .motion
            .longitude_deg_per_day
            .expect("speed");
        assert!(
            (speed - rate).abs() < 1e-4,
            "{body:?}: speed {speed} vs {rate}"
        );
    }
}

#[test]
fn mean_crossing_lands_on_the_target_from_the_settled_side() {
    // The returned instant trails the crossing by < 0.5 s and never precedes
    // it. Both bodies move direct, so the residual is in [0, speed × 0.5 s).
    let engine = EventEngine::new(packaged_backend());
    for (body, bound_deg) in [(CelestialBody::Sun, 1.0e-5), (CelestialBody::Moon, 1.0e-4)] {
        for target_deg in [0.0, 137.5, 359.9] {
            let crossing = engine
                .next_longitude_crossing(
                    body.clone(),
                    Longitude::from_degrees(target_deg),
                    MEAN,
                    tdb(2_451_545.0),
                )
                .unwrap()
                .expect("crossing");
            assert_eq!(crossing.frame, MEAN);
            let lon = engine
                .longitude_at(body.clone(), MEAN, crossing.instant)
                .unwrap();
            let residual = wrap(lon.degrees() - target_deg);
            assert!(
                (0.0..bound_deg).contains(&residual),
                "{body:?} target {target_deg}: residual {residual}"
            );
        }
    }
}

#[test]
fn a_frame_converts_to_the_tropical_reference() {
    let reference: CrossingReference = APPARENT.into();
    assert_eq!(reference.frame, APPARENT);
    assert_eq!(reference.zodiac, ZodiacMode::Tropical);
    assert_eq!(reference, CrossingReference::tropical(APPARENT));
    assert_eq!(
        lahiri(MEAN).zodiac,
        ZodiacMode::Sidereal {
            ayanamsa: Ayanamsa::Lahiri
        }
    );
}

#[test]
fn sidereal_longitude_is_the_mean_equinox_longitude_minus_the_ayanamsa() {
    let engine = EventEngine::new(packaged_backend());
    for body in [
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Saturn,
    ] {
        for jd in [2_420_000.5, 2_451_545.0, 2_480_000.5] {
            let ayanamsa = ayanamsa_deg(&Ayanamsa::Lahiri, jd);
            let delta_psi_deg = nutation(jd).unwrap().delta_psi_arcsec / 3600.0;

            let apparent = engine
                .longitude_at(body.clone(), APPARENT, tdb(jd))
                .unwrap();
            let sidereal = engine
                .longitude_at(body.clone(), lahiri(APPARENT), tdb(jd))
                .unwrap();
            let expected = apparent.degrees() - delta_psi_deg - ayanamsa;
            assert!(
                wrap(sidereal.degrees() - expected).abs() < 1e-10,
                "{body:?} {jd} apparent sidereal"
            );

            let mean = engine.longitude_at(body.clone(), MEAN, tdb(jd)).unwrap();
            let sidereal = engine
                .longitude_at(body.clone(), lahiri(MEAN), tdb(jd))
                .unwrap();
            assert!(
                wrap(sidereal.degrees() - (mean.degrees() - ayanamsa)).abs() < 1e-10,
                "{body:?} {jd} mean sidereal"
            );
            assert!((0.0..360.0).contains(&sidereal.degrees()));
        }
    }
}

#[test]
fn heliocentric_sidereal_is_unsupported() {
    let engine = EventEngine::new(packaged_backend());
    let t = tdb(2_451_545.0);
    let target = Longitude::from_degrees(0.0);
    let reference = lahiri(HELIO);
    for err in [
        engine
            .longitude_at(CelestialBody::Mars, &reference, t)
            .unwrap_err(),
        engine
            .position_at(CelestialBody::Mars, &reference, t)
            .unwrap_err(),
        engine
            .next_longitude_crossing(CelestialBody::Mars, target, &reference, t)
            .unwrap_err(),
        engine
            .previous_longitude_crossing(CelestialBody::Mars, target, &reference, t)
            .unwrap_err(),
        engine
            .longitude_crossings_in_range(
                CelestialBody::Mars,
                target,
                &reference,
                t,
                tdb(2_451_645.0),
            )
            .unwrap_err(),
    ] {
        assert!(
            matches!(err, EventError::UnsupportedFrame { .. }),
            "{err:?}"
        );
    }
}

#[test]
fn an_ayanamsa_without_offset_data_is_unsupported() {
    // No epoch, no offset: `sidereal_offset` returns None. There must be no
    // silent tropical fallback.
    let engine = EventEngine::new(packaged_backend());
    let t = tdb(2_451_545.0);
    let reference =
        CrossingReference::sidereal(APPARENT, Ayanamsa::Custom(CustomAyanamsa::new("no data")));
    for err in [
        engine
            .longitude_at(CelestialBody::Sun, &reference, t)
            .unwrap_err(),
        engine
            .position_at(CelestialBody::Sun, &reference, t)
            .unwrap_err(),
        engine
            .next_longitude_crossing(
                CelestialBody::Sun,
                Longitude::from_degrees(0.0),
                &reference,
                t,
            )
            .unwrap_err(),
        engine
            .previous_longitude_crossing(
                CelestialBody::Sun,
                Longitude::from_degrees(0.0),
                &reference,
                t,
            )
            .unwrap_err(),
        engine
            .longitude_crossings_in_range(
                CelestialBody::Sun,
                Longitude::from_degrees(0.0),
                &reference,
                t,
                tdb(2_451_645.0),
            )
            .unwrap_err(),
    ] {
        assert!(
            matches!(err, EventError::UnsupportedFrame { .. }),
            "{err:?}"
        );
    }
}

#[test]
fn a_non_finite_custom_ayanamsa_offset_is_unsupported() {
    // A NaN offset would otherwise surface as `Ok(NaN)`.
    let engine = EventEngine::new(packaged_backend());
    let mut custom = CustomAyanamsa::new("nan offset");
    custom.epoch = Some(JulianDay::from_days(2_451_545.0));
    custom.offset_degrees = Some(pleiades_types::Angle::from_degrees(f64::NAN));
    let reference = CrossingReference::sidereal(MEAN, Ayanamsa::Custom(custom));
    let err = engine
        .longitude_at(CelestialBody::Sun, &reference, tdb(2_451_545.0))
        .unwrap_err();
    assert!(
        matches!(err, EventError::UnsupportedFrame { .. }),
        "{err:?}"
    );
}

#[test]
fn a_custom_ayanamsa_with_epoch_and_offset_works() {
    let engine = EventEngine::new(packaged_backend());
    let mut custom = CustomAyanamsa::new("fixed 24 at J2000");
    custom.epoch = Some(JulianDay::from_days(2_451_545.0));
    custom.offset_degrees = Some(pleiades_types::Angle::from_degrees(24.0));
    let reference = CrossingReference::sidereal(MEAN, Ayanamsa::Custom(custom));
    let jd = 2_451_545.0;
    let mean = engine
        .longitude_at(CelestialBody::Sun, MEAN, tdb(jd))
        .unwrap();
    let sidereal = engine
        .longitude_at(CelestialBody::Sun, &reference, tdb(jd))
        .unwrap();
    assert!(wrap(sidereal.degrees() - (mean.degrees() - 24.0)).abs() < 1e-9);
}

#[test]
fn crossings_land_on_the_target_from_the_settled_side_in_every_new_reference() {
    let engine = EventEngine::new(packaged_backend());
    for reference in new_references() {
        for (body, bound_deg) in [(CelestialBody::Sun, 1.0e-5), (CelestialBody::Moon, 1.0e-4)] {
            // 0.0 and 359.9 sit on the wrap seam of the shifted longitude.
            for target_deg in [0.0, 137.5, 359.9] {
                let crossing = engine
                    .next_longitude_crossing(
                        body.clone(),
                        Longitude::from_degrees(target_deg),
                        &reference,
                        tdb(2_451_545.0),
                    )
                    .unwrap()
                    .expect("crossing");
                assert_eq!(crossing.frame, reference.frame);
                assert_eq!(crossing.zodiac, reference.zodiac);
                let lon = engine
                    .longitude_at(body.clone(), &reference, crossing.instant)
                    .unwrap();
                let residual = wrap(lon.degrees() - target_deg);
                assert!(
                    (0.0..bound_deg).contains(&residual),
                    "{reference:?} {body:?} target {target_deg}: residual {residual}"
                );
            }
        }
    }
}

#[test]
fn mars_2003_loop_gives_three_sidereal_crossings() {
    // Tropical 337.0° is crossed direct, retrograde, direct in the 2003
    // opposition loop (the corpus's triple crossing). The same point in the
    // Lahiri zodiac is 337.0° minus the ayanamsa.
    let engine = EventEngine::new(packaged_backend());
    let target = Longitude::from_degrees(337.0 - ayanamsa_deg(&Ayanamsa::Lahiri, 2_452_850.0));
    let crossings = engine
        .longitude_crossings_in_range(
            CelestialBody::Mars,
            target,
            lahiri(APPARENT),
            tdb(2_452_791.5),
            tdb(2_453_000.5),
        )
        .unwrap();
    assert_eq!(crossings.len(), 3, "{crossings:?}");
    assert!(crossings
        .windows(2)
        .all(|pair| pair[1].instant.julian_day.days() > pair[0].instant.julian_day.days() + 1.0));
}

#[test]
fn next_after_a_returned_sidereal_crossing_is_the_following_one() {
    let engine = EventEngine::new(packaged_backend());
    let reference = lahiri(APPARENT);
    for target_deg in [7.3, 127.3, 247.3] {
        let target = Longitude::from_degrees(target_deg);
        let found = engine
            .next_longitude_crossing(CelestialBody::Moon, target, &reference, tdb(2_451_545.0))
            .unwrap()
            .expect("the Moon crosses every longitude monthly");
        let following = engine
            .next_longitude_crossing(CelestialBody::Moon, target, &reference, found.instant)
            .unwrap()
            .expect("the Moon crosses every longitude monthly");
        let gap = following.instant.julian_day.days() - found.instant.julian_day.days();
        assert!(
            (27.0..28.0).contains(&gap),
            "target {target_deg}: gap {gap} days"
        );
    }
}

#[test]
fn previous_equals_the_last_crossing_in_range_in_the_new_references() {
    let engine = EventEngine::new(packaged_backend());
    let target = Longitude::from_degrees(100.0);
    let before = tdb(WINDOW_START_JD + 800.0);
    for reference in [
        CrossingReference::tropical(MEAN),
        lahiri(APPARENT),
        lahiri(MEAN),
    ] {
        let all = engine
            .longitude_crossings_in_range(
                CelestialBody::Sun,
                target,
                &reference,
                tdb(WINDOW_START_JD),
                before,
            )
            .unwrap();
        assert!(all.len() >= 2, "{reference:?}: {}", all.len());
        let last = all.last().unwrap();
        let previous = engine
            .previous_longitude_crossing(CelestialBody::Sun, target, &reference, before)
            .unwrap()
            .expect("previous crossing");
        assert!(
            (previous.instant.julian_day.days() - last.instant.julian_day.days()).abs() < 1e-6,
            "{reference:?}"
        );
    }
}

#[test]
fn position_longitude_is_bit_identical_to_longitude_at_in_every_new_reference() {
    let engine = EventEngine::new(packaged_backend());
    for reference in new_references() {
        for body in [CelestialBody::Sun, CelestialBody::Moon, CelestialBody::Mars] {
            for jd in [2_420_000.5, 2_451_545.0, 2_480_000.5] {
                let pos = engine
                    .position_at(body.clone(), &reference, tdb(jd))
                    .unwrap();
                let lon = engine
                    .longitude_at(body.clone(), &reference, tdb(jd))
                    .unwrap();
                assert_eq!(
                    pos.ecliptic.longitude.degrees().to_bits(),
                    lon.degrees().to_bits(),
                    "{reference:?} {body:?} {jd}"
                );
                assert_eq!(pos.frame, reference.frame);
                assert_eq!(pos.zodiac, reference.zodiac);
            }
        }
    }
}

#[test]
fn sidereal_speed_is_the_tropical_speed_minus_the_ayanamsa_rate() {
    // Lahiri drifts with general precession, 3.82e-5 deg/day. In the apparent
    // frame the removed nutation also changes the speed by the Δψ rate,
    // differenced over the same ±0.5 day as the engine's speed.
    const PRECESSION_DEG_PER_DAY: f64 = 3.82e-5;
    let engine = EventEngine::new(packaged_backend());
    let speed = |reference: CrossingReference, jd: f64| {
        engine
            .position_at(CelestialBody::Mars, reference, tdb(jd))
            .unwrap()
            .motion
            .longitude_deg_per_day
            .expect("speed")
    };
    for jd in [2_430_000.5, 2_451_545.0, 2_470_000.5] {
        let mean_drop = speed(MEAN.into(), jd) - speed(lahiri(MEAN), jd);
        assert!(
            (mean_drop - PRECESSION_DEG_PER_DAY).abs() < 2.0e-6,
            "jd {jd}: mean drop {mean_drop:e}"
        );
        let apparent_drop = speed(APPARENT.into(), jd) - speed(lahiri(APPARENT), jd);
        let delta_psi_rate = (nutation(jd + 0.5).unwrap().delta_psi_arcsec
            - nutation(jd - 0.5).unwrap().delta_psi_arcsec)
            / 3600.0;
        let expected = PRECESSION_DEG_PER_DAY + delta_psi_rate;
        assert!(
            (apparent_drop - expected).abs() < 2.0e-6,
            "jd {jd}: apparent drop {apparent_drop:e} vs expected {expected:e}"
        );
    }
}

#[test]
fn sidereal_position_has_a_speed_at_the_window_end() {
    // The later neighbour is outside the window, so the difference is
    // one-sided; the speed must still be reported.
    let engine = EventEngine::new(packaged_backend());
    let pos = engine
        .position_at(CelestialBody::Sun, lahiri(MEAN), tdb(WINDOW_END_JD))
        .unwrap();
    assert!(pos.motion.longitude_deg_per_day.is_some());
}

#[test]
fn one_reference_serves_many_calls_by_borrow() {
    let engine = EventEngine::new(packaged_backend());
    let reference = lahiri(APPARENT);
    let count = [0.0, 90.0, 180.0, 270.0]
        .into_iter()
        .filter_map(|target_deg| {
            engine
                .next_longitude_crossing(
                    CelestialBody::Sun,
                    Longitude::from_degrees(target_deg),
                    &reference,
                    tdb(2_451_545.0),
                )
                .unwrap()
        })
        .count();
    assert_eq!(count, 4);
}

#[test]
fn sidereal_apparent_chart_agrees_with_the_crossing_reference() {
    // Issue #120: both read a sidereal longitude on the mean equinox of date
    // minus the mean ayanamsa, so a crossing the engine finds sits on the
    // chart's own longitude. The mean chart still differs (FU-18 (b)).
    let engine = EventEngine::new(packaged_backend());
    let chart_engine = ChartEngine::new(packaged_backend());
    let bodies = [
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mars,
        CelestialBody::Saturn,
    ];
    for jd in [2_420_000.5, 2_451_545.0, 2_460_000.5, 2_480_000.5] {
        let chart = chart_engine
            .chart(
                &ChartRequest::new(tdb(jd))
                    .with_bodies(bodies.to_vec())
                    .with_zodiac_mode(ZodiacMode::Sidereal {
                        ayanamsa: Ayanamsa::Lahiri,
                    }),
            )
            .expect("chart");
        for body in &bodies {
            let chart_lon = chart
                .placement_for(body)
                .expect("placed")
                .position
                .ecliptic
                .as_ref()
                .expect("ecliptic")
                .longitude
                .degrees();
            let engine_lon = engine
                .longitude_at(body.clone(), lahiri(APPARENT), tdb(jd))
                .unwrap()
                .degrees();
            let arcsec = wrap(chart_lon - engine_lon) * 3600.0;
            assert!(
                arcsec.abs() < 0.001,
                "jd {jd} {body:?}: chart − crossing reference = {arcsec:.4}\""
            );
        }
    }
}

/// Diagnostic for FU-18 (b): how a sidereal mean chart differs from a
/// sidereal crossing reference (the apparent case is asserted above). Run with
/// `cargo test -p pleiades-events --test reference measure_chart_sidereal_conventions -- --nocapture --ignored`
#[test]
#[ignore]
fn measure_chart_sidereal_conventions() {
    let engine = EventEngine::new(packaged_backend());
    let zodiac = ZodiacMode::Sidereal {
        ayanamsa: Ayanamsa::Lahiri,
    };
    for jd in [2_420_000.5, 2_451_545.0, 2_460_000.5, 2_480_000.5] {
        for (label, apparentness, frame) in [
            ("apparent", Apparentness::Apparent, APPARENT),
            ("mean", Apparentness::Mean, MEAN),
        ] {
            let request = ChartRequest::new(tdb(jd))
                .with_bodies(vec![CelestialBody::Sun])
                .with_apparentness(apparentness)
                .with_zodiac_mode(zodiac.clone());
            let chart = ChartEngine::new(packaged_backend())
                .chart(&request)
                .expect("chart");
            let chart_lon = chart
                .placement_for(&CelestialBody::Sun)
                .expect("placed")
                .position
                .ecliptic
                .as_ref()
                .expect("ecliptic")
                .longitude
                .degrees();
            let engine_lon = engine
                .longitude_at(CelestialBody::Sun, lahiri(frame), tdb(jd))
                .unwrap()
                .degrees();
            let delta_psi = nutation(jd).unwrap().delta_psi_arcsec;
            eprintln!(
                "jd {jd} {label}: chart - crossing = {:.3}\" (Δψ = {delta_psi:.3}\")",
                wrap(chart_lon - engine_lon) * 3600.0
            );
        }
    }
}
