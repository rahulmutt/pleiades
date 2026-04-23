//! Chart assembly and higher-level chart helpers built on top of backend position queries.
//!
//! The first chart MVP keeps the workflow intentionally small: callers provide
//! a set of bodies, the façade queries the backend, and the result captures the
//! body placements plus their zodiac signs. House placement can be requested
//! explicitly for chart-aware consumers, which keeps the workflow practical
//! without hardwiring more chart logic than the façade needs.

use core::fmt;

use pleiades_ayanamsa::sidereal_offset;
use pleiades_backend::{
    Apparentness, EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest,
    EphemerisResult,
};
use pleiades_houses::{calculate_houses, house_for_longitude, HouseRequest, HouseSnapshot};
use pleiades_types::{
    Angle, CelestialBody, HouseSystem, Instant, Longitude, MotionDirection, ObserverLocation,
    ZodiacMode, ZodiacSign,
};

use crate::ChartEngine;

/// Default bodies included in a basic chart report.
pub const fn default_chart_bodies() -> &'static [CelestialBody] {
    &[
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mercury,
        CelestialBody::Venus,
        CelestialBody::Mars,
        CelestialBody::Jupiter,
        CelestialBody::Saturn,
        CelestialBody::Uranus,
        CelestialBody::Neptune,
        CelestialBody::Pluto,
    ]
}

/// A request for chart assembly.
#[derive(Clone, Debug, PartialEq)]
pub struct ChartRequest {
    /// The instant to chart.
    pub instant: Instant,
    /// Optional observer location.
    pub observer: Option<ObserverLocation>,
    /// Bodies to include in the chart report.
    pub bodies: Vec<CelestialBody>,
    /// Desired zodiac mode.
    pub zodiac_mode: ZodiacMode,
    /// Preferred apparentness for backend position queries.
    pub apparentness: Apparentness,
    /// Optional house-system request.
    pub house_system: Option<HouseSystem>,
}

impl ChartRequest {
    /// Creates a default tropical chart request.
    pub fn new(instant: Instant) -> Self {
        Self {
            instant,
            observer: None,
            bodies: default_chart_bodies().to_vec(),
            zodiac_mode: ZodiacMode::Tropical,
            apparentness: Apparentness::Mean,
            house_system: None,
        }
    }

    /// Sets the observer location.
    pub fn with_observer(mut self, observer: ObserverLocation) -> Self {
        self.observer = Some(observer);
        self
    }

    /// Sets the included bodies.
    pub fn with_bodies(mut self, bodies: Vec<CelestialBody>) -> Self {
        self.bodies = bodies;
        self
    }

    /// Sets the zodiac mode.
    pub fn with_zodiac_mode(mut self, zodiac_mode: ZodiacMode) -> Self {
        self.zodiac_mode = zodiac_mode;
        self
    }

    /// Sets the preferred apparentness.
    pub fn with_apparentness(mut self, apparentness: Apparentness) -> Self {
        self.apparentness = apparentness;
        self
    }

    /// Sets the house system.
    pub fn with_house_system(mut self, house_system: HouseSystem) -> Self {
        self.house_system = Some(house_system);
        self
    }
}

/// A single body placement within a chart.
#[derive(Clone, Debug, PartialEq)]
pub struct BodyPlacement {
    /// The queried body.
    pub body: CelestialBody,
    /// The raw backend result.
    pub position: EphemerisResult,
    /// The body’s zodiac sign in the requested mode, when ecliptic longitude is available.
    pub sign: Option<ZodiacSign>,
    /// The one-based house number, when house placement was requested.
    pub house: Option<usize>,
}

/// A chart snapshot suitable for user-facing reports.
#[derive(Clone, Debug, PartialEq)]
pub struct ChartSnapshot {
    /// Backend that produced the chart.
    pub backend_id: crate::BackendId,
    /// Instant that was charted.
    pub instant: Instant,
    /// Optional observer location.
    pub observer: Option<ObserverLocation>,
    /// Zodiac mode used for the chart.
    pub zodiac_mode: ZodiacMode,
    /// Optional house snapshot.
    pub houses: Option<HouseSnapshot>,
    /// Ordered body placements.
    pub placements: Vec<BodyPlacement>,
}

impl ChartSnapshot {
    /// Returns the number of body placements in the chart.
    pub fn len(&self) -> usize {
        self.placements.len()
    }

