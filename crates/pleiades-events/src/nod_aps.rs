//! Planetary/lunar orbital nodes and apsides — Swiss Ephemeris `swe_nod_aps`
//! analogue. See `EventEngine::nod_aps`.

use crate::error::EventError;

/// How the orbit is modeled — Swiss Ephemeris `SE_NODBIT_*` analogues.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodApsMethod {
    /// Mean orbital elements (`SE_NODBIT_MEAN`). Moon + Sun + Mercury–Neptune.
    Mean,
    /// Osculating ellipse from the instantaneous state (`SE_NODBIT_OSCU`).
    Osculating,
    /// Osculating ellipse about the solar-system barycenter for bodies beyond
    /// ~6 AU heliocentric distance (`SE_NODBIT_OSCU_BAR`); inside 6 AU this
    /// falls back to the heliocentric ellipse, matching Swiss Ephemeris.
    OsculatingBarycentric,
}

/// What the fourth point means — aphelion or the ellipse's second focus
/// (`SE_NODBIT_FOPOINT`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApsisConvention {
    /// Far apsis at distance `a(1+e)`.
    Aphelion,
    /// Second (empty) focus at distance `2ae`, same direction as the aphelion.
    SecondFocus,
}

/// One orbital point: geocentric true-ecliptic-of-date position and
/// central-difference speeds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodApsPoint {
    /// Ecliptic longitude, degrees in `[0, 360)`, true equinox of date.
    pub longitude_deg: f64,
    /// Ecliptic latitude, degrees in `[-90, 90]`.
    pub latitude_deg: f64,
    /// Geocentric distance, AU.
    pub distance_au: f64,
    /// dλ/dt, degrees/day (central difference over ±0.5 day).
    pub longitude_speed_deg_per_day: f64,
    /// dβ/dt, degrees/day.
    pub latitude_speed_deg_per_day: f64,
    /// d(distance)/dt, AU/day.
    pub distance_speed_au_per_day: f64,
}

