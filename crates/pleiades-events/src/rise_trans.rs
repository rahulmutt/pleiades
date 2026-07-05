//! Rise, set, and meridian-transit finding (`swe_rise_trans`, full-flag).

use crate::crossings::EventEngine;
use crate::ephemeris::{geocentric_apparent_ecliptic, read_mean_ecliptic};
use crate::error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
use crate::fixstar::fixed_star_apparent;
use crate::root::{crossings_in_range, first_crossing_after, wrap180};
use crate::semidiameter::semidiameter_deg;
use pleiades_apparent::{
    apparent_from_true, sidereal_time, topocentric_position, true_obliquity_degrees, Atmosphere,
};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{
    Angle, CelestialBody, EclipticCoordinates, Instant, JulianDay, Latitude, Longitude,
    ObserverLocation, TimeScale,
};

/// Scan step for rise/set bracketing: 2 minutes, small enough to separate
/// fast-Moon grazes without a per-body table (rise/set changes little across
/// the day compared to Task 8/9's longitude-crossing needs).
const RISE_SET_STEP_DAYS: f64 = 2.0 / 1440.0;

/// How far past a located zero-crossing to probe when classifying its
/// direction (ascending = rise, descending = set). Must clear the root
/// refiner's own uncertainty (`root::REFINE_TOLERANCE_DAYS`, 0.5s) so the
/// probe reads an unambiguous sign, while staying far shorter than the ~12h
/// spacing between consecutive rise/set events so it can never probe into the
/// next one.
const DIRECTION_PROBE_DAYS: f64 = 2.0 / 86_400.0;

/// Scan step for meridian-transit bracketing: 5 minutes. Unlike rise/set, the
/// hour-angle residual is monotonic-ascending through its single zero per
/// sidereal day at each transit, so there is no direction to classify — a
/// coarser step than `RISE_SET_STEP_DAYS` is fine.
const TRANSIT_STEP_DAYS: f64 = 5.0 / 1440.0;

/// How far forward of `after` `next_rise_set`'s `Rise`/`Set` arm searches
/// before giving up and returning `None`. This is a deliberate ~2.5×
/// SUPERSET of SE's own `swe_rise_trans` search window, not a match to it:
/// SE's culmination-point search only looks ~28h (~1.167 day) ahead
/// (`swecl.c`'s `jmax=14` steps of 2h) and reports "no event" if nothing
/// crosses in that window, rather than scanning for the true next
/// occurrence arbitrarily far in the future. 3.0 days was chosen instead of
/// ~1.167 because it must comfortably exceed the longest rise-to-rise
/// interval of any supported body (the Moon's, ~24h50m ≈ 1.035 day) so every
/// daily-rising body is always found, while staying short enough that a
/// body that is circumpolar right now (no rise/set for days or weeks)
/// reports `None` instead of a far-future event and a multi-minute linear
/// scan. Consequence of the widened window: a hypothetical body whose next
/// rise/set is 30–72h out would return `Some` here but `None` from SE — not
/// exercised by the corpus below. Tuned against the full 50-row rise-trans
/// corpus (46 real events, 4 `none` rows at lat 66.5N): all 46 events
/// resolve inside this span, and all 4 `none` rows have no crossing within
/// it either.
const RISE_SET_SEARCH_SPAN_DAYS: f64 = 3.0;

/// Which observer-local event to find.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiseSetEvent {
    /// Body crosses the horizon upward.
    Rise,
    /// Body crosses the horizon downward.
    Set,
    /// Upper (meridian) transit — hour angle 0.
    UpperTransit,
    /// Lower transit — hour angle ±12ʰ.
    LowerTransit,
}

/// Which point of the disc defines the event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscMode {
    /// Disc center.
    Center,
    /// Upper limb (SE default for rise/set).
    UpperLimb,
    /// Lower limb (`SE_BIT_DISC_BOTTOM`).
    LowerLimb,
}

