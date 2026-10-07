//! A body's apparent place over one rise/set or transit search, read at a
//! few lattice instants and interpolated between them (issue #204).
//!
//! A search evaluates its residual some twenty to thirty times: the hourly
//! grid out to the event, then the bisection. Reading the ephemeris for each
//! evaluation was 96 % of a search's cost. The body's geocentric apparent
//! place is smooth on the scale of hours, so a cubic through four samples
//! reproduces it far inside the search's own tolerance: a search then costs
//! the handful of lattice samples its span touches, however many times the
//! residual is evaluated.
//!
//! The lattice sits at absolute multiples of its step, not at offsets from
//! the query instant, and the four samples used at an instant are fixed by
//! that instant alone. The place a track returns is therefore a function of
//! the instant only, whichever search asks: the residual two chained searches
//! see is one function, which is what lets a search anchored at a returned
//! instant step past the event it describes (issues #80, #81).
//!
//! The samples live in the engine's [`PlaceCache`], not in the track, so
//! every rise/set and transit search one engine makes shares them: a daily
//! bracket of three searches, or a sweep of brackets over consecutive days,
//! reads each lattice sample once. A sample depends on the body and the
//! lattice instant alone, never on the observer, the atmosphere or the
//! options, so the cache is a pure memo: what it holds changes how many
//! reads a search makes, never what the search returns.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::ephemeris::geocentric_apparent_ecliptic;
use crate::error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
use pleiades_backend::EphemerisBackend;
use pleiades_types::CelestialBody;

/// Geocentric apparent ecliptic longitude (degrees), latitude (degrees) and
/// distance (AU).
pub(crate) type Place = (f64, f64, f64);

/// Lattice step for the Moon and Mercury: 6 hours. Measured 2026-10-06, a
/// cubic through four samples 6 hours apart reproduces the Moon's apparent
/// place within 0.020″ in longitude and 0.014″ in latitude, about 1.3 ms of
/// rise time, and Mercury's within 0.003″. At 12 hours the Moon is 0.33″ out
/// and Mercury 0.046″.
const FAST_STEP_DAYS: f64 = 0.25;

/// Lattice step for the Sun and the planets from Venus out: 12 hours.
/// Measured the same way, the worst is Venus, 0.002″ in longitude; the Sun
/// stays within 0.0001″.
const SLOW_STEP_DAYS: f64 = 0.5;

/// The lattice step for `body`, or `None` for a body whose place is read at
/// every instant. Only the bodies the steps above were measured for are
/// interpolated: an asteroid passing close to the Earth, or a lunar point,
/// need not be smooth on these steps.
fn lattice_step_days(body: &CelestialBody) -> Option<f64> {
    match body {
        CelestialBody::Moon | CelestialBody::Mercury => Some(FAST_STEP_DAYS),
        CelestialBody::Sun
        | CelestialBody::Venus
        | CelestialBody::Mars
        | CelestialBody::Jupiter
        | CelestialBody::Saturn
        | CelestialBody::Uranus
        | CelestialBody::Neptune
        | CelestialBody::Pluto => Some(SLOW_STEP_DAYS),
        _ => None,
    }
}

/// Most samples a [`PlaceCache`] holds before it starts over: about eleven
/// years of the Sun on its 12-hour lattice, or two of the Moon on its 6-hour
/// one, in well under a megabyte.
const PLACE_CACHE_CAPACITY: usize = 8192;

/// The lattice samples an engine has read, shared by all of its rise/set
/// and transit searches (issue #204).
///
/// Keyed by body and lattice index. `None` records a sample the window does
/// not reach, as [`BodyTrack`] defines it. Only those two outcomes are kept;
/// a backend error is returned to the caller and read again next time.
///
/// When an insert would exceed the capacity, the whole table is cleared
/// first. Clearing changes only how many reads later searches make.
///
/// The lock is held only to look up or insert, never across a backend
/// read. Two searches that miss the same sample at once both read it and
/// insert the same value. Each insert writes one complete entry, so a panic
/// can never leave the table inconsistent, and a poisoned lock is simply
/// taken over.
pub(crate) struct PlaceCache {
    capacity: usize,
    entries: Mutex<Entries>,
}

#[derive(Default)]
struct Entries {
    len: usize,
    by_body: HashMap<CelestialBody, HashMap<i64, Option<Place>>>,
}

