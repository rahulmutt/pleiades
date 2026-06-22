use core::fmt;

use pleiades_backend::{
    request_policy_summary_for_report, time_scale_policy_summary_for_report,
    validated_frame_policy_summary_for_report, Apparentness,
};
use pleiades_houses::HouseError;
use pleiades_types::{
    Angle, CelestialBody, Instant, MotionDirection, ObserverLocation, ZodiacMode,
};

use super::aspects::{
    angle_separation, best_aspect_definition, default_aspect_definitions,
    validate_aspect_definitions, AspectDefinition, AspectDefinitionValidationError, AspectMatch,
    AspectSummary,
};
use super::houses::{dominant_house_summary, HouseSummary, HouseSummaryValidationError};
use super::motion::MotionSummary;
use super::observer::{
    render_house_system_label, ObserverPolicy, ObserverSummary, ObserverSummaryValidationError,
};
use super::placement::{BodyPlacement, BodyPlacementValidationError};
use super::signs::{dominant_sign_summary, SignSummary, SignSummaryValidationError};
use crate::BackendId;
use pleiades_houses::HouseSnapshot;

/// A chart snapshot suitable for user-facing reports.
///
/// The snapshot keeps the observer posture explicit even when the chart does
/// not include houses. An observer location by itself still leaves the body
/// placements geocentric unless the separate body-position observer is set;
/// the `house-only` policy only appears when the snapshot also carries computed
/// houses.
///
/// # Example
///
/// ```
/// use pleiades_core::{Apparentness, BackendId, ChartSnapshot};
/// use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale, ZodiacMode};
///
/// let snapshot = ChartSnapshot {
///     backend_id: BackendId::new("demo"),
///     instant: Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
///     observer: Some(ObserverLocation::new(
///         Latitude::from_degrees(51.5),
///         Longitude::from_degrees(-0.1),
///         None,
///     )),
///     body_observer: None,
///     zodiac_mode: ZodiacMode::Tropical,
///     apparentness: Apparentness::Mean,
///     houses: None,
///     placements: Vec::new(),
/// };
///
/// let summary = snapshot.summary_line();
/// assert!(summary.contains("observer=geocentric"));
/// assert!(summary.contains("house system=none"));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ChartSnapshot {
    /// Backend that produced the chart.
    pub backend_id: BackendId,
    /// Instant that was charted.
    pub instant: Instant,
    /// Optional observer location.
    pub observer: Option<ObserverLocation>,
    /// Optional observer location used for topocentric body positions.
    pub body_observer: Option<ObserverLocation>,
    /// Zodiac mode used for the chart.
    pub zodiac_mode: ZodiacMode,
    /// Apparentness used for the backend position queries.
    pub apparentness: Apparentness,
    /// Optional house snapshot.
    pub houses: Option<HouseSnapshot>,
    /// Ordered body placements.
    pub placements: Vec<BodyPlacement>,
}

/// Errors returned when a chart snapshot no longer matches its stored rows.
#[derive(Clone, Debug, PartialEq)]
pub enum ChartSnapshotValidationError {
    /// The stored observer summary is invalid.
    InvalidObserverSummary(ObserverSummaryValidationError),
    /// The stored house snapshot is invalid.
    InvalidHouseSnapshot(HouseError),
    /// A body placement is invalid.
    InvalidPlacement {
        /// The one-based placement index.
        placement: usize,
        /// The body placement validation error.
        error: BodyPlacementValidationError,
    },
    /// The chart snapshot includes houses but not the required observer location.
    HouseSnapshotMissingObserver,
    /// A placement carries a house number without a house snapshot.
    PlacementHasHouseWithoutSnapshot {
        /// The one-based placement index.
        placement: usize,
        /// The body represented by the placement.
        body: CelestialBody,
        /// The invalid house number.
        house: usize,
    },
    /// A placement carries a house number outside the house snapshot's range.
    PlacementHouseOutOfRange {
        /// The one-based placement index.
        placement: usize,
        /// The body represented by the placement.
        body: CelestialBody,
        /// The invalid house number.
        house: usize,
        /// The available house count.
        house_count: usize,
    },
    /// The cached sign summary no longer matches the stored placements.
    InvalidSignSummary(SignSummaryValidationError),
    /// The cached house summary no longer matches the stored placements.
    InvalidHouseSummary(HouseSummaryValidationError),
}