/// The object whose event is sought.
#[derive(Clone, Debug)]
pub enum RiseSetTarget {
    /// A release-grade body.
    Body(CelestialBody),
    /// An arbitrary ecliptic point (longitude, latitude); pair with `no_ecl_lat`
    /// to force latitude 0 (rising of a zodiac degree).
    EclipticPoint(Longitude, Latitude),
    /// A curated fixed star by name.
    FixedStar(String),
}

/// `swe_rise_trans` flag bundle.
#[derive(Clone, Debug)]
pub struct RiseSetOptions {
    /// Disc convention.
    pub disc: DiscMode,
    /// Apply atmospheric refraction (`false` = `SE_BIT_NO_REFRACTION`).
    pub refraction: bool,
    /// Force ecliptic latitude 0 (`SE_BIT_GEOCTR_NO_ECL_LAT`).
    pub no_ecl_lat: bool,
    /// Hindu rising = `DISC_CENTER | NO_REFRACTION | GEOCTR_NO_ECL_LAT`.
    pub hindu: bool,
    /// Freeze semidiameter at mean distance (`SE_BIT_FIXED_DISC_SIZE`).
    pub fixed_disc_size: bool,
    /// Custom local horizon altitude, degrees (`swe_rise_trans_true_hor`).
    pub horizon_altitude_deg: Option<f64>,
}

impl Default for RiseSetOptions {
    fn default() -> Self {
        Self {
            disc: DiscMode::UpperLimb,
            refraction: true,
            no_ecl_lat: false,
            hindu: false,
            fixed_disc_size: false,
            horizon_altitude_deg: None,
        }
    }
}

impl RiseSetOptions {
    /// Resolves `hindu` into its component flags (SE composition).
    pub(crate) fn effective(&self) -> Self {
        if self.hindu {
            Self {
                disc: DiscMode::Center,
                refraction: false,
                no_ecl_lat: true,
                ..self.clone()
            }
        } else {
            self.clone()
        }
    }
}

/// Fail-closed guard for [`Atmosphere`] inputs: rejects non-finite pressure or
/// temperature before they can propagate NaN through refraction. Shared by
/// all four public entry points that accept an `Atmosphere`
/// (`next_rise_set`, `rise_sets_in_range`, `horizontal`,
/// `horizontal_to_equatorial`).
pub(crate) fn check_atmosphere(atmos: Atmosphere) -> Result<(), EventError> {
    if !atmos.pressure_mbar.is_finite() || !atmos.temperature_c.is_finite() {
        return Err(EventError::InvalidAtmosphere {
            detail: format!(
                "pressure={} temp={}",
                atmos.pressure_mbar, atmos.temperature_c
            ),
        });
    }
    Ok(())
}

/// A located rise/set/transit event (TDB).
#[derive(Clone, Debug)]
pub struct RiseSet {
    /// Which event this is.
    pub event: RiseSetEvent,
    /// Instant of the event (TDB).
    pub instant: Instant,
    /// The target the event is for.
    pub target: RiseSetTarget,
}

