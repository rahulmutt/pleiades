//! comparison core, tolerance, body-class, and report tests (white-box; moved verbatim from the former `tests.rs`).

use super::*;
use pleiades_core::{CoordinateFrame, JulianDay, Latitude, TimeScale};

#[test]
fn comparison_report_uses_the_snapshot_backend() {
    let report = render_comparison_report().expect("comparison should render");
    assert_eq!(report.lines().next(), Some("Comparison report"));
    assert!(report.lines().any(|line| line == "Comparison corpus"));
    assert!(report
        .lines()
        .any(|line| line == "  name: JPL Horizons comparison window"));
    assert!(report.lines().any(|line| {
            line == "  description: Source-backed comparison corpus built from the checked-in JPL Horizons snapshot across a small set of reference epochs, restricted to the bodies shared by the algorithmic comparison backend."
        }));
    assert!(report.lines().any(|line| line == "  Apparentness: Mean"));
    assert!(report.lines().any(|line| {
        line.starts_with("  epoch labels: JD 2415020.5 (TT)")
            && line.contains("JD 2451545.0 (TT)")
            && line.contains("JD 2453000.5 (TT)")
    }));
    assert!(report
        .lines()
        .any(|line| line == "  julian day span: 2415020.5 → 2453000.5"));
    assert!(report
        .lines()
        .any(|line| line == "Reference backend: jpl-snapshot"));
    assert!(report
        .lines()
        .any(|line| line == "Candidate backend: composite:pleiades-vsop87+pleiades-elp"));
}

#[test]
fn comparison_tolerance_policy_summary_matches_the_rendered_line() {
    let corpus = release_grade_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let summary = report.tolerance_policy_summary();

    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(
        summary.summary_line(),
        format_comparison_tolerance_policy_for_report(&report)
    );
    assert_eq!(
        summary
            .validated_summary_line()
            .expect("summary should validate"),
        summary.summary_line()
    );
    assert!(summary.summary_line().contains("frames=Ecliptic"));
    assert_eq!(summary.coverage.len(), summary.entries.len());
    assert_eq!(summary.comparison_body_count, report.body_summaries().len());
    assert!(summary.coverage.iter().any(|coverage| coverage.entry.scope
        == ComparisonToleranceScope::Pluto
        && coverage.body_count == 0
        && coverage.sample_count == 0));
    assert!(summary.coverage.iter().all(|coverage| coverage.entry.scope
        != ComparisonToleranceScope::Pluto
        || coverage.bodies.is_empty()));
    assert_eq!(summary.comparison_sample_count, report.summary.sample_count);
    assert_eq!(
        summary.comparison_window.start,
        corpus.summary().epochs.first().copied()
    );
    assert_eq!(
        summary.comparison_window.end,
        corpus.summary().epochs.last().copied()
    );
}

#[test]
fn comparison_tolerance_policy_summary_validated_summary_line_rejects_drift() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let mut summary = report.tolerance_policy_summary();
    summary.comparison_sample_count += 1;

    let error = summary
        .validated_summary_line()
        .expect_err("summary should reject drifted counts");
    assert!(error.to_string().contains("sample-count mismatch"));
}

#[test]
fn comparison_tolerance_policy_summary_renderer_falls_back_when_the_report_fails() {
    let rendered = render_comparison_tolerance_policy_summary_text_from_report(Err(
        "comparison report construction failed".to_string(),
    ));

    assert_eq!(
            rendered,
            "Comparison tolerance policy summary\nComparison tolerance policy unavailable (comparison report construction failed)\n"
        );
}

#[test]
fn pluto_fallback_summary_renderer_falls_back_when_the_report_fails() {
    let rendered = render_pluto_fallback_summary_text_from_report(Err(
        "comparison report construction failed".to_string(),
    ));

    assert_eq!(
            rendered,
            "Pluto fallback summary\nPluto fallback unavailable (comparison report construction failed)\n"
        );
}

#[test]
fn comparison_tolerance_scope_coverage_summary_and_alias_commands_render_the_scope_coverage() {
    let summary = render_cli(&["comparison-tolerance-scope-coverage-summary"])
        .expect("comparison tolerance scope coverage summary should render");
    assert!(summary.contains("Comparison tolerance scope coverage summary"));
    assert!(summary.contains("Scope coverage posture:"));
    assert_eq!(
        summary,
        render_comparison_tolerance_scope_coverage_summary_text()
    );
    assert_eq!(
        render_cli(&["comparison-tolerance-scope-coverage"])
            .expect("comparison tolerance scope coverage alias should render"),
        summary
    );
    assert_eq!(
        render_cli(&["comparison-tolerance-scope-coverage-summary", "extra"]).expect_err(
            "comparison tolerance scope coverage summary should reject extra arguments"
        ),
        "comparison-tolerance-scope-coverage-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-tolerance-scope-coverage", "extra"])
            .expect_err("comparison tolerance scope coverage alias should reject extra arguments"),
        "comparison-tolerance-scope-coverage does not accept extra arguments"
    );
}

#[test]
fn comparison_tolerance_scope_coverage_summary_renderer_fails_closed_on_invalid_rows() {
    let corpus = release_grade_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let mut summary = report.tolerance_policy_summary();
    summary.coverage[0].body_count = summary.coverage[0].bodies.len() + 1;

    let rendered =
        render_comparison_tolerance_scope_coverage_summary_text_from_summary(Ok(summary));

    assert!(rendered.contains("Comparison tolerance scope coverage summary"));
    assert!(rendered.contains("Comparison tolerance scope coverage unavailable"));
    assert!(rendered.contains("body-count mismatch"));
}

#[test]
fn comparison_body_class_tolerance_summary_and_alias_commands_render_the_posture() {
    let summary = render_cli(&["comparison-body-class-tolerance-summary"])
        .expect("comparison body-class tolerance summary should render");
    assert!(summary.contains("Comparison body-class tolerance summary"));
    assert_eq!(
        summary,
        render_comparison_body_class_tolerance_summary_text()
    );
    assert_eq!(
        render_cli(&["comparison-body-class-tolerance"])
            .expect("comparison body-class tolerance alias should render"),
        summary
    );
    assert_eq!(
        render_cli(&["comparison-body-class-tolerance-summary", "extra"])
            .expect_err("comparison body-class tolerance summary should reject extra arguments"),
        "comparison-body-class-tolerance-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-body-class-tolerance", "extra"])
            .expect_err("comparison body-class tolerance alias should reject extra arguments"),
        "comparison-body-class-tolerance does not accept extra arguments"
    );

    let posture = render_cli(&["comparison-body-class-tolerance-posture-summary"])
        .expect("comparison body-class tolerance posture summary should render");
    assert!(posture.contains("Comparison body-class tolerance posture summary"));
    assert_eq!(
        posture,
        render_comparison_body_class_tolerance_posture_summary_text()
    );
    assert_eq!(
        render_cli(&["comparison-body-class-tolerance-posture"])
            .expect("comparison body-class tolerance posture alias should render"),
        posture
    );
    assert_eq!(
        validated_comparison_body_class_tolerance_posture_for_report()
            .expect("comparison body-class tolerance posture helper should validate"),
        format_body_class_tolerance_posture_for_report()
    );
    assert_eq!(
        render_cli(&["comparison-body-class-tolerance-posture-summary", "extra"]).expect_err(
            "comparison body-class tolerance posture summary should reject extra arguments"
        ),
        "comparison-body-class-tolerance-posture-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-body-class-tolerance-posture", "extra"]).expect_err(
            "comparison body-class tolerance posture alias should reject extra arguments"
        ),
        "comparison-body-class-tolerance-posture does not accept extra arguments"
    );
}