impl fmt::Display for ChartSnapshotValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidObserverSummary(error) => {
                write!(f, "chart snapshot observer summary is invalid: {error}")
            }
            Self::InvalidHouseSnapshot(error) => {
                write!(f, "chart snapshot house snapshot is invalid: {error}")
            }
            Self::HouseSnapshotMissingObserver => {
                f.write_str("chart snapshot includes houses but no observer location")
            }
            Self::InvalidPlacement { placement, error } => {
                write!(f, "chart snapshot placement {placement} is invalid: {error}")
            }
            Self::PlacementHasHouseWithoutSnapshot {
                placement,
                body,
                house,
            } => write!(
                f,
                "chart snapshot placement {placement} for body {body} carries house {house} without a house snapshot"
            ),
            Self::PlacementHouseOutOfRange {
                placement,
                body,
                house,
                house_count,
            } => write!(
                f,
                "chart snapshot placement {placement} for body {body} carries house {house} but the house snapshot only has {house_count} cusps"
            ),
            Self::InvalidSignSummary(error) => {
                write!(f, "chart snapshot sign summary is invalid: {error}")
            }
            Self::InvalidHouseSummary(error) => {
                write!(f, "chart snapshot house summary is invalid: {error}")
            }
        }
    }
}

impl std::error::Error for ChartSnapshotValidationError {}

impl ChartSnapshot {
    /// Returns the number of body placements in the chart.
    pub fn len(&self) -> usize {
        self.placements.len()
    }

    /// Returns `true` if the chart contains no body placements.
    pub fn is_empty(&self) -> bool {
        self.placements.is_empty()
    }

    /// Returns the observer policy implied by the computed chart snapshot.
    pub fn observer_policy(&self) -> ObserverPolicy {
        if self.observer.is_some() && self.houses.is_some() {
            ObserverPolicy::HouseOnly
        } else {
            ObserverPolicy::Geocentric
        }
    }

    /// Returns the observer summary implied by the computed chart snapshot.
    pub fn observer_summary(&self) -> ObserverSummary {
        ObserverSummary::new(self.observer_policy(), self.observer.clone())
            .with_body_location(self.body_observer.clone())
    }

    /// Returns the observer summary implied by the computed chart snapshot after validation.
    pub fn validated_observer_summary(
        &self,
    ) -> Result<ObserverSummary, ObserverSummaryValidationError> {
        let summary = self.observer_summary();
        summary.validate()?;
        Ok(summary)
    }

    /// Returns the topocentric body-position observer label after validation.
    ///
    /// This is the chart-snapshot counterpart to
    /// [`super::request::ChartRequest::validated_body_observer_label()`], so report code can
    /// inspect the optional topocentric body observer without rebuilding the
    /// shared observer summary.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_core::{Apparentness, BackendId, ChartSnapshot};
    /// use pleiades_types::{Instant, JulianDay, Latitude, Longitude, TimeScale, ZodiacMode};
    ///
    /// let snapshot = ChartSnapshot {
    ///     backend_id: BackendId::new("demo"),
    ///     instant: Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    ///     observer: None,
    ///     body_observer: Some(pleiades_types::ObserverLocation::new(
    ///         Latitude::from_degrees(-33.9),
    ///         Longitude::from_degrees(151.2),
    ///         None,
    ///     )),
    ///     zodiac_mode: ZodiacMode::Tropical,
    ///     apparentness: Apparentness::Mean,
    ///     houses: None,
    ///     placements: Vec::new(),
    /// };
    ///
    /// let label = snapshot
    ///     .validated_body_observer_label()
    ///     .expect("valid chart snapshot body observer");
    /// assert!(label.contains("latitude=-33.9°"));
    /// ```
    pub fn validated_body_observer_label(&self) -> Result<String, ObserverSummaryValidationError> {
        self.observer_summary().validated_body_location_label()
    }

