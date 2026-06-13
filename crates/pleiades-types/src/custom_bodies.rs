//! Custom body identifiers: [`CustomBodyId`] and [`CustomDefinitionValidationError`].

use core::fmt;

/// A structured identifier for a custom body.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    /// Validates the custom body identifier fields.
    ///
    /// The catalog and designation must both be non-empty, must not contain
    /// leading or trailing whitespace, and must not contain the `:` separator
    /// used by the display representation.
    pub fn validate(&self) -> Result<(), CustomDefinitionValidationError> {
        validate_canonical_text("custom body id", "catalog", &self.catalog)?;
        validate_canonical_text("custom body id", "designation", &self.designation)?;

        if self.catalog.contains(':') {
            return Err(CustomDefinitionValidationError::contains_separator(
                "custom body id",
                "catalog",
                ':',
            ));
        }

        if self.designation.contains(':') {
            return Err(CustomDefinitionValidationError::contains_separator(
                "custom body id",
                "designation",
                ':',
            ));
        }

        Ok(())
    }
}

impl fmt::Display for CustomBodyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.catalog, self.designation)
    }
}

/// Validation failure for a custom body, house system, or ayanamsa definition.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CustomDefinitionValidationError {
    /// A required field was blank.
    Blank {
        /// The type of custom definition being validated.
        subject: &'static str,
        /// The invalid field name.
        field: String,
    },
    /// A field carried leading or trailing whitespace.
    Padded {
        /// The type of custom definition being validated.
        subject: &'static str,
        /// The invalid field name.
        field: String,
    },
    /// A field contained the `:` separator used by custom body identifiers.
    ContainsSeparator {
        /// The type of custom definition being validated.
        subject: &'static str,
        /// The invalid field name.
        field: String,
        /// The separator that was rejected.
        separator: char,
    },
    /// A list of aliases contained a duplicate value.
    DuplicateAlias {
        /// The type of custom definition being validated.
        subject: &'static str,
        /// The duplicated alias.
        alias: String,
    },
    /// A label collides with a built-in descriptor or alias.
    ReservedLabel {
        /// The type of custom definition being validated.
        subject: &'static str,
        /// The invalid field name.
        field: String,
        /// The reserved built-in label.
        label: String,
    },
    /// A numeric field was not finite.
    NonFinite {
        /// The type of custom definition being validated.
        subject: &'static str,
        /// The invalid field name.
        field: String,
    },
    /// Two optional fields must be supplied together.
    IncompletePair {
        /// The type of custom definition being validated.
        subject: &'static str,
        /// The first required field.
        first: String,
        /// The second required field.
        second: String,
    },
}

impl CustomDefinitionValidationError {
    pub(crate) fn blank(subject: &'static str, field: impl Into<String>) -> Self {
        Self::Blank {
            subject,
            field: field.into(),
        }
    }

    pub(crate) fn padded(subject: &'static str, field: impl Into<String>) -> Self {
        Self::Padded {
            subject,
            field: field.into(),
        }
    }

    pub(crate) fn contains_separator(
        subject: &'static str,
        field: impl Into<String>,
        separator: char,
    ) -> Self {
        Self::ContainsSeparator {
            subject,
            field: field.into(),
            separator,
        }
    }

    pub(crate) fn duplicate_alias(subject: &'static str, alias: impl Into<String>) -> Self {
        Self::DuplicateAlias {
            subject,
            alias: alias.into(),
        }
    }

    pub(crate) fn reserved_label(
        subject: &'static str,
        field: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self::ReservedLabel {
            subject,
            field: field.into(),
            label: label.into(),
        }
    }

    pub(crate) fn non_finite(subject: &'static str, field: impl Into<String>) -> Self {
        Self::NonFinite {
            subject,
            field: field.into(),
        }
    }

    pub(crate) fn incomplete_pair(
        subject: &'static str,
        first: impl Into<String>,
        second: impl Into<String>,
    ) -> Self {
        Self::IncompletePair {
            subject,
            first: first.into(),
            second: second.into(),
        }
    }

    /// Returns a compact one-line rendering of the validation failure.
    pub fn summary_line(&self) -> String {
        match self {
            Self::Blank { subject, field } => format!("{subject} {field} must not be blank"),
            Self::Padded { subject, field } => {
                format!("{subject} {field} must not have leading or trailing whitespace")
            }
            Self::ContainsSeparator {
                subject,
                field,
                separator,
            } => format!("{subject} {field} must not contain '{separator}'"),
            Self::DuplicateAlias { subject, alias } => {
                format!("{subject} aliases must be unique: duplicate {alias}")
            }
            Self::ReservedLabel {
                subject,
                field,
                label,
            } => {
                format!("{subject} {field} must not match a built-in label: {label}")
            }
            Self::NonFinite { subject, field } => format!("{subject} {field} must be finite"),
            Self::IncompletePair {
                subject,
                first,
                second,
            } => format!("{subject} requires both {first} and {second} when one is present"),
        }
    }
}

impl fmt::Display for CustomDefinitionValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for CustomDefinitionValidationError {}

pub(crate) fn validate_canonical_text(
    subject: &'static str,
    field: impl Into<String>,
    value: &str,
) -> Result<(), CustomDefinitionValidationError> {
    let field = field.into();
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err(CustomDefinitionValidationError::blank(subject, field));
    }

    if trimmed != value {
        return Err(CustomDefinitionValidationError::padded(subject, field));
    }

    Ok(())
}