#[test]
fn comparison_body_class_error_envelope_summary_and_alias_commands_render_the_envelopes() {
    let summary = render_cli(&["comparison-body-class-error-envelope-summary"])
        .expect("comparison body-class error envelope summary should render");
    assert!(summary.contains("Comparison body-class error envelope summary"));
    assert!(summary.contains("Body-class error envelopes:"));
    assert!(summary.contains("Luminaries"));
    assert_eq!(
        summary,
        render_comparison_body_class_error_envelope_summary_text()
    );
    assert_eq!(
        render_cli(&["comparison-body-class-error-envelope"])
            .expect("comparison body-class error envelope alias should render"),
        summary
    );
    assert_eq!(
        render_cli(&["comparison-body-class-error-envelope-summary", "extra"]).expect_err(
            "comparison body-class error envelope summary should reject extra arguments"
        ),
        "comparison-body-class-error-envelope-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-body-class-error-envelope", "extra"])
            .expect_err("comparison body-class error envelope alias should reject extra arguments"),
        "comparison-body-class-error-envelope does not accept extra arguments"
    );
}

#[test]
fn comparison_body_class_error_envelope_summary_renderer_fails_closed_on_invalid_rows() {
    let corpus = release_grade_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let mut summaries = report.body_class_summaries();
    summaries[0].sample_count = 0;

    let rendered =
        render_comparison_body_class_error_envelope_summary_text_from_summaries(Ok(summaries));

    assert!(rendered.contains("Comparison body-class error envelope summary"));
    assert!(rendered.contains("Comparison body-class error envelope unavailable"));
    assert!(rendered.contains("body-class summary is unavailable"));
}

#[test]
fn comparison_body_class_tolerance_summary_renderer_fails_closed_on_invalid_rows() {
    let corpus = release_grade_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let mut summaries = report.body_class_tolerance_summaries();
    summaries[0].body_count += 1;

    let rendered =
        render_comparison_body_class_tolerance_summary_text_from_summaries(Ok(summaries));

    assert!(rendered.contains("Comparison body-class tolerance summary"));
    assert!(rendered.contains("Comparison body-class tolerance unavailable"));
    assert!(rendered.contains("body-class tolerance summary body-count mismatch"));
}

#[test]
fn comparison_tolerance_catalog_entries_track_the_backend_family_and_scopes() {
    let entries = comparison_tolerance_catalog_entries(&BackendFamily::Algorithmic);

    assert_eq!(entries.len(), 6);
    assert_eq!(entries[0].scope, ComparisonToleranceScope::Luminary);
    assert_eq!(entries[1].scope, ComparisonToleranceScope::MajorPlanet);
    assert_eq!(entries[2].scope, ComparisonToleranceScope::LunarPoint);
    assert_eq!(entries[3].scope, ComparisonToleranceScope::Asteroid);
    assert_eq!(entries[4].scope, ComparisonToleranceScope::Custom);
    assert_eq!(entries[5].scope, ComparisonToleranceScope::Pluto);
    assert!(entries
        .iter()
        .all(|entry| entry.tolerance.backend_family == BackendFamily::Algorithmic));
}

#[test]
fn comparison_tolerance_catalog_entries_use_body_class_specific_limits() {
    let entries = comparison_tolerance_catalog_entries(&BackendFamily::Composite);

    assert_eq!(
        entries[0].summary_line(),
        "Luminaries: Δlon≤7.500°, Δlat≤0.750°, Δdist=0.001 AU"
    );
    assert_eq!(
        entries[1].summary_line(),
        "Major planets: Δlon≤0.010°, Δlat≤0.010°, Δdist=0.001 AU"
    );
    assert_eq!(
        entries[2].summary_line(),
        "Lunar points: Δlon≤0.100°, Δlat≤0.010°, Δdist=0.001 AU"
    );
    assert_eq!(
        entries[5].summary_line(),
        "Pluto fallback (approximate): Δlon≤45.000°, Δlat≤1.000°, Δdist=0.250 AU"
    );
}

#[test]
fn comparison_tolerance_scope_coverage_summary_validated_summary_line_rejects_body_count_drift() {
    let summary = ComparisonToleranceScopeCoverageSummary {
        entry: ComparisonToleranceEntry {
            scope: ComparisonToleranceScope::Luminary,
            tolerance: ComparisonTolerance {
                backend_family: BackendFamily::Algorithmic,
                profile: "test tolerance",
                max_longitude_delta_deg: 0.1,
                max_latitude_delta_deg: 0.2,
                max_distance_delta_au: Some(0.3),
            },
        },
        bodies: vec![CelestialBody::Sun],
        body_count: 2,
        sample_count: 1,
    };

    let error = summary
        .validated_summary_line()
        .expect_err("summary should reject body-count drift");
    assert!(error.to_string().contains("body-count mismatch"));
}

#[test]
fn comparison_tolerance_entry_has_a_validated_summary_line() {
    let entry = ComparisonToleranceEntry {
        scope: ComparisonToleranceScope::Luminary,
        tolerance: ComparisonTolerance {
            backend_family: BackendFamily::Algorithmic,
            profile: "test tolerance",
            max_longitude_delta_deg: 0.1,
            max_latitude_delta_deg: 0.2,
            max_distance_delta_au: Some(0.3),
        },
    };

    assert_eq!(entry.summary_line(), entry.to_string());
    assert_eq!(entry.validated_summary_line(), Ok(entry.summary_line()));
}

#[test]
fn comparison_tolerance_validation_rejects_blank_profile() {
    let error = validate_comparison_tolerance(&ComparisonTolerance {
        backend_family: BackendFamily::Algorithmic,
        profile: "",
        max_longitude_delta_deg: 0.1,
        max_latitude_delta_deg: 0.2,
        max_distance_delta_au: Some(0.3),
    })
    .expect_err("tolerance should reject a blank profile label");
    assert!(error.to_string().contains("must not be blank"));
}

#[test]
fn comparison_tolerance_scope_coverage_summary_has_a_validated_summary_line() {
    let summary = ComparisonToleranceScopeCoverageSummary {
        entry: ComparisonToleranceEntry {
            scope: ComparisonToleranceScope::Luminary,
            tolerance: ComparisonTolerance {
                backend_family: BackendFamily::Algorithmic,
                profile: "test tolerance",
                max_longitude_delta_deg: 0.1,
                max_latitude_delta_deg: 0.2,
                max_distance_delta_au: Some(0.3),
            },
        },
        bodies: vec![CelestialBody::Sun],
        body_count: 1,
        sample_count: 1,
    };

    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
}

#[test]
fn comparison_tolerance_validation_rejects_padded_profile() {
    let error = validate_comparison_tolerance(&ComparisonTolerance {
        backend_family: BackendFamily::Algorithmic,
        profile: " test tolerance ",
        max_longitude_delta_deg: 0.1,
        max_latitude_delta_deg: 0.2,
        max_distance_delta_au: Some(0.3),
    })
    .expect_err("tolerance should reject a padded profile label");
    assert!(error
        .to_string()
        .contains("contains surrounding whitespace"));
}

