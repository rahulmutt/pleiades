use super::*;

const D: StationKind = StationKind::TurnsDirect;
const R: StationKind = StationKind::TurnsRetrograde;
const CEILINGS: Ceilings = Ceilings {
    time_s: 600.0,
    lon_arcsec: 5.0,
};
const SECOND: f64 = 1.0 / 86_400.0;

fn found(jd: f64, lon_deg: f64, kind: StationKind) -> Found {
    Found { jd, lon_deg, kind }
}

/// A retrograde loop every 100 days: R at +0, D at +20.
fn loops(count: usize) -> Vec<Found> {
    (0..count)
        .flat_map(|i| {
            let t = 2_451_545.0 + 100.0 * i as f64;
            [found(t, 10.0, R), found(t + 20.0, 2.0, D)]
        })
        .collect()
}

#[test]
fn identical_lists_compare_exactly() {
    let corpus = loops(3);
    let residuals = compare_exact("geo Mars", &corpus, &corpus, CEILINGS).unwrap();
    assert_eq!(residuals.matched, 6);
    assert_eq!(residuals.max_time_s, 0.0);
    assert_eq!(residuals.max_lon_arcsec, 0.0);
}

#[test]
fn exact_comparison_rejects_a_missing_or_extra_station() {
    let corpus = loops(3);
    let mut engine = corpus.clone();
    engine.pop();
    assert!(matches!(
        compare_exact("geo Mars", &engine, &corpus, CEILINGS),
        Err(StationsError::CountMismatch {
            got: 5,
            want: 6,
            ..
        })
    ));
    assert!(matches!(
        compare_exact("geo Mars", &corpus, &engine, CEILINGS),
        Err(StationsError::CountMismatch {
            got: 6,
            want: 5,
            ..
        })
    ));
}

#[test]
fn exact_comparison_rejects_a_kind_mismatch() {
    let corpus = loops(2);
    let mut engine = corpus.clone();
    engine[1].kind = R;
    assert!(matches!(
        compare_exact("geo Mars", &engine, &corpus, CEILINGS),
        Err(StationsError::KindMismatch { index: 1, .. })
    ));
}

#[test]
fn exact_comparison_enforces_both_ceilings() {
    let corpus = loops(2);
    let mut late = corpus.clone();
    late[2].jd += 601.0 * SECOND;
    assert!(matches!(
        compare_exact("geo Mars", &late, &corpus, CEILINGS),
        Err(StationsError::CeilingExceeded {
            kind: "time_seconds",
            ..
        })
    ));
    let mut shifted = corpus.clone();
    shifted[2].lon_deg += 6.0 / 3600.0;
    assert!(matches!(
        compare_exact("geo Mars", &shifted, &corpus, CEILINGS),
        Err(StationsError::CeilingExceeded {
            kind: "longitude_arcsec",
            ..
        })
    ));
    let mut nan = corpus.clone();
    nan[0].lon_deg = f64::NAN;
    assert!(matches!(
        compare_exact("geo Mars", &nan, &corpus, CEILINGS),
        Err(StationsError::CeilingExceeded {
            kind: "longitude_arcsec",
            ..
        })
    ));
}

#[test]
fn longitude_residual_wraps_across_zero() {
    let corpus = vec![found(2_451_545.0, 359.999_9, R)];
    let engine = vec![found(2_451_545.0, 0.000_1, R)];
    let residuals = compare_exact("geo Mars", &engine, &corpus, CEILINGS).unwrap();
    assert!(
        (residuals.max_lon_arcsec - 0.72).abs() < 1e-6,
        "{residuals:?}"
    );
}

#[test]
fn residuals_record_the_maximum_and_the_signed_sum() {
    let corpus = loops(1);
    let mut engine = corpus.clone();
    engine[0].jd += 30.0 * SECOND;
    engine[1].jd -= 10.0 * SECOND;
    let residuals = compare_exact("geo Mars", &engine, &corpus, CEILINGS).unwrap();
    assert!((residuals.max_time_s - 30.0).abs() < 1e-3, "{residuals:?}");
    assert!(
        (residuals.sum_signed_time_s - 20.0).abs() < 1e-3,
        "{residuals:?}"
    );
}

