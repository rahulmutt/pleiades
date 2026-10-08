use super::*;

#[test]
fn default_options_are_upper_limb_refracted() {
    let o = RiseSetOptions::default();
    assert!(matches!(o.disc, DiscMode::UpperLimb));
    assert!(o.refraction);
    assert!(!o.hindu && !o.no_ecl_lat && !o.fixed_disc_size);
    assert!(o.horizon_altitude_deg.is_none());
}

#[test]
fn target_equatorial_matches_horizontal_for_a_star() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let obs = ObserverLocation::new(
        Latitude::from_degrees(40.0),
        Longitude::from_degrees(0.0),
        None,
    );
    let jd = 2_451_545.0;
    let (ra, dec) = engine
        .target_equatorial(
            &RiseSetTarget::FixedStar("Aldebaran".into()),
            &obs,
            &RiseSetOptions::default(),
            jd,
            None,
        )
        .unwrap();
    let equ = crate::fixstar::fixed_star_apparent(
        "Aldebaran",
        Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
    )
    .unwrap();
    assert!((ra - equ.right_ascension.degrees()).abs() < 1e-9);
    assert!((dec - equ.declination.degrees()).abs() < 1e-9);
}

#[test]
fn ecliptic_point_no_ecl_lat_forces_latitude_zero() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{Latitude, Longitude, ObserverLocation};
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let obs = ObserverLocation::new(
        Latitude::from_degrees(40.0),
        Longitude::from_degrees(0.0),
        None,
    );
    let opts = RiseSetOptions {
        no_ecl_lat: true,
        ..RiseSetOptions::default()
    };
    // A point at longitude 90° latitude 30° with no_ecl_lat behaves like latitude 0.
    let (_, dec_forced) = engine
        .target_equatorial(
            &RiseSetTarget::EclipticPoint(
                Longitude::from_degrees(90.0),
                Latitude::from_degrees(30.0),
            ),
            &obs,
            &opts,
            2_451_545.0,
            None,
        )
        .unwrap();
    let (_, dec_zero) = engine
        .target_equatorial(
            &RiseSetTarget::EclipticPoint(
                Longitude::from_degrees(90.0),
                Latitude::from_degrees(0.0),
            ),
            &obs,
            &opts,
            2_451_545.0,
            None,
        )
        .unwrap();
    assert!(
        (dec_forced - dec_zero).abs() < 1e-9,
        "no_ecl_lat should ignore supplied latitude"
    );
}

/// `no_ecl_lat` is SE's `SE_BIT_GEOCTR_NO_ECL_LAT`: a body is placed
/// geocentrically (no diurnal parallax or aberration) with latitude 0, so its
/// equatorial place is the geocentric apparent ecliptic place at latitude 0
/// rotated by the true obliquity (issue #241).
#[test]
fn body_no_ecl_lat_is_geocentric_with_latitude_zero() {
    use super::test_support::{chennai, composite};
    let engine = EventEngine::new(composite());
    let jd = 2_460_827.75;
    let opts = RiseSetOptions {
        no_ecl_lat: true,
        ..RiseSetOptions::default()
    };
    for body in [CelestialBody::Sun, CelestialBody::Moon] {
        let (ra, dec) = engine
            .target_equatorial(
                &RiseSetTarget::Body(body.clone()),
                &chennai(),
                &opts,
                jd,
                None,
            )
            .unwrap();
        let (lon, _, _) =
            geocentric_apparent_ecliptic(&engine.backend, body.clone(), "body", jd).unwrap();
        let eps = true_obliquity_degrees(jd).unwrap();
        let equ = EclipticCoordinates::new(
            Longitude::from_degrees(lon),
            Latitude::from_degrees(0.0),
            None,
        )
        .to_equatorial(Angle::from_degrees(eps));
        assert!(
            (ra - equ.right_ascension.degrees()).abs() < 1e-9
                && (dec - equ.declination.degrees()).abs() < 1e-9,
            "{body:?}: ({ra}, {dec}) is not the geocentric place ({}, {})",
            equ.right_ascension.degrees(),
            equ.declination.degrees()
        );
    }
}

