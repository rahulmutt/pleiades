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

// Opt-in: the planet-pair gate measured 18.5 minutes on 2026-10-02 (the asteroid pass adds about 41 s in release), which nightly
// `test-full` cannot afford. `mise run gate-aspects` runs it (its own nightly
// job and a `release-gate` dependency).
#[test]
fn aspects_gate_passes_within_ceilings() {
    if std::env::var("PLEIADES_FULL_ASPECTS_GATE").as_deref() != Ok("1") {
        eprintln!(
            "aspects_gate_passes_within_ceilings: skipped; set PLEIADES_FULL_ASPECTS_GATE=1 to run the 18-minute full gate"
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
    assert!(report.rows_validated >= MIN_ROWS_VALIDATED + MIN_ROWS_VALIDATED_ASTEROIDS);
    assert_eq!(
        report.pair_lines().len(),
        PAIRS.len() + ASTEROID_PAIRS.len()
    );
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
        validate_scoped(&tampered, MANIFEST, Scope::MeanSubset),
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
        validate_scoped(CORPUS_CSV, &manifest, Scope::MeanSubset),
        Err(AspectsError::ManifestDrift { .. })
    ));
}

#[test]
fn a_corpus_missing_an_event_fails_the_count() {
    let line = first_mean_mars_saturn_conjunction();
    let without = CORPUS_CSV.replacen(&format!("{line}\n"), "", 1);
    assert_ne!(without, CORPUS_CSV);
    match validate_scoped(&without, &manifest_for(&without), Scope::MeanSubset) {
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
    match validate_scoped(&shifted, &manifest_for(&shifted), Scope::MeanSubset) {
        Err(AspectsError::CeilingExceeded { series, kind, .. }) => {
            assert_eq!(series, "mean Mars-Saturn 0");
            assert_eq!(kind, "separation_arcsec");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn asteroid_corpus_parses_into_the_five_asteroid_pairs() {
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
            "Moon-Ceres"
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

#[test]
fn asteroid_corpus_tampering_fails_closed() {
    let tampered = ASTEROID_CSV.replacen("geo,Sun,Ceres,", "geo,Sun,Ceres, ", 1);
    assert_ne!(tampered, ASTEROID_CSV);
    assert!(matches!(
        validate_scoped(&tampered, ASTEROID_MANIFEST, Scope::Asteroids),
        Err(AspectsError::ChecksumMismatch { .. })
    ));
    let (rows, checksum) = parse_manifest(ASTEROID_MANIFEST).unwrap();
    let drifted = format!(
        "slice aspects-asteroids file=asteroids.csv role=aspects rows={} checksum={checksum}\n",
        rows + 1
    );
    assert!(matches!(
        validate_scoped(ASTEROID_CSV, &drifted, Scope::Asteroids),
        Err(AspectsError::ManifestDrift { .. })
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
