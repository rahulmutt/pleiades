use pleiades_jpl::ingest::{
    read_public_corpus, read_public_corpus_as, ExpectedProfile, InputFormat, Provenance,
};

const VECTORS: &str = include_str!("fixtures/ingest/horizons_vectors.txt");
const JSON: &str = include_str!("fixtures/ingest/horizons_api.json");
const CSV: &str = include_str!("fixtures/ingest/generic.csv");

#[test]
fn reads_vector_table_end_to_end() {
    let out = read_public_corpus(VECTORS.as_bytes(), &ExpectedProfile::default()).unwrap();
    assert_eq!(out.corpus.entries.len(), 2);
    assert_eq!(
        out.corpus.entries[0].body,
        pleiades_backend::CelestialBody::Mars
    );
    assert_eq!(out.provenance.frame, Provenance::Read);
}

#[test]
fn reads_horizons_json_end_to_end() {
    let out = read_public_corpus(JSON.as_bytes(), &ExpectedProfile::default()).unwrap();
    assert_eq!(out.corpus.entries.len(), 1);
    assert!(out.provenance.source_label.unwrap().contains("API"));
}

#[test]
fn reads_generic_csv_end_to_end() {
    let out = read_public_corpus(CSV.as_bytes(), &ExpectedProfile::default()).unwrap();
    assert_eq!(out.corpus.entries.len(), 2);
}

#[test]
fn explicit_format_bypasses_detection() {
    let out = read_public_corpus_as(
        VECTORS.as_bytes(),
        InputFormat::HorizonsVectorTable,
        &ExpectedProfile::default(),
    )
    .unwrap();
    assert_eq!(out.corpus.entries.len(), 2);
}
