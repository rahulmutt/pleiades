use super::windows::{
    check_window_edges, check_windows, parse_windows, split_windows, SpeedWindow,
};
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

// A separated station's time ceiling is an existence window about three days
// wide, so alone it passes every station moving by a day (issue #167 (a)).
// Most of the compared stations must also be close to their counterpart.
#[test]
fn separated_comparison_requires_most_stations_to_be_close() {
    const WIDE: Ceilings = Ceilings {
        time_s: 3.0 * 86_400.0,
        lon_arcsec: 5.0,
    };
    let corpus = loops(10);
    let late = |indices: &[usize]| {
        let mut engine = corpus.clone();
        for &index in indices {
            engine[index].jd += 1.0;
        }
        engine
    };
    // Every station a day late: all inside the existence window, none close.
    let all: Vec<usize> = (0..corpus.len()).collect();
    assert!(matches!(
        compare_separated("geo TrueNode", &late(&all), &corpus, WIDE),
        Err(StationsError::TooFewClose {
            close: 0,
            compared: 20,
            ..
        })
    ));
    // Two of twenty: 90 % are close, which is the floor.
    let residuals = compare_separated("geo TrueNode", &late(&[3, 11]), &corpus, WIDE).unwrap();
    assert_eq!((residuals.matched, residuals.close), (20, 18));
    // Three of twenty: 85 %.
    assert!(matches!(
        compare_separated("geo TrueNode", &late(&[3, 11, 15]), &corpus, WIDE),
        Err(StationsError::TooFewClose {
            close: 17,
            compared: 20,
            ..
        })
    ));
    // The boundary is inclusive, and the planets' comparison has no such rule.
    let mut on_the_boundary = corpus.clone();
    for station in &mut on_the_boundary {
        station.jd += TRUE_NODE_CLOSE_DAYS;
    }
    let residuals = compare_separated("geo TrueNode", &on_the_boundary, &corpus, WIDE).unwrap();
    assert_eq!(residuals.close, 20);
    assert!(compare_exact("geo Mars", &late(&all), &corpus, WIDE).is_ok());
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
        "geo,Chiron,2451596.1,17.5,R",
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

// Opt-in: the full gate was nightly `test-full-validate`'s long pole (~105 s
// on the 4-core runner on 2026-10-04, FU-23 (u); 28-29 s by 2026-10-15).
// `mise run gate-stations` runs it (its own nightly job and a `release-gate`
// dependency).
#[test]
fn stations_gate_passes_within_ceilings() {
    if std::env::var("PLEIADES_FULL_STATIONS_GATE").as_deref() != Ok("1") {
        eprintln!(
            "stations_gate_passes_within_ceilings: skipped; set PLEIADES_FULL_STATIONS_GATE=1 to run the full gate"
        );
        return;
    }
    let report = validate_stations_corpus().expect("stations gate passes");
    eprintln!("{}", report.summary_line());
    for line in report.series_lines() {
        eprintln!("{line}");
    }
    assert!(report.rows_validated >= MIN_ROWS_VALIDATED + MIN_ROWS_VALIDATED_ASTEROIDS);
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

#[test]
fn chunk_bounds_cover_the_span_contiguously_on_the_two_day_grid() {
    let (start, end) = FULL_SPAN;
    let bounds = chunk_bounds(start, end, CHUNK_DAYS);
    assert_eq!(bounds.len(), 20, "{bounds:?}");
    assert_eq!(bounds[0].0, start);
    assert_eq!(bounds[bounds.len() - 1].1, end);
    for pair in bounds.windows(2) {
        assert_eq!(pair[0].1, pair[1].0, "chunks are contiguous: {pair:?}");
        assert!(pair[0].1 > pair[0].0, "{pair:?}");
        // Every interior boundary is a sample of every engine step (0.25, 1
        // and 2 days), so a chunk brackets exactly as the single scan does.
        assert_eq!((pair[0].1 - start) % 2.0, 0.0, "{pair:?}");
    }
}

#[test]
fn chunk_bounds_of_a_span_shorter_than_a_chunk_is_the_span() {
    assert_eq!(
        chunk_bounds(2_451_545.0, 2_451_555.0, CHUNK_DAYS),
        vec![(2_451_545.0, 2_451_555.0)]
    );
}

#[test]
fn chunk_bounds_of_an_exact_multiple_has_no_empty_tail() {
    assert_eq!(
        chunk_bounds(10.0, 210.0, 100.0),
        vec![(10.0, 110.0), (110.0, 210.0)]
    );
}

#[test]
fn join_chunks_concatenates_in_order_and_drops_a_seam_duplicate() {
    let a = found(1.0, 0.0, R);
    let b = found(2.0, 0.0, D);
    let c = found(3.0, 0.0, R);
    // Distinct stations across a seam are all kept.
    assert_eq!(join_chunks(vec![vec![a, b], vec![c]]), vec![a, b, c]);
    // A station exactly on a boundary is found by both neighbours from the
    // one bracket they share, and is kept once.
    assert_eq!(join_chunks(vec![vec![a, b], vec![b, c]]), vec![a, b, c]);
    // The single scan can report a boundary root from both adjacent brackets;
    // the chunked scan then has it twice in one chunk and once in the next,
    // and keeps two, like the single scan.
    assert_eq!(
        join_chunks(vec![vec![a, b, b], vec![b, c]]),
        vec![a, b, b, c]
    );
    // Only the previous chunk's last station is compared, not the output's:
    // an empty chunk between two equal stations drops nothing.
    assert_eq!(join_chunks(vec![vec![a], vec![], vec![a]]), vec![a, a]);
    assert_eq!(join_chunks(vec![vec![], vec![a]]), vec![a]);
    assert_eq!(join_chunks(Vec::new()), Vec::<Found>::new());
}

#[test]
fn chunked_scan_matches_the_single_scan_at_every_seam() {
    // 100-day chunks over 1990–2030 put about 146 seams in each series. Mars
    // samples every 2 days and Mercury every day, so both grids meet the
    // boundaries; the engine results must agree to the bit.
    let all = parse_corpus(CORPUS_CSV).unwrap();
    let selected: Vec<&Series> = all
        .iter()
        .filter(|series| {
            series.group == Group::Mean
                && (series.body_name == "Mars" || series.body_name == "Mercury")
        })
        .collect();
    assert_eq!(selected.len(), 2);
    let engine = EventEngine::new(packaged_backend());
    let chunked = scan_series_chunked(&engine, &selected, 100.0);
    assert_eq!(chunked.len(), 2);
    for (series, got) in selected.iter().zip(chunked) {
        let (start, end) = span(series);
        let want: Vec<Found> = engine
            .stations_in_range(
                series.body.clone(),
                series.group.reference(),
                tdb(start),
                tdb(end),
            )
            .unwrap()
            .iter()
            .map(Found::from_station)
            .collect();
        assert_eq!(want.len(), series.stations.len(), "{}", series.body_name);
        assert_eq!(got.unwrap(), want, "{}", series.body_name);
    }
}

#[test]
fn asteroid_corpus_rows_parse_into_four_geo_series() {
    let series = parse_corpus(ASTEROID_CSV).expect("asteroid corpus parses");
    let names: Vec<&str> = series.iter().map(|s| s.body_name).collect();
    assert_eq!(names, ["Ceres", "Pallas", "Juno", "Vesta"]);
    assert!(series.iter().all(|s| s.group == Group::Geo));
    assert!(series.iter().all(|s| span(s) == FULL_SPAN));
    assert!(series.iter().all(|s| ceilings_for(s.body_name).is_some()));
}

#[test]
fn asteroid_corpus_tampering_fails_closed() {
    let tampered = ASTEROID_CSV.replacen(",R\n", ",D\n", 1);
    assert!(matches!(
        validate_scoped(&tampered, ASTEROID_MANIFEST, Scope::Asteroids),
        Err(StationsError::ChecksumMismatch { .. })
    ));
    let (rows, checksum) = parse_manifest(ASTEROID_MANIFEST).unwrap();
    let drifted = format!(
        "slice stations-asteroids file=asteroids.csv role=stations rows={} checksum={checksum}\n",
        rows + 1
    );
    assert!(matches!(
        validate_scoped(ASTEROID_CSV, &drifted, Scope::Asteroids),
        Err(StationsError::ManifestDrift { .. })
    ));
}

// Issue #254: the release-battery subset checks the asteroid corpus against
// its manifest before any scan, so these fail in milliseconds.
#[test]
fn committed_asteroid_corpus_matches_its_manifest() {
    let series = check_manifest(ASTEROID_CSV, ASTEROID_MANIFEST).expect("manifest matches");
    let rows: usize = series.iter().map(|s| s.stations.len()).sum();
    assert_eq!(rows, MIN_ROWS_VALIDATED_ASTEROIDS);
}

#[test]
fn mean_sid_subset_fails_a_tampered_asteroid_corpus() {
    let tampered = ASTEROID_CSV.replacen(",R\n", ",D\n", 1);
    assert_ne!(tampered, ASTEROID_CSV);
    assert!(matches!(
        validate_subset(&SubsetFiles {
            asteroid_csv: &tampered,
            ..SubsetFiles::committed()
        }),
        Err(StationsError::ChecksumMismatch { .. })
    ));
}

#[test]
fn mean_sid_subset_fails_an_asteroid_manifest_row_count_drift() {
    let (rows, checksum) = parse_manifest(ASTEROID_MANIFEST).unwrap();
    let drifted = format!(
        "slice stations-asteroids file=asteroids.csv role=stations rows={} checksum={checksum}\n",
        rows - 1
    );
    assert!(matches!(
        validate_subset(&SubsetFiles {
            asteroid_manifest: &drifted,
            ..SubsetFiles::committed()
        }),
        Err(StationsError::ManifestDrift { .. })
    ));
}

#[test]
fn a_missing_asteroid_station_fails_the_count() {
    let mut lines: Vec<&str> = ASTEROID_CSV.lines().collect();
    let first_row = lines
        .iter()
        .position(|l| l.starts_with("geo,Ceres,"))
        .expect("a Ceres row");
    lines.remove(first_row);
    let csv = lines.join("\n") + "\n";
    let (rows, _) = parse_manifest(ASTEROID_MANIFEST).unwrap();
    let manifest = format!(
        "slice stations-asteroids file=asteroids.csv role=stations rows={} checksum={}\n",
        rows - 1,
        fnv1a64(&csv)
    );
    assert!(matches!(
        validate_scoped(&csv, &manifest, Scope::Asteroids),
        Err(StationsError::CountMismatch { .. })
    ));
}

#[test]
fn asteroid_scope_floor_is_its_own() {
    assert_eq!(Scope::Asteroids.floor(), MIN_ROWS_VALIDATED_ASTEROIDS);
    assert!(Scope::Asteroids.includes(Group::Geo));
    let (rows, _) = parse_manifest(ASTEROID_MANIFEST).unwrap();
    assert_eq!(
        rows, MIN_ROWS_VALIDATED_ASTEROIDS,
        "every asteroid row is compared"
    );
}

#[test]
fn full_report_keeps_the_planet_summary_prefix() {
    let planets = StationsReport {
        rows_validated: 2,
        series_lines: vec!["geo Mercury: …".into()],
        summary_line: "Stations gate: 2 stations validated".into(),
    };
    let asteroids = StationsReport {
        rows_validated: 3,
        series_lines: vec!["geo Ceres: …".into()],
        summary_line: "Asteroid stations: 3 stations validated".into(),
    };
    let merged = planets.merged_with(asteroids);
    assert_eq!(merged.rows_validated, 5);
    assert_eq!(merged.series_lines, ["geo Mercury: …", "geo Ceres: …"]);
    assert!(merged
        .summary_line
        .starts_with("Stations gate: 2 stations validated; "));
    assert!(merged
        .summary_line
        .ends_with("Asteroid stations: 3 stations validated"));
}

fn window(lo: f64, hi: f64, se_stations: usize, sides: (i8, i8), miss: f64) -> SpeedWindow {
    SpeedWindow {
        group: Group::Geo,
        body_name: "TrueApogee",
        turn_jd: 0.5 * (lo + hi),
        miss,
        lo,
        hi,
        se_stations,
        lo_side: sides.0,
        hi_side: sides.1,
        margin: 0.002,
    }
}

#[test]
fn committed_new_corpora_match_their_manifests_and_windows() {
    for (csv, manifest, windows, windows_manifest) in [
        (
            TRUE_APOGEE_CSV,
            TRUE_APOGEE_MANIFEST,
            TRUE_APOGEE_WINDOWS_CSV,
            TRUE_APOGEE_WINDOWS_MANIFEST,
        ),
        (
            FICTITIOUS_CSV,
            FICTITIOUS_MANIFEST,
            FICTITIOUS_WINDOWS_CSV,
            FICTITIOUS_WINDOWS_MANIFEST,
        ),
    ] {
        let series = check_manifest(csv, manifest).expect("manifest matches");
        check_windows(windows, windows_manifest, &series).expect("windows match");
    }
}

#[test]
fn new_corpora_hold_the_expected_bodies() {
    let apogee = check_manifest(TRUE_APOGEE_CSV, TRUE_APOGEE_MANIFEST).unwrap();
    assert_eq!(apogee.len(), 1);
    assert_eq!(
        (apogee[0].group, apogee[0].body_name),
        (Group::Geo, "TrueApogee")
    );
    let fict = check_manifest(FICTITIOUS_CSV, FICTITIOUS_MANIFEST).unwrap();
    // WhiteMoon and Waldemath never station, so they have no series.
    assert_eq!(fict.len(), 17);
    assert!(fict.iter().all(|s| s.group == Group::Mean));
}

#[test]
fn subset_fails_tampered_new_corpora_and_windows() {
    let apogee = TRUE_APOGEE_CSV.replacen(",R\n", ",D\n", 1);
    assert!(matches!(
        validate_subset(&SubsetFiles {
            true_apogee_csv: &apogee,
            ..SubsetFiles::committed()
        }),
        Err(StationsError::ChecksumMismatch { .. })
    ));
    let fict = FICTITIOUS_CSV.replacen(",R\n", ",D\n", 1);
    assert!(matches!(
        validate_subset(&SubsetFiles {
            fictitious_csv: &fict,
            ..SubsetFiles::committed()
        }),
        Err(StationsError::ChecksumMismatch { .. })
    ));
    let windows = format!("{TRUE_APOGEE_WINDOWS_CSV}# tampered\n");
    assert!(matches!(
        validate_subset(&SubsetFiles {
            true_apogee_windows_csv: &windows,
            ..SubsetFiles::committed()
        }),
        Err(StationsError::ChecksumMismatch { .. })
    ));
    let (rows, checksum) = parse_manifest(FICTITIOUS_WINDOWS_MANIFEST).unwrap();
    let drifted = format!(
        "slice stations-fictitious-windows file=fictitious-windows.csv role=windows rows={} checksum={checksum}\n",
        rows + 1
    );
    assert!(matches!(
        validate_subset(&SubsetFiles {
            fictitious_windows_manifest: &drifted,
            ..SubsetFiles::committed()
        }),
        Err(StationsError::ManifestDrift { .. })
    ));
}

const WINDOWS_HEADER: &str = "group,body,turn_jd_tt,miss_deg_per_day,lo_jd_tt,hi_jd_tt,se_stations,lo_side,hi_side,margin_deg_per_day\n";

#[test]
fn windows_parse_and_reject_bad_rows() {
    let ok = format!("{WINDOWS_HEADER}geo,TrueApogee,2485441.4310000,5.950e-5,2485441.3000000,2485441.5600000,0,1,1,2.000e-3\n");
    let w = parse_windows(&ok).unwrap();
    assert_eq!(w.len(), 1);
    assert!(!w[0].strict());
    for bad in [
        // sides contradict the count's parity
        "geo,TrueApogee,2485441.431,5.95e-5,2485441.3,2485441.56,1,1,1,2e-3",
        "geo,TrueApogee,2485441.431,5.95e-5,2485441.3,2485441.56,0,1,-1,2e-3",
        // turn outside the window, lo >= hi
        "geo,TrueApogee,2485442.0,5.95e-5,2485441.3,2485441.56,0,1,1,2e-3",
        "geo,TrueApogee,2485441.431,5.95e-5,2485441.56,2485441.3,0,1,1,2e-3",
        // miss not within the margin, a non-positive margin, a bad side
        "geo,TrueApogee,2485441.431,2e-3,2485441.3,2485441.56,0,1,1,2e-3",
        "geo,TrueApogee,2485441.431,0,2485441.3,2485441.56,0,1,1,0",
        "geo,TrueApogee,2485441.431,5.95e-5,2485441.3,2485441.56,0,+1,1,2e-3",
        // unknown body, wrong field count
        "geo,Moon,2485441.431,5.95e-5,2485441.3,2485441.56,0,1,1,2e-3",
        "geo,TrueApogee,2485441.431,5.95e-5,2485441.3,2485441.56,0,1,1",
    ] {
        assert!(
            matches!(
                parse_windows(&format!("{WINDOWS_HEADER}{bad}\n")),
                Err(StationsError::MalformedWindows(_))
            ),
            "{bad}"
        );
    }
}

#[test]
fn a_window_count_that_disagrees_with_the_corpus_fails() {
    let series = check_manifest(TRUE_APOGEE_CSV, TRUE_APOGEE_MANIFEST).unwrap();
    let first = series[0].stations[0].jd;
    // A window around the first station that claims to hold none.
    let csv = format!(
        "{WINDOWS_HEADER}geo,TrueApogee,{first:.7},1.000e-4,{:.7},{:.7},0,1,1,2.000e-3\n",
        first - 0.01,
        first + 0.01
    );
    let manifest = format!(
        "slice stations-true-apogee-windows file=true-apogee-windows.csv role=windows rows=1 checksum={}\n",
        fnv1a64(&csv)
    );
    assert!(matches!(
        check_windows(&csv, &manifest, &series),
        Err(StationsError::WindowStationCount {
            recorded: 0,
            corpus: 1,
            ..
        })
    ));
}

#[test]
fn near_tangent_windows_accept_any_split_of_the_right_parity() {
    let w = [window(10.0, 12.0, 2, (1, 1), 0.0005)];
    let pair = |n: usize| -> Vec<Found> {
        (0..n)
            .map(|i| found(10.5 + 0.1 * i as f64, 1.0, if i % 2 == 0 { R } else { D }))
            .collect()
    };
    for (got, want) in [(0, 2), (2, 0), (2, 2), (4, 2)] {
        let (_, _, stats) = split_windows("t", &w, &pair(got), &pair(want)).unwrap();
        assert_eq!(stats.differing, usize::from(got != want));
    }
    for (got, want) in [(1, 2), (2, 1)] {
        assert!(matches!(
            split_windows("t", &w, &pair(got), &pair(want)),
            Err(StationsError::WindowParity { .. })
        ));
    }
}

#[test]
fn strict_windows_require_equal_counts() {
    let w = [window(10.0, 12.0, 2, (1, 1), 0.001)];
    assert!(w[0].strict());
    let pair = vec![found(10.5, 1.0, R), found(10.6, 1.0, D)];
    assert!(matches!(
        split_windows("t", &w, &[], &pair),
        Err(StationsError::WindowCount {
            engine: 0,
            corpus: 2,
            ..
        })
    ));
    let (eng, cor, stats) = split_windows("t", &w, &pair, &pair).unwrap();
    assert!(eng.is_empty() && cor.is_empty());
    assert_eq!((stats.windows, stats.strict, stats.differing), (1, 1, 0));
}

#[test]
fn stations_outside_windows_stay_strict() {
    let w = [window(10.0, 12.0, 0, (1, 1), 0.0005)];
    let outside = vec![found(12.000_001, 1.0, R)];
    let (eng, cor, _) = split_windows("t", &w, &[], &outside).unwrap();
    assert!(eng.is_empty());
    assert_eq!(cor.len(), 1);
}

#[test]
fn an_engine_speed_on_the_other_side_at_an_edge_fails() {
    let w = [window(10.0, 12.0, 1, (1, -1), 0.0005)];
    let falling = |jd: f64| Ok(if jd == 10.0 { 0.01 } else { -0.01 });
    assert!(check_window_edges("t", &w, falling).is_ok());
    assert!(matches!(
        check_window_edges("t", &w, |_| Ok(0.01)),
        Err(StationsError::WindowEdgeSide { jd_tt, se_side: -1, .. }) if jd_tt == 12.0
    ));
    // Exactly zero counts as negative, as in the reference tool.
    assert!(check_window_edges("t", &w, |jd| Ok(if jd == 10.0 { 0.0 } else { -0.01 })).is_err());
}

/// The value of the windows file's `# margin_deg_per_day=` header line.
fn header_margin(windows_csv: &str) -> f64 {
    windows_csv
        .lines()
        .find_map(|l| l.strip_prefix("# margin_deg_per_day="))
        .expect("margin header")
        .parse()
        .expect("margin header is a number")
}

/// Every extremum below the margin lies in a window, and the engine's central
/// difference there is within a third of the margin of SE's. Prints the
/// maximum difference.
fn check_margin_basis<B: EphemerisBackend>(
    extrema_csv: &str,
    windows_csv: &str,
    h: f64,
    engine: &EventEngine<B>,
    frame: CrossingFrame,
) {
    let margin = header_margin(windows_csv);
    let windows = parse_windows(windows_csv).unwrap();
    assert!(windows.iter().all(|w| w.margin == margin));
    let mut max_diff = 0.0_f64;
    let mut rows = 0;
    for line in extrema_csv.lines() {
        if line.is_empty() || line.starts_with('#') || line.starts_with("group,") {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        assert_eq!(f.len(), 4, "{line}");
        let group = Group::from_name(f[0]).unwrap();
        let (body, body_name) = body_from_name(f[1]).unwrap();
        let (jd, cd): (f64, f64) = (f[2].parse().unwrap(), f[3].parse().unwrap());
        let speed = engine_speed(engine, body, frame, jd, h).unwrap();
        let diff = (speed - cd).abs();
        max_diff = max_diff.max(diff);
        assert!(
            3.0 * diff <= margin,
            "{line}: engine {speed:e} differs by {diff:e}, margin {margin:e}"
        );
        if cd.abs() < margin {
            assert!(
                windows
                    .iter()
                    .any(|w| w.group == group && w.body_name == body_name && w.contains(jd)),
                "{line} is in no window"
            );
        }
        rows += 1;
    }
    assert!(rows > 0);
    eprintln!(
        "margin basis: {rows} extrema, max |engine - SE| {max_diff:e} deg/day, margin {margin:e}"
    );
}

/// Each corpus's margin is at least 3 times the engine-minus-SE speed
/// difference at every recorded SE extremum below 0.02 deg/day, and every
/// extremum below the margin lies inside a window (issue #167 (d)).
#[test]
fn margins_cover_the_engine_speed_difference_at_every_extremum() {
    check_margin_basis(
        TRUE_APOGEE_EXTREMA_CSV,
        TRUE_APOGEE_WINDOWS_CSV,
        TRUE_APOGEE_STEP_DAYS,
        &EventEngine::new(packaged_backend()),
        CrossingFrame::GeocentricApparentOfDate,
    );
    check_margin_basis(
        FICTITIOUS_EXTREMA_CSV,
        FICTITIOUS_WINDOWS_CSV,
        FICTITIOUS_STEP_DAYS,
        &fictitious_engine(),
        CrossingFrame::GeocentricMeanOfDate,
    );
}
