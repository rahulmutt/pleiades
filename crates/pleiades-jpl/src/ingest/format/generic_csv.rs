//! Tolerant generic JPL-style CSV front-end with column aliasing.

use crate::ingest::detect::InputFormat;
use crate::ingest::error::IngestError;
use crate::ingest::ir::{RawCorpus, RawEphemerisRecord, RawManifest};

const FORMAT: InputFormat = InputFormat::GenericCsv;

#[derive(Clone, Copy, PartialEq)]
enum Column {
    Body,
    Jd,
    X,
    Y,
    Z,
}

fn alias(header: &str) -> Option<Column> {
    match header.trim().to_ascii_lowercase().as_str() {
        "body" | "target" | "name" => Some(Column::Body),
        "jd" | "jdtdb" | "epoch" | "julian_day" => Some(Column::Jd),
        "x" | "px" | "x_km" => Some(Column::X),
        "y" | "py" | "y_km" => Some(Column::Y),
        "z" | "pz" | "z_km" => Some(Column::Z),
        _ => None,
    }
}

/// Parses a generic JPL-style CSV into the neutral IR.
pub fn parse(source: &str) -> Result<RawCorpus, IngestError> {
    let declared = parse_header(source);

    let mut data_lines = source
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty() && !l.trim_start().starts_with('#'));

    let (header_no, header_line) = data_lines.next().ok_or(IngestError::MissingMarker {
        format: FORMAT,
        marker: "header row",
    })?;

    let mut mapping = Vec::new();
    for raw in header_line.split(',') {
        let col = alias(raw).ok_or_else(|| IngestError::ColumnUnresolved {
            column: raw.trim().to_string(),
            format: FORMAT,
        })?;
        mapping.push(col);
    }
    let _ = header_no;

    let mut records = Vec::new();
    for (line_no, line) in data_lines {
        let fields: Vec<&str> = line.split(',').map(str::trim).collect();
        if fields.len() != mapping.len() {
            return Err(IngestError::Malformed {
                format: FORMAT,
                line: line_no + 1,
                detail: format!("expected {} columns, found {}", mapping.len(), fields.len()),
            });
        }
        let mut body = None;
        let mut jd = None;
        let mut pos = [0.0_f64; 3];
        for (col, value) in mapping.iter().zip(&fields) {
            match col {
                Column::Body => body = Some((*value).to_string()),
                Column::Jd => jd = Some(parse_num(value, line_no + 1)?),
                Column::X => pos[0] = parse_num(value, line_no + 1)?,
                Column::Y => pos[1] = parse_num(value, line_no + 1)?,
                Column::Z => pos[2] = parse_num(value, line_no + 1)?,
            }
        }
        records.push(RawEphemerisRecord {
            body_label: body.ok_or(IngestError::Malformed {
                format: FORMAT,
                line: line_no + 1,
                detail: "missing body column".to_string(),
            })?,
            epoch_jd: jd.ok_or(IngestError::Malformed {
                format: FORMAT,
                line: line_no + 1,
                detail: "missing JD column".to_string(),
            })?,
            pos,
            vel: None,
        });
    }
    Ok(RawCorpus { declared, records })
}

fn parse_num(value: &str, line: usize) -> Result<f64, IngestError> {
    value.parse::<f64>().map_err(|_| IngestError::Malformed {
        format: FORMAT,
        line,
        detail: format!("non-numeric field {value:?}"),
    })
}

fn comment_value(source: &str, key: &str) -> Option<String> {
    source.lines().find_map(|line| {
        let line = line.trim();
        let body = line.strip_prefix('#')?.trim();
        let rest = body.strip_prefix(key)?.trim_start();
        let rest = rest.strip_prefix(':')?.trim();
        if rest.is_empty() {
            None
        } else {
            Some(rest.to_string())
        }
    })
}

fn parse_header(source: &str) -> RawManifest {
    RawManifest {
        source_label: comment_value(source, "Source"),
        center: comment_value(source, "Center"),
        frame: comment_value(source, "Reference frame"),
        time_scale: comment_value(source, "Time scale"),
        units: comment_value(source, "Output units"),
        columns: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = include_str!("../../../tests/fixtures/ingest/generic.csv");

    #[test]
    fn parses_aliased_reordered_columns() {
        let raw = parse(SAMPLE).unwrap();
        assert_eq!(raw.records.len(), 2);
        assert_eq!(raw.records[0].body_label, "Mars");
        assert_eq!(raw.records[0].epoch_jd, 2_451_545.0);
        assert_eq!(raw.records[0].pos, [1.0, 2.0, 3.0]);
        assert_eq!(raw.records[1].body_label, "Venus");
    }

    #[test]
    fn reads_header_comment_metadata() {
        let raw = parse(SAMPLE).unwrap();
        assert_eq!(raw.declared.time_scale.as_deref(), Some("TDB"));
        assert_eq!(raw.declared.units.as_deref(), Some("KM"));
        assert_eq!(raw.declared.center.as_deref(), Some("500@0"));
    }

    #[test]
    fn unmappable_column_errors() {
        let bad = "body,JD,PX,PY,WAT\nMars,1.0,1,2,3\n";
        let err = parse(bad).unwrap_err();
        assert!(matches!(
            err,
            crate::ingest::IngestError::ColumnUnresolved { .. }
        ));
    }
}
