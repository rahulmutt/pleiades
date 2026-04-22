//! Shared primitive and domain-adjacent types used across the workspace.
//!
//! These types define the vocabulary for angles, time scales, celestial body
//! identifiers, observer locations, coordinate frames, and catalog selections.
//! Higher-level crates build on these semantics without re-labelling the same
//! concepts in backend-specific ways.
//!
//! # Examples
//!
//! ```
//! use pleiades_types::{Angle, Longitude};
//!
//! let angle = Angle::from_degrees(-30.0);
//! assert_eq!(angle.normalized_0_360().degrees(), 330.0);
//!
//! let lon = Longitude::from_degrees(390.0);
//! assert_eq!(lon.degrees(), 30.0);
//! ```

#![forbid(unsafe_code)]

use core::fmt;

/// An angular quantity measured in degrees.
///
/// `Angle` is intentionally neutral: it does not assume a normalization range.
/// Use [`Angle::normalized_0_360`] or [`Angle::normalized_signed`] when a
/// canonical wrap is required.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Angle(f64);

impl Angle {
    /// Creates a new angle measured in degrees.
    pub const fn from_degrees(degrees: f64) -> Self {
        Self(degrees)
    }

    /// Creates a new angle measured in radians.
    pub fn from_radians(radians: f64) -> Self {
        Self(radians.to_degrees())
    }

    /// Returns the underlying angle in degrees.
    pub const fn degrees(self) -> f64 {
        self.0
    }

    /// Returns the angle in radians.
    pub fn radians(self) -> f64 {
        self.0.to_radians()
    }

    /// Returns the angle normalized into the half-open range `[0, 360)`.
    pub fn normalized_0_360(self) -> Self {
        Self(self.0.rem_euclid(360.0))
    }

    /// Returns the angle normalized into the signed range `[-180, 180)`.
    pub fn normalized_signed(self) -> Self {
        let degrees = self.normalized_0_360().degrees();
        if degrees >= 180.0 {
            Self(degrees - 360.0)
        } else {
            Self(degrees)
        }
    }

    /// Returns `true` when the underlying numeric value is finite.
    pub const fn is_finite(self) -> bool {
        self.0.is_finite()
    }
}

impl fmt::Display for Angle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}°", self.0)
    }
}

/// A canonical ecliptic or longitude-like angle normalized into `[0, 360)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Longitude(Angle);

impl Longitude {
    /// Creates a longitude normalized into `[0, 360)`.
    pub fn from_degrees(degrees: f64) -> Self {
        Self(Angle::from_degrees(degrees).normalized_0_360())
    }

    /// Returns the longitude in degrees, already normalized into `[0, 360)`.
    pub const fn degrees(self) -> f64 {
        self.0.degrees()
    }

    /// Returns the underlying angle wrapper.
    pub const fn angle(self) -> Angle {
        self.0
    }
}

impl From<Angle> for Longitude {
    fn from(value: Angle) -> Self {
        Self::from_degrees(value.degrees())
    }
}

impl fmt::Display for Longitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// A signed latitude-like angle measured in degrees.
///
/// Latitude values are not automatically clamped; the caller is expected to
/// provide values consistent with the relevant coordinate system.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Latitude(Angle);

impl Latitude {
    /// Creates a latitude measured in degrees.
    pub const fn from_degrees(degrees: f64) -> Self {
        Self(Angle::from_degrees(degrees))
    }

    /// Returns the latitude in degrees.
    pub const fn degrees(self) -> f64 {
        self.0.degrees()
    }

    /// Returns the underlying angle wrapper.
    pub const fn angle(self) -> Angle {
        self.0
    }
}

impl From<Angle> for Latitude {
    fn from(value: Angle) -> Self {
        Self::from_degrees(value.degrees())
    }
}

impl fmt::Display for Latitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// A Julian day expressed as a floating-point day count.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct JulianDay(f64);

