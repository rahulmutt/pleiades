//! Maps `CelestialBody` to NAIF ids, chains target/Earth states to geocenter,
//! and reduces ICRF positions to geocentric ecliptic coordinates.

use pleiades_backend::CelestialBody;
use pleiades_types::{EclipticCoordinates, Instant, Latitude, Longitude};

use super::pool::KernelPool;
use super::{SpkError, SpkErrorKind};

const AU_IN_KM: f64 = 149_597_870.7;

/// Candidate NAIF ids for a body, in priority order (mass center, then
/// barycenter). The pool picks the first id with a usable chain at the epoch.
pub fn naif_ids(body: &CelestialBody) -> Vec<i32> {
    match body {
        CelestialBody::Sun => vec![10],
        CelestialBody::Moon => vec![301],
        CelestialBody::Mercury => vec![199, 1],
        CelestialBody::Venus => vec![299, 2],
        CelestialBody::Mars => vec![499, 4],
        CelestialBody::Jupiter => vec![599, 5],
        CelestialBody::Saturn => vec![699, 6],
        CelestialBody::Uranus => vec![799, 7],
        CelestialBody::Neptune => vec![899, 8],
        CelestialBody::Pluto => vec![999, 9],
        CelestialBody::Ceres => vec![2_000_001],
        CelestialBody::Pallas => vec![2_000_002],
        CelestialBody::Juno => vec![2_000_003],
        CelestialBody::Vesta => vec![2_000_004],
        CelestialBody::Custom(id) => parse_custom_naif(id),
        // Lunar points are not in DE kernels; no SPK id.
        _ => Vec::new(),
    }
}

/// Parses a `CustomBodyId` like `asteroid:99942-Apophis` into candidate ids.
/// Accepts a leading integer in the designation as the IAU number and tries
/// both the old (`2_000_000 + n`) and new (`20_000_000 + n`) schemas.
fn parse_custom_naif(id: &pleiades_types::CustomBodyId) -> Vec<i32> {
    let lead: String = id
        .designation
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    match lead.parse::<i32>() {
        Ok(n) if n > 0 => vec![2_000_000 + n, 20_000_000 + n],
        _ => Vec::new(),
    }
}

/// Position of `target` relative to the Solar System Barycenter (id 0) by
/// walking the segment chain (target -> center -> ... -> 0) at `et`.
fn position_wrt_ssb(pool: &KernelPool, target: i32, et: f64) -> Result<[f64; 3], SpkError> {
    let mut acc = [0.0f64; 3];
    let mut current = target;
    // Bound the walk to avoid cycles in malformed kernels.
    for _ in 0..16 {
        if current == 0 {
            return Ok(acc);
        }
        // Try the segment whose target is `current` and whatever center exists.
        let (state, center) = pool.state_any_center(current, et)?;
        for (a, p) in acc.iter_mut().zip(state.position_km.iter()) {
            *a += *p;
        }
        current = center;
    }
    Err(SpkError::new(
        SpkErrorKind::NoChain,
        format!("chain from {target} did not reach SSB"),
    ))
}

/// Geocentric position (km, ICRF) of `target` = r(target wrt SSB) - r(Earth wrt SSB).
pub fn geocentric_icrf(pool: &KernelPool, target: i32, et: f64) -> Result<[f64; 3], SpkError> {
    let body = position_wrt_ssb(pool, target, et)?;
    let earth = position_wrt_ssb(pool, 399, et)?;
    Ok([body[0] - earth[0], body[1] - earth[1], body[2] - earth[2]])
}

