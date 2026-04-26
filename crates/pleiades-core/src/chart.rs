//! Chart assembly and higher-level chart helpers built on top of backend position queries.
//!
//! The current chart façade keeps the workflow intentionally small: callers
//! provide a set of bodies, the façade queries the backend, and the result
//! captures the body placements plus their zodiac signs and the apparentness
//! mode used for the backend queries. House placement can be requested
//! explicitly for chart-aware consumers, which keeps the workflow practical
//! without hardwiring more chart logic than the façade needs.

use core::{fmt, time::Duration};

use pleiades_ayanamsa::sidereal_offset;
use pleiades_backend::{
    Apparentness, EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest,
    EphemerisResult,
};
use pleiades_houses::{calculate_houses, house_for_longitude, HouseRequest, HouseSnapshot};
use pleiades_types::{
    Angle, CelestialBody, HouseSystem, Instant, Longitude, MotionDirection, ObserverLocation,
    TimeScale, TimeScaleConversionError, ZodiacMode, ZodiacSign,
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
///
/// # Example
///
/// ```
/// use core::time::Duration;
/// use pleiades_core::ChartRequest;
/// use pleiades_types::{
///     HouseSystem, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
/// };
///
/// let request = ChartRequest::new(Instant::new(
///     JulianDay::from_days(2_451_545.0),
///     TimeScale::Utc,
/// ))
/// .with_observer(ObserverLocation::new(
///     Latitude::from_degrees(51.5),
///     Longitude::from_degrees(-0.1),
///     None,
/// ))
/// .with_house_system(HouseSystem::WholeSign)
/// .with_tdb_from_utc_signed(Duration::from_secs_f64(64.184), -0.001_657)
/// .expect("explicit time-scale conversion");
///
/// assert_eq!(request.instant.scale, TimeScale::Tdb);
/// assert_eq!(request.house_system, Some(HouseSystem::WholeSign));
/// assert!(request.observer.is_some());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ChartRequest {
    /// The instant to chart.
    pub instant: Instant,
    /// Optional observer location used for house calculations.
    ///
    /// Body positions are still queried geocentrically. A future topocentric
    /// chart mode should add an explicit position-mode field instead of
    /// reusing this house-observer value implicitly.
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

    /// Replaces the chart instant with a caller-supplied time-scale offset.
    ///
    /// This is a mechanical convenience wrapper over
    /// [`Instant::with_time_scale_offset`]. It keeps the chosen conversion
    /// policy explicit without introducing any built-in Delta T or leap-second
    /// logic.
    pub fn with_instant_time_scale_offset(
        mut self,
        target_scale: TimeScale,
        offset_seconds: f64,
    ) -> Self {
        self.instant = self
            .instant
            .with_time_scale_offset(target_scale, offset_seconds);
        self
    }

    /// Converts the chart instant from UT1 to TT using caller-supplied Delta T.
    ///
    /// This convenience method mirrors [`Instant::tt_from_ut1`] so chart callers
    /// can prepare a TT request surface before assembling houses or body
    /// positions.
    ///
    /// For signed `TT - UT1` policies, use [`ChartRequest::with_tt_from_ut1_signed`].
    pub fn with_tt_from_ut1(mut self, delta_t: Duration) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_ut1(delta_t)?;
        Ok(self)
    }

    /// Converts the chart instant from UT1 to TT using a caller-supplied signed offset.
    ///
    /// This convenience method mirrors [`Instant::tt_from_ut1_signed`] so chart
    /// callers can keep the chosen Delta T policy explicit when the `TT - UT1`
    /// correction is negative at the chosen epoch.
    pub fn with_tt_from_ut1_signed(
        mut self,
        offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_ut1_signed(offset_seconds)?;
        Ok(self)
    }

    /// Converts the chart instant from UTC to TT using caller-supplied offset.
    ///
    /// This convenience method mirrors [`Instant::tt_from_utc`] so chart callers
    /// can keep civil-time inputs explicit without introducing built-in leap-
    /// second or Delta T modeling in the façade.
    ///
    /// For signed `TT - UTC` policies, use [`ChartRequest::with_tt_from_utc_signed`].
    pub fn with_tt_from_utc(mut self, delta_t: Duration) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_utc(delta_t)?;
        Ok(self)
    }

    /// Converts the chart instant from UTC to TT using a caller-supplied signed offset.
    ///
    /// This convenience method mirrors [`Instant::tt_from_utc_signed`] so chart
    /// callers can keep civil-time inputs explicit when the `TT - UTC`
    /// correction is negative at the chosen epoch.
    pub fn with_tt_from_utc_signed(
        mut self,
        offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_utc_signed(offset_seconds)?;
        Ok(self)
    }

    /// Converts the chart instant from TT to TDB using caller-supplied offset.
    ///
    /// This convenience method mirrors [`Instant::tdb_from_tt`] so chart
    /// callers can keep the chosen relativistic policy explicit without baking
    /// a built-in TDB model into the façade.
    pub fn with_tdb_from_tt(mut self, offset: Duration) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tdb_from_tt(offset)?;
        Ok(self)
    }

    /// Converts the chart instant from TT to TDB using a caller-supplied
    /// signed offset.
    ///
    /// This convenience method mirrors [`Instant::tdb_from_tt_signed`] so
    /// chart callers can keep the chosen relativistic policy explicit when the
    /// TDB-TT correction is negative at the chosen epoch.
    pub fn with_tdb_from_tt_signed(
        mut self,
        offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tdb_from_tt_signed(offset_seconds)?;
        Ok(self)
    }

    /// Converts the chart instant from TDB to TT using a caller-supplied
    /// signed offset.
    ///
    /// This convenience method mirrors [`Instant::tt_from_tdb`] so chart
    /// callers can keep the chosen relativistic policy explicit when they
    /// already start from a TDB-tagged instant.
    pub fn with_tt_from_tdb(
        mut self,
        offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_tdb(offset_seconds)?;
        Ok(self)
    }

    /// Converts the chart instant from TDB to TT using a caller-supplied
    /// signed offset.
    ///
    /// This convenience method mirrors [`Instant::tt_from_tdb_signed`] so chart
    /// callers can keep the chosen relativistic policy explicit while matching
    /// the signed helper naming used for the other time-scale conversions.
    pub fn with_tt_from_tdb_signed(
        mut self,
        offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tt_from_tdb_signed(offset_seconds)?;
        Ok(self)
    }

    /// Converts the chart instant from UT1 to TDB using caller-supplied
    /// TT-UT1 and TDB-TT offsets.
    ///
    /// This convenience method mirrors [`Instant::tdb_from_ut1`] so chart
    /// callers can prepare a TDB request surface from dynamical-time inputs
    /// without introducing built-in Delta T or relativistic modeling in the
    /// façade.
    pub fn with_tdb_from_ut1(
        mut self,
        tt_offset: Duration,
        tdb_offset: Duration,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tdb_from_ut1(tt_offset, tdb_offset)?;
        Ok(self)
    }

    /// Converts the chart instant from UT1 to TDB using caller-supplied
    /// TT-UT1 and signed TDB-TT offsets.
    ///
    /// This convenience method mirrors [`Instant::tdb_from_ut1_signed`] so
    /// chart callers can prepare a TDB request surface from dynamical-time
    /// inputs when the TDB-TT correction is negative at the chosen epoch.
    pub fn with_tdb_from_ut1_signed(
        mut self,
        tt_offset: Duration,
        tdb_offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self
            .instant
            .tdb_from_ut1_signed(tt_offset, tdb_offset_seconds)?;
        Ok(self)
    }

    /// Converts the chart instant from UTC to TDB using caller-supplied
    /// TT-UTC and TDB-TT offsets.
    ///
    /// This convenience method mirrors [`Instant::tdb_from_utc`] so chart
    /// callers can prepare a TDB request surface from civil-time inputs
    /// without introducing built-in leap-second or relativistic modeling in the
    /// façade.
    pub fn with_tdb_from_utc(
        mut self,
        tt_offset: Duration,
        tdb_offset: Duration,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self.instant.tdb_from_utc(tt_offset, tdb_offset)?;
        Ok(self)
    }

    /// Converts the chart instant from UTC to TDB using caller-supplied
    /// TT-UTC and signed TDB-TT offsets.
    ///
    /// This convenience method mirrors [`Instant::tdb_from_utc_signed`] so
    /// chart callers can prepare a TDB request surface from civil-time inputs
    /// when the TDB-TT correction is negative at the chosen epoch.
    pub fn with_tdb_from_utc_signed(
        mut self,
        tt_offset: Duration,
        tdb_offset_seconds: f64,
    ) -> Result<Self, TimeScaleConversionError> {
        self.instant = self
            .instant
            .tdb_from_utc_signed(tt_offset, tdb_offset_seconds)?;
        Ok(self)
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
    /// Apparentness used for the backend position queries.
    pub apparentness: Apparentness,
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
    pub fn occupied_signs(&self) -> Vec<ZodiacSign> {
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

/// A summary of zodiac-sign placements in a chart snapshot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SignSummary {
    /// Placements in Aries.
    pub aries: usize,
    /// Placements in Taurus.
    pub taurus: usize,
    /// Placements in Gemini.
    pub gemini: usize,
    /// Placements in Cancer.
    pub cancer: usize,
    /// Placements in Leo.
    pub leo: usize,
    /// Placements in Virgo.
    pub virgo: usize,
    /// Placements in Libra.
    pub libra: usize,
    /// Placements in Scorpio.
    pub scorpio: usize,
    /// Placements in Sagittarius.
    pub sagittarius: usize,
    /// Placements in Capricorn.
    pub capricorn: usize,
    /// Placements in Aquarius.
    pub aquarius: usize,
    /// Placements in Pisces.
    pub pisces: usize,
}

impl SignSummary {
    fn increment(&mut self, sign: ZodiacSign) {
        match sign {
            ZodiacSign::Aries => self.aries += 1,
            ZodiacSign::Taurus => self.taurus += 1,
            ZodiacSign::Gemini => self.gemini += 1,
            ZodiacSign::Cancer => self.cancer += 1,
            ZodiacSign::Leo => self.leo += 1,
            ZodiacSign::Virgo => self.virgo += 1,
            ZodiacSign::Libra => self.libra += 1,
            ZodiacSign::Scorpio => self.scorpio += 1,
            ZodiacSign::Sagittarius => self.sagittarius += 1,
            ZodiacSign::Capricorn => self.capricorn += 1,
            ZodiacSign::Aquarius => self.aquarius += 1,
            ZodiacSign::Pisces => self.pisces += 1,
            _ => {}
        }
    }

    /// Returns `true` when the snapshot contains at least one sign placement.
    pub fn has_known_signs(self) -> bool {
        self.aries
            + self.taurus
            + self.gemini
            + self.cancer
            + self.leo
            + self.virgo
            + self.libra
            + self.scorpio
            + self.sagittarius
            + self.capricorn
            + self.aquarius
            + self.pisces
            > 0
    }

    /// Returns the occupied zodiac signs in canonical zodiac order.
    pub fn occupied_signs(self) -> Vec<ZodiacSign> {
        let mut signs = Vec::new();
        for (count, sign) in [
            (self.aries, ZodiacSign::Aries),
            (self.taurus, ZodiacSign::Taurus),
            (self.gemini, ZodiacSign::Gemini),
            (self.cancer, ZodiacSign::Cancer),
            (self.leo, ZodiacSign::Leo),
            (self.virgo, ZodiacSign::Virgo),
            (self.libra, ZodiacSign::Libra),
            (self.scorpio, ZodiacSign::Scorpio),
            (self.sagittarius, ZodiacSign::Sagittarius),
            (self.capricorn, ZodiacSign::Capricorn),
            (self.aquarius, ZodiacSign::Aquarius),
            (self.pisces, ZodiacSign::Pisces),
        ] {
            if count > 0 {
                signs.push(sign);
            }
        }
        signs
    }
}

fn dominant_sign_summary(summary: SignSummary) -> SignSummary {
    let max = [
        summary.aries,
        summary.taurus,
        summary.gemini,
        summary.cancer,
        summary.leo,
        summary.virgo,
        summary.libra,
        summary.scorpio,
        summary.sagittarius,
        summary.capricorn,
        summary.aquarius,
        summary.pisces,
    ]
    .into_iter()
    .max()
    .unwrap_or(0);

    if max == 0 {
        return SignSummary::default();
    }

    SignSummary {
        aries: if summary.aries == max {
            summary.aries
        } else {
            0
        },
        taurus: if summary.taurus == max {
            summary.taurus
        } else {
            0
        },
        gemini: if summary.gemini == max {
            summary.gemini
        } else {
            0
        },
        cancer: if summary.cancer == max {
            summary.cancer
        } else {
            0
        },
        leo: if summary.leo == max { summary.leo } else { 0 },
        virgo: if summary.virgo == max {
            summary.virgo
        } else {
            0
        },
        libra: if summary.libra == max {
            summary.libra
        } else {
            0
        },
        scorpio: if summary.scorpio == max {
            summary.scorpio
        } else {
            0
        },
        sagittarius: if summary.sagittarius == max {
            summary.sagittarius
        } else {
            0
        },
        capricorn: if summary.capricorn == max {
            summary.capricorn
        } else {
            0
        },
        aquarius: if summary.aquarius == max {
            summary.aquarius
        } else {
            0
        },
        pisces: if summary.pisces == max {
            summary.pisces
        } else {
            0
        },
    }
}

impl fmt::Display for SignSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut wrote_any = false;
        for (count, sign) in [
            (self.aries, ZodiacSign::Aries),
            (self.taurus, ZodiacSign::Taurus),
            (self.gemini, ZodiacSign::Gemini),
            (self.cancer, ZodiacSign::Cancer),
            (self.leo, ZodiacSign::Leo),
            (self.virgo, ZodiacSign::Virgo),
            (self.libra, ZodiacSign::Libra),
            (self.scorpio, ZodiacSign::Scorpio),
            (self.sagittarius, ZodiacSign::Sagittarius),
            (self.capricorn, ZodiacSign::Capricorn),
            (self.aquarius, ZodiacSign::Aquarius),
            (self.pisces, ZodiacSign::Pisces),
        ] {
            if count == 0 {
                continue;
            }
            if wrote_any {
                f.write_str(", ")?;
            }
            wrote_any = true;
            write!(f, "{} {}", count, sign)?;
        }

        if !wrote_any {
            f.write_str("no sign placements")?;
        }

        Ok(())
    }
}

