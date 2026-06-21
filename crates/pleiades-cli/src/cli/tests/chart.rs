//! Tests for the benchmark command and the chart command (rendering, time-scale conversions,
//! ayanamsa, houses, asteroids) and parse helpers (parse_ayanamsa, parse_body).

use pleiades_core::{Angle, Ayanamsa, CelestialBody, CustomAyanamsa, CustomBodyId, JulianDay};

use crate::cli::render_cli;
use crate::commands::chart::render_chart;
use crate::parse::{parse_ayanamsa, parse_body};

#[test]
fn benchmark_command_renders_a_report() {
    let rendered = render_cli(&["benchmark", "--rounds", "1"]).expect("benchmark should render");
    assert!(rendered.contains("Benchmark report"));
    assert!(rendered.contains("Summary: backend="));
}

#[test]
fn benchmark_command_rejects_duplicate_rounds_arguments() {
    let error = render_cli(&["benchmark", "--rounds", "1", "--rounds", "2"])
        .expect_err("benchmark should reject duplicate rounds arguments");
    assert!(error.contains("duplicate value for --rounds argument"));
}

#[test]
fn benchmark_command_rejects_extra_arguments() {
    let error = render_cli(&["benchmark", "--rounds", "1", "extra"])
        .expect_err("benchmark should reject extra arguments");
    assert_eq!(error, "unknown argument: extra");
}

#[test]
fn chart_command_renders_bodies() {
    let rendered = render_chart(&["--jd", "2451545.0", "--body", "Sun", "--body", "Moon"])
        .expect("chart should render");
    assert!(rendered.contains("Backend:"));
    assert!(rendered.contains("Sun"));
    assert!(rendered.contains("Moon"));
    assert!(rendered.contains("Apparentness: Mean"));
    assert!(rendered.contains("Sign summary:"));
}

#[test]
fn chart_command_rejects_apparent_positions_until_supported() {
    let error = render_chart(&["--jd", "2451545.0", "--apparent", "--body", "Sun"])
        .expect_err("current first-party backends should reject apparent requests");
    assert!(error.contains("UnsupportedApparentness"));
    assert!(error.contains("mean-state") || error.contains("mean geometric"));
}

#[test]
fn chart_command_renders_aspect_information() {
    let rendered = render_chart(&["--jd", "2451545.0", "--body", "Sun", "--body", "Moon"])
        .expect("chart should render");
    assert!(rendered.contains("Aspect summary: 1 Sextile"));
    assert!(rendered.contains("Aspects:"));
    assert!(rendered.contains("Sun Sextile Moon"));
}

#[test]
fn chart_command_can_convert_utc_to_tt_with_caller_supplied_delta_t() {
    let rendered = render_chart(&[
        "--jd",
        "2451545.0",
        "--utc",
        "--tt-offset-seconds",
        "64.184",
        "--body",
        "Sun",
    ])
    .expect("UTC chart should convert to TT with an explicit offset");
    assert!(rendered.contains("Instant: JD 2451545"));
    assert!(rendered.contains("(TT)"));
    assert!(rendered.contains("Sun"));
}

#[test]
fn chart_command_can_convert_utc_to_tt_with_explicit_alias() {
    let rendered = render_chart(&[
        "--jd",
        "2451545.0",
        "--utc",
        "--tt-from-utc-offset-seconds",
        "64.184",
        "--body",
        "Sun",
    ])
    .expect("UTC chart should convert to TT with the explicit UTC alias");
    assert!(rendered.contains("Instant: JD 2451545"));
    assert!(rendered.contains("(TT)"));
    assert!(rendered.contains("Sun"));
}

#[test]
fn chart_command_can_convert_ut1_to_tt_with_explicit_alias() {
    let rendered = render_chart(&[
        "--jd",
        "2451545.0",
        "--ut1",
        "--tt-from-ut1-offset-seconds",
        "64.184",
        "--body",
        "Sun",
    ])
    .expect("UT1 chart should convert to TT with the explicit UT1 alias");
    assert!(rendered.contains("Instant: JD 2451545"));
    assert!(rendered.contains("(TT)"));
    assert!(rendered.contains("Sun"));
}

#[test]
fn chart_command_can_render_tdb_tagged_instant() {
    let rendered = render_chart(&["--jd", "2451545.0", "--tdb", "--body", "Sun"])
        .expect("chart should render with a TDB-tagged instant");
    assert!(
        rendered.contains("Instant: JD 2451545 (TDB)")
            || rendered.contains("Instant: JD 2451545.0 (TDB)")
    );
    assert!(rendered.contains("Apparentness: Mean"));
}

