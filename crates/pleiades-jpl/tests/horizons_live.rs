//! Opt-in live Horizons fetch test. Skipped unless both the `horizons-fetch`
//! feature is enabled and `PLEIADES_HORIZONS_LIVE=1` is set (mirrors the
//! kernel-gated `corpus_regen` pattern). Never runs in default CI.

#![cfg(feature = "horizons-fetch")]

use pleiades_jpl::ingest::{
    fetch_public_corpus, ExpectedProfile, HorizonsQuery, HorizonsWireFormat, HttpHorizonsSource,
};

#[test]
fn live_fetch_mars_vectors() {
    if std::env::var("PLEIADES_HORIZONS_LIVE").ok().as_deref() != Some("1") {
        eprintln!("skipping live Horizons test (set PLEIADES_HORIZONS_LIVE=1 to run)");
        return;
    }
    let query = HorizonsQuery {
        command: "499".to_string(),
        center: "500@0".to_string(),
        start: 2_451_545.0,
        stop: 2_451_546.0,
        step: "1d".to_string(),
        format: HorizonsWireFormat::Text,
    };
    let out = fetch_public_corpus(&HttpHorizonsSource, &query, &ExpectedProfile::default())
        .expect("live fetch should succeed");
    assert!(!out.corpus.entries.is_empty());
}
