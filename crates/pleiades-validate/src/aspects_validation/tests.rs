use super::grazes::{parse_grazes, split_windows, GrazeWindow};
use super::*;

const CEILINGS: Ceilings = Ceilings {
    sep_arcsec: 10.0,
    lon_arcsec: 5.0,
};
const SECOND: f64 = 1.0 / 86_400.0;
const ARCSEC: f64 = 1.0 / 3600.0;

fn found(jd: f64, first_lon_deg: f64, second_lon_deg: f64) -> Found {
    Found {
        jd,
        first_lon_deg,
        second_lon_deg,
    }
}

fn expected(jd: f64, first_lon_deg: f64, second_lon_deg: f64, rel_speed: f64) -> Expected {
    Expected {
        found: found(jd, first_lon_deg, second_lon_deg),
        rel_speed_deg_per_day: rel_speed,
    }
}

/// A square every 100 days, alternately first ahead and first behind,
/// closing at 0.5 deg/day.
fn squares(count: usize) -> Vec<Expected> {
    (0..count)
        .map(|i| {
            let jd = 2_451_545.0 + 100.0 * i as f64;
            if i % 2 == 0 {
                expected(jd, 100.0, 10.0, 0.5)
            } else {
                expected(jd, 10.0, 100.0, -0.5)
            }
        })
        .collect()
}

fn engine_of(corpus: &[Expected]) -> Vec<Found> {
    corpus.iter().map(|event| event.found).collect()
}

#[test]
fn identical_lists_compare_exactly() {
    let corpus = squares(4);
    let residuals =
        compare_exact("geo A-B 90", 90.0, &engine_of(&corpus), &corpus, CEILINGS).unwrap();
    assert_eq!(residuals.matched, 4);
    assert_eq!(residuals.max_sep_arcsec, 0.0);
    assert_eq!(residuals.max_time_s, 0.0);
    assert_eq!(residuals.max_lon_arcsec, 0.0);
}

#[test]
fn two_empty_lists_compare_exactly() {
    let residuals = compare_exact("geo Sun-Mercury 60", 60.0, &[], &[], CEILINGS).unwrap();
    assert_eq!(residuals.matched, 0);
}

#[test]
fn a_missing_or_extra_event_is_a_count_mismatch() {
    let corpus = squares(4);
    let mut engine = engine_of(&corpus);
    engine.pop();
    assert!(matches!(
        compare_exact("geo A-B 90", 90.0, &engine, &corpus, CEILINGS),
        Err(AspectsError::CountMismatch {
            got: 3,
            want: 4,
            ..
        })
    ));
    assert!(matches!(
        compare_exact(
            "geo A-B 90",
            90.0,
            &engine_of(&corpus),
            &corpus[..3],
            CEILINGS
        ),
        Err(AspectsError::CountMismatch {
            got: 4,
            want: 3,
            ..
        })
    ));
    // An engine event where the corpus has none: the "never perfects" case.
    assert!(matches!(
        compare_exact("geo Sun-Mercury 60", 60.0, &engine, &[], CEILINGS),
        Err(AspectsError::CountMismatch {
            got: 3,
            want: 0,
            ..
        })
    ));
}

#[test]
fn an_event_on_the_other_side_is_a_side_mismatch() {
    let corpus = squares(2);
    let mut engine = engine_of(&corpus);
    engine[1] = found(engine[1].jd, 100.0, 10.0);
    assert!(matches!(
        compare_exact("geo A-B 90", 90.0, &engine, &corpus, CEILINGS),
        Err(AspectsError::SideMismatch { index: 1, .. })
    ));
}

// At a conjunction the sign of first − second is noise; at an opposition it
// flips at the wrap. Neither has a side.
#[test]
fn conjunctions_and_oppositions_have_no_side() {
    let conjunction = [expected(2_451_545.0, 100.0, 100.0 + 0.1 * ARCSEC, 0.5)];
    let engine = [found(2_451_545.0, 100.0 + 0.1 * ARCSEC, 100.0)];
    compare_exact("geo A-B 0", 0.0, &engine, &conjunction, CEILINGS).unwrap();
    let opposition = [expected(2_451_545.0, 280.0, 100.0 + 0.1 * ARCSEC, 0.5)];
    let engine = [found(2_451_545.0, 280.0 + 0.1 * ARCSEC, 100.0)];
    compare_exact("geo A-B 180", 180.0, &engine, &opposition, CEILINGS).unwrap();
}

// The separation residual is the time residual times the relative speed:
// 1000 s at 0.5 deg/day is 20.8 arcsec, 100 s is 2.08 arcsec.
#[test]
fn the_separation_ceiling_scales_time_by_relative_speed() {
    let corpus = squares(1);
    let late = |seconds: f64| vec![found(corpus[0].found.jd + seconds * SECOND, 100.0, 10.0)];
    let residuals = compare_exact("geo A-B 90", 90.0, &late(100.0), &corpus, CEILINGS).unwrap();
    assert!(
        (residuals.max_sep_arcsec - 2.083).abs() < 0.01,
        "{residuals:?}"
    );
    assert!((residuals.max_time_s - 100.0).abs() < 0.01, "{residuals:?}");
    assert!(
        (residuals.sum_signed_time_s - 100.0).abs() < 0.01,
        "{residuals:?}"
    );
    match compare_exact("geo A-B 90", 90.0, &late(1000.0), &corpus, CEILINGS) {
        Err(AspectsError::CeilingExceeded { kind, residual, .. }) => {
            assert_eq!(kind, "separation_arcsec");
            assert!((residual - 20.83).abs() < 0.1, "{residual}");
        }
        other => panic!("{other:?}"),
    }
    // The same 1000 s at a slow pair's 0.01 deg/day is 0.42 arcsec: within.
    let slow = [expected(corpus[0].found.jd, 100.0, 10.0, 0.01)];
    compare_exact("geo A-B 90", 90.0, &late(1000.0), &slow, CEILINGS).unwrap();
}

#[test]
fn either_longitude_can_exceed_its_ceiling() {
    let corpus = squares(1);
    for (first, second, kind) in [
        (100.0 + 6.0 * ARCSEC, 10.0, "first_longitude_arcsec"),
        (100.0, 10.0 - 6.0 * ARCSEC, "second_longitude_arcsec"),
    ] {
        let engine = [found(corpus[0].found.jd, first, second)];
        match compare_exact("geo A-B 90", 90.0, &engine, &corpus, CEILINGS) {
            Err(AspectsError::CeilingExceeded { kind: got, .. }) => assert_eq!(got, kind),
            other => panic!("{other:?}"),
        }
    }
    let within = [found(corpus[0].found.jd, 100.0 + 4.0 * ARCSEC, 10.0)];
    let residuals = compare_exact("geo A-B 90", 90.0, &within, &corpus, CEILINGS).unwrap();
    assert!(
        (residuals.max_lon_arcsec - 4.0).abs() < 1e-6,
        "{residuals:?}"
    );
}

#[test]
fn longitude_residual_wraps_across_zero() {
    assert!((lon_residual_arcsec(0.0001, 359.9999) - 0.72).abs() < 1e-6);
    assert!((lon_residual_arcsec(359.9999, 0.0001) - 0.72).abs() < 1e-6);
}

#[test]
fn a_nan_residual_fails_closed() {
    let corpus = squares(1);
    let engine = [found(f64::NAN, 100.0, 10.0)];
    assert!(matches!(
        compare_exact("geo A-B 90", 90.0, &engine, &corpus, CEILINGS),
        Err(AspectsError::CeilingExceeded { .. })
    ));
}

