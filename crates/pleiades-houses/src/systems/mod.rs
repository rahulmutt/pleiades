//! House-system calculation implementations.
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

use pleiades_apparent::nutation::nutation as apparent_nutation;
use pleiades_types::{
    Angle, HouseSystem, Instant, Latitude, Longitude, ObserverLocation,
    ObserverLocationValidationError,
};

use crate::error::{HouseError, HouseErrorKind};

/// Behaviour when a latitude-sensitive system is requested beyond its
/// documented latitude bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum HighLatitudePolicy {
    /// Reject with `InvalidLatitude` (the safe default).
    #[default]
    Strict,
    /// Reproduce Swiss Ephemeris's documented substitution: silently compute
    /// Porphyry cusps instead of returning an error when the observer latitude
    /// exceeds the system's documented bound.
    SwissEphemerisFallback,
}

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
    /// Behaviour when latitude exceeds the system's documented bound.
    pub high_latitude_policy: HighLatitudePolicy,
}

impl HouseRequest {
    /// Creates a new house calculation request.
    pub fn new(instant: Instant, observer: ObserverLocation, system: HouseSystem) -> Self {
        Self {
            instant,
            observer,
            system,
            obliquity: None,
            high_latitude_policy: HighLatitudePolicy::Strict,
        }
    }

    /// Overrides the obliquity used for angle derivation.
    pub fn with_obliquity(mut self, obliquity: Angle) -> Self {
        self.obliquity = Some(obliquity);
        self
    }

