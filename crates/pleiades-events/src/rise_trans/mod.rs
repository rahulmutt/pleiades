//! Rise, set, and meridian-transit finding (`swe_rise_trans`, full-flag).

use crate::crossings::EventEngine;
use crate::ephemeris::{geocentric_apparent_ecliptic, read_mean_ecliptic};
use crate::error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
use crate::fixstar::fixed_star_apparent;
use crate::root::wrap180;
use crate::semidiameter::semidiameter_deg;
use crate::time_scale::{local_apparent_sidereal_deg, tdb_jd};
use pleiades_apparent::{
    apparent_from_true, topocentric_position, true_obliquity_degrees, Atmosphere,
};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{
    Angle, CelestialBody, EclipticCoordinates, Instant, JulianDay, Latitude, Longitude,
    ObserverLocation, TimeScale,
};
use scan::{
    directed_crossings_in_range, first_directed_crossing_after, last_directed_crossing_before,
    Limits,
};
use track::BodyTrack;
pub(crate) use track::PlaceCache;

/// The instants the scanner may sample: the ephemeris window.
const WINDOW: Limits = Limits {
    earliest: WINDOW_START_JD,
    latest: WINDOW_END_JD,
};

/// Grid step for rise/set bracketing: 1 hour. The horizon scanner
/// (`scan.rs`) does not need the step to separate a graze's two crossings —
/// it catches grazes by refining any culmination that comes near the horizon
/// between samples — so the step only has to keep the three-sample parabola
/// estimate of each culmination honest, which hourly sampling of a
/// once-per-day sinusoid does comfortably (see `scan::GRAZE_MARGIN_DEG`).
/// Search cost is therefore ~1 residual evaluation per hour scanned plus one
/// ITP refinement per candidate event, instead of ~30 per hour at the former
/// 2-minute step (issue #70).
const RISE_SET_STEP_DAYS: f64 = 1.0 / 24.0;

/// Scan step for meridian-transit bracketing: 1 hour. The hour-angle residual
/// climbs at ~15°/h through its single zero per sidereal day and drops 360°
/// at the wrap seam half a day later, so any step well under 12 h puts each
/// transit in exactly one ascending bracket; 1 hour matches the rise/set grid
/// and keeps the transit search ~12× cheaper per hour scanned than the former
/// 5-minute step.
const TRANSIT_STEP_DAYS: f64 = 1.0 / 24.0;

/// How far a meridian-transit search looks from its query instant. A transit
/// recurs once per sidereal day for a star and at most every 25.3 hours for
/// the Moon, so 1.5 days always holds the next (or previous) one. The span is
/// what lets a search the ephemeris window cuts short report `OutOfWindow`:
/// a search that ended at the window's last instant would have nothing left
/// to be cut short of (issue #203).
const TRANSIT_SEARCH_SPAN_DAYS: f64 = 1.5;

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
    /// Use the geocentric place with ecliptic latitude forced to 0
    /// (`SE_BIT_GEOCTR_NO_ECL_LAT`). For a [`RiseSetTarget::Body`] this also
    /// drops diurnal parallax and diurnal aberration: the body is placed as
    /// seen from the Earth's centre, not from the observer.
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

/// A located rise/set/transit event.
///
/// `instant` is a genuine TDB instant, tagged `TimeScale::Tdb`, whatever the
/// scale of the query instant that found it: convert with
/// `pleiades_apparent::ut1_instant` (or `pleiades-time`) to read it as a UT
/// or civil time. See [`EventEngine::next_rise_set`] for how query instants
/// are interpreted.
///
/// The instant trails the event by less than the 0.5 s refinement tolerance
/// and never precedes it, so it can be handed back to a follow-on search; see
/// "Chaining searches" on [`EventEngine::next_rise_set`].
#[derive(Clone, Debug)]
pub struct RiseSet {
    /// Which event this is.
    pub event: RiseSetEvent,
    /// Instant of the event, TDB.
    pub instant: Instant,
    /// The target the event is for.
    pub target: RiseSetTarget,
}