#[test]
fn comparison_tolerance_policy_summary_validation_rejects_drift() {
    let summary = ComparisonTolerancePolicySummary {
        backend_family: BackendFamily::Algorithmic,
        entries: vec![ComparisonToleranceEntry {
            scope: ComparisonToleranceScope::Luminary,
            tolerance: ComparisonTolerance {
                backend_family: BackendFamily::Algorithmic,
                profile: "test tolerance",
                max_longitude_delta_deg: 0.1,
                max_latitude_delta_deg: 0.2,
                max_distance_delta_au: Some(0.3),
            },
        }],
        coverage: vec![ComparisonToleranceScopeCoverageSummary {
            entry: ComparisonToleranceEntry {
                scope: ComparisonToleranceScope::Luminary,
                tolerance: ComparisonTolerance {
                    backend_family: BackendFamily::Algorithmic,
                    profile: "test tolerance",
                    max_longitude_delta_deg: 0.1,
                    max_latitude_delta_deg: 0.2,
                    max_distance_delta_au: Some(0.3),
                },
            },
            bodies: vec![CelestialBody::Sun],
            body_count: 1,
            sample_count: 1,
        }],
        comparison_body_count: 2,
        comparison_sample_count: 1,
        comparison_window: TimeRange::new(None, None),
        coordinate_frames: vec![CoordinateFrame::Ecliptic],
    };

    let error = summary
        .validate()
        .expect_err("summary should reject body-count drift");
    assert!(error.to_string().contains("body-count mismatch"));
}

#[test]
fn comparison_tolerance_policy_summary_validation_rejects_invalid_comparison_window() {
    let summary = ComparisonTolerancePolicySummary {
        backend_family: BackendFamily::Algorithmic,
        entries: vec![ComparisonToleranceEntry {
            scope: ComparisonToleranceScope::Luminary,
            tolerance: ComparisonTolerance {
                backend_family: BackendFamily::Algorithmic,
                profile: "test tolerance",
                max_longitude_delta_deg: 0.1,
                max_latitude_delta_deg: 0.2,
                max_distance_delta_au: Some(0.3),
            },
        }],
        coverage: vec![ComparisonToleranceScopeCoverageSummary {
            entry: ComparisonToleranceEntry {
                scope: ComparisonToleranceScope::Luminary,
                tolerance: ComparisonTolerance {
                    backend_family: BackendFamily::Algorithmic,
                    profile: "test tolerance",
                    max_longitude_delta_deg: 0.1,
                    max_latitude_delta_deg: 0.2,
                    max_distance_delta_au: Some(0.3),
                },
            },
            bodies: vec![CelestialBody::Sun],
            body_count: 1,
            sample_count: 1,
        }],
        comparison_body_count: 1,
        comparison_sample_count: 1,
        comparison_window: TimeRange::new(
            Some(Instant::new(
                JulianDay::from_days(2_451_546.0),
                TimeScale::Tt,
            )),
            Some(Instant::new(
                JulianDay::from_days(2_451_545.0),
                TimeScale::Tt,
            )),
        ),
        coordinate_frames: vec![CoordinateFrame::Ecliptic],
    };

    let error = summary
        .validate()
        .expect_err("summary should reject an invalid comparison window");
    assert!(error.to_string().contains("invalid comparison window"));
    assert!(error.to_string().contains("must not precede the start"));
}

#[test]
fn comparison_tolerance_policy_summary_validation_rejects_duplicate_coordinate_frames() {
    let summary = ComparisonTolerancePolicySummary {
        backend_family: BackendFamily::Algorithmic,
        entries: vec![ComparisonToleranceEntry {
            scope: ComparisonToleranceScope::Luminary,
            tolerance: ComparisonTolerance {
                backend_family: BackendFamily::Algorithmic,
                profile: "test tolerance",
                max_longitude_delta_deg: 0.1,
                max_latitude_delta_deg: 0.2,
                max_distance_delta_au: Some(0.3),
            },
        }],
        coverage: vec![ComparisonToleranceScopeCoverageSummary {
            entry: ComparisonToleranceEntry {
                scope: ComparisonToleranceScope::Luminary,
                tolerance: ComparisonTolerance {
                    backend_family: BackendFamily::Algorithmic,
                    profile: "test tolerance",
                    max_longitude_delta_deg: 0.1,
                    max_latitude_delta_deg: 0.2,
                    max_distance_delta_au: Some(0.3),
                },
            },
            bodies: vec![CelestialBody::Sun],
            body_count: 1,
            sample_count: 1,
        }],
        comparison_body_count: 1,
        comparison_sample_count: 1,
        comparison_window: TimeRange::new(None, None),
        coordinate_frames: vec![CoordinateFrame::Ecliptic, CoordinateFrame::Ecliptic],
    };

    let error = summary
        .validate()
        .expect_err("summary should reject duplicate coordinate frames");
    assert!(error.to_string().contains("duplicate coordinate frame"));
}

#[test]
fn comparison_tolerance_policy_summary_validation_rejects_duplicate_bodies_across_scopes() {
    let summary = ComparisonTolerancePolicySummary {
        backend_family: BackendFamily::Algorithmic,
        entries: vec![
            ComparisonToleranceEntry {
                scope: ComparisonToleranceScope::Luminary,
                tolerance: ComparisonTolerance {
                    backend_family: BackendFamily::Algorithmic,
                    profile: "test tolerance",
                    max_longitude_delta_deg: 0.1,
                    max_latitude_delta_deg: 0.2,
                    max_distance_delta_au: Some(0.3),
                },
            },
            ComparisonToleranceEntry {
                scope: ComparisonToleranceScope::MajorPlanet,
                tolerance: ComparisonTolerance {
                    backend_family: BackendFamily::Algorithmic,
                    profile: "test tolerance",
                    max_longitude_delta_deg: 0.4,
                    max_latitude_delta_deg: 0.5,
                    max_distance_delta_au: Some(0.6),
                },
            },
        ],
        coverage: vec![
            ComparisonToleranceScopeCoverageSummary {
                entry: ComparisonToleranceEntry {
                    scope: ComparisonToleranceScope::Luminary,
                    tolerance: ComparisonTolerance {
                        backend_family: BackendFamily::Algorithmic,
                        profile: "test tolerance",
                        max_longitude_delta_deg: 0.1,
                        max_latitude_delta_deg: 0.2,
                        max_distance_delta_au: Some(0.3),
                    },
                },
                bodies: vec![CelestialBody::Sun],
                body_count: 1,
                sample_count: 1,
            },
            ComparisonToleranceScopeCoverageSummary {
                entry: ComparisonToleranceEntry {
                    scope: ComparisonToleranceScope::MajorPlanet,
                    tolerance: ComparisonTolerance {
                        backend_family: BackendFamily::Algorithmic,
                        profile: "test tolerance",
                        max_longitude_delta_deg: 0.4,
                        max_latitude_delta_deg: 0.5,
                        max_distance_delta_au: Some(0.6),
                    },
                },
                bodies: vec![CelestialBody::Sun],
                body_count: 1,
                sample_count: 1,
            },
        ],
        comparison_body_count: 2,
        comparison_sample_count: 2,
        comparison_window: TimeRange::new(None, None),
        coordinate_frames: vec![CoordinateFrame::Ecliptic],
    };

    let error = summary
        .validate()
        .expect_err("summary should reject duplicate bodies across scopes");
    assert!(error.to_string().contains("appears in multiple scope rows"));
    assert!(error.to_string().contains("Luminaries"));
    assert!(error.to_string().contains("Major planets"));
}