    /// Validates the stored observer summary, house snapshot, and placements.
    pub fn validate(&self) -> Result<(), ChartSnapshotValidationError> {
        self.observer_summary()
            .validate()
            .map_err(ChartSnapshotValidationError::InvalidObserverSummary)?;

        if self.houses.is_some() && self.observer.is_none() {
            return Err(ChartSnapshotValidationError::HouseSnapshotMissingObserver);
        }

        if let Some(houses) = &self.houses {
            houses
                .validate()
                .map_err(ChartSnapshotValidationError::InvalidHouseSnapshot)?;
        }

        for (index, placement) in self.placements.iter().enumerate() {
            placement.validate().map_err(|error| {
                ChartSnapshotValidationError::InvalidPlacement {
                    placement: index + 1,
                    error,
                }
            })?;

            if let Some(house) = placement.house {
                if let Some(houses) = &self.houses {
                    let house_count = houses.cusps.len();
                    if house > house_count {
                        return Err(ChartSnapshotValidationError::PlacementHouseOutOfRange {
                            placement: index + 1,
                            body: placement.body.clone(),
                            house,
                            house_count,
                        });
                    }
                } else {
                    return Err(
                        ChartSnapshotValidationError::PlacementHasHouseWithoutSnapshot {
                            placement: index + 1,
                            body: placement.body.clone(),
                            house,
                        },
                    );
                }
            }
        }

        let sign_count = self
            .placements
            .iter()
            .filter(|placement| placement.sign.is_some())
            .count();
        self.sign_summary()
            .validate(sign_count)
            .map_err(ChartSnapshotValidationError::InvalidSignSummary)?;
        self.house_summary()
            .validate(self.placements.len())
            .map_err(ChartSnapshotValidationError::InvalidHouseSummary)?;

        Ok(())
    }

    /// Returns a compact one-line summary of the computed chart snapshot.
    ///
    /// The summary is intended for diagnostics, validation reports, and
    /// release-facing snapshot notes. It mirrors the request-shape vocabulary
    /// used by [`super::request::ChartRequest::summary_line`] while adding the backend ID and
    /// the computed placement count so callers can compare the assembled chart
    /// against the original request at a glance. The stored house observer and
    /// body-position observer locations, when present, are rendered separately
    /// from the observer policy so the geocentric-versus-house-only split
    /// stays explicit.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_core::{Apparentness, BackendId, ChartSnapshot};
    /// use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale, ZodiacMode};
    ///
    /// let snapshot = ChartSnapshot {
    ///     backend_id: BackendId::new("demo"),
    ///     instant: Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    ///     observer: None,
    ///     body_observer: Some(ObserverLocation::new(
    ///         Latitude::from_degrees(-33.9),
    ///         Longitude::from_degrees(151.2),
    ///         None,
    ///     )),
    ///     zodiac_mode: ZodiacMode::Tropical,
    ///     apparentness: Apparentness::Mean,
    ///     houses: None,
    ///     placements: Vec::new(),
    /// };
    ///
    /// let summary = snapshot.summary_line();
    /// assert!(summary.contains("backend=demo"));
    /// assert!(summary.contains("body observer=latitude=-33.9°"));
    /// assert!(summary.contains("house system=none"));
    /// ```
    pub fn summary_line(&self) -> String {
        let house_system = self.houses.as_ref().map_or_else(
            || "none".to_string(),
            |houses| render_house_system_label(&houses.system),
        );
        let house_cusp_count = self.houses.as_ref().map_or(0, |houses| houses.cusps.len());

        format!(
            "backend={}; instant={} ({}); placements={}; zodiac={}; apparentness={}; {}; house system={}; house cusps={}",
            self.backend_id,
            self.instant.julian_day,
            self.instant.scale,
            self.placements.len(),
            self.zodiac_mode,
            self.apparentness,
            self.observer_summary(),
            house_system,
            house_cusp_count,
        )
    }