#[test]
fn residuals_absorb_maxima_and_sums() {
    let mut total = Residuals {
        matched: 2,
        max_sep_arcsec: 1.0,
        max_time_s: 50.0,
        max_lon_arcsec: 0.2,
        sum_signed_time_s: -10.0,
    };
    total.absorb(Residuals {
        matched: 3,
        max_sep_arcsec: 0.5,
        max_time_s: 80.0,
        max_lon_arcsec: 0.1,
        sum_signed_time_s: 4.0,
    });
    assert_eq!(total.matched, 5);
    assert_eq!(total.max_sep_arcsec, 1.0);
    assert_eq!(total.max_time_s, 80.0);
    assert_eq!(total.max_lon_arcsec, 0.2);
    assert_eq!(total.sum_signed_time_s, -6.0);
}

const SMALL_CSV: &str = "\
# a comment
group,first,second,angle_deg,jd_tt,first_lon_deg,second_lon_deg,rel_speed_deg_per_day
geo,Mars,Saturn,0,2451600.5000000,10.000000000,10.000000001,0.500000000
geo,Mars,Saturn,0,2452300.2500000,200.000000000,200.000000000,0.480000000
geo,Mars,Saturn,90,2451700.5000000,100.000000000,10.000000000,0.510000000
mean,Mercury,Venus,60,2451650.0000000,70.000000000,10.000000000,-0.900000000
";

#[test]
fn corpus_rows_parse_with_their_pair_and_angle() {
    let rows = parse_corpus(SMALL_CSV, &PAIRS).unwrap();
    assert_eq!(rows.len(), 4);
    let pair = |row: &Row| {
        (
            PAIRS[row.pair].group,
            PAIRS[row.pair].first,
            PAIRS[row.pair].second,
        )
    };
    assert_eq!(pair(&rows[0]), (Group::Geo, "Mars", "Saturn"));
    assert_eq!(ANGLES_DEG[rows[0].angle], 0.0);
    assert_eq!(rows[0].expected.found.jd, 2_451_600.5);
    assert_eq!(rows[0].expected.rel_speed_deg_per_day, 0.5);
    assert_eq!(ANGLES_DEG[rows[2].angle], 90.0);
    assert_eq!(pair(&rows[3]), (Group::Mean, "Mercury", "Venus"));
    assert_eq!(ANGLES_DEG[rows[3].angle], 60.0);
    assert_eq!(rows[3].expected.found.first_lon_deg, 70.0);
}

#[test]
fn malformed_rows_are_rejected() {
    let header =
        "group,first,second,angle_deg,jd_tt,first_lon_deg,second_lon_deg,rel_speed_deg_per_day\n";
    for bad in [
        "geo,Mars,Saturn,0,2451600.5,10.0,10.0\n",      // 7 fields
        "sid,Mars,Saturn,0,2451600.5,10.0,10.0,0.5\n",  // unknown group
        "geo,Mars,Venus,0,2451600.5,10.0,10.0,0.5\n",   // pair not in the corpus plan
        "geo,Saturn,Mars,0,2451600.5,10.0,10.0,0.5\n",  // pair in the wrong order
        "mean,Sun,Moon,0,2451600.5,10.0,10.0,0.5\n",    // pair not in this group
        "geo,Mars,Saturn,45,2451600.5,10.0,10.0,0.5\n", // angle not in the corpus plan
        "geo,Mars,Saturn,0,nope,10.0,10.0,0.5\n",       // not a number
        "geo,Mars,Saturn,0,2451600.5,NaN,10.0,0.5\n",   // not finite
        "geo,Mars,Saturn,0,2451600.5,10.0,10.0,0.5\ngeo,Mars,Saturn,0,2451600.5,10.0,10.0,0.5\n", // not ascending
    ] {
        let csv = format!("{header}{bad}");
        assert!(
            matches!(
                parse_corpus(&csv, &PAIRS),
                Err(AspectsError::MalformedRow(_))
            ),
            "{bad}"
        );
    }
}

#[test]
fn manifest_parses_rows_and_checksum() {
    assert_eq!(
        parse_manifest("slice aspects file=aspects.csv role=aspects rows=12 checksum=345\n")
            .unwrap(),
        (12, 345)
    );
    for bad in [
        "",
        "slice aspects rows=12\n",
        "slice aspects checksum=345\n",
        "slice aspects rows=x checksum=345\n",
    ] {
        assert!(
            matches!(parse_manifest(bad), Err(AspectsError::MalformedManifest(_))),
            "{bad}"
        );
    }
}

#[test]
fn too_few_validated_rows_fail_the_floor() {
    assert!(check_floor(10, 10).is_ok());
    assert!(matches!(
        check_floor(9, 10),
        Err(AspectsError::TooFewRowsValidated {
            validated: 9,
            floor: 10
        })
    ));
    // A floor of zero still demands one row.
    assert!(matches!(
        check_floor(0, 0),
        Err(AspectsError::TooFewRowsValidated {
            validated: 0,
            floor: 1
        })
    ));
}

#[test]
fn the_corpus_plan_matches_the_reference_tool() {
    assert_eq!(PAIRS.len(), 11);
    assert_eq!(
        PAIRS.iter().filter(|pair| pair.group == Group::Geo).count(),
        8
    );
    assert_eq!(
        PAIRS
            .iter()
            .filter(|pair| pair.group == Group::Mean)
            .count(),
        2
    );
    assert_eq!(
        PAIRS
            .iter()
            .filter(|pair| pair.group == Group::Helio)
            .count(),
        1
    );
    for pair in &PAIRS {
        assert!(body_from_name(pair.first).is_some(), "{}", pair.first);
        assert!(body_from_name(pair.second).is_some(), "{}", pair.second);
        let name = format!("{}-{}", pair.first, pair.second);
        assert!(ceilings_for(&name).is_some(), "{name}");
    }
    assert!(Scope::Full.includes(Group::Geo));
    assert!(Scope::MeanSubset.includes(Group::Mean));
    assert!(!Scope::MeanSubset.includes(Group::Geo));
    assert!(!Scope::MeanSubset.includes(Group::Helio));
}

/// A manifest that matches `csv`, so a test can change the corpus and get
/// past the checksum to the rule it is testing.
fn manifest_for(csv: &str) -> String {
    let rows = csv
        .lines()
        .filter(|line| {
            ["geo,", "mean,", "helio,"]
                .iter()
                .any(|g| line.starts_with(g))
        })
        .count();
    format!(
        "slice aspects file=aspects.csv role=aspects rows={rows} checksum={}",
        fnv1a64(csv)
    )
}

/// The first corpus line of the mean Mars–Saturn conjunction series.
fn first_mean_mars_saturn_conjunction() -> &'static str {
    CORPUS_CSV
        .lines()
        .find(|line| line.starts_with("mean,Mars,Saturn,0,"))
        .expect("the corpus has a mean Mars-Saturn conjunction")
}