#[test]
fn comparison_tolerance_entry_has_a_displayable_summary_line() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let summary = report.tolerance_policy_summary();
    let entry = summary
        .entries
        .first()
        .expect("comparison should include at least one tolerance entry");

    assert_eq!(entry.summary_line(), entry.to_string());
    entry
        .validate()
        .expect("reported tolerance entry should validate");
    assert!(entry.summary_line().contains(entry.scope.label()));
    assert!(entry.summary_line().contains("Δlon≤"));
    assert!(entry.summary_line().contains("Δlat≤"));
}

#[test]
fn comparison_tolerance_scope_coverage_summary_has_a_displayable_summary_line() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let summary = report.tolerance_policy_summary();
    let coverage = summary
        .coverage
        .first()
        .expect("comparison should include at least one tolerance coverage row");

    assert_eq!(coverage.summary_line(), coverage.to_string());
    assert!(coverage.summary_line().contains("backend family="));
    assert!(coverage
        .summary_line()
        .contains(coverage.entry.scope.label()));
    assert_eq!(coverage.body_count, coverage.bodies.len());
}

#[test]
fn body_tolerance_summary_has_a_displayable_summary_line() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let tolerance_summaries = report.tolerance_summaries();
    let summary = tolerance_summaries
        .first()
        .expect("comparison should include at least one tolerance summary");

    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(
        summary.validated_summary_line().unwrap(),
        summary.summary_line()
    );
    assert!(summary.summary_line().contains("backend family="));
    assert!(summary.summary_line().contains("status="));
}

#[test]
fn body_tolerance_summary_validate_accepts_the_reported_summary() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let tolerance_summaries = report.tolerance_summaries();
    let summary = tolerance_summaries
        .first()
        .expect("comparison should include at least one tolerance summary");

    summary
        .validate()
        .expect("reported body tolerance summary should validate");
}

#[test]
fn body_tolerance_summary_validate_rejects_margin_drift() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let tolerance_summaries = report.tolerance_summaries();
    let mut summary = tolerance_summaries
        .first()
        .expect("comparison should include at least one tolerance summary")
        .clone();

    summary.longitude_margin_deg += 1.0;
    let error = summary
        .validate()
        .expect_err("mutated body tolerance summary should fail validation");
    assert!(error.to_string().contains("longitude margin"));
}

#[test]
fn body_tolerance_summary_validate_rejects_zero_sample_counts() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let tolerance_summaries = report.tolerance_summaries();
    let mut summary = tolerance_summaries
        .first()
        .expect("comparison should include at least one tolerance summary")
        .clone();

    summary.sample_count = 0;
    let error = summary
        .validate()
        .expect_err("zero-sample body tolerance summary should fail validation");
    assert!(error.to_string().contains("has no samples to compare"));
}

#[test]
fn body_tolerance_summary_validated_summary_line_rejects_drift() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let tolerance_summaries = report.tolerance_summaries();
    let mut summary = tolerance_summaries
        .first()
        .expect("comparison should include at least one tolerance summary")
        .clone();

    summary.sample_count = 0;
    let error = summary
        .validated_summary_line()
        .expect_err("zero-sample body tolerance summary should fail validation");
    assert!(error.to_string().contains("has no samples to compare"));
}

#[test]
fn body_tolerance_summary_validate_rejects_non_finite_metrics() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let tolerance_summaries = report.tolerance_summaries();
    let mut summary = tolerance_summaries
        .first()
        .expect("comparison should include at least one tolerance summary")
        .clone();

    summary.max_latitude_delta_deg = f64::NAN;
    let error = summary
        .validate()
        .expect_err("non-finite body tolerance summary should fail validation");
    assert!(error.to_string().contains("has invalid latitude"));
}

#[test]
fn body_comparison_summary_has_a_displayable_summary_line() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let body_summaries = report.body_summaries();
    assert!(!corpus
        .summary()
        .epochs
        .iter()
        .any(|epoch| epoch.julian_day.days() == 2_451_913.5));
    let summary = body_summaries
        .first()
        .expect("comparison should include at least one body summary");

    assert_eq!(summary.summary_line(), summary.to_string());
    assert!(summary.summary_line().contains("samples="));
    assert!(summary.summary_line().contains("max Δlon="));
}

#[test]
fn body_comparison_summary_validate_accepts_the_reported_body_summary() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let body_summaries = report.body_summaries();
    let summary = body_summaries
        .first()
        .expect("comparison should include at least one body summary");

    summary
        .validate()
        .expect("reported body summary should validate");
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
}

#[test]
fn body_comparison_summary_validate_rejects_inconsistent_distance_fields() {
    let summary = BodyComparisonSummary {
        body: CelestialBody::Sun,
        sample_count: 1,
        max_longitude_delta_body: Some(CelestialBody::Sun),
        max_longitude_delta_deg: 0.1,
        mean_longitude_delta_deg: 0.1,
        rms_longitude_delta_deg: 0.1,
        max_latitude_delta_body: Some(CelestialBody::Sun),
        max_latitude_delta_deg: 0.1,
        mean_latitude_delta_deg: 0.1,
        rms_latitude_delta_deg: 0.1,
        max_distance_delta_body: Some(CelestialBody::Sun),
        max_distance_delta_au: None,
        mean_distance_delta_au: None,
        rms_distance_delta_au: None,
    };

    let error = summary
        .validate()
        .expect_err("mismatched distance fields should fail");
    assert!(error
        .to_string()
        .contains("distance metrics must either all be present or all be absent"));
}

#[test]
fn comparison_percentile_envelope_has_a_displayable_summary_line() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let envelope = comparison_percentile_envelope(&report.samples, 0.95);

    assert_eq!(envelope.summary_line(), envelope.to_string());
    assert!(envelope
        .summary_line()
        .contains("95th percentile absolute deltas:"));
    assert!(envelope.summary_line().contains("longitude"));
    assert!(envelope.summary_line().contains("latitude"));
}

#[test]
fn comparison_median_envelope_has_a_displayable_summary_line() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let envelope =
        comparison_median_envelope(&report.samples).expect("median envelope should exist");

    assert_eq!(envelope.summary_line(), envelope.to_string());
    assert!(envelope.summary_line().contains("median longitude delta:"));
    assert!(envelope.summary_line().contains("median latitude delta:"));
}

#[test]
fn comparison_envelope_summary_has_a_displayable_summary_line() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let envelope = comparison_envelope_summary(&report.summary, &report.samples);

    assert_eq!(envelope.summary_line(), envelope.to_string());
    assert_eq!(
        envelope.validated_summary_line(&report.samples),
        Ok(envelope.summary_line())
    );
    assert_eq!(
        envelope.percentile_line(),
        comparison_percentile_envelope(&report.samples, 0.95).summary_line()
    );
    assert_eq!(
        envelope.validated_percentile_line(&report.samples),
        Ok(envelope.percentile_line())
    );
    assert!(envelope.summary_line().contains("median longitude delta:"));
    assert!(envelope
        .percentile_line()
        .contains("95th percentile absolute deltas:"));
}

