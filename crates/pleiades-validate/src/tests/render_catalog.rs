//! house, ayanamsa, and lunar catalog rendering tests (white-box; moved verbatim from the former `tests.rs`).

use super::*;
use pleiades_core::JulianDay;

#[test]
fn house_validation_report_includes_representative_scenarios() {
    let report = house_validation_report();
    assert_eq!(report.scenarios.len(), 9);
    assert!(report
        .scenarios
        .iter()
        .any(|scenario| scenario.label == "Western hemisphere reference chart"));
    assert!(report
        .scenarios
        .iter()
        .any(|scenario| scenario.label == "Southern polar stress chart"));
    assert!(report
        .scenarios
        .iter()
        .any(|scenario| scenario.label == "Northern high-latitude mountain stress chart"));
    assert!(report
        .scenarios
        .iter()
        .any(|scenario| scenario.label == "Southern hemisphere reference chart"));
}

#[test]
fn custom_definition_ayanamsa_labels_summary_command_renders_the_labels() {
    let profile = current_compatibility_profile();
    let rendered = render_cli(&["custom-definition-ayanamsa-labels-summary"])
        .expect("custom-definition ayanamsa labels summary should render");

    assert_eq!(
        render_cli(&["custom-definition-ayanamsa-labels"]).unwrap(),
        rendered
    );
    assert_eq!(
        rendered,
        profile
            .validated_custom_definition_ayanamsa_labels_summary_line()
            .expect("custom-definition ayanamsa labels summary should validate")
    );
    assert_eq!(
        render_cli(&["custom-definition-ayanamsa-labels-summary", "extra"]).unwrap_err(),
        "custom-definition-ayanamsa-labels-summary does not accept extra arguments"
    );
    assert!(rendered.contains("Babylonian (House)"));
    assert!(rendered.contains("Babylonian (True Geoc)"));
}

#[test]
fn release_specific_canonical_name_summary_commands_render_the_labels() {
    let profile = current_compatibility_profile();

    let house_names = render_cli(&["release-house-system-canonical-names-summary"])
        .expect("release-specific house-system canonical names summary should render");
    assert_eq!(
        render_cli(&["release-house-system-canonical-names"]).unwrap(),
        house_names
    );
    assert_eq!(
        house_names,
        format!(
            "Release-specific house-system canonical names: {}",
            profile
                .validated_release_house_system_canonical_names_summary_line()
                .expect("release-specific house-system canonical names should validate")
        )
    );
    assert_eq!(
        render_cli(&["release-house-system-canonical-names-summary", "extra"]).unwrap_err(),
        "release-house-system-canonical-names-summary does not accept extra arguments"
    );
    assert!(house_names.contains("Equal (MC)"));
    assert!(house_names.contains("Gauquelin sectors"));

    let ayanamsa_names = render_cli(&["release-ayanamsa-canonical-names-summary"])
        .expect("release-specific ayanamsa canonical names summary should render");
    assert_eq!(
        render_cli(&["release-ayanamsa-canonical-names"]).unwrap(),
        ayanamsa_names
    );
    assert_eq!(
        ayanamsa_names,
        format!(
            "Release-specific ayanamsa canonical names: {}",
            profile
                .validated_release_ayanamsa_canonical_names_summary_line()
                .expect("release-specific ayanamsa canonical names should validate")
        )
    );
    assert_eq!(
        render_cli(&["release-ayanamsa-canonical-names-summary", "extra"]).unwrap_err(),
        "release-ayanamsa-canonical-names-summary does not accept extra arguments"
    );
    assert!(ayanamsa_names.contains("True Citra"));
    assert!(ayanamsa_names.contains("Valens Moon"));
}