    /// Returns a compact one-line summary after validating the snapshot data.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_core::{Apparentness, BackendId, ChartSnapshot};
    /// use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale, ZodiacMode};
    ///
    /// let snapshot = ChartSnapshot {
    ///     backend_id: BackendId::new("demo"),
    ///     instant: Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    ///     observer: Some(ObserverLocation::new(
    ///         Latitude::from_degrees(51.5),
    ///         Longitude::from_degrees(-0.1),
    ///         None,
    ///     )),
    ///     body_observer: Some(ObserverLocation::new(
    ///         Latitude::from_degrees(-33.9),
    ///         Longitude::from_degrees(151.2),
    ///         None,
    ///     )),
    ///     zodiac_mode: ZodiacMode::Tropical,
    ///     apparentness: Apparentness::Mean,
    ///     houses: None,
    ///     placements: Vec::new(),
    /// };
    ///
    /// let summary = snapshot
    ///     .validated_summary_line()
    ///     .expect("valid chart snapshot");
    /// assert!(summary.contains("observer=geocentric"));
    /// assert!(summary.contains("body observer=latitude=-33.9°"));
    /// ```
    pub fn validated_summary_line(&self) -> Result<String, ChartSnapshotValidationError> {
        self.validate()?;
        Ok(self.summary_line())
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

    /// Returns the backend motion sample for a requested body, if it is available.
    pub fn motion_for_body(&self, body: &CelestialBody) -> Option<&pleiades_types::Motion> {
        self.placement_for(body)?.motion()
    }

    /// Returns the longitudinal motion speed for a requested body, if it is available.
    pub fn longitude_speed_for(&self, body: &CelestialBody) -> Option<f64> {
        self.placement_for(body)?.longitude_speed()
    }

    /// Returns the latitudinal motion speed for a requested body, if it is available.
    pub fn latitude_speed_for(&self, body: &CelestialBody) -> Option<f64> {
        self.placement_for(body)?.latitude_speed()
    }

    /// Returns the radial motion speed for a requested body, if it is available.
    pub fn distance_speed_for(&self, body: &CelestialBody) -> Option<f64> {
        self.placement_for(body)?.distance_speed()
    }

    /// Returns the sign for a requested body, if sign placement was computed.
    pub fn sign_for_body(&self, body: &CelestialBody) -> Option<pleiades_types::ZodiacSign> {
        self.placement_for(body)?.sign
    }

    /// Returns the house number for a requested body, if house placement was computed.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_backend::{Apparentness, BackendId, EphemerisResult};
    /// use pleiades_core::{BodyPlacement, ChartSnapshot};
    /// use pleiades_types::{CelestialBody, CoordinateFrame, Instant, JulianDay, TimeScale, ZodiacMode, ZodiacSign};
    ///
    /// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    /// let position = EphemerisResult::new(
    ///     BackendId::new("demo"),
    ///     CelestialBody::Sun,
    ///     instant,
    ///     CoordinateFrame::Ecliptic,
    ///     ZodiacMode::Tropical,
    ///     Apparentness::Mean,
    /// );
    ///
    /// let chart = ChartSnapshot {
    ///     backend_id: BackendId::new("demo"),
    ///     instant,
    ///     observer: None,
    ///     body_observer: None,
    ///     zodiac_mode: ZodiacMode::Tropical,
    ///     apparentness: Apparentness::Mean,
    ///     houses: None,
    ///     placements: vec![BodyPlacement {
    ///         body: CelestialBody::Sun,
    ///         position,
    ///         sign: Some(ZodiacSign::Aries),
    ///         house: Some(1),
    ///         apparent: None,
    ///     }],
    /// };
    ///
    /// assert_eq!(chart.house_for_body(&CelestialBody::Sun), Some(1));
    /// assert_eq!(chart.house_for_body(&CelestialBody::Moon), None);
    /// ```
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
    pub fn placements_in_sign(
        &self,
        sign: pleiades_types::ZodiacSign,
    ) -> impl Iterator<Item = &BodyPlacement> {
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

    /// Returns the placements whose motion direction could not be determined.
    pub fn unknown_motion_placements(&self) -> impl Iterator<Item = &BodyPlacement> {
        self.placements
            .iter()
            .filter(|placement| placement.motion_direction().is_none())
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

    /// Returns the occupied zodiac signs in canonical zodiac order.
    pub fn occupied_signs(&self) -> Vec<pleiades_types::ZodiacSign> {
        self.sign_summary().occupied_signs()
    }

    /// Returns the occupied house numbers in ascending order.
    pub fn occupied_houses(&self) -> Vec<usize> {
        self.house_summary().occupied_houses()
    }

    /// Returns the placements that are currently classified as direct.
    pub fn direct_placements(&self) -> impl Iterator<Item = &BodyPlacement> {
        self.placements_with_motion_direction(MotionDirection::Direct)
    }

    /// Returns a summary of the chart's zodiac-sign placements.
    pub fn sign_summary(&self) -> SignSummary {
        let mut summary = SignSummary::default();

        for placement in &self.placements {
            if let Some(sign) = placement.sign {
                summary.increment(sign);
            }
        }

        summary
    }

    /// Returns a summary of the chart's occupied houses.
    pub fn house_summary(&self) -> HouseSummary {
        let mut summary = HouseSummary::default();

        for placement in &self.placements {
            match placement.house {
                Some(house) => summary.increment(house),
                None => summary.unknown += 1,
            }
        }

        summary
    }

    /// Returns only the sign counts tied for the highest occupancy.
    pub fn dominant_sign_summary(&self) -> SignSummary {
        dominant_sign_summary(self.sign_summary())
    }

    /// Returns only the house counts tied for the highest occupancy.
    pub fn dominant_house_summary(&self) -> HouseSummary {
        dominant_house_summary(self.house_summary())
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

    /// Returns a summary of the major aspects found in the chart.
    pub fn aspect_summary(&self) -> AspectSummary {
        AspectSummary::from_matches(&self.major_aspects())
    }

    /// Returns the major aspects found in the chart using the built-in orb defaults.
    pub fn major_aspects(&self) -> Vec<AspectMatch> {
        self.aspects_with_definitions_checked(default_aspect_definitions())
            .expect("default aspect definitions should be valid")
    }

    /// Returns aspect matches using custom aspect definitions after validating them.
    pub fn aspects_with_definitions_checked(
        &self,
        definitions: &[AspectDefinition],
    ) -> Result<Vec<AspectMatch>, AspectDefinitionValidationError> {
        validate_aspect_definitions(definitions)?;
        Ok(self.aspects_with_definitions(definitions))
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

impl fmt::Display for ChartSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Backend: {}", self.backend_id)?;
        writeln!(
            f,
            "Instant: {} ({})",
            self.instant.julian_day, self.instant.scale
        )?;
        writeln!(
            f,
            "Time-scale policy: {}",
            time_scale_policy_summary_for_report().summary_line()
        )?;
        if let Some(observer) = &self.observer {
            writeln!(
                f,
                "Observer: {} / {}",
                observer.latitude, observer.longitude
            )?;
            if self.houses.is_some() {
                writeln!(
                    f,
                    "Observer policy: used for houses only; body positions remain geocentric"
                )?;
            } else {
                writeln!(
                    f,
                    "Observer policy: geocentric body positions; no house observer supplied"
                )?;
            }
        } else {
            writeln!(
                f,
                "Observer policy: geocentric body positions; no house observer supplied"
            )?;
        }
        writeln!(
            f,
            "Frame policy: {}",
            validated_frame_policy_summary_for_report()
        )?;
        writeln!(f, "Zodiac mode: {}", self.zodiac_mode)?;
        writeln!(f, "Apparentness: {}", self.apparentness)?;
        writeln!(
            f,
            "Apparentness policy: {}",
            request_policy_summary_for_report().apparentness
        )?;
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
            writeln!(f, "  - {}", placement.summary_line())?;
        }

        let sign_summary = self.sign_summary();
        if sign_summary.has_known_signs() {
            writeln!(f, "Sign summary: {}", sign_summary)?;
        }

        let dominant_sign_summary = self.dominant_sign_summary();
        if dominant_sign_summary.has_known_signs() && dominant_sign_summary != sign_summary {
            writeln!(f, "Dominant sign summary: {}", dominant_sign_summary)?;
        }

        let house_summary = self.house_summary();
        if house_summary.has_known_houses() {
            writeln!(f, "House summary: {}", house_summary)?;
        }

        let dominant_house_summary = self.dominant_house_summary();
        if dominant_house_summary.has_known_houses() && dominant_house_summary != house_summary {
            writeln!(f, "Dominant house summary: {}", dominant_house_summary)?;
        }

        let motion_summary = self.motion_summary();
        if motion_summary.has_known_motion() || motion_summary.unknown > 0 {
            match motion_summary.validate(self.placements.len()) {
                Ok(()) => writeln!(f, "Motion summary: {}", motion_summary)?,
                Err(error) => writeln!(f, "Motion summary unavailable ({error})")?,
            }
        }

        let stationary_bodies: Vec<String> = self
            .stationary_placements()
            .map(|placement| placement.body.to_string())
            .collect();
        if !stationary_bodies.is_empty() {
            writeln!(f, "Stationary bodies: {}", stationary_bodies.join(", "))?;
        }

        let unknown_motion_bodies: Vec<String> = self
            .unknown_motion_placements()
            .map(|placement| placement.body.to_string())
            .collect();
        if !unknown_motion_bodies.is_empty() {
            writeln!(
                f,
                "Unknown motion bodies: {}",
                unknown_motion_bodies.join(", ")
            )?;
        }

        let retrograde_bodies: Vec<String> = self
            .retrograde_placements()
            .map(|placement| placement.body.to_string())
            .collect();
        if !retrograde_bodies.is_empty() {
            writeln!(f, "Retrograde bodies: {}", retrograde_bodies.join(", "))?;
        }

        let aspects = self.major_aspects();
        let aspect_summary = AspectSummary::from_matches(&aspects);
        if aspect_summary.has_known_aspects() {
            match aspect_summary.validate(aspects.len()) {
                Ok(()) => writeln!(f, "Aspect summary: {}", aspect_summary)?,
                Err(error) => writeln!(f, "Aspect summary unavailable ({error})")?,
            }
        }
        if !aspects.is_empty() {
            writeln!(f, "Aspects:")?;
            for aspect in aspects {
                writeln!(f, "  - {}", aspect)?;
            }
        }

        Ok(())
    }
}