#[test]
fn comparison_envelope_summary_rejects_median_drift() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let mut envelope = comparison_envelope_summary(&report.summary, &report.samples);
    envelope.median.longitude_delta_deg += 0.0001;

    let error = envelope
        .validated_summary_line(&report.samples)
        .expect_err("drifted median should fail validation");
    assert!(error
        .to_string()
        .contains("median drifted from the sampled comparison values"));
}

#[test]
fn comparison_envelope_summary_rejects_percentile_drift() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let mut envelope = comparison_envelope_summary(&report.summary, &report.samples);
    envelope.percentile.longitude_delta_deg += 0.0001;

    let error = envelope
        .validated_percentile_line(&report.samples)
        .expect_err("drifted percentile should fail validation");
    assert!(error
        .to_string()
        .contains("percentile drifted from the sampled comparison values"));
}

#[test]
fn comparison_median_and_percentile_envelopes_validate_their_fields() {
    let median = ComparisonMedianEnvelope {
        longitude_delta_deg: 1.25,
        latitude_delta_deg: 0.75,
        distance_delta_au: Some(0.03125),
    };
    let percentile = ComparisonPercentileEnvelope {
        longitude_delta_deg: 2.5,
        latitude_delta_deg: 1.5,
        distance_delta_au: Some(0.0625),
    };

    assert_eq!(median.validate(), Ok(()));
    assert_eq!(percentile.validate(), Ok(()));
    assert_eq!(median.summary_line(), median.to_string());
    assert_eq!(percentile.summary_line(), percentile.to_string());

    let invalid_median = ComparisonMedianEnvelope {
        longitude_delta_deg: -1.0,
        ..median
    };
    let median_error = invalid_median
        .validate()
        .expect_err("negative median deltas should fail validation");
    assert!(median_error
        .to_string()
        .contains("comparison median envelope field `longitude_delta_deg`"));

    let invalid_percentile = ComparisonPercentileEnvelope {
        distance_delta_au: Some(-0.5),
        ..percentile
    };
    let percentile_error = invalid_percentile
        .validate()
        .expect_err("negative percentile deltas should fail validation");
    assert!(percentile_error
        .to_string()
        .contains("comparison percentile envelope field `distance_delta_au`"));
}

#[test]
fn comparison_tail_envelope_is_publicly_reusable() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let envelope = comparison_tail_envelope(&report.samples).expect("tail envelope should exist");

    assert_eq!(envelope.summary_line(), envelope.to_string());
    assert_eq!(
        envelope.summary_line(),
        format_comparison_percentile_envelope_for_report(&report.samples)
    );
}

#[test]
fn regression_finding_has_a_displayable_summary_line() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison should build");
    let notable_regressions = report.notable_regressions();
    let finding = notable_regressions
        .first()
        .expect("comparison should include at least one notable regression");

    assert_eq!(finding.summary_line(), finding.to_string());
    assert_eq!(finding.validated_summary_line(), Ok(finding.summary_line()));
    assert!(finding.summary_line().contains("Δlon="));
    assert!(finding.summary_line().contains("Δlat="));
    assert!(finding.summary_line().contains("Δdist="));
}

#[test]
fn regression_finding_validated_summary_line_rejects_blank_notes() {
    let finding = RegressionFinding {
        body: CelestialBody::Mars,
        longitude_delta_deg: 0.25,
        latitude_delta_deg: 0.15,
        distance_delta_au: Some(0.01),
        note: "  ".to_string(),
    };

    let error = finding
        .validated_summary_line()
        .expect_err("blank regression notes should fail validation");
    assert!(error
        .to_string()
        .contains("regression finding note must not be blank"));
}

#[test]
fn comparison_audit_summary_has_a_displayable_summary_line() {
    let summary = ComparisonAuditSummary {
        body_count: 10,
        within_tolerance_body_count: 4,
        outside_tolerance_body_count: 6,
        regression_count: 12,
    };

    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(
            summary.summary_line(),
            "status=regressions found, bodies checked=10, within tolerance bodies=4, outside tolerance bodies=6, notable regressions=12"
        );
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(summary.validate(), Ok(()));
}

#[test]
fn comparison_audit_summary_validate_rejects_body_count_mismatch() {
    let summary = ComparisonAuditSummary {
        body_count: 10,
        within_tolerance_body_count: 4,
        outside_tolerance_body_count: 5,
        regression_count: 0,
    };

    let error = summary
        .validated_summary_line()
        .expect_err("mismatched audit counts should fail validation");
    assert!(error
        .to_string()
        .contains("comparison audit summary body-count mismatch"));
}

#[test]
fn comparison_audit_summary_validate_rejects_empty_body_counts() {
    let summary = ComparisonAuditSummary {
        body_count: 0,
        within_tolerance_body_count: 0,
        outside_tolerance_body_count: 0,
        regression_count: 0,
    };

    let error = summary
        .validate()
        .expect_err("an empty audit summary should fail validation");
    assert!(error
        .to_string()
        .contains("must include at least one compared body"));
}

#[test]
fn comparison_report_exposes_a_public_audit_summary() {
    let corpus = default_corpus();
    let reference = default_reference_backend();
    let candidate = default_candidate_backend();
    let report =
        compare_backends(&reference, &candidate, &corpus).expect("comparison report should build");
    let summary = report.comparison_audit_summary();

    assert_eq!(summary.summary_line(), summary.to_string());
    assert!(summary.validate().is_ok());
    assert_eq!(summary.body_count, report.tolerance_summaries().len());
    assert_eq!(summary.regression_count, report.notable_regressions().len());
}

#[test]
fn comparison_report_alias_renders_the_comparison_report() {
    let alias = render_cli(&["comparison-report"]).expect("comparison report should render");
    let command = render_cli(&["compare-backends"]).expect("compare-backends should render");

    assert_eq!(alias, command);
    assert!(alias.contains("Comparison report"));
    assert_eq!(
        render_cli(&["comparison-report", "extra"]).unwrap_err(),
        "comparison-report does not accept extra arguments"
    );
}

#[test]
fn comparison_audit_command_reports_clean_release_grade_corpus() {
    let report = render_cli(&["compare-backends-audit"]).expect("comparison audit should render");
    assert!(report.contains("Comparison tolerance audit"));
    assert!(report.contains("comparison corpus"));
    assert!(report.contains("julian day span:"));
    assert!(report.contains("Body-class error envelopes"));
    assert!(report.contains("rms longitude delta:"));
    assert!(report.contains("rms latitude delta:"));
    assert!(report.contains("rms distance delta:"));
    assert!(report.contains("Body-class tolerance posture"));
    assert!(report.contains("body-class tolerance posture:"));
    assert!(report.contains("outlier bodies: none"));
    assert!(report.contains("Tolerance policy"));
    assert!(report.contains("Notable regressions\n  none"));
    assert!(report.contains("regression bodies: none"));
    assert!(report.contains("Pluto fallback (approximate): backend family=composite, profile=phase-1 Pluto approximate fallback evidence, bodies=0 (none), samples=0"));

    let summary =
        render_cli(&["comparison-audit-summary"]).expect("comparison audit summary should render");
    assert!(summary.contains("status="));
    assert!(summary.contains("bodies checked="));
    assert!(summary.contains("within tolerance bodies="));
    assert!(summary.contains("outside tolerance bodies="));
    assert!(summary.contains("notable regressions="));
    assert_eq!(
        summary,
        render_cli(&["comparison-audit"]).expect("comparison audit alias should render")
    );
    assert_eq!(
        render_cli(&["comparison-audit-summary", "extra"]).unwrap_err(),
        "comparison-audit-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["comparison-audit", "extra"]).unwrap_err(),
        "comparison-audit does not accept extra arguments"
    );
}