impl<B: EphemerisBackend> EventEngine<B> {
    /// Topocentric right ascension / declination (degrees, apparent-of-date) of
    /// `target` for `observer` at `jd` (TDB Julian Day).
    ///
    /// - `FixedStar`: the curated catalog's apparent equatorial place (already
    ///   geocentric to the precision the catalog supports; no topocentric
    ///   correction is applied since stars have no meaningful parallax here).
    /// - `EclipticPoint`: a pure geocentric ecliptic → equatorial rotation using
    ///   the true obliquity of date; `opts.no_ecl_lat` forces latitude to 0.
    /// - `Body`: the geocentric apparent ecliptic position (from
    ///   `geocentric_apparent_ecliptic`), with `no_ecl_lat` applied, then
    ///   diurnal parallax + diurnal aberration via `topocentric_position`
    ///   before rotating to equatorial.
    pub(crate) fn target_equatorial(
        &self,
        target: &RiseSetTarget,
        observer: &ObserverLocation,
        opts: &RiseSetOptions,
        jd: f64,
    ) -> Result<(f64, f64), EventError> {
        let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
        let eps = true_obliquity_degrees(jd)
            .map_err(|e| EventError::Backend(format!("obliquity failed: {e}")))?;
        let lst = sidereal_time(at, observer.longitude).local_apparent_deg;
        match target {
            RiseSetTarget::FixedStar(name) => {
                let equ = fixed_star_apparent(name, at)?;
                Ok((equ.right_ascension.degrees(), equ.declination.degrees()))
            }
            RiseSetTarget::EclipticPoint(lon, lat) => {
                let lat = if opts.no_ecl_lat {
                    Latitude::from_degrees(0.0)
                } else {
                    *lat
                };
                let equ = EclipticCoordinates::new(*lon, lat, None)
                    .to_equatorial(Angle::from_degrees(eps));
                Ok((equ.right_ascension.degrees(), equ.declination.degrees()))
            }
            RiseSetTarget::Body(b) => {
                let (lon, lat, dist) =
                    geocentric_apparent_ecliptic(&self.backend, b.clone(), "body", jd)?;
                let lat = if opts.no_ecl_lat { 0.0 } else { lat };
                let ecl = EclipticCoordinates::new(
                    Longitude::from_degrees(lon),
                    Latitude::from_degrees(lat),
                    Some(dist),
                );
                let topo = topocentric_position(ecl, observer, lst, eps)
                    .map_err(|e| EventError::Backend(format!("topocentric failed: {e}")))?;
                let equ = topo.ecliptic.to_equatorial(Angle::from_degrees(eps));
                Ok((equ.right_ascension.degrees(), equ.declination.degrees()))
            }
        }
    }

    /// Apparent (refracted, when `opts.refraction`) topocentric altitude of the
    /// target at `jd` (TDB), in degrees. This is the function rise/set root-finds.
    pub(crate) fn target_apparent_altitude(
        &self,
        target: &RiseSetTarget,
        observer: &ObserverLocation,
        opts: &RiseSetOptions,
        atmos: Atmosphere,
        jd: f64,
    ) -> Result<f64, EventError> {
        let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
        let (ra_deg, dec_deg) = self.target_equatorial(target, observer, opts, jd)?;
        let phi = observer.latitude.degrees().to_radians();
        let lst = sidereal_time(at, observer.longitude).local_apparent_deg;
        let ha = (lst - ra_deg).to_radians();
        let dec = dec_deg.to_radians();
        let sin_alt = phi.sin() * dec.sin() + phi.cos() * dec.cos() * ha.cos();
        // Guard the asin domain: fail-closed, never-NaN.
        let true_alt = sin_alt.clamp(-1.0, 1.0).asin().to_degrees();
        Ok(if opts.refraction {
            apparent_from_true(true_alt, atmos)
        } else {
            true_alt
        })
    }

    /// The standard altitude `h0` the event is defined at: horizon geometry minus
    /// the disc term, plus any custom horizon. Neither refraction nor an
    /// elevation-based horizon dip are included here, matching SE's
    /// `swe_rise_trans` (Model B): refraction lives entirely in the apparent
    /// altitude returned by `target_apparent_altitude`, which the root-finder
    /// compares against this `h0`; a height-based dip is omitted because SE's
    /// default `swe_rise_trans` calls `swe_rise_trans_true_hor` with
    /// `horhgt = 0` (dip is only computed when `horhgt == -100`, a sentinel
    /// SE's caller never requests by default) — so applying a dip here would
    /// diverge from SE, not match it.
    pub(crate) fn standard_altitude(
        &self,
        target: &RiseSetTarget,
        _observer: &ObserverLocation,
        opts: &RiseSetOptions,
        _atmos: Atmosphere,
        jd: f64,
    ) -> Result<f64, EventError> {
        // Distance (AU) for semidiameter; 0 for points/stars.
        let distance_au = match target {
            RiseSetTarget::Body(b) => read_mean_ecliptic(&self.backend, b.clone(), "body", jd)?.2,
            _ => 0.0,
        };
        let mut h0 = 0.0_f64;
        // Disc term.
        let sd = semidiameter_deg(target, distance_au.max(1e-9), opts.fixed_disc_size);
        h0 += match opts.disc {
            DiscMode::UpperLimb => -sd,
            DiscMode::LowerLimb => sd,
            DiscMode::Center => 0.0,
        };
        // Custom local horizon altitude.
        if let Some(hor) = opts.horizon_altitude_deg {
            h0 += hor;
        }
        Ok(h0)
    }

