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
    CelestialBody, HouseSystem, Instant, Longitude, MotionDirection, ObserverLocation, ZodiacMode,
    ZodiacSign,
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
}

impl BodyPlacement {
    /// Returns the coarse direction of longitudinal motion when the backend supplied motion data.
    pub fn motion_direction(&self) -> Option<MotionDirection> {
        self.position.motion.as_ref()?.longitude_direction()
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

        let retrograde_bodies: Vec<String> = self
            .retrograde_placements()
            .map(|placement| placement.body.to_string())
            .collect();
        if !retrograde_bodies.is_empty() {
            writeln!(f, "Retrograde bodies: {}", retrograde_bodies.join(", "))?;
        }

        Ok(())
    }
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
        assert_eq!(chart.house_for_body(&CelestialBody::Mercury), None);
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
                .placements_in_house(8)
                .map(|placement| placement.body.clone())
                .collect::<Vec<_>>(),
            vec![CelestialBody::Mars]
        );
        assert!(chart.placements_in_house(12).next().is_none());
        assert_eq!(chart.retrograde_placements().count(), 1);
        assert!(chart.to_string().contains("Retrograde bodies: Mars"));
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