#[test]
fn comparison_report_uses_release_grade_corpus_without_pluto() {
    let corpus = release_grade_corpus();
    let report = compare_backends(
        &default_reference_backend(),
        &default_candidate_backend(),
        &corpus,
    )
    .expect("comparison should succeed");

    let regressions = report.notable_regressions();
    assert!(regressions.is_empty());

    let body_summaries = report.body_summaries();
    assert_eq!(body_summaries.len(), report.tolerance_summaries().len());
    assert!(body_summaries
        .iter()
        .all(|summary| summary.sample_count > 0 && summary.body != CelestialBody::Pluto));
    assert!(body_summaries
        .iter()
        .any(|summary| summary.body == CelestialBody::Jupiter
            && summary.max_longitude_delta_deg > 0.0
            && summary.max_longitude_delta_deg < 0.01));

    let archive = report.regression_archive();
    assert_eq!(archive.corpus_name, corpus.name);
    assert!(archive.cases.is_empty());
    let tolerance_summaries = report.tolerance_summaries();
    assert!(tolerance_summaries.iter().any(|summary| {
        summary.body == CelestialBody::Jupiter
            && summary.tolerance.profile.contains("full-file VSOP87B")
    }));
    assert!(tolerance_summaries
        .iter()
        .all(|summary| summary.body != CelestialBody::Pluto));
    let tolerance_policy_entries = report.tolerance_policy_entries();
    assert_eq!(tolerance_policy_entries.len(), 6);
    assert!(tolerance_policy_entries
        .iter()
        .any(|entry| entry.scope == ComparisonToleranceScope::Luminary));
    assert!(tolerance_policy_entries
        .iter()
        .any(|entry| entry.scope == ComparisonToleranceScope::Pluto));
    let body_class_tolerance_summaries = report.body_class_tolerance_summaries();
    assert!(body_class_tolerance_summaries.iter().any(|summary| {
        summary.class == BodyClass::MajorPlanet
            && summary.body_count >= 1
            && summary.sample_count >= summary.body_count
            && summary.outside_tolerance_body_count == 0
            && !summary.outside_bodies.contains(&CelestialBody::Pluto)
            && summary.max_longitude_delta_body.is_some()
            && summary.max_latitude_delta_body.is_some()
            && summary.max_distance_delta_body.is_some()
    }));

    let rendered = report.to_string();
    assert!(rendered.contains("Body comparison summaries"));
    assert!(rendered.contains("Body-class tolerance posture"));
    assert!(rendered.contains("Expected tolerance status"));
    assert!(rendered.contains("phase-1 full-file VSOP87B planetary evidence"));
    assert!(rendered.contains("Notable regressions"));
}

#[test]
fn body_class_summary_line_reuses_the_typed_formatter() {
    let summary = BodyClassSummary {
        class: BodyClass::MajorPlanet,
        sample_count: 1,
        max_longitude_delta_body: Some(CelestialBody::Mars),
        max_longitude_delta_deg: 1.234,
        sum_longitude_delta_deg: 1.0,
        sum_longitude_delta_sq_deg: 1.0,
        median_longitude_delta_deg: 1.0,
        percentile_longitude_delta_deg: 1.0,
        max_latitude_delta_body: Some(CelestialBody::Mars),
        max_latitude_delta_deg: 0.5,
        sum_latitude_delta_deg: 0.5,
        sum_latitude_delta_sq_deg: 0.25,
        median_latitude_delta_deg: 0.5,
        percentile_latitude_delta_deg: 0.5,
        max_distance_delta_body: Some(CelestialBody::Mars),
        max_distance_delta_au: Some(2.5),
        sum_distance_delta_au: 2.5,
        sum_distance_delta_sq_au: 6.25,
        distance_count: 1,
        median_distance_delta_au: Some(2.5),
        percentile_distance_delta_au: Some(2.5),
    };

    let expected = "samples=1, max Δlon=1.234000000000° (Mars), mean Δlon=1.000000000000°, median Δlon=1.000000000000°, 95th percentile longitude delta: 1.000000000000°, rms Δlon=1.000000000000°, max Δlat=0.500000000000° (Mars), mean Δlat=0.500000000000°, median Δlat=0.500000000000°, 95th percentile latitude delta: 0.500000000000°, rms Δlat=0.500000000000°, max Δdist=2.500000000000 AU (Mars), mean Δdist=2.500000000000 AU, median Δdist=2.500000000000 AU, 95th percentile distance delta: 2.500000000000 AU, rms Δdist=2.500000000000 AU";

    assert_eq!(summary.summary_line(), expected);
    assert_eq!(summary.validated_summary_line(), Ok(expected.to_string()));
    assert_eq!(summary.to_string(), expected);
    assert_eq!(
        format_body_class_comparison_envelope_for_report(&summary),
        expected
    );
}

#[test]
fn body_class_summary_validation_rejects_drift() {
    let mut summary = BodyClassSummary {
        class: BodyClass::MajorPlanet,
        sample_count: 1,
        max_longitude_delta_body: Some(CelestialBody::Mars),
        max_longitude_delta_deg: 1.234,
        sum_longitude_delta_deg: 1.0,
        sum_longitude_delta_sq_deg: 1.0,
        median_longitude_delta_deg: 1.0,
        percentile_longitude_delta_deg: 1.0,
        max_latitude_delta_body: Some(CelestialBody::Mars),
        max_latitude_delta_deg: 0.5,
        sum_latitude_delta_deg: 0.5,
        sum_latitude_delta_sq_deg: 0.25,
        median_latitude_delta_deg: 0.5,
        percentile_latitude_delta_deg: 0.5,
        max_distance_delta_body: Some(CelestialBody::Mars),
        max_distance_delta_au: Some(2.5),
        sum_distance_delta_au: 2.5,
        sum_distance_delta_sq_au: 6.25,
        distance_count: 1,
        median_distance_delta_au: Some(2.5),
        percentile_distance_delta_au: Some(2.5),
    };

    summary.max_latitude_delta_body = None;

    assert_eq!(
        summary.validate(),
        Err(BodyClassSummaryValidationError::FieldOutOfSync {
            class: BodyClass::MajorPlanet
        })
    );
    assert_eq!(
        summary.validated_summary_line(),
        Err(BodyClassSummaryValidationError::FieldOutOfSync {
            class: BodyClass::MajorPlanet
        })
    );
}