// Opt-in: the planet-pair gate measured 18.5 minutes as this test on
// 2026-10-02, which nightly `test-full` could not afford (the command now
// takes about 2 minutes on the nightly runner; see `validate_aspects_corpus`).
// `mise run gate-aspects` runs it (its own nightly job and a `release-gate`
// dependency).
#[test]
fn aspects_gate_passes_within_ceilings() {
    if std::env::var("PLEIADES_FULL_ASPECTS_GATE").as_deref() != Ok("1") {
        eprintln!(
            "aspects_gate_passes_within_ceilings: skipped; set PLEIADES_FULL_ASPECTS_GATE=1 to run the full gate"
        );
        return;
    }
    let started = std::time::Instant::now();
    let report = validate_aspects_corpus().expect("aspects gate passes");
    eprintln!("{}", report.summary_line());
    for line in report.pair_lines() {
        eprintln!("{line}");
    }
    eprintln!("full gate: {:.1} s", started.elapsed().as_secs_f64());
    assert!(
        report.rows_validated
            >= MIN_ROWS_VALIDATED + MIN_ROWS_VALIDATED_ASTEROIDS + MIN_ROWS_VALIDATED_FICTITIOUS
    );
    assert_eq!(
        report.pair_lines().len(),
        PAIRS.len() + ASTEROID_PAIRS.len() + FICTITIOUS_PAIRS.len()
    );
}

// Opt-in like the gate above: the lunar-point pass is the long half of the
// exact-aspect gate (`mise run gate-aspects-lunar`, its own nightly job and a
// `release-gate` dependency).
#[test]
fn lunar_point_aspects_gate_passes_within_ceilings() {
    if std::env::var("PLEIADES_FULL_ASPECTS_GATE").as_deref() != Ok("1") {
        eprintln!(
            "lunar_point_aspects_gate_passes_within_ceilings: skipped; set PLEIADES_FULL_ASPECTS_GATE=1 to run the lunar-point gate"
        );
        return;
    }
    let started = std::time::Instant::now();
    let report = validate_lunar_point_aspects_corpus().expect("lunar-point aspects gate passes");
    eprintln!("{}", report.summary_line());
    for line in report.pair_lines() {
        eprintln!("{line}");
    }
    eprintln!("lunar-point gate: {:.1} s", started.elapsed().as_secs_f64());
    assert!(report.rows_validated >= MIN_ROWS_VALIDATED_LUNAR_POINTS);
    assert_eq!(report.pair_lines().len(), LUNAR_POINT_PAIRS.len());
}

// Opt-in like the gates above (`mise run gate-aspects` runs the fictitious
// pass too, as part of `validate_aspects_corpus`).
#[test]
fn fictitious_aspects_gate_passes_within_ceilings() {
    if std::env::var("PLEIADES_FULL_ASPECTS_GATE").as_deref() != Ok("1") {
        eprintln!(
            "fictitious_aspects_gate_passes_within_ceilings: skipped; set PLEIADES_FULL_ASPECTS_GATE=1 to run the fictitious-body gate"
        );
        return;
    }
    let started = std::time::Instant::now();
    let report = validate_fictitious_aspects_corpus().expect("fictitious-body aspects gate passes");
    eprintln!("{}", report.summary_line());
    for line in report.pair_lines() {
        eprintln!("{line}");
    }
    eprintln!(
        "fictitious-body gate: {:.1} s",
        started.elapsed().as_secs_f64()
    );
    assert!(report.rows_validated >= MIN_ROWS_VALIDATED_FICTITIOUS);
    assert_eq!(report.pair_lines().len(), FICTITIOUS_PAIRS.len());
}

#[test]
fn fictitious_corpus_parses_into_the_twenty_eight_pairs() {
    let rows = parse_corpus(FICTITIOUS_CSV, &FICTITIOUS_PAIRS).expect("fictitious corpus parses");
    let (rows_manifest, _) = parse_manifest(FICTITIOUS_MANIFEST).unwrap();
    assert_eq!(rows.len(), rows_manifest);
    let names: Vec<String> = FICTITIOUS_PAIRS
        .iter()
        .map(|pair| format!("{}-{}", pair.first, pair.second))
        .collect();
    assert_eq!(
        names,
        [
            "Cupido-Sun",
            "Hades-Sun",
            "Zeus-Sun",
            "Kronos-Sun",
            "Apollon-Sun",
            "Admetos-Sun",
            "Vulkanus-Sun",
            "Poseidon-Sun",
            "Transpluto-Sun",
            "Nibiru-Sun",
            "Harrington-Sun",
            "NeptuneLeverrier-Sun",
            "NeptuneAdams-Sun",
            "PlutoLowell-Sun",
            "PlutoPickering-Sun",
            "Vulcan-Sun",
            "WhiteMoon-Sun",
            "Proserpina-Sun",
            "Waldemath-Sun",
            "Cupido-Saturn",
            "Hades-Saturn",
            "Zeus-Saturn",
            "Kronos-Saturn",
            "Apollon-Saturn",
            "Admetos-Saturn",
            "Vulkanus-Saturn",
            "Poseidon-Saturn",
            "Transpluto-Saturn"
        ]
    );
    let expected_counts = [
        1593, 1595, 1596, 1597, 1597, 1597, 1598, 1598, 1598, 1600, 1599, 1593, 1593, 1594, 1597,
        7461, 1371, 1598, 3300, 125, 118, 112, 107, 112, 126, 120, 119, 119,
    ];
    for (index, pair) in FICTITIOUS_PAIRS.iter().enumerate() {
        assert_eq!(pair.group, Group::Mean);
        assert_eq!(pair.span, FULL_SPAN);
        let count = rows.iter().filter(|row| row.pair == index).count();
        assert_eq!(count, expected_counts[index], "{}", names[index]);
        assert!(body_from_name(pair.first).is_some(), "{}", pair.first);
        assert!(body_from_name(pair.second).is_some(), "{}", pair.second);
        assert!(ceilings_for(&names[index]).is_some(), "{}", names[index]);
    }
    assert!(rows.iter().all(|row| row.pair < FICTITIOUS_PAIRS.len()));
}

#[test]
fn committed_fictitious_corpus_matches_its_manifest() {
    let rows = check_manifest(FICTITIOUS_CSV, FICTITIOUS_MANIFEST, &FICTITIOUS_PAIRS)
        .expect("manifest matches");
    let windows = check_grazes(
        FICTITIOUS_GRAZES_CSV,
        FICTITIOUS_GRAZES_MANIFEST,
        &FICTITIOUS_PAIRS,
        &rows,
    )
    .expect("graze windows match their manifest");
    let in_windows: usize = windows.iter().map(|w| w.se_events).sum();
    assert_eq!(rows.len() - in_windows, MIN_ROWS_VALIDATED_FICTITIOUS);
}

#[test]
fn committed_fictitious_graze_windows_match_their_manifest_and_corpus() {
    let rows = check_manifest(FICTITIOUS_CSV, FICTITIOUS_MANIFEST, &FICTITIOUS_PAIRS)
        .expect("fictitious corpus matches its manifest");
    let windows = check_grazes(
        FICTITIOUS_GRAZES_CSV,
        FICTITIOUS_GRAZES_MANIFEST,
        &FICTITIOUS_PAIRS,
        &rows,
    )
    .expect("graze windows match their manifest and the corpus");
    let (manifest_rows, _) = parse_manifest(FICTITIOUS_GRAZES_MANIFEST).unwrap();
    assert_eq!(windows.len(), manifest_rows);
    assert!(windows.iter().all(|w| w.pair < FICTITIOUS_PAIRS.len()));
}

