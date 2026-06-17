//! Format-neutral intermediate representation for external ingestion.
//!
//! Front-ends tokenize bytes into these types verbatim — every interpreted
//! attribute is an `Option`/`String` here. No defaulting, conversion, or
//! semantic mapping happens at this layer; that is the normalizer's job.

/// A parsed-but-uninterpreted external corpus.
#[derive(Clone, Debug, PartialEq)]
pub struct RawCorpus {
    /// Header metadata lifted verbatim from the source.
    pub declared: RawManifest,
    /// Row data lifted verbatim from the source.
    pub records: Vec<RawEphemerisRecord>,
}

/// Header strings exactly as the source declared them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RawManifest {
    /// Free-form source label (e.g. "JPL Horizons API v1.2").
    pub source_label: Option<String>,
    /// Declared center/origin (e.g. "500@0").
    pub center: Option<String>,
    /// Declared reference frame string.
    pub frame: Option<String>,
    /// Declared time-scale string.
    pub time_scale: Option<String>,
    /// Declared output-units string.
    pub units: Option<String>,
    /// Raw column order as seen.
    pub columns: Vec<String>,
}

/// One uninterpreted ephemeris row.
#[derive(Clone, Debug, PartialEq)]
pub struct RawEphemerisRecord {
    /// Raw target name/id as seen.
    pub body_label: String,
    /// Numeric Julian day as seen (the scale lives in `RawManifest::time_scale`).
    pub epoch_jd: f64,
    /// Raw x,y,z in source units (no conversion yet).
    pub pos: [f64; 3],
    /// Velocity captured if present; dropped during normalization.
    pub vel: Option<[f64; 3]>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_corpus_holds_declared_and_records() {
        let raw = RawCorpus {
            declared: RawManifest {
                source_label: Some("JPL Horizons".to_string()),
                center: Some("500@0".to_string()),
                frame: Some("Ecliptic of J2000.0".to_string()),
                time_scale: Some("TDB".to_string()),
                units: Some("KM-S".to_string()),
                columns: vec!["jd".to_string(), "x".to_string()],
            },
            records: vec![RawEphemerisRecord {
                body_label: "Mars".to_string(),
                epoch_jd: 2_451_545.0,
                pos: [1.0, 2.0, 3.0],
                vel: None,
            }],
        };
        assert_eq!(raw.records.len(), 1);
        assert_eq!(raw.declared.center.as_deref(), Some("500@0"));
    }
}
