//! House-system calculations for the baseline chart workflow.
//!
//! The catalog layer in this crate already enumerates the target compatibility
//! set. This module implements the practical house-placement workflows for the
//! baseline systems so the chart layer can offer real house cusps instead of
//! catalog-only stubs.
//!
//! Equal, Whole Sign, and Porphyry remain the simplest space/ecliptic systems.
//! Placidus, Koch, Alcabitius, and Topocentric use iterative or time-divisional
//! formulas. Regiomontanus, Campanus, Carter, Morinus, Meridian, and Axial
//! variants are projected from their equatorial or prime-vertical
//! constructions.
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

    /// Returns a compact one-line rendering of the house request.
    pub fn summary_line(&self) -> String {
        let obliquity = self
            .obliquity
            .map(|value| value.to_string())
            .unwrap_or_else(|| "auto".to_string());

        let system = self.system.to_string();

        format!(
            "instant={}; observer={}; system={}; obliquity={}",
            self.instant, self.observer, system, obliquity
        )
    }
}

impl fmt::Display for HouseRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
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
    ///
    /// Most systems expose 12 cusps, while Gauquelin sectors expose 36.
    pub cusps: Vec<Longitude>,
}

impl HouseSnapshot {
    /// Returns the cusp for a given one-based house number.
    pub fn cusp(&self, house: usize) -> Option<Longitude> {
        if house == 0 {
            None
        } else {
            self.cusps.get(house - 1).copied()
        }
    }

    /// Returns the one-based house number for a longitude using this snapshot's cusps.
    pub fn house_for_longitude(&self, longitude: Longitude) -> usize {
        house_for_longitude(longitude, &self.cusps)
    }

    /// Ensures the snapshot contains only finite numeric values.
    pub fn validate(&self) -> Result<(), HouseError> {
        validate_house_snapshot(self)
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
    /// The observer elevation was not finite when a topocentric correction was requested.
    InvalidElevation,
    /// The supplied obliquity override was not finite.
    InvalidObliquity,
    /// The calculation failed for a numerical reason.
    NumericalFailure,
}

impl fmt::Display for HouseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::UnsupportedHouseSystem => "UnsupportedHouseSystem",
            Self::InvalidLatitude => "InvalidLatitude",
            Self::InvalidElevation => "InvalidElevation",
            Self::InvalidObliquity => "InvalidObliquity",
            Self::NumericalFailure => "NumericalFailure",
        };
        f.write_str(label)
    }
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
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for HouseError {}

