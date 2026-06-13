//! Ayanamsa types: [`Ayanamsa`] and [`CustomAyanamsa`].

use core::fmt;

use crate::angles::Angle;
use crate::custom_bodies::{validate_canonical_text, CustomDefinitionValidationError};
use crate::time::JulianDay;

/// A built-in or custom ayanamsa selection.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// True Citra.
    TrueCitra,
    /// Suryasiddhanta (Revati) / SS Revati.
    SuryasiddhantaRevati,
    /// Suryasiddhanta (Citra) / SS Citra.
    SuryasiddhantaCitra,
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
    /// Udayagiri.
    Udayagiri,
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
    /// Babylonian (House).
    BabylonianHouse,
    /// Babylonian (Sissy).
    BabylonianSissy,
    /// Babylonian (True Geoc).
    BabylonianTrueGeoc,
    /// Babylonian (True Topc).
    BabylonianTrueTopc,
    /// Babylonian (True Obs).
    BabylonianTrueObs,
    /// Babylonian (House Obs).
    BabylonianHouseObs,
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
    /// Dhruva Galactic Center (Middle Mula).
    DhruvaGalacticCenterMula,
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

impl fmt::Display for Ayanamsa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Custom(custom) => write!(f, "{custom}"),
            _ => write!(f, "{self:?}"),
        }
    }
}

impl Ayanamsa {
    /// Validates the custom ayanamsa definition when this is a custom variant.
    ///
    /// Built-in ayanamsas are always valid; only structured custom labels and
    /// offset metadata are checked. This keeps malformed sidereal definitions
    /// from leaking into request validation or release-facing text.
    pub fn validate(&self) -> Result<(), CustomDefinitionValidationError> {
        match self {
            Self::Custom(custom) => custom.validate(),
            _ => Ok(()),
        }
    }

    /// Validates the custom ayanamsa definition against reserved built-in labels.
    ///
    /// Built-in ayanamsas are always valid; custom entries are checked with the
    /// same structural rules as [`validate`](Self::validate) and are then compared
    /// against a caller-supplied built-in label resolver.
    pub fn validate_against_reserved_labels(
        &self,
        is_reserved_label: impl Fn(&str) -> bool,
    ) -> Result<(), CustomDefinitionValidationError> {
        match self {
            Self::Custom(custom) => custom.validate_against_reserved_labels(is_reserved_label),
            _ => Ok(()),
        }
    }
}

/// A structured custom ayanamsa definition.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    /// Validates the custom ayanamsa definition.
    ///
    /// The canonical name and optional description must be non-empty and free
    /// of leading or trailing whitespace. If one of `epoch` or
    /// `offset_degrees` is supplied, the other must be supplied too so the
    /// sidereal offset can be reconstructed deterministically. Any supplied
    /// numeric values must also be finite.
    pub fn validate(&self) -> Result<(), CustomDefinitionValidationError> {
        validate_canonical_text("custom ayanamsa", "name", &self.name)?;

        if let Some(description) = &self.description {
            validate_canonical_text("custom ayanamsa", "description", description)?;
        }

        match (self.epoch, self.offset_degrees) {
            (Some(epoch), Some(offset_degrees)) => {
                if !epoch.days().is_finite() {
                    return Err(CustomDefinitionValidationError::non_finite(
                        "custom ayanamsa",
                        "epoch",
                    ));
                }

                if !offset_degrees.is_finite() {
                    return Err(CustomDefinitionValidationError::non_finite(
                        "custom ayanamsa",
                        "offset_degrees",
                    ));
                }
            }
            (None, None) => {}
            (Some(_), None) => {
                return Err(CustomDefinitionValidationError::incomplete_pair(
                    "custom ayanamsa",
                    "epoch",
                    "offset_degrees",
                ));
            }
            (None, Some(_)) => {
                return Err(CustomDefinitionValidationError::incomplete_pair(
                    "custom ayanamsa",
                    "offset_degrees",
                    "epoch",
                ));
            }
        }

        Ok(())
    }

    /// Validates the custom ayanamsa definition against reserved built-in labels.
    ///
    /// This is a stricter form of [`validate`](Self::validate) that also rejects
    /// a canonical name that collides with a published built-in ayanamsa label.
    /// That keeps user-defined sidereal definitions distinguishable from the
    /// built-in compatibility catalog when request validation or release profiles
    /// surface them.
    pub fn validate_against_reserved_labels(
        &self,
        is_reserved_label: impl Fn(&str) -> bool,
    ) -> Result<(), CustomDefinitionValidationError> {
        self.validate()?;

        if is_reserved_label(&self.name) {
            return Err(CustomDefinitionValidationError::reserved_label(
                "custom ayanamsa",
                "name",
                &self.name,
            ));
        }

        Ok(())
    }
}

impl fmt::Display for CustomAyanamsa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)?;

        let mut details = Vec::new();
        if let Some(epoch) = self.epoch {
            details.push(format!("epoch: {epoch}"));
        }
        if let Some(offset_degrees) = self.offset_degrees {
            details.push(format!("offset: {offset_degrees}"));
        }
        if let Some(description) = &self.description {
            details.push(description.clone());
        }

        if !details.is_empty() {
            write!(f, " [{}]", details.join(", "))?;
        }

        Ok(())
    }
}