    /// Selects the high-latitude behaviour (default: `Strict`).
    pub fn with_high_latitude_policy(mut self, policy: HighLatitudePolicy) -> Self {
        self.high_latitude_policy = policy;
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

    /// Validates the request's observer location and obliquity override.
    ///
    /// This is a lightweight preflight for callers that want to check the
    /// house-observer contract before invoking the full house calculation.
    /// The helper does not retag the instant or infer any time-scale policy;
    /// it only checks the same observer-location, obliquity, and topocentric
    /// elevation constraints enforced by [`calculate_houses`].
    pub fn validate(&self) -> Result<(), HouseError> {
        validated_obliquity(self).map(|_| ())
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

/// The full Swiss-Ephemeris `ascmc` chart-point set.
///
/// Longitudes are apparent, equinox-of-date, tropical, in `[0,360)`. Vertex,
/// equatorial ascendant, the co-ascendants, and the polar ascendant are ported
/// from Swiss Ephemeris `swehouse.c`; their numeric correctness is enforced by
/// the `validate-angles` parity gate.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct AscMc {
    /// Ascendant (`ascmc[0]`).
    pub ascendant: Longitude,
    /// Midheaven (`ascmc[1]`).
    pub midheaven: Longitude,
    /// Descendant (Ascendant + 180°).
    pub descendant: Longitude,
    /// Imum Coeli (Midheaven + 180°).
    pub imum_coeli: Longitude,
    /// ARMC — right ascension of the Midheaven / local apparent sidereal time in degrees (`ascmc[2]`).
    pub armc: Longitude,
    /// Vertex (`ascmc[3]`).
    pub vertex: Longitude,
    /// Antivertex (Vertex + 180°).
    pub antivertex: Longitude,
    /// Equatorial ascendant / East Point (`ascmc[4]`).
    pub equatorial_ascendant: Longitude,
    /// Co-ascendant, Koch definition (`ascmc[5]`).
    pub coascendant_koch: Longitude,
    /// Co-ascendant, Munkasey definition (`ascmc[6]`).
    pub coascendant_munkasey: Longitude,
    /// Polar ascendant, Munkasey definition (`ascmc[7]`).
    pub polar_ascendant: Longitude,
}

/// Computes the full chart-point set from ARMC (degrees), geographic latitude
/// (degrees), and obliquity (degrees). Errors if any point is non-finite.
fn asc_mc_from(armc_deg: f64, lat_deg: f64, obliquity_deg: f64) -> Result<AscMc, HouseError> {
    let obl = obliquity_deg.to_radians();
    let theta = armc_deg.to_radians();

    let ascendant = ascendant_for(armc_deg, lat_deg, obl);
    let midheaven =
        Longitude::from_degrees(theta.sin().atan2(theta.cos() * obl.cos()).to_degrees());

    // The following five points are faithful ports of Swiss Ephemeris
    // `swehouse.c` (function `swe_houses_armc` / `CalcH`, lines ~2001-2048),
    // enforced by the `validate-angles` SE-parity gate. The SE helper
    // `Asc1(x, f, sine, cose)` equals `ascendant_for(x - 90, f, obl)`, so each
    // `Asc1(th ± 90, f)` call maps to `ascendant_for(th ± 90 - 90, f)` below
    // (`th` == `armc_deg`, `fi` == `lat_deg`, `ekl` == `obliquity_deg`).

    // Equatorial ascendant / East Point (swehouse.c `equasc`): the Ascendant at
    // geographic latitude 0, i.e. `Asc1(th + 90, 0)`.
    let equatorial_ascendant = ascendant_for(armc_deg, 0.0, obl);

    // swehouse.c: `if (fi >= 0) f = 90 - fi; else f = -90 - fi;` — the pole
    // height used by both the vertex and the Munkasey co-ascendant.
    let f_pole = if lat_deg >= 0.0 {
        90.0 - lat_deg
    } else {
        -90.0 - lat_deg
    };

    // Vertex (swehouse.c `vertex = Asc1(th - 90, f)` == `ascendant_for(th - 180, f)`),
    // kept on the western hemisphere for |lat| <= obliquity via the SE
    // `swe_difdeg2n(vertex, mc) > 0` flip.
    let mut vertex_deg = ascendant_for(armc_deg - 180.0, f_pole, obl).degrees();
    if lat_deg.abs() <= obliquity_deg {
        let mut vemc = (vertex_deg - midheaven.degrees()).rem_euclid(360.0);
        if vemc > 180.0 {
            vemc -= 360.0;
        }
        if vemc > 0.0 {
            vertex_deg = (vertex_deg + 180.0).rem_euclid(360.0);
        }
    }
    let vertex = Longitude::from_degrees(vertex_deg);

    // Co-ascendant, W. Koch (swehouse.c `coasc1 = degnorm(Asc1(th - 90, fi) + 180)`
    // == `longitude_opposite(ascendant_for(th - 180, fi))`).
    let coascendant_koch = longitude_opposite(ascendant_for(armc_deg - 180.0, lat_deg, obl));

    // Co-ascendant, M. Munkasey (swehouse.c `coasc2 = Asc1(th + 90, f)`
    // == `ascendant_for(th, f)`, with the same pole height `f` as the vertex).
    let coascendant_munkasey = ascendant_for(armc_deg, f_pole, obl);

    // Polar ascendant, M. Munkasey (swehouse.c `polasc = Asc1(th - 90, fi)`
    // == `ascendant_for(th - 180, fi)` == `longitude_opposite(coasc1)`).
    let polar_ascendant = longitude_opposite(coascendant_koch);

    let points = AscMc {
        ascendant,
        midheaven,
        descendant: longitude_opposite(ascendant),
        imum_coeli: longitude_opposite(midheaven),
        armc: Longitude::from_degrees(armc_deg),
        vertex,
        antivertex: longitude_opposite(vertex),
        equatorial_ascendant,
        coascendant_koch,
        coascendant_munkasey,
        polar_ascendant,
    };

    for (label, p) in [
        ("ascendant", points.ascendant),
        ("midheaven", points.midheaven),
        ("vertex", points.vertex),
        ("equatorial_ascendant", points.equatorial_ascendant),
        ("coascendant_koch", points.coascendant_koch),
        ("coascendant_munkasey", points.coascendant_munkasey),
        ("polar_ascendant", points.polar_ascendant),
    ] {
        check_finite(format!("asc_mc {label}"), p.degrees())?;
    }
    Ok(points)
}

/// Computes the full chart-point set for an instant and observer. Obliquity
/// defaults to true obliquity of date (mean + Δε) when `None`.
pub fn chart_points(
    instant: Instant,
    observer: &ObserverLocation,
    obliquity: Option<Angle>,
) -> Result<AscMc, HouseError> {
    validate_observer(observer)?;
    let obliquity = match obliquity {
        Some(o) => {
            validate_obliquity(o)?;
            o
        }
        None => {
            let mean = mean_obliquity(instant);
            let (_dpsi, deps) = nutation_for(instant)?;
            let obl = Angle::from_degrees(mean.degrees() + deps);
            validate_obliquity(obl)?;
            obl
        }
    };
    let armc = local_sidereal_time(instant, observer.longitude).degrees();
    asc_mc_from(armc, observer.latitude.degrees(), obliquity.degrees())
}

/// Computes the full chart-point set from a supplied ARMC (the
/// `swe_houses_armc` case), geographic latitude, and obliquity.
pub fn chart_points_from_armc(
    armc: Longitude,
    geolat: Latitude,
    obliquity: Angle,
) -> Result<AscMc, HouseError> {
    validate_obliquity(obliquity)?;
    asc_mc_from(armc.degrees(), geolat.degrees(), obliquity.degrees())
}

/// A complete house cusp set.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct HouseSnapshot {
    /// House system used for the calculation.
    pub system: HouseSystem,
    /// Instant used for the calculation.
    pub instant: Instant,
    /// Observer location used for the calculation.
    pub observer: ObserverLocation,
    /// Obliquity used to derive the angles.
    pub obliquity: Angle,
    /// Derived angles (four classic angles).
    pub angles: HouseAngles,
    /// Full Swiss-Ephemeris `ascmc` chart points.
    pub asc_mc: AscMc,
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
    ///
    /// See [`crate::house_for_longitude`] for wraparound semantics and
    /// exact-boundary examples.
    pub fn house_for_longitude(&self, longitude: Longitude) -> usize {
        house_for_longitude(longitude, &self.cusps)
    }

    /// Returns a compact one-line rendering of the calculated house snapshot.
    pub fn summary_line(&self) -> String {
        format!(
            "system={}; instant={}; observer={}; obliquity={}; angles=ASC {}, MC {}, IC {}, DSC {}; cusp-count={}",
            self.system,
            self.instant,
            self.observer,
            self.obliquity,
            self.angles.ascendant,
            self.angles.midheaven,
            self.angles.imum_coeli,
            self.angles.descendant,
            self.cusps.len()
        )
    }

    /// Returns the compact one-line rendering after validating the snapshot.
    pub fn validated_summary_line(&self) -> Result<String, HouseError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Ensures the snapshot contains only finite numeric values and consistent
    /// opposite angle pairs.
    pub fn validate(&self) -> Result<(), HouseError> {
        validate_house_snapshot(self)
    }
}

impl fmt::Display for HouseSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// The number of cusps a house system produces: Gauquelin yields 36 sectors,
/// every other built-in yields the standard 12 cusps.
fn expected_cusp_count(system: &HouseSystem) -> usize {
    match system {
        HouseSystem::Gauquelin => 36,
        _ => 12,
    }
}

/// Computes the house cusps and derived angles for a request.
pub fn calculate_houses(request: &HouseRequest) -> Result<HouseSnapshot, HouseError> {
    let obliquity = validated_obliquity(request)?;

    // High-latitude policy: reject or substitute depending on the request setting.
    if let Some(descriptor) = crate::catalog::descriptor(&request.system) {
        if let Some(bound) = descriptor.max_abs_latitude_deg {
            let lat = request.observer.latitude.degrees();
            if lat.abs() > bound {
                match request.high_latitude_policy {
                    HighLatitudePolicy::Strict => {
                        return Err(HouseError::new(
                            HouseErrorKind::InvalidLatitude,
                            format!(
                                "{} is undefined beyond |latitude| {bound}\u{00b0} (got {lat:.4}\u{00b0})",
                                request.system
                            ),
                        ));
                    }
                    HighLatitudePolicy::SwissEphemerisFallback => {
                        // Porphyry yields 12 quadrant cusps — a valid substitute
                        // only for 12-cusp systems. It cannot represent a
                        // 36-sector system (Gauquelin), and no validated
                        // high-latitude reference exists for one, so reject
                        // rather than emit a dimensionally-invalid snapshot.
                        if expected_cusp_count(&request.system) != 12 {
                            return Err(HouseError::new(
                                HouseErrorKind::InvalidLatitude,
                                format!(
                                    "{} has no Porphyry high-latitude fallback and is undefined beyond |latitude| {bound}\u{00b0} (got {lat:.4}\u{00b0})",
                                    request.system
                                ),
                            ));
                        }
                        let angles = derive_angles(request.instant, &request.observer, obliquity);
                        let asc_mc = asc_mc_from(
                            local_sidereal_time(request.instant, request.observer.longitude)
                                .degrees(),
                            request.observer.latitude.degrees(),
                            obliquity.degrees(),
                        )?;
                        let snapshot = HouseSnapshot {
                            system: request.system.clone(),
                            instant: request.instant,
                            observer: request.observer.clone(),
                            obliquity,
                            angles,
                            asc_mc,
                            cusps: porphyry_houses(angles).into(),
                        };
                        snapshot.validate()?;
                        return Ok(snapshot);
                    }
                }
            }
        }
    }

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
        HouseSystem::Meridian | HouseSystem::Axial => {
            equatorial_projection_houses(request.instant, &request.observer, obliquity).into()
        }
        HouseSystem::Morinus => {
            morinus_houses(request.instant, &request.observer, obliquity).into()
        }
        HouseSystem::Topocentric => {
            topocentric_houses(request.instant, &request.observer, obliquity, angles)?.into()
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

    let asc_mc = asc_mc_from(
        local_sidereal_time(request.instant, request.observer.longitude).degrees(),
        request.observer.latitude.degrees(),
        obliquity.degrees(),
    )?;
    let snapshot = HouseSnapshot {
        system: request.system.clone(),
        instant: request.instant,
        observer: request.observer.clone(),
        obliquity,
        angles,
        asc_mc,
        cusps,
    };
    snapshot.validate()?;

    Ok(snapshot)
}

/// Returns the one-based house number for a longitude and cusp set.
///
/// Cusps are treated as the start of each house, and wraparound at 360° is
/// handled explicitly.
///
/// # Example
///
/// ```
/// use pleiades_houses::house_for_longitude;
/// use pleiades_types::Longitude;
///
/// let cusps = vec![
///     Longitude::from_degrees(0.0),
///     Longitude::from_degrees(30.0),
///     Longitude::from_degrees(60.0),
///     Longitude::from_degrees(90.0),
///     Longitude::from_degrees(120.0),
///     Longitude::from_degrees(150.0),
///     Longitude::from_degrees(180.0),
///     Longitude::from_degrees(210.0),
///     Longitude::from_degrees(240.0),
///     Longitude::from_degrees(270.0),
///     Longitude::from_degrees(300.0),
///     Longitude::from_degrees(330.0),
/// ];
///
/// assert_eq!(house_for_longitude(Longitude::from_degrees(359.0), &cusps), 12);
/// assert_eq!(house_for_longitude(Longitude::from_degrees(0.0), &cusps), 1);
/// assert_eq!(house_for_longitude(Longitude::from_degrees(30.0), &cusps), 2);
/// ```
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
    observer.validate().map_err(|error| match error {
        ObserverLocationValidationError::NonFiniteLatitude { value }
        | ObserverLocationValidationError::LatitudeOutOfRange { value } => HouseError::new(
            HouseErrorKind::InvalidLatitude,
            format!("observer latitude {value}° is outside the valid range"),
        ),
        ObserverLocationValidationError::NonFiniteLongitude { .. } => HouseError::new(
            HouseErrorKind::InvalidLongitude,
            "observer longitude must be finite",
        ),
        ObserverLocationValidationError::NonFiniteElevation { .. } => HouseError::new(
            HouseErrorKind::InvalidElevation,
            "observer elevation must be finite when provided",
        ),
        _ => HouseError::new(
            HouseErrorKind::NumericalFailure,
            "observer location validation failed unexpectedly",
        ),
    })
}

/// Returns `(Δψ_deg, Δε_deg)` for the given instant.
///
/// Maps nutation errors to [`HouseError`] (fail-closed).
fn nutation_for(instant: Instant) -> Result<(f64, f64), HouseError> {
    let jd = instant.julian_day.days();
    let nut = apparent_nutation(jd).map_err(|e| {
        HouseError::new(
            HouseErrorKind::NumericalFailure,
            format!("nutation computation failed: {e:?}"),
        )
    })?;
    Ok((nut.delta_psi_arcsec / 3600.0, nut.delta_eps_arcsec / 3600.0))
}

fn validated_obliquity(request: &HouseRequest) -> Result<Angle, HouseError> {
    validate_observer(&request.observer)?;
    validate_topocentric_observer(request)?;
    let obliquity = match request.obliquity {
        Some(o) => o,
        None => {
            let mean_obl = mean_obliquity(request.instant);
            let (_delta_psi_deg, delta_eps_deg) = nutation_for(request.instant)?;
            Angle::from_degrees(mean_obl.degrees() + delta_eps_deg)
        }
    };
    validate_obliquity(obliquity)
}

fn validate_topocentric_observer(request: &HouseRequest) -> Result<(), HouseError> {
    if matches!(request.system, HouseSystem::Topocentric) {
        topocentric_latitude(
            request.observer.latitude.degrees(),
            request.observer.elevation_m,
        )?;
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

    let expected_cusp_count = expected_cusp_count(&snapshot.system);
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

    if snapshot.angles.descendant != longitude_opposite(snapshot.angles.ascendant) {
        return Err(HouseError::new(
            HouseErrorKind::NumericalFailure,
            format!(
                "house calculation for {} produced a descendant that is not opposite the ascendant",
                snapshot.system
            ),
        ));
    }

    if snapshot.angles.imum_coeli != longitude_opposite(snapshot.angles.midheaven) {
        return Err(HouseError::new(
            HouseErrorKind::NumericalFailure,
            format!(
                "house calculation for {} produced an imum coeli that is not opposite the midheaven",
                snapshot.system
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
        theta
            .cos()
            .atan2(-(theta.sin() * obliquity.cos() + latitude.tan() * obliquity.sin()))
            .to_degrees(),
    );
    let midheaven = Longitude::from_degrees(
        theta
            .sin()
            .atan2(theta.cos() * obliquity.cos())
            .to_degrees(),
    );
    HouseAngles::new(ascendant, midheaven)
}

fn ascendant_for(sidereal_time_deg: f64, latitude_deg: f64, obliquity_rad: f64) -> Longitude {
    let theta = sidereal_time_deg.to_radians();
    let latitude = latitude_deg.to_radians();
    Longitude::from_degrees(
        theta
            .cos()
            .atan2(-(theta.sin() * obliquity_rad.cos() + latitude.tan() * obliquity_rad.sin()))
            .to_degrees(),
    )
}

fn local_sidereal_time(instant: Instant, longitude: Longitude) -> Angle {
    Angle::from_degrees(
        pleiades_apparent::sidereal::sidereal_time(instant, longitude).local_apparent_deg,
    )
}

fn mean_obliquity(instant: Instant) -> Angle {
    Angle::from_degrees(pleiades_apparent::nutation::mean_obliquity_degrees(
        instant.julian_day.days(),
    ))
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

    // Koch ("birthplace" / GOH) house system, following Swiss Ephemeris
    // `swehouse.c` (case 'K'). Each intermediate cusp is the ascendant
    // (`Asc1` projection) computed for the right ascension `ARMC + offset`,
    // where the offset trisects the meridian arc using the ascensional
    // difference `ad` of the Midheaven (`ad3 = ad / 3`).
    let th = local_sidereal_time(instant, observer.longitude).degrees();
    let latitude_deg = observer.latitude.degrees();
    let obliquity_deg = obliquity.degrees();
    let latitude = latitude_deg.to_radians();
    let obliquity = obliquity_deg.to_radians();
    let sine = obliquity.sin();
    let cose = obliquity.cos();

    // Koch is undefined within the polar circle, where the Midheaven's
    // ascensional difference is no longer real. Fail closed rather than
    // silently substituting another system.
    if latitude_deg.abs() >= 90.0 - obliquity_deg {
        return Err(HouseError::new(
            HouseErrorKind::NumericalFailure,
            "koch house system is undefined within the polar circle",
        ));
    }

    // Declination of the Midheaven: sin(decl) = sin(λ_MC) * sin(ε), where
    // λ_MC is the ecliptic longitude of the MC (`angles.midheaven`). `sina`
    // divides this by cos(latitude) (the cosine of the geographic pole height).
    let mc = angles.midheaven.degrees().to_radians();
    let sina = (mc.sin() * sine / latitude.cos()).clamp(-1.0, 1.0);
    let cosa = (1.0 - sina * sina).max(0.0).sqrt();
    let c = (latitude.tan() / cosa).atan();
    let ad3 = (c.sin() * sina).clamp(-1.0, 1.0).asin().to_degrees() / 3.0;

    cusps[10] = asc1(th + 30.0 - 2.0 * ad3, latitude_deg, sine, cose);
    cusps[11] = asc1(th + 60.0 - ad3, latitude_deg, sine, cose);
    cusps[1] = asc1(th + 120.0 + ad3, latitude_deg, sine, cose);
    cusps[2] = asc1(th + 150.0 + 2.0 * ad3, latitude_deg, sine, cose);

    cusps[4] = longitude_opposite(cusps[10]);
    cusps[5] = longitude_opposite(cusps[11]);
    cusps[7] = longitude_opposite(cusps[1]);
    cusps[8] = longitude_opposite(cusps[2]);

    for cusp in &cusps {
        if !cusp.degrees().is_finite() {
            return Err(HouseError::new(
                HouseErrorKind::NumericalFailure,
                "koch house cusp evaluated to a non-finite longitude",
            ));
        }
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

    // Trisect the diurnal semi-arc (RAMC → Ascendant) to place houses 11, 12.
    // House 10 (MC) is already set from `angles.midheaven`; start at k=1.
    cusps[10] = ecliptic_longitude_from_ra(st + diurnal / 3.0, obliquity);
    cusps[11] = ecliptic_longitude_from_ra(st + 2.0 * diurnal / 3.0, obliquity);

    // Trisect the nocturnal semi-arc (Ascendant → Descendant, via IC) to
    // place houses 2, 3.  House 1 (Ascendant) is already set; skip k=0.
    cusps[1] = ecliptic_longitude_from_ra(st + diurnal + nocturnal / 3.0, obliquity);
    cusps[2] = ecliptic_longitude_from_ra(st + diurnal + 2.0 * nocturnal / 3.0, obliquity);

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

        // Campanus divides the prime vertical into 30° arcs. For each division point
        // at angle d along the prime vertical (d=0 ≡ MC/zenith direction, d=90° ≡ East/ASC),
        // the prime-vertical altitude h_pv satisfies sin(h_pv)=cos(d) and cos(h_pv)=sin(d).
        //
        // The Campanus position circle (through the North and South horizon points and the
        // prime-vertical division point) intersects the ecliptic at longitude λ:
        //
        //   y =  cos(d)·sin(θ) + sin(d)·cos(φ)·cos(θ)
        //   x =  cos(d)·cos(θ)·cos(ε) − sin(d)·cos(φ)·sin(θ)·cos(ε) − sin(d)·sin(φ)·sin(ε)
        //   λ = atan2(y, x)
        //
        // where θ = RAMC (local sidereal time), φ = geographic latitude, ε = obliquity.
        let d = house_phase(house).to_radians();
        let y = d.cos() * st.sin() + d.sin() * latitude.cos() * st.cos();
        let x = d.cos() * st.cos() * obliquity.cos()
            - d.sin() * latitude.cos() * st.sin() * obliquity.cos()
            - d.sin() * latitude.sin() * obliquity.sin();
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

/// Morinus house system (Swiss Ephemeris code 'M').
///
/// The Morinus system divides the celestial equator into twelve equal 30° arcs
/// beginning at RA = RAMC + 90° (the IC meridian direction projected onto the
/// equator), then projects each arc endpoint onto the ecliptic using the full
/// spherical rotation from equatorial to ecliptic coordinates for a point on
/// the equator (declination = 0):
///
///   ecliptic longitude = atan2(sin(RA) * cos(eps), cos(RA))
///
/// This is the standard equatorial-to-ecliptic conversion formula for
/// dec = 0. It differs from the Meridian/Axial formula, which uses
/// `atan2(sin(RA), cos(RA) * cos(eps))` (the ecliptic-to-equatorial inverse).
/// Morinus is latitude-independent because only the sidereal time (RAMC)
/// and the obliquity enter the formula.
fn morinus_houses(
    instant: Instant,
    observer: &ObserverLocation,
    obliquity: Angle,
) -> [Longitude; 12] {
    let st = local_sidereal_time(instant, observer.longitude).degrees();
    let eps = obliquity.degrees().to_radians();
    let cos_eps = eps.cos();

    core::array::from_fn(|index| {
        let ra = (st + 90.0 + (index as f64) * 30.0).to_radians();
        Longitude::from_degrees(ra.sin().atan2(ra.cos() / cos_eps).to_degrees())
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
    // Swiss-Ephemeris 'H' (Horizon/Azimuth) — faithful port of CalcH's case 'H'.
    //
    // SE divides the horizon (azimuth circle) into twelve 30° sectors whose
    // boundaries are projected onto the ecliptic via the ascendant of a
    // fictitious tilted horizon. Three details govern the convention and are the
    // historical source of pleiades' ~100° offset:
    //
    //   1. The azimuth origin is `Asc1(th + 90, …)`, where `th = ARMC + 180`.
    //      pleiades' `ascendant_for(s, f)` is exactly `Asc1(s + 90, f)`, so the
    //      SE call `Asc1(th + 90, f)` must be made as `ascendant_for(th, f)` —
    //      passing `th + 90` double-counts the 90° azimuth quarter-turn.
    //   2. SE rotates the five computed cusps (1,2,3,11,12) by +180° afterwards
    //      (`cusp[i] = degnorm(cusp[i] + 180)`); omitting this rotation was the
    //      dominant error.
    //   3. The geographic-latitude → pole-height transform uses a STRICT `> 0`
    //      branch: at the equator SE takes `fi = -90 - lat` (= -90), not
    //      `90 - lat`. A `>= 0` branch flips cusp 1 by 180° at lat 0.
    let sidereal_time =
        (local_sidereal_time(instant, observer.longitude).degrees() + 180.0).rem_euclid(360.0);
    let obliquity = obliquity.degrees().to_radians();
    let latitude = observer.latitude.degrees();
    // SE strict-sign hemisphere branch (note: lat == 0 takes the `else` arm).
    let mut transformed_latitude = if latitude > 0.0 {
        90.0 - latitude
    } else {
        -90.0 - latitude
    };
    // Clamp away from the ±90° pole singularity (SE's VERY_SMALL guard) so the
    // cos(fi) == 0 azimuth branch is unreachable in practice.
    const VERY_SMALL: f64 = 1e-10;
    if (transformed_latitude.abs() - 90.0).abs() < VERY_SMALL {
        transformed_latitude = if transformed_latitude < 0.0 {
            -90.0 + VERY_SMALL
        } else {
            90.0 - VERY_SMALL
        };
    }
    let transformed_latitude_rad = transformed_latitude.to_radians();
    let fh1 = (transformed_latitude_rad.sin() / 2.0).asin().to_degrees();
    let fh2 = ((3.0_f64).sqrt() / 2.0 * transformed_latitude_rad.sin())
        .asin()
        .to_degrees();
    let cosfi = transformed_latitude_rad.cos();
    let (xh1, xh2) = if cosfi == 0.0 {
        if transformed_latitude > 0.0 {
            (90.0, 90.0)
        } else {
            (270.0, 270.0)
        }
    } else {
        (
            (3.0_f64.sqrt() / cosfi).atan().to_degrees(),
            (1.0 / 3.0_f64.sqrt() / cosfi).atan().to_degrees(),
        )
    };

    // `ascendant_for(s, f) == Asc1(s + 90, f)`; `longitude_opposite` applies the
    // SE +180° post-rotation. MC (cusp 10) is the true meridian, untouched.
    let mut cusps = [Longitude::from_degrees(0.0); 12];
    cusps[0] = longitude_opposite(ascendant_for(
        sidereal_time,
        transformed_latitude,
        obliquity,
    ));
    cusps[9] = angles.midheaven;
    cusps[10] = longitude_opposite(ascendant_for(sidereal_time - xh1, fh1, obliquity));
    cusps[11] = longitude_opposite(ascendant_for(sidereal_time - xh2, fh2, obliquity));
    cusps[1] = longitude_opposite(ascendant_for(sidereal_time + xh2, fh2, obliquity));
    cusps[2] = longitude_opposite(ascendant_for(sidereal_time + xh1, fh1, obliquity));
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
    instant: Instant,
    observer: &ObserverLocation,
    obliquity: Angle,
    angles: HouseAngles,
) -> Result<[Longitude; 36], HouseError> {
    // Swiss-Ephemeris computes the 36 Gauquelin sectors as a Placidus-family
    // semi-arc division: each diurnal quadrant of the day's apparent motion is
    // split into ninths of the relevant semi-arc (in hour angle), then each
    // boundary is projected back to ecliptic longitude — NOT a longitude lerp.
    //
    // Convention verified against the SE reference (sectors.csv, equator → 66°):
    //   * Anchors are the four chart angles directly:
    //       s1  = Ascendant, s10 = Midheaven, s19 = Descendant, s28 = Imum Coeli.
    //   * The upper hemisphere (s1..s18) is solved via the diurnal semi-arc:
    //       - s2..s9  lie between the ASC and MC, east of the upper meridian
    //         (positive hour angle), at fractions f = (10 − k)/9 of each point's
    //         own semi-diurnal arc.
    //       - s11..s18 lie between the MC and DSC, west of the upper meridian
    //         (negative hour angle), at fractions f = (k − 10)/9.
    //   * The lower hemisphere is the exact antipode of the upper one:
    //       s(k + 18) = opposite(s(k)). The SE reference confirms this to 0″,
    //       so the nocturnal quadrants need no separate solve.
    //
    // The semi-arc solve mirrors `solve_placidian_cusp` (cos(q/f) =
    // −tan φ·tan δ with α = RAMC + q, tan δ = sin α·tan ε), generalized from
    // thirds to an arbitrary fraction so the same Newton iteration places every
    // intermediate sector boundary.
    let mut cusps = [Longitude::from_degrees(0.0); 36];

    let ramc = local_sidereal_time(instant, observer.longitude).degrees();
    let latitude = observer.latitude.degrees();
    let obliquity_deg = obliquity.degrees();

    // Anchors come straight from the derived angles.
    cusps[0] = angles.ascendant; // s1  = ASC
    cusps[9] = angles.midheaven; // s10 = MC

    // Intermediate sectors of the upper hemisphere, by ninths of the semi-arc.
    for k in 2..=9 {
        let fraction = (10 - k) as f64 / 9.0;
        cusps[k - 1] = solve_gauquelin_sector(ramc, latitude, obliquity_deg, fraction, 1.0)?;
    }
    for k in 11..=18 {
        let fraction = (k - 10) as f64 / 9.0;
        cusps[k - 1] = solve_gauquelin_sector(ramc, latitude, obliquity_deg, fraction, -1.0)?;
    }

    // Lower hemisphere is the exact antipode of the upper hemisphere.
    for k in 1..=18 {
        cusps[k + 18 - 1] = longitude_opposite(cusps[k - 1]);
    }

    Ok(cusps)
}

/// Solves one Gauquelin sector boundary as a fraction `f` of an ecliptic
/// point's semi-diurnal arc, `sign` selecting the side of the upper meridian
/// (`+1` east, `−1` west). This is `solve_placidian_cusp`'s semi-arc condition
/// `cos(q/f) = −tan φ·tan δ` (with `α = RAMC + q`, `tan δ = sin α·tan ε`)
/// generalized from fixed thirds to an arbitrary fraction.
fn solve_gauquelin_sector(
    ramc_deg: f64,
    latitude_deg: f64,
    obliquity_deg: f64,
    fraction: f64,
    sign: f64,
) -> Result<Longitude, HouseError> {
    let latitude = latitude_deg.to_radians();
    let obliquity = obliquity_deg.to_radians();
    let tan_lat = latitude.tan();
    let tan_obliquity = obliquity.tan();
    let deg_per_rad = 180.0 / core::f64::consts::PI;

    let mut q = sign * fraction * 90.0;
    let mut converged = false;
    for _ in 0..64 {
        let alpha = ramc_deg + q;
        let alpha_rad = alpha.to_radians();
        let tan_delta = alpha_rad.sin() * tan_obliquity;
        let arg = (q / fraction).to_radians();
        let g = arg.cos() + tan_lat * tan_delta;
        let gp = (-(1.0 / fraction) * arg.sin() + tan_lat * tan_obliquity * alpha_rad.cos())
            / deg_per_rad;
        if gp.abs() < 1.0e-12 {
            return Err(HouseError::new(
                HouseErrorKind::NumericalFailure,
                "gauquelin sector iteration encountered a zero derivative",
            ));
        }
        let delta = -g / gp;
        q += delta;
        if delta.abs() < 1.0e-9 {
            converged = true;
            break;
        }
    }

    if !converged || !q.is_finite() {
        return Err(HouseError::new(
            HouseErrorKind::NumericalFailure,
            "gauquelin sector iteration failed to converge",
        ));
    }

    Ok(ecliptic_longitude_from_ra(ramc_deg + q, obliquity))
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
    angles: HouseAngles,
) -> Result<[Longitude; 12], HouseError> {
    // Topocentric (Polich-Page) houses. The angles (cusps 1/4/7/10) are the
    // already-validated Ascendant/MC pair; only the intermediate cusps differ
    // from Placidus. Each intermediate cusp is projected with `asc1` using a
    // "house pole" whose tangent is a third (or two thirds) of tan(latitude):
    //
    //   tan(pole_n) = (n / 3) * tan(latitude)
    //
    // and an equatorial offset from the RAMC of 30 degrees per third. This is
    // the Polich-Page trisection that Swiss Ephemeris implements for the
    // Topocentric ('T') system. It is independent of the geodetic-to-geocentric
    // latitude correction used elsewhere for diurnal parallax.
    let mut cusps = [Longitude::from_degrees(0.0); 12];
    cusps[0] = angles.ascendant;
    cusps[3] = angles.imum_coeli;
    cusps[6] = angles.descendant;
    cusps[9] = angles.midheaven;

    let st = local_sidereal_time(instant, observer.longitude).degrees();
    let tan_latitude = observer.latitude.degrees().to_radians().tan();
    let obliquity = obliquity.degrees().to_radians();
    let sine = obliquity.sin();
    let cose = obliquity.cos();

    // (house index, RAMC offset in degrees, fraction of tan(latitude)).
    // Only the four cusps adjacent to the meridian are solved directly; cusps
    // 5/6/8/9 are their ecliptic antipodes.
    const SPEC: [(usize, f64, f64); 4] = [
        (11, 30.0, 1.0 / 3.0),
        (12, 60.0, 2.0 / 3.0),
        (2, 120.0, 2.0 / 3.0),
        (3, 150.0, 1.0 / 3.0),
    ];

    for (house, offset, fraction) in SPEC {
        let pole = (fraction * tan_latitude).atan().to_degrees();
        let cusp = asc1(st + offset, pole, sine, cose);
        if !cusp.degrees().is_finite() {
            return Err(HouseError::new(
                HouseErrorKind::NumericalFailure,
                "topocentric cusp projection produced a non-finite longitude",
            ));
        }
        cusps[house - 1] = cusp;
    }

    cusps[4] = longitude_opposite(cusps[10]);
    cusps[5] = longitude_opposite(cusps[11]);
    cusps[7] = longitude_opposite(cusps[1]);
    cusps[8] = longitude_opposite(cusps[2]);

    Ok(cusps)
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

pub(crate) fn topocentric_latitude(
    latitude_deg: f64,
    elevation_m: Option<f64>,
) -> Result<Angle, HouseError> {
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
    // Placidus divides each ecliptic point's diurnal/nocturnal semi-arc into
    // thirds by hour-angle. A cusp lies where the point's meridian distance `q`
    // (the hour angle from the meridian, in degrees) equals a fraction `f` of
    // its own semi-arc. The semi-diurnal arc satisfies
    //   cos(semi_arc) = -tan(phi) * tan(delta),
    // and the cusp condition `q = f * semi_arc` therefore reduces to
    //   cos(q / f) = -tan(phi) * tan(delta).
    // For an ecliptic point of right ascension `alpha = RAMC + q`, the
    // declination follows from the obliquity via tan(delta) = sin(alpha)*tan(eps).
    //
    // Houses 11 and 12 sit east of the upper meridian (positive q) using
    // fractions 1/3 and 2/3 of the semi-diurnal arc. Houses 2 and 3 are the
    // antipodes of the symmetric west-of-meridian points (negative q) using
    // fractions 2/3 and 1/3, so they share the same solver and are reflected
    // through the opposite ecliptic longitude.
    let (fraction, sign, opposite) = match house {
        11 => (1.0 / 3.0, 1.0, false),
        12 => (2.0 / 3.0, 1.0, false),
        2 => (2.0 / 3.0, -1.0, true),
        3 => (1.0 / 3.0, -1.0, true),
        _ => {
            return Err(HouseError::new(
                HouseErrorKind::UnsupportedHouseSystem,
                format!("invalid placidian house {}", house),
            ))
        }
    };

    let latitude = latitude_deg.to_radians();
    let obliquity = obliquity_deg.to_radians();
    let tan_lat = latitude.tan();
    let tan_obliquity = obliquity.tan();
    let deg_per_rad = 180.0 / core::f64::consts::PI;

    // Residual g(q) = cos(q/f) + tan(phi)*tan(delta(alpha)), with alpha = st + q.
    // Solve g(q) = 0 with Newton iteration, seeded toward the correct quadrant.
    let mut q = sign * fraction * 90.0;

    let mut converged = false;
    for _ in 0..64 {
        let alpha = st_deg + q;
        let alpha_rad = alpha.to_radians();
        let tan_delta = alpha_rad.sin() * tan_obliquity;
        let arg = (q / fraction).to_radians();
        let g = arg.cos() + tan_lat * tan_delta;
        // dg/dq (per degree): derivative of cos(q/f) is -(1/f)*sin(q/f)*(pi/180);
        // derivative of tan(delta) term is tan(phi)*tan(eps)*cos(alpha)*(pi/180).
        let gp = (-(1.0 / fraction) * arg.sin() + tan_lat * tan_obliquity * alpha_rad.cos())
            / deg_per_rad;
        if gp.abs() < 1.0e-12 {
            return Err(HouseError::new(
                HouseErrorKind::NumericalFailure,
                "placidian cusp iteration encountered a zero derivative",
            ));
        }

        let delta = -g / gp;
        q += delta;
        if delta.abs() < 1.0e-9 {
            converged = true;
            break;
        }
    }

    if !converged || !q.is_finite() {
        return Err(HouseError::new(
            HouseErrorKind::NumericalFailure,
            "placidian cusp iteration failed to converge",
        ));
    }

    let ra = st_deg + q;
    let lon = ecliptic_longitude_from_ra(ra, obliquity);
    Ok(if opposite {
        longitude_opposite(lon)
    } else {
        lon
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
mod tests;