/// A Hindu moonrise is found on the geocentric place: at the returned
/// instant the geocentric latitude-0 Moon centre has just reached the
/// horizon, while the same place corrected for diurnal parallax still sits
/// about one horizontal parallax (~57′) below it. Before issue #241 the
/// search rooted the topocentric place instead.
#[test]
fn hindu_moonrise_roots_the_geocentric_place() {
    use super::test_support::{chennai, composite, tt};
    let engine = EventEngine::new(composite());
    let observer = chennai();
    let hindu = RiseSetOptions {
        hindu: true,
        ..RiseSetOptions::default()
    };
    let jd = engine
        .next_rise_set(
            RiseSetTarget::Body(CelestialBody::Moon),
            RiseSetEvent::Rise,
            observer.clone(),
            Atmosphere::default(),
            hindu,
            tt(2_460_827.75),
        )
        .expect("engine ok")
        .expect("a moonrise")
        .instant
        .julian_day
        .days();
    let eps = true_obliquity_degrees(jd).unwrap();
    let lst = local_apparent_sidereal_deg(jd, observer.longitude).unwrap();
    let (lon, _, dist) =
        geocentric_apparent_ecliptic(&engine.backend, CelestialBody::Moon, "body", jd).unwrap();
    let geocentric = EclipticCoordinates::new(
        Longitude::from_degrees(lon),
        Latitude::from_degrees(0.0),
        Some(dist),
    );
    let topocentric = topocentric_position(geocentric, &observer, lst, eps)
        .unwrap()
        .ecliptic;
    let altitude = |ecl: EclipticCoordinates| {
        let equ = ecl.to_equatorial(Angle::from_degrees(eps));
        let (phi, dec) = (
            observer.latitude.degrees().to_radians(),
            equ.declination.degrees().to_radians(),
        );
        let ha = (lst - equ.right_ascension.degrees()).to_radians();
        (phi.sin() * dec.sin() + phi.cos() * dec.cos() * ha.cos())
            .asin()
            .to_degrees()
    };
    // Settled within 0.5 s after the crossing: the Moon climbs at most
    // ~15″/s, so the geocentric centre is above the horizon by under 8″.
    let geo_alt = altitude(geocentric);
    assert!(
        (0.0..8.0 / 3600.0).contains(&geo_alt),
        "geocentric {geo_alt}°"
    );
    let topo_alt = altitude(topocentric);
    assert!(
        (-1.05..-0.85).contains(&topo_alt),
        "topocentric {topo_alt}°"
    );
}

#[test]
fn standard_altitude_sun_upper_limb_is_about_negative_semidiameter() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let obs = ObserverLocation::new(
        Latitude::from_degrees(40.0),
        Longitude::from_degrees(0.0),
        None,
    );
    let opts = RiseSetOptions::default(); // upper limb + refraction
    let h0 = engine
        .standard_altitude(
            &RiseSetTarget::Body(pleiades_types::CelestialBody::Sun),
            &obs,
            &opts,
            pleiades_apparent::Atmosphere::default(),
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb)
                .julian_day
                .days(),
            None,
        )
        .unwrap();
    // Model B (SE `swe_rise_trans`): refraction lives in the apparent
    // altitude that this h0 is compared against, not in h0 itself. For
    // upper-limb, h0 is just −SD ≈ −0.2666° (observed: −0.26657°).
    assert!((h0 + 0.2666).abs() < 0.02, "sun standard altitude {h0}");
}

