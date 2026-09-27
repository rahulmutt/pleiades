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
        )
        .unwrap();
    assert!(
        (dec_forced - dec_zero).abs() < 1e-9,
        "no_ecl_lat should ignore supplied latitude"
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
        )
        .unwrap();
    let h0 = engine
        .standard_altitude(
            &RiseSetTarget::Body(CelestialBody::Sun),
            &obs,
            &RiseSetOptions::default(),
            Atmosphere::default(),
            jd,
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
            .horizon_residual(&target, &obs, &opts, atmos, jd)
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
    // At upper transit the local hour angle H = LST − RA ≈ 0.
    let jd = t.instant.julian_day.days();
    let (ra, _dec) = engine
        .target_equatorial(
            &RiseSetTarget::Body(CelestialBody::Sun),
            &obs,
            &RiseSetOptions::default(),
            jd,
        )
        .unwrap();
    let lst = pleiades_apparent::sidereal_time(
        Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
        Longitude::from_degrees(0.0),
    )
    .local_apparent_deg;
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
