//! Horizons API JSON front-end. Extracts the `result` text payload and the
//! signature version, then delegates row parsing to the vector-table reader.

use crate::ingest::detect::InputFormat;
use crate::ingest::error::IngestError;
use crate::ingest::ir::RawCorpus;

use super::vector_table;

const FORMAT: InputFormat = InputFormat::HorizonsApiJson;

/// Parses a Horizons API JSON envelope into the neutral IR.
pub fn parse(source: &str) -> Result<RawCorpus, IngestError> {
    let result = json_string_field(source, "result").ok_or(IngestError::Malformed {
        format: FORMAT,
        line: 0,
        detail: "missing \"result\" string field".to_string(),
    })?;
    let version = json_string_field(source, "version");

    let mut corpus = vector_table::parse(&result)?;
    let label = match version {
        Some(v) => format!("JPL Horizons API v{v}"),
        None => "JPL Horizons API".to_string(),
    };
    corpus.declared.source_label = Some(label);
    Ok(corpus)
}

/// Extracts a JSON string field value by key, handling `\n`, `\t`, `\"`, `\\`,
/// and `\/` escapes. Minimal by design — Horizons envelopes are flat and the
/// fields we read are plain strings.
fn json_string_field(source: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let key_pos = source.find(&needle)?;
    let after_key = &source[key_pos + needle.len()..];
    let colon = after_key.find(':')?;
    let rest = after_key[colon + 1..].trim_start();
    let mut chars = rest.char_indices();
    // Expect an opening quote.
    if chars.next()?.1 != '"' {
        return None;
    }
    let mut out = String::new();
    let mut escaped = false;
    for (_, c) in chars {
        if escaped {
            out.push(match c {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '"' => '"',
                '\\' => '\\',
                '/' => '/',
                other => other,
            });
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            return Some(out);
        } else {
            out.push(c);
        }
    }
    None // unterminated string
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = include_str!("../../../tests/fixtures/ingest/horizons_api.json");

    #[test]
    fn extracts_result_block_and_delegates_rows() {
        let raw = parse(SAMPLE).unwrap();
        assert_eq!(raw.records.len(), 1);
        assert_eq!(raw.records[0].body_label, "Mars (499)");
        assert_eq!(raw.records[0].pos, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn carries_version_into_source_label() {
        let raw = parse(SAMPLE).unwrap();
        let label = raw.declared.source_label.unwrap();
        assert!(label.contains("1.2"), "got: {label}");
    }

    #[test]
    fn missing_result_field_errors() {
        let err = parse(r#"{ "signature": { "version": "1.2" } }"#).unwrap_err();
        assert!(matches!(err, crate::ingest::IngestError::Malformed { .. }));
    }
}