    /// The rise/set residual: apparent altitude minus standard altitude. Its
    /// zeros (ascending = rise, descending = set) are what `next_rise_set` and
    /// `rise_sets_in_range` root-find.
    fn horizon_residual(
        &self,
        target: &RiseSetTarget,
        observer: &ObserverLocation,
        opts: &RiseSetOptions,
        atmos: Atmosphere,
        jd: f64,
    ) -> Result<f64, EventError> {
        let alt = self.target_apparent_altitude(target, observer, opts, atmos, jd)?;
        let h0 = self.standard_altitude(target, observer, opts, atmos, jd)?;
        Ok(alt - h0)
    }

    /// Whether the residual is heading upward (ascending, i.e. a rise rather
    /// than a set) at `root_jd`, a zero-crossing already located by
    /// `first_crossing_after`/`crossings_in_range`. Both helpers detect ANY
    /// sign change (ascending or descending) — they do not discriminate
    /// direction — so callers must classify each root themselves. We sample
    /// just past the root, outside the bisection's `REFINE_TOLERANCE_DAYS`
    /// (0.5s) uncertainty, and read the residual's sign there: positive means
    /// the residual has risen above zero (ascending / rise), negative means it
    /// has fallen below (descending / set). `DIRECTION_PROBE_DAYS` (2s) is
    /// tiny compared to the ~12h spacing between consecutive rise/set events,
    /// so it can never probe into the next event.
    fn is_ascending_crossing(
        &self,
        target: &RiseSetTarget,
        observer: &ObserverLocation,
        opts: &RiseSetOptions,
        atmos: Atmosphere,
        root_jd: f64,
    ) -> Result<bool, EventError> {
        let probe = self.horizon_residual(
            target,
            observer,
            opts,
            atmos,
            root_jd + DIRECTION_PROBE_DAYS,
        )?;
        Ok(probe > 0.0)
    }