#[test]
fn chart_command_can_convert_tdb_to_tt_with_signed_offset() {
    let rendered = render_chart(&[
        "--jd",
        "2451545.0",
        "--tdb",
        "--tt-from-tdb-offset-seconds",
        "-0.001657",
        "--body",
        "Sun",
    ])
    .expect("TDB-tagged chart should accept a signed TT-TDB offset");
    assert!(rendered.contains("Instant: JD"));
    assert!(rendered.contains("(TT)"));
}

#[test]
fn chart_command_can_convert_tt_to_tdb_with_explicit_tt_source_offset() {
    let rendered = render_chart(&[
        "--jd",
        "2451545.0",
        "--tt",
        "--tdb-from-tt-offset-seconds",
        "-0.001657",
        "--body",
        "Sun",
    ])
    .expect("TT-tagged chart should accept an explicit TT-to-TDB offset flag");
    assert!(rendered.contains("Instant: JD"));
    assert!(rendered.contains("(TDB)"));
}

#[test]
fn chart_command_can_convert_utc_to_tdb_with_explicit_alias() {
    let rendered = render_chart(&[
        "--jd",
        "2451545.0",
        "--utc",
        "--tt-offset-seconds",
        "64.184",
        "--tdb-from-utc-offset-seconds",
        "-0.001657",
        "--body",
        "Sun",
    ])
    .expect("UTC-tagged chart should accept an explicit UTC-to-TDB alias");
    assert!(rendered.contains("Instant: JD"));
    assert!(rendered.contains("(TDB)"));
}

#[test]
fn chart_command_can_convert_ut1_to_tdb_with_explicit_alias() {
    let rendered = render_chart(&[
        "--jd",
        "2451545.0",
        "--ut1",
        "--tt-offset-seconds",
        "64.184",
        "--tdb-from-ut1-offset-seconds",
        "-0.001657",
        "--body",
        "Sun",
    ])
    .expect("UT1-tagged chart should accept an explicit UT1-to-TDB alias");
    assert!(rendered.contains("Instant: JD"));
    assert!(rendered.contains("(TDB)"));
}

#[test]
fn chart_command_rejects_conflicting_tdb_offset_aliases() {
    let error = render_chart(&[
        "--jd",
        "2451545.0",
        "--tt",
        "--tdb-offset-seconds",
        "-0.001657",
        "--tdb-from-tt-offset-seconds",
        "-0.001657",
        "--body",
        "Sun",
    ])
    .expect_err("TT-tagged chart requests should reject conflicting TDB-TT aliases");
    assert!(error.contains("conflicting TDB-TT offset flags"));
}

#[test]
fn chart_command_rejects_conflicting_tdb_offset_aliases_in_either_order() {
    let error = render_chart(&[
        "--jd",
        "2451545.0",
        "--tt",
        "--tdb-from-tt-offset-seconds",
        "-0.001657",
        "--tdb-offset-seconds",
        "-0.001657",
        "--body",
        "Sun",
    ])
    .expect_err(
        "TT-tagged chart requests should reject conflicting TDB-TT aliases regardless of order",
    );
    assert!(error.contains("conflicting TDB-TT offset flags"));
}

#[test]
fn chart_command_rejects_repeated_tt_offset_flags() {
    let error = render_chart(&[
        "--jd",
        "2451545.0",
        "--utc",
        "--tt-offset-seconds",
        "64.184",
        "--tt-offset-seconds",
        "65.0",
        "--body",
        "Sun",
    ])
    .expect_err("UTC-tagged chart requests should reject duplicate TT offset flags");
    assert!(error.contains("conflicting TT offset flags"));
}

#[test]
fn chart_command_rejects_repeated_tt_offset_aliases() {
    let error = render_chart(&[
        "--jd",
        "2451545.0",
        "--utc",
        "--tt-offset-seconds",
        "64.184",
        "--tt-from-utc-offset-seconds",
        "65.0",
        "--body",
        "Sun",
    ])
    .expect_err("UTC-tagged chart requests should reject duplicate TT offset aliases");
    assert!(error.contains("conflicting TT offset flags"));
}

#[test]
fn chart_command_rejects_repeated_tdb_offset_flags() {
    let error = render_chart(&[
        "--jd",
        "2451545.0",
        "--tt",
        "--tdb-offset-seconds",
        "-0.001657",
        "--tdb-offset-seconds",
        "-0.002",
        "--body",
        "Sun",
    ])
    .expect_err("TT-tagged chart requests should reject duplicate TDB-TT offset flags");
    assert!(error.contains("conflicting TDB-TT offset flags"));
}

#[test]
fn chart_command_rejects_repeated_time_scale_flags_even_when_the_same_scale_is_reused() {
    let error = render_chart(&["--jd", "2451545.0", "--tt", "--tt", "--body", "Sun"])
        .expect_err("chart requests should reject duplicate time-scale tags");
    assert!(error.contains("conflicting time-scale flags"));
}