/// Computes the house cusps and derived angles for a request.
pub fn calculate_houses(request: &HouseRequest) -> Result<HouseSnapshot, HouseError> {
    validate_observer(&request.observer)?;

    let obliquity = validate_obliquity(
        request
            .obliquity
            .unwrap_or_else(|| mean_obliquity(request.instant)),
    )?;
    let angles = derive_angles(request.instant, &request.observer, obliquity);
    let cusps = match &request.system {
        HouseSystem::Equal => equal_houses(angles.ascendant).into(),
        HouseSystem::EqualMidheaven => equal_midheaven_houses(angles.midheaven).into(),
        HouseSystem::EqualAries => equal_aries_houses().into(),
        HouseSystem::Vehlow => vehlow_equal_houses(angles.ascendant).into(),
        HouseSystem::Sripati => sripati_houses(angles).into(),
        HouseSystem::WholeSign => whole_sign_houses(angles.ascendant).into(),
        HouseSystem::Porphyry => porphyry_houses(angles).into(),
        HouseSystem::Placidus => {
            placidus_houses(request.instant, &request.observer, obliquity, angles)?.into()
        }
        HouseSystem::Koch => {
            koch_houses(request.instant, &request.observer, obliquity, angles)?.into()
        }
        HouseSystem::Regiomontanus => {
            regiomontanus_houses(request.instant, &request.observer, obliquity, angles).into()
        }
        HouseSystem::Campanus => {
            campanus_houses(request.instant, &request.observer, obliquity, angles).into()
        }
        HouseSystem::Carter => carter_houses(angles, obliquity).into(),
        HouseSystem::Horizon => {
            horizon_houses(request.instant, &request.observer, obliquity, angles).into()
        }
        HouseSystem::Apc => {
            apc_houses(request.instant, &request.observer, obliquity, angles).into()
        }
        HouseSystem::KrusinskiPisaGoelzer => {
            krusinski_pisa_goelzer_houses(request.instant, &request.observer, obliquity, angles)
                .into()
        }
        HouseSystem::Alcabitius => {
            alcabitius_houses(request.instant, &request.observer, obliquity, angles).into()
        }
        HouseSystem::Albategnius => albategnius_houses(angles).into(),
        HouseSystem::PullenSd => pullen_sd_houses(angles).into(),
        HouseSystem::PullenSr => pullen_sr_houses(angles).into(),
        HouseSystem::Sunshine => {
            sunshine_houses(request.instant, &request.observer, obliquity, angles).into()
        }
        HouseSystem::Gauquelin => {
            gauquelin_houses(request.instant, &request.observer, obliquity, angles)?.into()
        }
        HouseSystem::Meridian | HouseSystem::Axial | HouseSystem::Morinus => {
            equatorial_projection_houses(request.instant, &request.observer, obliquity).into()
        }
        HouseSystem::Topocentric => {
            topocentric_houses(request.instant, &request.observer, obliquity)?.into()
        }
        HouseSystem::Custom(custom) => {
            return Err(HouseError::new(
                HouseErrorKind::UnsupportedHouseSystem,
                format!("house placement for custom house system {custom} is not implemented yet"),
            ))
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

    let snapshot = HouseSnapshot {
        system: request.system.clone(),
        instant: request.instant,
        observer: request.observer.clone(),
        obliquity,
        angles,
        cusps,
    };
    snapshot.validate()?;

    Ok(snapshot)
}

/// Returns the one-based house number for a longitude and cusp set.
///
/// Cusps are treated as the start of each house, and wraparound at 360° is
/// handled explicitly.
pub fn house_for_longitude(longitude: Longitude, cusps: &[Longitude]) -> usize {
    if cusps.is_empty() {
        return 1;
    }

    let longitude = longitude.degrees().rem_euclid(360.0);
    for (index, cusp) in cusps.iter().enumerate() {
        let start = cusp.degrees();
        let end = cusps[(index + 1) % cusps.len()].degrees();
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

/// Rejects NaN and infinite obliquity overrides before they can flow into the
/// quadrant formulas.
fn validate_obliquity(obliquity: Angle) -> Result<Angle, HouseError> {
    if !obliquity.is_finite() {
        return Err(HouseError::new(
            HouseErrorKind::InvalidObliquity,
            "house obliquity override must be finite",
        ));
    }

    Ok(obliquity)
}

fn validate_house_snapshot(snapshot: &HouseSnapshot) -> Result<(), HouseError> {
    check_finite("obliquity", snapshot.obliquity.degrees())?;
    check_finite("ascendant", snapshot.angles.ascendant.degrees())?;
    check_finite("descendant", snapshot.angles.descendant.degrees())?;
    check_finite("midheaven", snapshot.angles.midheaven.degrees())?;
    check_finite("imum coeli", snapshot.angles.imum_coeli.degrees())?;

    let expected_cusp_count = match snapshot.system {
        HouseSystem::Gauquelin => 36,
        _ => 12,
    };
    if snapshot.cusps.len() != expected_cusp_count {
        return Err(HouseError::new(
            HouseErrorKind::NumericalFailure,
            format!(
                "house calculation for {} produced {} cusps (expected {})",
                snapshot.system,
                snapshot.cusps.len(),
                expected_cusp_count
            ),
        ));
    }

    for (index, cusp) in snapshot.cusps.iter().enumerate() {
        check_finite(format!("cusp {}", index + 1), cusp.degrees())?;
    }

    Ok(())
}

fn check_finite(label: impl Into<String>, value: f64) -> Result<(), HouseError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(HouseError::new(
            HouseErrorKind::NumericalFailure,
            format!("house calculation produced a non-finite {}", label.into()),
        ))
    }
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

fn ascendant_for(sidereal_time_deg: f64, latitude_deg: f64, obliquity_rad: f64) -> Longitude {
    let theta = sidereal_time_deg.to_radians();
    let latitude = latitude_deg.to_radians();
    Longitude::from_degrees(
        (-theta.cos())
            .atan2(theta.sin() * obliquity_rad.cos() + latitude.tan() * obliquity_rad.sin())
            .to_degrees(),
    )
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

fn equal_midheaven_houses(midheaven: Longitude) -> [Longitude; 12] {
    core::array::from_fn(|index| {
        Longitude::from_degrees(midheaven.degrees() + 90.0 + (index as f64) * 30.0)
    })
}

fn vehlow_equal_houses(ascendant: Longitude) -> [Longitude; 12] {
    core::array::from_fn(|index| {
        Longitude::from_degrees(ascendant.degrees() - 15.0 + (index as f64) * 30.0)
    })
}

fn equal_aries_houses() -> [Longitude; 12] {
    core::array::from_fn(|index| Longitude::from_degrees((index as f64) * 30.0))
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

fn carter_houses(angles: HouseAngles, obliquity: Angle) -> [Longitude; 12] {
    let reference_ra =
        right_ascension_from_ecliptic_longitude(angles.ascendant, obliquity.degrees().to_radians());

    core::array::from_fn(|index| {
        ecliptic_longitude_from_ra(
            reference_ra + (index as f64) * 30.0,
            obliquity.degrees().to_radians(),
        )
    })
}

fn horizon_houses(
    instant: Instant,
    observer: &ObserverLocation,
    obliquity: Angle,
    angles: HouseAngles,
) -> [Longitude; 12] {
    let sidereal_time =
        (local_sidereal_time(instant, observer.longitude).degrees() + 180.0).rem_euclid(360.0);
    let obliquity = obliquity.degrees().to_radians();
    let latitude = observer.latitude.degrees();
    let transformed_latitude = if latitude >= 0.0 {
        90.0 - latitude
    } else {
        -90.0 - latitude
    };
    let transformed_latitude_rad = transformed_latitude.to_radians();
    let fh1 = (transformed_latitude_rad.sin() / 2.0).asin().to_degrees();
    let fh2 = ((3.0_f64).sqrt() / 2.0 * transformed_latitude_rad.sin())
        .asin()
        .to_degrees();
    let cosfi = transformed_latitude_rad.cos();
    let xh1 = if cosfi.abs() < f64::EPSILON {
        if transformed_latitude >= 0.0 {
            90.0
        } else {
            270.0
        }
    } else {
        (3.0_f64.sqrt() / cosfi).atan().to_degrees()
    };
    let xh2 = if cosfi.abs() < f64::EPSILON {
        if transformed_latitude >= 0.0 {
            90.0
        } else {
            270.0
        }
    } else {
        (1.0 / 3.0_f64.sqrt() / cosfi).atan().to_degrees()
    };

    let mut cusps = [Longitude::from_degrees(0.0); 12];
    cusps[0] = ascendant_for(sidereal_time + 90.0, transformed_latitude, obliquity);
    cusps[9] = angles.midheaven;
    cusps[10] = ascendant_for(sidereal_time + 90.0 - xh1, fh1, obliquity);
    cusps[11] = ascendant_for(sidereal_time + 90.0 - xh2, fh2, obliquity);
    cusps[1] = ascendant_for(sidereal_time + 90.0 + xh2, fh2, obliquity);
    cusps[2] = ascendant_for(sidereal_time + 90.0 + xh1, fh1, obliquity);
    cusps[3] = longitude_opposite(cusps[9]);
    cusps[4] = longitude_opposite(cusps[10]);
    cusps[5] = longitude_opposite(cusps[11]);
    cusps[6] = longitude_opposite(cusps[0]);
    cusps[7] = longitude_opposite(cusps[1]);
    cusps[8] = longitude_opposite(cusps[2]);
    cusps
}

fn apc_sector(n: usize, latitude_rad: f64, obliquity_rad: f64, sidereal_rad: f64) -> Longitude {
    let tan_lat = latitude_rad.tan();
    let tan_obliquity = obliquity_rad.tan();
    let kv = (tan_lat * tan_obliquity * sidereal_rad.cos())
        .atan2(1.0 + tan_lat * tan_obliquity * sidereal_rad.sin());
    let sin_kv = kv.sin();
    let is_below_hor = n < 8;
    let k = if is_below_hor {
        (n as isize - 1) as f64
    } else {
        (n as isize - 13) as f64
    };
    let a = if is_below_hor {
        kv + sidereal_rad
            + core::f64::consts::FRAC_PI_2
            + k * (core::f64::consts::FRAC_PI_2 - kv) / 3.0
    } else {
        kv + sidereal_rad
            + core::f64::consts::FRAC_PI_2
            + k * (core::f64::consts::FRAC_PI_2 + kv) / 3.0
    };
    let y = sin_kv * sidereal_rad.sin() + a.sin();
    let x = obliquity_rad.cos() * (sin_kv * sidereal_rad.cos() + a.cos())
        + obliquity_rad.sin() * tan_lat * (sidereal_rad - a).sin();
    Longitude::from_degrees(y.atan2(x).to_degrees())
}

fn apc_houses(
    instant: Instant,
    observer: &ObserverLocation,
    obliquity: Angle,
    angles: HouseAngles,
) -> [Longitude; 12] {
    let sidereal_rad = local_sidereal_time(instant, observer.longitude)
        .degrees()
        .to_radians();
    let latitude_rad = observer.latitude.degrees().to_radians();
    let obliquity_rad = obliquity.degrees().to_radians();

    let mut cusps = core::array::from_fn(|index| {
        apc_sector(index + 1, latitude_rad, obliquity_rad, sidereal_rad)
    });
    cusps[0] = angles.ascendant;
    cusps[9] = angles.midheaven;
    cusps
}

fn krusinski_pisa_goelzer_houses(
    instant: Instant,
    observer: &ObserverLocation,
    obliquity: Angle,
    angles: HouseAngles,
) -> [Longitude; 12] {
    let sidereal_time = local_sidereal_time(instant, observer.longitude).degrees();
    let latitude = observer.latitude.degrees();
    let obliquity_deg = obliquity.degrees();

    let mut ascendant = angles.ascendant;
    if signed_longitude_difference(ascendant.degrees(), angles.midheaven.degrees()) < 0.0 {
        ascendant = longitude_opposite(ascendant);
    }

    let mut house_circle_point = [ascendant.degrees(), 0.0, 1.0];
    spherical_cotrans(&mut house_circle_point, -obliquity_deg);
    house_circle_point[0] = normalize_degrees(house_circle_point[0] - (sidereal_time - 90.0));
    spherical_cotrans(&mut house_circle_point, -(90.0 - latitude));
    let horizon_offset = house_circle_point[0];

    let mut cusps = [Longitude::from_degrees(0.0); 12];
    for index in 0..6 {
        let mut point = [30.0 * index as f64, 0.0, 1.0];
        spherical_cotrans(&mut point, 90.0);
        point[0] = normalize_degrees(point[0] + horizon_offset);
        spherical_cotrans(&mut point, 90.0 - latitude);
        point[0] = normalize_degrees(point[0] + (sidereal_time - 90.0));
        cusps[index] = ecliptic_longitude_from_ra(point[0], obliquity_deg.to_radians());
        cusps[index + 6] = longitude_opposite(cusps[index]);
    }

    cusps[0] = ascendant;
    cusps[6] = longitude_opposite(ascendant);
    cusps
}

fn sripati_houses(angles: HouseAngles) -> [Longitude; 12] {
    let porphyry = porphyry_houses(angles);
    core::array::from_fn(|index| {
        let previous = porphyry[(index + 11) % 12];
        midpoint_longitude(previous, porphyry[index])
    })
}

fn complete_opposite_houses(cusps: &mut [Longitude; 12]) {
    cusps[3] = longitude_opposite(cusps[9]);
    cusps[4] = longitude_opposite(cusps[10]);
    cusps[5] = longitude_opposite(cusps[11]);
    cusps[6] = longitude_opposite(cusps[0]);
    cusps[7] = longitude_opposite(cusps[1]);
    cusps[8] = longitude_opposite(cusps[2]);
}

fn gauquelin_houses(
    _instant: Instant,
    _observer: &ObserverLocation,
    _obliquity: Angle,
    angles: HouseAngles,
) -> Result<[Longitude; 36], HouseError> {
    let mut cusps = [Longitude::from_degrees(0.0); 36];
    let ascendant = angles.ascendant;
    let midheaven = angles.midheaven;
    let descendant = longitude_opposite(ascendant);
    let ic = longitude_opposite(midheaven);

    let lerp = |start: Longitude, end: Longitude, fraction: f64| {
        Longitude::from_degrees(normalize_degrees(
            start.degrees()
                + signed_longitude_difference(start.degrees(), end.degrees()) * fraction,
        ))
    };

    for (index, cusp) in cusps.iter_mut().take(9).enumerate() {
        *cusp = lerp(ascendant, midheaven, index as f64 / 9.0);
    }
    cusps[9] = midheaven;

    for (index, cusp) in cusps[10..18].iter_mut().enumerate() {
        *cusp = lerp(midheaven, descendant, (index + 1) as f64 / 9.0);
    }
    cusps[18] = descendant;

    for (index, cusp) in cusps[19..27].iter_mut().enumerate() {
        *cusp = lerp(descendant, ic, (index + 1) as f64 / 9.0);
    }
    cusps[27] = ic;

    for (index, cusp) in cusps[28..36].iter_mut().enumerate() {
        *cusp = lerp(ic, ascendant, (index + 1) as f64 / 9.0);
    }

    Ok(cusps)
}

fn albategnius_houses(angles: HouseAngles) -> [Longitude; 12] {
    let mut cusps = [Longitude::from_degrees(0.0); 12];
    cusps[0] = angles.ascendant;
    cusps[9] = angles.midheaven;

    let mut ascendant = angles.ascendant;
    let mut acmc = signed_longitude_difference(ascendant.degrees(), angles.midheaven.degrees());
    if acmc < 0.0 {
        ascendant = longitude_opposite(ascendant);
        acmc = signed_longitude_difference(ascendant.degrees(), angles.midheaven.degrees());
    }
    cusps[0] = ascendant;

    let q1 = 180.0 - acmc;
    let d = (acmc - 90.0) / 4.0;
    if acmc <= 30.0 {
        cusps[10] = Longitude::from_degrees(angles.midheaven.degrees() + acmc / 2.0);
        cusps[11] = cusps[10];
    } else {
        cusps[10] = Longitude::from_degrees(angles.midheaven.degrees() + 30.0 + d);
        cusps[11] = Longitude::from_degrees(angles.midheaven.degrees() + 60.0 + 3.0 * d);
    }

    let d = (q1 - 90.0) / 4.0;
    if q1 <= 30.0 {
        cusps[1] = Longitude::from_degrees(ascendant.degrees() + q1 / 2.0);
        cusps[2] = cusps[1];
    } else {
        cusps[1] = Longitude::from_degrees(ascendant.degrees() + 30.0 + d);
        cusps[2] = Longitude::from_degrees(ascendant.degrees() + 60.0 + 3.0 * d);
    }

    complete_opposite_houses(&mut cusps);
    cusps
}

fn pullen_sd_houses(angles: HouseAngles) -> [Longitude; 12] {
    let mut cusps = [Longitude::from_degrees(0.0); 12];
    cusps[0] = angles.ascendant;
    cusps[9] = angles.midheaven;

    let mut ascendant = angles.ascendant;
    let mut acmc = signed_longitude_difference(ascendant.degrees(), angles.midheaven.degrees());
    if acmc < 0.0 {
        ascendant = longitude_opposite(ascendant);
        acmc = signed_longitude_difference(ascendant.degrees(), angles.midheaven.degrees());
    }
    cusps[0] = ascendant;

    let q1 = 180.0 - acmc;
    let d = (acmc - 90.0) / 4.0;
    if acmc <= 30.0 {
        cusps[10] = Longitude::from_degrees(angles.midheaven.degrees() + acmc / 2.0);
        cusps[11] = cusps[10];
    } else {
        cusps[10] = Longitude::from_degrees(angles.midheaven.degrees() + 30.0 + d);
        cusps[11] = Longitude::from_degrees(angles.midheaven.degrees() + 60.0 + 3.0 * d);
    }

    let d = (q1 - 90.0) / 4.0;
    if q1 <= 30.0 {
        cusps[1] = Longitude::from_degrees(ascendant.degrees() + q1 / 2.0);
        cusps[2] = cusps[1];
    } else {
        cusps[1] = Longitude::from_degrees(ascendant.degrees() + 30.0 + d);
        cusps[2] = Longitude::from_degrees(ascendant.degrees() + 60.0 + 3.0 * d);
    }

    complete_opposite_houses(&mut cusps);
    cusps
}

fn pullen_sr_houses(angles: HouseAngles) -> [Longitude; 12] {
    let mut cusps = [Longitude::from_degrees(0.0); 12];
    cusps[0] = angles.ascendant;
    cusps[9] = angles.midheaven;

    let mut ascendant = angles.ascendant;
    let mut acmc = signed_longitude_difference(ascendant.degrees(), angles.midheaven.degrees());
    if acmc < 0.0 {
        ascendant = longitude_opposite(ascendant);
        acmc = signed_longitude_difference(ascendant.degrees(), angles.midheaven.degrees());
    }
    cusps[0] = ascendant;

    let mut q = acmc;
    if q > 90.0 {
        q = 180.0 - q;
    }

    let (x, xr, xr3, xr4) = if q < 1.0e-30 {
        (0.0, 0.0, 0.0, 180.0)
    } else {
        let c = (180.0 - q) / q;
        let csq = c * c;
        let ccr = (csq - c).cbrt();
        let cqx = (2.0_f64.powf(2.0 / 3.0) * ccr + 1.0).sqrt();
        let r1 = 0.5 * cqx;
        let r2 = 0.5 * (-2.0 * (1.0 - 2.0 * c) / cqx - 2.0_f64.powf(2.0 / 3.0) * ccr + 2.0).sqrt();
        let r = r1 + r2 - 0.5;
        let x = q / (2.0 * r + 1.0);
        let xr = r * x;
        let xr3 = xr * r * r;
        let xr4 = xr3 * r;
        (x, xr, xr3, xr4)
    };

    if acmc > 90.0 {
        cusps[10] = Longitude::from_degrees(angles.midheaven.degrees() + xr3);
        cusps[11] = Longitude::from_degrees(cusps[10].degrees() + xr4);
        cusps[1] = Longitude::from_degrees(ascendant.degrees() + xr);
        cusps[2] = Longitude::from_degrees(cusps[1].degrees() + x);
    } else {
        cusps[10] = Longitude::from_degrees(angles.midheaven.degrees() + xr);
        cusps[11] = Longitude::from_degrees(cusps[10].degrees() + x);
        cusps[1] = Longitude::from_degrees(ascendant.degrees() + xr3);
        cusps[2] = Longitude::from_degrees(cusps[1].degrees() + xr4);
    }

    complete_opposite_houses(&mut cusps);
    cusps
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

fn sunshine_houses(
    instant: Instant,
    observer: &ObserverLocation,
    obliquity: Angle,
    angles: HouseAngles,
) -> [Longitude; 12] {
    const SUNSHINE_KEEP_MC_SOUTH: bool = false;

    let sidereal_time = local_sidereal_time(instant, observer.longitude).degrees();
    let latitude = observer.latitude.degrees();
    let obliquity_deg = obliquity.degrees();
    let sundec = apparent_solar_declination(instant, obliquity).degrees();
    let mc_under_horizon = latitude.signum() != 0.0
        && (latitude - apparent_midheaven_declination(sidereal_time, obliquity_deg)).abs() > 90.0;

    let mut cusps = [Longitude::from_degrees(0.0); 12];
    let mut ascendant = angles.ascendant;
    let mut midheaven = angles.midheaven;
    let acmc = signed_longitude_difference(ascendant.degrees(), midheaven.degrees());
    if acmc < 0.0 {
        ascendant = longitude_opposite(ascendant);
        if !SUNSHINE_KEEP_MC_SOUTH {
            midheaven = longitude_opposite(midheaven);
        }
    }

    cusps[0] = ascendant;
    cusps[3] = longitude_opposite(midheaven);
    cusps[6] = longitude_opposite(ascendant);
    cusps[9] = midheaven;

    let offsets = sunshine_offsets(latitude, sundec);
    let sin_ecl = obliquity_deg.to_radians().sin();
    let cos_ecl = obliquity_deg.to_radians().cos();

    for house in [2usize, 3, 5, 6, 8, 9, 11, 12] {
        let offset = offsets[house];
        let xhs = 2.0
            * (sundec.to_radians().cos() * (offset.to_radians() / 2.0).sin())
                .asin()
                .to_degrees();
        let cosa = (sundec.to_radians().tan() * (xhs.to_radians() / 2.0).tan()).clamp(-1.0, 1.0);
        let alph = cosa.acos().to_degrees();
        let (alpha2, b) = if house > 7 {
            (180.0 - alph, 90.0 - latitude + sundec)
        } else {
            (alph, 90.0 - latitude - sundec)
        };

        let cosc = xhs.to_radians().cos() * b.to_radians().cos()
            + xhs.to_radians().sin() * b.to_radians().sin() * alpha2.to_radians().cos();
        let c = cosc.clamp(-1.0, 1.0).acos().to_degrees();
        let sinzd = if c.abs() < f64::EPSILON {
            0.0
        } else {
            xhs.to_radians().sin() * alpha2.to_radians().sin() / c.to_radians().sin()
        };
        let zd = sinzd.clamp(-1.0, 1.0).asin().to_degrees();
        let rax = (latitude.to_radians().cos() * zd.to_radians().tan())
            .atan()
            .to_degrees();
        let pole = (sinzd * latitude.to_radians().sin())
            .clamp(-1.0, 1.0)
            .asin()
            .to_degrees();
        let pole = if house <= 6 { -pole } else { pole };
        let a = if house <= 6 {
            sidereal_time + 180.0 + rax
        } else {
            sidereal_time + rax
        };
        cusps[house - 1] = asc1(a, pole, sin_ecl, cos_ecl);
    }

    if mc_under_horizon && !SUNSHINE_KEEP_MC_SOUTH {
        for house in [2usize, 3, 5, 6, 8, 9, 11, 12] {
            cusps[house - 1] = longitude_opposite(cusps[house - 1]);
        }
    }

    cusps
}

fn sunshine_offsets(latitude_deg: f64, sun_declination_deg: f64) -> [f64; 13] {
    let mut offsets = [0.0; 13];
    let tan_product = sun_declination_deg.to_radians().tan() * latitude_deg.to_radians().tan();
    let ascensional_difference = tan_product.clamp(-1.0, 1.0).asin().to_degrees();
    let nocturnal_semi_arc = 90.0 - ascensional_difference;
    let diurnal_semi_arc = 90.0 + ascensional_difference;
    offsets[2] = -2.0 * nocturnal_semi_arc / 3.0;
    offsets[3] = -nocturnal_semi_arc / 3.0;
    offsets[5] = nocturnal_semi_arc / 3.0;
    offsets[6] = 2.0 * nocturnal_semi_arc / 3.0;
    offsets[8] = -2.0 * diurnal_semi_arc / 3.0;
    offsets[9] = -diurnal_semi_arc / 3.0;
    offsets[11] = diurnal_semi_arc / 3.0;
    offsets[12] = 2.0 * diurnal_semi_arc / 3.0;
    offsets
}

fn apparent_solar_declination(instant: Instant, obliquity: Angle) -> Angle {
    let days = instant.julian_day.days() - 2_451_545.0;
    let mean_longitude = Angle::from_degrees(280.460 + 0.985_647_4 * days).normalized_0_360();
    let mean_anomaly = Angle::from_degrees(357.528 + 0.985_600_3 * days).normalized_0_360();
    let lambda = Angle::from_degrees(
        mean_longitude.degrees()
            + 1.915 * mean_anomaly.radians().sin()
            + 0.020 * (2.0 * mean_anomaly.radians()).sin(),
    )
    .normalized_0_360();
    Angle::from_degrees(
        (obliquity.radians().sin() * lambda.radians().sin())
            .asin()
            .to_degrees(),
    )
}

fn apparent_midheaven_declination(sidereal_time_deg: f64, obliquity_deg: f64) -> f64 {
    (sidereal_time_deg.to_radians().sin() * obliquity_deg.to_radians().tan())
        .atan()
        .to_degrees()
}

fn topocentric_latitude(latitude_deg: f64, elevation_m: Option<f64>) -> Result<Angle, HouseError> {
    let latitude = latitude_deg.to_radians();
    let elevation = match elevation_m {
        Some(elevation) if elevation.is_finite() => elevation,
        Some(_) => {
            return Err(HouseError::new(
                HouseErrorKind::InvalidElevation,
                "observer elevation must be finite when provided",
            ))
        }
        None => 0.0,
    };

    // Use the geodetic-to-geocentric conversion for the observer latitude, so
    // topocentric house placement reflects the actual Earth ellipsoid instead
    // of a rough spherical approximation.
    let semi_major_m = 6_378_137.0;
    let flattening = 1.0 / 298.257_223_563;
    let eccentricity_sq = flattening * (2.0 - flattening);
    let sin_lat = latitude.sin();
    let cos_lat = latitude.cos();
    let prime_vertical = semi_major_m / (1.0 - eccentricity_sq * sin_lat * sin_lat).sqrt();
    let x = (prime_vertical + elevation) * cos_lat;
    let z = (prime_vertical * (1.0 - eccentricity_sq) + elevation) * sin_lat;
    Ok(Angle::from_degrees(z.atan2(x).to_degrees()))
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

fn right_ascension_from_ecliptic_longitude(longitude: Longitude, obliquity: f64) -> f64 {
    let longitude = longitude.degrees().to_radians();
    (longitude.sin() * obliquity.cos())
        .atan2(longitude.cos())
        .to_degrees()
}

fn ecliptic_longitude_from_ra(ra_deg: f64, obliquity: f64) -> Longitude {
    let ra = ra_deg.to_radians();
    Longitude::from_degrees(ra.sin().atan2(ra.cos() * obliquity.cos()).to_degrees())
}

fn interpolate_longitude(start: Longitude, end: Longitude, fraction: f64) -> Longitude {
    let span = (end.degrees() - start.degrees()).rem_euclid(360.0);
    Longitude::from_degrees(start.degrees() + span * fraction)
}

fn midpoint_longitude(start: Longitude, end: Longitude) -> Longitude {
    interpolate_longitude(start, end, 0.5)
}

fn asc1(x1: f64, pole_height: f64, sine: f64, cose: f64) -> Longitude {
    let x1 = normalize_degrees(x1);
    let quadrant = (x1 / 90.0).floor() as i32 + 1;
    let lon = match quadrant {
        1 => asc2(x1, pole_height, sine, cose),
        2 => 180.0 - asc2(180.0 - x1, -pole_height, sine, cose),
        3 => 180.0 + asc2(x1 - 180.0, -pole_height, sine, cose),
        _ => 360.0 - asc2(360.0 - x1, pole_height, sine, cose),
    };
    Longitude::from_degrees(lon)
}

fn asc2(x: f64, pole_height: f64, sine: f64, cose: f64) -> f64 {
    let mut value = -pole_height.to_radians().tan() * sine + cose * x.to_radians().cos();
    if value.abs() < 1.0e-12 {
        value = 0.0;
    }
    let sinx = x.to_radians().sin();
    let mut longitude = if sinx.abs() < 1.0e-12 {
        if value < 0.0 {
            -1.0e-12
        } else {
            1.0e-12
        }
    } else if value == 0.0 {
        if sinx < 0.0 {
            -90.0
        } else {
            90.0
        }
    } else {
        (sinx / value).atan().to_degrees()
    };
    if longitude < 0.0 {
        longitude += 180.0;
    }
    longitude
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

fn normalize_degrees(degrees: f64) -> f64 {
    degrees.rem_euclid(360.0)
}

fn signed_longitude_difference(a: f64, b: f64) -> f64 {
    let delta = normalize_degrees(a - b);
    if delta >= 180.0 {
        delta - 360.0
    } else {
        delta
    }
}

fn spherical_cotrans(coord: &mut [f64; 3], angle_deg: f64) {
    let lon = coord[0].to_radians();
    let lat = coord[1].to_radians();
    let radius = coord[2];
    let x = radius * lat.cos() * lon.cos();
    let y = radius * lat.cos() * lon.sin();
    let z = radius * lat.sin();

    let angle = angle_deg.to_radians();
    let y_rot = y * angle.cos() + z * angle.sin();
    let z_rot = -y * angle.sin() + z * angle.cos();
    let radius = (x * x + y_rot * y_rot + z_rot * z_rot).sqrt();

    coord[0] = y_rot.atan2(x).to_degrees();
    coord[1] = z_rot.atan2((x * x + y_rot * y_rot).sqrt()).to_degrees();
    coord[2] = radius;
}

fn catalog_name(system: &HouseSystem) -> &'static str {
    match system {
        HouseSystem::Placidus => "Placidus",
        HouseSystem::Koch => "Koch",
        HouseSystem::Porphyry => "Porphyry",
        HouseSystem::Regiomontanus => "Regiomontanus",
        HouseSystem::Campanus => "Campanus",
        HouseSystem::Carter => "Carter (poli-equatorial)",
        HouseSystem::Horizon => "Horizon/Azimuth",
        HouseSystem::Apc => "APC",
        HouseSystem::KrusinskiPisaGoelzer => "Krusinski-Pisa-Goelzer",
        HouseSystem::Equal => "Equal",
        HouseSystem::EqualMidheaven => "Equal (MC)",
        HouseSystem::EqualAries => "Equal (1=Aries)",
        HouseSystem::Vehlow => "Vehlow Equal",
        HouseSystem::Sripati => "Sripati",
        HouseSystem::WholeSign => "Whole Sign",
        HouseSystem::Alcabitius => "Alcabitius",
        HouseSystem::Albategnius => "Albategnius",
        HouseSystem::PullenSd => "Pullen SD",
        HouseSystem::PullenSr => "Pullen SR",
        HouseSystem::Meridian => "Meridian",
        HouseSystem::Axial => "Axial",
        HouseSystem::Topocentric => "Topocentric",
        HouseSystem::Morinus => "Morinus",
        HouseSystem::Sunshine => "Sunshine",
        HouseSystem::Gauquelin => "Gauquelin sectors",
        HouseSystem::Custom(_) => "Custom",
        _ => "Unspecified",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{Angle, CustomHouseSystem, Latitude};

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
    fn house_request_summary_line_reports_instant_observer_system_and_obliquity() {
        let request = sample_request(HouseSystem::WholeSign);
        assert_eq!(
            request.summary_line(),
            format!(
                "instant={}; observer={}; system={}; obliquity=auto",
                &request.instant, &request.observer, &request.system
            )
        );
        assert_eq!(request.to_string(), request.summary_line());

        let request_with_obliquity = request.with_obliquity(Angle::from_degrees(23.5));
        assert_eq!(
            request_with_obliquity.summary_line(),
            format!(
                "instant={}; observer={}; system={}; obliquity=23.5°",
                &request_with_obliquity.instant,
                &request_with_obliquity.observer,
                &request_with_obliquity.system
            )
        );

        let mut custom = CustomHouseSystem::new("House from custom notes");
        custom.aliases.push("Custom alias".to_string());
        custom.notes = Some("extra custom context".to_string());
        let custom_request = HouseRequest::new(
            Instant::new(
                pleiades_types::JulianDay::from_days(2_451_545.0),
                pleiades_types::TimeScale::Tt,
            ),
            observer(),
            HouseSystem::Custom(custom.clone()),
        );
        assert!(custom_request.summary_line().contains(&custom.to_string()));
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
    fn equal_midheaven_and_vehlow_variants_are_available() {
        let mc_snapshot = calculate_houses(&sample_request(HouseSystem::EqualMidheaven))
            .expect("equal (MC) houses should work");
        assert!(
            (mc_snapshot.cusps[9].degrees() - mc_snapshot.angles.midheaven.degrees()).abs()
                < 1.0e-12
        );
        assert_eq!(
            (mc_snapshot.cusps[1].degrees() - mc_snapshot.cusps[0].degrees()).rem_euclid(360.0),
            30.0
        );
        assert_eq!(
            (mc_snapshot.cusps[0].degrees() - mc_snapshot.angles.midheaven.degrees())
                .rem_euclid(360.0),
            90.0
        );

        let vehlow_snapshot = calculate_houses(&sample_request(HouseSystem::Vehlow))
            .expect("vehlow houses should work");
        assert_eq!(
            (vehlow_snapshot.angles.ascendant.degrees() - vehlow_snapshot.cusps[0].degrees())
                .rem_euclid(360.0),
            15.0
        );
        assert_eq!(
            (vehlow_snapshot.cusps[1].degrees() - vehlow_snapshot.cusps[0].degrees())
                .rem_euclid(360.0),
            30.0
        );
    }

    #[test]
    fn sripati_midpoints_follow_porphyry_segments() {
        let snapshot = calculate_houses(&sample_request(HouseSystem::Sripati))
            .expect("sripati houses should work");
        let porphyry = calculate_houses(&sample_request(HouseSystem::Porphyry))
            .expect("porphyry houses should work");
        assert_eq!(
            snapshot.cusps[0],
            midpoint_longitude(porphyry.cusps[11], porphyry.cusps[0])
        );
        assert_eq!(
            snapshot.cusps[3],
            midpoint_longitude(porphyry.cusps[2], porphyry.cusps[3])
        );
        assert_eq!(
            snapshot.cusps[9],
            midpoint_longitude(porphyry.cusps[8], porphyry.cusps[9])
        );
    }

    #[test]
    fn equal_aries_houses_start_at_zero_aries() {
        let snapshot = calculate_houses(&sample_request(HouseSystem::EqualAries))
            .expect("equal Aries houses should work");
        assert_eq!(snapshot.cusps[0].degrees(), 0.0);
        assert_eq!(snapshot.cusps[1].degrees(), 30.0);
        assert_eq!(snapshot.cusps[11].degrees(), 330.0);
    }

    #[test]
    fn placidian_houses_report_numerical_failure_at_the_pole() {
        let mut request =
            sample_request(HouseSystem::Placidus).with_obliquity(Angle::from_degrees(0.0));
        request.observer.latitude = Latitude::from_degrees(90.0);
        request.observer.longitude = Longitude::from_degrees(0.0);

        let error = calculate_houses(&request).expect_err("polar Placidus iteration should fail");
        assert_eq!(error.kind, HouseErrorKind::NumericalFailure);
        assert!(error.message.contains("zero derivative"));
    }

    #[test]
    fn topocentric_latitude_uses_geocentric_correction() {
        let sea_level = topocentric_latitude(45.0, None).expect("latitude should convert");
        let mountain = topocentric_latitude(45.0, Some(2_000.0)).expect("latitude should convert");

        assert!((sea_level.degrees() - 44.807_576).abs() < 1.0e-6);
        assert!(mountain.degrees() > sea_level.degrees());
    }

    #[test]
    fn topocentric_latitude_rejects_non_finite_elevation() {
        let error = topocentric_latitude(45.0, Some(f64::NAN))
            .expect_err("non-finite elevation should fail");
        assert_eq!(error.kind, HouseErrorKind::InvalidElevation);
        assert!(error
            .message
            .contains("observer elevation must be finite when provided"));
    }

    #[test]
    fn topocentric_houses_follow_the_geocentric_latitude_correction() {
        let mut topocentric_request = sample_request(HouseSystem::Topocentric);
        topocentric_request.observer.latitude = Latitude::from_degrees(45.0);
        topocentric_request.observer.longitude = Longitude::from_degrees(10.0);
        topocentric_request.observer.elevation_m = Some(2_000.0);

        let topocentric =
            calculate_houses(&topocentric_request).expect("topocentric houses should work");
        assert_eq!(topocentric.cusps.len(), 12);

        let corrected_latitude =
            topocentric_latitude(45.0, Some(2_000.0)).expect("topocentric latitude should convert");
        let mut placidus_request = topocentric_request.clone();
        placidus_request.system = HouseSystem::Placidus;
        placidus_request.observer.latitude = Latitude::from_degrees(corrected_latitude.degrees());

        let placidus = calculate_houses(&placidus_request)
            .expect("placidus houses should match the corrected latitude");

        let mut geodetic_placidus_request = topocentric_request.clone();
        geodetic_placidus_request.system = HouseSystem::Placidus;
        let geodetic_placidus =
            calculate_houses(&geodetic_placidus_request).expect("placidus houses should work");

        assert_ne!(topocentric.cusps[1], geodetic_placidus.cusps[1]);
        assert_eq!(topocentric.angles, geodetic_placidus.angles);
        assert_eq!(topocentric.cusps, placidus.cusps);
    }

    #[test]
    fn observer_latitudes_outside_the_valid_range_are_rejected() {
        let mut request = sample_request(HouseSystem::Equal);
        request.observer.latitude = Latitude::from_degrees(90.000_1);

        let error = calculate_houses(&request).expect_err("invalid observer latitude should fail");
        assert_eq!(error.kind, HouseErrorKind::InvalidLatitude);
        assert!(error
            .message
            .contains("observer latitude 90.0001° is outside the valid range"));
    }

    #[test]
    fn non_finite_obliquity_overrides_are_rejected() {
        for system in [
            HouseSystem::Equal,
            HouseSystem::Placidus,
            HouseSystem::Topocentric,
        ] {
            for obliquity in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                let request =
                    sample_request(system.clone()).with_obliquity(Angle::from_degrees(obliquity));

                let error = calculate_houses(&request).expect_err("invalid obliquity should fail");
                assert_eq!(error.kind, HouseErrorKind::InvalidObliquity);
                assert!(error.message.contains("obliquity override must be finite"));
            }
        }
    }

    #[test]
    fn house_snapshots_reject_non_finite_derived_values() {
        let mut cusps = vec![Longitude::from_degrees(0.0); 12];
        cusps[4] = Longitude::from_degrees(f64::NAN);

        let snapshot = HouseSnapshot {
            system: HouseSystem::Equal,
            instant: sample_request(HouseSystem::Equal).instant,
            observer: sample_request(HouseSystem::Equal).observer,
            obliquity: Angle::from_degrees(23.4),
            angles: HouseAngles {
                ascendant: Longitude::from_degrees(15.0),
                descendant: Longitude::from_degrees(195.0),
                midheaven: Longitude::from_degrees(45.0),
                imum_coeli: Longitude::from_degrees(225.0),
            },
            cusps,
        };

        let error = snapshot
            .validate()
            .expect_err("non-finite cusp should fail");
        assert_eq!(error.kind, HouseErrorKind::NumericalFailure);
        assert!(error.message.contains("non-finite cusp 5"));
    }

    #[test]
    fn custom_house_systems_are_reported_explicitly_when_unsupported() {
        let mut custom = CustomHouseSystem::new("My Custom Houses");
        custom.aliases.push("MCH".to_string());
        custom.notes = Some("user-defined formula".to_string());

        let error = calculate_houses(&sample_request(HouseSystem::Custom(custom)))
            .expect_err("custom house systems should still be rejected");
        assert_eq!(error.kind, HouseErrorKind::UnsupportedHouseSystem);
        assert_eq!(
            error.to_string(),
            "UnsupportedHouseSystem: house placement for custom house system My Custom Houses [aliases: MCH] (user-defined formula) is not implemented yet"
        );
    }

    #[test]
    fn baseline_quadrant_systems_are_implemented() {
        for system in [
            HouseSystem::Placidus,
            HouseSystem::Koch,
            HouseSystem::Regiomontanus,
            HouseSystem::Campanus,
            HouseSystem::Carter,
            HouseSystem::Alcabitius,
            HouseSystem::Meridian,
            HouseSystem::Axial,
            HouseSystem::Morinus,
            HouseSystem::Topocentric,
            HouseSystem::KrusinskiPisaGoelzer,
        ] {
            let snapshot = calculate_houses(&sample_request(system.clone()))
                .expect("baseline quadrant system should calculate");
            assert_eq!(snapshot.cusps.len(), 12);
        }
    }

    #[test]
    fn carter_houses_follow_ascendant_centered_equatorial_spacing() {
        let request = sample_request(HouseSystem::Carter).with_obliquity(Angle::from_degrees(0.0));
        let snapshot = calculate_houses(&request).expect("carter houses should work");
        assert!(
            (snapshot.cusps[0].degrees() - snapshot.angles.ascendant.degrees()).abs() < 1.0e-10
        );
        assert_eq!(
            (snapshot.cusps[1].degrees() - snapshot.cusps[0].degrees()).rem_euclid(360.0),
            30.0
        );
    }

    #[test]
    fn albategnius_and_pullen_release_systems_are_available() {
        for system in [
            HouseSystem::Albategnius,
            HouseSystem::PullenSd,
            HouseSystem::PullenSr,
        ] {
            let snapshot = calculate_houses(&sample_request(system.clone()))
                .expect("release house system should calculate");
            assert_eq!(snapshot.cusps.len(), 12);
            assert_eq!(snapshot.cusps[9], snapshot.angles.midheaven);
            assert_eq!(
                (snapshot.cusps[6].degrees() - snapshot.cusps[0].degrees()).rem_euclid(360.0),
                180.0
            );
        }
    }

    #[test]
    fn sunshine_release_system_anchors_the_documented_axes() {
        let snapshot = calculate_houses(&sample_request(HouseSystem::Sunshine))
            .expect("sunshine houses should work");
        assert_eq!(snapshot.cusps.len(), 12);
        assert!(
            snapshot.cusps[0] == snapshot.angles.ascendant
                || snapshot.cusps[0] == longitude_opposite(snapshot.angles.ascendant)
        );
        assert!(
            snapshot.cusps[9] == snapshot.angles.midheaven
                || snapshot.cusps[9] == longitude_opposite(snapshot.angles.midheaven)
        );
        assert_eq!(snapshot.cusps[3], longitude_opposite(snapshot.cusps[9]));
        assert_eq!(snapshot.cusps[6], longitude_opposite(snapshot.cusps[0]));
        assert_eq!(
            (snapshot.cusps[9].degrees() - snapshot.cusps[0].degrees()).rem_euclid(360.0),
            90.0
        );
    }

    #[test]
    fn horizon_and_apc_release_systems_are_available() {
        let horizon = calculate_houses(&sample_request(HouseSystem::Horizon))
            .expect("horizon houses should work");
        assert_eq!(horizon.cusps.len(), 12);
        assert_eq!(horizon.cusps[9], horizon.angles.midheaven);
        assert_ne!(horizon.cusps[0], horizon.angles.ascendant);

        let apc =
            calculate_houses(&sample_request(HouseSystem::Apc)).expect("apc houses should work");
        assert_eq!(apc.cusps.len(), 12);
        assert_eq!(apc.cusps[0], apc.angles.ascendant);
        assert_eq!(apc.cusps[9], apc.angles.midheaven);
    }

    #[test]
    fn krusinski_pisa_goelzer_release_system_is_available() {
        let snapshot = calculate_houses(&sample_request(HouseSystem::KrusinskiPisaGoelzer))
            .expect("krusinski-pisa-goelzer houses should work");
        assert_eq!(snapshot.cusps.len(), 12);
        assert!(snapshot.cusps[0].degrees().is_finite());
        assert_eq!(
            (snapshot.cusps[6].degrees() - snapshot.cusps[0].degrees()).rem_euclid(360.0),
            180.0
        );
        assert_ne!(snapshot.cusps[0], snapshot.cusps[9]);
    }

    #[test]
    fn gauquelin_release_system_exposes_thirty_six_sectors() {
        let snapshot = calculate_houses(&sample_request(HouseSystem::Gauquelin))
            .expect("gauquelin sectors should work");
        assert_eq!(snapshot.cusps.len(), 36);
        assert_eq!(snapshot.cusps[0], snapshot.angles.ascendant);
        assert_eq!(snapshot.cusps[9], snapshot.angles.midheaven);
        assert_eq!(
            snapshot.cusps[18],
            longitude_opposite(snapshot.angles.ascendant)
        );
        assert_eq!(
            snapshot.cusps[27],
            longitude_opposite(snapshot.angles.midheaven)
        );
        assert_eq!(snapshot.cusp(36), Some(snapshot.cusps[35]));
        assert_eq!(
            (snapshot.cusps[1].degrees() - snapshot.cusps[0].degrees()).rem_euclid(360.0),
            350.0
        );
        assert_eq!(
            (snapshot.cusps[35].degrees() - snapshot.cusps[34].degrees()).rem_euclid(360.0),
            350.0
        );
    }

    #[test]
    fn house_snapshots_reject_wrong_cusp_counts() {
        let snapshot = HouseSnapshot {
            system: HouseSystem::Gauquelin,
            instant: sample_request(HouseSystem::Gauquelin).instant,
            observer: observer(),
            obliquity: Angle::from_degrees(23.4),
            angles: HouseAngles {
                ascendant: Longitude::from_degrees(15.0),
                descendant: Longitude::from_degrees(195.0),
                midheaven: Longitude::from_degrees(45.0),
                imum_coeli: Longitude::from_degrees(225.0),
            },
            cusps: vec![Longitude::from_degrees(0.0); 12],
        };

        let error = snapshot
            .validate()
            .expect_err("wrong cusp count should fail");
        assert_eq!(error.kind, HouseErrorKind::NumericalFailure);
        assert!(error
            .message
            .contains("house calculation for Gauquelin sectors produced 12 cusps (expected 36)"));
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
        let snapshot = HouseSnapshot {
            system: HouseSystem::Equal,
            instant: sample_request(HouseSystem::Equal).instant,
            observer: observer(),
            obliquity: Angle::from_degrees(23.4),
            angles: HouseAngles {
                ascendant: Longitude::from_degrees(15.0),
                descendant: Longitude::from_degrees(195.0),
                midheaven: Longitude::from_degrees(75.0),
                imum_coeli: Longitude::from_degrees(255.0),
            },
            cusps: cusps.to_vec(),
        };

        assert_eq!(
            house_for_longitude(Longitude::from_degrees(359.0), &cusps),
            1
        );
        assert_eq!(house_for_longitude(Longitude::from_degrees(0.0), &cusps), 2);
        assert_eq!(
            snapshot.house_for_longitude(Longitude::from_degrees(15.0)),
            2
        );
        assert_eq!(
            snapshot.house_for_longitude(Longitude::from_degrees(29.999)),
            2
        );
        assert_eq!(
            snapshot.house_for_longitude(Longitude::from_degrees(30.0)),
            3
        );
        assert_eq!(
            snapshot.house_for_longitude(Longitude::from_degrees(44.999)),
            3
        );
    }
}