/// A summary of occupied houses in a chart snapshot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HouseSummary {
    /// Placements in the first house.
    pub first: usize,
    /// Placements in the second house.
    pub second: usize,
    /// Placements in the third house.
    pub third: usize,
    /// Placements in the fourth house.
    pub fourth: usize,
    /// Placements in the fifth house.
    pub fifth: usize,
    /// Placements in the sixth house.
    pub sixth: usize,
    /// Placements in the seventh house.
    pub seventh: usize,
    /// Placements in the eighth house.
    pub eighth: usize,
    /// Placements in the ninth house.
    pub ninth: usize,
    /// Placements in the tenth house.
    pub tenth: usize,
    /// Placements in the eleventh house.
    pub eleventh: usize,
    /// Placements in the twelfth house.
    pub twelfth: usize,
    /// Placements without an assigned house.
    pub unknown: usize,
}

impl HouseSummary {
    fn increment(&mut self, house: usize) {
        match house {
            1 => self.first += 1,
            2 => self.second += 1,
            3 => self.third += 1,
            4 => self.fourth += 1,
            5 => self.fifth += 1,
            6 => self.sixth += 1,
            7 => self.seventh += 1,
            8 => self.eighth += 1,
            9 => self.ninth += 1,
            10 => self.tenth += 1,
            11 => self.eleventh += 1,
            12 => self.twelfth += 1,
            _ => self.unknown += 1,
        }
    }

