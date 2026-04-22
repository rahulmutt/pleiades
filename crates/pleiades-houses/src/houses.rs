//! House-system calculations for the baseline chart MVP.
//!
//! The catalog layer in this crate already enumerates the target compatibility
//! set. This module now implements the first practical house-placement
//! workflows for the baseline systems so the chart layer can offer real house
//! cusps instead of catalog-only placeholders.
//!
//! Equal, Whole Sign, and Porphyry remain the simplest space/ecliptic systems.
//! Placidus, Koch, Alcabitius, and Topocentric use iterative or time-divisional
//! formulas. Regiomontanus, Campanus, Morinus, Meridian, and Axial variants
//! are projected from their equatorial or prime-vertical constructions.
//!
//! The formulas are intentionally explicit and documented so later validation
//! work can tighten them further without changing the public API surface.

use core::fmt;

use pleiades_types::{Angle, HouseSystem, Instant, Longitude, ObserverLocation};

/// A request for house calculation.
#[derive(Clone, Debug, PartialEq)]
pub struct HouseRequest {
    /// The instant being charted.
    pub instant: Instant,
    /// The observer location used to derive horizon-dependent angles.
    pub observer: ObserverLocation,
    /// The selected house system.
    pub system: HouseSystem,
    /// Optional obliquity override in degrees.
    pub obliquity: Option<Angle>,
}

impl HouseRequest {
    /// Creates a new house calculation request.
    pub fn new(instant: Instant, observer: ObserverLocation, system: HouseSystem) -> Self {
        Self {
            instant,
            observer,
            system,
            obliquity: None,
        }
    }

    /// Overrides the obliquity used for angle derivation.
    pub fn with_obliquity(mut self, obliquity: Angle) -> Self {
        self.obliquity = Some(obliquity);
        self
    }
}

/// Derived chart angles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HouseAngles {
    /// Ascendant.
    pub ascendant: Longitude,
    /// Descendant.
    pub descendant: Longitude,
    /// Midheaven.
    pub midheaven: Longitude,
    /// Imum Coeli.
    pub imum_coeli: Longitude,
}

impl HouseAngles {
    /// Creates the four angle points from ascendant and midheaven.
    pub fn new(ascendant: Longitude, midheaven: Longitude) -> Self {
        Self {
            ascendant,
            descendant: longitude_opposite(ascendant),
            midheaven,
            imum_coeli: longitude_opposite(midheaven),
        }
    }
}

/// A complete house cusp set.
#[derive(Clone, Debug, PartialEq)]
pub struct HouseSnapshot {
    /// House system used for the calculation.
    pub system: HouseSystem,
    /// Instant used for the calculation.
    pub instant: Instant,
    /// Observer location used for the calculation.
    pub observer: ObserverLocation,
    /// Obliquity used to derive the angles.
    pub obliquity: Angle,
    /// Derived angles.
    pub angles: HouseAngles,
    /// House cusps in house-number order.
    pub cusps: [Longitude; 12],
}

impl HouseSnapshot {
    /// Returns the cusp for a given one-based house number.
    pub const fn cusp(&self, house: usize) -> Option<Longitude> {
        if house == 0 || house > 12 {
            None
        } else {
            Some(self.cusps[house - 1])
        }
    }
}

/// Error categories for house calculations.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum HouseErrorKind {
    /// The selected house system is catalogued but not yet implemented.
    UnsupportedHouseSystem,
    /// The observer latitude is outside the mathematically valid range.
    InvalidLatitude,
    /// The calculation failed for a numerical reason.
    NumericalFailure,
}

/// A structured house-calculation error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HouseError {
    /// Error category.
    pub kind: HouseErrorKind,
    /// Human-readable message.
    pub message: String,
}

