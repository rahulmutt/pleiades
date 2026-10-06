use super::*;

#[test]
fn sidereal_position_gate_passes_within_ceilings() {
    let report = validate_sidereal_position_corpus().expect("sidereal-position gate passes");
    assert_eq!(report.rows_validated, 2680);
    eprintln!("{}", report.summary_line());
}

#[test]
fn tampered_corpus_fails_the_checksum() {
    let tampered = CORPUS_CSV.replacen("2415385.5,", "2415385.5, ", 1);
    assert!(matches!(
        validate(&tampered, MANIFEST),
        Err(SiderealPositionError::ChecksumMismatch { .. })
    ));
}

#[test]
fn manifest_row_count_drift_fails_closed() {
    let checksum = fnv1a64(CORPUS_CSV);
    let manifest = format!(
        "slice sidereal-position file=sidereal-position.csv role=sidereal-position rows=2679 checksum={checksum}"
    );
    assert!(matches!(
        validate(CORPUS_CSV, &manifest),
        Err(SiderealPositionError::ManifestDrift {
            rows_csv: 2680,
            rows_manifest: 2679
        })
    ));
}

#[test]
fn manifest_without_a_slice_line_is_rejected() {
    assert!(matches!(
        validate(CORPUS_CSV, "rows=2680"),
        Err(SiderealPositionError::MalformedManifest(_))
    ));
}

#[test]
fn malformed_rows_are_rejected() {
    for bad in [
        "2451545.0,Lahiri,Mars,1.0,0.0",
        "2451545.0,Lahiri,Mars,NaN,0.0,0.5",
        "2451545.0,Lahiri,Vulcan,1.0,0.0,0.5",
        "2451545.0,Nonesuch,Mars,1.0,0.0,0.5",
    ] {
        let manifest = format!("slice x rows=1 checksum={}", fnv1a64(bad));
        assert!(
            matches!(
                validate(bad, &manifest),
                Err(SiderealPositionError::MalformedRow(_))
            ),
            "{bad}"
        );
    }
}

/// The first corpus row of `body` (1901, Lahiri) with field `index` replaced
/// by `f(value)`, and a manifest for that one row.
fn shifted_row(body: &str, index: usize, f: impl Fn(f64) -> f64) -> (String, String) {
    let line = CORPUS_CSV
        .lines()
        .find(|l| l.starts_with("2415385.5,Lahiri,") && l.split(',').nth(2) == Some(body))
        .expect("a 1901 Lahiri row");
    let mut fields: Vec<String> = line.split(',').map(str::to_string).collect();
    let value: f64 = fields[index].parse().unwrap();
    fields[index] = format!("{:.12}", f(value));
    let csv = fields.join(",");
    let manifest = format!("slice x rows=1 checksum={}", fnv1a64(&csv));
    (csv, manifest)
}

fn exceeded_kind(result: Result<SiderealPositionReport, SiderealPositionError>) -> &'static str {
    match result {
        Err(SiderealPositionError::CeilingExceeded { kind, .. }) => kind,
        other => panic!("expected a ceiling to be exceeded, got {other:?}"),
    }
}

#[test]
fn an_unshifted_row_passes_alone() {
    let (csv, manifest) = shifted_row("Saturn", 3, |lon| lon);
    assert_eq!(validate(&csv, &manifest).unwrap().rows_validated, 1);
}

// Before issue #164 the chart took the ayanamsa off a J2000 longitude, which
// at 1901 is 4978″ of precession away from the reference. Moving the
// reference by that much puts today's chart the same distance from it.
#[test]
fn a_longitude_left_on_the_j2000_equinox_fails() {
    for body in ["Sun", "Moon", "Saturn"] {
        let (csv, manifest) = shifted_row(body, 3, |lon| lon - 4978.0 / 3600.0);
        assert_eq!(
            exceeded_kind(validate(&csv, &manifest)),
            "longitude_arcsec",
            "{body}"
        );
    }
}

#[test]
fn a_latitude_left_on_the_j2000_ecliptic_fails() {
    // The two ecliptics are tilted by 46″ at 1901.
    let (csv, manifest) = shifted_row("Saturn", 4, |lat| lat + 46.0 / 3600.0);
    assert_eq!(exceeded_kind(validate(&csv, &manifest)), "latitude_arcsec");
}

// A speed that misses the precession rate is off by 0.138″/day.
#[test]
fn a_speed_without_the_precession_rate_fails() {
    for body in ["Sun", "Saturn"] {
        let (csv, manifest) = shifted_row(body, 5, |speed| speed + 0.138 / 3600.0);
        assert_eq!(
            exceeded_kind(validate(&csv, &manifest)),
            "longitude_speed_arcsec_per_day",
            "{body}"
        );
    }
}