#[test]
fn chart_command_rejects_repeated_apparentness_flags_even_when_the_same_mode_is_reused() {
    let error = render_chart(&["--jd", "2451545.0", "--mean", "--mean", "--body", "Sun"])
        .expect_err("chart requests should reject duplicate apparentness tags");
    assert!(error.contains("conflicting apparentness flags"));
}

#[test]
fn chart_command_rejects_tdb_offsets_for_tdb_tagged_instants() {
    let error = render_chart(&[
        "--jd",
        "2451545.0",
        "--tdb",
        "--tdb-offset-seconds",
        "-0.001657",
        "--body",
        "Sun",
    ])
    .expect_err("TDB-tagged chart requests should reject a caller-supplied TDB-TT offset");
    assert!(error.contains("--tdb-offset-seconds"));
}

#[test]
fn chart_command_rejects_tdb_from_tt_offsets_for_utc_tagged_instants() {
    let error = render_chart(&[
        "--jd",
        "2451545.0",
        "--utc",
        "--tt-offset-seconds",
        "64.184",
        "--tdb-from-tt-offset-seconds",
        "-0.001657",
        "--body",
        "Sun",
    ])
    .expect_err("UTC-tagged chart requests should reject the TT-only TDB offset alias");
    assert!(error.contains("--tdb-from-tt-offset-seconds"));
}

#[test]
fn chart_command_can_convert_tt_to_tdb_with_signed_offset() {
    let rendered = render_chart(&[
        "--jd",
        "2451545.0",
        "--tt",
        "--tdb-offset-seconds",
        "-0.001657",
        "--body",
        "Sun",
    ])
    .expect("TT-tagged chart should accept a signed TDB-TT offset");
    assert!(rendered.contains("Instant: JD"));
    assert!(rendered.contains("(TDB)"));
}

#[test]
fn chart_command_can_convert_utc_to_tdb_with_signed_offset() {
    let rendered = render_chart(&[
        "--jd",
        "2451545.0",
        "--utc",
        "--tt-offset-seconds",
        "64.184",
        "--tdb-offset-seconds",
        "-0.001657",
        "--body",
        "Sun",
    ])
    .expect("UTC chart should accept a signed TDB-TT offset");
    assert!(rendered.contains("Instant: JD 2451545"));
    assert!(rendered.contains("(TDB)"));
}

#[test]
fn chart_command_rejects_tt_offsets_for_tt_tagged_instants() {
    let error = render_chart(&[
        "--jd",
        "2451545.0",
        "--tt-offset-seconds",
        "64.184",
        "--body",
        "Sun",
    ])
    .expect_err("TT-tagged chart requests should reject a caller-supplied TT offset");
    assert!(error.contains("--tt-offset-seconds"));
}

#[test]
fn chart_command_rejects_tt_offsets_for_tdb_tagged_instants() {
    let error = render_chart(&[
        "--jd",
        "2451545.0",
        "--tdb",
        "--tt-offset-seconds",
        "64.184",
        "--body",
        "Sun",
    ])
    .expect_err("TDB-tagged chart requests should reject a caller-supplied TT offset");
    assert!(error.contains("--tt-offset-seconds"));
}

#[test]
fn chart_command_rejects_tdb_retagging_offsets_for_utc_tagged_instants() {
    let error = render_chart(&[
        "--jd",
        "2451545.0",
        "--utc",
        "--tt-offset-seconds",
        "64.184",
        "--tt-from-tdb-offset-seconds",
        "-0.001657",
        "--body",
        "Sun",
    ])
    .expect_err("UTC-tagged chart requests should reject a TDB-to-TT retagging offset");
    assert!(error.contains("--tt-from-tdb-offset-seconds"));
}

#[test]
fn chart_command_can_convert_ut1_to_tdb_with_caller_supplied_offsets() {
    let rendered = render_chart(&[
        "--jd",
        "2451545.0",
        "--ut1",
        "--tt-offset-seconds",
        "64.184",
        "--tdb-offset-seconds",
        "0.001657",
        "--body",
        "Sun",
    ])
    .expect("UT1 chart should convert to TDB with explicit offsets");
    assert!(rendered.contains("Instant: JD 2451545"));
    assert!(rendered.contains("(TDB)"));
    assert!(rendered.contains("Sun"));
}

#[test]
fn chart_command_accepts_sidereal_ayanamsa() {
    let rendered = render_chart(&["--jd", "2451545.0", "--ayanamsa", "Lahiri", "--body", "Sun"])
        .expect("sidereal chart should render");
    assert!(rendered.contains("Sidereal"));
    assert!(rendered.contains("Lahiri"));
}

#[test]
fn chart_command_can_render_house_information() {
    let rendered = render_chart(&[
        "--jd",
        "2451545.0",
        "--lat",
        "0.0",
        "--lon",
        "0.0",
        "--house-system",
        "Whole Sign",
        "--body",
        "Sun",
    ])
    .expect("house-aware chart should render");
    assert!(rendered.contains("House system:"));
    assert!(rendered.contains("House cusps:"));
    assert!(rendered.contains("Sun"));
    assert!(rendered.contains(" 1:"));
}