#[test]
fn fictitious_scope_floor_is_its_own() {
    assert_eq!(Scope::Fictitious.floor(), MIN_ROWS_VALIDATED_FICTITIOUS);
    assert!(Scope::Fictitious.includes(Group::Mean));
    assert_eq!(Scope::Fictitious.pairs().len(), FICTITIOUS_PAIRS.len());
    let (rows, _) = parse_manifest(FICTITIOUS_MANIFEST).unwrap();
    let windows = parse_grazes(FICTITIOUS_GRAZES_CSV, &FICTITIOUS_PAIRS).unwrap();
    let in_windows: usize = windows.iter().map(|w| w.se_events).sum();
    assert_eq!(
        rows - in_windows,
        MIN_ROWS_VALIDATED_FICTITIOUS,
        "every fictitious-body row outside a graze window is compared"
    );
}

#[test]
fn mean_subset_fails_a_tampered_fictitious_corpus() {
    let tampered = FICTITIOUS_CSV.replacen("mean,Cupido,Sun,", "mean,Cupido,Sun, ", 1);
    assert_ne!(tampered, FICTITIOUS_CSV);
    assert!(matches!(
        validate_subset(
            ASTEROID_CSV,
            ASTEROID_MANIFEST,
            LUNAR_POINT_CSV,
            LUNAR_POINT_MANIFEST,
            LUNAR_POINT_GRAZES_CSV,
            LUNAR_POINT_GRAZES_MANIFEST,
            &tampered,
            FICTITIOUS_MANIFEST,
            FICTITIOUS_GRAZES_CSV,
            FICTITIOUS_GRAZES_MANIFEST
        ),
        Err(AspectsError::ChecksumMismatch { .. })
    ));
}

#[test]
fn mean_subset_fails_tampered_fictitious_graze_windows() {
    // The committed file holds no windows, so the tamper lands on its header
    // comment; the checksum still binds the whole file.
    let (manifest_rows, _) = parse_manifest(FICTITIOUS_GRAZES_MANIFEST).unwrap();
    let tampered = if manifest_rows == 0 {
        assert!(
            FICTITIOUS_GRAZES_CSV
                .lines()
                .all(|line| line.starts_with('#') || line.starts_with("group,")),
            "the file holds only its header and comments"
        );
        FICTITIOUS_GRAZES_CSV.replacen("# Source: as", "# Source: As", 1)
    } else {
        FICTITIOUS_GRAZES_CSV.replacen(",2\n", ",4\n", 1)
    };
    assert_ne!(tampered, FICTITIOUS_GRAZES_CSV);
    assert!(matches!(
        validate_subset(
            ASTEROID_CSV,
            ASTEROID_MANIFEST,
            LUNAR_POINT_CSV,
            LUNAR_POINT_MANIFEST,
            LUNAR_POINT_GRAZES_CSV,
            LUNAR_POINT_GRAZES_MANIFEST,
            FICTITIOUS_CSV,
            FICTITIOUS_MANIFEST,
            &tampered,
            FICTITIOUS_GRAZES_MANIFEST
        ),
        Err(AspectsError::ChecksumMismatch { .. })
    ));
}

#[test]
fn mean_subset_fails_a_fictitious_manifest_row_count_drift() {
    let (rows, checksum) = parse_manifest(FICTITIOUS_MANIFEST).unwrap();
    let drifted = format!(
        "slice aspects-fictitious file=fictitious.csv role=aspects rows={} checksum={checksum}\n",
        rows - 1
    );
    assert!(matches!(
        validate_subset(
            ASTEROID_CSV,
            ASTEROID_MANIFEST,
            LUNAR_POINT_CSV,
            LUNAR_POINT_MANIFEST,
            LUNAR_POINT_GRAZES_CSV,
            LUNAR_POINT_GRAZES_MANIFEST,
            FICTITIOUS_CSV,
            &drifted,
            FICTITIOUS_GRAZES_CSV,
            FICTITIOUS_GRAZES_MANIFEST
        ),
        Err(AspectsError::ManifestDrift { .. })
    ));
}

#[test]
fn mean_subset_passes_and_compares_only_the_mean_group() {
    let started = std::time::Instant::now();
    let report = validate_aspects_corpus_subset().expect("aspects subset passes");
    eprintln!("{}", report.summary_line());
    for line in report.pair_lines() {
        eprintln!("{line}");
    }
    eprintln!("mean subset: {:.1} s", started.elapsed().as_secs_f64());
    assert_eq!(report.rows_validated, MIN_ROWS_VALIDATED_MEAN_SUBSET);
    assert!(
        report
            .summary_line()
            .starts_with("Aspects gate (mean subset): "),
        "{}",
        report.summary_line()
    );
    let lines = report.pair_lines();
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(
        lines.iter().all(|line| line.starts_with("mean ")),
        "{lines:?}"
    );
}

#[test]
fn a_tampered_corpus_fails_the_checksum_in_either_scope() {
    // A geo row, which the subset does not compare: the whole-corpus
    // checksum must still catch it.
    let tampered = CORPUS_CSV.replacen("geo,Sun,Moon,", "geo,Sun,Moon, ", 1);
    assert_ne!(tampered, CORPUS_CSV);
    assert!(matches!(
        validate(&tampered, MANIFEST),
        Err(AspectsError::ChecksumMismatch { .. })
    ));
    assert!(matches!(
        validate_scoped(&tampered, MANIFEST, Scope::MeanSubset, None),
        Err(AspectsError::ChecksumMismatch { .. })
    ));
}

#[test]
fn manifest_row_count_drift_fails_closed() {
    let (rows, checksum) = parse_manifest(MANIFEST).unwrap();
    let manifest = format!(
        "slice aspects file=aspects.csv role=aspects rows={} checksum={checksum}",
        rows + 1
    );
    assert!(matches!(
        validate_scoped(CORPUS_CSV, &manifest, Scope::MeanSubset, None),
        Err(AspectsError::ManifestDrift { .. })
    ));
}

#[test]
fn a_corpus_missing_an_event_fails_the_count() {
    let line = first_mean_mars_saturn_conjunction();
    let without = CORPUS_CSV.replacen(&format!("{line}\n"), "", 1);
    assert_ne!(without, CORPUS_CSV);
    match validate_scoped(&without, &manifest_for(&without), Scope::MeanSubset, None) {
        Err(AspectsError::CountMismatch { series, got, want }) => {
            assert_eq!(series, "mean Mars-Saturn 0");
            assert_eq!(got, want + 1);
        }
        other => panic!("{other:?}"),
    }
}

