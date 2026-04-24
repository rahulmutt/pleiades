//! Ayanamsa catalog definitions and compatibility metadata.
//!
//! This crate currently focuses on the catalog layer: it enumerates the
//! baseline built-in ayanamsas, their common aliases, and notes about their
//! intended interoperability role. It also carries the baseline epoch/offset
//! metadata used by the chart-layer sidereal conversion helper, plus a first
//! stage-6 batch of historical anchor-point variants so the release profile can
//! distinguish baseline coverage from broader compatibility breadth.
//!
//! # Examples
//!
//! ```
//! use pleiades_ayanamsa::{baseline_ayanamsas, resolve_ayanamsa};
//!
//! let catalog = baseline_ayanamsas();
//! assert!(catalog.iter().any(|entry| entry.canonical_name == "Lahiri"));
//!
//! assert_eq!(resolve_ayanamsa("KP"), Some(pleiades_types::Ayanamsa::Krishnamurti));
//! assert_eq!(resolve_ayanamsa("Krishnamurti Paddhati"), Some(pleiades_types::Ayanamsa::Krishnamurti));
//! assert_eq!(resolve_ayanamsa("Krishnamurti ayanamsa"), Some(pleiades_types::Ayanamsa::Krishnamurti));
//! ```

#![forbid(unsafe_code)]

use pleiades_types::{Angle, Ayanamsa, Instant, JulianDay};

/// A catalog entry for a built-in ayanamsa.
#[derive(Clone, Debug, PartialEq)]
pub struct AyanamsaDescriptor {
    /// The strongly typed ayanamsa identifier.
    pub ayanamsa: Ayanamsa,
    /// The canonical name used in compatibility profiles.
    pub canonical_name: &'static str,
    /// Alternate names or software-specific aliases.
    pub aliases: &'static [&'static str],
    /// Short notes about the definition or interoperability constraints.
    pub notes: &'static str,
    /// Reference epoch for the published offset, when available.
    pub epoch: Option<JulianDay>,
    /// Reference sidereal offset in degrees at the reference epoch, when available.
    pub offset_degrees: Option<Angle>,
}

impl AyanamsaDescriptor {
    /// Creates a new descriptor.
    pub const fn new(
        ayanamsa: Ayanamsa,
        canonical_name: &'static str,
        aliases: &'static [&'static str],
        notes: &'static str,
        epoch: Option<JulianDay>,
        offset_degrees: Option<Angle>,
    ) -> Self {
        Self {
            ayanamsa,
            canonical_name,
            aliases,
            notes,
            epoch,
            offset_degrees,
        }
    }

    /// Returns the sidereal offset at the provided instant, when the catalog
    /// entry carries enough metadata to derive one.
    pub fn offset_at(&self, instant: Instant) -> Option<Angle> {
        offset_from_components(self.epoch, self.offset_degrees, instant)
    }

    /// Returns `true` if the provided label matches the canonical name or one
    /// of the documented aliases.
    pub fn matches_label(&self, label: &str) -> bool {
        self.canonical_name.eq_ignore_ascii_case(label)
            || self
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(label))
    }

    /// Returns `true` when both reference metadata fields are present.
    pub fn has_sidereal_metadata(&self) -> bool {
        self.epoch.is_some() && self.offset_degrees.is_some()
    }
}

