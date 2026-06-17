//! Live Horizons fetch seam. Compiled only under `feature = "horizons-fetch"`.
//!
//! The fetch produces raw bytes that re-enter the same offline parse/normalize
//! path; nothing here parses ephemeris data itself.

use super::error::IngestError;
use super::profile::ExpectedProfile;
use super::{read_public_corpus, PublicCorpus};

/// The wire format to request from Horizons.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HorizonsWireFormat {
    /// Plain vector-table text.
    Text,
    /// API JSON envelope.
    Json,
}

/// A Horizons ephemeris query.
#[derive(Clone, Debug, PartialEq)]
pub struct HorizonsQuery {
    /// Target body id (Horizons `COMMAND`).
    pub command: String,
    /// Center/origin (e.g. "500@0").
    pub center: String,
    /// Start Julian day.
    pub start: f64,
    /// Stop Julian day.
    pub stop: f64,
    /// Step size (e.g. "1d").
    pub step: String,
    /// Requested wire format.
    pub format: HorizonsWireFormat,
}

impl HorizonsQuery {
    /// Builds the Horizons API request URL for this query.
    pub fn to_url(&self) -> String {
        let fmt = match self.format {
            HorizonsWireFormat::Text => "text",
            HorizonsWireFormat::Json => "json",
        };
        format!(
            "https://ssd.jpl.nasa.gov/api/horizons.api?format={fmt}&EPHEM_TYPE=VECTORS&CSV_FORMAT=YES\
&COMMAND='{}'&CENTER='{}'&START_TIME='JD{}'&STOP_TIME='JD{}'&STEP_SIZE='{}'",
            self.command, self.center, self.start, self.stop, self.step
        )
    }
}

/// A source of raw Horizons product bytes. The real implementation hits the
/// network; tests inject saved fixtures.
pub trait HorizonsSource {
    /// Returns the raw bytes of a Horizons product for `query`.
    fn fetch(&self, query: &HorizonsQuery) -> Result<Vec<u8>, IngestError>;
}

/// Fetches and ingests a Horizons product through the offline path.
pub fn fetch_public_corpus<S: HorizonsSource>(
    source: &S,
    query: &HorizonsQuery,
    expected: &ExpectedProfile,
) -> Result<PublicCorpus, IngestError> {
    let bytes = source.fetch(query)?;
    let mut out = read_public_corpus(&bytes, expected)?;
    out.provenance.request_url = Some(query.to_url());
    Ok(out)
}

/// Live HTTP implementation of [`HorizonsSource`] over `ureq`.
pub struct HttpHorizonsSource;

impl HorizonsSource for HttpHorizonsSource {
    fn fetch(&self, query: &HorizonsQuery) -> Result<Vec<u8>, IngestError> {
        let url = query.to_url();
        let response = ureq::get(&url).call().map_err(|error| IngestError::Fetch {
            detail: error.to_string(),
        })?;
        let mut buf = Vec::new();
        response
            .into_reader()
            .read_to_end(&mut buf)
            .map_err(|error| IngestError::Fetch {
                detail: error.to_string(),
            })?;
        Ok(buf)
    }
}

use std::io::Read as _;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::{ExpectedProfile, Provenance};

    struct FakeSource(&'static str);

    impl HorizonsSource for FakeSource {
        fn fetch(&self, _query: &HorizonsQuery) -> Result<Vec<u8>, crate::ingest::IngestError> {
            Ok(self.0.as_bytes().to_vec())
        }
    }

    const VECTORS: &str = include_str!("../../tests/fixtures/ingest/horizons_vectors.txt");

    #[test]
    fn fetch_routes_bytes_through_offline_path() {
        let source = FakeSource(VECTORS);
        let query = HorizonsQuery {
            command: "499".to_string(),
            center: "500@0".to_string(),
            start: 2_451_545.0,
            stop: 2_451_546.0,
            step: "1d".to_string(),
            format: HorizonsWireFormat::Text,
        };
        let out = fetch_public_corpus(&source, &query, &ExpectedProfile::default()).unwrap();
        assert_eq!(out.corpus.entries.len(), 2);
        assert_eq!(out.provenance.frame, Provenance::Read);
        assert!(out.provenance.request_url.is_some());
    }
}