#[test]
fn body_class_tolerance_summary_reuses_the_typed_formatter() {
    let summary = BodyClassToleranceSummary {
        class: BodyClass::MajorPlanet,
        tolerance: ComparisonTolerance {
            backend_family: BackendFamily::ReferenceData,
            profile: "phase-1 body-class tolerance",
            max_longitude_delta_deg: 1.5,
            max_latitude_delta_deg: 0.5,
            max_distance_delta_au: Some(3.0),
        },
        body_count: 2,
        sample_count: 2,
        within_tolerance_body_count: 1,
        outside_tolerance_body_count: 1,
        outside_tolerance_sample_count: 1,
        max_longitude_delta_body: Some(CelestialBody::Mars),
        max_longitude_delta_deg: Some(1.0),
        max_latitude_delta_body: Some(CelestialBody::Jupiter),
        max_latitude_delta_deg: Some(0.25),
        max_distance_delta_body: Some(CelestialBody::Saturn),
        max_distance_delta_au: Some(2.5),
        sum_longitude_delta_deg: 2.0,
        sum_longitude_delta_sq_deg: 2.0,
        sum_latitude_delta_deg: 0.5,
        sum_latitude_delta_sq_deg: 0.125,
        sum_distance_delta_au: 3.0,
        sum_distance_delta_sq_au: 4.5,
        distance_count: 2,
        median_longitude_delta_deg: 1.0,
        percentile_longitude_delta_deg: 1.0,
        median_latitude_delta_deg: 0.25,
        percentile_latitude_delta_deg: 0.25,
        median_distance_delta_au: Some(1.5),
        percentile_distance_delta_au: Some(1.5),
        outside_bodies: vec![CelestialBody::Mars],
    };

    let expected = "Major planets: backend family=reference data, profile=phase-1 body-class tolerance, bodies=2, samples=2, within tolerance bodies=1, outside tolerance bodies=1, limit Δlon≤1.500000°, margin Δlon=+0.500000000000°, limit Δlat≤0.500000°, margin Δlat=+0.250000000000°, limit Δdist=3.000000 AU, margin Δdist=+0.500000000000 AU, max Δlon=1.000000000000° (Mars), max Δlat=0.250000000000° (Jupiter), max Δdist=2.500000000000 AU (Saturn)";

    assert!(summary.validate().is_ok());
    assert_eq!(summary.summary_line(), expected);
    assert_eq!(summary.to_string(), expected);
    assert_eq!(
        format_body_class_tolerance_envelope_for_report(&summary),
        expected
    );
}

#[test]
fn body_class_tolerance_summary_rejects_count_drift() {
    let summary = BodyClassToleranceSummary {
        class: BodyClass::MajorPlanet,
        tolerance: ComparisonTolerance {
            backend_family: BackendFamily::ReferenceData,
            profile: "phase-1 body-class tolerance",
            max_longitude_delta_deg: 1.5,
            max_latitude_delta_deg: 0.5,
            max_distance_delta_au: Some(3.0),
        },
        body_count: 1,
        sample_count: 1,
        within_tolerance_body_count: 1,
        outside_tolerance_body_count: 0,
        outside_tolerance_sample_count: 0,
        max_longitude_delta_body: Some(CelestialBody::Mars),
        max_longitude_delta_deg: Some(1.0),
        max_latitude_delta_body: Some(CelestialBody::Jupiter),
        max_latitude_delta_deg: Some(0.25),
        max_distance_delta_body: Some(CelestialBody::Saturn),
        max_distance_delta_au: Some(2.5),
        sum_longitude_delta_deg: 1.0,
        sum_longitude_delta_sq_deg: 1.0,
        sum_latitude_delta_deg: 0.25,
        sum_latitude_delta_sq_deg: 0.0625,
        sum_distance_delta_au: 2.5,
        sum_distance_delta_sq_au: 6.25,
        distance_count: 1,
        median_longitude_delta_deg: 1.0,
        percentile_longitude_delta_deg: 1.0,
        median_latitude_delta_deg: 0.25,
        percentile_latitude_delta_deg: 0.25,
        median_distance_delta_au: Some(2.5),
        percentile_distance_delta_au: Some(2.5),
        outside_bodies: vec![CelestialBody::Mars, CelestialBody::Jupiter],
    };

    let rendered = format_body_class_tolerance_envelope_for_report(&summary);
    assert!(rendered.contains("body-class tolerance envelope unavailable"));
    assert!(summary.validate().is_err());
}

#[test]
fn comparison_summary_summary_line_includes_bodies_and_counts() {
    let summary = ComparisonSummary {
        sample_count: 3,
        max_longitude_delta_body: Some(CelestialBody::Sun),
        max_longitude_delta_deg: 0.123_456_789_012,
        mean_longitude_delta_deg: 0.012_345_678_901,
        rms_longitude_delta_deg: 0.023_456_789_012,
        max_latitude_delta_body: Some(CelestialBody::Moon),
        max_latitude_delta_deg: 0.223_456_789_012,
        mean_latitude_delta_deg: 0.032_345_678_901,
        rms_latitude_delta_deg: 0.043_456_789_012,
        max_distance_delta_body: Some(CelestialBody::Mars),
        max_distance_delta_au: Some(0.001_234_567_89),
        mean_distance_delta_au: Some(0.000_234_567_89),
        rms_distance_delta_au: Some(0.000_334_567_89),
    };

    let rendered = summary.summary_line();
    assert!(rendered.contains("samples: 3"));
    assert!(rendered.contains("max longitude delta: 0.123456789012° (Sun)"));
    assert!(rendered.contains("max latitude delta: 0.223456789012° (Moon)"));
    assert!(rendered.contains("max distance delta: 0.001234567890 AU (Mars)"));
    assert_eq!(rendered, format!("{summary}"));
    assert_eq!(summary.validated_summary_line(), Ok(rendered));
    assert!(summary.validate().is_ok());
}

#[test]
fn comparison_summary_validated_summary_line_rejects_zero_sample_drift() {
    let summary = ComparisonSummary {
        sample_count: 0,
        max_longitude_delta_body: Some(CelestialBody::Sun),
        max_longitude_delta_deg: 0.123_456_789_012,
        mean_longitude_delta_deg: 0.012_345_678_901,
        rms_longitude_delta_deg: 0.023_456_789_012,
        max_latitude_delta_body: Some(CelestialBody::Moon),
        max_latitude_delta_deg: 0.223_456_789_012,
        mean_latitude_delta_deg: 0.032_345_678_901,
        rms_latitude_delta_deg: 0.043_456_789_012,
        max_distance_delta_body: Some(CelestialBody::Mars),
        max_distance_delta_au: Some(0.001_234_567_89),
        mean_distance_delta_au: Some(0.000_234_567_89),
        rms_distance_delta_au: Some(0.000_334_567_89),
    };

    let error = summary
        .validated_summary_line()
        .expect_err("zero-sample drift should fail validation");
    assert!(error.to_string().contains(
        "comparison summary with zero samples must not carry per-body or distance extrema"
    ));
}

#[test]
fn comparison_envelope_formatter_rejects_empty_sample_slices() {
    let summary = ComparisonSummary {
        sample_count: 1,
        max_longitude_delta_body: Some(CelestialBody::Sun),
        max_longitude_delta_deg: 0.123_456_789_012,
        mean_longitude_delta_deg: 0.012_345_678_901,
        rms_longitude_delta_deg: 0.023_456_789_012,
        max_latitude_delta_body: Some(CelestialBody::Moon),
        max_latitude_delta_deg: 0.223_456_789_012,
        mean_latitude_delta_deg: 0.032_345_678_901,
        rms_latitude_delta_deg: 0.043_456_789_012,
        max_distance_delta_body: Some(CelestialBody::Mars),
        max_distance_delta_au: Some(0.001_234_567_89),
        mean_distance_delta_au: Some(0.000_234_567_89),
        rms_distance_delta_au: Some(0.000_334_567_89),
    };

    let envelope = format_comparison_envelope_for_report(&summary, &[]);
    assert!(envelope.contains("comparison envelope unavailable"));
    assert!(envelope.contains("sample-count mismatch") || envelope.contains("no samples"));

    let percentile = format_comparison_percentile_envelope_for_report(&[]);
    assert!(percentile.contains("comparison percentile envelope unavailable"));
    assert!(percentile.contains("comparison sample slice is empty"));
}

