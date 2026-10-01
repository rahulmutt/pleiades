use super::*;

#[test]
fn helio_position_gate_passes_within_ceilings() {
    let report = validate_helio_position_corpus().expect("helio-position gate passes");
    assert!(report.rows_validated >= MIN_ROWS_VALIDATED);
    eprintln!("{}", report.summary_line());
}

#[test]
fn tampered_corpus_fails_the_checksum() {
    let tampered = CORPUS_CSV.replacen("2415020.5,", "2415020.5, ", 1);
    assert!(matches!(
        validate(&tampered, MANIFEST),
        Err(HelioPositionError::ChecksumMismatch { .. })
    ));
}

#[test]
fn manifest_row_count_drift_fails_closed() {
    let checksum = fnv1a64(CORPUS_CSV);
    let manifest = format!(
        "slice helio-position file=helio-position.csv role=helio-position rows=25423 checksum={checksum}"
    );
    assert!(matches!(
        validate(CORPUS_CSV, &manifest),
        Err(HelioPositionError::ManifestDrift {
            rows_csv: 25424,
            rows_manifest: 25423
        })
    ));
}

#[test]
fn manifest_without_a_slice_line_is_rejected() {
    assert!(matches!(
        validate(CORPUS_CSV, "rows=25424"),
        Err(HelioPositionError::MalformedManifest(_))
    ));
}

#[test]
fn malformed_rows_are_rejected() {
    for bad in [
        "2451545.0,Mars,1.0,0.0",
        "2451545.0,Mars,NaN,0.0,1.5,0.5,0.0,0.0",
        "2451545.0,Vulcan,1.0,0.0,1.5,0.5,0.0,0.0",
        "2451545.0,Mars,1.0,0.0,0.0,0.5,0.0,0.0",
    ] {
        let manifest = format!("slice x rows=1 checksum={}", fnv1a64(bad));
        assert!(
            matches!(
                validate(bad, &manifest),
                Err(HelioPositionError::MalformedRow(_))
            ),
            "{bad}"
        );
    }
}

/// One real corpus row with field `index` replaced by `f(value)`.
fn shifted_row(body: &str, index: usize, f: impl Fn(f64) -> f64) -> (String, String) {
    let line = CORPUS_CSV
        .lines()
        .find(|l| l.starts_with("245") && l.split(',').nth(1) == Some(body))
        .expect("a data row near J2000");
    let mut fields: Vec<String> = line.split(',').map(str::to_string).collect();
    let value: f64 = fields[index].parse().unwrap();
    fields[index] = format!("{:.12}", f(value));
    let csv = fields.join(",");
    let manifest = format!("slice x rows=1 checksum={}", fnv1a64(&csv));
    (csv, manifest)
}

#[test]
fn a_shifted_reference_longitude_exceeds_the_ceiling() {
    // 0.1° = 360″, well above the light-time floor.
    let (csv, manifest) = shifted_row("Mars", 2, |v| (v + 0.1).rem_euclid(360.0));
    assert!(matches!(
        validate(&csv, &manifest),
        Err(HelioPositionError::CeilingExceeded {
            body: "Mars",
            kind: "longitude_arcsec",
            ..
        })
    ));
}

#[test]
fn a_shifted_reference_speed_exceeds_the_ceiling() {
    // 0.01 deg/day = 36″/day.
    let (csv, manifest) = shifted_row("Mars", 5, |v| v + 0.01);
    assert!(matches!(
        validate(&csv, &manifest),
        Err(HelioPositionError::CeilingExceeded {
            body: "Mars",
            kind: "longitude_speed_arcsec_per_day",
            ..
        })
    ));
}
