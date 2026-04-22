//! House-system calculations for the baseline chart MVP.
//!
//! The catalog layer in this crate already enumerates the target compatibility
//! set. This module adds the first concrete house-placement workflow on top of
//! that vocabulary. The initial implementation focuses on the simpler and more
//! robust systems first: Equal, Whole Sign, and Porphyry.
//!
//! More latitude-sensitive or time-divisional systems are still catalogued,
//! but remain explicit unsupported cases for this slice so callers get a clear
//! error instead of a silent approximation.

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
    let cusps = match request.system {
        HouseSystem::Equal => equal_houses(angles.ascendant),
        HouseSystem::WholeSign => whole_sign_houses(angles.ascendant),
        HouseSystem::Porphyry => porphyry_houses(angles),
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

fn interpolate_longitude(start: Longitude, end: Longitude, fraction: f64) -> Longitude {
    let span = (end.degrees() - start.degrees()).rem_euclid(360.0);
    Longitude::from_degrees(start.degrees() + span * fraction)
}

fn longitude_opposite(longitude: Longitude) -> Longitude {
    Longitude::from_degrees(longitude.degrees() + 180.0)
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

    #[test]
    fn equal_houses_step_in_thirty_degree_increments() {
        let request = HouseRequest::new(
            Instant::new(
                pleiades_types::JulianDay::from_days(2_451_545.0),
                pleiades_types::TimeScale::Tt,
            ),
            observer(),
            HouseSystem::Equal,
        );

        let snapshot = calculate_houses(&request).expect("equal houses should work");
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
        let request = HouseRequest::new(
            Instant::new(
                pleiades_types::JulianDay::from_days(2_451_545.0),
                pleiades_types::TimeScale::Tt,
            ),
            observer(),
            HouseSystem::WholeSign,
        );

        let snapshot = calculate_houses(&request).expect("whole sign houses should work");
        assert_eq!(snapshot.cusps[0].degrees() % 30.0, 0.0);
        assert!(snapshot.cusps[0].degrees() <= snapshot.angles.ascendant.degrees());
        assert_eq!(
            (snapshot.cusps[1].degrees() - snapshot.cusps[0].degrees()).rem_euclid(360.0),
            30.0
        );
    }

    #[test]
    fn porphyry_divides_quadrants_evenly() {
        let request = HouseRequest::new(
            Instant::new(
                pleiades_types::JulianDay::from_days(2_451_545.0),
                pleiades_types::TimeScale::Tt,
            ),
            observer(),
            HouseSystem::Porphyry,
        );

        let snapshot = calculate_houses(&request).expect("porphyry houses should work");
        assert_eq!(snapshot.cusps[0], snapshot.angles.ascendant);
        assert_eq!(snapshot.cusps[3], snapshot.angles.imum_coeli);
        assert_eq!(snapshot.cusps[6], snapshot.angles.descendant);
        assert_eq!(snapshot.cusps[9], snapshot.angles.midheaven);
    }

    #[test]
    fn unsupported_house_systems_are_reported_explicitly() {
        let request = HouseRequest::new(
            Instant::new(
                pleiades_types::JulianDay::from_days(2_451_545.0),
                pleiades_types::TimeScale::Tt,
            ),
            observer(),
            HouseSystem::Placidus,
        );

        let error =
            calculate_houses(&request).expect_err("placidus should be unsupported in this slice");
        assert_eq!(error.kind, HouseErrorKind::UnsupportedHouseSystem);
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
