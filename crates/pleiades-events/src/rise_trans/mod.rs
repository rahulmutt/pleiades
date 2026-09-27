//! Rise, set, and meridian-transit finding (`swe_rise_trans`, full-flag).

use crate::crossings::EventEngine;
use crate::ephemeris::{geocentric_apparent_ecliptic, read_mean_ecliptic};
use crate::error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
use crate::fixstar::fixed_star_apparent;
use crate::root::{crossings_in_range, first_crossing_after, last_crossing_before, wrap180};
use crate::semidiameter::semidiameter_deg;
use pleiades_apparent::{
    apparent_from_true, sidereal_time, topocentric_position, true_obliquity_degrees, Atmosphere,
};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{
    Angle, CelestialBody, EclipticCoordinates, Instant, JulianDay, Latitude, Longitude,
    ObserverLocation, TimeScale,
};
use scan::{
    first_horizon_crossing_after, horizon_crossings_in_range, last_horizon_crossing_before,
};

/// Grid step for rise/set bracketing: 1 hour. The horizon scanner
/// (`scan.rs`) does not need the step to separate a graze's two crossings —
/// it catches grazes by refining any culmination that comes near the horizon
/// between samples — so the step only has to keep the three-sample parabola
/// estimate of each culmination honest, which hourly sampling of a
/// once-per-day sinusoid does comfortably (see `scan::GRAZE_MARGIN_DEG`).
/// Search cost is therefore ~1 residual evaluation per hour scanned plus one
/// bisection per candidate event, instead of ~30 per hour at the former
/// 2-minute step (issue #70).
const RISE_SET_STEP_DAYS: f64 = 1.0 / 24.0;

/// Scan step for meridian-transit bracketing: 1 hour. The hour-angle residual
/// is monotonic-ascending at ~15°/h through its single zero per sidereal day,
/// so any step well under the 12 h wrap-seam guard in `root` brackets each
/// transit exactly once; 1 hour matches the rise/set grid and keeps the
/// transit search ~12× cheaper per hour scanned than the former 5-minute step.
const TRANSIT_STEP_DAYS: f64 = 1.0 / 24.0;

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
/// all five public entry points that accept an `Atmosphere`
/// (`next_rise_set`, `previous_rise_set`, `rise_sets_in_range`, `horizontal`,
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
    /// `rise_sets_in_range` root-find through the horizon scanner in `scan`,
    /// which reads each crossing's direction from the signs across its bracket.
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
                let scan_start = after_jd.max(WINDOW_START_JD + RISE_SET_STEP_DAYS);
                let root = first_horizon_crossing_after(
                    |jd| self.horizon_residual(&target, &observer, &opts, atmos, jd),
                    scan_start,
                    scan_end,
                    RISE_SET_STEP_DAYS,
                    want_ascending,
                )?;
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

    /// Last rise/set/transit at or before `before`, or `None`. The backward
    /// twin of [`next_rise_set`](Self::next_rise_set): the two partition the
    /// event sequence, with events at or before the query instant belonging
    /// here and events after it to `next_rise_set`. (An event within the
    /// 0.5 s bisection tolerance of `before` may land on either side.)
    ///
    /// This is the question observer-local calendars ask most — the Hindu
    /// civil day, its vara and hora, and every muhurta and panchanga reading
    /// are anchored to the sunrise that began the day containing the query
    /// instant, i.e. the last sunrise at or before it.
    ///
    /// Same options, time base, and bounded-window semantics as
    /// `next_rise_set`: `Rise`/`Set` search only `RISE_SET_SEARCH_SPAN_DAYS`
    /// back from `before`, so a body that has been circumpolar for longer
    /// than that returns `None`; meridian transits always occur within a
    /// sidereal day and are unaffected. Early-terminating: the search walks
    /// backward from `before` and stops at the first event found, so its
    /// cost does not depend on how far back the event is within the span.
    /// The result agrees with `rise_sets_in_range(before − span, before)
    /// .last()` to within the bisection tolerance.
    pub fn previous_rise_set(
        &self,
        target: RiseSetTarget,
        event: RiseSetEvent,
        observer: ObserverLocation,
        atmos: Atmosphere,
        opts: RiseSetOptions,
        before: Instant,
    ) -> Result<Option<RiseSet>, EventError> {
        observer
            .validate()
            .map_err(|e| EventError::InvalidObserver {
                detail: e.to_string(),
            })?;
        check_atmosphere(atmos)?;
        let opts = opts.effective();
        let before_jd = before.julian_day.days();
        self.check_window(before_jd)?;
        match event {
            RiseSetEvent::Rise | RiseSetEvent::Set => {
                let scan_start = (before_jd - RISE_SET_SEARCH_SPAN_DAYS)
                    .max(WINDOW_START_JD + RISE_SET_STEP_DAYS);
                let scan_end = before_jd.min(WINDOW_END_JD - RISE_SET_STEP_DAYS);
                let want_ascending = matches!(event, RiseSetEvent::Rise);
                let root = last_horizon_crossing_before(
                    |jd| self.horizon_residual(&target, &observer, &opts, atmos, jd),
                    scan_start,
                    scan_end,
                    RISE_SET_STEP_DAYS,
                    want_ascending,
                )?;
                Ok(root.filter(|&jd| jd <= before_jd).map(|jd| RiseSet {
                    event,
                    target: target.clone(),
                    instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
                }))
            }
            RiseSetEvent::UpperTransit | RiseSetEvent::LowerTransit => {
                self.previous_transit(target, event, observer, opts, before)
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
                let roots = horizon_crossings_in_range(
                    |jd| self.horizon_residual(&target, &observer, &opts, atmos, jd),
                    scan_start,
                    scan_end,
                    RISE_SET_STEP_DAYS,
                    want_ascending,
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

    /// Last meridian transit at or before `before`. Early-terminating: walks
    /// the same window-anchored grid as `transits_in_range` backward from
    /// `before` (as `previous_longitude_crossing` does for crossings) and
    /// stops at the first transit found, always within a sidereal day.
    pub(crate) fn previous_transit(
        &self,
        target: RiseSetTarget,
        event: RiseSetEvent,
        observer: ObserverLocation,
        opts: RiseSetOptions,
        before: Instant,
    ) -> Result<Option<RiseSet>, EventError> {
        let lower = matches!(event, RiseSetEvent::LowerTransit);
        let before_jd = before.julian_day.days();
        let scan_start = WINDOW_START_JD + TRANSIT_STEP_DAYS;
        let scan_end = before_jd.min(WINDOW_END_JD - TRANSIT_STEP_DAYS);
        let root = last_crossing_before(
            |jd| self.hour_angle_residual(&target, &observer, &opts, lower, jd),
            scan_start,
            scan_end,
            TRANSIT_STEP_DAYS,
        )?;
        Ok(root.filter(|&jd| jd <= before_jd).map(|jd| RiseSet {
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
mod tests;

mod scan;
