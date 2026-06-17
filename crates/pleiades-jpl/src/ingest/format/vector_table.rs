//! Horizons CSV-format VECTORS text front-end.

use crate::ingest::detect::InputFormat;
use crate::ingest::error::IngestError;
use crate::ingest::ir::{RawCorpus, RawEphemerisRecord, RawManifest};

const FORMAT: InputFormat = InputFormat::HorizonsVectorTable;

/// Parses a Horizons CSV-format vector text block into the neutral IR.
pub fn parse(source: &str) -> Result<RawCorpus, IngestError> {
    let declared = parse_header(source);
    let body_label = declared_target(source);
    let records = parse_rows(source, &body_label)?;
    Ok(RawCorpus { declared, records })
}

fn header_value<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    source.lines().find_map(|line| {
        let line = line.trim();
        let rest = line.strip_prefix(key)?;
        let rest = rest.trim_start();
        let rest = rest.strip_prefix(':')?.trim();
        // Drop trailing "{...}" provenance braces Horizons appends.
        let rest = rest.split('{').next().unwrap_or(rest).trim();
        if rest.is_empty() {
            None
        } else {
            Some(rest)
        }
    })
}

fn parse_header(source: &str) -> RawManifest {
    // Output units line carries both distance and time, e.g. "KM-S"; the time
    // scale is implied by the JDTDB column label, so read it from the header.
    let time_scale = if source.contains("JDTDB") || source.contains("(TDB)") {
        Some("TDB".to_string())
    } else if source.contains("JDTT") || source.contains("(TT)") {
        Some("TT".to_string())
    } else {
        None
    };
    RawManifest {
        source_label: Some("JPL Horizons".to_string()),
        center: header_value(source, "Center body name").map(str::to_string),
        frame: header_value(source, "Reference frame").map(str::to_string),
        time_scale,
        units: header_value(source, "Output units").map(str::to_string),
        columns: vec![],
    }
}

fn declared_target(source: &str) -> String {
    header_value(source, "Target body name")
        .unwrap_or("unknown")
        .to_string()
}

fn parse_rows(source: &str, body_label: &str) -> Result<Vec<RawEphemerisRecord>, IngestError> {
    let start = source.find("$$SOE").ok_or(IngestError::MissingMarker {
        format: FORMAT,
        marker: "$$SOE",
    })?;
    let after_start = &source[start + "$$SOE".len()..];
    let end = after_start
        .find("$$EOE")
        .ok_or(IngestError::MissingMarker {
            format: FORMAT,
            marker: "$$EOE",
        })?;
    let block = &after_start[..end];

    let mut records = Vec::new();
    for (offset, raw_line) in block.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        // Expect: JD, CalendarDate, X, Y, Z[, VX, VY, VZ]
        if fields.len() < 5 {
            return Err(IngestError::Malformed {
                format: FORMAT,
                line: offset + 1,
                detail: format!("expected at least 5 columns, found {}", fields.len()),
            });
        }
        let parse_num = |idx: usize| -> Result<f64, IngestError> {
            fields[idx]
                .parse::<f64>()
                .map_err(|_| IngestError::Malformed {
                    format: FORMAT,
                    line: offset + 1,
                    detail: format!("non-numeric field {:?}", fields[idx]),
                })
        };
        records.push(RawEphemerisRecord {
            body_label: body_label.to_string(),
            epoch_jd: parse_num(0)?,
            pos: [parse_num(2)?, parse_num(3)?, parse_num(4)?],
            vel: None,
        });
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = include_str!("../../../tests/fixtures/ingest/horizons_vectors.txt");

    #[test]
    fn parses_header_metadata() {
        let raw = parse(SAMPLE).unwrap();
        assert_eq!(
            raw.declared.center.as_deref(),
            Some("Solar System Barycenter (0)")
        );
        assert_eq!(raw.declared.frame.as_deref(), Some("Ecliptic of J2000.0"));
        assert_eq!(raw.declared.time_scale.as_deref(), Some("TDB"));
        assert_eq!(raw.declared.units.as_deref(), Some("KM-S"));
    }

    #[test]
    fn parses_rows_between_markers() {
        let raw = parse(SAMPLE).unwrap();
        assert_eq!(raw.records.len(), 2);
        assert_eq!(raw.records[0].body_label, "Mars (499)");
        assert_eq!(raw.records[0].epoch_jd, 2_451_545.0);
        assert_eq!(raw.records[0].pos, [1.0, 2.0, 3.0]);
        assert_eq!(raw.records[1].pos, [4.0, 5.0, 6.0]);
    }

    #[test]
    fn missing_eoe_is_an_error() {
        let truncated = SAMPLE.replace("$$EOE", "");
        let err = parse(&truncated).unwrap_err();
        assert!(matches!(
            err,
            crate::ingest::IngestError::MissingMarker {
                marker: "$$EOE",
                ..
            }
        ));
    }
}
