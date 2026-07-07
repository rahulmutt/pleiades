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
}

use crate::crossings::EventEngine;
use crate::ephemeris::{read_mean_ecliptic, read_mean_longitude, spherical_to_cartesian};
use crate::mean_elements::{
    elem_index, mean_elements_of_date, MOON_MEAN_ECC, MOON_MEAN_INCL_DEG, MOON_MEAN_SEMA_AU,
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
            let node = read_mean_longitude(&self.backend, CelestialBody::MeanNode, "MeanNode", jd)?;
            let peri =
                read_mean_longitude(&self.backend, CelestialBody::MeanPerigee, "MeanPerigee", jd)?;
            // Backend lunar-point longitudes are J2000-frame at the boundary;
            // bring them to the mean equinox of date like any body read.
            let node = precess_ecliptic_j2000_to_date(node, 0.0, jd)
                .map_err(|e| EventError::Backend(format!("precession failed: {e}")))?
                .longitude_deg;
            let peri = precess_ecliptic_j2000_to_date(peri, 0.0, jd)
                .map_err(|e| EventError::Backend(format!("precession failed: {e}")))?
                .longitude_deg;
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
            // Sun: SE negates the heliocentric Earth-orbit point to produce
            // the geocentric "Black Sun" point (swecl.c:5482-5484). Planets:
            // heliocentric point + geocentric Sun = geocentric point.
            let geo = if *body == CelestialBody::Sun {
                scale3(helio, -1.0)
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

    /// Placeholder until the osculating task lands.
    fn osculating_points_at(
        &self,
        body: &CelestialBody,
        _jd: f64,
        _method: NodApsMethod,
        _convention: ApsisConvention,
    ) -> Result<[RawPoint; 4], EventError> {
        Err(EventError::UnsupportedNodAps {
            detail: format!("osculating nod_aps for {body} not yet implemented"),
        })
    }
}
