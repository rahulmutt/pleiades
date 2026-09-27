//! Regression tests for issue #74: rise/set/transit and horizontal
//! coordinates must honour the `TimeScale` tag on their instants, rotate the
//! sky with UT1, sample bodies in TDB, and return instants that are genuinely
//! TDB under their `Tdb` tag.
//!
//! Fixture: the issue's own evidence — the Sun at Greenwich (51.4769 N, 0 E),
//! civil 2020-01-01 00:00 UTC (JD 2458849.5 as a UT day; ΔT ≈ 71.6 s), for
//! which Swiss Ephemeris `swe_rise_trans` reports sunrise at 08:05:09.6 UT.

use super::*;
use crate::HorizontalInput;
use pleiades_apparent::{ut1_instant, Atmosphere};
use pleiades_data::packaged_backend;
use pleiades_types::{CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation};

/// 2020-01-01 00:00 as a Julian Day number (read as UT).
const JD_2020_01_01: f64 = 2_458_849.5;
/// Swiss Ephemeris sunrise for the fixture, 08:05:09.6 UT, as a UT Julian Day.
const SE_SUNRISE_JD_UT: f64 = JD_2020_01_01 + (8.0 * 3600.0 + 5.0 * 60.0 + 9.6) / 86_400.0;
const SECONDS_PER_DAY: f64 = 86_400.0;

fn greenwich() -> ObserverLocation {
    ObserverLocation::new(
        Latitude::from_degrees(51.4769),
        Longitude::from_degrees(0.0),
        None,
    )
}

fn at(jd: f64, scale: TimeScale) -> Instant {
    Instant::new(JulianDay::from_days(jd), scale)
}

fn sunrise_after(engine: &EventEngine<impl EphemerisBackend>, after: Instant) -> Instant {
    engine
        .next_rise_set(
            RiseSetTarget::Body(CelestialBody::Sun),
            RiseSetEvent::Rise,
            greenwich(),
            Atmosphere::default(),
            RiseSetOptions::default(),
            after,
        )
        .expect("engine ok")
        .expect("the Sun rises daily at Greenwich")
        .instant
}

fn sunrise_before(engine: &EventEngine<impl EphemerisBackend>, before: Instant) -> Instant {
    engine
        .previous_rise_set(
            RiseSetTarget::Body(CelestialBody::Sun),
            RiseSetEvent::Rise,
            greenwich(),
            Atmosphere::default(),
            RiseSetOptions::default(),
            before,
        )
        .expect("engine ok")
        .expect("the Sun rises daily at Greenwich")
        .instant
}

/// The returned instant is TDB: converting it to UT1 by its own tag must land
/// on Swiss Ephemeris's UT sunrise, not ΔT early. (Before the fix the
/// returned day was UT1-scale under a `Tdb` tag, so this read 08:03:54.7.)
#[test]
fn returned_sunrise_read_by_its_tag_matches_swiss_ephemeris_ut() {
    let engine = EventEngine::new(packaged_backend());
    // A TT-tagged `after` on the civil midnight, as `tt_from_utc_civil` would
    // produce it (the numeric day is ΔT later than the UT day).
    let delta_t_days = delta_t_days_at(JD_2020_01_01);
    let after_tt = at(JD_2020_01_01 + delta_t_days, TimeScale::Tt);

    let rise = sunrise_after(&engine, after_tt);
    assert_eq!(rise.scale, TimeScale::Tdb);

    let rise_ut = ut1_instant(rise).expect("ΔT table readable");
    let residual_s = (rise_ut.julian_day.days() - SE_SUNRISE_JD_UT) * SECONDS_PER_DAY;
    assert!(
        residual_s.abs() < 10.0,
        "sunrise read as UT is {residual_s:.1} s from Swiss Ephemeris (ΔT-class error if ~70 s)"
    );
}