    /// Returns `true` when the snapshot contains at least one assigned house placement.
    pub fn has_known_houses(self) -> bool {
        self.first
            + self.second
            + self.third
            + self.fourth
            + self.fifth
            + self.sixth
            + self.seventh
            + self.eighth
            + self.ninth
            + self.tenth
            + self.eleventh
            + self.twelfth
            > 0
    }

    /// Returns the occupied house numbers in ascending order.
    pub fn occupied_houses(self) -> Vec<usize> {
        let mut houses = Vec::new();
        for (count, house) in [
            (self.first, 1usize),
            (self.second, 2),
            (self.third, 3),
            (self.fourth, 4),
            (self.fifth, 5),
            (self.sixth, 6),
            (self.seventh, 7),
            (self.eighth, 8),
            (self.ninth, 9),
            (self.tenth, 10),
            (self.eleventh, 11),
            (self.twelfth, 12),
        ] {
            if count > 0 {
                houses.push(house);
            }
        }
        houses
    }
}

fn dominant_house_summary(summary: HouseSummary) -> HouseSummary {
    let max = [
        summary.first,
        summary.second,
        summary.third,
        summary.fourth,
        summary.fifth,
        summary.sixth,
        summary.seventh,
        summary.eighth,
        summary.ninth,
        summary.tenth,
        summary.eleventh,
        summary.twelfth,
    ]
    .into_iter()
    .max()
    .unwrap_or(0);

    if max == 0 {
        return HouseSummary::default();
    }

    HouseSummary {
        first: if summary.first == max {
            summary.first
        } else {
            0
        },
        second: if summary.second == max {
            summary.second
        } else {
            0
        },
        third: if summary.third == max {
            summary.third
        } else {
            0
        },
        fourth: if summary.fourth == max {
            summary.fourth
        } else {
            0
        },
        fifth: if summary.fifth == max {
            summary.fifth
        } else {
            0
        },
        sixth: if summary.sixth == max {
            summary.sixth
        } else {
            0
        },
        seventh: if summary.seventh == max {
            summary.seventh
        } else {
            0
        },
        eighth: if summary.eighth == max {
            summary.eighth
        } else {
            0
        },
        ninth: if summary.ninth == max {
            summary.ninth
        } else {
            0
        },
        tenth: if summary.tenth == max {
            summary.tenth
        } else {
            0
        },
        eleventh: if summary.eleventh == max {
            summary.eleventh
        } else {
            0
        },
        twelfth: if summary.twelfth == max {
            summary.twelfth
        } else {
            0
        },
        unknown: 0,
    }
}

