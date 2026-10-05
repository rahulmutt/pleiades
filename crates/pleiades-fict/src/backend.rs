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
    let mut d = after - before;
    while d > 180.0 {
        d -= 360.0;
    }
    while d < -180.0 {
        d += 360.0;
    }
    d
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
mod tests {
    use super::*;

    // A trivial Sun source placing the Sun at 1 AU along +x (geocentric), i.e.
    // Earth at -1 AU heliocentric — enough to exercise the assembly path.
    #[derive(Debug, Clone)]
    struct StubSun;
    impl EphemerisBackend for StubSun {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("stub-sun"),
                version: "0.0.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance {
                    summary: "stub Sun source for FictitiousBackend tests".to_string(),
                    data_sources: vec![],
                },
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
                body_claims: vec![BodyClaim::release_grade(
                    CelestialBody::Sun,
                    AccuracyClass::Exact,
                    ClaimEvidence::AlgorithmicModel,
                )],
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
            body == CelestialBody::Sun
        }
        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            let mut r = EphemerisResult::new(
                BackendId::new("stub-sun"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            );
            r.ecliptic = Some(EclipticCoordinates::new(
                Longitude::from_degrees(0.0),
                Latitude::from_degrees(0.0),
                Some(1.0),
            ));
            Ok(r)
        }
    }

    #[test]
    fn supports_only_fictitious_bodies() {
        let b = FictitiousBackend::new(StubSun);
        assert!(b.supports_body(CelestialBody::Cupido));
        assert!(!b.supports_body(CelestialBody::Mars));
    }

    #[test]
    fn position_returns_ecliptic_and_motion_for_a_fictitious_body() {
        let b = FictitiousBackend::new(StubSun);
        let req = EphemerisRequest::new(
            CelestialBody::Cupido,
            Instant::new(JulianDay::from_days(crate::J2000_JD), TimeScale::Tt),
        );
        let r = b.position(&req).unwrap();
        assert!(r.ecliptic.is_some());
        assert!(r.motion.is_some());
    }

    // A Sun source with a hard bounded window `[LO, HI]`, rejecting any
    // instant outside it — mirroring a packaged ephemeris backend's fixed
    // data range (e.g. exactly 1900-01-01..2100-01-01 TT).
    #[derive(Debug, Clone)]
    struct WindowedSun {
        lo: f64,
        hi: f64,
    }
    impl EphemerisBackend for WindowedSun {
        fn metadata(&self) -> BackendMetadata {
            StubSun.metadata()
        }
        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Sun
        }
        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            let jd = req.instant.julian_day.days();
            if jd < self.lo || jd > self.hi {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::OutOfRangeInstant,
                    "outside the windowed Sun source's covered range",
                ));
            }
            StubSun.position(req)
        }
    }

    #[test]
    fn motion_falls_back_to_one_sided_difference_at_a_data_window_edge() {
        // The Sun source only covers [LO, HI]; a symmetric ±0.5 d probe at
        // either edge would need Sun data half a day beyond the boundary.
        // Before the fallback, `position()` failed outright here.
        const LO: f64 = 2_415_020.5;
        const HI: f64 = 2_488_069.5;
        let b = FictitiousBackend::new(WindowedSun { lo: LO, hi: HI });

        for edge in [LO, HI] {
            let req = EphemerisRequest::new(
                CelestialBody::Cupido,
                Instant::new(JulianDay::from_days(edge), TimeScale::Tt),
            );
            let r = b
                .position(&req)
                .unwrap_or_else(|e| panic!("position at window edge {edge} failed: {e}"));
            assert!(r.ecliptic.is_some());
            assert!(r.motion.is_some());
        }
    }

    #[test]
    fn unsupported_body_fails_closed() {
        let b = FictitiousBackend::new(StubSun);
        let req = EphemerisRequest::new(
            CelestialBody::Mars,
            Instant::new(JulianDay::from_days(crate::J2000_JD), TimeScale::Tt),
        );
        assert!(b.position(&req).is_err());
    }

    #[test]
    fn position_without_motion_is_position_minus_motion() {
        let backend = FictitiousBackend::new(StubSun);
        for body in [CelestialBody::Cupido, CelestialBody::WhiteMoon] {
            for jd in [2_451_545.0, 2_460_763.5] {
                let req = EphemerisRequest::new(
                    body.clone(),
                    Instant::new(JulianDay::from_days(jd), TimeScale::Tt),
                );
                let full = backend.position(&req).unwrap();
                let free = backend.position_without_motion(&req).unwrap();
                assert!(full.motion.is_some());
                assert_eq!(
                    EphemerisResult {
                        motion: None,
                        ..full
                    },
                    free,
                    "{body:?} {jd}"
                );
            }
        }
    }

    #[test]
    fn position_without_motion_at_the_sun_sources_window_edge() {
        // At either edge of a bounded Sun source the speed is one-sided; the
        // motion-free place must equal position's.
        const LO: f64 = 2_415_020.5;
        const HI: f64 = 2_488_069.5;
        let b = FictitiousBackend::new(WindowedSun { lo: LO, hi: HI });
        for edge in [LO, HI] {
            let req = EphemerisRequest::new(
                CelestialBody::Cupido,
                Instant::new(JulianDay::from_days(edge), TimeScale::Tt),
            );
            let full = b.position(&req).unwrap();
            let free = b.position_without_motion(&req).unwrap();
            assert_eq!(
                EphemerisResult {
                    motion: None,
                    ..full
                },
                free,
                "{edge}"
            );
        }
    }

    #[test]
    fn position_without_motion_succeeds_where_only_the_motion_fails() {
        // The documented widening: a Sun source covering only the instant
        // itself fails both speed probes, so `position` errors, but the place
        // alone is available.
        const AT: f64 = 2_451_545.0;
        let b = FictitiousBackend::new(WindowedSun { lo: AT, hi: AT });
        let req = EphemerisRequest::new(
            CelestialBody::Cupido,
            Instant::new(JulianDay::from_days(AT), TimeScale::Tt),
        );
        assert!(b.position(&req).is_err());
        let free = b.position_without_motion(&req).unwrap();
        assert!(free.ecliptic.is_some());
        assert_eq!(free.motion, None);
    }

    #[test]
    fn the_sun_source_is_read_without_motion() {
        use core::sync::atomic::{AtomicUsize, Ordering};
        struct CountingSun {
            full: AtomicUsize,
            motion_free: AtomicUsize,
        }
        impl EphemerisBackend for CountingSun {
            fn metadata(&self) -> BackendMetadata {
                StubSun.metadata()
            }
            fn supports_body(&self, body: CelestialBody) -> bool {
                StubSun.supports_body(body)
            }
            fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
                self.full.fetch_add(1, Ordering::SeqCst);
                StubSun.position(req)
            }
            fn position_without_motion(
                &self,
                req: &EphemerisRequest,
            ) -> Result<EphemerisResult, EphemerisError> {
                self.motion_free.fetch_add(1, Ordering::SeqCst);
                StubSun.position_without_motion(req)
            }
        }
        let backend = FictitiousBackend::new(CountingSun {
            full: AtomicUsize::new(0),
            motion_free: AtomicUsize::new(0),
        });
        let req = EphemerisRequest::new(
            CelestialBody::Cupido,
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        );
        backend.position(&req).unwrap();
        assert_eq!(backend.sun_source.full.load(Ordering::SeqCst), 0);
        // The place plus two speed probes.
        assert_eq!(backend.sun_source.motion_free.load(Ordering::SeqCst), 3);
    }
}