// 0.05 day at Mars–Saturn's relative speed is tens of arcseconds, far over
// the measured ceiling.
#[test]
fn a_shifted_reference_instant_exceeds_the_separation_ceiling() {
    let line = first_mean_mars_saturn_conjunction();
    let mut fields: Vec<String> = line.split(',').map(str::to_string).collect();
    let jd: f64 = fields[4].parse().unwrap();
    fields[4] = format!("{:.7}", jd + 0.05);
    let shifted = CORPUS_CSV.replacen(line, &fields.join(","), 1);
    assert_ne!(shifted, CORPUS_CSV);
    match validate_scoped(&shifted, &manifest_for(&shifted), Scope::MeanSubset, None) {
        Err(AspectsError::CeilingExceeded { series, kind, .. }) => {
            assert_eq!(series, "mean Mars-Saturn 0");
            assert_eq!(kind, "separation_arcsec");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn asteroid_corpus_parses_into_the_seven_asteroid_pairs() {
    let rows = parse_corpus(ASTEROID_CSV, &ASTEROID_PAIRS).expect("asteroid corpus parses");
    let (rows_manifest, _) = parse_manifest(ASTEROID_MANIFEST).unwrap();
    assert_eq!(rows.len(), rows_manifest);
    let names: Vec<String> = ASTEROID_PAIRS
        .iter()
        .map(|pair| format!("{}-{}", pair.first, pair.second))
        .collect();
    assert_eq!(
        names,
        [
            "Sun-Ceres",
            "Sun-Pallas",
            "Sun-Juno",
            "Sun-Vesta",
            "Moon-Ceres",
            "Mars-Vesta",
            "Jupiter-Ceres"
        ]
    );
    for (index, pair) in ASTEROID_PAIRS.iter().enumerate() {
        assert_eq!(pair.group, Group::Geo);
        assert!(
            rows.iter().any(|row| row.pair == index),
            "no rows for {}",
            names[index]
        );
        assert!(body_from_name(pair.first).is_some(), "{}", pair.first);
        assert!(body_from_name(pair.second).is_some(), "{}", pair.second);
        assert!(ceilings_for(&names[index]).is_some(), "{}", names[index]);
    }
    assert!(rows.iter().all(|row| row.pair < ASTEROID_PAIRS.len()));
    // The planet plan does not know the asteroid pairs.
    assert!(matches!(
        parse_corpus(ASTEROID_CSV, &PAIRS),
        Err(AspectsError::MalformedRow(_))
    ));
}

/// Issue #253: a Sun or Moon pair's separation never turns, so without a
/// slow pair the gate would never meet an asteroid's retrograde loop. A
/// pair whose corpus holds events at both signs of the relative speed has
/// a turning separation.
#[test]
fn the_asteroid_corpus_has_pairs_whose_separation_turns() {
    let rows = parse_corpus(ASTEROID_CSV, &ASTEROID_PAIRS).expect("asteroid corpus parses");
    let turning: Vec<String> = ASTEROID_PAIRS
        .iter()
        .enumerate()
        .filter(|(index, _)| {
            let speeds = || {
                rows.iter()
                    .filter(|row| row.pair == *index)
                    .map(|row| row.expected.rel_speed_deg_per_day)
            };
            speeds().any(|s| s > 0.0) && speeds().any(|s| s < 0.0)
        })
        .map(|(_, pair)| format!("{}-{}", pair.first, pair.second))
        .collect();
    assert_eq!(turning, ["Mars-Vesta", "Jupiter-Ceres"]);
}

#[test]
fn asteroid_corpus_tampering_fails_closed() {
    let tampered = ASTEROID_CSV.replacen("geo,Sun,Ceres,", "geo,Sun,Ceres, ", 1);
    assert_ne!(tampered, ASTEROID_CSV);
    assert!(matches!(
        validate_scoped(&tampered, ASTEROID_MANIFEST, Scope::Asteroids, None),
        Err(AspectsError::ChecksumMismatch { .. })
    ));
    let (rows, checksum) = parse_manifest(ASTEROID_MANIFEST).unwrap();
    let drifted = format!(
        "slice aspects-asteroids file=asteroids.csv role=aspects rows={} checksum={checksum}\n",
        rows + 1
    );
    assert!(matches!(
        validate_scoped(ASTEROID_CSV, &drifted, Scope::Asteroids, None),
        Err(AspectsError::ManifestDrift { .. })
    ));
}

// Issue #254: the release-battery subset checks the asteroid corpus against
// its manifest before any scan, so these fail in milliseconds.
#[test]
fn committed_asteroid_corpus_matches_its_manifest() {
    let rows =
        check_manifest(ASTEROID_CSV, ASTEROID_MANIFEST, &ASTEROID_PAIRS).expect("manifest matches");
    assert_eq!(rows.len(), MIN_ROWS_VALIDATED_ASTEROIDS);
}

#[test]
fn mean_subset_fails_a_tampered_asteroid_corpus() {
    let tampered = ASTEROID_CSV.replacen("geo,Sun,Ceres,", "geo,Sun,Ceres, ", 1);
    assert_ne!(tampered, ASTEROID_CSV);
    assert!(matches!(
        validate_subset(
            &tampered,
            ASTEROID_MANIFEST,
            LUNAR_POINT_CSV,
            LUNAR_POINT_MANIFEST,
            LUNAR_POINT_GRAZES_CSV,
            LUNAR_POINT_GRAZES_MANIFEST,
            FICTITIOUS_CSV,
            FICTITIOUS_MANIFEST,
            FICTITIOUS_GRAZES_CSV,
            FICTITIOUS_GRAZES_MANIFEST
        ),
        Err(AspectsError::ChecksumMismatch { .. })
    ));
}

#[test]
fn mean_subset_fails_an_asteroid_manifest_row_count_drift() {
    let (rows, checksum) = parse_manifest(ASTEROID_MANIFEST).unwrap();
    let drifted = format!(
        "slice aspects-asteroids file=asteroids.csv role=aspects rows={} checksum={checksum}\n",
        rows - 1
    );
    assert!(matches!(
        validate_subset(
            ASTEROID_CSV,
            &drifted,
            LUNAR_POINT_CSV,
            LUNAR_POINT_MANIFEST,
            LUNAR_POINT_GRAZES_CSV,
            LUNAR_POINT_GRAZES_MANIFEST,
            FICTITIOUS_CSV,
            FICTITIOUS_MANIFEST,
            FICTITIOUS_GRAZES_CSV,
            FICTITIOUS_GRAZES_MANIFEST
        ),
        Err(AspectsError::ManifestDrift { .. })
    ));
}

#[test]
fn a_missing_asteroid_aspect_fails_the_count() {
    let mut lines: Vec<&str> = ASTEROID_CSV.lines().collect();
    let first_row = lines
        .iter()
        .position(|l| l.starts_with("geo,Sun,Ceres,0,"))
        .expect("a Sun-Ceres conjunction row");
    lines.remove(first_row);
    let csv = lines.join("\n") + "\n";
    let (rows, _) = parse_manifest(ASTEROID_MANIFEST).unwrap();
    let manifest = format!(
        "slice aspects-asteroids file=asteroids.csv role=aspects rows={} checksum={}\n",
        rows - 1,
        fnv1a64(&csv)
    );
    assert!(matches!(
        validate_scoped(&csv, &manifest, Scope::Asteroids, None),
        Err(AspectsError::CountMismatch { .. })
    ));
}

#[test]
fn asteroid_scope_floor_is_its_own() {
    assert_eq!(Scope::Asteroids.floor(), MIN_ROWS_VALIDATED_ASTEROIDS);
    assert!(Scope::Asteroids.includes(Group::Geo));
    assert_eq!(Scope::Asteroids.pairs().len(), ASTEROID_PAIRS.len());
    assert_eq!(Scope::Full.pairs().len(), PAIRS.len());
    assert_eq!(Scope::MeanSubset.pairs().len(), PAIRS.len());
    let (rows, _) = parse_manifest(ASTEROID_MANIFEST).unwrap();
    assert_eq!(
        rows, MIN_ROWS_VALIDATED_ASTEROIDS,
        "every asteroid row is compared"
    );
}

#[test]
fn lunar_point_corpus_parses_into_the_twenty_two_lunar_pairs() {
    let rows = parse_corpus(LUNAR_POINT_CSV, &LUNAR_POINT_PAIRS).expect("lunar corpus parses");
    let (rows_manifest, _) = parse_manifest(LUNAR_POINT_MANIFEST).unwrap();
    assert_eq!(rows.len(), rows_manifest);
    let names: Vec<String> = LUNAR_POINT_PAIRS
        .iter()
        .map(|pair| format!("{}-{}", pair.first, pair.second))
        .collect();
    assert_eq!(
        names,
        [
            "TrueNode-Sun",
            "TrueNode-Mars",
            "TrueNode-Jupiter",
            "TrueNode-Saturn",
            "TrueNode-Uranus",
            "TrueNode-Neptune",
            "TrueNode-Pluto",
            "TrueApogee-Sun",
            "TrueApogee-Jupiter",
            "TrueApogee-Saturn",
            "TrueApogee-Pluto",
            "MeanNode-Sun",
            "MeanNode-Mars",
            "MeanNode-Jupiter",
            "MeanNode-Saturn",
            "MeanNode-Uranus",
            "MeanNode-Neptune",
            "MeanNode-Pluto",
            "MeanApogee-Sun",
            "MeanApogee-Jupiter",
            "MeanApogee-Saturn",
            "MeanApogee-Pluto"
        ]
    );
    let expected_counts = [
        1686, 990, 247, 166, 119, 109, 107, 3555, 3438, 3485, 3421, 1686, 996, 255, 152, 105, 95,
        93, 1418, 173, 130, 174,
    ];
    for (index, pair) in LUNAR_POINT_PAIRS.iter().enumerate() {
        assert_eq!(pair.group, Group::Geo);
        let count = rows.iter().filter(|row| row.pair == index).count();
        assert_eq!(count, expected_counts[index], "{}", names[index]);
        assert!(body_from_name(pair.first).is_some(), "{}", pair.first);
        assert!(body_from_name(pair.second).is_some(), "{}", pair.second);
        assert!(ceilings_for(&names[index]).is_some(), "{}", names[index]);
    }
    assert!(rows.iter().all(|row| row.pair < LUNAR_POINT_PAIRS.len()));
}

#[test]
fn lunar_point_corpus_tampering_fails_closed() {
    let tampered = LUNAR_POINT_CSV.replacen("geo,TrueNode,Sun,", "geo,TrueNode,Sun, ", 1);
    assert_ne!(tampered, LUNAR_POINT_CSV);
    assert!(matches!(
        validate_scoped(&tampered, LUNAR_POINT_MANIFEST, Scope::LunarPoints, None),
        Err(AspectsError::ChecksumMismatch { .. })
    ));
    let (rows, checksum) = parse_manifest(LUNAR_POINT_MANIFEST).unwrap();
    let drifted = format!(
        "slice aspects-lunar-points file=lunar-points.csv role=aspects rows={} checksum={checksum}\n",
        rows + 1
    );
    assert!(matches!(
        validate_scoped(LUNAR_POINT_CSV, &drifted, Scope::LunarPoints, None),
        Err(AspectsError::ManifestDrift { .. })
    ));
}

#[test]
fn committed_lunar_point_corpus_matches_its_manifest() {
    let rows = check_manifest(LUNAR_POINT_CSV, LUNAR_POINT_MANIFEST, &LUNAR_POINT_PAIRS)
        .expect("manifest matches");
    let windows = check_grazes(
        LUNAR_POINT_GRAZES_CSV,
        LUNAR_POINT_GRAZES_MANIFEST,
        &LUNAR_POINT_PAIRS,
        &rows,
    )
    .expect("graze windows match their manifest");
    let in_windows: usize = windows.iter().map(|w| w.se_events).sum();
    assert_eq!(rows.len() - in_windows, MIN_ROWS_VALIDATED_LUNAR_POINTS);
}

#[test]
fn mean_subset_fails_a_tampered_lunar_point_corpus() {
    let tampered = LUNAR_POINT_CSV.replacen("geo,TrueNode,Sun,", "geo,TrueNode,Sun, ", 1);
    assert_ne!(tampered, LUNAR_POINT_CSV);
    assert!(matches!(
        validate_subset(
            ASTEROID_CSV,
            ASTEROID_MANIFEST,
            &tampered,
            LUNAR_POINT_MANIFEST,
            LUNAR_POINT_GRAZES_CSV,
            LUNAR_POINT_GRAZES_MANIFEST,
            FICTITIOUS_CSV,
            FICTITIOUS_MANIFEST,
            FICTITIOUS_GRAZES_CSV,
            FICTITIOUS_GRAZES_MANIFEST
        ),
        Err(AspectsError::ChecksumMismatch { .. })
    ));
}

#[test]
fn mean_subset_fails_a_lunar_point_manifest_row_count_drift() {
    let (rows, checksum) = parse_manifest(LUNAR_POINT_MANIFEST).unwrap();
    let drifted = format!(
        "slice aspects-lunar-points file=lunar-points.csv role=aspects rows={} checksum={checksum}\n",
        rows - 1
    );
    assert!(matches!(
        validate_subset(
            ASTEROID_CSV,
            ASTEROID_MANIFEST,
            LUNAR_POINT_CSV,
            &drifted,
            LUNAR_POINT_GRAZES_CSV,
            LUNAR_POINT_GRAZES_MANIFEST,
            FICTITIOUS_CSV,
            FICTITIOUS_MANIFEST,
            FICTITIOUS_GRAZES_CSV,
            FICTITIOUS_GRAZES_MANIFEST
        ),
        Err(AspectsError::ManifestDrift { .. })
    ));
}

#[test]
fn committed_graze_windows_match_their_manifest_and_corpus() {
    let rows = check_manifest(LUNAR_POINT_CSV, LUNAR_POINT_MANIFEST, &LUNAR_POINT_PAIRS)
        .expect("lunar corpus matches its manifest");
    let windows = check_grazes(
        LUNAR_POINT_GRAZES_CSV,
        LUNAR_POINT_GRAZES_MANIFEST,
        &LUNAR_POINT_PAIRS,
        &rows,
    )
    .expect("graze windows match their manifest and the corpus");
    assert!(!windows.is_empty());
    // The spike's 1.84" TrueApogee-Jupiter turn; the spike did not record its angle.
    assert!(windows.iter().any(|w| {
        let pair = &LUNAR_POINT_PAIRS[w.pair];
        pair.first == "TrueApogee"
            && pair.second == "Jupiter"
            && w.lo <= 2_420_750.82
            && 2_420_750.82 <= w.hi
    }));
    assert!(windows.iter().all(
        |w| ["TrueApogee", "MeanNode", "MeanApogee"].contains(&LUNAR_POINT_PAIRS[w.pair].first)
    ));
}

#[test]
fn mean_subset_fails_tampered_graze_windows() {
    let tampered = LUNAR_POINT_GRAZES_CSV.replacen(",2\n", ",4\n", 1);
    assert_ne!(tampered, LUNAR_POINT_GRAZES_CSV);
    assert!(matches!(
        validate_subset(
            ASTEROID_CSV,
            ASTEROID_MANIFEST,
            LUNAR_POINT_CSV,
            LUNAR_POINT_MANIFEST,
            &tampered,
            LUNAR_POINT_GRAZES_MANIFEST,
            FICTITIOUS_CSV,
            FICTITIOUS_MANIFEST,
            FICTITIOUS_GRAZES_CSV,
            FICTITIOUS_GRAZES_MANIFEST
        ),
        Err(AspectsError::ChecksumMismatch { .. })
    ));
}

#[test]
fn mean_subset_fails_a_window_whose_se_events_disagree() {
    let tampered = LUNAR_POINT_GRAZES_CSV.replacen(",2\n", ",4\n", 1);
    assert_ne!(tampered, LUNAR_POINT_GRAZES_CSV);
    let (rows, _) = parse_manifest(LUNAR_POINT_GRAZES_MANIFEST).unwrap();
    let manifest = format!(
        "slice aspects-lunar-points-grazes file=lunar-points-grazes.csv role=grazes rows={rows} checksum={}\n",
        fnv1a64(&tampered)
    );
    assert!(matches!(
        validate_subset(
            ASTEROID_CSV,
            ASTEROID_MANIFEST,
            LUNAR_POINT_CSV,
            LUNAR_POINT_MANIFEST,
            &tampered,
            &manifest,
            FICTITIOUS_CSV,
            FICTITIOUS_MANIFEST,
            FICTITIOUS_GRAZES_CSV,
            FICTITIOUS_GRAZES_MANIFEST
        ),
        Err(AspectsError::GrazeEventCount { .. })
    ));
}

#[test]
fn mean_subset_fails_a_graze_manifest_row_count_drift() {
    let (rows, checksum) = parse_manifest(LUNAR_POINT_GRAZES_MANIFEST).unwrap();
    let drifted = format!(
        "slice aspects-lunar-points-grazes file=lunar-points-grazes.csv role=grazes rows={} checksum={checksum}\n",
        rows - 1
    );
    assert!(matches!(
        validate_subset(
            ASTEROID_CSV,
            ASTEROID_MANIFEST,
            LUNAR_POINT_CSV,
            LUNAR_POINT_MANIFEST,
            LUNAR_POINT_GRAZES_CSV,
            &drifted,
            FICTITIOUS_CSV,
            FICTITIOUS_MANIFEST,
            FICTITIOUS_GRAZES_CSV,
            FICTITIOUS_GRAZES_MANIFEST
        ),
        Err(AspectsError::ManifestDrift { .. })
    ));
}

#[test]
#[ignore = "slow: full lunar-point scan"]
fn the_full_report_lists_graze_windows_on_lunar_lines() {
    let report = validate_scoped(
        LUNAR_POINT_CSV,
        LUNAR_POINT_MANIFEST,
        Scope::LunarPoints,
        Some((LUNAR_POINT_GRAZES_CSV, LUNAR_POINT_GRAZES_MANIFEST)),
    )
    .expect("lunar-point gate passes");
    assert!(report
        .pair_lines()
        .iter()
        .all(|line| line.contains("graze windows ")));
}

// No missing-event test is repeated for this corpus: the count path is covered by
// `a_corpus_missing_an_event_fails_the_count` and the `compare_exact` tests, while
// scanning a true-node pair costs minutes at the 0.25 d step.

#[test]
fn lunar_point_scope_floor_is_its_own() {
    assert_eq!(Scope::LunarPoints.floor(), MIN_ROWS_VALIDATED_LUNAR_POINTS);
    assert!(Scope::LunarPoints.includes(Group::Geo));
    assert_eq!(Scope::LunarPoints.pairs().len(), LUNAR_POINT_PAIRS.len());
    let (rows, _) = parse_manifest(LUNAR_POINT_MANIFEST).unwrap();
    let windows = parse_grazes(LUNAR_POINT_GRAZES_CSV, &LUNAR_POINT_PAIRS).unwrap();
    let in_windows: usize = windows.iter().map(|w| w.se_events).sum();
    assert_eq!(
        rows - in_windows,
        MIN_ROWS_VALIDATED_LUNAR_POINTS,
        "every lunar-point row outside a graze window is compared"
    );
}

#[test]
fn full_report_keeps_the_planet_summary_prefix() {
    let planets = AspectsReport {
        rows_validated: 2,
        pair_lines: vec!["geo Sun-Moon: …".into()],
        summary_line: "Aspects gate: 2 exact aspects validated".into(),
    };
    let asteroids = AspectsReport {
        rows_validated: 3,
        pair_lines: vec!["geo Sun-Ceres: …".into()],
        summary_line: "Asteroid aspects: 3 exact aspects validated".into(),
    };
    let merged = planets.merged_with(asteroids);
    assert_eq!(merged.rows_validated, 5);
    assert_eq!(merged.pair_lines, ["geo Sun-Moon: …", "geo Sun-Ceres: …"]);
    assert!(merged
        .summary_line
        .starts_with("Aspects gate: 2 exact aspects validated; "));
    assert!(merged
        .summary_line
        .ends_with("Asteroid aspects: 3 exact aspects validated"));
}

fn window(angle: usize, lo: f64, hi: f64) -> GrazeWindow {
    GrazeWindow {
        pair: 0,
        angle,
        lo,
        hi,
        se_events: 0,
    }
}

/// `n` events at 60 degrees, spaced 0.1 day from `start`, first body ahead.
fn sextiles_from(start: f64, n: usize) -> Vec<Expected> {
    (0..n)
        .map(|i| expected(start + 0.1 * i as f64, 60.0, 0.0, 0.5))
        .collect()
}

#[test]
fn a_graze_window_accepts_any_even_split() {
    let w = [GrazeWindow {
        se_events: 2,
        ..window(1, 10.0, 12.0)
    }];
    for (got, want) in [(0, 2), (2, 0), (2, 2), (4, 2), (0, 0)] {
        let mut corpus = sextiles_from(10.5, want);
        corpus.push(expected(20.0, 60.0, 0.0, 0.5));
        let mut engine = engine_of(&sextiles_from(10.5, got));
        engine.push(found(20.0, 60.0, 0.0));
        let (eng_out, cor_out, stats) =
            split_windows("t", &w, &engine, &corpus).expect("even split is accepted");
        assert_eq!(eng_out, vec![found(20.0, 60.0, 0.0)]);
        assert_eq!(cor_out.len(), 1);
        assert_eq!(stats.windows, 1);
        assert_eq!(stats.differing, usize::from(got != want));
        let compared = if got == want { got } else { 0 };
        assert_eq!(stats.compared, compared);
    }
}

#[test]
fn a_nan_residual_inside_a_graze_window_is_reported_not_dropped() {
    let w = [GrazeWindow {
        se_events: 2,
        ..window(1, 10.0, 12.0)
    }];
    let corpus = sextiles_from(10.5, 2);
    let mut engine = engine_of(&corpus);
    engine[1].first_lon_deg = f64::NAN;
    engine[1].second_lon_deg = f64::NAN;
    let (_, _, stats) = split_windows("t", &w, &engine, &corpus).expect("report-only");
    assert_eq!(stats.compared, 2);
    assert!(stats.max_lon_arcsec.is_nan());
}

#[test]
fn a_count_of_the_wrong_parity_in_a_graze_window_fails() {
    // An even window (se_events 2): an odd count on either side fails.
    let even = [GrazeWindow {
        se_events: 2,
        ..window(1, 10.0, 12.0)
    }];
    for (got, want) in [(1, 2), (2, 1), (3, 0)] {
        let corpus = sextiles_from(10.5, want);
        let engine = engine_of(&sextiles_from(10.5, got));
        assert!(matches!(
            split_windows("t", &even, &engine, &corpus),
            Err(AspectsError::GrazeWindowParity { .. })
        ));
    }
    // An odd window (se_events 1, edges straddling the level): an even count fails.
    let odd = [GrazeWindow {
        se_events: 1,
        ..window(1, 10.0, 12.0)
    }];
    for (got, want) in [(2, 1), (0, 1), (1, 2)] {
        let corpus = sextiles_from(10.5, want);
        let engine = engine_of(&sextiles_from(10.5, got));
        assert!(matches!(
            split_windows("t", &odd, &engine, &corpus),
            Err(AspectsError::GrazeWindowParity { .. })
        ));
    }
}

#[test]
fn an_odd_window_accepts_any_odd_split() {
    let w = [GrazeWindow {
        se_events: 1,
        ..window(1, 10.0, 12.0)
    }];
    for (got, want) in [(1, 1), (3, 1), (1, 3)] {
        let corpus = sextiles_from(10.5, want);
        let engine = engine_of(&sextiles_from(10.5, got));
        let (_, _, stats) =
            split_windows("t", &w, &engine, &corpus).expect("odd split in an odd window");
        assert_eq!(stats.differing, usize::from(got != want));
    }
}

#[test]
fn an_event_just_outside_a_window_stays_strict() {
    let w = [window(1, 10.0, 12.0)];
    let corpus = vec![expected(12.000_001, 60.0, 0.0, 0.5)];
    let (eng_out, cor_out, stats) = split_windows("t", &w, &[], &corpus).unwrap();
    assert!(eng_out.is_empty());
    assert_eq!(cor_out.len(), 1, "an event past hi is not excused");
    assert_eq!(stats.windows, 1);
    let ceilings = Ceilings {
        sep_arcsec: 1.0,
        lon_arcsec: 1.0,
    };
    assert!(matches!(
        compare_exact("t", 60.0, &eng_out, &cor_out, ceilings),
        Err(AspectsError::CountMismatch { .. })
    ));
    // Both window edges are inside.
    let edges = vec![
        expected(10.0, 60.0, 0.0, 0.5),
        expected(12.0, 60.0, 0.0, 0.5),
    ];
    let (_, cor_out, _) = split_windows("t", &w, &[], &edges).unwrap();
    assert!(cor_out.is_empty());
}

#[test]
fn equal_window_counts_report_residuals_without_ceilings() {
    let w = [window(1, 10.0, 12.0)];
    let corpus = sextiles_from(10.5, 2);
    // 600 s late and 40" off in longitude: far past any ceiling, still accepted.
    let engine: Vec<Found> = corpus
        .iter()
        .map(|e| found(e.found.jd + 600.0 / 86_400.0, 60.0 + 40.0 / 3600.0, 0.0))
        .collect();
    let (_, _, stats) = split_windows("t", &w, &engine, &corpus).unwrap();
    assert_eq!(stats.compared, 2);
    assert!((stats.max_time_s - 600.0).abs() < 1e-3);
    assert!((stats.max_lon_arcsec - 40.0).abs() < 1e-6);
    assert!(stats.max_sep_arcsec > 0.0);
}

#[test]
fn two_windows_of_one_series_are_checked_separately() {
    let w = [
        GrazeWindow {
            se_events: 2,
            ..window(1, 10.0, 12.0)
        },
        GrazeWindow {
            se_events: 2,
            ..window(1, 30.0, 32.0)
        },
    ];
    let mut corpus = sextiles_from(10.5, 2);
    corpus.extend(sextiles_from(30.5, 2));
    // 0 in the first window, 3 in the second: the second fails on its own.
    let engine = engine_of(&sextiles_from(30.5, 3));
    assert!(matches!(
        split_windows("t", &w, &engine, &corpus),
        Err(AspectsError::GrazeWindowParity { .. })
    ));
    let engine = engine_of(&sextiles_from(30.5, 2));
    let (_, _, stats) = split_windows("t", &w, &engine, &corpus).unwrap();
    assert_eq!((stats.windows, stats.differing, stats.compared), (2, 1, 2));
}

#[test]
fn a_window_applies_only_to_its_own_series() {
    // A 60-degree window of pair 0 must not excuse a 90-degree row, or a row of
    // pair 1, at the same instant.
    let rows = [
        Row {
            pair: 0,
            angle: 1,
            expected: expected(10.5, 60.0, 0.0, 0.5),
        },
        Row {
            pair: 0,
            angle: 1,
            expected: expected(10.6, 60.0, 0.0, 0.5),
        },
        Row {
            pair: 0,
            angle: 2,
            expected: expected(10.55, 90.0, 0.0, 0.5),
        },
        Row {
            pair: 1,
            angle: 1,
            expected: expected(10.55, 60.0, 0.0, 0.5),
        },
    ];
    let windows = [GrazeWindow {
        pair: 0,
        angle: 1,
        lo: 10.0,
        hi: 12.0,
        se_events: 2,
    }];
    let of_series = super::grazes::windows_of(&windows, 0, 2);
    assert!(of_series.is_empty());
    let of_series = super::grazes::windows_of(&windows, 1, 1);
    assert!(of_series.is_empty());
    assert_eq!(super::grazes::windows_of(&windows, 0, 1).len(), 1);
    assert!(super::grazes::check_window_counts(&windows, &rows).is_ok());
}

fn grazes_csv(rows: &[&str]) -> String {
    let mut csv = String::from(
        "# test\ngroup,first,second,angle_deg,turn_jd_tt,miss_arcsec,lo_jd_tt,hi_jd_tt,se_events\n",
    );
    for row in rows {
        csv.push_str(row);
        csv.push('\n');
    }
    csv
}

#[test]
fn graze_rows_parse_and_reject_bad_windows() {
    let ok = grazes_csv(&[
        "geo,TrueApogee,Jupiter,60,2420750.8200000,1.840,2420750.5000000,2420751.1000000,2",
    ]);
    let windows = parse_grazes(&ok, &LUNAR_POINT_PAIRS).expect("parses");
    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0].angle, 1);
    assert_eq!(windows[0].se_events, 2);
    for bad in [
        // lo >= hi
        "geo,TrueApogee,Jupiter,60,2420750.8,1.8,2420751.1,2420750.5,2",
        // turn outside the window
        "geo,TrueApogee,Jupiter,60,2420752.0,1.8,2420750.5,2420751.1,2",
        // unknown pair
        "geo,TrueApogee,Mars,60,2420750.8,1.8,2420750.5,2420751.1,2",
        // angle not in the plan
        "geo,TrueApogee,Jupiter,45,2420750.8,1.8,2420750.5,2420751.1,2",
        // wrong field count
        "geo,TrueApogee,Jupiter,60,2420750.8,1.8,2420750.5,2420751.1",
    ] {
        assert!(
            matches!(
                parse_grazes(&grazes_csv(&[bad]), &LUNAR_POINT_PAIRS),
                Err(AspectsError::MalformedGrazes(_))
            ),
            "{bad}"
        );
    }
    let overlapping = grazes_csv(&[
        "geo,TrueApogee,Jupiter,60,2420750.8,1.8,2420750.5,2420751.1,2",
        "geo,TrueApogee,Jupiter,60,2420751.0,2.0,2420750.9,2420751.4,0",
    ]);
    assert!(matches!(
        parse_grazes(&overlapping, &LUNAR_POINT_PAIRS),
        Err(AspectsError::MalformedGrazes(_))
    ));
}

#[test]
fn a_window_whose_se_events_disagree_with_the_corpus_fails() {
    let rows = [
        Row {
            pair: 0,
            angle: 1,
            expected: expected(10.5, 60.0, 0.0, 0.5),
        },
        Row {
            pair: 0,
            angle: 1,
            expected: expected(10.6, 60.0, 0.0, 0.5),
        },
    ];
    let windows = [GrazeWindow {
        pair: 0,
        angle: 1,
        lo: 10.0,
        hi: 12.0,
        se_events: 0,
    }];
    assert!(matches!(
        super::grazes::check_window_counts(&windows, &rows),
        Err(AspectsError::GrazeEventCount {
            recorded: 0,
            corpus: 2,
            ..
        })
    ));
}