impl fmt::Display for HouseSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut wrote_any = false;
        for (count, house) in [
            (self.first, 1usize),
            (self.second, 2),
            (self.third, 3),
            (self.fourth, 4),
            (self.fifth, 5),
            (self.sixth, 6),
            (self.seventh, 7),
            (self.eighth, 8),
            (self.ninth, 9),
            (self.tenth, 10),
            (self.eleventh, 11),
            (self.twelfth, 12),
        ] {
            if count == 0 {
                continue;
            }
            if wrote_any {
                f.write_str(", ")?;
            }
            wrote_any = true;
            write!(f, "{} in {} house", count, house_ordinal(house))?;
        }

        if self.unknown > 0 {
            if wrote_any {
                f.write_str(", ")?;
            }
            wrote_any = true;
            write!(f, "{} unassigned", self.unknown)?;
        }

        if !wrote_any {
            f.write_str("no house placements")?;
        }

        Ok(())
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

/// A summary of major aspect matches in a chart snapshot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AspectSummary {
    /// Placements matched by conjunction.
    pub conjunction: usize,
    /// Placements matched by sextile.
    pub sextile: usize,
    /// Placements matched by square.
    pub square: usize,
    /// Placements matched by trine.
    pub trine: usize,
    /// Placements matched by opposition.
    pub opposition: usize,
}

impl AspectSummary {
    fn from_matches(matches: &[AspectMatch]) -> Self {
        let mut summary = Self::default();

        for aspect in matches {
            summary.increment(aspect.kind);
        }

        summary
    }

    fn increment(&mut self, kind: AspectKind) {
        match kind {
            AspectKind::Conjunction => self.conjunction += 1,
            AspectKind::Sextile => self.sextile += 1,
            AspectKind::Square => self.square += 1,
            AspectKind::Trine => self.trine += 1,
            AspectKind::Opposition => self.opposition += 1,
        }
    }

    /// Returns `true` when the snapshot contains at least one major aspect match.
    pub fn has_known_aspects(self) -> bool {
        self.conjunction + self.sextile + self.square + self.trine + self.opposition > 0
    }
}