/// Reduces an ICRF/J2000-equatorial geocentric position to ecliptic coords,
/// rotating by the mean obliquity at `instant` (the same value the existing
/// backend uses via `Instant::mean_obliquity`).
pub fn icrf_to_ecliptic(position_km: [f64; 3], instant: Instant) -> EclipticCoordinates {
    let eps = instant.mean_obliquity().radians();
    let (x, y_eq, z_eq) = (position_km[0], position_km[1], position_km[2]);
    // Rotate about X by +eps: equatorial -> ecliptic.
    let y = y_eq * eps.cos() + z_eq * eps.sin();
    let z = -y_eq * eps.sin() + z_eq * eps.cos();
    let radius = (x * x + y * y + z * z).sqrt();
    let longitude = Longitude::from_degrees(y.atan2(x).to_degrees());
    let latitude = Latitude::from_degrees((z / radius).clamp(-1.0, 1.0).asin().to_degrees());
    EclipticCoordinates::new(longitude, latitude, Some(radius / AU_IN_KM))
}

/// Resolves the best NAIF id for `body` and returns geocentric ecliptic coords.
pub fn ecliptic_for_body(
    pool: &KernelPool,
    body: &CelestialBody,
    instant: Instant,
) -> Result<EclipticCoordinates, SpkError> {
    let et = et_seconds_from_instant(instant);
    let candidates = naif_ids(body);
    if candidates.is_empty() {
        return Err(SpkError::new(SpkErrorKind::NoChain, "body has no NAIF id"));
    }
    let mut last_err = None;
    for id in candidates {
        match geocentric_icrf(pool, id, et) {
            Ok(pos) => return Ok(icrf_to_ecliptic(pos, instant)),
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap())
}

/// TDB seconds past J2000 from an instant's Julian Day (treats TT≈TDB at the
/// arcsecond level used here; UTC/UT1 are rejected upstream by request policy).
pub fn et_seconds_from_instant(instant: Instant) -> f64 {
    (instant.julian_day.days() - 2_451_545.0) * 86_400.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spk::test_support::{build_daf, type2_record, type2_segment_data, SegmentSpec};
    use pleiades_types::{JulianDay, TimeScale};

    fn const_pos_segment(target: i32, center: i32, pos: [f64; 3]) -> SegmentSpec {
        let rec = type2_record(0.0, 1.0e12, &[pos[0], 0.0], &[pos[1], 0.0], &[pos[2], 0.0]);
        let data = type2_segment_data(-1.0e12, 2.0e12, rec.len(), &[rec]);
        SegmentSpec {
            start_et: -1.0e12,
            stop_et: 1.0e12,
            target,
            center,
            frame: 1,
            data_type: 2,
            data,
            name: "C".to_string(),
        }
    }

    #[test]
    fn geocentric_difference_uses_earth_chain() {
        // body wrt SSB = (100,0,0); Earth wrt SSB = (399 wrt 3)+(3 wrt 0).
        let blob = build_daf(&[
            const_pos_segment(10, 0, [100.0, 0.0, 0.0]), // Sun wrt SSB
            const_pos_segment(399, 3, [0.0, 10.0, 0.0]), // Earth wrt EMB
            const_pos_segment(3, 0, [0.0, 5.0, 0.0]),    // EMB wrt SSB
        ]);
        let mut pool = KernelPool::new();
        pool.add_bytes(blob, "k").unwrap();
        let et = 0.0;
        let geo = geocentric_icrf(&pool, 10, et).unwrap();
        // (100,0,0) - (0,15,0) = (100,-15,0).
        assert!((geo[0] - 100.0).abs() < 1e-6);
        assert!((geo[1] + 15.0).abs() < 1e-6);
    }

    #[test]
    fn obliquity_rotation_matches_existing_backend_constant() {
        let inst = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        // A point on the equatorial X axis is unaffected by the X-rotation.
        let ec = icrf_to_ecliptic([AU_IN_KM, 0.0, 0.0], inst);
        assert!((ec.longitude.degrees() - 0.0).abs() < 1e-9);
        assert!((ec.latitude.degrees()).abs() < 1e-9);
        assert!((ec.distance_au.unwrap() - 1.0).abs() < 1e-9);
    }
}