const BASELINE_AYANAMSAS: &[AyanamsaDescriptor] = &[
    AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Lahiri",
        &[
            "Chitra Paksha",
            "Chitrapaksha",
            "Chitra-paksha",
            "Lahiri ayanamsa",
        ],
        "Default Indian sidereal standard in many astrology workflows.",
        Some(JulianDay::from_days(2_435_553.5)),
        Some(Angle::from_degrees(23.245_524_743)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Raman,
        "Raman",
        &["B. V. Raman", "B.V. Raman", "B V Raman", "Raman ayanamsa"],
        "Popular named sidereal offset used in classical astrology software.",
        Some(JulianDay::from_days(2_415_020.0)),
        Some(Angle::from_degrees(21.014_44)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Krishnamurti,
        "Krishnamurti",
        &[
            "KP",
            "Krishnamurti Ayanamsha",
            "Krishnamurti Ayanamsa",
            "Krishnamurti ayanamsa",
            "Krishnamurti (Swiss)",
            "Krishnamurti Paddhati",
            "KP ayanamsa",
        ],
        "Krishnamurti Paddhati ayanamsa.",
        Some(JulianDay::from_days(2_415_020.0)),
        Some(Angle::from_degrees(22.363_889)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::FaganBradley,
        "Fagan/Bradley",
        &["Fagan/Bradley Ayanamsha", "Fagan Bradley", "Fagan-Bradley"],
        "Western sidereal reference used by several astrology packages.",
        Some(JulianDay::from_days(2_433_282.423_46)),
        Some(Angle::from_degrees(24.042_044_444)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::TrueChitra,
        "True Chitra",
        &["Chitra", "True Chitra ayanamsa"],
        "True Chitra / Chitra-based sidereal variant.",
        Some(JulianDay::from_days(2_435_553.5)),
        Some(Angle::from_degrees(23.245_524_743)),
    ),
];

const RELEASE_AYANAMSAS: &[AyanamsaDescriptor] = &[
    AyanamsaDescriptor::new(
        Ayanamsa::TrueCitra,
        "True Citra",
        &["True Citra ayanamsa"],
        "True Citra sidereal mode with the published zero point used by Swiss Ephemeris-style interoperability tables.",
        Some(JulianDay::from_days(1_825_182.872_330)),
        Some(Angle::from_degrees(50.256_748_3)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::J2000,
        "J2000",
        &["J2000.0"],
        "Swiss Ephemeris J2000 sidereal frame anchored to the standard J2000.0 epoch.",
        Some(JulianDay::from_days(2_451_545.0)),
        Some(Angle::from_degrees(23.853_177_78)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::J1900,
        "J1900",
        &["J1900.0"],
        "Swiss Ephemeris J1900 sidereal frame anchored to the standard J1900.0 epoch.",
        Some(JulianDay::from_days(2_415_020.0)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::B1950,
        "B1950",
        &["B1950.0"],
        "Swiss Ephemeris B1950 sidereal frame anchored to the FK4 B1950.0 epoch.",
        Some(JulianDay::from_days(2_433_281.5)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::TrueRevati,
        "True Revati",
        &["True Revati ayanamsa"],
        "True-nakshatra mode with the Revati reference point fixed to the Swiss Ephemeris zero date.",
        Some(JulianDay::from_days(1_926_902.658_267)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::TrueMula,
        "True Mula",
        &["True Mula (Chandra Hari)", "True Mula ayanamsa", "Chandra Hari"],
        "True-nakshatra mode with the Mula reference point fixed to the Swiss Ephemeris zero date.",
        Some(JulianDay::from_days(1_805_889.671_313)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::SuryasiddhantaRevati,
        "Suryasiddhanta (Revati)",
        &["SS Revati", "Suryasiddhanta Revati", "Surya Siddhanta Revati"],
        "Swiss Ephemeris SS Revati mode, anchored to the published Revati zero point used by the Surya Siddhanta family.",
        Some(JulianDay::from_days(1_924_230.267_296)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::SuryasiddhantaCitra,
        "Suryasiddhanta (Citra)",
        &["SS Citra", "Suryasiddhanta Citra", "Surya Siddhanta Citra"],
        "Swiss Ephemeris SS Citra mode, anchored to the published Citra zero point used by the Surya Siddhanta family.",
        Some(JulianDay::from_days(1_903_396.812_865_4)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::LahiriIcrc,
        "Lahiri (ICRC)",
        &["ICRC Lahiri", "Lahiri ICRC"],
        "The 1956 Indian Calendar Reform Committee standard with a round 23°15′ reference value.",
        Some(JulianDay::from_days(2_435_553.5)),
        Some(Angle::from_degrees(23.25)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Lahiri1940,
        "Lahiri (1940)",
        &["Lahiri original", "Panchanga Darpan Lahiri"],
        "Lahiri's earlier zero-date variant published in Panchanga Darpan.",
        Some(JulianDay::from_days(2_415_020.0)),
        Some(Angle::from_degrees(22.445_972_222_222_223)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::UshaShashi,
        "Usha Shashi",
        &["Ushashashi", "Usha-Shashi", "Usha/Shashi", "Usha Shashi ayanamsa", "Revati"],
        "Revati-bound zero-point variant used in the Greek-Arabic-Hindu tradition.",
        Some(JulianDay::from_days(2_415_020.5)),
        Some(Angle::from_degrees(18.660_961_111_111_11)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Suryasiddhanta499,
        "Suryasiddhanta (499 CE)",
        &["Suryasiddhanta 499", "Surya Siddhanta 499", "Suryasiddhanta 499 CE", "Surya Siddhanta 499 CE", "Surya Siddhanta", "Suryasiddhanta"],
        "Suryasiddhanta zero-point variant anchored to the 499 CE equinox.",
        Some(JulianDay::from_days(1_903_396.812_865_393_5)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Aryabhata499,
        "Aryabhata (499 CE)",
        &[
            "Aryabhata 499",
            "Aryabhata 499 CE",
            "Aryabhata",
            "Aryabhatan Kaliyuga",
            "Aryabhata Kaliyuga",
        ],
        "Aryabhata zero-point variant anchored to the 499 CE dawn tradition.",
        Some(JulianDay::from_days(1_903_396.789_532_060_3)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Sassanian,
        "Sassanian",
        &["Zij al-Shah", "Sasanian"],
        "Sassanian zero-point variant anchored to the 564 CE table-reform epoch.",
        Some(JulianDay::from_days(1_927_135.874_779_3)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::DeLuce,
        "DeLuce",
        &["De Luce", "DeLuce ayanamsa"],
        "Swiss Ephemeris DeLuce sidereal mode, documented by Astrodienst as a standard built-in ayanamsa option.",
        Some(JulianDay::from_days(2_451_545.0)),
        Some(Angle::from_degrees(23.245_522_556)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Yukteshwar,
        "Yukteshwar",
        &["Yukteswar", "Sri Yukteswar", "Sri Yukteshwar", "Shri Yukteswar", "Shri Yukteshwar", "Yukteshwar ayanamsa"],
        "Swiss Ephemeris Yukteshwar sidereal mode, documented as a built-in ayanamsa option with a Sri Yukteswar-compatible naming family.",
        Some(JulianDay::from_days(2_451_545.0)),
        Some(Angle::from_degrees(22.628_888_9)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::PvrPushyaPaksha,
        "PVR Pushya-paksha",
        &["Pushya-paksha", "Pushya Paksha", "P.V.R. Narasimha Rao", "PVR", "True Pushya (PVRN Rao)"],
        "P.V.R. Narasimha Rao's Pushya-paksha ayanamsa, exposed in Swiss Ephemeris as a built-in sidereal mode.",
        Some(JulianDay::from_days(2_451_545.0)),
        Some(Angle::from_degrees(23.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Sheoran,
        "Sheoran",
        &["Sunil Sheoran", "Vedic Sheoran", "Sheoran ayanamsa", "\"Vedic\"/Sheoran"],
        "Sheoran's Vedic ayanamsa, anchored to the published zero point used by Swiss Ephemeris.",
        Some(JulianDay::from_days(1_789_947.090_881)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Hipparchus,
        "Hipparchus",
        &["Hipparchos"],
        "Swiss Ephemeris' Hipparchus sidereal mode, named for the Greek astronomer whose precession model underlies the historical reference frame.",
        Some(JulianDay::from_days(1_674_484.0)),
        Some(Angle::from_degrees(-9.333_333_333_333_334)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianKugler1,
        "Babylonian (Kugler 1)",
        &["Babylonian/Kugler 1", "Babylonian Kugler 1", "Babylonian 1"],
        "Babylonian sidereal mode associated with Kugler's first reconstruction, with the Swiss Ephemeris zero point at JD 1833923.577692 (+0309/01/05 01:51:52.62 UT).",
        Some(JulianDay::from_days(1_833_923.577_692)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianKugler2,
        "Babylonian (Kugler 2)",
        &["Babylonian/Kugler 2", "Babylonian Kugler 2", "Babylonian 2"],
        "Babylonian sidereal mode associated with Kugler's second reconstruction, with the Swiss Ephemeris zero point at JD 1797039.206820 (+0208/01/10 16:57:49.23 UT).",
        Some(JulianDay::from_days(1_797_039.206_820)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianKugler3,
        "Babylonian (Kugler 3)",
        &["Babylonian/Kugler 3", "Babylonian Kugler 3", "Babylonian 3"],
        "Babylonian sidereal mode associated with Kugler's third reconstruction, with the Swiss Ephemeris zero point at JD 1774637.420172 (+0146/09/09 22:05:02.88 UT).",
        Some(JulianDay::from_days(1_774_637.420_172)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianHuber,
        "Babylonian (Huber)",
        &["Babylonian/Huber", "Babylonian Huber"],
        "Babylonian sidereal mode associated with Huber's reconstruction.",
        Some(JulianDay::from_days(1_721_171.5)),
        Some(Angle::from_degrees(-0.120_555_555_555_555_55)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianEtaPiscium,
        "Babylonian (Eta Piscium)",
        &["Babylonian/Eta Piscium", "Babylonian Eta Piscium", "Eta Piscium"],
        "Babylonian sidereal mode aligned to the Eta Piscium fiducial star.",
        Some(JulianDay::from_days(1_807_871.964_797)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianAldebaran,
        "Babylonian (Aldebaran)",
        &["Babylonian/Aldebaran = 15 Tau", "Babylonian Aldebaran", "Babylonian 15 Tau", "15 Tau"],
        "Babylonian sidereal mode aligned to Aldebaran / 15 Taurus.",
        Some(JulianDay::from_days(1_801_643.133_503)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianHouse,
        "Babylonian (House)",
        &["Babylonian House", "BABYL_HOUSE"],
        "Babylonian sidereal mode labeled BABYL_HOUSE in Swiss Ephemeris.",
        None,
        None,
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianSissy,
        "Babylonian (Sissy)",
        &["Babylonian Sissy", "BABYL_SISSY"],
        "Babylonian sidereal mode labeled BABYL_SISSY in Swiss Ephemeris.",
        None,
        None,
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianTrueGeoc,
        "Babylonian (True Geoc)",
        &["Babylonian True Geoc", "BABYL_TRUE_GEOC"],
        "Babylonian sidereal mode labeled BABYL_TRUE_GEOC in Swiss Ephemeris.",
        None,
        None,
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianTrueTopc,
        "Babylonian (True Topc)",
        &["Babylonian True Topc", "BABYL_TRUE_TOPC"],
        "Babylonian sidereal mode labeled BABYL_TRUE_TOPC in Swiss Ephemeris.",
        None,
        None,
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianTrueObs,
        "Babylonian (True Obs)",
        &["Babylonian True Obs", "BABYL_TRUE_OBS"],
        "Babylonian sidereal mode labeled BABYL_TRUE_OBS in Swiss Ephemeris.",
        None,
        None,
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianHouseObs,
        "Babylonian (House Obs)",
        &["Babylonian House Obs", "BABYL_HOUSE_OBS"],
        "Babylonian sidereal mode labeled BABYL_HOUSE_OBS in Swiss Ephemeris.",
        None,
        None,
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticCenter,
        "Galactic Center",
        &["Galact. Center = 0 Sag", "Gal. Center = 0 Sag", "0 Sag", "Galactic center"],
        "Galactic-center sidereal reference mode fixed at 0 Sagittarius.",
        Some(JulianDay::from_days(1_746_340.540_490)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticEquator,
        "Galactic Equator",
        &["Galactic equator", "Gal. Eq."],
        "Galactic-equator sidereal reference mode. The true/modern variant is anchored to the 1665728.603158 JD zero point described in the Swiss Ephemeris documentation.",
        Some(JulianDay::from_days(1_665_728.603_158)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::TruePushya,
        "True Pushya",
        &["Pushya", "True Pushya ayanamsa"],
        "True-nakshatra Pushya reference mode exposed by Swiss Ephemeris and anchored to the published zero date.",
        Some(JulianDay::from_days(1_855_769.248_315)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Udayagiri,
        "Udayagiri",
        &["Udayagiri ayanamsa"],
        "Udayagiri sidereal mode treated as the Lahiri/Chitrapaksha/Chitra Paksha 285 CE reference family in the Swiss Ephemeris interoperability catalog.",
        Some(JulianDay::from_days(1_825_235.164_583)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::DjwhalKhul,
        "Djwhal Khul",
        &["Djwhal", "Djwhal Khul ayanamsa"],
        "Djwhal Khul sidereal mode, anchored to the published zero date used by the Swiss Ephemeris family.",
        Some(JulianDay::from_days(1_706_703.948_006)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::JnBhasin,
        "JN Bhasin",
        &["J. N. Bhasin", "J.N. Bhasin", "Bhasin"],
        "J. N. Bhasin sidereal mode.",
        Some(JulianDay::from_days(2_454_239.282_537)),
        Some(Angle::from_degrees(0.013_968_911_416_666_667)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Suryasiddhanta499MeanSun,
        "Suryasiddhanta (Mean Sun)",
        &[
            "Suryasiddhanta, mean Sun",
            "Surya Siddhanta, mean Sun",
            "Suryasiddhanta mean sun",
            "Surya Siddhanta mean sun",
            "Suryasiddhanta MSUN",
            "Surya Siddhanta MSUN",
        ],
        "Suryasiddhanta mean-sun variant anchored to the published 514 CE zero point used by Swiss Ephemeris.",
        Some(JulianDay::from_days(1_909_045.584_433)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Aryabhata499MeanSun,
        "Aryabhata (Mean Sun)",
        &["Aryabhata, mean Sun", "Aryabhata mean sun", "Aryabhata MSUN"],
        "Aryabhata mean-sun variant anchored to the published 516 CE zero point used by Swiss Ephemeris.",
        Some(JulianDay::from_days(1_909_650.815_331)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianBritton,
        "Babylonian (Britton)",
        &["Babylonian/Britton", "Babylonian Britton"],
        "Babylonian sidereal mode associated with Britton's reconstruction, with the Swiss Ephemeris zero point at JD 1805415.712776 (+0230/12/17 05:06:23.86 UT).",
        Some(JulianDay::from_days(1_805_415.712_776)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Aryabhata522,
        "Aryabhata (522 CE)",
        &["Aryabhata 522", "Aryabhata 522 CE"],
        "Aryabhata zero-point variant anchored to the published 522 CE reference date.",
        Some(JulianDay::from_days(1_911_797.740_782)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::LahiriVP285,
        "Lahiri (VP285)",
        &["Lahiri VP285", "VP285"],
        "Lahiri VP285 reference family anchored to the 285 CE mean-sun zero point used by Swiss Ephemeris.",
        Some(JulianDay::from_days(1_825_235.164_583)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::KrishnamurtiVP291,
        "Krishnamurti (VP291)",
        &["KP VP291", "Krishnamurti VP291", "Krishnamurti-Senthilathiban", "VP291"],
        "Krishnamurti variant aligned with the VP291 reference family and anchored to the published 291 CE zero point.",
        Some(JulianDay::from_days(1_827_424.663_554)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::TrueSheoran,
        "True Sheoran",
        &["Sheoran true", "True Sheoran ayanamsa"],
        "True-nakshatra Sheoran reference mode with the Swiss Ephemeris zero point at JD 1789947.090881 (+0188/08/09 14:10:52.11 UT).",
        Some(JulianDay::from_days(1_789_947.090_881)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticCenterRgilbrand,
        "Galactic Center (Rgilbrand)",
        &["Galactic Center (Gil Brand)", "Gil Brand", "Rgilbrand", "Galactic center Rgilbrand"],
        "Galactic-center reference mode attributed to Rgilbrand, with the Swiss Ephemeris zero point at JD 1861740.329525 (+0385/03/03 19:54:30.99 UT).",
        Some(JulianDay::from_days(1_861_740.329_525)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticCenterMardyks,
        "Galactic Center (Mardyks)",
        &["Skydram", "Skydram/Galactic Alignment", "Skydram (Mardyks)", "Mardyks", "Galactic center Mardyks"],
        "Galactic-center reference mode attributed to Mardyks, with the Swiss Ephemeris zero point at JD 1662951.794251 (-0160/11/27 07:03:43.27 UT).",
        Some(JulianDay::from_days(1_662_951.794_251)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticCenterMulaWilhelm,
        "Galactic Center (Mula/Wilhelm)",
        &["Dhruva/Gal.Center/Mula (Wilhelm)", "Mula Wilhelm", "Wilhelm", "Galactic center Mula/Wilhelm"],
        "Galactic-center reference mode aligned to the Mula/Wilhelm tradition, with the Swiss Ephemeris zero point at JD 1946834.818321 (+0618/02/25 07:38:22.96 UT).",
        Some(JulianDay::from_days(1_946_834.818_321)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::DhruvaGalacticCenterMula,
        "Dhruva Galactic Center (Middle Mula)",
        &[
            "Dhruva/Gal.Center/Mula",
            "Dhruva Galactic Center Middle Mula",
            "Dhruva Middle Mula",
            "Middle of Mula",
        ],
        "Dhruva projection of the Galactic Center to the middle of Mula for interoperability with Wilhelm-style sidereal selections.",
        Some(JulianDay::from_days(1_946_834.818_321)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticCenterCochrane,
        "Galactic Center (Cochrane)",
        &["Cochrane (Gal.Center = 0 Cap)", "Gal. Center = 0 Cap", "Cochrane", "Galactic center Cochrane"],
        "Galactic-center reference mode attributed to Cochrane and catalogued with the Swiss Ephemeris zero-point epoch.",
        Some(JulianDay::from_days(1_662_951.794_251)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticEquatorIau1958,
        "Galactic Equator (IAU 1958)",
        &["Galactic Equator (IAU1958)", "IAU 1958", "Galactic equator IAU 1958"],
        "Galactic-equator reference mode using the IAU 1958 definition.",
        Some(JulianDay::from_days(1_667_118.376_332)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticEquatorTrue,
        "Galactic Equator (True)",
        &["True galactic equator", "Galactic equator true"],
        "Galactic-equator reference mode using the true-galactic definition. The true/modern variant is anchored to the 1665728.603158 JD zero point described in the Swiss Ephemeris documentation.",
        Some(JulianDay::from_days(1_665_728.603_158)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticEquatorMula,
        "Galactic Equator (Mula)",
        &["Galactic Equator mid-Mula", "Mula galactic equator", "Galactic equator Mula"],
        "Galactic-equator reference mode aligned to the Mula tradition and anchored to the Swiss Ephemeris mid-Mula zero point.",
        Some(JulianDay::from_days(1_840_527.426_262)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticEquatorFiorenza,
        "Galactic Equator (Fiorenza)",
        &["Fiorenza", "Galactic equator Fiorenza"],
        "Galactic-equator reference mode attributed to Fiorenza and catalogued with the Swiss Ephemeris J2000.0 reference epoch and 25° zero-point offset.",
        Some(JulianDay::from_days(2_451_544.5)),
        Some(Angle::from_degrees(25.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::ValensMoon,
        "Valens Moon",
        &["Vettius Valens", "Valens", "Moon", "Moon sign ayanamsa"],
        "Valens Moon sidereal mode, catalogued with the Swiss Ephemeris reference epoch and offset from the header metadata.",
        Some(JulianDay::from_days(1_775_845.5)),
        Some(Angle::from_degrees(-2.942_2)),
    ),
];

static BUILT_IN_AYANAMSAS: [AyanamsaDescriptor; 59] = [
    AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Lahiri",
        &["Chitra Paksha", "Chitrapaksha", "Chitra-paksha", "Lahiri ayanamsa"],
        "Default Indian sidereal standard in many astrology workflows.",
        Some(JulianDay::from_days(2_435_553.5)),
        Some(Angle::from_degrees(23.245_524_743)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Raman,
        "Raman",
        &["B. V. Raman", "B.V. Raman", "B V Raman", "Raman ayanamsa"],
        "Popular named sidereal offset used in classical astrology software.",
        Some(JulianDay::from_days(2_415_020.0)),
        Some(Angle::from_degrees(21.014_44)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Krishnamurti,
        "Krishnamurti",
        &[
            "KP",
            "Krishnamurti Ayanamsha",
            "Krishnamurti ayanamsa",
            "Krishnamurti (Swiss)",
            "Krishnamurti Paddhati",
            "KP ayanamsa",
        ],
        "Krishnamurti Paddhati ayanamsa.",
        Some(JulianDay::from_days(2_415_020.0)),
        Some(Angle::from_degrees(22.363_889)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::FaganBradley,
        "Fagan/Bradley",
        &["Fagan/Bradley Ayanamsha", "Fagan Bradley", "Fagan-Bradley"],
        "Western sidereal reference used by several astrology packages.",
        Some(JulianDay::from_days(2_433_282.423_46)),
        Some(Angle::from_degrees(24.042_044_444)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::TrueChitra,
        "True Chitra",
        &["Chitra", "True Chitra ayanamsa"],
        "True Chitra / Chitra-based sidereal variant.",
        Some(JulianDay::from_days(2_435_553.5)),
        Some(Angle::from_degrees(23.245_524_743)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::TrueCitra,
        "True Citra",
        &[
            "True Citra ayanamsa",
            "True Citra Paksha",
            "True Chitra Paksha",
            "True Chitrapaksha",
        ],
        "True Citra sidereal mode with the published zero point used by Swiss Ephemeris-style interoperability tables.",
        Some(JulianDay::from_days(1_825_182.872_330)),
        Some(Angle::from_degrees(50.256_748_3)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::J2000,
        "J2000",
        &["J2000.0"],
        "Swiss Ephemeris J2000 sidereal frame anchored to the standard J2000.0 epoch.",
        Some(JulianDay::from_days(2_451_545.0)),
        Some(Angle::from_degrees(23.853_177_78)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::J1900,
        "J1900",
        &["J1900.0"],
        "Swiss Ephemeris J1900 sidereal frame anchored to the standard J1900.0 epoch.",
        Some(JulianDay::from_days(2_415_020.0)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::B1950,
        "B1950",
        &["B1950.0"],
        "Swiss Ephemeris B1950 sidereal frame anchored to the FK4 B1950.0 epoch.",
        Some(JulianDay::from_days(2_433_281.5)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::TrueRevati,
        "True Revati",
        &["True Revati ayanamsa"],
        "True-nakshatra mode with the Revati reference point fixed to the Swiss Ephemeris zero date.",
        Some(JulianDay::from_days(1_926_902.658_267)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::TrueMula,
        "True Mula",
        &["True Mula (Chandra Hari)", "True Mula ayanamsa", "Chandra Hari"],
        "True-nakshatra mode with the Mula reference point fixed to the Swiss Ephemeris zero date.",
        Some(JulianDay::from_days(1_805_889.671_313)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::SuryasiddhantaRevati,
        "Suryasiddhanta (Revati)",
        &["SS Revati", "Suryasiddhanta Revati", "Surya Siddhanta Revati"],
        "Swiss Ephemeris SS Revati mode, anchored to the published Revati zero point used by the Surya Siddhanta family.",
        Some(JulianDay::from_days(1_924_230.267_296)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::SuryasiddhantaCitra,
        "Suryasiddhanta (Citra)",
        &["SS Citra", "Suryasiddhanta Citra", "Surya Siddhanta Citra"],
        "Swiss Ephemeris SS Citra mode, anchored to the published Citra zero point used by the Surya Siddhanta family.",
        Some(JulianDay::from_days(1_903_396.812_865_4)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::LahiriIcrc,
        "Lahiri (ICRC)",
        &["ICRC Lahiri", "Lahiri ICRC"],
        "The 1956 Indian Calendar Reform Committee standard with a round 23°15′ reference value.",
        Some(JulianDay::from_days(2_435_553.5)),
        Some(Angle::from_degrees(23.25)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Lahiri1940,
        "Lahiri (1940)",
        &["Lahiri original", "Panchanga Darpan Lahiri"],
        "Lahiri's earlier zero-date variant published in Panchanga Darpan.",
        Some(JulianDay::from_days(2_415_020.0)),
        Some(Angle::from_degrees(22.445_972_222_222_223)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::UshaShashi,
        "Usha Shashi",
        &["Ushashashi", "Usha-Shashi", "Usha/Shashi", "Usha Shashi ayanamsa", "Revati"],
        "Revati-bound zero-point variant used in the Greek-Arabic-Hindu tradition.",
        Some(JulianDay::from_days(2_415_020.5)),
        Some(Angle::from_degrees(18.660_961_111_111_11)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Suryasiddhanta499,
        "Suryasiddhanta (499 CE)",
        &["Suryasiddhanta 499", "Surya Siddhanta 499", "Suryasiddhanta 499 CE", "Surya Siddhanta 499 CE", "Surya Siddhanta", "Suryasiddhanta"],
        "Suryasiddhanta zero-point variant anchored to the 499 CE equinox.",
        Some(JulianDay::from_days(1_903_396.812_865_393_5)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Aryabhata499,
        "Aryabhata (499 CE)",
        &[
            "Aryabhata 499",
            "Aryabhata 499 CE",
            "Aryabhata",
            "Aryabhatan Kaliyuga",
            "Aryabhata Kaliyuga",
        ],
        "Aryabhata zero-point variant anchored to the 499 CE dawn tradition.",
        Some(JulianDay::from_days(1_903_396.789_532_060_3)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Sassanian,
        "Sassanian",
        &["Zij al-Shah", "Sasanian"],
        "Sassanian zero-point variant anchored to the 564 CE table-reform epoch.",
        Some(JulianDay::from_days(1_927_135.874_779_3)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::DeLuce,
        "DeLuce",
        &["De Luce", "DeLuce ayanamsa"],
        "Swiss Ephemeris DeLuce sidereal mode, documented by Astrodienst as a standard built-in ayanamsa option.",
        Some(JulianDay::from_days(2_451_545.0)),
        Some(Angle::from_degrees(23.245_522_556)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Yukteshwar,
        "Yukteshwar",
        &["Yukteswar", "Sri Yukteswar", "Sri Yukteshwar", "Shri Yukteswar", "Shri Yukteshwar", "Yukteshwar ayanamsa"],
        "Swiss Ephemeris Yukteshwar sidereal mode, documented as a built-in ayanamsa option with a Sri Yukteswar-compatible naming family.",
        Some(JulianDay::from_days(2_451_545.0)),
        Some(Angle::from_degrees(22.628_888_9)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::PvrPushyaPaksha,
        "PVR Pushya-paksha",
        &["Pushya-paksha", "Pushya Paksha", "P.V.R. Narasimha Rao", "PVR", "True Pushya (PVRN Rao)"],
        "P.V.R. Narasimha Rao's Pushya-paksha ayanamsa, exposed in Swiss Ephemeris as a built-in sidereal mode.",
        Some(JulianDay::from_days(2_451_545.0)),
        Some(Angle::from_degrees(23.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Sheoran,
        "Sheoran",
        &["Sunil Sheoran", "Vedic Sheoran", "Sheoran ayanamsa", "\"Vedic\"/Sheoran"],
        "Sheoran's Vedic ayanamsa, anchored to the published zero point used by Swiss Ephemeris.",
        Some(JulianDay::from_days(1_789_947.090_881)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Hipparchus,
        "Hipparchus",
        &["Hipparchos"],
        "Swiss Ephemeris' Hipparchus sidereal mode, named for the Greek astronomer whose precession model underlies the historical reference frame.",
        Some(JulianDay::from_days(1_674_484.0)),
        Some(Angle::from_degrees(-9.333_333_333_333_334)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianKugler1,
        "Babylonian (Kugler 1)",
        &["Babylonian/Kugler 1", "Babylonian Kugler 1", "Babylonian 1"],
        "Babylonian sidereal mode associated with Kugler's first reconstruction, with the Swiss Ephemeris zero point at JD 1833923.577692 (+0309/01/05 01:51:52.62 UT).",
        Some(JulianDay::from_days(1_833_923.577_692)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianKugler2,
        "Babylonian (Kugler 2)",
        &["Babylonian/Kugler 2", "Babylonian Kugler 2", "Babylonian 2"],
        "Babylonian sidereal mode associated with Kugler's second reconstruction, with the Swiss Ephemeris zero point at JD 1797039.206820 (+0208/01/10 16:57:49.23 UT).",
        Some(JulianDay::from_days(1_797_039.206_820)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianKugler3,
        "Babylonian (Kugler 3)",
        &["Babylonian/Kugler 3", "Babylonian Kugler 3", "Babylonian 3"],
        "Babylonian sidereal mode associated with Kugler's third reconstruction, with the Swiss Ephemeris zero point at JD 1774637.420172 (+0146/09/09 22:05:02.88 UT).",
        Some(JulianDay::from_days(1_774_637.420_172)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianHuber,
        "Babylonian (Huber)",
        &["Babylonian/Huber", "Babylonian Huber"],
        "Babylonian sidereal mode associated with Huber's reconstruction.",
        Some(JulianDay::from_days(1_721_171.5)),
        Some(Angle::from_degrees(-0.120_555_555_555_555_55)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianEtaPiscium,
        "Babylonian (Eta Piscium)",
        &["Babylonian/Eta Piscium", "Babylonian Eta Piscium", "Eta Piscium"],
        "Babylonian sidereal mode aligned to the Eta Piscium fiducial star.",
        Some(JulianDay::from_days(1_807_871.964_797)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianAldebaran,
        "Babylonian (Aldebaran)",
        &["Babylonian/Aldebaran = 15 Tau", "Babylonian Aldebaran", "Babylonian 15 Tau", "15 Tau"],
        "Babylonian sidereal mode aligned to Aldebaran / 15 Taurus.",
        Some(JulianDay::from_days(1_801_643.133_503)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianHouse,
        "Babylonian (House)",
        &["Babylonian House", "BABYL_HOUSE"],
        "Babylonian sidereal mode labeled BABYL_HOUSE in Swiss Ephemeris.",
        None,
        None,
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianSissy,
        "Babylonian (Sissy)",
        &["Babylonian Sissy", "BABYL_SISSY"],
        "Babylonian sidereal mode labeled BABYL_SISSY in Swiss Ephemeris.",
        None,
        None,
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianTrueGeoc,
        "Babylonian (True Geoc)",
        &["Babylonian True Geoc", "BABYL_TRUE_GEOC"],
        "Babylonian sidereal mode labeled BABYL_TRUE_GEOC in Swiss Ephemeris.",
        None,
        None,
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianTrueTopc,
        "Babylonian (True Topc)",
        &["Babylonian True Topc", "BABYL_TRUE_TOPC"],
        "Babylonian sidereal mode labeled BABYL_TRUE_TOPC in Swiss Ephemeris.",
        None,
        None,
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianTrueObs,
        "Babylonian (True Obs)",
        &["Babylonian True Obs", "BABYL_TRUE_OBS"],
        "Babylonian sidereal mode labeled BABYL_TRUE_OBS in Swiss Ephemeris.",
        None,
        None,
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianHouseObs,
        "Babylonian (House Obs)",
        &["Babylonian House Obs", "BABYL_HOUSE_OBS"],
        "Babylonian sidereal mode labeled BABYL_HOUSE_OBS in Swiss Ephemeris.",
        None,
        None,
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticCenter,
        "Galactic Center",
        &["Galact. Center = 0 Sag", "Gal. Center = 0 Sag", "0 Sag", "Galactic center"],
        "Galactic-center sidereal reference mode fixed at 0 Sagittarius.",
        Some(JulianDay::from_days(1_746_340.540_490)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticEquator,
        "Galactic Equator",
        &["Galactic equator", "Gal. Eq."],
        "Galactic-equator sidereal reference mode. The true/modern variant is anchored to the 1665728.603158 JD zero point described in the Swiss Ephemeris documentation.",
        Some(JulianDay::from_days(1_665_728.603_158)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::TruePushya,
        "True Pushya",
        &["Pushya", "True Pushya ayanamsa"],
        "True-nakshatra Pushya reference mode exposed by Swiss Ephemeris and anchored to the published zero date.",
        Some(JulianDay::from_days(1_855_769.248_315)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Udayagiri,
        "Udayagiri",
        &["Udayagiri ayanamsa"],
        "Udayagiri sidereal mode treated as the Lahiri/Chitrapaksha/Chitra Paksha 285 CE reference family in the Swiss Ephemeris interoperability catalog.",
        Some(JulianDay::from_days(1_825_235.164_583)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::DjwhalKhul,
        "Djwhal Khul",
        &["Djwhal", "Djwhal Khul ayanamsa"],
        "Djwhal Khul sidereal mode, anchored to the published zero date used by the Swiss Ephemeris family.",
        Some(JulianDay::from_days(1_706_703.948_006)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::JnBhasin,
        "JN Bhasin",
        &["J. N. Bhasin", "J.N. Bhasin", "Bhasin"],
        "J. N. Bhasin sidereal mode.",
        Some(JulianDay::from_days(2_454_239.282_537)),
        Some(Angle::from_degrees(0.013_968_911_416_666_667)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Suryasiddhanta499MeanSun,
        "Suryasiddhanta (Mean Sun)",
        &[
            "Suryasiddhanta, mean Sun",
            "Surya Siddhanta, mean Sun",
            "Suryasiddhanta mean sun",
            "Surya Siddhanta mean sun",
            "Suryasiddhanta MSUN",
            "Surya Siddhanta MSUN",
        ],
        "Suryasiddhanta mean-sun variant anchored to the published 514 CE zero point used by Swiss Ephemeris.",
        Some(JulianDay::from_days(1_909_045.584_433)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Aryabhata499MeanSun,
        "Aryabhata (Mean Sun)",
        &["Aryabhata, mean Sun", "Aryabhata mean sun", "Aryabhata MSUN"],
        "Aryabhata mean-sun variant anchored to the published 516 CE zero point used by Swiss Ephemeris.",
        Some(JulianDay::from_days(1_909_650.815_331)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::BabylonianBritton,
        "Babylonian (Britton)",
        &["Babylonian/Britton", "Babylonian Britton"],
        "Babylonian sidereal mode associated with Britton's reconstruction, with the Swiss Ephemeris zero point at JD 1805415.712776 (+0230/12/17 05:06:23.86 UT).",
        Some(JulianDay::from_days(1_805_415.712_776)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Aryabhata522,
        "Aryabhata (522 CE)",
        &["Aryabhata 522", "Aryabhata 522 CE"],
        "Aryabhata zero-point variant anchored to the published 522 CE reference date.",
        Some(JulianDay::from_days(1_911_797.740_782)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::LahiriVP285,
        "Lahiri (VP285)",
        &["Lahiri VP285", "VP285"],
        "Lahiri VP285 reference family anchored to the 285 CE mean-sun zero point used by Swiss Ephemeris.",
        Some(JulianDay::from_days(1_825_235.164_583)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::KrishnamurtiVP291,
        "Krishnamurti (VP291)",
        &["KP VP291", "Krishnamurti VP291", "Krishnamurti-Senthilathiban", "VP291"],
        "Krishnamurti variant aligned with the VP291 reference family and anchored to the published 291 CE zero point.",
        Some(JulianDay::from_days(1_827_424.663_554)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::TrueSheoran,
        "True Sheoran",
        &["Sheoran true", "True Sheoran ayanamsa"],
        "True-nakshatra Sheoran reference mode with the Swiss Ephemeris zero point at JD 1789947.090881 (+0188/08/09 14:10:52.11 UT).",
        Some(JulianDay::from_days(1_789_947.090_881)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticCenterRgilbrand,
        "Galactic Center (Rgilbrand)",
        &["Galactic Center (Gil Brand)", "Gil Brand", "Rgilbrand", "Galactic center Rgilbrand"],
        "Galactic-center reference mode attributed to Rgilbrand, with the Swiss Ephemeris zero point at JD 1861740.329525 (+0385/03/03 19:54:30.99 UT).",
        Some(JulianDay::from_days(1_861_740.329_525)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticCenterMardyks,
        "Galactic Center (Mardyks)",
        &["Skydram", "Skydram/Galactic Alignment", "Skydram (Mardyks)", "Mardyks", "Galactic center Mardyks"],
        "Galactic-center reference mode attributed to Mardyks, with the Swiss Ephemeris zero point at JD 1662951.794251 (-0160/11/27 07:03:43.27 UT).",
        Some(JulianDay::from_days(1_662_951.794_251)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticCenterMulaWilhelm,
        "Galactic Center (Mula/Wilhelm)",
        &["Dhruva/Gal.Center/Mula (Wilhelm)", "Mula Wilhelm", "Wilhelm", "Galactic center Mula/Wilhelm"],
        "Galactic-center reference mode aligned to the Mula/Wilhelm tradition, with the Swiss Ephemeris zero point at JD 1946834.818321 (+0618/02/25 07:38:22.96 UT).",
        Some(JulianDay::from_days(1_946_834.818_321)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::DhruvaGalacticCenterMula,
        "Dhruva Galactic Center (Middle Mula)",
        &[
            "Dhruva/Gal.Center/Mula",
            "Dhruva Galactic Center Middle Mula",
            "Dhruva Middle Mula",
            "Middle of Mula",
        ],
        "Dhruva projection of the Galactic Center to the middle of Mula for interoperability with Wilhelm-style sidereal selections.",
        Some(JulianDay::from_days(1_946_834.818_321)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticCenterCochrane,
        "Galactic Center (Cochrane)",
        &["Cochrane (Gal.Center = 0 Cap)", "Gal. Center = 0 Cap", "Cochrane", "Galactic center Cochrane"],
        "Galactic-center reference mode attributed to Cochrane and catalogued with the Swiss Ephemeris zero-point epoch.",
        Some(JulianDay::from_days(1_662_951.794_251)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticEquatorIau1958,
        "Galactic Equator (IAU 1958)",
        &["Galactic Equator (IAU1958)", "IAU 1958", "Galactic equator IAU 1958"],
        "Galactic-equator reference mode using the IAU 1958 definition.",
        Some(JulianDay::from_days(1_667_118.376_332)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticEquatorTrue,
        "Galactic Equator (True)",
        &["True galactic equator", "Galactic equator true"],
        "Galactic-equator reference mode using the true-galactic definition. The true/modern variant is anchored to the 1665728.603158 JD zero point described in the Swiss Ephemeris documentation.",
        Some(JulianDay::from_days(1_665_728.603_158)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticEquatorMula,
        "Galactic Equator (Mula)",
        &["Galactic Equator mid-Mula", "Mula galactic equator", "Galactic equator Mula"],
        "Galactic-equator reference mode aligned to the Mula tradition and anchored to the Swiss Ephemeris mid-Mula zero point.",
        Some(JulianDay::from_days(1_840_527.426_262)),
        Some(Angle::from_degrees(0.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::GalacticEquatorFiorenza,
        "Galactic Equator (Fiorenza)",
        &["Fiorenza", "Galactic equator Fiorenza"],
        "Galactic-equator reference mode attributed to Fiorenza and catalogued with the Swiss Ephemeris J2000.0 reference epoch and 25° zero-point offset.",
        Some(JulianDay::from_days(2_451_544.5)),
        Some(Angle::from_degrees(25.0)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::ValensMoon,
        "Valens Moon",
        &["Vettius Valens", "Valens", "Moon", "Moon sign ayanamsa"],
        "Valens Moon sidereal mode, catalogued with the Swiss Ephemeris reference epoch and offset from the header metadata.",
        Some(JulianDay::from_days(1_775_845.5)),
        Some(Angle::from_degrees(-2.942_2)),
    ),
];

/// Returns the baseline built-in ayanamsa catalog.
pub const fn baseline_ayanamsas() -> &'static [AyanamsaDescriptor] {
    BASELINE_AYANAMSAS
}

/// Returns the release-specific ayanamsa additions beyond the baseline milestone.
pub const fn release_ayanamsas() -> &'static [AyanamsaDescriptor] {
    RELEASE_AYANAMSAS
}

/// Returns the full built-in ayanamsa catalog shipped by this release line.
pub const fn built_in_ayanamsas() -> &'static [AyanamsaDescriptor] {
    &BUILT_IN_AYANAMSAS
}

const CUSTOM_DEFINITION_ONLY_AYANAMSAS: &[&str] = &[
    "Babylonian (House)",
    "Babylonian (Sissy)",
    "Babylonian (True Geoc)",
    "Babylonian (True Topc)",
    "Babylonian (True Obs)",
    "Babylonian (House Obs)",
];

fn is_custom_definition_only_ayanamsa(canonical_name: &str) -> bool {
    CUSTOM_DEFINITION_ONLY_AYANAMSAS
        .iter()
        .any(|name| name.eq_ignore_ascii_case(canonical_name))
}

/// A summary of which built-in ayanamsas have sidereal reference metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AyanamsaMetadataCoverage {
    /// Total number of built-in ayanamsas.
    pub total: usize,
    /// Built-in entries that provide both a reference epoch and a reference offset.
    pub with_sidereal_metadata: usize,
    /// Built-in entries that intentionally model custom-definition labels and
    /// therefore omit sidereal metadata.
    pub custom_definition_only: Vec<&'static str>,
    /// Canonical names for built-in entries that are still missing one or both fields.
    pub without_sidereal_metadata: Vec<&'static str>,
}

impl AyanamsaMetadataCoverage {
    /// Returns `true` when every built-in ayanamsa that is meant to carry
    /// sidereal metadata does so.
    pub fn is_complete(&self) -> bool {
        self.without_sidereal_metadata.is_empty()
    }
}

/// Returns a coverage summary for the built-in ayanamsa catalog.
pub fn metadata_coverage() -> AyanamsaMetadataCoverage {
    let mut custom_definition_only = Vec::new();
    let mut without_sidereal_metadata = Vec::new();
    let mut with_sidereal_metadata = 0;

    for entry in built_in_ayanamsas() {
        if entry.has_sidereal_metadata() {
            with_sidereal_metadata += 1;
        } else if is_custom_definition_only_ayanamsa(entry.canonical_name) {
            custom_definition_only.push(entry.canonical_name);
        } else {
            without_sidereal_metadata.push(entry.canonical_name);
        }
    }

    AyanamsaMetadataCoverage {
        total: built_in_ayanamsas().len(),
        with_sidereal_metadata,
        custom_definition_only,
        without_sidereal_metadata,
    }
}

/// Finds the descriptor for a typed ayanamsa selection.
pub fn descriptor(ayanamsa: &Ayanamsa) -> Option<&'static AyanamsaDescriptor> {
    built_in_ayanamsas()
        .iter()
        .find(|entry| entry.ayanamsa == *ayanamsa)
}

/// Resolves an ayanamsa label to a built-in type.
pub fn resolve_ayanamsa(label: &str) -> Option<Ayanamsa> {
    built_in_ayanamsas()
        .iter()
        .find(|entry| entry.matches_label(label))
        .map(|entry| entry.ayanamsa.clone())
}

/// Returns the sidereal offset for the provided ayanamsa and instant.
///
/// Built-in catalog entries use the published reference epoch and offset
/// metadata where available. Custom ayanamsas can supply the same information
/// directly on the `CustomAyanamsa` definition.
pub fn sidereal_offset(ayanamsa: &Ayanamsa, instant: Instant) -> Option<Angle> {
    match ayanamsa {
        Ayanamsa::Custom(custom) => {
            offset_from_components(custom.epoch, custom.offset_degrees, instant)
        }
        other => descriptor(other).and_then(|entry| entry.offset_at(instant)),
    }
}

fn offset_from_components(
    epoch: Option<JulianDay>,
    offset_degrees: Option<Angle>,
    instant: Instant,
) -> Option<Angle> {
    let offset = offset_degrees?;
    let epoch = epoch?;
    let centuries = (instant.julian_day.days() - epoch.days()) / 36_525.0;
    Some(Angle::from_degrees(
        offset.degrees() + centuries * SIDEREAL_PRECESSION_DEGREES_PER_CENTURY,
    ))
}

const SIDEREAL_PRECESSION_DEGREES_PER_CENTURY: f64 = 1.396_971_277_777_777_8;

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::CustomAyanamsa;

    #[test]
    fn baseline_catalog_includes_required_milestone_entries() {
        let names: Vec<_> = baseline_ayanamsas()
            .iter()
            .map(|entry| entry.canonical_name)
            .collect();

        for expected in [
            "Lahiri",
            "Raman",
            "Krishnamurti",
            "Fagan/Bradley",
            "True Chitra",
        ] {
            assert!(names.contains(&expected), "missing {expected}");
        }
    }

    #[test]
    fn aliases_resolve_to_builtin_ayanamsas() {
        assert_eq!(resolve_ayanamsa("KP"), Some(Ayanamsa::Krishnamurti));
        assert_eq!(
            resolve_ayanamsa("Krishnamurti (Swiss)"),
            Some(Ayanamsa::Krishnamurti)
        );
        assert_eq!(
            resolve_ayanamsa("Krishnamurti Paddhati"),
            Some(Ayanamsa::Krishnamurti)
        );
        assert_eq!(
            resolve_ayanamsa("Krishnamurti Ayanamsa"),
            Some(Ayanamsa::Krishnamurti)
        );
        assert_eq!(
            resolve_ayanamsa("Krishnamurti ayanamsa"),
            Some(Ayanamsa::Krishnamurti)
        );
        assert_eq!(
            resolve_ayanamsa("KP ayanamsa"),
            Some(Ayanamsa::Krishnamurti)
        );
        assert_eq!(
            resolve_ayanamsa("fagan-bradley"),
            Some(Ayanamsa::FaganBradley)
        );
        assert_eq!(resolve_ayanamsa("Chitra Paksha"), Some(Ayanamsa::Lahiri));
        assert_eq!(resolve_ayanamsa("Chitra-paksha"), Some(Ayanamsa::Lahiri));
        assert_eq!(resolve_ayanamsa("chitrapaksha"), Some(Ayanamsa::Lahiri));
        assert_eq!(resolve_ayanamsa("Lahiri ayanamsa"), Some(Ayanamsa::Lahiri));
        assert_eq!(resolve_ayanamsa("B.V. Raman"), Some(Ayanamsa::Raman));
        assert_eq!(resolve_ayanamsa("B V Raman"), Some(Ayanamsa::Raman));
        assert_eq!(resolve_ayanamsa("Raman ayanamsa"), Some(Ayanamsa::Raman));
        assert_eq!(resolve_ayanamsa("J2000.0"), Some(Ayanamsa::J2000));
        assert_eq!(resolve_ayanamsa("J1900.0"), Some(Ayanamsa::J1900));
        assert_eq!(resolve_ayanamsa("B1950.0"), Some(Ayanamsa::B1950));
        assert_eq!(resolve_ayanamsa("True Revati"), Some(Ayanamsa::TrueRevati));
        assert_eq!(resolve_ayanamsa("True Mula"), Some(Ayanamsa::TrueMula));
        assert_eq!(resolve_ayanamsa("True Citra"), Some(Ayanamsa::TrueCitra));
        assert_eq!(
            resolve_ayanamsa("True Chitra ayanamsa"),
            Some(Ayanamsa::TrueChitra)
        );
        assert_eq!(
            resolve_ayanamsa("True Citra ayanamsa"),
            Some(Ayanamsa::TrueCitra)
        );
        assert_eq!(
            resolve_ayanamsa("True Citra Paksha"),
            Some(Ayanamsa::TrueCitra)
        );
        assert_eq!(
            resolve_ayanamsa("True Chitra Paksha"),
            Some(Ayanamsa::TrueCitra)
        );
        assert_eq!(
            resolve_ayanamsa("True Chitrapaksha"),
            Some(Ayanamsa::TrueCitra)
        );
        assert_eq!(
            resolve_ayanamsa("SS Revati"),
            Some(Ayanamsa::SuryasiddhantaRevati)
        );
        assert_eq!(
            resolve_ayanamsa("SS Citra"),
            Some(Ayanamsa::SuryasiddhantaCitra)
        );
        assert_eq!(resolve_ayanamsa("ICRC Lahiri"), Some(Ayanamsa::LahiriIcrc));
        assert_eq!(
            resolve_ayanamsa("Panchanga Darpan Lahiri"),
            Some(Ayanamsa::Lahiri1940)
        );
        assert_eq!(resolve_ayanamsa("Revati"), Some(Ayanamsa::UshaShashi));
        assert_eq!(
            resolve_ayanamsa("Usha Shashi ayanamsa"),
            Some(Ayanamsa::UshaShashi)
        );
        assert_eq!(resolve_ayanamsa("Moon"), Some(Ayanamsa::ValensMoon));
        assert_eq!(resolve_ayanamsa("Aryabhata"), Some(Ayanamsa::Aryabhata499));
        assert_eq!(
            resolve_ayanamsa("Aryabhata 499"),
            Some(Ayanamsa::Aryabhata499)
        );
        assert_eq!(
            resolve_ayanamsa("Aryabhata 499 CE"),
            Some(Ayanamsa::Aryabhata499)
        );
        assert_eq!(
            resolve_ayanamsa("Aryabhata Kaliyuga"),
            Some(Ayanamsa::Aryabhata499)
        );
        assert_eq!(
            resolve_ayanamsa("Suryasiddhanta 499"),
            Some(Ayanamsa::Suryasiddhanta499)
        );
        assert_eq!(
            resolve_ayanamsa("Surya Siddhanta 499"),
            Some(Ayanamsa::Suryasiddhanta499)
        );
        assert_eq!(
            resolve_ayanamsa("Suryasiddhanta 499 CE"),
            Some(Ayanamsa::Suryasiddhanta499)
        );
        assert_eq!(
            resolve_ayanamsa("Surya Siddhanta 499 CE"),
            Some(Ayanamsa::Suryasiddhanta499)
        );
        assert_eq!(resolve_ayanamsa("Zij al-Shah"), Some(Ayanamsa::Sassanian));
        assert_eq!(resolve_ayanamsa("Sasanian"), Some(Ayanamsa::Sassanian));
        assert_eq!(resolve_ayanamsa("De Luce"), Some(Ayanamsa::DeLuce));
        assert_eq!(resolve_ayanamsa("Yukteswar"), Some(Ayanamsa::Yukteshwar));
        assert_eq!(
            resolve_ayanamsa("Yukteshwar ayanamsa"),
            Some(Ayanamsa::Yukteshwar)
        );
        assert_eq!(
            resolve_ayanamsa("Sri Yukteshwar"),
            Some(Ayanamsa::Yukteshwar)
        );
        assert_eq!(
            resolve_ayanamsa("Shri Yukteswar"),
            Some(Ayanamsa::Yukteshwar)
        );
        assert_eq!(
            resolve_ayanamsa("Shri Yukteshwar"),
            Some(Ayanamsa::Yukteshwar)
        );
        assert_eq!(
            resolve_ayanamsa("P.V.R. Narasimha Rao"),
            Some(Ayanamsa::PvrPushyaPaksha)
        );
        assert_eq!(
            resolve_ayanamsa("True Pushya (PVRN Rao)"),
            Some(Ayanamsa::PvrPushyaPaksha)
        );
        assert_eq!(
            resolve_ayanamsa("Pushya-paksha"),
            Some(Ayanamsa::PvrPushyaPaksha)
        );
        assert_eq!(resolve_ayanamsa("Usha/Shashi"), Some(Ayanamsa::UshaShashi));
        assert_eq!(resolve_ayanamsa("Sunil Sheoran"), Some(Ayanamsa::Sheoran));
        assert_eq!(
            resolve_ayanamsa("\"Vedic\"/Sheoran"),
            Some(Ayanamsa::Sheoran)
        );
        assert_eq!(resolve_ayanamsa("Hipparchos"), Some(Ayanamsa::Hipparchus));
        assert_eq!(
            resolve_ayanamsa("Babylonian/Kugler 1"),
            Some(Ayanamsa::BabylonianKugler1)
        );
        assert_eq!(
            resolve_ayanamsa("Babylonian/Kugler 2"),
            Some(Ayanamsa::BabylonianKugler2)
        );
        assert_eq!(
            resolve_ayanamsa("Babylonian/Kugler 3"),
            Some(Ayanamsa::BabylonianKugler3)
        );
        assert_eq!(
            resolve_ayanamsa("Babylonian/Huber"),
            Some(Ayanamsa::BabylonianHuber)
        );
        assert_eq!(
            resolve_ayanamsa("Babylonian/Eta Piscium"),
            Some(Ayanamsa::BabylonianEtaPiscium)
        );
        assert_eq!(
            resolve_ayanamsa("Babylonian/Aldebaran = 15 Tau"),
            Some(Ayanamsa::BabylonianAldebaran)
        );
        assert_eq!(
            resolve_ayanamsa("Babylonian/Britton"),
            Some(Ayanamsa::BabylonianBritton)
        );
        assert_eq!(
            resolve_ayanamsa("BABYL_HOUSE"),
            Some(Ayanamsa::BabylonianHouse)
        );
        assert_eq!(
            resolve_ayanamsa("BABYL_SISSY"),
            Some(Ayanamsa::BabylonianSissy)
        );
        assert_eq!(
            resolve_ayanamsa("BABYL_TRUE_GEOC"),
            Some(Ayanamsa::BabylonianTrueGeoc)
        );
        assert_eq!(
            resolve_ayanamsa("BABYL_TRUE_TOPC"),
            Some(Ayanamsa::BabylonianTrueTopc)
        );
        assert_eq!(
            resolve_ayanamsa("BABYL_TRUE_OBS"),
            Some(Ayanamsa::BabylonianTrueObs)
        );
        assert_eq!(
            resolve_ayanamsa("BABYL_HOUSE_OBS"),
            Some(Ayanamsa::BabylonianHouseObs)
        );
        assert_eq!(
            resolve_ayanamsa("Galact. Center = 0 Sag"),
            Some(Ayanamsa::GalacticCenter)
        );
        assert_eq!(
            resolve_ayanamsa("Cochrane (Gal.Center = 0 Cap)"),
            Some(Ayanamsa::GalacticCenterCochrane)
        );
        assert_eq!(
            resolve_ayanamsa("Galactic Center (Gil Brand)"),
            Some(Ayanamsa::GalacticCenterRgilbrand)
        );
        assert_eq!(
            resolve_ayanamsa("Galactic Center (Rgilbrand)"),
            Some(Ayanamsa::GalacticCenterRgilbrand)
        );
        assert_eq!(
            resolve_ayanamsa("Skydram"),
            Some(Ayanamsa::GalacticCenterMardyks)
        );
        assert_eq!(
            resolve_ayanamsa("Skydram/Galactic Alignment"),
            Some(Ayanamsa::GalacticCenterMardyks)
        );
        assert_eq!(
            resolve_ayanamsa("Skydram (Mardyks)"),
            Some(Ayanamsa::GalacticCenterMardyks)
        );
        assert_eq!(
            resolve_ayanamsa("Mula Wilhelm"),
            Some(Ayanamsa::GalacticCenterMulaWilhelm)
        );
        assert_eq!(
            resolve_ayanamsa("Wilhelm"),
            Some(Ayanamsa::GalacticCenterMulaWilhelm)
        );
        assert_eq!(
            resolve_ayanamsa("True Mula (Chandra Hari)"),
            Some(Ayanamsa::TrueMula)
        );
        assert_eq!(
            resolve_ayanamsa("Dhruva/Gal.Center/Mula (Wilhelm)"),
            Some(Ayanamsa::GalacticCenterMulaWilhelm)
        );
        assert_eq!(
            resolve_ayanamsa("Gal. Eq."),
            Some(Ayanamsa::GalacticEquator)
        );
        assert_eq!(
            resolve_ayanamsa("Galactic Equator (IAU1958)"),
            Some(Ayanamsa::GalacticEquatorIau1958)
        );
        assert_eq!(
            resolve_ayanamsa("Galactic Equator mid-Mula"),
            Some(Ayanamsa::GalacticEquatorMula)
        );
        assert_eq!(resolve_ayanamsa("True Pushya"), Some(Ayanamsa::TruePushya));
        assert_eq!(resolve_ayanamsa("Udayagiri"), Some(Ayanamsa::Udayagiri));
        assert_eq!(resolve_ayanamsa("Djwhal"), Some(Ayanamsa::DjwhalKhul));
        assert_eq!(resolve_ayanamsa("J.N. Bhasin"), Some(Ayanamsa::JnBhasin));
        assert_eq!(resolve_ayanamsa("Bhasin"), Some(Ayanamsa::JnBhasin));
        assert_eq!(
            resolve_ayanamsa("Suryasiddhanta, mean Sun"),
            Some(Ayanamsa::Suryasiddhanta499MeanSun)
        );
        assert_eq!(
            resolve_ayanamsa("Surya Siddhanta, mean Sun"),
            Some(Ayanamsa::Suryasiddhanta499MeanSun)
        );
        assert_eq!(
            resolve_ayanamsa("Surya Siddhanta mean sun"),
            Some(Ayanamsa::Suryasiddhanta499MeanSun)
        );
        assert_eq!(
            resolve_ayanamsa("Aryabhata, mean Sun"),
            Some(Ayanamsa::Aryabhata499MeanSun)
        );
        assert_eq!(
            resolve_ayanamsa("Aryabhata 522"),
            Some(Ayanamsa::Aryabhata522)
        );
        assert_eq!(
            resolve_ayanamsa("Aryabhata 522 CE"),
            Some(Ayanamsa::Aryabhata522)
        );
        assert_eq!(resolve_ayanamsa("VP285"), Some(Ayanamsa::LahiriVP285));
        assert_eq!(resolve_ayanamsa("VP291"), Some(Ayanamsa::KrishnamurtiVP291));
        assert_eq!(
            resolve_ayanamsa("Krishnamurti-Senthilathiban"),
            Some(Ayanamsa::KrishnamurtiVP291)
        );
        assert_eq!(
            resolve_ayanamsa("Vettius Valens"),
            Some(Ayanamsa::ValensMoon)
        );
        assert_eq!(resolve_ayanamsa("Valens"), Some(Ayanamsa::ValensMoon));
        assert_eq!(
            resolve_ayanamsa("Moon sign ayanamsa"),
            Some(Ayanamsa::ValensMoon)
        );
        assert_eq!(
            resolve_ayanamsa("True Sheoran"),
            Some(Ayanamsa::TrueSheoran)
        );
    }

    #[test]
    fn release_catalog_includes_stage_six_ayanamsa_variants() {
        let names: Vec<_> = release_ayanamsas()
            .iter()
            .map(|entry| entry.canonical_name)
            .collect();

        for expected in [
            "True Citra",
            "J2000",
            "J1900",
            "B1950",
            "True Revati",
            "True Mula",
            "Suryasiddhanta (Revati)",
            "Suryasiddhanta (Citra)",
            "Lahiri (ICRC)",
            "Lahiri (1940)",
            "Usha Shashi",
            "Suryasiddhanta (499 CE)",
            "Aryabhata (499 CE)",
            "Sassanian",
            "DeLuce",
            "Yukteshwar",
            "PVR Pushya-paksha",
            "Sheoran",
            "Hipparchus",
            "Babylonian (Kugler 1)",
            "Babylonian (Kugler 2)",
            "Babylonian (Kugler 3)",
            "Babylonian (Huber)",
            "Babylonian (Eta Piscium)",
            "Babylonian (Aldebaran)",
            "Babylonian (House)",
            "Babylonian (Sissy)",
            "Babylonian (True Geoc)",
            "Babylonian (True Topc)",
            "Babylonian (True Obs)",
            "Babylonian (House Obs)",
            "True Pushya",
            "Udayagiri",
            "Djwhal Khul",
            "JN Bhasin",
            "Suryasiddhanta (Mean Sun)",
            "Aryabhata (Mean Sun)",
            "Babylonian (Britton)",
            "Aryabhata (522 CE)",
            "Lahiri (VP285)",
            "Krishnamurti (VP291)",
            "True Sheoran",
            "Galactic Center",
            "Galactic Center (Rgilbrand)",
            "Galactic Center (Mardyks)",
            "Galactic Center (Mula/Wilhelm)",
            "Dhruva Galactic Center (Middle Mula)",
            "Galactic Center (Cochrane)",
            "Galactic Equator",
            "Galactic Equator (IAU 1958)",
            "Galactic Equator (True)",
            "Galactic Equator (Mula)",
            "Galactic Equator (Fiorenza)",
            "Valens Moon",
        ] {
            assert!(names.contains(&expected), "missing {expected}");
        }
    }

    #[test]
    fn release_descriptor_aliases_do_not_repeat_canonical_labels() {
        assert!(built_in_ayanamsas()
            .iter()
            .all(|entry| { !entry.aliases.contains(&entry.canonical_name) }));
    }

    #[test]
    fn ayanamsa_catalog_round_trips_all_built_ins_and_aliases() {
        use std::collections::HashSet;

        let built_in = built_in_ayanamsas();
        let mut unique_names = HashSet::new();

        assert_eq!(
            built_in.len(),
            baseline_ayanamsas().len() + release_ayanamsas().len()
        );

        for entry in baseline_ayanamsas()
            .iter()
            .chain(release_ayanamsas().iter())
        {
            assert!(
                unique_names.insert(entry.canonical_name),
                "duplicate canonical ayanamsa name {}",
                entry.canonical_name
            );
            assert_eq!(
                descriptor(&entry.ayanamsa).map(|d| d.canonical_name),
                Some(entry.canonical_name)
            );
            assert_eq!(
                resolve_ayanamsa(entry.canonical_name),
                Some(entry.ayanamsa.clone())
            );
            for alias in entry.aliases {
                assert_eq!(resolve_ayanamsa(alias), Some(entry.ayanamsa.clone()));
            }
        }

        for entry in built_in {
            assert!(unique_names.contains(entry.canonical_name));
        }
    }

    #[test]
    fn sidereal_offset_is_available_for_baseline_ayanamsas() {
        let instant = Instant::new(
            JulianDay::from_days(2_451_545.0),
            pleiades_types::TimeScale::Tt,
        );
        let offset = sidereal_offset(&Ayanamsa::Lahiri, instant).expect("offset should exist");
        assert!(offset.degrees().is_finite());
    }

    #[test]
    fn selected_release_ayanamsas_carry_reference_metadata() {
        let hipparchus = descriptor(&Ayanamsa::Hipparchus).expect("Hipparchus descriptor");
        assert_eq!(hipparchus.epoch, Some(JulianDay::from_days(1_674_484.0)));
        assert_eq!(
            hipparchus.offset_degrees,
            Some(Angle::from_degrees(-9.333_333_333_333_334))
        );

        let jn_bhasin = descriptor(&Ayanamsa::JnBhasin).expect("JN Bhasin descriptor");
        assert_eq!(
            jn_bhasin.epoch,
            Some(JulianDay::from_days(2_454_239.282_537))
        );
        assert_eq!(
            jn_bhasin.offset_degrees,
            Some(Angle::from_degrees(0.013_968_911_416_666_667))
        );

        let true_citra = descriptor(&Ayanamsa::TrueCitra).expect("True Citra descriptor");
        assert_eq!(
            true_citra.epoch,
            Some(JulianDay::from_days(1_825_182.872_330))
        );
        assert_eq!(
            true_citra.offset_degrees,
            Some(Angle::from_degrees(50.256_748_3))
        );
        assert_eq!(
            sidereal_offset(
                &Ayanamsa::TrueCitra,
                Instant::new(
                    JulianDay::from_days(1_825_182.872_330),
                    pleiades_types::TimeScale::Tt
                ),
            ),
            Some(Angle::from_degrees(50.256_748_3))
        );

        let kugler1 =
            descriptor(&Ayanamsa::BabylonianKugler1).expect("Babylonian Kugler 1 descriptor");
        assert_eq!(kugler1.epoch, Some(JulianDay::from_days(1_833_923.577_692)));
        assert_eq!(kugler1.offset_degrees, Some(Angle::from_degrees(0.0)));

        let kugler2 =
            descriptor(&Ayanamsa::BabylonianKugler2).expect("Babylonian Kugler 2 descriptor");
        assert_eq!(kugler2.epoch, Some(JulianDay::from_days(1_797_039.206_820)));
        assert_eq!(kugler2.offset_degrees, Some(Angle::from_degrees(0.0)));

        let eta_piscium =
            descriptor(&Ayanamsa::BabylonianEtaPiscium).expect("Babylonian Eta Piscium descriptor");
        assert_eq!(
            eta_piscium.epoch,
            Some(JulianDay::from_days(1_807_871.964_797))
        );
        assert_eq!(eta_piscium.offset_degrees, Some(Angle::from_degrees(0.0)));

        let aldebaran =
            descriptor(&Ayanamsa::BabylonianAldebaran).expect("Babylonian Aldebaran descriptor");
        assert_eq!(
            aldebaran.epoch,
            Some(JulianDay::from_days(1_801_643.133_503))
        );
        assert_eq!(aldebaran.offset_degrees, Some(Angle::from_degrees(0.0)));

        let galactic_true =
            descriptor(&Ayanamsa::GalacticEquatorTrue).expect("Galactic Equator (True) descriptor");
        assert_eq!(
            galactic_true.epoch,
            Some(JulianDay::from_days(1_665_728.603_158))
        );
        assert_eq!(galactic_true.offset_degrees, Some(Angle::from_degrees(0.0)));

        let galactic = descriptor(&Ayanamsa::GalacticEquatorIau1958)
            .expect("Galactic Equator (IAU 1958) descriptor");
        assert_eq!(
            galactic.epoch,
            Some(JulianDay::from_days(1_667_118.376_332))
        );
        assert_eq!(galactic.offset_degrees, Some(Angle::from_degrees(0.0)));

        let galactic_mula =
            descriptor(&Ayanamsa::GalacticEquatorMula).expect("Galactic Equator (Mula) descriptor");
        assert_eq!(
            galactic_mula.epoch,
            Some(JulianDay::from_days(1_840_527.426_262))
        );
        assert_eq!(galactic_mula.offset_degrees, Some(Angle::from_degrees(0.0)));

        let valens = descriptor(&Ayanamsa::ValensMoon).expect("Valens Moon descriptor");
        assert_eq!(valens.epoch, Some(JulianDay::from_days(1_775_845.5)));
        assert_eq!(valens.offset_degrees, Some(Angle::from_degrees(-2.942_2)));

        let udayagiri = descriptor(&Ayanamsa::Udayagiri).expect("Udayagiri descriptor");
        assert_eq!(
            udayagiri.epoch,
            Some(JulianDay::from_days(1_825_235.164_583))
        );
        assert_eq!(udayagiri.offset_degrees, Some(Angle::from_degrees(0.0)));

        let vp285 = descriptor(&Ayanamsa::LahiriVP285).expect("Lahiri VP285 descriptor");
        assert_eq!(vp285.epoch, Some(JulianDay::from_days(1_825_235.164_583)));
        assert_eq!(vp285.offset_degrees, Some(Angle::from_degrees(0.0)));

        let kugler3 =
            descriptor(&Ayanamsa::BabylonianKugler3).expect("Babylonian Kugler 3 descriptor");
        assert_eq!(kugler3.epoch, Some(JulianDay::from_days(1_774_637.420_172)));
        assert_eq!(kugler3.offset_degrees, Some(Angle::from_degrees(0.0)));

        let britton =
            descriptor(&Ayanamsa::BabylonianBritton).expect("Babylonian Britton descriptor");
        assert_eq!(britton.epoch, Some(JulianDay::from_days(1_805_415.712_776)));
        assert_eq!(britton.offset_degrees, Some(Angle::from_degrees(0.0)));

        let cochrane = descriptor(&Ayanamsa::GalacticCenterCochrane)
            .expect("Galactic Center (Cochrane) descriptor");
        assert_eq!(
            cochrane.epoch,
            Some(JulianDay::from_days(1_662_951.794_251))
        );
        assert_eq!(cochrane.offset_degrees, Some(Angle::from_degrees(0.0)));

        let mardyks = descriptor(&Ayanamsa::GalacticCenterMardyks)
            .expect("Galactic Center (Mardyks) descriptor");
        assert_eq!(mardyks.epoch, Some(JulianDay::from_days(1_662_951.794_251)));
        assert_eq!(mardyks.offset_degrees, Some(Angle::from_degrees(0.0)));

        let true_pushya = descriptor(&Ayanamsa::TruePushya).expect("True Pushya descriptor");
        assert_eq!(
            true_pushya.epoch,
            Some(JulianDay::from_days(1_855_769.248_315))
        );
        assert_eq!(true_pushya.offset_degrees, Some(Angle::from_degrees(0.0)));

        let ss_revati =
            descriptor(&Ayanamsa::SuryasiddhantaRevati).expect("Suryasiddhanta Revati descriptor");
        assert_eq!(
            ss_revati.epoch,
            Some(JulianDay::from_days(1_924_230.267_296))
        );
        assert_eq!(ss_revati.offset_degrees, Some(Angle::from_degrees(0.0)));

        let ss_citra =
            descriptor(&Ayanamsa::SuryasiddhantaCitra).expect("Suryasiddhanta Citra descriptor");
        assert_eq!(
            ss_citra.epoch,
            Some(JulianDay::from_days(1_903_396.812_865_4))
        );
        assert_eq!(ss_citra.offset_degrees, Some(Angle::from_degrees(0.0)));

        let djwhal = descriptor(&Ayanamsa::DjwhalKhul).expect("Djwhal Khul descriptor");
        assert_eq!(djwhal.epoch, Some(JulianDay::from_days(1_706_703.948_006)));
        assert_eq!(djwhal.offset_degrees, Some(Angle::from_degrees(0.0)));

        let sheoran = descriptor(&Ayanamsa::Sheoran).expect("Sheoran descriptor");
        assert_eq!(sheoran.epoch, Some(JulianDay::from_days(1_789_947.090_881)));
        assert_eq!(sheoran.offset_degrees, Some(Angle::from_degrees(0.0)));

        let instant = Instant::new(
            JulianDay::from_days(2_451_545.0),
            pleiades_types::TimeScale::Tt,
        );
        assert!(sidereal_offset(&Ayanamsa::BabylonianHuber, instant)
            .expect("Huber offset should exist")
            .degrees()
            .is_finite());
        assert!(sidereal_offset(&Ayanamsa::GalacticEquatorIau1958, instant)
            .expect("Galactic Equator offset should exist")
            .degrees()
            .is_finite());
        assert_eq!(
            sidereal_offset(
                &Ayanamsa::TruePushya,
                Instant::new(
                    JulianDay::from_days(1_855_769.248_315),
                    pleiades_types::TimeScale::Tt
                ),
            ),
            Some(Angle::from_degrees(0.0))
        );
        assert_eq!(
            sidereal_offset(
                &Ayanamsa::DjwhalKhul,
                Instant::new(
                    JulianDay::from_days(1_706_703.948_006),
                    pleiades_types::TimeScale::Tt
                ),
            ),
            Some(Angle::from_degrees(0.0))
        );
        assert_eq!(
            sidereal_offset(
                &Ayanamsa::Sheoran,
                Instant::new(
                    JulianDay::from_days(1_789_947.090_881),
                    pleiades_types::TimeScale::Tt
                ),
            ),
            Some(Angle::from_degrees(0.0))
        );
        assert!(sidereal_offset(&Ayanamsa::ValensMoon, instant)
            .expect("Valens Moon offset should exist")
            .degrees()
            .is_finite());
        assert_eq!(
            sidereal_offset(
                &Ayanamsa::GalacticCenterCochrane,
                Instant::new(
                    JulianDay::from_days(1_662_951.794_251),
                    pleiades_types::TimeScale::Tt
                ),
            ),
            Some(Angle::from_degrees(0.0))
        );
    }

    #[test]
    fn metadata_coverage_reports_remaining_gaps() {
        let coverage = metadata_coverage();
        let expected_custom_definition_only: Vec<_> = [
            "Babylonian (House)",
            "Babylonian (Sissy)",
            "Babylonian (True Geoc)",
            "Babylonian (True Topc)",
            "Babylonian (True Obs)",
            "Babylonian (House Obs)",
        ]
        .into_iter()
        .collect();
        let expected_without: Vec<_> = built_in_ayanamsas()
            .iter()
            .filter(|entry| {
                !entry.has_sidereal_metadata()
                    && !super::is_custom_definition_only_ayanamsa(entry.canonical_name)
            })
            .map(|entry| entry.canonical_name)
            .collect();

        assert_eq!(coverage.total, built_in_ayanamsas().len());
        assert_eq!(
            coverage.with_sidereal_metadata
                + coverage.custom_definition_only.len()
                + coverage.without_sidereal_metadata.len(),
            coverage.total
        );
        assert_eq!(
            coverage.custom_definition_only,
            expected_custom_definition_only
        );
        assert_eq!(coverage.without_sidereal_metadata, expected_without);
        assert!(coverage.is_complete());
        assert!(coverage
            .custom_definition_only
            .iter()
            .all(|name| name.starts_with("Babylonian (")));
        assert!(coverage.without_sidereal_metadata.is_empty());
    }

    #[test]
    fn krishnamurti_vp291_descriptor_uses_the_published_zero_point() {
        let descriptor =
            descriptor(&Ayanamsa::KrishnamurtiVP291).expect("Krishnamurti VP291 descriptor");
        assert_eq!(
            descriptor.epoch,
            Some(JulianDay::from_days(1_827_424.663_554))
        );
        assert_eq!(descriptor.offset_degrees, Some(Angle::from_degrees(0.0)));
        assert_eq!(
            descriptor.offset_at(Instant::new(
                JulianDay::from_days(1_827_424.663_554),
                pleiades_types::TimeScale::Tt
            )),
            Some(Angle::from_degrees(0.0))
        );
    }

    #[test]
    fn scheduled_historical_reference_modes_use_the_published_zero_points() {
        let true_sheoran = descriptor(&Ayanamsa::TrueSheoran).expect("True Sheoran descriptor");
        assert_eq!(
            true_sheoran.epoch,
            Some(JulianDay::from_days(1_789_947.090_881))
        );
        assert_eq!(true_sheoran.offset_degrees, Some(Angle::from_degrees(0.0)));
        assert_eq!(
            true_sheoran.offset_at(Instant::new(
                JulianDay::from_days(1_789_947.090_881),
                pleiades_types::TimeScale::Tt
            )),
            Some(Angle::from_degrees(0.0))
        );

        let rgilbrand = descriptor(&Ayanamsa::GalacticCenterRgilbrand)
            .expect("Galactic Center (Rgilbrand) descriptor");
        assert_eq!(
            rgilbrand.epoch,
            Some(JulianDay::from_days(1_861_740.329_525))
        );
        assert_eq!(rgilbrand.offset_degrees, Some(Angle::from_degrees(0.0)));
        assert_eq!(
            rgilbrand.offset_at(Instant::new(
                JulianDay::from_days(1_861_740.329_525),
                pleiades_types::TimeScale::Tt
            )),
            Some(Angle::from_degrees(0.0))
        );

        let mula_wilhelm = descriptor(&Ayanamsa::GalacticCenterMulaWilhelm)
            .expect("Galactic Center (Mula/Wilhelm) descriptor");
        assert_eq!(
            mula_wilhelm.epoch,
            Some(JulianDay::from_days(1_946_834.818_321))
        );
        assert_eq!(mula_wilhelm.offset_degrees, Some(Angle::from_degrees(0.0)));
        assert_eq!(
            mula_wilhelm.offset_at(Instant::new(
                JulianDay::from_days(1_946_834.818_321),
                pleiades_types::TimeScale::Tt
            )),
            Some(Angle::from_degrees(0.0))
        );
    }

    #[test]
    fn custom_ayanamsa_uses_explicit_epoch_and_offset_metadata() {
        let custom = Ayanamsa::Custom(CustomAyanamsa {
            name: "True Balarama".to_owned(),
            description: Some("Custom label for a non-built-in sidereal variant".to_owned()),
            epoch: Some(JulianDay::from_days(2_451_545.0)),
            offset_degrees: Some(Angle::from_degrees(12.5)),
        });
        let instant = Instant::new(
            JulianDay::from_days(2_451_545.0),
            pleiades_types::TimeScale::Tt,
        );

        let offset = sidereal_offset(&custom, instant).expect("custom offset should exist");
        assert_eq!(offset, Angle::from_degrees(12.5));
    }
}