impl Default for PlaceCache {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaceCache {
    pub(crate) fn new() -> Self {
        Self::with_capacity(PLACE_CACHE_CAPACITY)
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            entries: Mutex::new(Entries::default()),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Entries> {
        self.entries.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The remembered sample, or `None` when it has not been read.
    pub(crate) fn get(&self, body: &CelestialBody, index: i64) -> Option<Option<Place>> {
        self.lock().by_body.get(body)?.get(&index).copied()
    }

    pub(crate) fn insert(&self, body: &CelestialBody, index: i64, place: Option<Place>) {
        let mut guard = self.lock();
        let entries = &mut *guard;
        if entries
            .by_body
            .get(body)
            .is_some_and(|samples| samples.contains_key(&index))
        {
            return;
        }
        if entries.len >= self.capacity {
            entries.by_body.clear();
            entries.len = 0;
        }
        entries
            .by_body
            .entry(body.clone())
            .or_default()
            .insert(index, place);
        entries.len += 1;
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.lock().len
    }
}

/// One body's apparent place for the length of one search, read through
/// the engine's [`PlaceCache`].
pub(crate) struct BodyTrack<'a, B> {
    backend: &'a B,
    cache: &'a PlaceCache,
    body: CelestialBody,
    step_days: f64,
}

impl<'a, B: EphemerisBackend> BodyTrack<'a, B> {
    /// A track for `body`, or `None` for a body that is not interpolated.
    pub(crate) fn new(backend: &'a B, cache: &'a PlaceCache, body: &CelestialBody) -> Option<Self> {
        lattice_step_days(body).map(|step_days| Self {
            backend,
            cache,
            body: body.clone(),
            step_days,
        })
    }

    /// The body's geocentric apparent place at `jd` (TDB).
    ///
    /// Interpolated from the four lattice samples around `jd`. Where one of
    /// them cannot be read, within a lattice step or two of either end of the
    /// ephemeris window, the place is read at `jd` itself, so an instant the
    /// engine can serve is never refused for its neighbours' sake.
    pub(crate) fn place(&self, jd: f64) -> Result<Place, EventError> {
        let index = (jd / self.step_days).floor() as i64;
        let mut stencil = [(0.0, 0.0, 0.0); 4];
        for (slot, offset) in (-1..=2).enumerate() {
            match self.sample(index + offset)? {
                Some(place) => stencil[slot] = place,
                None => return self.read(jd),
            }
        }
        let u = jd / self.step_days - index as f64;
        Ok(interpolate(stencil, u))
    }

    /// The place at `jd` itself, read from the backend.
    fn read(&self, jd: f64) -> Result<Place, EventError> {
        geocentric_apparent_ecliptic(self.backend, self.body.clone(), "body", jd)
    }

    /// The lattice sample at `index`, read on first use by any search of the
    /// engine. `None` where the window does not reach: the lattice instant
    /// lies outside it, or the read there reports `OutOfWindow` (a planet
    /// within light-time of the window's start).
    fn sample(&self, index: i64) -> Result<Option<Place>, EventError> {
        if let Some(place) = self.cache.get(&self.body, index) {
            return Ok(place);
        }
        let jd = index as f64 * self.step_days;
        let place = if (WINDOW_START_JD..=WINDOW_END_JD).contains(&jd) {
            match self.read(jd) {
                Ok(place) => Some(place),
                Err(EventError::OutOfWindow { .. }) => None,
                Err(error) => return Err(error),
            }
        } else {
            None
        };
        self.cache.insert(&self.body, index, place);
        Ok(place)
    }
}

/// The cubic through four equally spaced samples, evaluated a fraction `u`
/// of a step past the second one (`0 ≤ u < 1`). Longitudes are unwrapped
/// against the first sample, so a body crossing 0° is interpolated across
/// the seam.
fn interpolate(stencil: [Place; 4], u: f64) -> Place {
    let weights = [
        -u * (u - 1.0) * (u - 2.0) / 6.0,
        (u + 1.0) * (u - 1.0) * (u - 2.0) / 2.0,
        -(u + 1.0) * u * (u - 2.0) / 2.0,
        (u + 1.0) * u * (u - 1.0) / 6.0,
    ];
    let first_longitude = stencil[0].0;
    let mut place = (0.0, 0.0, 0.0);
    for ((longitude, latitude, distance), weight) in stencil.into_iter().zip(weights) {
        let unwrapped =
            first_longitude + (longitude - first_longitude + 180.0).rem_euclid(360.0) - 180.0;
        place.0 += weight * unwrapped;
        place.1 += weight * latitude;
        place.2 += weight * distance;
    }
    (place.0.rem_euclid(360.0), place.1, place.2)
}

#[cfg(test)]
mod tests;