    /// Next rise/set/transit strictly after `after`, or `None` if it does not
    /// occur before the ephemeris window's end. For `Rise`/`Set`, the search
    /// is additionally bounded to `RISE_SET_SEARCH_SPAN_DAYS` past `after` —
    /// a short-horizon search in the same spirit as SE's `swe_rise_trans`
    /// (which reports "no event" past its own, narrower ~28h window) but not
    /// numerically matching it: `RISE_SET_SEARCH_SPAN_DAYS` is a wider,
    /// Moon-covering superset (see its doc comment for why). A body that is
    /// circumpolar right now and does not rise/set again within that span
    /// returns `None`, even though it may rise far in the future (use
    /// `rise_sets_in_range` with an explicit, longer window for that
    /// question). Meridian transits are unaffected — they always occur
    /// within a sidereal day, well inside the bound.
    pub fn next_rise_set(
        &self,
        target: RiseSetTarget,
        event: RiseSetEvent,
        observer: ObserverLocation,
        atmos: Atmosphere,
        opts: RiseSetOptions,
        after: Instant,
    ) -> Result<Option<RiseSet>, EventError> {
        observer
            .validate()
            .map_err(|e| EventError::InvalidObserver {
                detail: e.to_string(),
            })?;
        check_atmosphere(atmos)?;
        let opts = opts.effective();
        let after_jd = after.julian_day.days();
        self.check_window(after_jd)?;
        match event {
            RiseSetEvent::Rise | RiseSetEvent::Set => {
                let scan_end =
                    (after_jd + RISE_SET_SEARCH_SPAN_DAYS).min(WINDOW_END_JD - RISE_SET_STEP_DAYS);
                let want_ascending = matches!(event, RiseSetEvent::Rise);
                // `first_crossing_after` finds the next zero-crossing of the
                // residual regardless of direction (ascending = rise,
                // descending = set); skip crossings of the wrong direction by
                // resuming the scan just past each rejected root.
                let mut scan_start = after_jd.max(WINDOW_START_JD + RISE_SET_STEP_DAYS);
                let root = loop {
                    let candidate = first_crossing_after(
                        |jd| self.horizon_residual(&target, &observer, &opts, atmos, jd),
                        scan_start,
                        scan_end,
                        RISE_SET_STEP_DAYS,
                    )?;
                    match candidate {
                        None => break None,
                        Some(jd) => {
                            if self.is_ascending_crossing(&target, &observer, &opts, atmos, jd)?
                                == want_ascending
                            {
                                break Some(jd);
                            }
                            // Resume exactly at the rejected root. `first_crossing_after`'s
                            // first step may re-refine this same root (bounded, cheap) before
                            // its scan advances past it to the next crossing.
                            scan_start = jd;
                        }
                    }
                };
                Ok(root.filter(|&jd| jd > after_jd).map(|jd| RiseSet {
                    event,
                    target: target.clone(),
                    instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
                }))
            }
            RiseSetEvent::UpperTransit | RiseSetEvent::LowerTransit => {
                self.next_transit(target, event, observer, opts, after)
            }
        }
    }

    /// All rise/set/transit events of `event` kind in `[start, end]`, ascending.
    #[allow(clippy::too_many_arguments)]
    pub fn rise_sets_in_range(
        &self,
        target: RiseSetTarget,
        event: RiseSetEvent,
        observer: ObserverLocation,
        atmos: Atmosphere,
        opts: RiseSetOptions,
        start: Instant,
        end: Instant,
    ) -> Result<Vec<RiseSet>, EventError> {
        observer
            .validate()
            .map_err(|e| EventError::InvalidObserver {
                detail: e.to_string(),
            })?;
        check_atmosphere(atmos)?;
        let opts = opts.effective();
        let start_jd = start.julian_day.days();
        let end_jd = end.julian_day.days();
        self.check_window(start_jd)?;
        self.check_window(end_jd)?;
        match event {
            RiseSetEvent::Rise | RiseSetEvent::Set => {
                let want_ascending = matches!(event, RiseSetEvent::Rise);
                let scan_start = start_jd.max(WINDOW_START_JD + RISE_SET_STEP_DAYS);
                let scan_end = end_jd.min(WINDOW_END_JD - RISE_SET_STEP_DAYS);
                // `crossings_in_range` returns every zero-crossing (both rise
                // and set, alternating); classify each by direction and keep
                // only the ones matching `event` (see `is_ascending_crossing`).
                let roots = crossings_in_range(
                    |jd| self.horizon_residual(&target, &observer, &opts, atmos, jd),
                    scan_start,
                    scan_end,
                    RISE_SET_STEP_DAYS,
                )?;
                let mut out = Vec::with_capacity(roots.len());
                for jd in roots {
                    if self.is_ascending_crossing(&target, &observer, &opts, atmos, jd)?
                        == want_ascending
                    {
                        out.push(RiseSet {
                            event,
                            target: target.clone(),
                            instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
                        });
                    }
                }
                Ok(out)
            }
            RiseSetEvent::UpperTransit | RiseSetEvent::LowerTransit => {
                self.transits_in_range(target, event, observer, opts, start, end)
            }
        }
    }