#[test]
fn sun_rises_and_sets_within_a_day() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{
        CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
    };
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let obs = ObserverLocation::new(
        Latitude::from_degrees(40.0),
        Longitude::from_degrees(0.0),
        None,
    );
    let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    let rise = engine
        .next_rise_set(
            RiseSetTarget::Body(CelestialBody::Sun),
            RiseSetEvent::Rise,
            obs.clone(),
            Atmosphere::default(),
            RiseSetOptions::default(),
            after,
        )
        .unwrap();
    let set = engine
        .next_rise_set(
            RiseSetTarget::Body(CelestialBody::Sun),
            RiseSetEvent::Set,
            obs.clone(),
            Atmosphere::default(),
            RiseSetOptions::default(),
            after,
        )
        .unwrap();
    let rise = rise.expect("a rise within the window");
    let set = set.expect("a set within the window");
    // At the rise instant the apparent altitude equals the standard altitude.
    let jd = rise.instant.julian_day.days();
    let alt = engine
        .target_apparent_altitude(
            &RiseSetTarget::Body(CelestialBody::Sun),
            &obs,
            &RiseSetOptions::default(),
            Atmosphere::default(),
            jd,
            None,
        )
        .unwrap();
    let h0 = engine
        .standard_altitude(
            &RiseSetTarget::Body(CelestialBody::Sun),
            &obs,
            &RiseSetOptions::default(),
            Atmosphere::default(),
            jd,
            None,
        )
        .unwrap();
    assert!((alt - h0).abs() < 1e-3, "altitude {alt} vs h0 {h0} at rise");
    assert!(set.instant.julian_day.days() != rise.instant.julian_day.days());
}

/// Regression for the rise-vs-set DIRECTION, not merely that rise != set.
/// `sun_rises_and_sets_within_a_day` above would still pass if
/// the scanner's bracket-sign direction test were reversed (rise/set labels
/// swapped), since it only checks `alt ≈ h0` and `rise != set`. Here we sample the residual
/// (`target_apparent_altitude - standard_altitude`) just before and just
/// after each event and assert the sign change goes the correct way: rise
/// must be ASCENDING (below -> above), set must be DESCENDING (above ->
/// below). A reversed classifier fails these assertions.
#[test]
fn rise_is_ascending_and_set_is_descending() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{
        CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
    };
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let obs = ObserverLocation::new(
        Latitude::from_degrees(40.0),
        Longitude::from_degrees(0.0),
        None,
    );
    let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    let target = RiseSetTarget::Body(CelestialBody::Sun);
    let opts = RiseSetOptions::default();
    let atmos = Atmosphere::default();

    let rise = engine
        .next_rise_set(
            target.clone(),
            RiseSetEvent::Rise,
            obs.clone(),
            atmos,
            opts.clone(),
            after,
        )
        .unwrap()
        .expect("a rise within the window");
    let set = engine
        .next_rise_set(
            target.clone(),
            RiseSetEvent::Set,
            obs.clone(),
            atmos,
            opts.clone(),
            after,
        )
        .unwrap()
        .expect("a set within the window");

    // 120s: well outside the bisection's REFINE_TOLERANCE_DAYS (0.5s) noise
    // floor, but tiny compared to the ~12h spacing between consecutive
    // rise/set events, so it stays within the same monotonic segment of the
    // residual.
    const DT: f64 = 120.0 / 86_400.0;
    let resid = |jd: f64| {
        engine
            .horizon_residual(&target, &obs, &opts, atmos, jd, None)
            .unwrap()
    };

    let rise_jd = rise.instant.julian_day.days();
    assert!(
        resid(rise_jd - DT) < 0.0,
        "expected below horizon just before rise"
    );
    assert!(
        resid(rise_jd + DT) > 0.0,
        "expected above horizon just after rise"
    );

    let set_jd = set.instant.julian_day.days();
    assert!(
        resid(set_jd - DT) > 0.0,
        "expected above horizon just before set"
    );
    assert!(
        resid(set_jd + DT) < 0.0,
        "expected below horizon just after set"
    );
}