#[test]
fn comparison_envelope_formatter_rejects_mixed_distance_channels() {
    let summary = ComparisonSummary {
        sample_count: 2,
        max_longitude_delta_body: Some(CelestialBody::Sun),
        max_longitude_delta_deg: 0.123_456_789_012,
        mean_longitude_delta_deg: 0.012_345_678_901,
        rms_longitude_delta_deg: 0.023_456_789_012,
        max_latitude_delta_body: Some(CelestialBody::Moon),
        max_latitude_delta_deg: 0.223_456_789_012,
        mean_latitude_delta_deg: 0.032_345_678_901,
        rms_latitude_delta_deg: 0.043_456_789_012,
        max_distance_delta_body: Some(CelestialBody::Mars),
        max_distance_delta_au: Some(0.001_234_567_89),
        mean_distance_delta_au: Some(0.000_234_567_89),
        rms_distance_delta_au: Some(0.000_334_567_89),
    };
    let samples = vec![
        ComparisonSample {
            body: CelestialBody::Sun,
            reference: EclipticCoordinates::new(
                Longitude::from_degrees(10.0),
                Latitude::from_degrees(1.0),
                Some(1.0),
            ),
            candidate: EclipticCoordinates::new(
                Longitude::from_degrees(10.1),
                Latitude::from_degrees(1.1),
                Some(1.1),
            ),
            longitude_delta_deg: 0.1,
            latitude_delta_deg: 0.1,
            distance_delta_au: Some(0.1),
        },
        ComparisonSample {
            body: CelestialBody::Moon,
            reference: EclipticCoordinates::new(
                Longitude::from_degrees(20.0),
                Latitude::from_degrees(2.0),
                None,
            ),
            candidate: EclipticCoordinates::new(
                Longitude::from_degrees(20.2),
                Latitude::from_degrees(2.2),
                None,
            ),
            longitude_delta_deg: 0.2,
            latitude_delta_deg: 0.2,
            distance_delta_au: None,
        },
    ];

    let envelope = format_comparison_envelope_for_report(&summary, &samples);
    assert!(
        envelope.contains("comparison envelope unavailable")
            || envelope.contains("distance deltas must either all be present or all be absent")
    );
}

#[test]
fn body_class_coverage_summary_commands_render_the_matching_blocks() {
    let production_generation = render_cli(&["production-generation-body-class-coverage-summary"])
        .expect("production generation body-class coverage summary should render");
    assert!(production_generation.contains("Production generation body-class coverage:"));
    let production_generation_alias = render_cli(&["production-body-class-coverage-summary"])
        .expect("production body-class coverage summary alias should render");
    assert_eq!(production_generation_alias, production_generation);
    assert_eq!(
        production_generation,
        production_generation_snapshot_body_class_coverage_summary_for_report()
    );

    let comparison = render_cli(&["comparison-snapshot-body-class-coverage-summary"])
        .expect("comparison snapshot body-class coverage summary should render");
    assert!(comparison.contains("Comparison snapshot body-class coverage:"));
    let comparison_alias = render_cli(&["comparison-body-class-coverage-summary"])
        .expect("comparison body-class coverage summary alias should render");
    assert_eq!(comparison_alias, comparison);
    assert_eq!(
        comparison,
        comparison_snapshot_body_class_coverage_summary_for_report()
    );

    let reference = render_cli(&["reference-snapshot-body-class-coverage-summary"])
        .expect("reference snapshot body-class coverage summary should render");
    assert!(reference.contains("Reference snapshot body-class coverage:"));
    let reference_alias = render_cli(&["reference-body-class-coverage-summary"])
        .expect("reference body-class coverage summary alias should render");
    assert_eq!(reference_alias, reference);
    assert_eq!(
        reference,
        reference_snapshot_body_class_coverage_summary_for_report()
    );

    let independent_holdout_source_window =
        render_cli(&["independent-holdout-source-window-summary"])
            .expect("independent hold-out source window summary should render");
    assert!(independent_holdout_source_window.contains("Independent hold-out source windows:"));
    assert!(independent_holdout_source_window.contains("source-backed samples"));
    assert_eq!(
        independent_holdout_source_window,
        independent_holdout_snapshot_source_window_summary_for_report()
    );

    let independent_holdout_quarter_day_boundary =
        render_cli(&["independent-holdout-quarter-day-boundary-summary"])
            .expect("independent hold-out quarter-day boundary summary should render");
    assert!(independent_holdout_quarter_day_boundary
        .contains("Independent hold-out quarter-day boundary samples:"));
    assert!(independent_holdout_quarter_day_boundary.contains("quarter-day boundary samples"));
    assert_eq!(
        independent_holdout_quarter_day_boundary,
        independent_holdout_snapshot_quarter_day_boundary_summary_for_report()
    );
    assert_eq!(
        render_cli(&["independent-holdout-quarter-day-boundary"])
            .expect("independent hold-out quarter-day boundary alias should render"),
        independent_holdout_quarter_day_boundary
    );

    let independent_holdout = render_cli(&["independent-holdout-summary"])
        .expect("independent hold-out summary should render");
    assert!(independent_holdout.contains("JPL independent hold-out:"));
    assert!(independent_holdout.contains("transparency evidence only"));
    assert_eq!(
        independent_holdout,
        jpl_independent_holdout_summary_for_report()
    );

    let independent_holdout_source = render_cli(&["independent-holdout-source-summary"])
        .expect("independent hold-out source summary should render");
    assert!(independent_holdout_source.contains("Independent hold-out source:"));
    assert!(independent_holdout_source.contains("hold-out source"));
    assert_eq!(
        independent_holdout_source,
        independent_holdout_source_summary_for_report()
    );

    let independent_holdout_high_curvature =
        render_cli(&["independent-holdout-high-curvature-summary"])
            .expect("independent hold-out high-curvature summary should render");
    assert!(independent_holdout_high_curvature
        .contains("JPL independent hold-out high-curvature evidence:"));
    assert!(independent_holdout_high_curvature.contains("high-curvature interpolation window"));
    assert_eq!(
        independent_holdout_high_curvature,
        independent_holdout_high_curvature_summary_for_report()
    );

    let holdout = render_cli(&["independent-holdout-body-class-coverage-summary"])
        .expect("independent hold-out body-class coverage summary should render");
    assert!(holdout.contains("Independent hold-out body-class coverage:"));
    let holdout_alias = render_cli(&["holdout-body-class-coverage-summary"])
        .expect("holdout body-class coverage summary alias should render");
    assert_eq!(holdout_alias, holdout);
    assert_eq!(
        holdout,
        independent_holdout_snapshot_body_class_coverage_summary_for_report()
    );
}