    /// Returns `true` if the chart contains no body placements.
    pub fn is_empty(&self) -> bool {
        self.placements.is_empty()
    }

    /// Returns the first placement for a requested body, if present.
    pub fn placement_for(&self, body: &CelestialBody) -> Option<&BodyPlacement> {
        self.placements
            .iter()
            .find(|placement| &placement.body == body)
    }

    /// Returns the motion direction for a requested body, if it is known.
    pub fn motion_direction_for(&self, body: &CelestialBody) -> Option<MotionDirection> {
        self.placement_for(body)?.motion_direction()
    }

    /// Returns the sign for a requested body, if sign placement was computed.
    pub fn sign_for_body(&self, body: &CelestialBody) -> Option<ZodiacSign> {
        self.placement_for(body)?.sign
    }

    /// Returns the house number for a requested body, if house placement was computed.
    pub fn house_for_body(&self, body: &CelestialBody) -> Option<usize> {
        self.placement_for(body)?.house
    }

    /// Returns the angular separation between two bodies when both have ecliptic longitudes.
    pub fn angular_separation(&self, left: &CelestialBody, right: &CelestialBody) -> Option<Angle> {
        let left = self
            .placement_for(left)?
            .position
            .ecliptic
            .as_ref()?
            .longitude;
        let right = self
            .placement_for(right)?
            .position
            .ecliptic
            .as_ref()?
            .longitude;
        Some(angle_separation(left, right))
    }

    /// Returns all placements assigned to a requested zodiac sign.
    pub fn placements_in_sign(&self, sign: ZodiacSign) -> impl Iterator<Item = &BodyPlacement> {
        self.placements
            .iter()
            .filter(move |placement| placement.sign == Some(sign))
    }

    /// Returns all placements assigned to a requested house number.
    pub fn placements_in_house(&self, house: usize) -> impl Iterator<Item = &BodyPlacement> {
        self.placements
            .iter()
            .filter(move |placement| placement.house == Some(house))
    }

    /// Returns the placements that are currently classified as retrograde.
    pub fn retrograde_placements(&self) -> impl Iterator<Item = &BodyPlacement> {
        self.placements.iter().filter(|placement| {
            matches!(
                placement.motion_direction(),
                Some(MotionDirection::Retrograde)
            )
        })
    }

    /// Returns the placements that are currently classified as stationary.
    pub fn stationary_placements(&self) -> impl Iterator<Item = &BodyPlacement> {
        self.placements.iter().filter(|placement| {
            matches!(
                placement.motion_direction(),
                Some(MotionDirection::Stationary)
            )
        })
    }

    /// Returns the placements that are currently classified with a requested motion direction.
    pub fn placements_with_motion_direction(
        &self,
        direction: MotionDirection,
    ) -> impl Iterator<Item = &BodyPlacement> {
        self.placements
            .iter()
            .filter(move |placement| placement.motion_direction() == Some(direction))
    }

    /// Returns the placements that are currently classified as direct.
    pub fn direct_placements(&self) -> impl Iterator<Item = &BodyPlacement> {
        self.placements_with_motion_direction(MotionDirection::Direct)
    }

    /// Returns a summary of the chart's motion-direction classifications.
    pub fn motion_summary(&self) -> MotionSummary {
        let mut summary = MotionSummary::default();

        for placement in &self.placements {
            match placement.motion_direction() {
                Some(MotionDirection::Direct) => summary.direct += 1,
                Some(MotionDirection::Stationary) => summary.stationary += 1,
                Some(MotionDirection::Retrograde) => summary.retrograde += 1,
                None => summary.unknown += 1,
            }
        }

        summary
    }

    /// Returns the major aspects found in the chart using the built-in orb defaults.
    pub fn major_aspects(&self) -> Vec<AspectMatch> {
        self.aspects_with_definitions(default_aspect_definitions())
    }

    /// Returns aspect matches using custom aspect definitions.
    pub fn aspects_with_definitions(&self, definitions: &[AspectDefinition]) -> Vec<AspectMatch> {
        let mut matches = Vec::new();

        for (left_index, left) in self.placements.iter().enumerate() {
            let Some(left_longitude) = left
                .position
                .ecliptic
                .as_ref()
                .map(|coords| coords.longitude)
            else {
                continue;
            };

            for right in self.placements.iter().skip(left_index + 1) {
                let Some(right_longitude) = right
                    .position
                    .ecliptic
                    .as_ref()
                    .map(|coords| coords.longitude)
                else {
                    continue;
                };

                let separation = angle_separation(left_longitude, right_longitude);
                if let Some(definition) = best_aspect_definition(separation, definitions) {
                    matches.push(AspectMatch {
                        left: left.body.clone(),
                        right: right.body.clone(),
                        kind: definition.kind,
                        separation,
                        orb: Angle::from_degrees(
                            (separation.degrees() - definition.exact_degrees).abs(),
                        ),
                    });
                }
            }
        }

        matches
    }
}