impl fmt::Display for AspectSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut wrote_any = false;
        for (count, aspect) in [
            (self.conjunction, AspectKind::Conjunction),
            (self.sextile, AspectKind::Sextile),
            (self.square, AspectKind::Square),
            (self.trine, AspectKind::Trine),
            (self.opposition, AspectKind::Opposition),
        ] {
            if count == 0 {
                continue;
            }
            if wrote_any {
                f.write_str(", ")?;
            }
            wrote_any = true;
            write!(f, "{} {}", count, aspect)?;
        }

        if !wrote_any {
            f.write_str("no major aspects")?;
        }

        Ok(())
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
            "Instant: {} ({})",
            self.instant.julian_day, self.instant.scale
        )?;
        writeln!(
            f,
            "Time-scale policy: caller-supplied instant scale; no built-in Delta T or relativistic model"
        )?;
        if let Some(observer) = &self.observer {
            writeln!(
                f,
                "Observer: {} / {}",
                observer.latitude, observer.longitude
            )?;
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
        writeln!(
            f,
            "Frame policy: ecliptic body positions are requested from the backend; house calculations remain separate"
        )?;
        writeln!(f, "Zodiac mode: {}", self.zodiac_mode)?;
        writeln!(f, "Apparentness: {}", self.apparentness)?;
        writeln!(
            f,
            "Apparentness policy: {}",
            match self.apparentness {
                Apparentness::Mean => {
                    "mean geometric request by default; apparent corrections require backend support"
                }
                Apparentness::Apparent => {
                    "apparent request explicitly requested; backend support required"
                }
                _ => "requested apparentness is backend-dependent",
            }
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
                "  - {:<12} {:>9}  {:<10}  {:>3}  {:<10}  {}",
                placement.body, longitude, sign, house, motion, placement.position.quality,
            )?;
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
            writeln!(f, "Aspect summary: {}", aspect_summary)?;
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

fn house_ordinal(house: usize) -> &'static str {
    match house {
        1 => "1st",
        2 => "2nd",
        3 => "3rd",
        4 => "4th",
        5 => "5th",
        6 => "6th",
        7 => "7th",
        8 => "8th",
        9 => "9th",
        10 => "10th",
        11 => "11th",
        12 => "12th",
        _ => "unknown",
    }
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
        pleiades_houses::HouseErrorKind::InvalidObliquity => EphemerisErrorKind::InvalidRequest,
        pleiades_houses::HouseErrorKind::NumericalFailure => EphemerisErrorKind::NumericalFailure,
        _ => EphemerisErrorKind::InvalidRequest,
    };

    EphemerisError::new(kind, error.message)
}

impl<B: EphemerisBackend> ChartEngine<B> {
    /// Assembles a basic chart snapshot from the backend.
    ///
    /// # Example
    ///
    /// ```
    /// use pleiades_backend::{
    ///     AccuracyClass, Apparentness, BackendCapabilities, BackendFamily, BackendId,
    ///     BackendMetadata, BackendProvenance, EphemerisBackend, EphemerisError, EphemerisRequest,
    ///     EphemerisResult, QualityAnnotation, TimeRange,
    /// };
    /// use pleiades_core::{ChartEngine, ChartRequest};
    /// use pleiades_types::{
    ///     Angle, CelestialBody, CoordinateFrame, EclipticCoordinates, HouseSystem, Instant,
    ///     JulianDay, Latitude, Longitude, ObserverLocation, TimeScale, ZodiacSign,
    /// };
    ///
    /// struct DemoBackend;
    ///
    /// impl EphemerisBackend for DemoBackend {
    ///     fn metadata(&self) -> BackendMetadata {
    ///         BackendMetadata {
    ///             id: BackendId::new("demo"),
    ///             version: "0.1.0".to_string(),
    ///             family: BackendFamily::Algorithmic,
    ///             provenance: BackendProvenance::new("demo chart backend"),
    ///             nominal_range: TimeRange::new(None, None),
    ///             supported_time_scales: vec![TimeScale::Tt],
    ///             body_coverage: vec![CelestialBody::Sun],
    ///             supported_frames: vec![CoordinateFrame::Ecliptic],
    ///             capabilities: BackendCapabilities::default(),
    ///             accuracy: AccuracyClass::Approximate,
    ///             deterministic: true,
    ///             offline: true,
    ///         }
    ///     }
    ///
    ///     fn supports_body(&self, body: CelestialBody) -> bool {
    ///         body == CelestialBody::Sun
    ///     }
    ///
    ///     fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
    ///         let mut result = EphemerisResult::new(
    ///             BackendId::new("demo"),
    ///             request.body.clone(),
    ///             request.instant,
    ///             request.frame,
    ///             request.zodiac_mode.clone(),
    ///             request.apparent,
    ///         );
    ///         let ecliptic = EclipticCoordinates::new(
    ///             Longitude::from_degrees(15.0),
    ///             Latitude::from_degrees(0.0),
    ///             Some(1.0),
    ///         );
    ///         result.ecliptic = Some(ecliptic);
    ///         result.equatorial = Some(ecliptic.to_equatorial(Angle::from_degrees(23.4)));
    ///         result.motion = Some(pleiades_types::Motion::new(Some(1.0), None, None));
    ///         result.quality = QualityAnnotation::Exact;
    ///         Ok(result)
    ///     }
    /// }
    ///
    /// let request = ChartRequest::new(Instant::new(
    ///     JulianDay::from_days(2_451_545.0),
    ///     TimeScale::Utc,
    /// ))
    /// .with_tt_from_utc_signed(64.184)
    /// .expect("explicit UTC-to-TT conversion")
    /// .with_observer(ObserverLocation::new(
    ///     Latitude::from_degrees(51.5),
    ///     Longitude::from_degrees(-0.1),
    ///     None,
    /// ))
    /// .with_house_system(HouseSystem::WholeSign)
    /// .with_bodies(vec![CelestialBody::Sun]);
    ///
    /// let snapshot = ChartEngine::new(DemoBackend)
    ///     .chart(&request)
    ///     .expect("demo chart should assemble");
    ///
    /// assert_eq!(snapshot.backend_id.as_str(), "demo");
    /// assert_eq!(snapshot.zodiac_mode, pleiades_types::ZodiacMode::Tropical);
    /// assert_eq!(snapshot.apparentness, Apparentness::Mean);
    /// assert_eq!(snapshot.sign_for_body(&CelestialBody::Sun), Some(ZodiacSign::Aries));
    /// assert!(snapshot.houses.is_some());
    /// assert_eq!(snapshot.placements.len(), 1);
    /// ```
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
                    observer: None,
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
            apparentness: request.apparentness,
            houses,
            placements,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    use pleiades_backend::{
        AccuracyClass, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
        BackendProvenance, QualityAnnotation,
    };
    use pleiades_types::{EclipticCoordinates, Latitude, Longitude, ObserverLocation, TimeScale};

