//! Format detection for external ingestion inputs.

use core::fmt;

use super::error::IngestError;

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

/// Sniffs the input bytes to choose a front-end. Cheap and deterministic.
///
/// Order matters — first match wins.
pub fn detect_format(bytes: &[u8]) -> Result<InputFormat, IngestError> {
    let text = String::from_utf8_lossy(bytes);
    let trimmed = text.trim_start();

    // 1. Horizons API JSON: starts with `{` and carries the envelope keys.
    if trimmed.starts_with('{')
        && trimmed.contains("\"result\"")
        && trimmed.contains("\"signature\"")
    {
        return Ok(InputFormat::HorizonsApiJson);
    }

    // 2. Horizons vector table: bracketed by ephemeris markers.
    if text.contains("$$SOE") && text.contains("$$EOE") {
        return Ok(InputFormat::HorizonsVectorTable);
    }

    // 3. Generic CSV: a `#`-comment header block or a plain delimited row table.
    if text.lines().any(|l| l.trim_start().starts_with('#'))
        || text.lines().any(|l| l.contains(','))
    {
        return Ok(InputFormat::GenericCsv);
    }

    Err(IngestError::UnrecognizedFormat {
        looked_for: vec!["horizons-json", "vector-table", "generic-csv"],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_horizons_json_by_signature_keys() {
        let bytes = br#"{ "signature": { "version": "1.2" }, "result": "..." }"#;
        assert_eq!(detect_format(bytes).unwrap(), InputFormat::HorizonsApiJson);
    }

    #[test]
    fn detects_vector_table_by_soe_markers() {
        let bytes = b"Center body : SSB\n$$SOE\n2451545.0, A.D. ...\n$$EOE\n";
        assert_eq!(
            detect_format(bytes).unwrap(),
            InputFormat::HorizonsVectorTable
        );
    }

    #[test]
    fn detects_generic_csv_by_header_comment() {
        let bytes = b"# Columns: jd, body, x, y, z\n2451545.0, Mars, 1, 2, 3\n";
        assert_eq!(detect_format(bytes).unwrap(), InputFormat::GenericCsv);
    }

    #[test]
    fn rejects_unrecognized_input() {
        let err = detect_format(b"\x00\x01 not data").unwrap_err();
        assert!(matches!(
            err,
            crate::ingest::IngestError::UnrecognizedFormat { .. }
        ));
    }
}
