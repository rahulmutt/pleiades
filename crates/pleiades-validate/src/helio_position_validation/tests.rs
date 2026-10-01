use super::*;

#[test]
fn helio_position_gate_passes_within_ceilings() {
    let report = validate_helio_position_corpus().expect("helio-position gate passes");
    // Both window edges are inside the engine's window: every row validates.
    assert_eq!(report.rows_skipped_oor, 0);
    assert_eq!(report.rows_validated, 25_424);
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
    // 0.1° = 360″, a hundred times the longitude ceiling.
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

/// Asserts that `validate` rejects `(csv, manifest)` with a ceiling breach of
/// `kind` on `body`.
fn assert_ceiling_exceeded(csv: &str, manifest: &str, body: &str, kind: &str) {
    match validate(csv, manifest) {
        Err(HelioPositionError::CeilingExceeded {
            body: got_body,
            kind: got_kind,
            ..
        }) => assert_eq!((got_body, got_kind), (body, kind)),
        other => panic!("expected {body} {kind} ceiling breach, got {other:?}"),
    }
}

#[test]
fn a_shifted_reference_latitude_exceeds_the_ceiling() {
    // 0.01° = 36″ against a 0.36″ ceiling.
    let (csv, manifest) = shifted_row("Mars", 3, |v| v + 0.01);
    assert_ceiling_exceeded(&csv, &manifest, "Mars", "latitude_arcsec");
}

#[test]
fn a_shifted_reference_distance_exceeds_the_ceiling() {
    // 1e-3 relative against a 4.7e-6 ceiling.
    let (csv, manifest) = shifted_row("Mars", 4, |v| v * 1.001);
    assert_ceiling_exceeded(&csv, &manifest, "Mars", "distance_rel");
}

#[test]
fn a_shifted_reference_latitude_speed_exceeds_the_ceiling() {
    // 0.001 deg/day = 3.6″/day against a 0.14″/day ceiling.
    let (csv, manifest) = shifted_row("Mars", 6, |v| v + 0.001);
    assert_ceiling_exceeded(&csv, &manifest, "Mars", "latitude_speed_arcsec_per_day");
}

#[test]
fn a_shifted_reference_distance_speed_exceeds_the_ceiling() {
    // 1e-5 AU/day against a 2.8e-7 AU/day ceiling.
    let (csv, manifest) = shifted_row("Mars", 7, |v| v + 1e-5);
    assert_ceiling_exceeded(&csv, &manifest, "Mars", "distance_speed_au_per_day");
}

#[test]
fn a_shifted_pluto_longitude_exceeds_the_pluto_ceiling() {
    // 0.01° = 36″ against Pluto's 1.8″ ceiling.
    let (csv, manifest) = shifted_row("Pluto", 2, |v| (v + 0.01).rem_euclid(360.0));
    assert_ceiling_exceeded(&csv, &manifest, "Pluto", "longitude_arcsec");
}

#[test]
fn a_corpus_with_every_row_out_of_window_validates_too_few_rows() {
    // `validate` caps the floor at the manifest row count, so a one-row
    // corpus whose only row is out of the engine's window (1800-01-01) is
    // the reachable way to validate fewer rows than the floor.
    let csv = "2378496.5,Mars,1.0,0.0,1.5,0.5,0.0,0.0";
    let manifest = format!("slice x rows=1 checksum={}", fnv1a64(csv));
    assert!(matches!(
        validate(csv, &manifest),
        Err(HelioPositionError::TooFewRowsValidated {
            validated: 0,
            floor: 1
        })
    ));
}

/// 0.137″/day, the general-precession rate, in deg/day.
const PRECESSION_RATE_DEG_PER_DAY: f64 = 3.8056e-5;

#[test]
fn outer_speeds_shifted_by_the_precession_rate_fail_closed() {
    // Real Jupiter–Neptune rows near J2000, each longitude speed shifted by
    // the precession rate. That shift exceeds the per-row longitude-speed
    // ceiling (0.12″/day), so the per-row check fires before the mean check;
    // `speed_convention_check_rejects_a_precession_offset` covers the mean.
    let rows: Vec<String> = ["Jupiter", "Saturn", "Uranus", "Neptune"]
        .iter()
        .map(|body| shifted_row(body, 5, |v| v + PRECESSION_RATE_DEG_PER_DAY).0)
        .collect();
    let csv = rows.join("\n");
    let manifest = format!("slice x rows=4 checksum={}", fnv1a64(&csv));
    assert_ceiling_exceeded(&csv, &manifest, "Jupiter", "longitude_speed_arcsec_per_day");
}

#[test]
fn speed_convention_check_rejects_a_precession_offset() {
    let bound = OUTER_LON_SPEED_MEAN_SIGNED_BOUND_ARCSEC_PER_DAY;
    let precession_arcsec_per_day = PRECESSION_RATE_DEG_PER_DAY * 3600.0;
    for mean in [
        precession_arcsec_per_day,
        -precession_arcsec_per_day,
        bound,
        -bound,
        f64::NAN,
    ] {
        assert!(
            matches!(
                check_speed_convention(mean, bound),
                Err(HelioPositionError::SpeedConventionMismatch { .. })
            ),
            "{mean}"
        );
    }
    for mean in [0.0, 0.000_232, -0.019] {
        assert!(check_speed_convention(mean, bound).is_ok(), "{mean}");
    }
}