    struct ToyChartBackend;

    #[derive(Clone)]
    struct RecordingChartBackend {
        observers: Arc<Mutex<Vec<Option<ObserverLocation>>>>,
    }

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
            if request.observer.is_some() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidObserver,
                    "toy chart backend is geocentric only",
                ));
            }

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

    impl EphemerisBackend for RecordingChartBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("recording-chart"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("recording chart backend"),
                nominal_range: pleiades_types::TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_coverage: vec![CelestialBody::Sun],
                supported_frames: vec![pleiades_types::CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities::default(),
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            matches!(body, CelestialBody::Sun)
        }

        fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            self.observers
                .lock()
                .expect("observer log should be lockable")
                .push(request.observer.clone());
            let mut result = EphemerisResult::new(
                BackendId::new("recording-chart"),
                request.body.clone(),
                request.instant,
                request.frame,
                request.zodiac_mode.clone(),
                request.apparent,
            );
            result.ecliptic = Some(EclipticCoordinates::new(
                Longitude::from_degrees(15.0),
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
        assert_eq!(chart.sign_summary().aries, 1);
        assert_eq!(chart.sign_summary().taurus, 1);
        let rendered = chart.to_string();
        assert!(rendered.contains("Sun"));
        assert!(rendered.contains("Moon"));
        assert!(rendered.contains("Sign summary: 1 Aries, 1 Taurus"));
        assert!(rendered.contains("Instant: JD 2451545 (TT)"));
        assert!(rendered.contains(
            "Frame policy: ecliptic body positions are requested from the backend; house calculations remain separate"
        ));
        assert!(rendered.contains("Apparentness policy: mean geometric request by default; apparent corrections require backend support"));
    }

    #[test]
    fn chart_snapshot_without_observer_renders_geocentric_policy_line() {
        let instant = Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        );
        let chart = ChartSnapshot {
            backend_id: BackendId::new("toy-chart"),
            instant,
            observer: None,
            zodiac_mode: ZodiacMode::Tropical,
            apparentness: Apparentness::Mean,
            houses: None,
            placements: vec![],
        };

        let rendered = chart.to_string();
        assert!(rendered
            .contains("Observer policy: geocentric body positions; no house observer supplied"));
        assert!(!rendered
            .contains("Observer policy: used for houses only; body positions remain geocentric"));
    }

    #[test]
    fn chart_snapshot_exposes_dominant_sign_and_house_summaries() {
        let instant = Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        );

        let chart = ChartSnapshot {
            backend_id: BackendId::new("toy-chart"),
            instant,
            observer: None,
            zodiac_mode: ZodiacMode::Tropical,
            apparentness: Apparentness::Apparent,
            houses: None,
            placements: vec![
                BodyPlacement {
                    body: CelestialBody::Sun,
                    position: EphemerisResult::new(
                        BackendId::new("toy-chart"),
                        CelestialBody::Sun,
                        instant,
                        pleiades_types::CoordinateFrame::Ecliptic,
                        ZodiacMode::Tropical,
                        Apparentness::Apparent,
                    ),
                    sign: Some(ZodiacSign::Aries),
                    house: Some(1),
                },
                BodyPlacement {
                    body: CelestialBody::Moon,
                    position: EphemerisResult::new(
                        BackendId::new("toy-chart"),
                        CelestialBody::Moon,
                        instant,
                        pleiades_types::CoordinateFrame::Ecliptic,
                        ZodiacMode::Tropical,
                        Apparentness::Apparent,
                    ),
                    sign: Some(ZodiacSign::Aries),
                    house: Some(1),
                },
                BodyPlacement {
                    body: CelestialBody::Mars,
                    position: EphemerisResult::new(
                        BackendId::new("toy-chart"),
                        CelestialBody::Mars,
                        instant,
                        pleiades_types::CoordinateFrame::Ecliptic,
                        ZodiacMode::Tropical,
                        Apparentness::Apparent,
                    ),
                    sign: Some(ZodiacSign::Taurus),
                    house: Some(2),
                },
                BodyPlacement {
                    body: CelestialBody::Mercury,
                    position: EphemerisResult::new(
                        BackendId::new("toy-chart"),
                        CelestialBody::Mercury,
                        instant,
                        pleiades_types::CoordinateFrame::Ecliptic,
                        ZodiacMode::Tropical,
                        Apparentness::Apparent,
                    ),
                    sign: Some(ZodiacSign::Taurus),
                    house: Some(2),
                },
                BodyPlacement {
                    body: CelestialBody::Jupiter,
                    position: EphemerisResult::new(
                        BackendId::new("toy-chart"),
                        CelestialBody::Jupiter,
                        instant,
                        pleiades_types::CoordinateFrame::Ecliptic,
                        ZodiacMode::Tropical,
                        Apparentness::Apparent,
                    ),
                    sign: Some(ZodiacSign::Gemini),
                    house: Some(8),
                },
            ],
        };

        assert_eq!(
            chart.dominant_sign_summary(),
            SignSummary {
                aries: 2,
                taurus: 2,
                gemini: 0,
                cancer: 0,
                leo: 0,
                virgo: 0,
                libra: 0,
                scorpio: 0,
                sagittarius: 0,
                capricorn: 0,
                aquarius: 0,
                pisces: 0,
            }
        );
        assert_eq!(
            chart.dominant_house_summary(),
            HouseSummary {
                first: 2,
                second: 2,
                third: 0,
                fourth: 0,
                fifth: 0,
                sixth: 0,
                seventh: 0,
                eighth: 0,
                ninth: 0,
                tenth: 0,
                eleventh: 0,
                twelfth: 0,
                unknown: 0,
            }
        );

        let rendered = chart.to_string();
        assert!(rendered.contains("Dominant sign summary: 2 Aries, 2 Taurus"));
        assert!(rendered.contains("Dominant house summary: 2 in 1st house, 2 in 2nd house"));
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
        let rendered = chart.to_string();
        assert!(rendered.contains("Zodiac mode: Sidereal (Lahiri)"));
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
        assert_eq!(chart.apparentness, Apparentness::Apparent);
        let rendered = chart.to_string();
        assert!(rendered.contains("Apparentness: Apparent"));
        assert!(rendered.contains(
            "Apparentness policy: apparent request explicitly requested; backend support required"
        ));
        assert!(rendered.contains("Time-scale policy: caller-supplied instant scale; no built-in Delta T or relativistic model"));
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
        let rendered = chart.to_string();
        assert!(rendered.contains("House system: Whole Sign"));
        assert!(rendered
            .contains("Observer policy: used for houses only; body positions remain geocentric"));
    }

    #[test]
    fn chart_snapshot_keeps_house_observer_out_of_body_position_requests() {
        let observers = Arc::new(Mutex::new(Vec::new()));
        let engine = ChartEngine::new(RecordingChartBackend {
            observers: Arc::clone(&observers),
        });
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        ))
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(12.5),
            Longitude::from_degrees(45.0),
            Some(125.0),
        ))
        .with_house_system(crate::HouseSystem::WholeSign)
        .with_bodies(vec![CelestialBody::Sun]);

        let chart = engine
            .chart(&request)
            .expect("chart should render with a house observer");

        assert!(chart.houses.is_some());
        assert_eq!(chart.observer, request.observer);
        let observers = observers.lock().expect("observer log should be lockable");
        assert_eq!(observers.len(), 1);
        assert!(observers.iter().all(Option::is_none));
    }

    #[test]
    fn chart_request_can_apply_a_caller_supplied_time_scale_offset() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Ut1,
        ))
        .with_instant_time_scale_offset(TimeScale::Tt, 64.184);

        assert_eq!(request.instant.scale, TimeScale::Tt);
        let expected = 2_451_545.0 + 64.184 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_ut1_to_tt() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Ut1,
        ))
        .with_tt_from_ut1(Duration::from_secs_f64(64.184))
        .expect("UT1 chart request should convert to TT");

        assert_eq!(request.instant.scale, TimeScale::Tt);
        let expected = 2_451_545.0 + 64.184 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_ut1_to_tt_with_signed_offset() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Ut1,
        ))
        .with_tt_from_ut1_signed(64.184)
        .expect("UT1 chart request should accept signed TT offsets");

        assert_eq!(request.instant.scale, TimeScale::Tt);
        let expected = 2_451_545.0 + 64.184 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_utc_to_tt() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Utc,
        ))
        .with_tt_from_utc(Duration::from_secs_f64(64.184))
        .expect("UTC chart request should convert to TT");

        assert_eq!(request.instant.scale, TimeScale::Tt);
        let expected = 2_451_545.0 + 64.184 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_utc_to_tt_with_signed_offset() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Utc,
        ))
        .with_tt_from_utc_signed(64.184)
        .expect("UTC chart request should accept signed TT offsets");

        assert_eq!(request.instant.scale, TimeScale::Tt);
        let expected = 2_451_545.0 + 64.184 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_tt_to_tdb() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .with_tdb_from_tt(Duration::from_secs_f64(0.001_657))
        .expect("TT chart request should convert to TDB");

        assert_eq!(request.instant.scale, TimeScale::Tdb);
        let expected = 2_451_545.0 + 0.001_657 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_tt_to_tdb_with_signed_offset() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .with_tdb_from_tt_signed(-0.001_657)
        .expect("TT chart request should accept signed TDB offsets");

        assert_eq!(request.instant.scale, TimeScale::Tdb);
        let expected = 2_451_545.0 - 0.001_657 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_tdb_to_tt() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tdb,
        ))
        .with_tt_from_tdb(-0.001_657)
        .expect("TDB chart request should convert to TT");

        assert_eq!(request.instant.scale, TimeScale::Tt);
        let expected = 2_451_545.0 - 0.001_657 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_tdb_to_tt_with_signed_offset() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tdb,
        ))
        .with_tt_from_tdb_signed(-0.001_657)
        .expect("TDB chart request should accept signed TT offsets");

        assert_eq!(request.instant.scale, TimeScale::Tt);
        let expected = 2_451_545.0 - 0.001_657 / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_ut1_to_tdb() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Ut1,
        ))
        .with_tdb_from_ut1(
            Duration::from_secs_f64(64.184),
            Duration::from_secs_f64(0.001_657),
        )
        .expect("UT1 chart request should convert to TDB");

        assert_eq!(request.instant.scale, TimeScale::Tdb);
        let expected = 2_451_545.0 + (64.184 + 0.001_657) / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_ut1_to_tdb_with_signed_offset() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Ut1,
        ))
        .with_tdb_from_ut1_signed(Duration::from_secs_f64(64.184), -0.001_657)
        .expect("UT1 chart request should accept signed TDB offsets");

        assert_eq!(request.instant.scale, TimeScale::Tdb);
        let expected = 2_451_545.0 + (64.184 - 0.001_657) / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_utc_to_tdb() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Utc,
        ))
        .with_tdb_from_utc(
            Duration::from_secs_f64(64.184),
            Duration::from_secs_f64(0.001_657),
        )
        .expect("UTC chart request should convert to TDB");

        assert_eq!(request.instant.scale, TimeScale::Tdb);
        let expected = 2_451_545.0 + (64.184 + 0.001_657) / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    }

    #[test]
    fn chart_request_can_convert_utc_to_tdb_with_signed_offset() {
        let request = ChartRequest::new(Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Utc,
        ))
        .with_tdb_from_utc_signed(Duration::from_secs_f64(64.184), -0.001_657)
        .expect("UTC chart request should accept signed TDB offsets");

        assert_eq!(request.instant.scale, TimeScale::Tdb);
        let expected = 2_451_545.0 + (64.184 - 0.001_657) / 86_400.0;
        assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
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

        let mut unknown = EphemerisResult::new(
            BackendId::new("toy-chart"),
            CelestialBody::Jupiter,
            instant,
            pleiades_types::CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Apparent,
        );
        unknown.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(105.0),
            Latitude::from_degrees(0.0),
            None,
        ));

        let chart = ChartSnapshot {
            backend_id: BackendId::new("toy-chart"),
            instant,
            observer: None,
            zodiac_mode: ZodiacMode::Tropical,
            apparentness: Apparentness::Apparent,
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
                BodyPlacement {
                    body: CelestialBody::Jupiter,
                    position: unknown,
                    sign: Some(ZodiacSign::Leo),
                    house: Some(9),
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
            chart.occupied_signs(),
            vec![
                ZodiacSign::Aries,
                ZodiacSign::Taurus,
                ZodiacSign::Cancer,
                ZodiacSign::Leo,
            ]
        );
        assert_eq!(chart.occupied_houses(), vec![1, 2, 8, 9]);
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
        assert_eq!(chart.unknown_motion_placements().count(), 1);
        assert_eq!(chart.retrograde_placements().count(), 1);
        assert_eq!(
            chart.house_summary(),
            HouseSummary {
                first: 1,
                second: 1,
                third: 0,
                fourth: 0,
                fifth: 0,
                sixth: 0,
                seventh: 0,
                eighth: 1,
                ninth: 1,
                tenth: 0,
                eleventh: 0,
                twelfth: 0,
                unknown: 0,
            }
        );
        assert_eq!(
            chart.motion_summary(),
            MotionSummary {
                direct: 1,
                stationary: 1,
                retrograde: 1,
                unknown: 1,
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
        assert!(rendered.contains(
            "House summary: 1 in 1st house, 1 in 2nd house, 1 in 8th house, 1 in 9th house"
        ));
        assert!(
            rendered.contains("Motion summary: 1 direct, 1 stationary, 1 retrograde, 1 unknown")
        );
        assert!(rendered.contains("Stationary bodies: Mercury"));
        assert!(rendered.contains("Unknown motion bodies: Jupiter"));
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
            apparentness: Apparentness::Apparent,
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
        assert_eq!(
            chart.aspect_summary(),
            AspectSummary {
                conjunction: 0,
                sextile: 1,
                square: 0,
                trine: 0,
                opposition: 0,
            }
        );
        let rendered = chart.to_string();
        assert!(rendered.contains("Aspect summary: 1 Sextile"));
        assert!(rendered.contains("Aspects:"));
        assert!(rendered.contains("Sun Sextile Moon"));
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
            apparentness: Apparentness::Apparent,
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
