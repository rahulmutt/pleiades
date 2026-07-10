//! Per-mode-class arcsecond ceilings for the ayanamsa numeric gate.
//! Mirrors `pleiades-houses/src/thresholds.rs`: ceilings are set to
//! `ceil(measured_max × 2)` over the committed SE corpus, with a 1.0″ floor.
#![forbid(unsafe_code)]

use pleiades_types::Ayanamsa;

/// Computation class of a gated ayanamsa mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AyanamsaModeClass {
    /// Fixed offset at a reference epoch plus general precession (e.g. Lahiri,
    /// Raman, Krishnamurti, Fagan/Bradley, plus the 17 promoted offset modes
    /// such as J2000/J1900/B1950 and the Suryasiddhanta/Aryabhata families).
    OffsetDefined,
    /// Sidereal point pinned to a fixed star, fit to Swiss Ephemeris
    /// (True Chitra, True Citra, True Revati, True Pushya, True Mula, True Sheoran).
    TrueStar,
    /// Sidereal point pinned to a galactic reference (center / equator), fit to
    /// Swiss Ephemeris mean ayanamsa.
    Galactic,
    /// Offset/historical sidereal mode whose linear anchor+precession model
    /// missed the OffsetDefined ceiling, promoted instead by a committed cubic
    /// fit to Swiss Ephemeris mean ayanamsa (same mechanism as TrueStar/Galactic).
    FittedOffset,
}

/// Arcsecond ceiling for one mode class.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AyanamsaCeiling {
    /// Max allowed |residual| on the sidereal offset, arcseconds.
    pub offset_arcsec: f64,
}

/// Returns the measured-derived ceiling for a mode class.
///
/// Ceilings are `ceil(measured_max × 2)` over the committed SE **mean**-ayanamsa
/// corpus (230 rows, 1900–2100), with a 1.0″ floor. Measured maxima (Task 1 residuals):
/// - OffsetDefined: max residual 1.370402″ → ceil(2 × 1.370402) = ceil(2.740804) = 3.0″.
/// - TrueStar (6 modes): max residual ≈ 0.175″ (TrueRevati) → ceil(2 × 0.175) = ceil(0.350) = 1.0″ (floor).
/// - Galactic (9 modes): max residual ≈ 0.057″ (GalacticCenterMulaWilhelm) → ceil(2 × 0.057) = ceil(0.114) = 1.0″ (floor).
/// - FittedOffset (12 modes): max residual ≈ 0.000005″ (5e-6″) → ceil(2 × 0.000005) = ceil(0.00001) = 1.0″ (floor).
pub fn ayanamsa_mode_ceiling(class: AyanamsaModeClass) -> AyanamsaCeiling {
    match class {
        AyanamsaModeClass::OffsetDefined => AyanamsaCeiling { offset_arcsec: 3.0 },
        AyanamsaModeClass::TrueStar => AyanamsaCeiling { offset_arcsec: 1.0 },
        AyanamsaModeClass::Galactic => AyanamsaCeiling { offset_arcsec: 1.0 },
        AyanamsaModeClass::FittedOffset => AyanamsaCeiling { offset_arcsec: 1.0 },
    }
}