impl BodyPlacement {
    /// Returns the coarse direction of longitudinal motion when the backend supplied motion data.
    pub fn motion_direction(&self) -> Option<MotionDirection> {
        self.position.motion.as_ref()?.longitude_direction()
    }
}

/// A summary of motion-direction classifications in a chart snapshot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MotionSummary {
    /// Placements classified as direct.
    pub direct: usize,
    /// Placements classified as stationary.
    pub stationary: usize,
    /// Placements classified as retrograde.
    pub retrograde: usize,
    /// Placements without enough motion data to classify.
    pub unknown: usize,
}

impl MotionSummary {
    /// Returns `true` when the snapshot contains at least one known motion classification.
    pub fn has_known_motion(self) -> bool {
        self.direct + self.stationary + self.retrograde > 0
    }
}

/// A major aspect family in the chart façade.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AspectKind {
    /// Bodies are aligned near the same longitude.
    Conjunction,
    /// Bodies are separated by roughly 60 degrees.
    Sextile,
    /// Bodies are separated by roughly 90 degrees.
    Square,
    /// Bodies are separated by roughly 120 degrees.
    Trine,
    /// Bodies are separated by roughly 180 degrees.
    Opposition,
}

impl AspectKind {
    /// Returns the canonical display name for the aspect family.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Conjunction => "Conjunction",
            Self::Sextile => "Sextile",
            Self::Square => "Square",
            Self::Trine => "Trine",
            Self::Opposition => "Opposition",
        }
    }
}

impl fmt::Display for AspectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// An aspect definition used for configurable matching.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectDefinition {
    /// The aspect family.
    pub kind: AspectKind,
    /// The exact angular separation that defines the aspect.
    pub exact_degrees: f64,
    /// The maximum allowed orb in degrees.
    pub orb_degrees: f64,
}

impl AspectDefinition {
    /// Creates a new aspect definition.
    pub const fn new(kind: AspectKind, exact_degrees: f64, orb_degrees: f64) -> Self {
        Self {
            kind,
            exact_degrees,
            orb_degrees,
        }
    }
}

/// A matched aspect between two bodies.
#[derive(Clone, Debug, PartialEq)]
pub struct AspectMatch {
    /// The body on the left side of the pairing.
    pub left: CelestialBody,
    /// The body on the right side of the pairing.
    pub right: CelestialBody,
    /// The matched aspect family.
    pub kind: AspectKind,
    /// The measured angular separation between the bodies.
    pub separation: Angle,
    /// The absolute orb from the exact aspect angle.
    pub orb: Angle,
}

impl fmt::Display for AspectMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} (separation: {}, orb: {})",
            self.left, self.kind, self.right, self.separation, self.orb
        )
    }
}