#[test]
fn upper_transit_puts_body_on_the_meridian() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{
        CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
    };
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let obs = ObserverLocation::new(
        Latitude::from_degrees(40.0),
        Longitude::from_degrees(0.0),
        None,
    );
    let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    let t = engine
        .next_rise_set(
            RiseSetTarget::Body(CelestialBody::Sun),
            RiseSetEvent::UpperTransit,
            obs.clone(),
            Atmosphere::default(),
            RiseSetOptions::default(),
            after,
        )
        .unwrap()
        .expect("a transit");
    // At upper transit the local hour angle H = LST − RA ≈ 0, with sidereal
    // time evaluated at the UT1 re-expression of the returned TDB instant.
    let jd = t.instant.julian_day.days();
    let (ra, _dec) = engine
        .target_equatorial(
            &RiseSetTarget::Body(CelestialBody::Sun),
            &obs,
            &RiseSetOptions::default(),
            jd,
            None,
        )
        .unwrap();
    let ut1 =
        pleiades_apparent::ut1_instant(Instant::new(JulianDay::from_days(jd), TimeScale::Tdb))
            .unwrap();
    let lst =
        pleiades_apparent::sidereal_time(ut1, Longitude::from_degrees(0.0)).local_apparent_deg;
    let ha = crate::root::wrap180(lst - ra);
    assert!(ha.abs() < 0.05, "hour angle at upper transit {ha} deg");
}

#[test]
fn standard_altitude_no_refraction_center_is_zero() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let obs = ObserverLocation::new(
        Latitude::from_degrees(40.0),
        Longitude::from_degrees(0.0),
        None,
    );
    let opts = RiseSetOptions {
        disc: DiscMode::Center,
        refraction: false,
        ..RiseSetOptions::default()
    };
    let h0 = engine
        .standard_altitude(
            &RiseSetTarget::FixedStar("Sirius".into()),
            &obs,
            &opts,
            pleiades_apparent::Atmosphere::default(),
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb)
                .julian_day
                .days(),
            None,
        )
        .unwrap();
    assert!(h0.abs() < 1e-9, "no-refraction center h0 {h0}");
}

#[test]
fn circumpolar_high_latitude_returns_none() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{
        CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
    };
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    // Near the pole a mid-declination body may never cross the horizon in a day.
    let obs = ObserverLocation::new(
        Latitude::from_degrees(89.9),
        Longitude::from_degrees(0.0),
        None,
    );
    let start = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    let end = Instant::new(JulianDay::from_days(2_451_545.5), TimeScale::Tdb);
    let out = engine
        .rise_sets_in_range(
            RiseSetTarget::Body(CelestialBody::Sun),
            RiseSetEvent::Rise,
            obs,
            Atmosphere::default(),
            RiseSetOptions::default(),
            start,
            end,
        )
        .unwrap();
    assert!(
        out.is_empty(),
        "circumpolar: no rise expected, got {}",
        out.len()
    );
}

#[test]
fn circumpolar_next_rise_set_returns_none_not_far_future() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{
        CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
    };
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    // Near the pole the Sun is circumpolar for weeks/months at a time; it
    // may still rise eventually (near the equinox), but not within
    // `RISE_SET_SEARCH_SPAN_DAYS`. Mirrors `circumpolar_high_latitude_returns_none`
    // (which uses `rise_sets_in_range` over an explicit short window) but
    // exercises `next_rise_set`'s own bounded-search behavior instead.
    let obs = ObserverLocation::new(
        Latitude::from_degrees(89.9),
        Longitude::from_degrees(0.0),
        None,
    );
    let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    let out = engine
        .next_rise_set(
            RiseSetTarget::Body(CelestialBody::Sun),
            RiseSetEvent::Rise,
            obs,
            Atmosphere::default(),
            RiseSetOptions::default(),
            after,
        )
        .unwrap();
    assert!(
        out.is_none(),
        "circumpolar-now Sun should return None within the bounded search span, got {out:?}"
    );
}