impl JulianDay {
    /// Creates a new Julian day value.
    pub const fn from_days(days: f64) -> Self {
        Self(days)
    }

    /// Returns the raw floating-point day count.
    pub const fn days(self) -> f64 {
        self.0
    }
}

impl fmt::Display for JulianDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JD {}", self.0)
    }
}

/// A supported astronomical time scale.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum TimeScale {
    /// Coordinated Universal Time.
    Utc,
    /// Universal Time 1.
    Ut1,
    /// Terrestrial Time.
    Tt,
    /// Barycentric Dynamical Time.
    Tdb,
}

/// A Julian day tagged with a time scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Instant {
    /// The numeric Julian day value.
    pub julian_day: JulianDay,
    /// The time scale used by the Julian day value.
    pub scale: TimeScale,
}

impl Instant {
    /// Creates a new instant from a Julian day and time scale.
    pub const fn new(julian_day: JulianDay, scale: TimeScale) -> Self {
        Self { julian_day, scale }
    }
}

/// A geographic observer location.
#[derive(Clone, Debug, PartialEq)]
pub struct ObserverLocation {
    /// Geographic latitude.
    pub latitude: Latitude,
    /// Geographic longitude, expressed in degrees east of Greenwich.
    pub longitude: Longitude,
    /// Optional elevation above sea level in meters.
    pub elevation_m: Option<f64>,
}

impl ObserverLocation {
    /// Creates a new observer location.
    pub const fn new(latitude: Latitude, longitude: Longitude, elevation_m: Option<f64>) -> Self {
        Self {
            latitude,
            longitude,
            elevation_m,
        }
    }
}

/// The coordinate frame requested from a backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum CoordinateFrame {
    /// Ecliptic longitude/latitude coordinates.
    Ecliptic,
    /// Equatorial right ascension/declination coordinates.
    Equatorial,
}

/// Whether coordinates should be interpreted in tropical or sidereal mode.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ZodiacMode {
    /// Tropical zodiac.
    Tropical,
    /// Sidereal zodiac using the selected ayanamsa definition.
    Sidereal { ayanamsa: Ayanamsa },
}

/// One of the twelve zodiac signs.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum ZodiacSign {
    /// Aries 0°–30°.
    Aries,
    /// Taurus 30°–60°.
    Taurus,
    /// Gemini 60°–90°.
    Gemini,
    /// Cancer 90°–120°.
    Cancer,
    /// Leo 120°–150°.
    Leo,
    /// Virgo 150°–180°.
    Virgo,
    /// Libra 180°–210°.
    Libra,
    /// Scorpio 210°–240°.
    Scorpio,
    /// Sagittarius 240°–270°.
    Sagittarius,
    /// Capricorn 270°–300°.
    Capricorn,
    /// Aquarius 300°–330°.
    Aquarius,
    /// Pisces 330°–360°.
    Pisces,
}

impl ZodiacSign {
    /// Returns the sign corresponding to a normalized ecliptic longitude.
    pub fn from_longitude(longitude: Longitude) -> Self {
        match (longitude.degrees() / 30.0).floor() as usize % 12 {
            0 => Self::Aries,
            1 => Self::Taurus,
            2 => Self::Gemini,
            3 => Self::Cancer,
            4 => Self::Leo,
            5 => Self::Virgo,
            6 => Self::Libra,
            7 => Self::Scorpio,
            8 => Self::Sagittarius,
            9 => Self::Capricorn,
            10 => Self::Aquarius,
            _ => Self::Pisces,
        }
    }

    /// Returns the sign's display name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Aries => "Aries",
            Self::Taurus => "Taurus",
            Self::Gemini => "Gemini",
            Self::Cancer => "Cancer",
            Self::Leo => "Leo",
            Self::Virgo => "Virgo",
            Self::Libra => "Libra",
            Self::Scorpio => "Scorpio",
            Self::Sagittarius => "Sagittarius",
            Self::Capricorn => "Capricorn",
            Self::Aquarius => "Aquarius",
            Self::Pisces => "Pisces",
        }
    }
}