impl fmt::Display for ChartSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Backend: {}", self.backend_id)?;
        writeln!(
            f,
            "Instant: {} ({:?})",
            self.instant.julian_day, self.instant.scale
        )?;
        if let Some(observer) = &self.observer {
            writeln!(
                f,
                "Observer: {} / {}",
                observer.latitude, observer.longitude
            )?;
        }
        writeln!(f, "Zodiac mode: {:?}", self.zodiac_mode)?;
        if let Some(houses) = &self.houses {
            let house_name = crate::house_system_descriptor(&houses.system)
                .map(|descriptor| descriptor.canonical_name)
                .unwrap_or("Custom");
            writeln!(f, "House system: {}", house_name)?;
            writeln!(f, "House cusps:")?;
            for (index, cusp) in houses.cusps.iter().enumerate() {
                writeln!(f, "  - {:>2}: {}", index + 1, cusp)?;
            }
        }
        writeln!(f, "Bodies:")?;
        for placement in &self.placements {
            let longitude = placement
                .position
                .ecliptic
                .as_ref()
                .map(|coords| format!("{}", coords.longitude))
                .unwrap_or_else(|| "n/a".to_string());
            let sign = placement
                .sign
                .map(|sign| sign.to_string())
                .unwrap_or_else(|| "n/a".to_string());
            let house = placement
                .house
                .map(|house| house.to_string())
                .unwrap_or_else(|| "n/a".to_string());
            let motion = placement
                .motion_direction()
                .map(|direction| direction.to_string())
                .unwrap_or_else(|| "n/a".to_string());
            writeln!(
                f,
                "  - {:<12} {:>9}  {:<10}  {:>3}  {:<10}  {:?}",
                placement.body, longitude, sign, house, motion, placement.position.quality,
            )?;
        }

        let motion_summary = self.motion_summary();
        if motion_summary.has_known_motion() || motion_summary.unknown > 0 {
            writeln!(
                f,
                "Motion summary: {} direct, {} stationary, {} retrograde, {} unknown",
                motion_summary.direct,
                motion_summary.stationary,
                motion_summary.retrograde,
                motion_summary.unknown,
            )?;
        }

        let stationary_bodies: Vec<String> = self
            .stationary_placements()
            .map(|placement| placement.body.to_string())
            .collect();
        if !stationary_bodies.is_empty() {
            writeln!(f, "Stationary bodies: {}", stationary_bodies.join(", "))?;
        }

        let retrograde_bodies: Vec<String> = self
            .retrograde_placements()
            .map(|placement| placement.body.to_string())
            .collect();
        if !retrograde_bodies.is_empty() {
            writeln!(f, "Retrograde bodies: {}", retrograde_bodies.join(", "))?;
        }

        let aspects = self.major_aspects();
        if !aspects.is_empty() {
            writeln!(f, "Aspects:")?;
            for aspect in aspects {
                writeln!(f, "  - {}", aspect)?;
            }
        }

        Ok(())
    }
}

const DEFAULT_ASPECT_DEFINITIONS: [AspectDefinition; 5] = [
    AspectDefinition::new(AspectKind::Conjunction, 0.0, 8.0),
    AspectDefinition::new(AspectKind::Sextile, 60.0, 4.0),
    AspectDefinition::new(AspectKind::Square, 90.0, 6.0),
    AspectDefinition::new(AspectKind::Trine, 120.0, 6.0),
    AspectDefinition::new(AspectKind::Opposition, 180.0, 8.0),
];

fn default_aspect_definitions() -> &'static [AspectDefinition] {
    &DEFAULT_ASPECT_DEFINITIONS
}

fn angle_separation(left: Longitude, right: Longitude) -> Angle {
    let difference = (right.degrees() - left.degrees()).rem_euclid(360.0);
    let separation = if difference > 180.0 {
        360.0 - difference
    } else {
        difference
    };
    Angle::from_degrees(separation)
}

fn best_aspect_definition(
    separation: Angle,
    definitions: &[AspectDefinition],
) -> Option<AspectDefinition> {
    definitions
        .iter()
        .copied()
        .filter(|definition| {
            (separation.degrees() - definition.exact_degrees).abs() <= definition.orb_degrees
        })
        .min_by(|left, right| {
            let left_orb = (separation.degrees() - left.exact_degrees).abs();
            let right_orb = (separation.degrees() - right.exact_degrees).abs();
            left_orb
                .partial_cmp(&right_orb)
                .unwrap_or(core::cmp::Ordering::Equal)
        })
}

/// Converts a tropical longitude into the requested zodiac mode.
///
/// Tropical mode returns the input unchanged. Sidereal mode subtracts the
/// resolved ayanamsa for the provided instant.
pub fn sidereal_longitude(
    longitude: Longitude,
    instant: Instant,
    zodiac_mode: &ZodiacMode,
) -> Result<Longitude, EphemerisError> {
    match zodiac_mode {
        ZodiacMode::Tropical => Ok(longitude),
        ZodiacMode::Sidereal { ayanamsa } => {
            let offset = sidereal_offset(ayanamsa, instant).ok_or_else(|| {
                EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "sidereal conversion requires an ayanamsa with reference offset metadata",
                )
            })?;
            Ok(Longitude::from_degrees(
                longitude.degrees() - offset.degrees(),
            ))
        }
        _ => Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            "unsupported zodiac mode",
        )),
    }
}