#[test]
fn ayanamsa_reference_offsets_summary_command_renders_the_summary() {
    let rendered = render_cli(&["ayanamsa-reference-offsets-summary"])
        .expect("ayanamsa reference offsets summary should render");
    assert_eq!(
        render_cli(&["ayanamsa-reference-offsets"]).unwrap(),
        rendered
    );

    let summary =
        summarize_ayanamsa_reference_offsets().expect("reference offsets summary should validate");
    assert_eq!(
        validated_ayanamsa_reference_offsets_summary_for_report(&summary),
        Ok(summary.to_string())
    );

    assert!(rendered.contains("Ayanamsa reference offsets: representative zero-point examples:"));
    assert!(rendered.contains("Lahiri: epoch=JD 2435553.5; offset=23.245524743°"));
    assert!(rendered.contains("Lahiri (ICRC): epoch=JD 2435553.5; offset=23.25°"));
    assert!(rendered.contains("Lahiri (1940): epoch=JD 2415020; offset=22.445972222222224°"));
    assert!(rendered.contains("Usha Shashi: epoch=JD 2415020.5; offset=18.66096111111111°"));
    assert!(rendered.contains("Raman: epoch=JD 2415020; offset=21.01444°"));
    assert!(rendered.contains("Krishnamurti: epoch=JD 2415020; offset=22.363889°"));
    assert!(rendered.contains("Fagan/Bradley: epoch=JD 2433282.42346; offset=24.042044444°"));
    assert!(rendered.contains("True Chitra: epoch=JD 2435553.5; offset=23.245524743°"));
    assert!(rendered.contains("True Revati: epoch=JD 1926902.658267; offset=0°"));
    assert!(rendered.contains("True Mula: epoch=JD 1805889.671313; offset=0°"));
    assert!(rendered.contains("True Citra: epoch=JD 1825182.87233; offset=50.2567483°"));
    assert!(rendered.contains("DeLuce: epoch=JD 2451545; offset=23.245522556°"));
    assert!(rendered.contains("Yukteshwar: epoch=JD 2451545; offset=22.6288889°"));
    assert!(rendered.contains("PVR Pushya-paksha: epoch=JD 2451545; offset=23°"));
    assert!(rendered.contains("J2000: epoch=JD 2451545; offset=23.85317778°"));
    assert!(rendered.contains("J1900: epoch=JD 2415020; offset=0°"));
    assert!(rendered.contains("B1950: epoch=JD 2433281.5; offset=0°"));
    assert!(rendered.contains("True Pushya: epoch=JD 1855769.248315; offset=0°"));
    assert!(rendered.contains("Udayagiri: epoch=JD 1825235.164583; offset=0°"));
    assert!(rendered.contains("Lahiri (VP285): epoch=JD 1825235.164583; offset=0°"));
    assert!(rendered.contains("Krishnamurti (VP291): epoch=JD 1827424.663554; offset=0°"));
    assert!(rendered.contains("Sheoran: epoch=JD 1789947.090881; offset=0°"));
    assert!(rendered.contains("True Sheoran: epoch="));
    assert!(rendered.contains("Hipparchus: epoch=JD 1674484; offset=-9.333333333333334°"));
    assert!(rendered.contains("Djwhal Khul: epoch=JD 1706703.948006; offset=0°"));
    assert!(rendered.contains("Galactic Center: epoch="));
    assert!(rendered.contains("Galactic Center (Rgilbrand): epoch="));
    assert!(rendered.contains("Galactic Center (Mardyks): epoch="));
    assert!(rendered.contains("Galactic Center (Cochrane): epoch="));
    assert!(rendered.contains("Galactic Center (Mula/Wilhelm): epoch="));
    assert!(rendered.contains("Dhruva Galactic Center (Middle Mula): epoch="));
    assert!(rendered.contains("Galactic Equator (IAU 1958): epoch=JD 1667118.376332; offset=0°"));
    assert!(rendered.contains("Galactic Equator (True): epoch=JD 1665728.603158; offset=0°"));
    assert!(rendered.contains("Galactic Equator (Mula): epoch=JD 1840527.426262; offset=0°"));
    assert!(rendered.contains("Galactic Equator (Fiorenza): epoch=JD 2451544.5; offset=25°"));
    assert!(rendered.contains("Valens Moon: epoch=JD 1775845.5; offset=-2.9422°"));
    assert!(rendered.contains("Suryasiddhanta (499 CE): epoch=JD 1903396.8128653935; offset=0°"));
    assert!(rendered.contains("Suryasiddhanta (Mean Sun): epoch=JD 1909045.584433; offset=0°"));
    assert!(rendered.contains("Aryabhata (Mean Sun): epoch=JD 1909650.815331; offset=0°"));
    assert!(rendered.contains("Aryabhata (522 CE): epoch=JD 1911797.740782; offset=0°"));
}