#[test]
fn chart_command_accepts_custom_ayanamsa_definitions() {
    for (label, offset) in [
        ("True Balarama", 12.5),
        ("Aphoric", -3.25),
        ("Takra", 0.125),
    ] {
        let ayanamsa = format!("custom:{label}|2451545.0|{offset}");
        let rendered = render_chart(&[
            "--jd",
            "2451545.0",
            "--ayanamsa",
            &ayanamsa,
            "--body",
            "Sun",
        ])
        .expect("custom ayanamsa chart should render");

        assert!(rendered.contains("Sidereal"));
        assert!(rendered.contains(label));
        assert!(rendered.contains(&offset.to_string()));
        assert!(rendered.contains("Custom ayanamsa definition supplied via the CLI"));
    }
}

#[test]
fn parse_ayanamsa_accepts_custom_definition_labels() {
    for (label, offset) in [
        ("True Balarama", 12.5),
        ("Aphoric", -3.25),
        ("Takra", 0.125),
    ] {
        let definition = format!("custom-definition:{label}|2451545.0|{offset}");
        let custom = parse_ayanamsa(&definition).expect("custom ayanamsa should parse");

        assert_eq!(
            custom,
            Ayanamsa::Custom(CustomAyanamsa {
                name: label.to_owned(),
                description: Some("Custom ayanamsa definition supplied via the CLI".to_owned()),
                epoch: Some(JulianDay::from_days(2_451_545.0)),
                offset_degrees: Some(Angle::from_degrees(offset)),
            })
        );
    }
}

#[test]
fn parse_ayanamsa_rejects_padded_custom_definition_names() {
    let error =
        parse_ayanamsa("custom: True Balarama|2451545.0|12.5").expect_err("padding should fail");
    assert_eq!(
        error,
        "custom ayanamsa name must not have leading or trailing whitespace"
    );
}

#[test]
fn chart_command_routes_selected_asteroids_via_jpl_fallback() {
    let rendered = render_chart(&["--jd", "2451545.0", "--body", "Ceres"])
        .expect("asteroid chart should render");
    assert!(rendered.contains("Ceres"));
    assert!(rendered.contains("Backend:"));
}

#[test]
fn parse_body_accepts_lunar_apogee_and_perigee_labels() {
    assert_eq!(
        parse_body(Some("mean apogee")).unwrap(),
        CelestialBody::MeanApogee
    );
    assert_eq!(
        parse_body(Some("true apogee")).unwrap(),
        CelestialBody::TrueApogee
    );
    assert_eq!(
        parse_body(Some("mean perigee")).unwrap(),
        CelestialBody::MeanPerigee
    );
    assert_eq!(
        parse_body(Some("true perigee")).unwrap(),
        CelestialBody::TruePerigee
    );
}

#[test]
fn parse_body_accepts_custom_catalog_designations() {
    let body = parse_body(Some("asteroid:433-Eros")).expect("custom body should parse");
    assert_eq!(
        body,
        CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
    );
    assert_eq!(body.to_string(), "asteroid:433-Eros");
}

#[test]
fn civil_flag_parses_and_reports_provenance() {
    let out = render_chart(&[
        "--civil",
        "2017-01-01T00:00:00",
        "--civil-scale",
        "utc",
        "--body",
        "sun",
    ])
    .expect("chart renders");
    assert!(
        out.contains("path=utc-leap-second"),
        "missing provenance: {out}"
    );
    assert!(out.contains("quality=exact"), "missing quality: {out}");
}

#[test]
fn civil_flag_conflicts_with_jd() {
    let err = render_chart(&["--civil", "2017-01-01T00:00:00", "--jd", "2451545.0"])
        .expect_err("should reject mixing --civil with --jd");
    assert!(err.contains("--civil"));
}

#[test]
fn parse_body_rejects_padded_custom_catalog_designations() {
    let error = parse_body(Some("asteroid: 433-Eros")).expect_err("padding should fail");
    assert_eq!(
        error,
        "custom body id designation must not have leading or trailing whitespace"
    );
}

#[test]
fn parse_body_accepts_lunar_nodes() {
    assert_eq!(
        parse_body(Some("mean node")).unwrap(),
        CelestialBody::MeanNode
    );
    assert_eq!(
        parse_body(Some("mean lunar node")).unwrap(),
        CelestialBody::MeanNode
    );
    assert_eq!(
        parse_body(Some("true node")).unwrap(),
        CelestialBody::TrueNode
    );
    assert_eq!(
        parse_body(Some("true lunar node")).unwrap(),
        CelestialBody::TrueNode
    );
}
