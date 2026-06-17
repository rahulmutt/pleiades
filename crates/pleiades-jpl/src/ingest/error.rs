//! Structured, fail-closed ingestion errors.

use core::fmt;

use super::detect::InputFormat;

/// The semantic attribute an ingestion check is about.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Attribute {
    /// Coordinate frame.
    Frame,
    /// Time scale.
    TimeScale,
    /// Output units.
    Units,
    /// Center / origin.
    Center,
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Frame => "frame",
            Self::TimeScale => "time scale",
            Self::Units => "units",
            Self::Center => "center",
        };
        f.write_str(label)
    }
}

/// Everything that can go wrong while ingesting an external product.
#[derive(Clone, Debug, PartialEq)]
pub enum IngestError {
    /// No format matched during detection.
    UnrecognizedFormat {
        /// The format names the detector tried.
        looked_for: Vec<&'static str>,
    },
    /// A front-end could not tokenize a line.
    Malformed {
        /// Which format front-end was parsing.
        format: InputFormat,
        /// 1-based line number within the source.
        line: usize,
        /// Human-readable detail.
        detail: String,
    },
    /// A required structural marker was absent (e.g. `$$EOE`).
    MissingMarker {
        /// Which format front-end was parsing.
        format: InputFormat,
        /// The marker that was expected.
        marker: &'static str,
    },
    /// A generic-CSV column header could not be mapped to a known field.
    ColumnUnresolved {
        /// The raw column header.
        column: String,
        /// Which format front-end was parsing.
        format: InputFormat,
    },
    /// The source declared a value we do not support.
    Unsupported {
        /// The attribute.
        attribute: Attribute,
        /// The unsupported value.
        value: String,
    },
    /// Source and `ExpectedProfile` disagree on an attribute.
    Contradiction {
        /// The attribute.
        attribute: Attribute,
        /// What the source declared.
        declared: String,
        /// What the caller asserted.
        expected: String,
    },
    /// An attribute was silent in the source and absent from `ExpectedProfile`.
    Undetermined {
        /// The attribute.
        attribute: Attribute,
    },
    /// A row's body label did not map to a known body.
    UnknownBody {
        /// The raw body label.
        label: String,
    },
    /// A row carried a non-finite or otherwise invalid numeric value.
    MalformedRow {
        /// The epoch the row claimed.
        epoch_jd: f64,
        /// Human-readable detail.
        detail: String,
    },
    /// A live fetch failed (only constructed under `horizons-fetch`).
    Fetch {
        /// Human-readable detail.
        detail: String,
    },
}

impl fmt::Display for IngestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnrecognizedFormat { looked_for } => {
                write!(f, "unrecognized input format; looked for: {}", looked_for.join(", "))
            }
            Self::Malformed { format, line, detail } => {
                write!(f, "malformed {format} input at line {line}: {detail}")
            }
            Self::MissingMarker { format, marker } => {
                write!(f, "missing {marker} marker in {format} input")
            }
            Self::ColumnUnresolved { column, format } => {
                write!(f, "unresolved column {column:?} in {format} input")
            }
            Self::Unsupported { attribute, value } => {
                write!(f, "unsupported {attribute}: {value:?}")
            }
            Self::Contradiction { attribute, declared, expected } => write!(
                f,
                "contradiction on {attribute}: source declared {declared:?} but caller asserted {expected:?}"
            ),
            Self::Undetermined { attribute } => {
                write!(f, "undetermined {attribute}: not in source and not asserted")
            }
            Self::UnknownBody { label } => write!(f, "unknown body label: {label:?}"),
            Self::MalformedRow { epoch_jd, detail } => {
                write!(f, "malformed row at JD {epoch_jd}: {detail}")
            }
            Self::Fetch { detail } => write!(f, "fetch failed: {detail}"),
        }
    }
}

impl std::error::Error for IngestError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contradiction_names_both_sides() {
        let err = IngestError::Contradiction {
            attribute: Attribute::TimeScale,
            declared: "UTC".to_string(),
            expected: "TDB".to_string(),
        };
        let text = err.to_string();
        assert!(text.contains("time scale"), "got: {text}");
        assert!(text.contains("UTC") && text.contains("TDB"), "got: {text}");
    }

    #[test]
    fn unrecognized_lists_what_it_looked_for() {
        let err = IngestError::UnrecognizedFormat {
            looked_for: vec!["horizons-json", "vector-table", "generic-csv"],
        };
        assert!(err.to_string().contains("vector-table"));
    }
}