impl fmt::Display for ZodiacSign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Whether a backend should prefer apparent or mean values where both exist.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum Apparentness {
    /// Apparent values, including light-time and related corrections when available.
    Apparent,
    /// Mean values.
    Mean,
}

/// The built-in and custom body identifiers recognized by the shared API.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CelestialBody {
    /// The Sun.
    Sun,
    /// The Moon.
    Moon,
    /// Mercury.
    Mercury,
    /// Venus.
    Venus,
    /// Mars.
    Mars,
    /// Jupiter.
    Jupiter,
    /// Saturn.
    Saturn,
    /// Uranus.
    Uranus,
    /// Neptune.
    Neptune,
    /// Pluto.
    Pluto,
    /// The mean lunar node.
    MeanNode,
    /// The true lunar node.
    TrueNode,
    /// The mean lunar apogee.
    MeanApogee,
    /// The true lunar apogee.
    TrueApogee,
    /// Ceres.
    Ceres,
    /// Pallas.
    Pallas,
    /// Juno.
    Juno,
    /// Vesta.
    Vesta,
    /// A body that is not yet one of the built-in identifiers.
    Custom(CustomBodyId),
}

impl CelestialBody {
    /// Returns a stable human-readable name for built-in bodies.
    pub const fn built_in_name(&self) -> Option<&'static str> {
        match self {
            Self::Sun => Some("Sun"),
            Self::Moon => Some("Moon"),
            Self::Mercury => Some("Mercury"),
            Self::Venus => Some("Venus"),
            Self::Mars => Some("Mars"),
            Self::Jupiter => Some("Jupiter"),
            Self::Saturn => Some("Saturn"),
            Self::Uranus => Some("Uranus"),
            Self::Neptune => Some("Neptune"),
            Self::Pluto => Some("Pluto"),
            Self::MeanNode => Some("Mean Node"),
            Self::TrueNode => Some("True Node"),
            Self::MeanApogee => Some("Mean Apogee"),
            Self::TrueApogee => Some("True Apogee"),
            Self::Ceres => Some("Ceres"),
            Self::Pallas => Some("Pallas"),
            Self::Juno => Some("Juno"),
            Self::Vesta => Some("Vesta"),
            Self::Custom(_) => None,
        }
    }
}

/// A structured identifier for a custom body.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CustomBodyId {
    /// A coarse namespace for the body source, such as `asteroid` or `hypothetical`.
    pub catalog: String,
    /// The designation within the namespace.
    pub designation: String,
}

impl CustomBodyId {
    /// Creates a new custom body identifier.
    pub fn new(catalog: impl Into<String>, designation: impl Into<String>) -> Self {
        Self {
            catalog: catalog.into(),
            designation: designation.into(),
        }
    }
}

impl fmt::Display for CustomBodyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.catalog, self.designation)
    }
}

/// A built-in or custom house system selection.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HouseSystem {
    /// Placidus.
    Placidus,
    /// Koch.
    Koch,
    /// Porphyry.
    Porphyry,
    /// Regiomontanus.
    Regiomontanus,
    /// Campanus.
    Campanus,
    /// Carter (poli-equatorial) houses.
    Carter,
    /// Horizon/Azimuth houses.
    Horizon,
    /// APC houses.
    Apc,
    /// Krusinski-Pisa-Goelzer houses.
    KrusinskiPisaGoelzer,
    /// Equal houses.
    Equal,
    /// Equal houses with the Midheaven on cusp 10.
    EqualMidheaven,
    /// Equal houses with the first house anchored at 0° Aries.
    EqualAries,
    /// Vehlow equal houses, with the Ascendant centered in house 1.
    Vehlow,
    /// Sripati houses.
    Sripati,
    /// Whole sign houses.
    WholeSign,
    /// Alcabitius.
    Alcabitius,
    /// Albategnius / Savard-A.
    Albategnius,
    /// Pullen sinusoidal delta (Neo-Porphyry).
    PullenSd,
    /// Pullen sinusoidal ratio.
    PullenSr,
    /// Meridian-style systems.
    Meridian,
    /// Axial variants documented by specific software.
    Axial,
    /// Topocentric (Polich-Page).
    Topocentric,
    /// Morinus.
    Morinus,
    /// Sunshine (Bob Makransky / Dieter Treindl family).
    Sunshine,
    /// Gauquelin sectors.
    Gauquelin,
    /// A custom house system definition.
    Custom(CustomHouseSystem),
}