impl HouseError {
    /// Creates a new structured house error.
    pub fn new(kind: HouseErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for HouseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for HouseError {}

/// Computes the house cusps and derived angles for a request.
pub fn calculate_houses(request: &HouseRequest) -> Result<HouseSnapshot, HouseError> {
    validate_observer(&request.observer)?;

    let obliquity = request
        .obliquity
        .unwrap_or_else(|| mean_obliquity(request.instant));
    let angles = derive_angles(request.instant, &request.observer, obliquity);
    let cusps = match &request.system {
        HouseSystem::Equal => equal_houses(angles.ascendant),
        HouseSystem::WholeSign => whole_sign_houses(angles.ascendant),
        HouseSystem::Porphyry => porphyry_houses(angles),
        HouseSystem::Placidus => {
            placidus_houses(request.instant, &request.observer, obliquity, angles)?
        }
        HouseSystem::Koch => koch_houses(request.instant, &request.observer, obliquity, angles)?,
        HouseSystem::Regiomontanus => {
            regiomontanus_houses(request.instant, &request.observer, obliquity, angles)
        }
        HouseSystem::Campanus => {
            campanus_houses(request.instant, &request.observer, obliquity, angles)
        }
        HouseSystem::Alcabitius => {
            alcabitius_houses(request.instant, &request.observer, obliquity, angles)
        }
        HouseSystem::Meridian | HouseSystem::Axial | HouseSystem::Morinus => {
            equatorial_projection_houses(request.instant, &request.observer, obliquity)
        }
        HouseSystem::Topocentric => {
            topocentric_houses(request.instant, &request.observer, obliquity)?
        }
        _ => {
            return Err(HouseError::new(
                HouseErrorKind::UnsupportedHouseSystem,
                format!(
                    "house placement for {} is not implemented yet",
                    catalog_name(&request.system)
                ),
            ))
        }
    };

    Ok(HouseSnapshot {
        system: request.system.clone(),
        instant: request.instant,
        observer: request.observer.clone(),
        obliquity,
        angles,
        cusps,
    })
}

/// Returns the one-based house number for a longitude and cusp set.
///
/// Cusps are treated as the start of each house, and wraparound at 360° is
/// handled explicitly.
pub fn house_for_longitude(longitude: Longitude, cusps: &[Longitude; 12]) -> usize {
    let longitude = longitude.degrees().rem_euclid(360.0);
    for (index, cusp) in cusps.iter().enumerate() {
        let start = cusp.degrees();
        let end = cusps[(index + 1) % 12].degrees();
        if longitude_in_arc(longitude, start, end) {
            return index + 1;
        }
    }

    1
}

fn validate_observer(observer: &ObserverLocation) -> Result<(), HouseError> {
    let latitude = observer.latitude.degrees();
    if !latitude.is_finite() || latitude.abs() > 90.0 {
        return Err(HouseError::new(
            HouseErrorKind::InvalidLatitude,
            format!("observer latitude {latitude}° is outside the valid range"),
        ));
    }

    Ok(())
}

fn derive_angles(instant: Instant, observer: &ObserverLocation, obliquity: Angle) -> HouseAngles {
    let sidereal_time = local_sidereal_time(instant, observer.longitude);
    let obliquity = obliquity.normalized_signed().degrees().to_radians();
    let latitude = observer.latitude.degrees().to_radians();
    let theta = sidereal_time.degrees().to_radians();

    let ascendant = Longitude::from_degrees(
        (-theta.cos())
            .atan2(theta.sin() * obliquity.cos() + latitude.tan() * obliquity.sin())
            .to_degrees(),
    );
    let midheaven = Longitude::from_degrees(
        (theta.sin() * obliquity.cos())
            .atan2(theta.cos())
            .to_degrees(),
    );
    HouseAngles::new(ascendant, midheaven)
}

fn local_sidereal_time(instant: Instant, longitude: Longitude) -> Angle {
    let jd = instant.julian_day.days();
    let centuries = (jd - 2_451_545.0) / 36_525.0;
    let gmst = 280.460_618_37
        + 360.985_647_366_29 * (jd - 2_451_545.0)
        + 0.000_387_933 * centuries * centuries
        - centuries * centuries * centuries / 38_710_000.0;
    Angle::from_degrees(gmst + longitude.degrees()).normalized_0_360()
}

fn mean_obliquity(instant: Instant) -> Angle {
    let centuries = (instant.julian_day.days() - 2_451_545.0) / 36_525.0;
    Angle::from_degrees(
        23.439_291_111_111_11
            - 0.013_004_166_666_666_667 * centuries
            - 0.000_000_163_888_888_888_888_88 * centuries * centuries
            + 0.000_000_503_611_111_111_111_1 * centuries * centuries * centuries,
    )
}

fn equal_houses(ascendant: Longitude) -> [Longitude; 12] {
    core::array::from_fn(|index| {
        Longitude::from_degrees(ascendant.degrees() + (index as f64) * 30.0)
    })
}

fn whole_sign_houses(ascendant: Longitude) -> [Longitude; 12] {
    let first_cusp = Longitude::from_degrees((ascendant.degrees() / 30.0).floor() * 30.0);
    core::array::from_fn(|index| {
        Longitude::from_degrees(first_cusp.degrees() + (index as f64) * 30.0)
    })
}

fn porphyry_houses(angles: HouseAngles) -> [Longitude; 12] {
    let mut cusps = [Longitude::from_degrees(0.0); 12];
    cusps[0] = angles.ascendant;
    cusps[1] = interpolate_longitude(angles.ascendant, angles.imum_coeli, 1.0 / 3.0);
    cusps[2] = interpolate_longitude(angles.ascendant, angles.imum_coeli, 2.0 / 3.0);
    cusps[3] = angles.imum_coeli;
    cusps[4] = interpolate_longitude(angles.imum_coeli, angles.descendant, 1.0 / 3.0);
    cusps[5] = interpolate_longitude(angles.imum_coeli, angles.descendant, 2.0 / 3.0);
    cusps[6] = angles.descendant;
    cusps[7] = interpolate_longitude(angles.descendant, angles.midheaven, 1.0 / 3.0);
    cusps[8] = interpolate_longitude(angles.descendant, angles.midheaven, 2.0 / 3.0);
    cusps[9] = angles.midheaven;
    cusps[10] = interpolate_longitude(angles.midheaven, angles.ascendant, 1.0 / 3.0);
    cusps[11] = interpolate_longitude(angles.midheaven, angles.ascendant, 2.0 / 3.0);
    cusps
}

fn placidus_houses(
    instant: Instant,
    observer: &ObserverLocation,
    obliquity: Angle,
    angles: HouseAngles,
) -> Result<[Longitude; 12], HouseError> {
    let mut cusps = [Longitude::from_degrees(0.0); 12];
    cusps[0] = angles.ascendant;
    cusps[3] = angles.imum_coeli;
    cusps[6] = angles.descendant;
    cusps[9] = angles.midheaven;

    let st = local_sidereal_time(instant, observer.longitude).degrees();
    cusps[10] = solve_placidian_cusp(st, observer.latitude.degrees(), obliquity.degrees(), 11)?;
    cusps[11] = solve_placidian_cusp(st, observer.latitude.degrees(), obliquity.degrees(), 12)?;
    cusps[1] = solve_placidian_cusp(st, observer.latitude.degrees(), obliquity.degrees(), 2)?;
    cusps[2] = solve_placidian_cusp(st, observer.latitude.degrees(), obliquity.degrees(), 3)?;

    cusps[4] = longitude_opposite(cusps[10]);
    cusps[5] = longitude_opposite(cusps[11]);
    cusps[7] = longitude_opposite(cusps[1]);
    cusps[8] = longitude_opposite(cusps[2]);

    Ok(cusps)
}

fn koch_houses(
    instant: Instant,
    observer: &ObserverLocation,
    obliquity: Angle,
    angles: HouseAngles,
) -> Result<[Longitude; 12], HouseError> {
    let mut cusps = [Longitude::from_degrees(0.0); 12];
    cusps[0] = angles.ascendant;
    cusps[3] = angles.imum_coeli;
    cusps[6] = angles.descendant;
    cusps[9] = angles.midheaven;

    let st = local_sidereal_time(instant, observer.longitude).degrees();
    let latitude = observer.latitude.degrees().to_radians();
    let obliquity = obliquity.degrees().to_radians();
    let z = (st.to_radians().sin() * latitude.tan() * obliquity.tan())
        .clamp(-1.0, 1.0)
        .asin()
        .to_degrees();

    for house in 1..=12 {
        if matches!(house, 1 | 4 | 7 | 10) {
            continue;
        }

        let b = house_phase(house);
        let hemisphere_sign = if b < 180.0 { 1.0 } else { -1.0 };
        let k = if b < 180.0 { 1.0 } else { -1.0 };
        let h = st + b + hemisphere_sign * z;
        let x = h.to_radians().cos() * obliquity.cos() - k * latitude.tan() * obliquity.sin();
        let y = h.to_radians().sin();
        cusps[house - 1] = Longitude::from_degrees(y.atan2(x).to_degrees());
    }

    Ok(cusps)
}

fn alcabitius_houses(
    instant: Instant,
    observer: &ObserverLocation,
    obliquity: Angle,
    angles: HouseAngles,
) -> [Longitude; 12] {
    let mut cusps = [Longitude::from_degrees(0.0); 12];
    cusps[0] = angles.ascendant;
    cusps[3] = angles.imum_coeli;
    cusps[6] = angles.descendant;
    cusps[9] = angles.midheaven;

    let st = local_sidereal_time(instant, observer.longitude).degrees();
    let latitude = observer.latitude.degrees().to_radians();
    let obliquity = obliquity.degrees().to_radians();
    let ascendant_longitude = angles.ascendant.degrees().to_radians();
    let ascendant_declination = (ascendant_longitude.sin() * obliquity.sin()).asin();
    let ascensional_difference = (latitude.tan() * ascendant_declination.tan())
        .clamp(-1.0, 1.0)
        .asin()
        .to_degrees();
    let diurnal = 90.0 + ascensional_difference;
    let nocturnal = 90.0 - ascensional_difference;

    let above = [10usize, 11, 12];
    for (index, house) in above.iter().enumerate() {
        let offset = diurnal * (index as f64) / 3.0;
        let ra = st + offset;
        cusps[*house - 1] = ecliptic_longitude_from_ra(ra, obliquity);
    }

    let below = [1usize, 2, 3];
    for (index, house) in below.iter().enumerate() {
        let offset = diurnal + nocturnal * ((index as f64) + 1.0) / 3.0;
        let ra = st + offset;
        cusps[*house - 1] = ecliptic_longitude_from_ra(ra, obliquity);
    }

    cusps[4] = longitude_opposite(cusps[10]);
    cusps[5] = longitude_opposite(cusps[11]);
    cusps[7] = longitude_opposite(cusps[1]);
    cusps[8] = longitude_opposite(cusps[2]);

    cusps
}

fn regiomontanus_houses(
    instant: Instant,
    observer: &ObserverLocation,
    obliquity: Angle,
    angles: HouseAngles,
) -> [Longitude; 12] {
    let mut cusps = [Longitude::from_degrees(0.0); 12];
    cusps[0] = angles.ascendant;
    cusps[3] = angles.imum_coeli;
    cusps[6] = angles.descendant;
    cusps[9] = angles.midheaven;

    let st = local_sidereal_time(instant, observer.longitude)
        .degrees()
        .to_radians();
    let latitude = observer.latitude.degrees().to_radians();
    let obliquity = obliquity.degrees().to_radians();

    for house in 1..=12 {
        if matches!(house, 1 | 4 | 7 | 10) {
            continue;
        }

        let d = house_phase(house).to_radians();
        let v = d.sin() * latitude.sin() * obliquity.sin();
        let x = (st + d).cos() * latitude.cos() * obliquity.cos() - v;
        let y = (st + d).sin() * latitude.cos();
        cusps[house - 1] = Longitude::from_degrees(y.atan2(x).to_degrees());
    }

    cusps
}

fn campanus_houses(
    instant: Instant,
    observer: &ObserverLocation,
    obliquity: Angle,
    angles: HouseAngles,
) -> [Longitude; 12] {
    let mut cusps = [Longitude::from_degrees(0.0); 12];
    cusps[0] = angles.ascendant;
    cusps[3] = angles.imum_coeli;
    cusps[6] = angles.descendant;
    cusps[9] = angles.midheaven;

    let st = local_sidereal_time(instant, observer.longitude)
        .degrees()
        .to_radians();
    let latitude = observer.latitude.degrees().to_radians();
    let obliquity = obliquity.degrees().to_radians();

    for house in 1..=12 {
        if matches!(house, 1 | 4 | 7 | 10) {
            continue;
        }

        let z = house_phase(house).to_radians();
        let p = (z.sin() * latitude.cos()).atan2(z.cos());
        let v = p.sin() * latitude.sin() * obliquity.sin();
        let x = (st + z).cos() * latitude.cos() * obliquity.cos() - v;
        let y = (st + z).sin();
        cusps[house - 1] = Longitude::from_degrees(y.atan2(x).to_degrees());
    }

    cusps
}

fn equatorial_projection_houses(
    instant: Instant,
    observer: &ObserverLocation,
    obliquity: Angle,
) -> [Longitude; 12] {
    let st = local_sidereal_time(instant, observer.longitude).degrees();

    core::array::from_fn(|index| {
        let house = index + 1;
        let ra = st + house_phase(house);
        ecliptic_longitude_from_ra(ra, obliquity.degrees().to_radians())
    })
}

fn topocentric_houses(
    instant: Instant,
    observer: &ObserverLocation,
    obliquity: Angle,
) -> Result<[Longitude; 12], HouseError> {
    let corrected_latitude =
        topocentric_latitude(observer.latitude.degrees(), observer.elevation_m)?;
    let corrected_observer = ObserverLocation::new(
        corrected_latitude.into(),
        observer.longitude,
        observer.elevation_m,
    );
    let corrected_angles = derive_angles(instant, &corrected_observer, obliquity);
    placidus_houses(instant, &corrected_observer, obliquity, corrected_angles)
}

fn topocentric_latitude(latitude_deg: f64, elevation_m: Option<f64>) -> Result<Angle, HouseError> {
    let latitude = latitude_deg.to_radians();
    let radius_m = 6_371_000.0;
    let scale = match elevation_m {
        Some(elevation) if elevation.is_finite() => radius_m / (radius_m + elevation),
        Some(_) => {
            return Err(HouseError::new(
                HouseErrorKind::InvalidLatitude,
                "observer elevation must be finite when provided",
            ))
        }
        None => 1.0,
    };

    let sin_effective = (scale * latitude.sin()).clamp(-1.0, 1.0);
    Ok(Angle::from_degrees(sin_effective.asin().to_degrees()))
}

fn solve_placidian_cusp(
    st_deg: f64,
    latitude_deg: f64,
    obliquity_deg: f64,
    house: usize,
) -> Result<Longitude, HouseError> {
    let k = match house {
        11 => 1.0 / 3.0,
        12 => 2.0 / 3.0,
        2 => -2.0 / 3.0,
        3 => -1.0 / 3.0,
        _ => {
            return Err(HouseError::new(
                HouseErrorKind::UnsupportedHouseSystem,
                format!("invalid placidian house {}", house),
            ))
        }
    };

    let latitude = latitude_deg.to_radians();
    let obliquity = obliquity_deg.to_radians();
    let c = latitude.cos();
    let s = latitude.sin() * obliquity.tan();
    let mut q = 90.0;

    for _ in 0..32 {
        let ra = st_deg + q;
        let q_rad = q.to_radians();
        let ra_rad = ra.to_radians();
        let f = c * q_rad.cos() + k * s * ra_rad.sin();
        let fp = (-c * q_rad.sin() + k * s * ra_rad.cos()) * core::f64::consts::PI / 180.0;
        if fp.abs() < 1.0e-12 {
            return Err(HouseError::new(
                HouseErrorKind::NumericalFailure,
                "placidian cusp iteration encountered a zero derivative",
            ));
        }

        let delta = -f / fp;
        q += delta;
        if delta.abs() < 1.0e-8 {
            break;
        }
    }

    let ra = st_deg + q;
    let lon = ecliptic_longitude_from_ra(ra, obliquity);
    Ok(match house {
        11 | 12 => lon,
        2 | 3 => longitude_opposite(lon),
        _ => unreachable!(),
    })
}

fn ecliptic_longitude_from_ra(ra_deg: f64, obliquity: f64) -> Longitude {
    let ra = ra_deg.to_radians();
    Longitude::from_degrees(ra.sin().atan2(ra.cos() * obliquity.cos()).to_degrees())
}

fn interpolate_longitude(start: Longitude, end: Longitude, fraction: f64) -> Longitude {
    let span = (end.degrees() - start.degrees()).rem_euclid(360.0);
    Longitude::from_degrees(start.degrees() + span * fraction)
}

fn longitude_opposite(longitude: Longitude) -> Longitude {
    Longitude::from_degrees(longitude.degrees() + 180.0)
}

fn house_phase(house: usize) -> f64 {
    ((house + 2) % 12) as f64 * 30.0
}

fn longitude_in_arc(longitude: f64, start: f64, end: f64) -> bool {
    if start <= end {
        longitude >= start && longitude < end
    } else {
        longitude >= start || longitude < end
    }
}

fn catalog_name(system: &HouseSystem) -> &'static str {
    match system {
        HouseSystem::Placidus => "Placidus",
        HouseSystem::Koch => "Koch",
        HouseSystem::Porphyry => "Porphyry",
        HouseSystem::Regiomontanus => "Regiomontanus",
        HouseSystem::Campanus => "Campanus",
        HouseSystem::Equal => "Equal",
        HouseSystem::WholeSign => "Whole Sign",
        HouseSystem::Alcabitius => "Alcabitius",
        HouseSystem::Meridian => "Meridian",
        HouseSystem::Axial => "Axial",
        HouseSystem::Topocentric => "Topocentric",
        HouseSystem::Morinus => "Morinus",
        HouseSystem::Custom(_) => "Custom",
        _ => "Unspecified",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::Latitude;

    fn observer() -> ObserverLocation {
        ObserverLocation::new(
            Latitude::from_degrees(0.0),
            Longitude::from_degrees(0.0),
            None,
        )
    }

    fn sample_request(system: HouseSystem) -> HouseRequest {
        HouseRequest::new(
            Instant::new(
                pleiades_types::JulianDay::from_days(2_451_545.0),
                pleiades_types::TimeScale::Tt,
            ),
            observer(),
            system,
        )
    }

    #[test]
    fn equal_houses_step_in_thirty_degree_increments() {
        let snapshot = calculate_houses(&sample_request(HouseSystem::Equal))
            .expect("equal houses should work");
        assert_eq!(snapshot.cusps.len(), 12);
        assert_eq!(
            snapshot.cusps[0].degrees(),
            snapshot.angles.ascendant.degrees()
        );
        assert_eq!(
            (snapshot.cusps[1].degrees() - snapshot.cusps[0].degrees()).rem_euclid(360.0),
            30.0
        );
        assert_eq!(
            (snapshot.cusps[3].degrees() - snapshot.cusps[0].degrees()).rem_euclid(360.0),
            90.0
        );
    }

    #[test]
    fn whole_sign_houses_start_at_the_rising_sign_boundary() {
        let snapshot = calculate_houses(&sample_request(HouseSystem::WholeSign))
            .expect("whole sign houses should work");
        assert_eq!(snapshot.cusps[0].degrees() % 30.0, 0.0);
        assert!(snapshot.cusps[0].degrees() <= snapshot.angles.ascendant.degrees());
        assert_eq!(
            (snapshot.cusps[1].degrees() - snapshot.cusps[0].degrees()).rem_euclid(360.0),
            30.0
        );
    }

    #[test]
    fn porphyry_divides_quadrants_evenly() {
        let snapshot = calculate_houses(&sample_request(HouseSystem::Porphyry))
            .expect("porphyry houses should work");
        assert_eq!(snapshot.cusps[0], snapshot.angles.ascendant);
        assert_eq!(snapshot.cusps[3], snapshot.angles.imum_coeli);
        assert_eq!(snapshot.cusps[6], snapshot.angles.descendant);
        assert_eq!(snapshot.cusps[9], snapshot.angles.midheaven);
    }

    #[test]
    fn baseline_quadrant_systems_are_implemented() {
        for system in [
            HouseSystem::Placidus,
            HouseSystem::Koch,
            HouseSystem::Regiomontanus,
            HouseSystem::Campanus,
            HouseSystem::Alcabitius,
            HouseSystem::Meridian,
            HouseSystem::Axial,
            HouseSystem::Morinus,
            HouseSystem::Topocentric,
        ] {
            let snapshot = calculate_houses(&sample_request(system.clone()))
                .expect("baseline quadrant system should calculate");
            assert_eq!(snapshot.cusps.len(), 12);
        }
    }

    #[test]
    fn house_assignment_respects_wraparound() {
        let cusps = [
            Longitude::from_degrees(330.0),
            Longitude::from_degrees(0.0),
            Longitude::from_degrees(30.0),
            Longitude::from_degrees(60.0),
            Longitude::from_degrees(90.0),
            Longitude::from_degrees(120.0),
            Longitude::from_degrees(150.0),
            Longitude::from_degrees(180.0),
            Longitude::from_degrees(210.0),
            Longitude::from_degrees(240.0),
            Longitude::from_degrees(270.0),
            Longitude::from_degrees(300.0),
        ];

        assert_eq!(
            house_for_longitude(Longitude::from_degrees(359.0), &cusps),
            1
        );
        assert_eq!(
            house_for_longitude(Longitude::from_degrees(15.0), &cusps),
            2
        );
    }
}