#[test]
fn out_of_window_and_bad_atmosphere_fail_closed() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{
        CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
    };
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let obs = ObserverLocation::new(
        Latitude::from_degrees(40.0),
        Longitude::from_degrees(0.0),
        None,
    );
    let err = engine
        .next_rise_set(
            RiseSetTarget::Body(CelestialBody::Sun),
            RiseSetEvent::Rise,
            obs.clone(),
            Atmosphere::default(),
            RiseSetOptions::default(),
            Instant::new(JulianDay::from_days(2_000_000.0), TimeScale::Tdb),
        )
        .unwrap_err();
    assert!(matches!(err, EventError::OutOfWindow { .. }));

    let bad = Atmosphere {
        pressure_mbar: f64::NAN,
        temperature_c: 15.0,
    };
    let err = engine
        .next_rise_set(
            RiseSetTarget::Body(CelestialBody::Sun),
            RiseSetEvent::Rise,
            obs,
            bad,
            RiseSetOptions::default(),
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb),
        )
        .unwrap_err();
    assert!(matches!(err, EventError::InvalidAtmosphere { .. }));
}

#[test]
fn unknown_star_target_fails_closed() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let obs = ObserverLocation::new(
        Latitude::from_degrees(40.0),
        Longitude::from_degrees(0.0),
        None,
    );
    let err = engine
        .next_rise_set(
            RiseSetTarget::FixedStar("Nope".into()),
            RiseSetEvent::Rise,
            obs,
            Atmosphere::default(),
            RiseSetOptions::default(),
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb),
        )
        .unwrap_err();
    assert!(matches!(err, EventError::UnknownFixedStar { .. }));
}

/// Shared arrange block for the `previous_rise_set` tests: the linear
/// Sun/Moon test backend and a mid-latitude observer, as the tests above use.
fn mid_latitude_fixture() -> (
    EventEngine<pleiades_backend::test_backend::LinearSunMoon>,
    ObserverLocation,
) {
    use pleiades_backend::test_backend::LinearSunMoon;
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let obs = ObserverLocation::new(
        Latitude::from_degrees(40.0),
        Longitude::from_degrees(0.0),
        None,
    );
    (engine, obs)
}

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

/// 1 s in days: twice the bisection tolerance, so two independently
/// bracketed refinements of the same root always agree within it.
const ONE_SECOND_DAYS: f64 = 1.0 / 86_400.0;

/// `previous_rise_set` and `next_rise_set` partition the event sequence:
/// events at or before the query instant belong to `previous`, events after
/// it to `next`. Probed 60 s either side of a located sunrise so the check
/// does not hinge on the bisection's own 0.5 s uncertainty at the root.
#[test]
fn previous_rise_set_partitions_events_with_next() {
    let (engine, obs) = mid_latitude_fixture();
    let sun = RiseSetTarget::Body(CelestialBody::Sun);
    let rise = engine
        .next_rise_set(
            sun.clone(),
            RiseSetEvent::Rise,
            obs.clone(),
            Atmosphere::default(),
            RiseSetOptions::default(),
            tdb(2_451_545.0),
        )
        .unwrap()
        .expect("a rise")
        .instant
        .julian_day
        .days();
    let previous = |before: f64| {
        engine
            .previous_rise_set(
                sun.clone(),
                RiseSetEvent::Rise,
                obs.clone(),
                Atmosphere::default(),
                RiseSetOptions::default(),
                tdb(before),
            )
            .unwrap()
            .expect("a previous rise")
            .instant
            .julian_day
            .days()
    };
    let just_after = previous(rise + 60.0 * ONE_SECOND_DAYS);
    assert!(
        (just_after - rise).abs() < ONE_SECOND_DAYS,
        "previous just after the rise should return it: {just_after} vs {rise}"
    );
    let just_before = previous(rise - 60.0 * ONE_SECOND_DAYS);
    assert!(
        just_before < rise - 0.5 && just_before > rise - 1.5,
        "previous just before the rise should return the prior day's: {just_before} vs {rise}"
    );
}

