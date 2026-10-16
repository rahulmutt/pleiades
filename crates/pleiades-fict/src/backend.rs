//! `FictitiousBackend` — serves the SE fictitious bodies through the standard
//! backend trait. Heliocentric bodies are geocentricized by reusing a Sun-source
//! backend (Earth heliocentric = − Sun geocentric); geocentric-orbit bodies are
//! returned directly. Output is mean, geometric, geocentric, J2000 mean ecliptic.

use pleiades_backend::{
    validate_observer_policy, validate_request_policy, validate_zodiac_policy, AccuracyClass,
    BackendCapabilities, BackendFamily, BackendId, BackendMetadata, BackendProvenance, BodyClaim,
    ClaimEvidence, EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest,
    EphemerisResult, QualityAnnotation,
};
use pleiades_types::{
    CelestialBody, CoordinateFrame, EclipticCoordinates, Instant, JulianDay, Latitude, Longitude,
    Motion, TimeRange, TimeScale, ZodiacMode,
};

use crate::elements::{elements_for, Center, KeplerElements};
use crate::PACKAGE_NAME;

/// The 19 SE fictitious bodies this backend serves.
pub fn fictitious_bodies() -> Vec<CelestialBody> {
    crate::elements::TABLE
        .iter()
        .map(|(b, _)| b.clone())
        .collect()
}

/// Body claims: every fictitious body is release-grade *by definition* — parity
/// with SE's `seorbel.txt`-driven Kepler orbit, gated by `validate-fictitious`.
pub fn fictitious_body_claims() -> Vec<BodyClaim> {
    fictitious_bodies()
        .into_iter()
        .map(|body| {
            BodyClaim::release_grade(body, AccuracyClass::Exact, ClaimEvidence::AlgorithmicModel)
        })
        .collect()
}

/// A fictitious-body backend parameterized over a Sun-source backend `S`.
#[derive(Debug, Clone)]
pub struct FictitiousBackend<S> {
    sun_source: S,
}

impl<S: EphemerisBackend> FictitiousBackend<S> {
    /// Create a backend that uses `sun_source` for the Sun's geocentric position.
    pub const fn new(sun_source: S) -> Self {
        Self { sun_source }
    }

    /// Earth's heliocentric J2000-ecliptic Cartesian position (AU) at `instant`,
    /// obtained as the negation of the Sun's geocentric position from the source.
    fn earth_heliocentric(&self, instant: Instant) -> Result<[f64; 3], EphemerisError> {
        let req = EphemerisRequest::new(CelestialBody::Sun, instant);
        // Only the Sun's place is used; its motion would cost a bounded or
        // finite-differencing source extra evaluations (issue #128).
        let sun = self.sun_source.position_without_motion(&req)?;
        let ecl = sun.ecliptic.ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::MissingDataset,
                "Sun source returned no ecliptic position for the fictitious-body geocentric assembly",
            )
        })?;
        let (sx, sy, sz) = spherical_to_cartesian(&ecl);
        Ok([-sx, -sy, -sz])
    }

    /// Geocentric J2000-mean-ecliptic coordinates for a fictitious body at `instant`.
    fn geocentric_ecliptic(
        &self,
        el: &KeplerElements,
        instant: Instant,
    ) -> Result<EclipticCoordinates, EphemerisError> {
        let jd = instant.julian_day.days();
        let (bx, by, bz) = el.state_at(jd);
        let (gx, gy, gz) = match el.center {
            Center::Geocentric => (bx, by, bz),
            Center::Heliocentric => {
                let earth = self.earth_heliocentric(instant)?;
                (bx - earth[0], by - earth[1], bz - earth[2])
            }
        };
        Ok(cartesian_to_ecliptic(gx, gy, gz))
    }

    /// Symmetric finite-difference motion (±0.5 d). The fictitious bodies are
    /// slow, so the span's truncation is negligible; the VSOP87 and ELP
    /// backends use a shorter one (issue #140).
    ///
    /// Falls back to a one-sided (±0.5 d against `instant` itself) difference
    /// when only one of the two symmetric probes is available. This matters at
    /// the edges of a Sun-source backend's bounded ephemeris window (e.g. a
    /// packaged-data window of exactly 1900-01-01..2100-01-01 TT): a
    /// heliocentric fictitious body sampled at the window's first or last
    /// instant needs the Sun's position half a day beyond that boundary to
    /// take a central difference, which a bounded backend correctly rejects.
    /// The body's own position at `instant` is always available (it is the
    /// value `position()` just computed), so a one-sided difference against
    /// it is used instead of failing the whole request over a motion-only
    /// edge effect.
    fn motion(&self, el: &KeplerElements, instant: Instant) -> Result<Motion, EphemerisError> {
        const HALF: f64 = 0.5;
        let at = |offset: f64| -> Result<EclipticCoordinates, EphemerisError> {
            let shifted = Instant::new(
                JulianDay::from_days(instant.julian_day.days() + offset),
                instant.scale,
            );
            self.geocentric_ecliptic(el, shifted)
        };
        let (before, after, full) = match (at(-HALF), at(HALF)) {
            (Ok(before), Ok(after)) => (before, after, HALF * 2.0),
            (Err(_), Ok(after)) => (self.geocentric_ecliptic(el, instant)?, after, HALF),
            (Ok(before), Err(_)) => (before, self.geocentric_ecliptic(el, instant)?, HALF),
            (Err(e), Err(_)) => return Err(e),
        };
        let lon_speed =
            signed_longitude_delta(before.longitude.degrees(), after.longitude.degrees()) / full;
        let lat_speed = (after.latitude.degrees() - before.latitude.degrees()) / full;
        let dist_speed = match (before.distance_au, after.distance_au) {
            (Some(b), Some(a)) => Some((a - b) / full),
            _ => None,
        };
        Ok(Motion::new(Some(lon_speed), Some(lat_speed), dist_speed))
    }

    /// Shared body of `position` and `position_without_motion`; the speed is
    /// computed only when `with_motion` is set.
    fn compute(
        &self,
        req: &EphemerisRequest,
        with_motion: bool,
    ) -> Result<EphemerisResult, EphemerisError> {
        let el = elements_for(req.body.clone()).ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                "the fictitious backend serves only SE seorbel.txt bodies 40–58",
            )
        })?;

        validate_zodiac_policy(req, "the fictitious backend", &[ZodiacMode::Tropical])?;
        validate_request_policy(
            req,
            "the fictitious backend",
            &[TimeScale::Tt, TimeScale::Tdb],
            &[CoordinateFrame::Ecliptic],
            true,
            false,
        )?;
        validate_observer_policy(req, "the fictitious backend", false)?;

        let mut result = EphemerisResult::new(
            BackendId::new(PACKAGE_NAME),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        result.quality = QualityAnnotation::Exact;
        result.ecliptic = Some(self.geocentric_ecliptic(el, req.instant)?);
        if with_motion {
            result.motion = Some(self.motion(el, req.instant)?);
        }
        Ok(result)
    }
}