/// A structured custom house-system definition.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CustomHouseSystem {
    /// Stable human-readable name.
    pub name: String,
    /// Optional alternative names or aliases.
    pub aliases: Vec<String>,
    /// Optional notes about formula, assumptions, or limits.
    pub notes: Option<String>,
}

impl CustomHouseSystem {
    /// Creates a custom house-system definition.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            aliases: Vec::new(),
            notes: None,
        }
    }
}

/// A built-in or custom ayanamsa selection.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Ayanamsa {
    /// Lahiri.
    Lahiri,
    /// Raman.
    Raman,
    /// Krishnamurti.
    Krishnamurti,
    /// Fagan/Bradley.
    FaganBradley,
    /// True Chitra.
    TrueChitra,
    /// J2000.0 reference-frame mode.
    J2000,
    /// J1900.0 reference-frame mode.
    J1900,
    /// B1950.0 reference-frame mode.
    B1950,
    /// True Revati.
    TrueRevati,
    /// True Mula.
    TrueMula,
    /// True Pushya.
    TruePushya,
    /// Djwhal Khul.
    DjwhalKhul,
    /// J. N. Bhasin.
    JnBhasin,
    /// Suryasiddhanta mean-sun variant.
    Suryasiddhanta499MeanSun,
    /// Aryabhata mean-sun variant.
    Aryabhata499MeanSun,
    /// The 1956 Indian Astronomical Ephemeris / ICRC Lahiri definition.
    LahiriIcrc,
    /// Lahiri's 1940 zero-date variant.
    Lahiri1940,
    /// Usha/Shashi, anchored to the Revati tradition.
    UshaShashi,
    /// Suryasiddhanta-equinox variant anchored in 499 CE.
    Suryasiddhanta499,
    /// Aryabhata-equinox variant anchored in 499 CE.
    Aryabhata499,
    /// Sassanian zero-point variant anchored in 564 CE.
    Sassanian,
    /// DeLuce ayanamsa.
    DeLuce,
    /// Yukteshwar ayanamsa.
    Yukteshwar,
    /// P.V.R. Narasimha Rao's Pushya-paksha ayanamsa.
    PvrPushyaPaksha,
    /// Sheoran ayanamsa.
    Sheoran,
    /// Hipparchus / Hipparchos ayanamsa.
    Hipparchus,
    /// Babylonian (Kugler 1).
    BabylonianKugler1,
    /// Babylonian (Kugler 2).
    BabylonianKugler2,
    /// Babylonian (Kugler 3).
    BabylonianKugler3,
    /// Babylonian (Huber).
    BabylonianHuber,
    /// Babylonian (Eta Piscium).
    BabylonianEtaPiscium,
    /// Babylonian (Aldebaran / 15 Tau).
    BabylonianAldebaran,
    /// Babylonian (Britton).
    BabylonianBritton,
    /// Aryabhata (522 CE).
    Aryabhata522,
    /// Lahiri (VP285).
    LahiriVP285,
    /// Krishnamurti (VP291).
    KrishnamurtiVP291,
    /// True Sheoran.
    TrueSheoran,
    /// Galactic Center.
    GalacticCenter,
    /// Galactic Center (Rgilbrand).
    GalacticCenterRgilbrand,
    /// Galactic Center (Mardyks).
    GalacticCenterMardyks,
    /// Galactic Center (Mula/Wilhelm).
    GalacticCenterMulaWilhelm,
    /// Galactic Center (Cochrane).
    GalacticCenterCochrane,
    /// Galactic Equator.
    GalacticEquator,
    /// Galactic Equator (IAU 1958).
    GalacticEquatorIau1958,
    /// Galactic Equator (True).
    GalacticEquatorTrue,
    /// Galactic Equator (Mula).
    GalacticEquatorMula,
    /// Galactic Equator (Fiorenza).
    GalacticEquatorFiorenza,
    /// Valens Moon.
    ValensMoon,
    /// A custom ayanamsa formula or offset table.
    Custom(CustomAyanamsa),
}