#[test]
fn ayanamsa_provenance_summary_command_renders_the_summary() {
    let rendered = render_cli(&["ayanamsa-provenance-summary"])
        .expect("ayanamsa provenance summary should render");
    assert_eq!(render_cli(&["ayanamsa-provenance"]).unwrap(), rendered);
    assert_eq!(rendered, format_ayanamsa_provenance_for_report());
    assert!(rendered.contains("Ayanamsa provenance: representative provenance examples:"));
    assert!(rendered.contains("True Citra —"));
    assert!(rendered.contains("True Revati —"));
    assert!(rendered.contains("True Mula —"));
    assert!(rendered.contains("True Pushya —"));
    assert!(rendered.contains("Udayagiri —"));
    assert!(rendered.contains("True Sheoran —"));
    assert!(rendered.contains("Babylonian (Britton) —"));
    assert!(rendered.contains("Galactic Center (Rgilbrand) —"));
    assert!(rendered.contains("Babylonian (Kugler 1) —"));
    assert!(rendered.contains("Galactic Equator —"));
    assert!(rendered.contains("Suryasiddhanta (Mean Sun) —"));
    assert!(rendered.contains("Aryabhata (522 CE) —"));
    assert!(rendered.contains("Valens Moon —"));
    assert_eq!(
        render_cli(&["ayanamsa-provenance-summary", "extra"])
            .expect_err("ayanamsa provenance summary should reject extra arguments"),
        "ayanamsa-provenance-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["ayanamsa-provenance", "extra"])
            .expect_err("ayanamsa provenance alias should reject extra arguments"),
        "ayanamsa-provenance does not accept extra arguments"
    );
}

#[test]
fn ayanamsa_audit_summary_command_renders_the_summary() {
    let rendered =
        render_cli(&["ayanamsa-audit-summary"]).expect("ayanamsa audit summary should render");
    assert_eq!(render_cli(&["ayanamsa-audit"]).unwrap(), rendered);
    assert_eq!(rendered, format_ayanamsa_audit_for_report());
    assert!(rendered.contains("Ayanamsa audit: ayanamsa catalog validation:"));
    assert!(rendered.contains("ayanamsa sidereal metadata:"));
    assert!(rendered.contains("Ayanamsa reference offsets:"));
    assert!(rendered.contains("Ayanamsa provenance:"));
    assert_eq!(
        render_cli(&["ayanamsa-audit-summary", "extra"])
            .expect_err("ayanamsa audit summary should reject extra arguments"),
        "ayanamsa-audit-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["ayanamsa-audit", "extra"])
            .expect_err("ayanamsa audit alias should reject extra arguments"),
        "ayanamsa-audit does not accept extra arguments"
    );
}