// A graze: a D/R pair 0.3 day apart that only one side has. Neither station
// is separated, so the lists still agree.
#[test]
fn separated_comparison_ignores_a_close_pair_on_either_side() {
    let corpus = loops(3);
    let mut engine = corpus.clone();
    let graze = 2_451_545.0 + 60.0;
    engine.insert(2, found(graze, 5.0, R));
    engine.insert(3, found(graze + 0.3, 5.0, D));
    engine.sort_by(|a, b| a.jd.total_cmp(&b.jd));
    let residuals = compare_separated("geo TrueNode", &engine, &corpus, CEILINGS).unwrap();
    assert_eq!(residuals.matched, 6);
    let residuals = compare_separated("geo TrueNode", &corpus, &engine, CEILINGS).unwrap();
    assert_eq!(residuals.matched, 6);
}

#[test]
fn separated_comparison_rejects_a_missing_separated_station() {
    let corpus = loops(3);
    let mut engine = corpus.clone();
    engine.remove(4);
    assert!(matches!(
        compare_separated("geo TrueNode", &engine, &corpus, CEILINGS),
        Err(StationsError::Unmatched { side: "corpus", .. })
    ));
    assert!(matches!(
        compare_separated("geo TrueNode", &corpus, &engine, CEILINGS),
        Err(StationsError::Unmatched { side: "engine", .. })
    ));
}

#[test]
fn separated_comparison_requires_the_same_kind_within_the_time_ceiling() {
    let corpus = loops(2);
    let mut wrong_kind = corpus.clone();
    wrong_kind[2].kind = D;
    assert!(matches!(
        compare_separated("geo TrueNode", &wrong_kind, &corpus, CEILINGS),
        Err(StationsError::Unmatched { .. })
    ));
    let mut late = corpus.clone();
    late[2].jd += 601.0 * SECOND;
    assert!(matches!(
        compare_separated("geo TrueNode", &late, &corpus, CEILINGS),
        Err(StationsError::Unmatched { .. })
    ));
    let mut shifted = corpus.clone();
    shifted[2].lon_deg += 6.0 / 3600.0;
    assert!(matches!(
        compare_separated("geo TrueNode", &shifted, &corpus, CEILINGS),
        Err(StationsError::CeilingExceeded {
            kind: "longitude_arcsec",
            ..
        })
    ));
}

#[test]
fn corpus_rows_parse_into_series() {
    let csv = "# comment\ngroup,body,jd_tt,lon_deg,kind\n\
               geo,Mercury,2451596.1234567,17.5,R\n\
               geo,Mercury,2451617.7,2.25,D\n\
               sid,Saturn,2451800.5,60.0,D\n";
    let series = parse_corpus(csv).unwrap();
    assert_eq!(series.len(), 2);
    assert_eq!(series[0].group, Group::Geo);
    assert_eq!(series[0].body_name, "Mercury");
    assert_eq!(
        series[0].stations,
        vec![
            found(2_451_596.123_456_7, 17.5, R),
            found(2_451_617.7, 2.25, D)
        ]
    );
    assert_eq!(series[1].group, Group::Sid);
    assert_eq!(series[1].body, CelestialBody::Saturn);
}

#[test]
fn malformed_rows_are_rejected() {
    for bad in [
        "geo,Mercury,2451596.1,17.5",
        "geo,Mercury,2451596.1,17.5,R,extra",
        "helio,Mercury,2451596.1,17.5,R",
        "geo,Vulcan,2451596.1,17.5,R",
        "geo,Mercury,NaN,17.5,R",
        "geo,Mercury,2451596.1,inf,R",
        "geo,Mercury,2451596.1,17.5,X",
        "geo,Mercury,2451617.7,2.25,D\ngeo,Mercury,2451596.1,17.5,R",
    ] {
        assert!(
            matches!(parse_corpus(bad), Err(StationsError::MalformedRow(_))),
            "{bad}"
        );
    }
}

#[test]
fn manifest_parses_rows_and_checksum() {
    assert_eq!(
        parse_manifest("slice stations file=stations.csv role=stations rows=12 checksum=99")
            .unwrap(),
        (12, 99)
    );
    for bad in [
        "rows=12 checksum=99",
        "slice stations rows=12",
        "slice stations checksum=9",
    ] {
        assert!(
            matches!(
                parse_manifest(bad),
                Err(StationsError::MalformedManifest(_))
            ),
            "{bad}"
        );
    }
}

