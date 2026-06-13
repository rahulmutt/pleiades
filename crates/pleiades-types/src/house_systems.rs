//! House system types: [`HouseSystem`] and [`CustomHouseSystem`].

use core::fmt;

use crate::custom_bodies::{validate_canonical_text, CustomDefinitionValidationError};

/// A built-in or custom house system selection.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl fmt::Display for HouseSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Placidus => f.write_str("Placidus"),
            Self::Koch => f.write_str("Koch"),
            Self::Porphyry => f.write_str("Porphyry"),
            Self::Regiomontanus => f.write_str("Regiomontanus"),
            Self::Campanus => f.write_str("Campanus"),
            Self::Carter => f.write_str("Carter (poli-equatorial)"),
            Self::Horizon => f.write_str("Horizon/Azimuth"),
            Self::Apc => f.write_str("APC"),
            Self::KrusinskiPisaGoelzer => f.write_str("Krusinski-Pisa-Goelzer"),
            Self::Equal => f.write_str("Equal"),
            Self::EqualMidheaven => f.write_str("Equal (MC)"),
            Self::EqualAries => f.write_str("Equal (1=Aries)"),
            Self::Vehlow => f.write_str("Vehlow Equal"),
            Self::Sripati => f.write_str("Sripati"),
            Self::WholeSign => f.write_str("Whole Sign"),
            Self::Alcabitius => f.write_str("Alcabitius"),
            Self::Albategnius => f.write_str("Albategnius"),
            Self::PullenSd => f.write_str("Pullen SD"),
            Self::PullenSr => f.write_str("Pullen SR"),
            Self::Meridian => f.write_str("Meridian"),
            Self::Axial => f.write_str("Axial"),
            Self::Topocentric => f.write_str("Topocentric"),
            Self::Morinus => f.write_str("Morinus"),
            Self::Sunshine => f.write_str("Sunshine"),
            Self::Gauquelin => f.write_str("Gauquelin sectors"),
            Self::Custom(custom) => fmt::Display::fmt(custom, f),
        }
    }
}

impl HouseSystem {
    /// Validates the custom house-system definition when this is a custom variant.
    ///
    /// Built-in house systems are always valid; only structured custom labels,
    /// aliases, and notes are checked. This keeps malformed catalog entries
    /// from leaking into request summaries or release profiles.
    pub fn validate(&self) -> Result<(), CustomDefinitionValidationError> {
        match self {
            Self::Custom(custom) => custom.validate(),
            _ => Ok(()),
        }
    }

    /// Validates the custom house-system definition against reserved built-in labels.
    ///
    /// Built-in house systems are always valid; custom entries are checked with
    /// the same structural rules as [`validate`](Self::validate) and are then
    /// compared against a caller-supplied built-in label resolver.
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

/// A structured custom house-system definition.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    /// Validates the custom house-system definition.
    ///
    /// The canonical name must be non-empty and free of leading or trailing
    /// whitespace. Aliases and notes, when present, must follow the same
    /// canonical-text rule, and aliases must be unique after ASCII
    /// case-normalization, including against the canonical name.
    pub fn validate(&self) -> Result<(), CustomDefinitionValidationError> {
        validate_canonical_text("custom house system", "name", &self.name)?;

        let mut aliases: Vec<&str> = Vec::with_capacity(self.aliases.len());
        for (index, alias) in self.aliases.iter().enumerate() {
            let field = format!("alias[{index}]");
            validate_canonical_text("custom house system", field, alias)?;

            if self.name.eq_ignore_ascii_case(alias)
                || aliases
                    .iter()
                    .any(|existing| existing.eq_ignore_ascii_case(alias))
            {
                return Err(CustomDefinitionValidationError::duplicate_alias(
                    "custom house system",
                    alias,
                ));
            }

            aliases.push(alias);
        }

        if let Some(notes) = &self.notes {
            validate_canonical_text("custom house system", "notes", notes)?;
        }

        Ok(())
    }

    /// Validates the custom house-system definition against reserved built-in labels.
    ///
    /// This is a stricter form of [`validate`](Self::validate) that also rejects
    /// a canonical name or alias that collides with a published built-in house-system
    /// label. That keeps user-defined catalog entries distinguishable from the
    /// built-in compatibility catalog when request validation or release profiles
    /// surface them.
    pub fn validate_against_reserved_labels(
        &self,
        is_reserved_label: impl Fn(&str) -> bool,
    ) -> Result<(), CustomDefinitionValidationError> {
        self.validate()?;

        if is_reserved_label(&self.name) {
            return Err(CustomDefinitionValidationError::reserved_label(
                "custom house system",
                "name",
                &self.name,
            ));
        }

        for (index, alias) in self.aliases.iter().enumerate() {
            if is_reserved_label(alias) {
                return Err(CustomDefinitionValidationError::reserved_label(
                    "custom house system",
                    format!("alias[{index}]"),
                    alias,
                ));
            }
        }

        Ok(())
    }
}

impl fmt::Display for CustomHouseSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)?;

        if !self.aliases.is_empty() {
            write!(f, " [aliases: {}]", self.aliases.join(", "))?;
        }

        if let Some(notes) = &self.notes {
            write!(f, " ({notes})")?;
        }

        Ok(())
    }
}