impl<B: EphemerisBackend> EventEngine<B> {
    /// Topocentric right ascension / declination (degrees, apparent-of-date) of
    /// `target` for `observer` at `jd` (TDB Julian Day). Body positions are
    /// sampled at `jd`; the local sidereal time that places them against the
    /// observer's sky is evaluated at its UT1 re-expression (`jd − ΔT`).
    ///
    /// - `FixedStar`: the curated catalog's apparent equatorial place (already
    ///   geocentric to the precision the catalog supports; no topocentric
    ///   correction is applied since stars have no meaningful parallax here).
    /// - `EclipticPoint`: a pure geocentric ecliptic → equatorial rotation using
    ///   the true obliquity of date; `opts.no_ecl_lat` forces latitude to 0.
    /// - `Body`: the geocentric apparent ecliptic position (from
    ///   `geocentric_apparent_ecliptic`), corrected for diurnal parallax +
    ///   diurnal aberration via `topocentric_position` before rotating to
    ///   equatorial. With `opts.no_ecl_lat` the place stays geocentric and
    ///   its latitude is forced to 0, as `SE_BIT_GEOCTR_NO_ECL_LAT` does. A search passes the body's `track`,
    ///   which supplies that position from a few lattice samples (see
    ///   [`BodyTrack`]); `None` reads it from the backend at `jd`.
    pub(crate) fn target_equatorial(
        &self,
        target: &RiseSetTarget,
        observer: &ObserverLocation,
        opts: &RiseSetOptions,
        jd: f64,
        track: Option<&BodyTrack<'_, B>>,
    ) -> Result<(f64, f64), EventError> {
        let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
        let eps = true_obliquity_degrees(jd)
            .map_err(|e| EventError::Backend(format!("obliquity failed: {e}")))?;
        let lst = local_apparent_sidereal_deg(jd, observer.longitude)?;
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
                let (lon, lat, dist) = match track {
                    Some(track) => track.place(jd)?,
                    None => geocentric_apparent_ecliptic(&self.backend, b.clone(), "body", jd)?,
                };
                if opts.no_ecl_lat {
                    let ecl = EclipticCoordinates::new(
                        Longitude::from_degrees(lon),
                        Latitude::from_degrees(0.0),
                        None,
                    );
                    let equ = ecl.to_equatorial(Angle::from_degrees(eps));
                    return Ok((equ.right_ascension.degrees(), equ.declination.degrees()));
                }
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
        track: Option<&BodyTrack<'_, B>>,
    ) -> Result<f64, EventError> {
        let (ra_deg, dec_deg) = self.target_equatorial(target, observer, opts, jd, track)?;
        let phi = observer.latitude.degrees().to_radians();
        let lst = local_apparent_sidereal_deg(jd, observer.longitude)?;
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
        track: Option<&BodyTrack<'_, B>>,
    ) -> Result<f64, EventError> {
        // Distance (AU) for semidiameter; 0 for points/stars. A tracked body's
        // distance comes with its place, which saves the second read every
        // sample used to make for it (issue #204).
        let distance_au = match (target, track) {
            (RiseSetTarget::Body(_), Some(track)) => track.place(jd)?.2,
            (RiseSetTarget::Body(b), None) => {
                read_mean_ecliptic(&self.backend, b.clone(), "body", jd)?.2
            }
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

    /// The place source one search uses for its target: a [`BodyTrack`] for
    /// a body that has one, `None` for a target read at every instant.
    fn track(&self, target: &RiseSetTarget) -> Option<BodyTrack<'_, B>> {
        match target {
            RiseSetTarget::Body(body) => BodyTrack::new(&self.backend, &self.places, body),
            _ => None,
        }
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
        track: Option<&BodyTrack<'_, B>>,
    ) -> Result<f64, EventError> {
        let alt = self.target_apparent_altitude(target, observer, opts, atmos, jd, track)?;
        let h0 = self.standard_altitude(target, observer, opts, atmos, jd, track)?;
        Ok(alt - h0)
    }

    /// Next rise/set/transit strictly after `after`, or `None` if the search
    /// span holds none.
    ///
    /// # Time scales
    ///
    /// `after` is read by its `TimeScale` tag: `Tdb`/`Tt` days are taken as
    /// TDB (they differ by under 2 ms), `Ut1`/`Utc` days have ΔT added (UTC is
    /// treated as UT1; no DUT1 table). Any other scale fails closed with
    /// [`EventError::UnsupportedTimeScale`]. Internally, body positions are
    /// sampled in TDB and Earth rotation (sidereal time, hour angle, diurnal
    /// parallax) at the UT1 re-expression of each sampled day, so a TT-tagged
    /// query built from civil time via `pleiades-time` searches from the
    /// intended physical instant. The returned [`RiseSet::instant`] is TDB,
    /// tagged `Tdb`, regardless of the query's tag — the same convention as
    /// every other event surface in this crate. Accuracy in UT/civil time is
    /// bounded by the packaged ΔT model (observed through 2020, extrapolated
    /// beyond).
    ///
    /// # Search window
    ///
    /// For `Rise`/`Set`, the search
    /// is additionally bounded to `RISE_SET_SEARCH_SPAN_DAYS` past `after` —
    /// a short-horizon search in the same spirit as SE's `swe_rise_trans`
    /// (which reports "no event" past its own, narrower ~28h window) but not
    /// numerically matching it: `RISE_SET_SEARCH_SPAN_DAYS` is a wider,
    /// Moon-covering superset (see its doc comment for why). A body that is
    /// circumpolar right now and does not rise/set again within that span
    /// returns `None`, even though it may rise far in the future (use
    /// `rise_sets_in_range` with an explicit, longer window for that
    /// question). A meridian transit is searched for 1.5 days ahead, which
    /// always holds the next one, so a transit search never returns `None`.
    ///
    /// # The window's ends
    ///
    /// The search runs to the ephemeris window's last instant and finds an
    /// event there. When the window ends before the search span does and no
    /// event lies before its end, the result is
    /// [`EventError::OutOfWindow`], naming the first instant past the window
    /// the search needed: the event may exist, but it cannot be computed.
    /// `Ok(None)` therefore means only that the span holds no event, which
    /// is the answer for a circumpolar target.
    ///
    /// [`previous_rise_set`](Self::previous_rise_set) answers the same way
    /// at the window's start. There a planet has one more limit: its
    /// apparent place is read a light-time earlier, so it cannot be read
    /// within a light-time of the window's first instant, and a search that
    /// needs a sample there is `OutOfWindow` as well. The Sun, a fixed star
    /// and an ecliptic point are readable from the first instant on.
    ///
    /// # Chaining searches
    ///
    /// Every returned instant is the settled end of its refinement bracket:
    /// the event has already happened there, less than 0.5 s earlier. The
    /// Sun is up at a returned sunrise, down at a returned sunset, and past
    /// the meridian at a returned transit. Which side of a query instant an
    /// event falls on is likewise read from the target's state AT that
    /// instant, not from comparing two refined instants.
    ///
    /// An instant this engine returned can therefore be handed straight
    /// back:
    ///
    /// - as `after`, the search steps past the event the instant describes
    ///   and returns the following one;
    /// - as `before` to [`previous_rise_set`](Self::previous_rise_set), the
    ///   search returns that same event;
    /// - a rise searched from a returned set, or a set from a returned rise,
    ///   is found however short the night or day between them, as it is
    ///   from any other instant.
    ///
    /// This holds exactly when the follow-on search uses the same target,
    /// observer, atmosphere, and options, and the instant is passed back
    /// unchanged. A different disc or refraction setting defines a different
    /// event, and an instant converted to another time scale and back may
    /// move by a rounding step.
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
        let after_jd = tdb_jd(after)?;
        self.check_window(after_jd)?;
        match event {
            RiseSetEvent::Rise | RiseSetEvent::Set => {
                let want_ascending = matches!(event, RiseSetEvent::Rise);
                let track = self.track(&target);
                let root = first_directed_crossing_after(
                    |jd| {
                        self.horizon_residual(&target, &observer, &opts, atmos, jd, track.as_ref())
                    },
                    after_jd,
                    after_jd + RISE_SET_SEARCH_SPAN_DAYS,
                    RISE_SET_STEP_DAYS,
                    WINDOW,
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
    /// here and events after it to `next_rise_set`. The partition holds at
    /// every instant, including the ones the engine returns: given a
    /// returned instant, this finds the event it describes and
    /// `next_rise_set` the one after it (see "Chaining searches" there).
    ///
    /// This is the question observer-local calendars ask most — the Hindu
    /// civil day, its vara and hora, and every muhurta and panchanga reading
    /// are anchored to the sunrise that began the day containing the query
    /// instant, i.e. the last sunrise at or before it.
    ///
    /// Same options, time-scale handling (`before` read by its tag, result
    /// TDB), and bounded-window semantics as `next_rise_set`: `Rise`/`Set`
    /// search only `RISE_SET_SEARCH_SPAN_DAYS`
    /// back from `before`, so a body that has been circumpolar for longer
    /// than that returns `None`; meridian transits always occur within a
    /// sidereal day and are unaffected. A search the window's start cuts
    /// short, with no event after the start, is [`EventError::OutOfWindow`]
    /// (see "The window's ends" there). Early-terminating: the search walks
    /// backward from `before` and stops at the first event found, so its
    /// cost does not depend on how far back the event is within the span.
    /// The result agrees with `rise_sets_in_range(before − span, before)
    /// .last()` to within the refinement tolerance.
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
        let before_jd = tdb_jd(before)?;
        self.check_window(before_jd)?;
        match event {
            RiseSetEvent::Rise | RiseSetEvent::Set => {
                let want_ascending = matches!(event, RiseSetEvent::Rise);
                let track = self.track(&target);
                let root = last_directed_crossing_before(
                    |jd| {
                        self.horizon_residual(&target, &observer, &opts, atmos, jd, track.as_ref())
                    },
                    before_jd - RISE_SET_SEARCH_SPAN_DAYS,
                    before_jd,
                    RISE_SET_STEP_DAYS,
                    WINDOW,
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

    /// All rise/set/transit events of `event` kind after `start` and up to
    /// `end`, ascending. `start` and `end` are read by their `TimeScale` tags
    /// and every returned instant is TDB, as for
    /// [`next_rise_set`](Self::next_rise_set).
    ///
    /// `start` is exclusive in the sense of `next_rise_set`: an event that
    /// has already happened at `start`, such as the one a returned instant
    /// describes, is left out, so consecutive ranges that share a returned
    /// instant as their boundary do not report it twice. At `end` the
    /// returned instant itself is compared, so an event within the 0.5 s
    /// refinement tolerance before `end` may be left out.
    ///
    /// Both ends must lie inside the ephemeris window, and events are found
    /// right up to either end of it. A planet's range that starts within a
    /// light-time of the window's first instant is
    /// [`EventError::OutOfWindow`]: its apparent place cannot be read there.
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
        let start_jd = tdb_jd(start)?;
        let end_jd = tdb_jd(end)?;
        self.check_window(start_jd)?;
        self.check_window(end_jd)?;
        match event {
            RiseSetEvent::Rise | RiseSetEvent::Set => {
                let want_ascending = matches!(event, RiseSetEvent::Rise);
                let track = self.track(&target);
                let roots = directed_crossings_in_range(
                    |jd| {
                        self.horizon_residual(&target, &observer, &opts, atmos, jd, track.as_ref())
                    },
                    start_jd,
                    end_jd,
                    RISE_SET_STEP_DAYS,
                    WINDOW,
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
    /// zero of `H − 180` (also wrapped). The residual ascends through its
    /// single zero per sidereal day (LST advances ~361°/day against a
    /// slowly-moving target RA) and falls 360° at the wrap seam half a day
    /// later. The transit searches therefore ask the `scan` module for
    /// ASCENDING crossings: the seam reads as a descending sign change, which
    /// the scanner skips without refining, and the residual has no same-sign
    /// extremum for the culmination check to act on. Sharing the scanner
    /// gives transits the same query-anchored grid as rise/set, so both
    /// partition the event sequence the same way.
    fn hour_angle_residual(
        &self,
        target: &RiseSetTarget,
        observer: &ObserverLocation,
        opts: &RiseSetOptions,
        lower: bool,
        jd: f64,
        track: Option<&BodyTrack<'_, B>>,
    ) -> Result<f64, EventError> {
        let (ra, _dec) = self.target_equatorial(target, observer, opts, jd, track)?;
        let lst = local_apparent_sidereal_deg(jd, observer.longitude)?;
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
        let after_jd = tdb_jd(after)?;
        let track = self.track(&target);
        let root = first_directed_crossing_after(
            |jd| self.hour_angle_residual(&target, &observer, &opts, lower, jd, track.as_ref()),
            after_jd,
            after_jd + TRANSIT_SEARCH_SPAN_DAYS,
            TRANSIT_STEP_DAYS,
            WINDOW,
            true,
        )?;
        Ok(root.filter(|&jd| jd > after_jd).map(|jd| RiseSet {
            event,
            target: target.clone(),
            instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
        }))
    }

    /// Last meridian transit at or before `before`. Early-terminating: walks
    /// a grid anchored at `before` backward and stops at the first transit
    /// found, always within a sidereal day.
    pub(crate) fn previous_transit(
        &self,
        target: RiseSetTarget,
        event: RiseSetEvent,
        observer: ObserverLocation,
        opts: RiseSetOptions,
        before: Instant,
    ) -> Result<Option<RiseSet>, EventError> {
        let lower = matches!(event, RiseSetEvent::LowerTransit);
        let before_jd = tdb_jd(before)?;
        let track = self.track(&target);
        let root = last_directed_crossing_before(
            |jd| self.hour_angle_residual(&target, &observer, &opts, lower, jd, track.as_ref()),
            before_jd - TRANSIT_SEARCH_SPAN_DAYS,
            before_jd,
            TRANSIT_STEP_DAYS,
            WINDOW,
            true,
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
        let track = self.track(&target);
        let roots = directed_crossings_in_range(
            |jd| self.hour_angle_residual(&target, &observer, &opts, lower, jd, track.as_ref()),
            tdb_jd(start)?,
            tdb_jd(end)?,
            TRANSIT_STEP_DAYS,
            WINDOW,
            true,
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

mod track;

#[cfg(test)]
mod test_support;

#[cfg(test)]
mod time_scale_tests;

#[cfg(test)]
mod chain_tests;

#[cfg(test)]
mod window_edge_tests;

#[cfg(test)]
mod cost_tests;

#[cfg(test)]
mod cache_tests;