/// Maps a typed ayanamsa to its gated mode class, or `None` if it is not gated.
pub fn ayanamsa_mode_class(ayanamsa: &Ayanamsa) -> Option<AyanamsaModeClass> {
    match ayanamsa {
        // Original 4 offset-defined modes.
        Ayanamsa::Lahiri
        | Ayanamsa::Raman
        | Ayanamsa::Krishnamurti
        | Ayanamsa::FaganBradley
        // Promoted P set (17 modes, empirically validated in Task 2).
        | Ayanamsa::J2000
        | Ayanamsa::J1900
        | Ayanamsa::B1950
        | Ayanamsa::UshaShashi
        | Ayanamsa::DjwhalKhul
        | Ayanamsa::Yukteshwar
        | Ayanamsa::JnBhasin
        | Ayanamsa::Sassanian
        | Ayanamsa::LahiriIcrc
        | Ayanamsa::Lahiri1940
        | Ayanamsa::Aryabhata522
        | Ayanamsa::Suryasiddhanta499
        | Ayanamsa::Suryasiddhanta499MeanSun
        | Ayanamsa::Aryabhata499
        | Ayanamsa::Aryabhata499MeanSun
        | Ayanamsa::SuryasiddhantaRevati
        | Ayanamsa::SuryasiddhantaCitra => Some(AyanamsaModeClass::OffsetDefined),
        Ayanamsa::TrueChitra | Ayanamsa::TrueCitra
        | Ayanamsa::TrueRevati | Ayanamsa::TruePushya
        | Ayanamsa::TrueMula | Ayanamsa::TrueSheoran => Some(AyanamsaModeClass::TrueStar),
        Ayanamsa::GalacticCenter
        | Ayanamsa::GalacticCenterRgilbrand
        | Ayanamsa::GalacticEquatorIau1958
        | Ayanamsa::GalacticEquatorTrue
        | Ayanamsa::GalacticEquatorMula
        | Ayanamsa::GalacticCenterMardyks
        | Ayanamsa::GalacticCenterMulaWilhelm
        | Ayanamsa::GalacticCenterCochrane
        | Ayanamsa::GalacticEquatorFiorenza => Some(AyanamsaModeClass::Galactic),
        Ayanamsa::DeLuce
        | Ayanamsa::BabylonianKugler1
        | Ayanamsa::BabylonianKugler2
        | Ayanamsa::BabylonianKugler3
        | Ayanamsa::BabylonianHuber
        | Ayanamsa::BabylonianEtaPiscium
        | Ayanamsa::BabylonianAldebaran
        | Ayanamsa::Hipparchus
        | Ayanamsa::BabylonianBritton
        | Ayanamsa::ValensMoon
        | Ayanamsa::LahiriVP285
        | Ayanamsa::KrishnamurtiVP291 => Some(AyanamsaModeClass::FittedOffset),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_class_has_finite_positive_ceiling() {
        for class in [
            AyanamsaModeClass::OffsetDefined,
            AyanamsaModeClass::TrueStar,
            AyanamsaModeClass::Galactic,
            AyanamsaModeClass::FittedOffset,
        ] {
            let c = ayanamsa_mode_ceiling(class);
            assert!(c.offset_arcsec.is_finite() && c.offset_arcsec > 0.0);
        }
    }

    #[test]
    fn promoted_offset_modes_are_gated_as_offset_defined() {
        // All 17 promoted P modes must map to OffsetDefined.
        for m in [
            Ayanamsa::J2000,
            Ayanamsa::J1900,
            Ayanamsa::B1950,
            Ayanamsa::UshaShashi,
            Ayanamsa::DjwhalKhul,
            Ayanamsa::Yukteshwar,
            Ayanamsa::JnBhasin,
            Ayanamsa::Sassanian,
            Ayanamsa::LahiriIcrc,
            Ayanamsa::Lahiri1940,
            Ayanamsa::Aryabhata522,
            Ayanamsa::Suryasiddhanta499,
            Ayanamsa::Suryasiddhanta499MeanSun,
            Ayanamsa::Aryabhata499,
            Ayanamsa::Aryabhata499MeanSun,
            Ayanamsa::SuryasiddhantaRevati,
            Ayanamsa::SuryasiddhantaCitra,
        ] {
            assert_eq!(
                ayanamsa_mode_class(&m),
                Some(AyanamsaModeClass::OffsetDefined),
                "{m:?} should be OffsetDefined"
            );
        }
        // Original 4 offset modes still gated.
        assert_eq!(
            ayanamsa_mode_class(&Ayanamsa::Lahiri),
            Some(AyanamsaModeClass::OffsetDefined)
        );
        // TrueStar sanity check.
        assert_eq!(
            ayanamsa_mode_class(&Ayanamsa::TrueChitra),
            Some(AyanamsaModeClass::TrueStar)
        );
        // Promoted to FittedOffset (not OffsetDefined, not None).
        assert_eq!(
            ayanamsa_mode_class(&Ayanamsa::KrishnamurtiVP291),
            Some(AyanamsaModeClass::FittedOffset)
        );
        assert_eq!(
            ayanamsa_mode_class(&Ayanamsa::LahiriVP285),
            Some(AyanamsaModeClass::FittedOffset)
        );
        // Galactic-class control (now gated as Galactic, not OffsetDefined).
        assert_eq!(
            ayanamsa_mode_class(&Ayanamsa::GalacticCenter),
            Some(AyanamsaModeClass::Galactic)
        );
    }

    #[test]
    fn promoted_fitted_modes_map_to_their_class() {
        for m in [
            Ayanamsa::TrueRevati,
            Ayanamsa::TruePushya,
            Ayanamsa::TrueMula,
            Ayanamsa::TrueSheoran,
        ] {
            assert_eq!(
                ayanamsa_mode_class(&m),
                Some(AyanamsaModeClass::TrueStar),
                "{m:?}"
            );
        }
        for m in [
            Ayanamsa::GalacticCenter,
            Ayanamsa::GalacticCenterRgilbrand,
            Ayanamsa::GalacticEquatorIau1958,
            Ayanamsa::GalacticEquatorTrue,
            Ayanamsa::GalacticEquatorMula,
            Ayanamsa::GalacticCenterMardyks,
            Ayanamsa::GalacticCenterMulaWilhelm,
            Ayanamsa::GalacticCenterCochrane,
            Ayanamsa::GalacticEquatorFiorenza,
        ] {
            assert_eq!(
                ayanamsa_mode_class(&m),
                Some(AyanamsaModeClass::Galactic),
                "{m:?}"
            );
        }
        // Expected-deferred: no distinct SE code -> stay ungated.
        assert_eq!(
            ayanamsa_mode_class(&Ayanamsa::DhruvaGalacticCenterMula),
            None
        );
        assert_eq!(ayanamsa_mode_class(&Ayanamsa::GalacticEquator), None);
    }

    #[test]
    fn promoted_fitted_offset_modes_map_to_their_class() {
        for m in [
            Ayanamsa::DeLuce,
            Ayanamsa::BabylonianKugler1,
            Ayanamsa::BabylonianKugler2,
            Ayanamsa::BabylonianKugler3,
            Ayanamsa::BabylonianHuber,
            Ayanamsa::BabylonianEtaPiscium,
            Ayanamsa::BabylonianAldebaran,
            Ayanamsa::Hipparchus,
            Ayanamsa::BabylonianBritton,
            Ayanamsa::ValensMoon,
            Ayanamsa::LahiriVP285,
            Ayanamsa::KrishnamurtiVP291,
        ] {
            assert_eq!(
                ayanamsa_mode_class(&m),
                Some(AyanamsaModeClass::FittedOffset),
                "{m:?}"
            );
        }
        // No-SE_SIDM anchorless modes stay ungated.
        assert_eq!(ayanamsa_mode_class(&Ayanamsa::Udayagiri), None);
        assert_eq!(ayanamsa_mode_class(&Ayanamsa::PvrPushyaPaksha), None);
        assert_eq!(ayanamsa_mode_class(&Ayanamsa::Sheoran), None);
    }
}