/// A structured custom ayanamsa definition.
#[derive(Clone, Debug, PartialEq)]
pub struct CustomAyanamsa {
    /// Stable human-readable name.
    pub name: String,
    /// Optional description of the formula or offset policy.
    pub description: Option<String>,
    /// Optional epoch the definition is tied to.
    pub epoch: Option<JulianDay>,
    /// Optional fixed offset in degrees for simple offset-based variants.
    pub offset_degrees: Option<Angle>,
}

impl CustomAyanamsa {
    /// Creates a custom ayanamsa definition.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            epoch: None,
            offset_degrees: None,
        }
    }
}

/// Ecliptic position data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EclipticCoordinates {
    /// Ecliptic longitude.
    pub longitude: Longitude,
    /// Ecliptic latitude.
    pub latitude: Latitude,
    /// Distance in astronomical units when available.
    pub distance_au: Option<f64>,
}

impl EclipticCoordinates {
    /// Creates a new ecliptic coordinate sample.
    pub const fn new(longitude: Longitude, latitude: Latitude, distance_au: Option<f64>) -> Self {
        Self {
            longitude,
            latitude,
            distance_au,
        }
    }
}

/// Equatorial position data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EquatorialCoordinates {
    /// Right ascension.
    pub right_ascension: Angle,
    /// Declination.
    pub declination: Latitude,
    /// Distance in astronomical units when available.
    pub distance_au: Option<f64>,
}

impl EquatorialCoordinates {
    /// Creates a new equatorial coordinate sample.
    pub const fn new(
        right_ascension: Angle,
        declination: Latitude,
        distance_au: Option<f64>,
    ) -> Self {
        Self {
            right_ascension,
            declination,
            distance_au,
        }
    }
}

/// The coarse direction of longitudinal motion.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum MotionDirection {
    /// Motion is prograde or direct.
    Direct,
    /// Motion is effectively stationary at the chosen precision.
    Stationary,
    /// Motion is retrograde.
    Retrograde,
}

impl fmt::Display for MotionDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Direct => "Direct",
            Self::Stationary => "Stationary",
            Self::Retrograde => "Retrograde",
        };
        f.write_str(label)
    }
}

/// Apparent motion data for a position sample.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Motion {
    /// Longitude speed in degrees per day.
    pub longitude_deg_per_day: Option<f64>,
    /// Latitude speed in degrees per day.
    pub latitude_deg_per_day: Option<f64>,
    /// Distance speed in astronomical units per day.
    pub distance_au_per_day: Option<f64>,
}

impl Motion {
    /// Creates a new motion sample.
    pub const fn new(
        longitude_deg_per_day: Option<f64>,
        latitude_deg_per_day: Option<f64>,
        distance_au_per_day: Option<f64>,
    ) -> Self {
        Self {
            longitude_deg_per_day,
            latitude_deg_per_day,
            distance_au_per_day,
        }
    }