#[test]
fn stations_gate_passes_within_ceilings() {
    let report = validate_stations_corpus().expect("stations gate passes");
    eprintln!("{}", report.summary_line());
    for line in report.series_lines() {
        eprintln!("{line}");
    }
    assert!(report.rows_validated >= MIN_ROWS_VALIDATED);
}

#[test]
fn mean_sid_subset_passes_and_compares_only_the_mean_and_sid_series() {
    let report = validate_stations_corpus_subset().expect("stations subset passes");
    eprintln!("{}", report.summary_line());
    assert_eq!(report.rows_validated, MIN_ROWS_VALIDATED_MEAN_SID_SUBSET);
    assert!(
        report
            .summary_line()
            .starts_with("Stations gate (mean/sid subset): "),
        "{}",
        report.summary_line()
    );
    let lines = report.series_lines();
    assert_eq!(lines.len(), 6, "{lines:?}");
    assert!(
        lines
            .iter()
            .all(|line| line.starts_with("mean ") || line.starts_with("sid ")),
        "{lines:?}"
    );
}

#[test]
fn mean_sid_subset_still_fails_a_tampered_corpus_checksum() {
    // Tamper a geo row, which the subset does not compare: the whole-corpus
    // checksum must still catch it.
    let tampered = CORPUS_CSV.replacen("geo,Mercury,", "geo,Mercury, ", 1);
    assert!(matches!(
        validate_scoped(&tampered, MANIFEST, Scope::MeanSidSubset),
        Err(StationsError::ChecksumMismatch { .. })
    ));
}

#[test]
fn tampered_corpus_fails_the_checksum() {
    let tampered = CORPUS_CSV.replacen("geo,Mercury,", "geo,Mercury, ", 1);
    assert!(matches!(
        validate(&tampered, MANIFEST),
        Err(StationsError::ChecksumMismatch { .. })
    ));
}

#[test]
fn manifest_row_count_drift_fails_closed() {
    let (rows, checksum) = parse_manifest(MANIFEST).unwrap();
    let manifest = format!(
        "slice stations file=stations.csv role=stations rows={} checksum={checksum}",
        rows - 1
    );
    assert!(matches!(
        validate(CORPUS_CSV, &manifest),
        Err(StationsError::ManifestDrift { .. })
    ));
}

#[test]
fn a_corpus_missing_a_station_fails_the_count() {
    // Drop Mars's first mean-of-date station: the engine then finds one more
    // than the corpus has.
    let line = CORPUS_CSV
        .lines()
        .find(|l| l.starts_with("mean,Mars,"))
        .expect("a mean Mars row");
    let csv: String = CORPUS_CSV
        .lines()
        .filter(|l| l.starts_with("mean,Mars,") && *l != line)
        .map(|l| format!("{l}\n"))
        .collect();
    let rows = csv.lines().count();
    let manifest = format!("slice x rows={rows} checksum={}", fnv1a64(&csv));
    assert!(matches!(
        validate(&csv, &manifest),
        Err(StationsError::CountMismatch { .. })
    ));
}

#[test]
fn a_shifted_reference_instant_exceeds_the_time_ceiling() {
    // Move Mars's first mean-of-date station by 0.5 day, far above any
    // planetary time ceiling.
    let line = CORPUS_CSV
        .lines()
        .find(|l| l.starts_with("mean,Mars,"))
        .expect("a mean Mars row");
    let mut fields: Vec<String> = line.split(',').map(str::to_string).collect();
    let jd: f64 = fields[2].parse().unwrap();
    fields[2] = format!("{:.7}", jd + 0.5);
    let csv: String = CORPUS_CSV
        .lines()
        .filter(|l| l.starts_with("mean,Mars,"))
        .map(|l| {
            if l == line {
                format!("{}\n", fields.join(","))
            } else {
                format!("{l}\n")
            }
        })
        .collect();
    let rows = csv.lines().count();
    let manifest = format!("slice x rows={rows} checksum={}", fnv1a64(&csv));
    assert!(matches!(
        validate(&csv, &manifest),
        Err(StationsError::CeilingExceeded {
            kind: "time_seconds",
            ..
        })
    ));
}

#[test]
fn an_empty_corpus_validates_too_few_rows() {
    let csv = "group,body,jd_tt,lon_deg,kind\n";
    let manifest = format!("slice x rows=0 checksum={}", fnv1a64(csv));
    assert!(matches!(
        validate(csv, &manifest),
        Err(StationsError::TooFewRowsValidated { validated: 0, .. })
    ));
}