fn map_house_error(error: pleiades_houses::HouseError) -> EphemerisError {
    let kind = match error.kind {
        pleiades_houses::HouseErrorKind::UnsupportedHouseSystem => {
            EphemerisErrorKind::InvalidRequest
        }
        pleiades_houses::HouseErrorKind::InvalidLatitude => EphemerisErrorKind::InvalidObserver,
        pleiades_houses::HouseErrorKind::NumericalFailure => EphemerisErrorKind::NumericalFailure,
        _ => EphemerisErrorKind::InvalidRequest,
    };

    EphemerisError::new(kind, error.message)
}

impl<B: EphemerisBackend> ChartEngine<B> {
    /// Assembles a basic chart snapshot from the backend.
    pub fn chart(&self, request: &ChartRequest) -> Result<ChartSnapshot, EphemerisError> {
        let metadata = self.backend.metadata();
        let backend_id = metadata.id.clone();
        let native_sidereal = matches!(request.zodiac_mode, ZodiacMode::Sidereal { .. })
            && metadata.capabilities.native_sidereal;
        let backend_zodiac_mode = if native_sidereal {
            request.zodiac_mode.clone()
        } else {
            ZodiacMode::Tropical
        };

        let houses = if let Some(system) = &request.house_system {
            let observer = request.observer.clone().ok_or_else(|| {
                EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "house placement requires an observer location",
                )
            })?;

            let house_request = HouseRequest::new(request.instant, observer, system.clone());
            let mut snapshot = calculate_houses(&house_request).map_err(map_house_error)?;

            if matches!(request.zodiac_mode, ZodiacMode::Sidereal { .. }) {
                for cusp in &mut snapshot.cusps {
                    *cusp = sidereal_longitude(*cusp, request.instant, &request.zodiac_mode)?;
                }
                snapshot.angles.ascendant = sidereal_longitude(
                    snapshot.angles.ascendant,
                    request.instant,
                    &request.zodiac_mode,
                )?;
                snapshot.angles.descendant = sidereal_longitude(
                    snapshot.angles.descendant,
                    request.instant,
                    &request.zodiac_mode,
                )?;
                snapshot.angles.midheaven = sidereal_longitude(
                    snapshot.angles.midheaven,
                    request.instant,
                    &request.zodiac_mode,
                )?;
                snapshot.angles.imum_coeli = sidereal_longitude(
                    snapshot.angles.imum_coeli,
                    request.instant,
                    &request.zodiac_mode,
                )?;
            }

            Some(snapshot)
        } else {
            None
        };

        let placements = request
            .bodies
            .iter()
            .map(|body| {
                let body_request = EphemerisRequest {
                    body: body.clone(),
                    instant: request.instant,
                    observer: request.observer.clone(),
                    frame: pleiades_types::CoordinateFrame::Ecliptic,
                    zodiac_mode: backend_zodiac_mode.clone(),
                    apparent: request.apparentness,
                };
                let mut position = self.backend.position(&body_request)?;
                let sign = if matches!(request.zodiac_mode, ZodiacMode::Sidereal { .. })
                    && !native_sidereal
                {
                    let instant = position.instant;
                    let longitude = position.ecliptic.as_mut().map(|coords| &mut coords.longitude);
                    let longitude = longitude.ok_or_else(|| {
                        EphemerisError::new(
                            EphemerisErrorKind::InvalidRequest,
                            "sidereal chart assembly requires ecliptic coordinates from the backend",
                        )
                    })?;
                    *longitude = sidereal_longitude(*longitude, instant, &request.zodiac_mode)?;
                    Some(ZodiacSign::from_longitude(*longitude))
                } else {
                    position
                        .ecliptic
                        .as_ref()
                        .map(|coords| ZodiacSign::from_longitude(coords.longitude))
                };
                if matches!(request.zodiac_mode, ZodiacMode::Sidereal { .. }) {
                    position.zodiac_mode = request.zodiac_mode.clone();
                }
                let house = houses.as_ref().and_then(|snapshot| {
                    position
                        .ecliptic
                        .as_ref()
                        .map(|coords| house_for_longitude(coords.longitude, &snapshot.cusps))
                });
                Ok(BodyPlacement {
                    body: body.clone(),
                    position,
                    sign,
                    house,
                })
            })
            .collect::<Result<Vec<_>, EphemerisError>>()?;