/// The four orbital points returned by [`EventEngine::nod_aps`]
/// (ascending node, descending node, perihelion, aphelion-or-focus).
///
/// [`EventEngine::nod_aps`]: crate::EventEngine::nod_aps
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodesApsides {
    /// Ascending-node point.
    pub ascending: NodApsPoint,
    /// Descending-node point.
    pub descending: NodApsPoint,
    /// Near apsis (perihelion; perigee for the Moon).
    pub perihelion: NodApsPoint,
    /// Far apsis — or second focus under [`ApsisConvention::SecondFocus`].
    pub aphelion: NodApsPoint,
    /// The method that actually served the request.
    pub method: NodApsMethod,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crossings::EventEngine;
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};

    fn tdb(jd: f64) -> Instant {
        Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
    }

    #[test]
    fn lunar_point_bodies_are_rejected() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_545.0));
        let err = engine
            .nod_aps(
                CelestialBody::MeanNode,
                tdb(2_451_545.0),
                NodApsMethod::Osculating,
                ApsisConvention::Aphelion,
            )
            .unwrap_err();
        assert!(matches!(err, EventError::UnsupportedNodAps { .. }));
    }

    #[test]
    fn mean_method_for_pluto_is_rejected() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_545.0));
        let err = engine
            .nod_aps(
                CelestialBody::Pluto,
                tdb(2_451_545.0),
                NodApsMethod::Mean,
                ApsisConvention::Aphelion,
            )
            .unwrap_err();
        assert!(matches!(err, EventError::UnsupportedNodAps { .. }));
    }

    #[test]
    fn out_of_window_and_margin_fail_closed() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_545.0));
        for jd in [2_400_000.5, crate::error::WINDOW_START_JD + 0.25] {
            let err = engine
                .nod_aps(
                    CelestialBody::Mars,
                    tdb(jd),
                    NodApsMethod::Mean,
                    ApsisConvention::Aphelion,
                )
                .unwrap_err();
            assert!(matches!(err, EventError::OutOfWindow { .. }), "jd {jd}");
        }
    }

    #[test]
    fn wrap_delta_handles_the_zero_crossing() {
        assert!((wrap_delta(359.5, 0.5) - 1.0).abs() < 1e-12);
        assert!((wrap_delta(0.5, 359.5) + 1.0).abs() < 1e-12);
        assert!((wrap_delta(10.0, 20.0) - 10.0).abs() < 1e-12);
    }

    #[test]
    fn aberration_shifts_by_v_over_c_transverse() {
        // Point on +x, observer velocity on +y: shift ≈ |v|/c radians toward +y.
        let v = 0.0172; // ~Earth orbital speed, AU/day
        let p = aberrate([2.0, 0.0, 0.0], [0.0, v, 0.0]);
        let expected = (v / LIGHT_SPEED_AU_PER_DAY).atan();
        let got = p[1].atan2(p[0]);
        assert!((got - expected).abs() < 1e-9);
        // Distance preserved.
        let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        assert!((r - 2.0).abs() < 1e-12);
    }

    #[test]
    fn osculating_moon_on_linear_backend_is_degenerate_not_panicking() {
        // LinearSunMoon's Moon holds a *constant* geocentric distance
        // (0.00257 AU) and a constant angular rate (13.176396 deg/day). This
        // is not quite consistent with a circular orbit at the real
        // geocentric GM (GEO_GM_AU3_DAY2): v^2*r/mu comes out ~0.998, not
        // exactly 1, so the implied osculating ellipse has a small but
        // genuine, always-nonzero eccentricity (~0.0022 at any instant and
        // any constant latitude — verified analytically: with r*v = 0
        // held by this backend's parametrization, the specific orbital
        // energy stays negative for every latitude in [0, 90) degrees, so
        // the state is never actually unbound or exactly circular). The
        // engine must still produce a finite, sane result here — never NaN
        // or a panic — while continuing to fail closed with a typed error
        // for any genuinely pathological state (e.g. a missing/erroring
        // backend read).
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_545.0));
        let res = engine.nod_aps(
            CelestialBody::Moon,
            tdb(2_451_545.0),
            NodApsMethod::Osculating,
            ApsisConvention::Aphelion,
        );
        match res {
            Ok(points) => {
                for p in [
                    points.ascending,
                    points.descending,
                    points.perihelion,
                    points.aphelion,
                ] {
                    assert!(p.longitude_deg.is_finite(), "{p:?}");
                    assert!(p.latitude_deg.is_finite(), "{p:?}");
                    assert!(p.distance_au.is_finite() && p.distance_au > 0.0, "{p:?}");
                    assert!(p.longitude_speed_deg_per_day.is_finite(), "{p:?}");
                    assert!(p.latitude_speed_deg_per_day.is_finite(), "{p:?}");
                    assert!(p.distance_speed_au_per_day.is_finite(), "{p:?}");
                }
            }
            Err(EventError::DegenerateNodAps { .. })
            | Err(EventError::Backend(_))
            | Err(EventError::MissingCoordinates { .. }) => {}
            Err(other) => panic!("unexpected error variant: {other:?}"),
        }
    }
}

use crate::crossings::EventEngine;
use crate::ephemeris::{read_mean_ecliptic, read_mean_longitude, spherical_to_cartesian};
use crate::mean_elements::{
    elem_index, mean_elements_of_date, mu_au3_day2, EARTH_MOON_MASS_RATIO, MOON_MEAN_ECC,
    MOON_MEAN_INCL_DEG, MOON_MEAN_SEMA_AU,
};
use pleiades_apparent::nutation::nutation;
use pleiades_apparent::precess_ecliptic_j2000_to_date;
use pleiades_apsides::{points_from_elements, ApsisPoint, KeplerianElements};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{CelestialBody, Instant};

/// Speed of light, AU/day.
const LIGHT_SPEED_AU_PER_DAY: f64 = 173.144_632_674_240_3;
/// Half-span for point-speed central differences (mirrors pleiades-data's
/// osculating-apsis motion pattern).
const SPEED_HALF_SPAN_DAYS: f64 = 0.5;
/// The full pipeline samples at up to `jd ± (0.5 + 2·dt)`; keep 1 day clear of
/// the window edges.
const WINDOW_MARGIN_DAYS: f64 = 1.0;

/// One point's position triple before speed assembly.
#[derive(Clone, Copy, Debug)]
struct RawPoint {
    lon_deg: f64,
    lat_deg: f64,
    dist_au: f64,
}

/// Shortest signed longitude difference `b − a` in degrees.
fn wrap_delta(a: f64, b: f64) -> f64 {
    let mut d = (b - a).rem_euclid(360.0);
    if d > 180.0 {
        d -= 360.0;
    }
    d
}

