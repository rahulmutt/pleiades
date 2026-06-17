//! Format detection for external ingestion inputs.

use core::fmt;

/// The external input shapes the reader understands.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputFormat {
    /// Horizons `$$SOE`/`$$EOE` CSV-vector text.
    HorizonsVectorTable,
    /// Horizons API JSON envelope.
    HorizonsApiJson,
    /// Generic JPL-style CSV.
    GenericCsv,
}

impl fmt::Display for InputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::HorizonsVectorTable => "vector-table",
            Self::HorizonsApiJson => "horizons-json",
            Self::GenericCsv => "generic-csv",
        };
        f.write_str(label)
    }
}