#[test]
fn previous_rise_set_matches_rise_sets_in_range_last() {
    let (engine, obs) = mid_latitude_fixture();
    let sun = RiseSetTarget::Body(CelestialBody::Sun);
    let before = 2_451_545.3;
    for event in [RiseSetEvent::Rise, RiseSetEvent::Set] {
        let expected = engine
            .rise_sets_in_range(
                sun.clone(),
                event,
                obs.clone(),
                Atmosphere::default(),
                RiseSetOptions::default(),
                tdb(before - 3.0),
                tdb(before),
            )
            .unwrap()
            .last()
            .expect("range has an event")
            .instant
            .julian_day
            .days();
        let actual = engine
            .previous_rise_set(
                sun.clone(),
                event,
                obs.clone(),
                Atmosphere::default(),
                RiseSetOptions::default(),
                tdb(before),
            )
            .unwrap()
            .expect("a previous event")
            .instant
            .julian_day
            .days();
        assert!(
            (expected - actual).abs() < ONE_SECOND_DAYS,
            "{event:?}: {expected} vs {actual}"
        );
        assert!(actual <= before);
    }
}

#[test]
fn previous_upper_transit_puts_body_on_the_meridian() {
    let (engine, obs) = mid_latitude_fixture();
    let before = 2_451_545.3;
    let t = engine
        .previous_rise_set(
            RiseSetTarget::Body(CelestialBody::Sun),
            RiseSetEvent::UpperTransit,
            obs.clone(),
            Atmosphere::default(),
            RiseSetOptions::default(),
            tdb(before),
        )
        .unwrap()
        .expect("a previous transit");
    let jd = t.instant.julian_day.days();
    assert!(
        jd <= before && jd > before - 1.1,
        "transit {jd} vs {before}"
    );
    let (ra, _dec) = engine
        .target_equatorial(
            &RiseSetTarget::Body(CelestialBody::Sun),
            &obs,
            &RiseSetOptions::default(),
            jd,
            None,
        )
        .unwrap();
    let ut1 = pleiades_apparent::ut1_instant(tdb(jd)).unwrap();
    let lst =
        pleiades_apparent::sidereal_time(ut1, Longitude::from_degrees(0.0)).local_apparent_deg;
    let ha = crate::root::wrap180(lst - ra);
    assert!(
        ha.abs() < 0.05,
        "hour angle at previous upper transit {ha} deg"
    );
}

#[test]
fn circumpolar_previous_rise_set_returns_none() {
    use pleiades_backend::test_backend::LinearSunMoon;
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let obs = ObserverLocation::new(
        Latitude::from_degrees(89.9),
        Longitude::from_degrees(0.0),
        None,
    );
    let out = engine
        .previous_rise_set(
            RiseSetTarget::Body(CelestialBody::Sun),
            RiseSetEvent::Rise,
            obs,
            Atmosphere::default(),
            RiseSetOptions::default(),
            tdb(2_451_545.0),
        )
        .unwrap();
    assert!(
        out.is_none(),
        "circumpolar-now Sun should have no rise in the span, got {out:?}"
    );
}

#[test]
fn previous_rise_set_fails_closed() {
    let (engine, obs) = mid_latitude_fixture();
    let err = engine
        .previous_rise_set(
            RiseSetTarget::Body(CelestialBody::Sun),
            RiseSetEvent::Rise,
            obs.clone(),
            Atmosphere::default(),
            RiseSetOptions::default(),
            tdb(2_000_000.0),
        )
        .unwrap_err();
    assert!(matches!(err, EventError::OutOfWindow { .. }));

    let bad = Atmosphere {
        pressure_mbar: f64::NAN,
        temperature_c: 15.0,
    };
    let err = engine
        .previous_rise_set(
            RiseSetTarget::Body(CelestialBody::Sun),
            RiseSetEvent::Rise,
            obs.clone(),
            bad,
            RiseSetOptions::default(),
            tdb(2_451_545.0),
        )
        .unwrap_err();
    assert!(matches!(err, EventError::InvalidAtmosphere { .. }));

    let err = engine
        .previous_rise_set(
            RiseSetTarget::FixedStar("Nope".into()),
            RiseSetEvent::Rise,
            obs,
            Atmosphere::default(),
            RiseSetOptions::default(),
            tdb(2_451_545.0),
        )
        .unwrap_err();
    assert!(matches!(err, EventError::UnknownFixedStar { .. }));
}