        Ok(ChartSnapshot {
            backend_id,
            instant: request.instant,
            observer: request.observer.clone(),
            zodiac_mode: request.zodiac_mode.clone(),
            houses,
            placements,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::{
        AccuracyClass, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
        BackendProvenance, QualityAnnotation,
    };
    use pleiades_types::{EclipticCoordinates, Latitude, Longitude, TimeScale};

    struct ToyChartBackend;

    impl EphemerisBackend for ToyChartBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("toy-chart"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("toy chart backend"),
                nominal_range: pleiades_types::TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_coverage: vec![CelestialBody::Sun, CelestialBody::Moon],
                supported_frames: vec![pleiades_types::CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities::default(),
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            matches!(body, CelestialBody::Sun | CelestialBody::Moon)
        }

        fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            let longitude = match request.body {
                CelestialBody::Sun => Longitude::from_degrees(15.0),
                CelestialBody::Moon => Longitude::from_degrees(45.0),
                _ => {
                    return Err(EphemerisError::new(
                        EphemerisErrorKind::UnsupportedBody,
                        "unsupported",
                    ))
                }
            };
            let mut result = EphemerisResult::new(
                BackendId::new("toy-chart"),
                request.body.clone(),
                request.instant,
                request.frame,
                request.zodiac_mode.clone(),
                request.apparent,
            );
            result.quality = QualityAnnotation::Approximate;
            result.ecliptic = Some(EclipticCoordinates::new(
                longitude,
                Latitude::from_degrees(0.0),
                Some(1.0),
            ));
            Ok(result)
        }
    }

    #[test]
    fn default_body_list_contains_the_luminaries() {
        let bodies = default_chart_bodies();
        assert!(bodies.contains(&CelestialBody::Sun));
        assert!(bodies.contains(&CelestialBody::Moon));
    }

    #[test]
    fn sidereal_longitude_applies_ayanamsa() {
        let instant = Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        );
        let sidereal = sidereal_longitude(
            Longitude::from_degrees(5.0),
            instant,
            &ZodiacMode::Sidereal {
                ayanamsa: crate::Ayanamsa::Lahiri,
            },
        )
        .expect("sidereal conversion should work");

        assert_eq!(ZodiacSign::from_longitude(sidereal), ZodiacSign::Pisces);
    }