    /// The meridian-transit residual: local hour angle `H = LST − RA`, wrapped
    /// to `(−180, 180]`. Upper transit is the zero of `H`; lower transit is the
    /// zero of `H − 180` (also wrapped). Unlike the rise/set horizon residual,
    /// this residual is monotonic-ascending through its single zero per
    /// sidereal day (LST advances ~361°/day against a slowly-moving target RA),
    /// so `first_crossing_after`/`crossings_in_range` locate upper and lower
    /// transits unambiguously — no post-hoc direction classification needed.
    fn hour_angle_residual(
        &self,
        target: &RiseSetTarget,
        observer: &ObserverLocation,
        opts: &RiseSetOptions,
        lower: bool,
        jd: f64,
    ) -> Result<f64, EventError> {
        let (ra, _dec) = self.target_equatorial(target, observer, opts, jd)?;
        let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
        let lst = sidereal_time(at, observer.longitude).local_apparent_deg;
        let ha = lst - ra;
        Ok(if lower {
            wrap180(ha - 180.0)
        } else {
            wrap180(ha)
        })
    }

    /// Next meridian transit strictly after `after`.
    pub(crate) fn next_transit(
        &self,
        target: RiseSetTarget,
        event: RiseSetEvent,
        observer: ObserverLocation,
        opts: RiseSetOptions,
        after: Instant,
    ) -> Result<Option<RiseSet>, EventError> {
        let lower = matches!(event, RiseSetEvent::LowerTransit);
        let after_jd = after.julian_day.days();
        let scan_start = after_jd.max(WINDOW_START_JD + TRANSIT_STEP_DAYS);
        let scan_end = WINDOW_END_JD - TRANSIT_STEP_DAYS;
        let root = first_crossing_after(
            |jd| self.hour_angle_residual(&target, &observer, &opts, lower, jd),
            scan_start,
            scan_end,
            TRANSIT_STEP_DAYS,
        )?;
        Ok(root.filter(|&jd| jd > after_jd).map(|jd| RiseSet {
            event,
            target: target.clone(),
            instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
        }))
    }

    /// All meridian transits in `[start, end]`.
    pub(crate) fn transits_in_range(
        &self,
        target: RiseSetTarget,
        event: RiseSetEvent,
        observer: ObserverLocation,
        opts: RiseSetOptions,
        start: Instant,
        end: Instant,
    ) -> Result<Vec<RiseSet>, EventError> {
        let lower = matches!(event, RiseSetEvent::LowerTransit);
        let scan_start = start
            .julian_day
            .days()
            .max(WINDOW_START_JD + TRANSIT_STEP_DAYS);
        let scan_end = end.julian_day.days().min(WINDOW_END_JD - TRANSIT_STEP_DAYS);
        let roots = crossings_in_range(
            |jd| self.hour_angle_residual(&target, &observer, &opts, lower, jd),
            scan_start,
            scan_end,
            TRANSIT_STEP_DAYS,
        )?;
        Ok(roots
            .into_iter()
            .map(|jd| RiseSet {
                event,
                target: target.clone(),
                instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
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
        use pleiades_types::{
            Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
        };
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
        use pleiades_types::{
            Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
        };
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
    /// `is_ascending_crossing` were reversed (rise/set labels swapped), since it
    /// only checks `alt ≈ h0` and `rise != set`. Here we sample the residual
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

        // 120s: well outside the bisection's REFINE_TOLERANCE_DAYS (0.5s) and
        // DIRECTION_PROBE_DAYS (2s) noise floors, but tiny compared to the
        // ~12h spacing between consecutive rise/set events, so it stays
        // within the same monotonic segment of the residual.
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
        use pleiades_types::{
            Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
        };
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
        use pleiades_types::{
            Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
        };
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
}
