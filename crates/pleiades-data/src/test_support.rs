//! Shared test setup helpers for the `pleiades-data` test suite.
//!
//! Builders and fixtures extracted from repeated arrange blocks in tests so
//! every test file can import them with `use crate::test_support::*;`.

use crate::*;

/// Build a TT-scale [`Instant`] from a Julian-day number.
pub(crate) fn instant_tt(days: f64) -> Instant {
    Instant::new(JulianDay::from_days(days), TimeScale::Tt)
}

/// Shared synthetic ephemeris backend for kernel-free unit tests.
///
/// Longitude advances 1 deg/day, latitude has a small sinusoidal wobble, and
/// distance stays near 1 AU. All values are smooth analytic functions, making
/// them easy to fit exactly with a degree-8 polynomial over small spans.
pub(crate) struct Synthetic;

impl pleiades_backend::EphemerisBackend for Synthetic {
    fn metadata(&self) -> pleiades_backend::BackendMetadata {
        unimplemented!()
    }

    fn supports_body(&self, _body: pleiades_backend::CelestialBody) -> bool {
        true
    }

    fn position(
        &self,
        req: &pleiades_backend::EphemerisRequest,
    ) -> Result<pleiades_backend::EphemerisResult, pleiades_backend::EphemerisError> {
        let jd = req.instant.julian_day.days();
        let lon = (jd * 1.0).rem_euclid(360.0);
        let lat = 0.1 * (jd / 50.0).sin();
        let dist = 1.0 + 0.01 * (jd / 80.0).cos();
        let mut r = pleiades_backend::EphemerisResult::new(
            pleiades_backend::BackendId::new("synthetic"),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        r.ecliptic = Some(pleiades_backend::EclipticCoordinates::new(
            pleiades_backend::Longitude::from_degrees(lon),
            pleiades_backend::Latitude::from_degrees(lat),
            Some(dist),
        ));
        Ok(r)
    }
}

/// [`Synthetic`], except that it refuses `refused` from `refused_from_jd` on,
/// the way a kernel that does not cover a body over the whole window does.
pub(crate) struct SyntheticRefusing {
    pub(crate) refused: CelestialBody,
    pub(crate) refused_from_jd: f64,
}

impl pleiades_backend::EphemerisBackend for SyntheticRefusing {
    fn metadata(&self) -> pleiades_backend::BackendMetadata {
        unimplemented!()
    }

    fn supports_body(&self, _body: pleiades_backend::CelestialBody) -> bool {
        true
    }

    fn position(
        &self,
        req: &pleiades_backend::EphemerisRequest,
    ) -> Result<pleiades_backend::EphemerisResult, pleiades_backend::EphemerisError> {
        if req.body == self.refused && req.instant.julian_day.days() >= self.refused_from_jd {
            return Err(pleiades_backend::EphemerisError::new(
                pleiades_backend::EphemerisErrorKind::OutOfRangeInstant,
                "no segment covers the instant",
            ));
        }
        Synthetic.position(req)
    }
}
