//! Kernel-gated: regenerates the holdout slice and verifies it is byte-identical
//! to the checked-in `holdout.csv`, which must carry 8-column rows (position +
//! velocity). Skipped unless `PLEIADES_DE_KERNEL` points at de440.bsp.

/// Maintainer helper: regenerates `holdout.csv` into `data/corpus/holdout.csv`
/// from de440 and prints the FNV-1a checksum for the manifest.
/// Run: `PLEIADES_DE_KERNEL=... cargo test -p pleiades-jpl -- --ignored print_holdout_checksum`
#[test]
#[ignore]
fn print_holdout_checksum() {
    let kernel = std::env::var("PLEIADES_DE_KERNEL").expect("set PLEIADES_DE_KERNEL");
    let csv = pleiades_jpl::regenerate_holdout_slice_csv(&kernel).unwrap();
    use pleiades_jpl::spk::corpus_manifest::corpus_checksum64;
    let checksum = corpus_checksum64(&csv);
    let rows = csv
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .count();
    println!("rows={rows} checksum={checksum}");
    // Write to the committed file path (cargo test runs from crate root).
    std::fs::write("data/corpus/holdout.csv", &csv).expect("writing holdout.csv");
    println!("Written to data/corpus/holdout.csv");
}

#[test]
fn holdout_slice_regenerates_with_velocity() {
    let Some(kernel) = std::env::var_os("PLEIADES_DE_KERNEL") else {
        eprintln!("skipping: PLEIADES_DE_KERNEL not set");
        return;
    };
    let regenerated = pleiades_jpl::regenerate_holdout_slice_csv(&kernel).unwrap();
    // Committed file must be byte-identical to a fresh de440 regen.
    let committed = include_str!("../data/corpus/holdout.csv");
    assert_eq!(
        regenerated, committed,
        "committed holdout.csv drifted from de440 regen"
    );
    // Velocity columns must be present in the header and data rows.
    assert!(
        regenerated.contains("vx_km_s"),
        "holdout.csv must declare velocity column vx_km_s"
    );
    // Every data row must have 8 comma-separated fields.
    let bad_rows: Vec<&str> = regenerated
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .filter(|l| l.split(',').count() != 8)
        .collect();
    assert!(
        bad_rows.is_empty(),
        "expected 8 columns in every data row; bad rows: {:?}",
        &bad_rows[..bad_rows.len().min(3)]
    );
}
