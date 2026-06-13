use core::fmt;
use std::borrow::Cow;

/// Stable identifier for a backend implementation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct BackendId(String);

impl BackendId {
    /// Creates a new backend identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BackendId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The high-level backend family.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BackendFamily {
    /// A formula-based or algorithmic backend.
    Algorithmic,
    /// A backend backed primarily by reference data.
    ReferenceData,
    /// A backend backed by compressed packaged artifacts.
    CompressedData,
    /// A backend that routes across multiple providers.
    Composite,
    /// A future or project-specific family.
    Other(String),
}

/// A coarse posture label for how a backend family is typically categorized in release summaries.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BackendFamilyPosture {
    /// Formula-driven backends.
    Algorithmic,
    /// Reference-data and packaged-data backends.
    DataBacked,
    /// Multi-provider routing backends.
    Routing,
    /// Families that do not yet have a sharper public posture label.
    Other,
}

impl BackendFamilyPosture {
    /// Returns a stable human-readable label for the posture classification.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Algorithmic => "algorithmic",
            Self::DataBacked => "data-backed",
            Self::Routing => "routing",
            Self::Other => "other",
        }
    }
}

impl fmt::Display for BackendFamilyPosture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl BackendFamily {
    /// Returns a stable human-readable label for the backend family.
    pub fn display_name(&self) -> Cow<'_, str> {
        match self {
            Self::Algorithmic => Cow::Borrowed("Algorithmic"),
            Self::ReferenceData => Cow::Borrowed("ReferenceData"),
            Self::CompressedData => Cow::Borrowed("CompressedData"),
            Self::Composite => Cow::Borrowed("Composite"),
            Self::Other(value) => Cow::Owned(format!("Other({value})")),
        }
    }

    /// Returns `true` when the backend is driven primarily by external source data.
    pub const fn is_data_backed(&self) -> bool {
        matches!(self, Self::ReferenceData | Self::CompressedData)
    }

    /// Returns `true` when the backend is formula-driven rather than data-backed.
    pub const fn is_algorithmic(&self) -> bool {
        matches!(self, Self::Algorithmic)
    }

    /// Returns `true` when the backend routes across multiple providers.
    pub const fn is_routing(&self) -> bool {
        matches!(self, Self::Composite)
    }

    /// Returns the coarse posture classification used in compact release summaries.
    pub const fn posture(&self) -> BackendFamilyPosture {
        if self.is_algorithmic() {
            BackendFamilyPosture::Algorithmic
        } else if self.is_data_backed() {
            BackendFamilyPosture::DataBacked
        } else if self.is_routing() {
            BackendFamilyPosture::Routing
        } else {
            BackendFamilyPosture::Other
        }
    }

    /// Returns a short posture label for release-facing summaries.
    pub const fn posture_label(&self) -> &'static str {
        self.posture().label()
    }
}

impl fmt::Display for BackendFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.display_name())
    }
}

/// A rough accuracy class for a backend.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AccuracyClass {
    /// Exact or source-equivalent within the backend's documented model.
    Exact,
    /// High accuracy suitable for production use.
    High,
    /// Moderate accuracy.
    Moderate,
    /// Approximate or preliminary accuracy.
    Approximate,
    /// Accuracy class is unknown or not yet published.
    Unknown,
}

impl AccuracyClass {
    /// Returns a stable human-readable label for the accuracy class.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Exact => "Exact",
            Self::High => "High",
            Self::Moderate => "Moderate",
            Self::Approximate => "Approximate",
            Self::Unknown => "Unknown",
        }
    }
}

impl fmt::Display for AccuracyClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}
