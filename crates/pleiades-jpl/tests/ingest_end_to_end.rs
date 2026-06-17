use pleiades_jpl::ingest::{
    read_public_corpus, read_public_corpus_as, ExpectedProfile, InputFormat, Provenance,
};

const VECTORS: &str = include_str!("fixtures/ingest/horizons_vectors.txt");
const JSON: &str = include_str!("fixtures/ingest/horizons_api.json");
const CSV: &str = include_str!("fixtures/ingest/generic.csv");
const ANCHOR: &str = include_str!("fixtures/ingest/anchor_mars.txt");

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

#[test]
fn anchor_matches_committed_corpus_within_km_tolerance() {
    let out = read_public_corpus(ANCHOR.as_bytes(), &ExpectedProfile::default()).unwrap();
    let e = &out.corpus.entries[0];
    // Committed corpus row: interior.csv, Mars at epoch_jd=2305447.5.
    let expected_x_km = -89_078_264.814_350_f64;
    let expected_y_km = 66_972_394.971_609_f64;
    let expected_z_km = 7_422_033.407_297_f64;
    assert!(
        (e.x_km - expected_x_km).abs() <= 1.0,
        "x dx={}",
        e.x_km - expected_x_km
    );
    assert!(
        (e.y_km - expected_y_km).abs() <= 1.0,
        "y dy={}",
        e.y_km - expected_y_km
    );
    assert!(
        (e.z_km - expected_z_km).abs() <= 1.0,
        "z dz={}",
        e.z_km - expected_z_km
    );
}