#[test]
fn ayanamsa_provenance_summary_validated_summary_line_rejects_note_drift() {
    let summary = AyanamsaProvenanceSummary {
        examples: vec![AyanamsaProvenanceExample {
            canonical_name: "Example",
            provenance_note: " drifted note ",
        }],
    };

    assert!(summary.summary_line().contains(" drifted note "));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn ayanamsa_metadata_coverage_summary_command_renders_the_summary() {
    let rendered = render_cli(&["ayanamsa-metadata-coverage-summary"])
        .expect("ayanamsa metadata coverage summary should render");
    assert_eq!(
        render_cli(&["ayanamsa-metadata-coverage"]).unwrap(),
        rendered
    );

    assert_eq!(rendered, metadata_coverage().summary_line());
    assert!(rendered.contains(
            "ayanamsa sidereal metadata: 53/59 entries with both a reference epoch and offset; custom-definition-only=6 labels: Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs); missing-sidereal-metadata=none"
        ));
}

#[test]
fn house_validation_summary_command_renders_the_summary() {
    let rendered =
        render_cli(&["house-validation-summary"]).expect("house validation summary should render");

    assert!(rendered.contains("House validation corpus: 9 scenarios"));
    assert!(
        rendered.contains("formula families: Equal, Whole Sign, Quadrant, Equatorial projection")
    );
    assert!(rendered.contains("latitude-sensitive systems: Koch, Placidus, Topocentric"));
    assert_eq!(rendered, house_validation_summary_for_report());
}

#[test]
fn house_validation_alias_command_renders_the_summary() {
    let rendered = render_cli(&["house-validation"]).expect("house validation alias should render");

    assert_eq!(rendered, house_validation_summary_for_report());
    assert_eq!(
        render_cli(&["house-validation", "extra"])
            .expect_err("house validation alias should reject extra arguments"),
        "house-validation does not accept extra arguments"
    );
}

#[test]
fn release_house_validation_summary_command_renders_the_release_summary() {
    let rendered = render_cli(&["release-house-validation-summary"])
        .expect("release house validation summary should render");

    assert!(rendered.contains("House code aliases:"));
    assert_eq!(rendered, release_house_validation_summary_for_report());
    assert_eq!(
        render_cli(&["release-house-validation"])
            .expect("release house validation alias should render"),
        release_house_validation_summary_for_report()
    );
    assert_eq!(
        render_cli(&["release-house-validation", "extra"])
            .expect_err("release house validation alias should reject extra arguments"),
        "release-house-validation does not accept extra arguments"
    );
}

#[test]
fn release_house_validation_summary_validation_rejects_drift() {
    let summary = release_house_validation_summary_for_report();
    let drifted_summary = format!("{summary} drift");

    let error = ensure_release_house_validation_summary_matches_current_rendering(&drifted_summary)
        .expect_err("drifted release house validation summary should be rejected");

    assert!(error
        .to_string()
        .contains("release house validation summary no longer matches"));
}

#[test]
fn house_formula_families_summary_command_renders_the_family_list() {
    let rendered = render_cli(&["house-formula-families-summary"])
        .expect("house formula families summary should render");

    assert!(rendered.contains("Equal"));
    assert!(rendered.contains("Whole Sign"));
    assert!(rendered.contains("Quadrant"));
    assert_eq!(rendered, format_house_formula_families_for_report());
}

#[test]
fn house_formula_families_alias_command_renders_the_family_list() {
    let rendered = render_cli(&["house-formula-families"])
        .expect("house formula families alias should render");

    assert_eq!(rendered, format_house_formula_families_for_report());
}

#[test]
fn house_latitude_sensitive_summary_command_renders_the_system_list() {
    let rendered = render_cli(&["house-latitude-sensitive-summary"])
        .expect("latitude-sensitive house systems summary should render");

    assert_eq!(
            rendered,
            "Latitude-sensitive house systems: 8 (Placidus, Koch, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Topocentric, Sunshine, Gauquelin sectors)"
        );
    assert_eq!(
        rendered,
        format_latitude_sensitive_house_systems_for_report()
    );
}

#[test]
fn house_latitude_sensitive_alias_command_renders_the_system_list() {
    let rendered = render_cli(&["house-latitude-sensitive"])
        .expect("latitude-sensitive house systems alias should render");

    assert_eq!(
        rendered,
        format_latitude_sensitive_house_systems_for_report()
    );
}

#[test]
fn house_latitude_sensitive_constraints_summary_command_renders_the_constraints() {
    let rendered = render_cli(&["house-latitude-sensitive-constraints-summary"])
        .expect("latitude-sensitive house constraints summary should render");

    assert_eq!(
        render_cli(&["house-latitude-sensitive-constraints"]).unwrap(),
        rendered
    );
    assert_eq!(
        render_cli(&["house-latitude-sensitive-constraints-summary", "extra"]).unwrap_err(),
        "house-latitude-sensitive-constraints-summary does not accept extra arguments"
    );
    assert_eq!(
        rendered,
        format_latitude_sensitive_house_constraints_for_report()
    );
    assert!(rendered.contains("Latitude-sensitive house constraints: 8"));
    assert!(rendered
        .contains("Placidus [Quadrant system; can fail or become unstable at extreme latitudes.]"));
    assert!(rendered.contains("Topocentric [Topocentric (Polich-Page) house system with geodetic-to-geocentric latitude correction.]"));
}

#[test]
fn house_latitude_sensitive_summary_command_rejects_extra_arguments() {
    assert_eq!(
        render_cli(&["house-latitude-sensitive-summary", "extra"])
            .expect_err("latitude-sensitive house systems summary should reject extra arguments"),
        "house-latitude-sensitive-summary does not accept extra arguments"
    );
}

#[test]
fn house_latitude_sensitive_failure_modes_summary_command_renders_the_failure_modes() {
    let rendered = render_cli(&["house-latitude-sensitive-failure-modes-summary"])
        .expect("latitude-sensitive house failure modes summary should render");

    assert_eq!(
        rendered,
        format_latitude_sensitive_house_failure_modes_for_report()
    );
}

#[test]
fn house_latitude_sensitive_failure_modes_alias_command_renders_the_failure_modes() {
    let rendered = render_cli(&["house-latitude-sensitive-failure-modes"])
        .expect("latitude-sensitive house failure modes alias should render");

    assert_eq!(
        rendered,
        format_latitude_sensitive_house_failure_modes_for_report()
    );
}

#[test]
fn house_latitude_sensitive_failure_modes_summary_command_rejects_extra_arguments() {
    assert_eq!(
        render_cli(&["house-latitude-sensitive-failure-modes-summary", "extra"]).expect_err(
            "latitude-sensitive house failure modes summary should reject extra arguments"
        ),
        "house-latitude-sensitive-failure-modes-summary does not accept extra arguments"
    );
}

#[test]
fn house_code_aliases_summary_command_renders_the_alias_table() {
    let rendered = render_cli(&["house-code-aliases-summary"])
        .expect("house code aliases summary should render");

    assert!(rendered.contains("P -> Placidus"));
    assert!(rendered.contains("T -> Topocentric"));
    assert!(rendered.contains("X -> Meridian"));
    assert_eq!(rendered, format_house_code_aliases_for_report());
}

#[test]
fn target_house_scope_summary_command_renders_the_scope() {
    let rendered = render_cli(&["target-house-scope-summary"])
        .expect("target house scope summary should render");

    assert_eq!(rendered, render_target_house_scope_summary());
    assert_eq!(render_cli(&["target-house-scope"]).unwrap(), rendered);
    assert!(rendered.contains("Target house scope:"));
    assert!(rendered.contains("Baseline milestone:"));
}

#[test]
fn target_ayanamsa_scope_summary_command_renders_the_scope() {
    let rendered = render_cli(&["target-ayanamsa-scope-summary"])
        .expect("target ayanamsa scope summary should render");

    assert_eq!(rendered, render_target_ayanamsa_scope_summary());
    assert_eq!(render_cli(&["target-ayanamsa-scope"]).unwrap(), rendered);
    assert!(rendered.contains("Target ayanamsa scope:"));
    assert!(rendered.contains("Baseline milestone:"));
}

#[test]
fn house_code_alias_summary_command_rejects_extra_arguments() {
    let error = render_cli(&["house-code-alias-summary", "extra"])
        .expect_err("house code alias summary should reject extra arguments");

    assert_eq!(
        error,
        "house-code-alias-summary does not accept extra arguments"
    );
}

#[test]
fn ayanamsa_catalog_validation_summary_command_renders_the_summary() {
    let rendered = render_cli(&["ayanamsa-catalog-validation-summary"])
        .expect("ayanamsa catalog validation summary should render");

    assert_eq!(
        render_cli(&["ayanamsa-catalog-validation"]).unwrap(),
        rendered
    );
    assert!(rendered.contains("ayanamsa catalog validation: ok"));
    assert!(rendered.contains("baseline=5, release=54"));
    assert_eq!(
        rendered,
        ayanamsa_catalog_validation_summary()
            .validated_summary_line()
            .expect("ayanamsa catalog validation summary should validate")
    );
}

#[test]
fn lunar_theory_catalog_summary_command_renders_the_summary() {
    let rendered = render_cli(&["lunar-theory-catalog-summary"])
        .expect("lunar theory catalog summary should render");

    assert_eq!(render_cli(&["lunar-theory-catalog"]).unwrap(), rendered);
    assert_eq!(rendered, lunar_theory_catalog_summary_for_report());
    assert!(rendered.contains("lunar theory catalog: 1 entry, 1 selected entry"));
}

#[test]
fn lunar_theory_catalog_validation_summary_command_renders_the_summary() {
    let rendered = render_cli(&["lunar-theory-catalog-validation-summary"])
        .expect("lunar theory catalog validation summary should render");

    assert_eq!(
        render_cli(&["lunar-theory-catalog-validation"]).unwrap(),
        rendered
    );
    assert_eq!(
        rendered,
        validated_lunar_theory_catalog_validation_summary_for_report()
    );
    assert!(rendered.contains("lunar theory catalog validation: ok"));
}

#[test]
fn lunar_theory_catalog_validation_summary_matches_current_rendering() {
    let summary = validated_lunar_theory_catalog_validation_summary_for_report();

    ensure_lunar_theory_catalog_validation_summary_matches_current_rendering(&summary)
        .expect("lunar theory catalog validation summary should match the current rendering");
}

#[test]
fn lunar_theory_catalog_validation_summary_validation_rejects_drift() {
    let summary = validated_lunar_theory_catalog_validation_summary_for_report();
    let drifted_summary = summary.replace("aliases=1", "aliases=2");

    let error =
        ensure_lunar_theory_catalog_validation_summary_matches_current_rendering(&drifted_summary)
            .expect_err("drifted lunar theory catalog validation summary should be rejected");
    assert!(error
        .to_string()
        .contains("no longer matches the current lunar theory catalog posture"));
}

#[test]
fn lunar_theory_source_selection_summary_command_renders_the_summary() {
    let rendered = render_cli(&["lunar-theory-source-selection-summary"])
        .expect("lunar theory source selection summary should render");

    assert_eq!(
        render_cli(&["lunar-theory-source-selection"]).unwrap(),
        rendered
    );
    assert_eq!(
        rendered,
        pleiades_elp::lunar_theory_source_selection_summary_for_report()
    );
    assert!(rendered.contains("lunar source selection:"));
    assert!(rendered.contains("Meeus-style truncated lunar baseline"));
}

#[test]
fn ayanamsa_reference_offsets_summary_rejects_duplicate_labels() {
    let summary = AyanamsaReferenceOffsetsSummary {
        examples: vec![
            AyanamsaReferenceOffsetExample {
                canonical_name: "Lahiri",
                epoch: JulianDay::from_days(2_435_553.5),
                offset_degrees: Angle::from_degrees(23.245_524_743),
            },
            AyanamsaReferenceOffsetExample {
                canonical_name: "lahiri",
                epoch: JulianDay::from_days(2_451_544.5),
                offset_degrees: Angle::from_degrees(25.0),
            },
        ],
    };

    let error = validated_ayanamsa_reference_offsets_summary_for_report(&summary)
        .expect_err("duplicate ayanamsa reference labels should fail validation");
    assert!(error
        .to_string()
        .contains("ayanamsa reference offsets contains a case-insensitive duplicate name"));
}

#[test]
fn lunar_theory_request_policy_and_frame_treatment_commands_render_the_policy_blocks() {
    let request_policy = render_cli(&["lunar-theory-request-policy-summary"])
        .expect("lunar theory request policy summary should render");
    assert_eq!(request_policy, lunar_theory_request_policy_summary());
    assert_eq!(
        render_cli(&["lunar-theory-request-policy"])
            .expect("lunar theory request policy alias should render"),
        request_policy
    );
    assert_eq!(
        render_cli(&["lunar-theory-request-policy", "extra"])
            .expect_err("lunar theory request policy alias should reject extra arguments"),
        "lunar-theory-request-policy does not accept extra arguments"
    );

    let frame_treatment = render_cli(&["lunar-theory-frame-treatment-summary"])
        .expect("lunar theory frame treatment summary should render");
    assert_eq!(
        frame_treatment,
        lunar_theory_frame_treatment_summary_for_report()
    );
    assert_eq!(
        render_cli(&["lunar-theory-frame-treatment"])
            .expect("lunar theory frame treatment alias should render"),
        frame_treatment
    );
    assert_eq!(
        render_cli(&["lunar-theory-frame-treatment", "extra"])
            .expect_err("lunar theory frame treatment alias should reject extra arguments"),
        "lunar-theory-frame-treatment does not accept extra arguments"
    );

    let limitations = render_cli(&["lunar-theory-limitations-summary"])
        .expect("lunar theory limitations summary should render");
    assert_eq!(limitations, lunar_theory_limitations_summary_for_report());
    assert_eq!(
        render_cli(&["lunar-theory-limitations"]).unwrap(),
        lunar_theory_limitations_summary_for_report()
    );
    let limitations_error = render_cli(&["lunar-theory-limitations", "extra"])
        .expect_err("lunar theory limitations alias should reject extra arguments");
    assert_eq!(
        limitations_error,
        "lunar-theory-limitations-summary does not accept extra arguments"
    );
}

#[test]
fn lunar_boundary_summary_alias_command_renders_the_lunar_boundary_block() {
    let rendered = render_cli(&["lunar-boundary-summary"])
        .expect("lunar boundary summary alias should render");

    assert!(rendered.contains("Reference lunar boundary evidence:"));
    assert_eq!(
        rendered,
        reference_snapshot_lunar_boundary_summary_for_report()
    );
}