fn spherical_to_cartesian(ecl: &EclipticCoordinates) -> (f64, f64, f64) {
    let r = ecl.distance_au.unwrap_or(1.0);
    let lon = ecl.longitude.degrees().to_radians();
    let lat = ecl.latitude.degrees().to_radians();
    (
        r * lat.cos() * lon.cos(),
        r * lat.cos() * lon.sin(),
        r * lat.sin(),
    )
}

fn cartesian_to_ecliptic(x: f64, y: f64, z: f64) -> EclipticCoordinates {
    let r = (x * x + y * y + z * z).sqrt();
    let lon = y.atan2(x).to_degrees().rem_euclid(360.0);
    let lat = if r == 0.0 {
        0.0
    } else {
        (z / r).asin().to_degrees()
    };
    EclipticCoordinates::new(
        Longitude::from_degrees(lon),
        Latitude::from_degrees(lat),
        Some(r),
    )
}

fn signed_longitude_delta(before: f64, after: f64) -> f64 {
    // `%` is exact and keeps the dividend's sign, so `d` is in (-360, 360).
    // Same results as the former normalising loops on every finite input
    // (both map into [-180, 180], keeping +-180 as is); an infinite or NaN
    // delta gives NaN instead of hanging (issue #162).
    let d = (after - before) % 360.0;
    if d > 180.0 {
        d - 360.0
    } else if d < -180.0 {
        d + 360.0
    } else {
        d
    }
}

impl<S: EphemerisBackend> EphemerisBackend for FictitiousBackend<S> {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            id: BackendId::new(PACKAGE_NAME),
            version: env!("CARGO_PKG_VERSION").to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance {
                summary: "Fictitious/hypothetical bodies (SE seorbel.txt 40–58) as unperturbed Kepler orbits; definitional parity with Swiss Ephemeris via validate-fictitious.".to_string(),
                data_sources: vec![
                    "Osculating elements transcribed from Swiss Ephemeris seorbel.txt; unperturbed Kepler propagation in pure Rust.".to_string(),
                ],
            },
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
            body_claims: fictitious_body_claims(),
            supported_frames: vec![CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities {
                geocentric: true,
                topocentric: false,
                apparent: false,
                mean: true,
                batch: true,
                native_sidereal: false,
            },
            accuracy: AccuracyClass::Exact,
            deterministic: true,
            offline: true,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        elements_for(body).is_some()
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        self.compute(req, true)
    }

    fn position_without_motion(
        &self,
        req: &EphemerisRequest,
    ) -> Result<EphemerisResult, EphemerisError> {
        self.compute(req, false)
    }
}

#[cfg(test)]
mod tests;