/// The `TimeScale` tag on `before` is read: the same numeric day tagged `Tt`
/// and `Ut1` denote instants ΔT apart, and a sunrise falling inside that gap
/// belongs to `previous_rise_set` for one tag but not the other.
#[test]
fn previous_rise_set_honours_the_tag_on_before() {
    let engine = EventEngine::new(packaged_backend());
    let rise_tdb = sunrise_after(&engine, at(JD_2020_01_01, TimeScale::Tdb))
        .julian_day
        .days();
    // A numeric day 30 s before the sunrise's TDB day. Read as TT/TDB it
    // precedes the sunrise; read as UT1 (+ΔT ≈ 71.6 s) it follows it.
    let probe = rise_tdb - 30.0 / SECONDS_PER_DAY;

    let previous_tt = sunrise_before(&engine, at(probe, TimeScale::Tt))
        .julian_day
        .days();
    let previous_ut1 = sunrise_before(&engine, at(probe, TimeScale::Ut1))
        .julian_day
        .days();
    let previous_utc = sunrise_before(&engine, at(probe, TimeScale::Utc))
        .julian_day
        .days();

    assert!(
        rise_tdb - previous_tt > 0.9,
        "TT-tagged probe precedes the sunrise, so the previous one is yesterday's: got {previous_tt} vs sunrise {rise_tdb}"
    );
    assert!(
        (previous_ut1 - rise_tdb).abs() * SECONDS_PER_DAY < 1.0,
        "UT1-tagged probe follows the sunrise, so it is the previous one: got {previous_ut1} vs sunrise {rise_tdb}"
    );
    assert!(
        (previous_utc - previous_ut1).abs() * SECONDS_PER_DAY < 1.0,
        "UTC is treated as UT1 (no DUT1 table): got {previous_utc} vs {previous_ut1}"
    );
}

/// `horizontal_to_equatorial` rotates the sky with UT1: for the same numeric
/// day, a `Tt` tag and a `Ut1` tag differ by ΔT of Earth rotation in the
/// right ascension placed on the meridian.
#[test]
fn horizontal_to_equatorial_honours_the_tag_on_at() {
    let engine = EventEngine::new(packaged_backend());
    let ra_on_meridian = |scale: TimeScale| {
        engine
            .horizontal_to_equatorial(
                0.0,
                40.0,
                false,
                greenwich(),
                Atmosphere::default(),
                at(JD_2020_01_01, scale),
            )
            .expect("engine ok")
            .0
            .degrees()
    };
    let ra_tt = ra_on_meridian(TimeScale::Tt);
    let ra_ut1 = ra_on_meridian(TimeScale::Ut1);
    // The same number read as TT is the UT1 instant N − ΔT, i.e. ΔT earlier
    // than when read as UT1, so its sky has rotated ΔT × (sidereal rate) less
    // far: ≈ −0.299° of right ascension in 2020.
    let expected_deg = -delta_t_days_at(JD_2020_01_01) * 360.985_647;
    let got_deg = crate::root::wrap180(ra_tt - ra_ut1);
    assert!(
        (got_deg - expected_deg).abs() < 0.01,
        "RA shift between Tt and Ut1 tags: got {got_deg:.4}°, expected {expected_deg:.4}°"
    );
}

/// `horizontal` and `horizontal_to_equatorial` agree on the tag: the round
/// trip through both must be the identity for a non-`Tdb` tag too.
#[test]
fn horizontal_round_trip_is_identity_for_a_tt_tag() {
    let engine = EventEngine::new(packaged_backend());
    let when = at(JD_2020_01_01, TimeScale::Tt);
    let ra_in = 123.0;
    let dec_in = 17.0;
    let h = engine
        .horizontal(
            HorizontalInput::Equatorial(Angle::from_degrees(ra_in), Latitude::from_degrees(dec_in)),
            greenwich(),
            Atmosphere::default(),
            when,
        )
        .expect("engine ok");
    let (ra, dec) = engine
        .horizontal_to_equatorial(
            h.azimuth,
            h.true_altitude,
            false,
            greenwich(),
            Atmosphere::default(),
            when,
        )
        .expect("engine ok");
    assert!(crate::root::wrap180(ra.degrees() - ra_in).abs() < 1e-6);
    assert!((dec.degrees() - dec_in).abs() < 1e-6);
}

fn delta_t_days_at(jd: f64) -> f64 {
    pleiades_time::deltat::delta_t(jd)
        .expect("ΔT table readable")
        .0
        / SECONDS_PER_DAY
}