    /// Returns the coarse longitudinal motion direction when that speed is available.
    ///
    /// The classification is sign-based: positive speed is direct, negative speed is retrograde,
    /// and an exact zero speed is stationary.
    pub fn longitude_direction(self) -> Option<MotionDirection> {
        self.longitude_deg_per_day.map(|speed| {
            if speed > 0.0 {
                MotionDirection::Direct
            } else if speed < 0.0 {
                MotionDirection::Retrograde
            } else {
                MotionDirection::Stationary
            }
        })
    }
}

/// A Julian-day interval.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimeRange {
    /// Inclusive lower bound.
    pub start: Option<Instant>,
    /// Inclusive upper bound.
    pub end: Option<Instant>,
}

impl TimeRange {
    /// Creates a new time range.
    pub const fn new(start: Option<Instant>, end: Option<Instant>) -> Self {
        Self { start, end }
    }

    /// Returns `true` if the given instant is inside the range.
    pub fn contains(&self, instant: Instant) -> bool {
        let after_start = self.start.is_none_or(|start| {
            same_scale_and_jd(instant, start)
                && instant.julian_day.days() >= start.julian_day.days()
        });
        let before_end = self.end.is_none_or(|end| {
            same_scale_and_jd(instant, end) && instant.julian_day.days() <= end.julian_day.days()
        });
        after_start && before_end
    }
}

fn same_scale_and_jd(a: Instant, b: Instant) -> bool {
    a.scale == b.scale && a.julian_day.days().is_finite() && b.julian_day.days().is_finite()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_normalization_wraps_correctly() {
        assert_eq!(
            Angle::from_degrees(-30.0).normalized_0_360().degrees(),
            330.0
        );
        assert_eq!(
            Angle::from_degrees(390.0).normalized_0_360().degrees(),
            30.0
        );
        assert_eq!(
            Angle::from_degrees(190.0).normalized_signed().degrees(),
            -170.0
        );
    }

    #[test]
    fn longitude_is_always_normalized() {
        assert_eq!(Longitude::from_degrees(390.0).degrees(), 30.0);
        assert_eq!(Longitude::from(Angle::from_degrees(-30.0)).degrees(), 330.0);
    }

    #[test]
    fn built_in_body_names_are_stable() {
        assert_eq!(CelestialBody::Sun.built_in_name(), Some("Sun"));
        assert_eq!(
            CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")).built_in_name(),
            None
        );
    }

    #[test]
    fn zodiac_signs_follow_longitude_bands() {
        assert_eq!(
            ZodiacSign::from_longitude(Longitude::from_degrees(0.0)),
            ZodiacSign::Aries
        );
        assert_eq!(
            ZodiacSign::from_longitude(Longitude::from_degrees(29.999)),
            ZodiacSign::Aries
        );
        assert_eq!(
            ZodiacSign::from_longitude(Longitude::from_degrees(30.0)),
            ZodiacSign::Taurus
        );
    }

    #[test]
    fn time_range_checks_scale_and_julian_day() {
        let start = Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt);
        let end = Instant::new(JulianDay::from_days(2451546.0), TimeScale::Tt);
        let range = TimeRange::new(Some(start), Some(end));

        assert!(range.contains(Instant::new(JulianDay::from_days(2451545.5), TimeScale::Tt)));
        assert!(!range.contains(Instant::new(
            JulianDay::from_days(2451545.5),
            TimeScale::Utc
        )));
    }

    #[test]
    fn motion_direction_tracks_the_sign_of_longitude_speed() {
        assert_eq!(
            Motion::new(Some(0.12), None, None).longitude_direction(),
            Some(MotionDirection::Direct)
        );
        assert_eq!(
            Motion::new(Some(-0.04), None, None).longitude_direction(),
            Some(MotionDirection::Retrograde)
        );
        assert_eq!(
            Motion::new(Some(0.0), None, None).longitude_direction(),
            Some(MotionDirection::Stationary)
        );
        assert_eq!(Motion::new(None, None, None).longitude_direction(), None);
    }
}
