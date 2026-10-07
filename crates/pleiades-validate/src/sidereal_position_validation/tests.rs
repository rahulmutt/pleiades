use super::*;

#[test]
fn sidereal_position_gate_passes_within_ceilings() {
    let report = validate_sidereal_position_corpus().expect("sidereal-position gate passes");
    assert_eq!(report.rows_validated, 3082);
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
        "slice sidereal-position file=sidereal-position.csv role=sidereal-position rows=3081 checksum={checksum}"
    );
    assert!(matches!(
        validate(CORPUS_CSV, &manifest),
        Err(SiderealPositionError::ManifestDrift {
            rows_csv: 3082,
            rows_manifest: 3081
        })
    ));
}

// A corpus that lost rows together with its manifest passes the checksum and
// the row count, so only the floor can catch it.
#[test]
fn a_truncated_corpus_with_a_matching_manifest_fails_the_floor() {
    let truncated: String = CORPUS_CSV
        .lines()
        .take(20)
        .map(|line| format!("{line}\n"))
        .collect();
    let rows = parse_corpus(&truncated)
        .expect("truncated corpus parses")
        .len();
    let manifest = format!(
        "slice sidereal-position file=sidereal-position.csv role=sidereal-position rows={rows} checksum={}",
        fnv1a64(&truncated)
    );
    assert_eq!(
        validate(&truncated, &manifest).unwrap().rows_validated,
        rows
    );
    assert!(matches!(
        validate_with_floor(&truncated, &manifest, MIN_ROWS_VALIDATED),
        Err(SiderealPositionError::TooFewRowsValidated { validated, floor: 3082 })
            if validated == rows
    ));
}

#[test]
fn manifest_without_a_slice_line_is_rejected() {
    assert!(matches!(
        validate(CORPUS_CSV, "rows=3082"),
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
        "2451545.0,TrueCitra,Mars,1.0,0.0,0.5,mean",
        "2451545.0,TrueCitra,Mars,1.0,0.0,0.5,apparent,apparent",
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

/// The first apparent corpus row of `body` (1901, True Citra) with `edit`
/// applied to its fields, and a manifest for that one row.
fn apparent_row(body: &str, edit: impl FnOnce(&mut Vec<String>)) -> (String, String) {
    let line = CORPUS_CSV
        .lines()
        .find(|l| l.contains(&format!(",TrueCitra,{body},")) && l.ends_with(",apparent"))
        .expect("an apparent True Citra row");
    let mut fields: Vec<String> = line.split(',').map(str::to_string).collect();
    edit(&mut fields);
    let csv = fields.join(",");
    let manifest = format!("slice x rows=1 checksum={}", fnv1a64(&csv));
    (csv, manifest)
}

#[test]
fn an_unshifted_apparent_row_passes_alone() {
    let (csv, manifest) = apparent_row("Sun", |_| {});
    validate_with_floor(&csv, &manifest, 1).expect("passes");
}

// Without the anchor star's aberration the chart is ~20″ off Swiss Ephemeris:
// the apparent rows are what hold the composition end to end.
#[test]
fn an_apparent_row_read_with_the_mean_ayanamsa_fails() {
    // Shift the SE longitude by the correction at that row, i.e. what a chart
    // using the mean ayanamsa would match.
    let (csv, manifest) = apparent_row("Sun", |fields| {
        let jd: f64 = fields[0].parse().unwrap();
        let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
        let c = pleiades_core::apparent_star_ayanamsa_correction(&Ayanamsa::TrueCitra, instant)
            .unwrap()
            .degrees();
        let lon: f64 = fields[3].parse().unwrap();
        fields[3] = format!("{:.9}", lon + c);
    });
    assert_eq!(
        exceeded_kind(validate_with_floor(&csv, &manifest, 1)),
        "longitude_arcsec"
    );
}

/// Measurement helper, not a gate: prints the apparent rows' maxima the
/// apparent ceilings are sized from, and the row behind each.
/// `cargo test -p pleiades-validate --lib measure_apparent_sidereal -- --ignored --nocapture`
#[test]
#[ignore]
fn measure_apparent_sidereal_maxima() {
    let engine = ChartEngine::new(packaged_backend());
    let rows = parse_corpus(CORPUS_CSV).unwrap();
    for class in ["Sun", "Moon", "Mars"] {
        let mut worst = [(0.0f64, 0.0f64, ""); 3];
        for row in rows.iter().filter(|r| r.apparent && r.body_name == class) {
            let (lon, lat, speed) = chart_place(&engine, row).unwrap();
            let residuals = [
                (wrap_deg(lon, row.lon_deg) * 3600.0).abs(),
                ((lat - row.lat_deg) * 3600.0).abs(),
                ((speed - row.lon_speed) * 3600.0).abs(),
            ];
            for (w, r) in worst.iter_mut().zip(residuals) {
                if r > w.0 {
                    *w = (r, row.jd_tt, row.ayanamsa_name);
                }
            }
        }
        println!(
            "{class}: lon {:.4}\" ({} {}) lat {:.4}\" ({} {}) speed {:.4}\"/d ({} {})",
            worst[0].0,
            worst[0].2,
            worst[0].1,
            worst[1].0,
            worst[1].2,
            worst[1].1,
            worst[2].0,
            worst[2].2,
            worst[2].1
        );
    }
}