fn norm3(a: [f64; 3]) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn add3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale3(a: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn cartesian_to_raw(p: [f64; 3]) -> RawPoint {
    let r = norm3(p);
    RawPoint {
        lon_deg: p[1].atan2(p[0]).to_degrees().rem_euclid(360.0),
        lat_deg: (p[2] / r).asin().to_degrees(),
        dist_au: r,
    }
}

/// First-order annual aberration: rotate the unit direction by v⊥/c, keep the
/// distance (SE applies `swi_aberr_light` to each returned point).
fn aberrate(p: [f64; 3], v_obs_au_day: [f64; 3]) -> [f64; 3] {
    let r = norm3(p);
    let u = scale3(p, 1.0 / r);
    let shifted = add3(u, scale3(v_obs_au_day, 1.0 / LIGHT_SPEED_AU_PER_DAY));
    scale3(shifted, r / norm3(shifted))
}

fn apsis_to_raw(p: &ApsisPoint) -> RawPoint {
    RawPoint {
        lon_deg: p.longitude_deg,
        lat_deg: p.latitude_deg,
        dist_au: p.distance_au,
    }
}

impl<B: EphemerisBackend> EventEngine<B> {
    /// Nodes and apsides of a body's orbit — Swiss Ephemeris `swe_nod_aps`
    /// analogue. Returns the ascending node, descending node, perihelion, and
    /// aphelion (or second focus) as geocentric true-ecliptic-of-date
    /// positions with ±0.5-day central-difference speeds.
    ///
    /// Fail-closed: `Mean` outside Moon/Sun/Mercury–Neptune, lunar-point
    /// bodies, and out-of-window instants (including a 1-day sampling margin)
    /// return typed errors.
    pub fn nod_aps(
        &self,
        body: CelestialBody,
        instant: Instant,
        method: NodApsMethod,
        convention: ApsisConvention,
    ) -> Result<NodesApsides, EventError> {
        let jd = instant.julian_day.days();
        self.check_window(jd)?;
        // The speed/state sampling reaches jd ± (0.5 + 2·dt); enforce a hard
        // margin so every backend read stays in-window.
        if !((crate::error::WINDOW_START_JD + WINDOW_MARGIN_DAYS)
            ..=(crate::error::WINDOW_END_JD - WINDOW_MARGIN_DAYS))
            .contains(&jd)
        {
            return Err(EventError::OutOfWindow { julian_day: jd });
        }
        if matches!(
            body,
            CelestialBody::MeanNode
                | CelestialBody::TrueNode
                | CelestialBody::MeanApogee
                | CelestialBody::TrueApogee
                | CelestialBody::MeanPerigee
                | CelestialBody::TruePerigee
        ) {
            return Err(EventError::UnsupportedNodAps {
                detail: format!("{body} is itself a node/apsis body"),
            });
        }
        let p0 = self.points_at(&body, jd, method, convention)?;
        let pm = self.points_at(&body, jd - SPEED_HALF_SPAN_DAYS, method, convention)?;
        let pp = self.points_at(&body, jd + SPEED_HALF_SPAN_DAYS, method, convention)?;
        let assemble = |i: usize| NodApsPoint {
            longitude_deg: p0[i].lon_deg,
            latitude_deg: p0[i].lat_deg,
            distance_au: p0[i].dist_au,
            longitude_speed_deg_per_day: wrap_delta(pm[i].lon_deg, pp[i].lon_deg)
                / (2.0 * SPEED_HALF_SPAN_DAYS),
            latitude_speed_deg_per_day: (pp[i].lat_deg - pm[i].lat_deg)
                / (2.0 * SPEED_HALF_SPAN_DAYS),
            distance_speed_au_per_day: (pp[i].dist_au - pm[i].dist_au)
                / (2.0 * SPEED_HALF_SPAN_DAYS),
        };
        Ok(NodesApsides {
            ascending: assemble(0),
            descending: assemble(1),
            perihelion: assemble(2),
            aphelion: assemble(3),
            method,
        })
    }

    /// Swiss Ephemeris `method = 0` default: mean elements where they exist
    /// (Moon, Sun, Mercury–Neptune), osculating otherwise.
    pub fn nod_aps_default(
        &self,
        body: CelestialBody,
        instant: Instant,
        convention: ApsisConvention,
    ) -> Result<NodesApsides, EventError> {
        let method = if body == CelestialBody::Moon || elem_index(&body).is_some() {
            NodApsMethod::Mean
        } else {
            NodApsMethod::Osculating
        };
        self.nod_aps(body, instant, method, convention)
    }

    /// Positions of the four points at one instant (no speeds).
    fn points_at(
        &self,
        body: &CelestialBody,
        jd: f64,
        method: NodApsMethod,
        convention: ApsisConvention,
    ) -> Result<[RawPoint; 4], EventError> {
        match method {
            NodApsMethod::Mean => self.mean_points_at(body, jd, convention),
            NodApsMethod::Osculating | NodApsMethod::OsculatingBarycentric => {
                self.osculating_points_at(body, jd, method, convention)
            }
        }
    }

    /// Geocentric Sun vector in the MEAN ecliptic of date (no Δψ), plus the
    /// observer (Earth) velocity in the same frame for aberration.
    fn sun_geo_mean_of_date(&self, jd: f64) -> Result<([f64; 3], [f64; 3]), EventError> {
        let at = |jd: f64| -> Result<[f64; 3], EventError> {
            let (lon, lat, dist) =
                read_mean_ecliptic(&self.backend, CelestialBody::Sun, "Sun", jd)?;
            let p = precess_ecliptic_j2000_to_date(lon, lat, jd)
                .map_err(|e| EventError::Backend(format!("precession failed: {e}")))?;
            Ok(spherical_to_cartesian(
                p.longitude_deg,
                p.latitude_deg,
                dist,
            ))
        };
        let s0 = at(jd)?;
        let sm = at(jd - SPEED_HALF_SPAN_DAYS)?;
        let sp = at(jd + SPEED_HALF_SPAN_DAYS)?;
        // Earth heliocentric velocity = −d(geocentric Sun)/dt.
        let v_obs = scale3(sub3(sp, sm), -1.0 / (2.0 * SPEED_HALF_SPAN_DAYS));
        Ok((s0, v_obs))
    }

    /// Mean-element points. Built in the mean ecliptic of date, recentered
    /// geocentric, aberrated (except the Moon), then shifted to the true
    /// equinox (`λ += Δψ`).
    fn mean_points_at(
        &self,
        body: &CelestialBody,
        jd: f64,
        convention: ApsisConvention,
    ) -> Result<[RawPoint; 4], EventError> {
        let second_focus = convention == ApsisConvention::SecondFocus;
        let elements = if *body == CelestialBody::Moon {
            // The ELP backend's MeanNode/MeanPerigee longitudes are Meeus-style
            // mean-lunar-element polynomials (e.g. Ω0 = 125.04452° at J2000);
            // those are OF-DATE quantities by construction, despite the module
            // boundary's nominal J2000 labeling — do NOT re-precess them here.
            // Verified by spot check against the committed corpus: at
            // jd 2415100.5 the raw backend read is ~17″ off SE's mean-node
            // longitude (arcsecond-class, as expected for a cross-theory
            // comparison); precessing it as if it were J2000 moves it ~5034″
            // off, matching the diagnosed +1-century MEAN_MOON longitude
            // residual (~5024″ — one full century of precession, 1.396°/cy).
            let node = read_mean_longitude(&self.backend, CelestialBody::MeanNode, "MeanNode", jd)?;
            let peri =
                read_mean_longitude(&self.backend, CelestialBody::MeanPerigee, "MeanPerigee", jd)?;
            KeplerianElements {
                node_deg: node,
                peri_lon_deg: peri,
                incl_deg: MOON_MEAN_INCL_DEG,
                eccentricity: MOON_MEAN_ECC,
                semi_major_au: MOON_MEAN_SEMA_AU,
            }
        } else {
            let idx = elem_index(body).ok_or_else(|| EventError::UnsupportedNodAps {
                detail: format!("no SE mean elements for {body}; use Osculating"),
            })?;
            mean_elements_of_date(idx, jd)
        };
        let pts = points_from_elements(&elements, second_focus).map_err(|e| {
            EventError::DegenerateNodAps {
                detail: format!("{body} mean elements: {e:?}"),
            }
        })?;
        let in_plane = [pts.ascending, pts.descending, pts.perihelion, pts.aphelion];
        let dpsi_deg = nutation(jd)
            .map_err(|e| EventError::Backend(format!("nutation failed: {e}")))?
            .delta_psi_arcsec
            / 3600.0;
        let mut out = [RawPoint {
            lon_deg: 0.0,
            lat_deg: 0.0,
            dist_au: 0.0,
        }; 4];
        if *body == CelestialBody::Moon {
            // Geocentric orbit: points are already geocentric; SE disables
            // aberration for the geocentric Moon.
            for (o, p) in out.iter_mut().zip(in_plane.iter()) {
                let mut raw = apsis_to_raw(p);
                raw.lon_deg = (raw.lon_deg + dpsi_deg).rem_euclid(360.0);
                *o = raw;
            }
            return Ok(out);
        }
        let (sun_geo, v_obs) = self.sun_geo_mean_of_date(jd)?;
        for (o, p) in out.iter_mut().zip(in_plane.iter()) {
            let helio = spherical_to_cartesian(p.longitude_deg, p.latitude_deg, p.distance_au);
            // SE's shared output stage (swecl.c ~5480) always recenters the
            // Earth-orbit point to geocentric FIRST (`xp -= xobs`, i.e.
            // heliocentric + geocentric Sun here), and only THEN, for the
            // Sun exclusively, negates the already-geocentric vector
            // (swecl.c:5482-5484) to produce the geocentric "Black Sun"
            // point. Planets just keep the recentered (non-negated) vector.
            let geo = if *body == CelestialBody::Sun {
                scale3(add3(helio, sun_geo), -1.0)
            } else {
                add3(helio, sun_geo)
            };
            let aberrated = aberrate(geo, v_obs);
            let mut raw = cartesian_to_raw(aberrated);
            raw.lon_deg = (raw.lon_deg + dpsi_deg).rem_euclid(360.0);
            *o = raw;
        }
        Ok(out)
    }

    /// Body geocentric position in the RAW J2000 mean ecliptic frame — no
    /// precession, no nutation. All osculating-state sampling and centering
    /// arithmetic happens in this frame (see [`Self::osculating_state`]);
    /// the single frame rotation to true-of-date is applied once, at the
    /// end, to the assembled center-epoch state.
    fn body_geo_j2000(
        &self,
        body: &CelestialBody,
        label: &'static str,
        jd: f64,
    ) -> Result<[f64; 3], EventError> {
        let (lon, lat, dist) = read_mean_ecliptic(&self.backend, body.clone(), label, jd)?;
        Ok(spherical_to_cartesian(lon, lat, dist))
    }

    /// Sun-to-SSB offset R in the RAW J2000 frame (matching `sun_geo`'s
    /// frame), from the giant-planet heliocentric vectors weighted by their
    /// SE mass ratios (Jupiter–Pluto). `r_bary = r_helio − R`. Rotated to
    /// true-of-date alongside the rest of the state in
    /// [`Self::osculating_state`], not here.
    fn sun_to_ssb_offset(&self, jd: f64, sun_geo: [f64; 3]) -> Result<[f64; 3], EventError> {
        use crate::mean_elements::SUN_MASS_RATIO;
        const GIANTS: [(CelestialBody, &str, usize); 5] = [
            (CelestialBody::Jupiter, "Jupiter", 4),
            (CelestialBody::Saturn, "Saturn", 5),
            (CelestialBody::Uranus, "Uranus", 6),
            (CelestialBody::Neptune, "Neptune", 7),
            (CelestialBody::Pluto, "Pluto", 8),
        ];
        let mut r = [0.0_f64; 3];
        for (body, label, mi) in GIANTS {
            let geo = self.body_geo_j2000(&body, label, jd)?;
            let helio = sub3(geo, sun_geo);
            r = add3(r, scale3(helio, 1.0 / SUN_MASS_RATIO[mi]));
        }
        Ok(r)
    }

    /// The body's sampling state, centered for element formation, in the true
    /// ecliptic of date at `jd`. Returns (center_pos, velocity, recentering
    /// vector to go point→geocentric, negate_output).
    ///
    /// All backend sampling and centering arithmetic (helio = geo − sun, the
    /// Sun→EMB remap, the SSB offset R) happens in the RAW J2000 frame, and
    /// velocity is the central difference of those J2000-centered positions.
    /// Differencing already-of-date snapshots instead (three different
    /// rotating frames at t−dt/t/t+dt) would smuggle in a spurious
    /// frame-rotation term ≈ precession-rate × r that swamps the real
    /// eccentricity signal for slow, large-`a` bodies (Neptune: precession
    /// rate ~6.7e-7 rad/day × r≈30 AU vs. e≈0.006 — exactly the SE parity
    /// bug this fixes). This matches SE's own approach (sweph.c
    /// `swi_plan_for_osc_elem`: acquire position+velocity in J2000 with
    /// `SEFLG_J2000|SEFLG_NONUT|SEFLG_TRUEPOS|SEFLG_SPEED`, "speed vector has
    /// to be rotated, but daily precession ... may not be added").
    ///
    /// The center-epoch position, velocity, and recentering vector are
    /// rotated ONCE, J2000 → true ecliptic of date at `jd`, after all
    /// sampling/centering/differencing is done — so `elements_from_state`
    /// forms the ellipse (and hence the nodes) directly in the of-date
    /// plane, rather than rotating already-formed points afterward (which
    /// would misplace low-inclination nodes by (plane tilt)/sin(i)).
    #[allow(clippy::type_complexity)]
    fn osculating_state(
        &self,
        body: &CelestialBody,
        jd: f64,
        method: NodApsMethod,
    ) -> Result<([f64; 3], [f64; 3], [f64; 3], bool), EventError> {
        // Per-body sample step (SE: NODE_CALC_INTV for the Moon, 0.001·r for
        // the rest — swecl.c:5256/5264).
        const NODE_CALC_INTV_DAYS: f64 = 1.0e-4;
        let label = "nod-aps body";
        if *body == CelestialBody::Moon {
            let dt = NODE_CALC_INTV_DAYS;
            let s = |jd: f64| self.body_geo_j2000(&CelestialBody::Moon, "Moon", jd);
            let (p0, pm, pp) = (s(jd)?, s(jd - dt)?, s(jd + dt)?);
            let vel = scale3(sub3(pp, pm), 1.0 / (2.0 * dt));
            // Geocentric orbit: points come out geocentric already.
            let pos = rotate_j2000_to_true_of_date(p0, jd)?;
            let vel = rotate_j2000_to_true_of_date(vel, jd)?;
            return Ok((pos, vel, [0.0; 3], false));
        }
        // Everything else forms a Sun-centered (or SSB-centered) ellipse, all
        // in the raw J2000 frame.
        let sun = |jd: f64| self.body_geo_j2000(&CelestialBody::Sun, "Sun", jd);
        let helio_at = |jd: f64| -> Result<[f64; 3], EventError> {
            if *body == CelestialBody::Sun {
                // SE remaps the Sun to the Earth-Moon barycenter
                // (swecl.c:5116-5117, 5281-5286): EMB = Earth + Moon/(μ_ratio+1),
                // with Earth_helio = −Sun_geo.
                let sun_geo = sun(jd)?;
                let moon_geo = self.body_geo_j2000(&CelestialBody::Moon, "Moon", jd)?;
                Ok(add3(
                    scale3(sun_geo, -1.0),
                    scale3(moon_geo, 1.0 / (EARTH_MOON_MASS_RATIO + 1.0)),
                ))
            } else {
                let geo = self.body_geo_j2000(body, label, jd)?;
                Ok(sub3(geo, sun(jd)?))
            }
        };
        let h0 = helio_at(jd)?;
        let r_helio = norm3(h0);
        let dt = NODE_CALC_INTV_DAYS * 10.0 * r_helio;
        let use_bary = method == NodApsMethod::OsculatingBarycentric && r_helio > 6.0;
        let recenter_at = |jd: f64| -> Result<[f64; 3], EventError> {
            if use_bary {
                // point_geo = point_bary + Sun_geo + R.
                Ok(add3(sun(jd)?, self.sun_to_ssb_offset(jd, sun(jd)?)?))
            } else {
                sun(jd)
            }
        };
        let centered_at = |jd: f64| -> Result<[f64; 3], EventError> {
            let h = helio_at(jd)?;
            if use_bary {
                let r = self.sun_to_ssb_offset(jd, sun(jd)?)?;
                Ok(sub3(h, r))
            } else {
                Ok(h)
            }
        };
        let (p0, pm, pp) = (
            centered_at(jd)?,
            centered_at(jd - dt)?,
            centered_at(jd + dt)?,
        );
        let vel = scale3(sub3(pp, pm), 1.0 / (2.0 * dt));
        // Sun included: SE's shared output stage always recenters the Sun's
        // point too (see `mean_points_at`'s comment) before negating it, so
        // the Sun gets the same true geocentric-Sun recenter vector as every
        // other body — never the zero vector.
        let recenter = recenter_at(jd)?;
        let pos = rotate_j2000_to_true_of_date(p0, jd)?;
        let vel = rotate_j2000_to_true_of_date(vel, jd)?;
        let recenter = rotate_j2000_to_true_of_date(recenter, jd)?;
        Ok((pos, vel, recenter, *body == CelestialBody::Sun))
    }

    /// Osculating points: form the instantaneous ellipse in the true ecliptic
    /// of date, take its four points, recenter geocentric, aberrate
    /// (except the Moon).
    fn osculating_points_at(
        &self,
        body: &CelestialBody,
        jd: f64,
        method: NodApsMethod,
        convention: ApsisConvention,
    ) -> Result<[RawPoint; 4], EventError> {
        let second_focus = convention == ApsisConvention::SecondFocus;
        let (pos, vel, recenter, negate) = self.osculating_state(body, jd, method)?;
        let mu = mu_au3_day2(body);
        let elements = pleiades_apsides::elements_from_state(pos, vel, mu).map_err(|e| {
            EventError::DegenerateNodAps {
                detail: format!("{body} osculating state: {e:?}"),
            }
        })?;
        let pts = points_from_elements(&elements, second_focus).map_err(|e| {
            EventError::DegenerateNodAps {
                detail: format!("{body} osculating ellipse: {e:?}"),
            }
        })?;
        let in_frame = [pts.ascending, pts.descending, pts.perihelion, pts.aphelion];
        let mut out = [RawPoint {
            lon_deg: 0.0,
            lat_deg: 0.0,
            dist_au: 0.0,
        }; 4];
        if *body == CelestialBody::Moon {
            for (o, p) in out.iter_mut().zip(in_frame.iter()) {
                *o = apsis_to_raw(p);
            }
            return Ok(out);
        }
        let (_, v_obs) = self.sun_geo_mean_of_date(jd)?;
        for (o, p) in out.iter_mut().zip(in_frame.iter()) {
            let v = spherical_to_cartesian(p.longitude_deg, p.latitude_deg, p.distance_au);
            // SE's shared output stage (swecl.c ~5480) recenters to geocentric
            // FIRST (`v + recenter`), and only THEN, for the Sun exclusively,
            // negates the already-geocentric vector (swecl.c:5482-5484) — see
            // `mean_points_at`'s matching comment.
            let geo = if negate {
                scale3(add3(v, recenter), -1.0)
            } else {
                add3(v, recenter)
            };
            *o = cartesian_to_raw(aberrate(geo, v_obs));
        }
        Ok(out)
    }
    // Note: no `λ += Δψ` here — the osculating frame already carries Δψ (samples
    // were rotated to the TRUE ecliptic of date). The observer velocity from
    // `sun_geo_mean_of_date` is a mean-frame vector; the ≤17″ frame mismatch on
    // a v/c≈20″ correction is < 2 mas — irrelevant, don't add machinery.
}

/// Rotates a vector from the raw J2000 mean-ecliptic frame to the true
/// ecliptic of date at `jd`: precession (J2000 → mean equinox/ecliptic of
/// date) plus nutation in longitude (mean → true equinox of date), applied to
/// the vector's DIRECTION, with its magnitude preserved exactly. A rigid
/// rotation carries every vector's direction through the same map, so this is
/// valid for velocity and recentering vectors, not just positions — it is
/// exactly the "rotate the speed vector, but don't add precession/nutation
/// rates to it" step SE performs in `swi_plan_for_osc_elem`.
fn rotate_j2000_to_true_of_date(v: [f64; 3], jd: f64) -> Result<[f64; 3], EventError> {
    let r = norm3(v);
    if r == 0.0 {
        return Ok(v);
    }
    let raw = cartesian_to_raw(v);
    let precessed = precess_ecliptic_j2000_to_date(raw.lon_deg, raw.lat_deg, jd)
        .map_err(|e| EventError::Backend(format!("precession failed: {e}")))?;
    let dpsi_deg = nutation(jd)
        .map_err(|e| EventError::Backend(format!("nutation failed: {e}")))?
        .delta_psi_arcsec
        / 3600.0;
    let lon_deg = (precessed.longitude_deg + dpsi_deg).rem_euclid(360.0);
    Ok(spherical_to_cartesian(lon_deg, precessed.latitude_deg, r))
}