    #[test]
    fn chart_snapshot_assigns_signs() {
        let engine = ChartEngine::new(ToyChartBackend);
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        ))
        .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon]);

        let chart = engine.chart(&request).expect("chart should render");
        assert_eq!(chart.backend_id.as_str(), "toy-chart");
        assert_eq!(chart.len(), 2);
        assert_eq!(chart.placements[0].sign, Some(ZodiacSign::Aries));
        assert_eq!(chart.placements[1].sign, Some(ZodiacSign::Taurus));
        let rendered = chart.to_string();
        assert!(rendered.contains("Sun"));
        assert!(rendered.contains("Moon"));
    }

    #[test]
    fn chart_snapshot_supports_sidereal_signs() {
        let engine = ChartEngine::new(ToyChartBackend);
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        ))
        .with_bodies(vec![CelestialBody::Sun])
        .with_zodiac_mode(ZodiacMode::Sidereal {
            ayanamsa: crate::Ayanamsa::Lahiri,
        });

        let chart = engine
            .chart(&request)
            .expect("sidereal chart should render");
        assert_eq!(chart.zodiac_mode, request.zodiac_mode);
        assert_eq!(
            chart.placements[0].position.zodiac_mode,
            request.zodiac_mode
        );
        assert_eq!(chart.placements[0].sign, Some(ZodiacSign::Pisces));
    }

    #[test]
    fn chart_snapshot_preserves_apparentness_choice() {
        let engine = ChartEngine::new(ToyChartBackend);
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        ))
        .with_bodies(vec![CelestialBody::Sun])
        .with_apparentness(Apparentness::Apparent);

        let chart = engine
            .chart(&request)
            .expect("apparent chart should render");
        assert_eq!(
            chart.placements[0].position.apparent,
            Apparentness::Apparent
        );
    }

    #[test]
    fn chart_snapshot_supports_house_placement() {
        let engine = ChartEngine::new(ToyChartBackend);
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        ))
        .with_observer(pleiades_types::ObserverLocation::new(
            Latitude::from_degrees(0.0),
            Longitude::from_degrees(0.0),
            None,
        ))
        .with_house_system(crate::HouseSystem::WholeSign)
        .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon]);

        let chart = engine
            .chart(&request)
            .expect("house-aware chart should render");
        assert!(chart.houses.is_some());
        assert_eq!(chart.placements.len(), 2);
        assert!(chart
            .placements
            .iter()
            .all(|placement| placement.house.is_some()));
        assert!(chart.to_string().contains("House system: Whole Sign"));
    }

    #[test]
    fn body_placement_exposes_motion_direction() {
        let mut result = EphemerisResult::new(
            BackendId::new("toy-chart"),
            CelestialBody::Mars,
            Instant::new(
                pleiades_types::JulianDay::from_days(2451545.0),
                TimeScale::Tt,
            ),
            pleiades_types::CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Apparent,
        );
        result.motion = Some(pleiades_types::Motion::new(Some(-0.012), None, None));

        let placement = BodyPlacement {
            body: CelestialBody::Mars,
            position: result,
            sign: None,
            house: None,
        };

        assert_eq!(
            placement.motion_direction(),
            Some(MotionDirection::Retrograde)
        );
    }

    #[test]
    fn chart_snapshot_supports_body_lookup_and_retrograde_summary() {
        let instant = Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        );
        let mut retrograde = EphemerisResult::new(
            BackendId::new("toy-chart"),
            CelestialBody::Mars,
            instant,
            pleiades_types::CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Apparent,
        );
        retrograde.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(90.0),
            Latitude::from_degrees(0.0),
            None,
        ));
        retrograde.motion = Some(pleiades_types::Motion::new(Some(-0.01), None, None));

        let mut direct = EphemerisResult::new(
            BackendId::new("toy-chart"),
            CelestialBody::Sun,
            instant,
            pleiades_types::CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Apparent,
        );
        direct.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(15.0),
            Latitude::from_degrees(0.0),
            None,
        ));
        direct.motion = Some(pleiades_types::Motion::new(Some(0.01), None, None));

        let mut stationary = EphemerisResult::new(
            BackendId::new("toy-chart"),
            CelestialBody::Mercury,
            instant,
            pleiades_types::CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Apparent,
        );
        stationary.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(45.0),
            Latitude::from_degrees(0.0),
            None,
        ));
        stationary.motion = Some(pleiades_types::Motion::new(Some(0.0), None, None));

        let chart = ChartSnapshot {
            backend_id: BackendId::new("toy-chart"),
            instant,
            observer: None,
            zodiac_mode: ZodiacMode::Tropical,
            houses: None,
            placements: vec![
                BodyPlacement {
                    body: CelestialBody::Sun,
                    position: direct,
                    sign: Some(ZodiacSign::Aries),
                    house: Some(1),
                },
                BodyPlacement {
                    body: CelestialBody::Mercury,
                    position: stationary,
                    sign: Some(ZodiacSign::Taurus),
                    house: Some(2),
                },
                BodyPlacement {
                    body: CelestialBody::Mars,
                    position: retrograde,
                    sign: Some(ZodiacSign::Cancer),
                    house: Some(8),
                },
            ],
        };

        assert!(chart.placement_for(&CelestialBody::Sun).is_some());
        assert_eq!(
            chart.sign_for_body(&CelestialBody::Sun),
            Some(ZodiacSign::Aries)
        );
        assert_eq!(
            chart.sign_for_body(&CelestialBody::Mars),
            Some(ZodiacSign::Cancer)
        );
        assert_eq!(chart.house_for_body(&CelestialBody::Sun), Some(1));
        assert_eq!(chart.house_for_body(&CelestialBody::Mars), Some(8));
        assert_eq!(chart.house_for_body(&CelestialBody::Mercury), Some(2));
        assert_eq!(
            chart.motion_direction_for(&CelestialBody::Mars),
            Some(MotionDirection::Retrograde)
        );
        assert_eq!(
            chart
                .placements_in_sign(ZodiacSign::Cancer)
                .map(|placement| placement.body.clone())
                .collect::<Vec<_>>(),
            vec![CelestialBody::Mars]
        );
        assert!(chart
            .placements_in_sign(ZodiacSign::Pisces)
            .next()
            .is_none());
        assert_eq!(
            chart
                .placements_in_house(2)
                .map(|placement| placement.body.clone())
                .collect::<Vec<_>>(),
            vec![CelestialBody::Mercury]
        );
        assert_eq!(
            chart
                .placements_in_house(8)
                .map(|placement| placement.body.clone())
                .collect::<Vec<_>>(),
            vec![CelestialBody::Mars]
        );
        assert!(chart.placements_in_house(12).next().is_none());
        assert_eq!(chart.stationary_placements().count(), 1);
        assert_eq!(chart.retrograde_placements().count(), 1);
        assert_eq!(
            chart.motion_summary(),
            MotionSummary {
                direct: 1,
                stationary: 1,
                retrograde: 1,
                unknown: 0,
            }
        );
        assert_eq!(
            chart
                .placements_with_motion_direction(MotionDirection::Direct)
                .map(|placement| placement.body.clone())
                .collect::<Vec<_>>(),
            vec![CelestialBody::Sun]
        );
        assert_eq!(
            chart
                .placements_with_motion_direction(MotionDirection::Stationary)
                .map(|placement| placement.body.clone())
                .collect::<Vec<_>>(),
            vec![CelestialBody::Mercury]
        );
        assert_eq!(
            chart
                .direct_placements()
                .map(|placement| placement.body.clone())
                .collect::<Vec<_>>(),
            vec![CelestialBody::Sun]
        );
        let rendered = chart.to_string();
        assert!(
            rendered.contains("Motion summary: 1 direct, 1 stationary, 1 retrograde, 0 unknown")
        );
        assert!(rendered.contains("Stationary bodies: Mercury"));
        assert!(rendered.contains("Retrograde bodies: Mars"));
    }

    #[test]
    fn chart_snapshot_exposes_major_aspects_and_angular_separation() {
        let instant = Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        );
        let mut sun = EphemerisResult::new(
            BackendId::new("toy-chart"),
            CelestialBody::Sun,
            instant,
            pleiades_types::CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Apparent,
        );
        sun.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(15.0),
            Latitude::from_degrees(0.0),
            None,
        ));

        let mut moon = EphemerisResult::new(
            BackendId::new("toy-chart"),
            CelestialBody::Moon,
            instant,
            pleiades_types::CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Apparent,
        );
        moon.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(75.0),
            Latitude::from_degrees(0.0),
            None,
        ));

        let chart = ChartSnapshot {
            backend_id: BackendId::new("toy-chart"),
            instant,
            observer: None,
            zodiac_mode: ZodiacMode::Tropical,
            houses: None,
            placements: vec![
                BodyPlacement {
                    body: CelestialBody::Sun,
                    position: sun,
                    sign: Some(ZodiacSign::Aries),
                    house: Some(1),
                },
                BodyPlacement {
                    body: CelestialBody::Moon,
                    position: moon,
                    sign: Some(ZodiacSign::Taurus),
                    house: Some(2),
                },
            ],
        };

        assert_eq!(
            chart.angular_separation(&CelestialBody::Sun, &CelestialBody::Moon),
            Some(Angle::from_degrees(60.0))
        );

        let aspects = chart.major_aspects();
        assert_eq!(aspects.len(), 1);
        let aspect = &aspects[0];
        assert_eq!(aspect.left, CelestialBody::Sun);
        assert_eq!(aspect.right, CelestialBody::Moon);
        assert_eq!(aspect.kind, AspectKind::Sextile);
        assert_eq!(aspect.separation, Angle::from_degrees(60.0));
        assert_eq!(aspect.orb, Angle::from_degrees(0.0));
        assert!(chart.to_string().contains("Aspects:"));
        assert!(chart.to_string().contains("Sun Sextile Moon"));
    }

    #[test]
    fn chart_snapshot_renders_custom_body_identifiers() {
        let instant = Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        );
        let custom_body =
            CelestialBody::Custom(pleiades_types::CustomBodyId::new("asteroid", "433-Eros"));
        let mut result = EphemerisResult::new(
            BackendId::new("toy-chart"),
            custom_body.clone(),
            instant,
            pleiades_types::CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Apparent,
        );
        result.motion = Some(pleiades_types::Motion::new(Some(-0.01), None, None));

        let chart = ChartSnapshot {
            backend_id: BackendId::new("toy-chart"),
            instant,
            observer: None,
            zodiac_mode: ZodiacMode::Tropical,
            houses: None,
            placements: vec![BodyPlacement {
                body: custom_body,
                position: result,
                sign: None,
                house: None,
            }],
        };

        let rendered = chart.to_string();
        assert!(rendered.contains("asteroid:433-Eros"));
        assert!(rendered.contains("Retrograde bodies: asteroid:433-Eros"));
    }
}
