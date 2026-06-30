use super::*;
use pleiades_backend::QualityAnnotation;
use pleiades_types::Longitude;

#[test]
fn package_name_is_stable() {
    assert_eq!(PACKAGE_NAME, "pleiades-elp");
}

#[test]
fn lunar_theory_summary_mentions_the_selected_lunar_theory() {
    let summary = lunar_theory_summary();
    let theory = lunar_theory_specification();
    let formatted = format_lunar_theory_specification(&theory);

    assert_eq!(summary, formatted);
    assert_eq!(theory.summary_line(), summary);
    assert_eq!(theory.to_string(), summary);
    let source = theory.source_selection();
    let source_selection = lunar_theory_source_selection();
    assert_eq!(source_selection, source);
    assert!(summary.contains(theory.model_name));
    assert!(summary.contains(theory.source_identifier));
    assert!(summary.contains(theory.source_family.label()));
    assert!(
        summary.contains("selected key: source identifier=meeus-style-truncated-lunar-baseline")
    );
    assert_eq!(theory.source_family, lunar_theory_source_family());
    assert_eq!(
        theory.source_family.to_string(),
        theory.source_family.label()
    );
    assert_eq!(source.family, theory.source_family);
    assert_eq!(source.source_aliases, theory.source_aliases);
    assert_eq!(source.identifier, theory.source_identifier);
    assert_eq!(source.citation, theory.source_citation);
    assert_eq!(
        source_selection.family_label(),
        theory.source_family.label()
    );
    assert_eq!(source.material, theory.source_material);
    assert_eq!(source.redistribution_note, theory.redistribution_note);
    assert_eq!(source.license_note, theory.license_note);
    assert_eq!(source.family_label(), theory.source_family.label());
    let family_summary = lunar_theory_source_family_summary();
    assert_eq!(family_summary.family, theory.source_family);
    assert_eq!(family_summary.family_label, theory.source_family.label());
    assert_eq!(
        family_summary.selected_source_identifier,
        theory.source_identifier
    );
    assert_eq!(family_summary.selected_model_name, theory.model_name);
    assert_eq!(
        family_summary.selected_catalog_key,
        LunarTheoryCatalogKey::SourceIdentifier(theory.source_identifier)
    );
    assert_eq!(
        family_summary.selected_family_key,
        LunarTheoryCatalogKey::SourceFamily(theory.source_family)
    );
    assert_eq!(
        family_summary.selected_alias_count,
        theory.source_aliases.len()
    );
    assert_eq!(family_summary.summary_line(), family_summary.to_string());
    assert_eq!(
        lunar_theory_source_family_summary_for_report(),
        family_summary.summary_line()
    );
    assert_eq!(
        source.catalog_key(),
        LunarTheoryCatalogKey::SourceIdentifier(theory.source_identifier)
    );
    assert_eq!(
        source.family_key(),
        LunarTheoryCatalogKey::SourceFamily(theory.source_family)
    );
    assert_eq!(
        source.catalog_key().to_string(),
        "source identifier=meeus-style-truncated-lunar-baseline"
    );
    assert_eq!(
        source.family_key().to_string(),
        "source family=Meeus-style truncated analytical baseline"
    );
    assert_eq!(resolve_lunar_theory_by_selection(source), Some(theory));
    let catalog = lunar_theory_catalog();
    assert_eq!(
        resolve_lunar_theory("Meeus-style truncated lunar baseline"),
        Some(theory)
    );
    assert_eq!(
        resolve_lunar_theory_by_alias(theory.source_aliases[0]),
        Some(theory)
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_alias(theory.source_aliases[0]),
        Some(catalog[0])
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::Alias(theory.source_aliases[0],)),
        Some(catalog[0])
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_label(theory.source_aliases[0]),
        Some(catalog[0])
    );
    assert!(resolve_lunar_theory_by_alias("not-a-lunar-alias").is_none());

    let source_summary = lunar_theory_source_summary();
    assert_eq!(source_summary.model_name, theory.model_name);
    assert_eq!(source_summary.source_identifier, theory.source_identifier);
    assert_eq!(source_summary.source_family, theory.source_family);
    assert_eq!(
        source_summary.catalog_key,
        theory.source_selection().catalog_key()
    );
    assert_eq!(
        source_summary.source_family_key,
        theory.source_selection().family_key()
    );
    assert_eq!(
        source_summary.source_family_label,
        theory.source_family.label()
    );
    assert_eq!(source_summary.citation, theory.source_citation);
    assert_eq!(source_summary.provenance, theory.source_material);
    assert_eq!(source_summary.validation_window, theory.validation_window);
    assert_eq!(source_summary.source_aliases, theory.source_aliases);
    assert_eq!(
        source_summary.redistribution_note,
        theory.redistribution_note
    );
    assert_eq!(source_summary.license_note, theory.license_note);
    assert_eq!(
        source_summary.summary_line(),
        format_lunar_theory_source_summary(&source_summary)
    );
    assert_eq!(source_summary.to_string(), source_summary.summary_line());
    assert!(source_summary.validate().is_ok());
    assert_eq!(
        source_summary.validated_summary_line().unwrap(),
        source_summary.summary_line()
    );
    assert_eq!(
        format_lunar_theory_source_summary(&source_summary),
        lunar_theory_source_summary_for_report()
    );
    let mut drifted_source_summary = source_summary;
    drifted_source_summary.source_identifier = "not-the-current-selection";
    let error = drifted_source_summary
        .validate()
        .expect_err("drifted summary should fail validation");
    assert_eq!(
        error.to_string(),
        "the lunar source summary field `source_identifier` is out of sync with the current selection"
    );
    assert_eq!(
        drifted_source_summary.validated_summary_line().unwrap_err().to_string(),
        "the lunar source summary field `source_identifier` is out of sync with the current selection"
    );
    assert_eq!(
        format_validated_lunar_theory_source_summary_for_report(&drifted_source_summary),
        "lunar source selection: unavailable (the lunar source summary field `source_identifier` is out of sync with the current selection)"
    );
    let drifted_catalog_key = LunarTheorySourceSummary {
        catalog_key: LunarTheoryCatalogKey::SourceFamily(source.family),
        ..source_summary
    };
    let error = drifted_catalog_key
        .validate()
        .expect_err("drifted catalog key should fail validation");
    assert_eq!(
        error.to_string(),
        "the lunar source summary field `catalog_key` is out of sync with the current selection"
    );
    assert_eq!(
        format_validated_lunar_theory_source_summary_for_report(&drifted_catalog_key),
        "lunar source selection: unavailable (the lunar source summary field `catalog_key` is out of sync with the current selection)"
    );
    let drifted_family_key = LunarTheorySourceSummary {
        source_family_key: LunarTheoryCatalogKey::SourceIdentifier(source.identifier),
        ..source_summary
    };
    let error = drifted_family_key
        .validate()
        .expect_err("drifted family key should fail validation");
    assert_eq!(
        error.to_string(),
        "the lunar source summary field `source_family_key` is out of sync with the current selection"
    );
    assert_eq!(
        format_validated_lunar_theory_source_summary_for_report(&drifted_family_key),
        "lunar source selection: unavailable (the lunar source summary field `source_family_key` is out of sync with the current selection)"
    );
    let drifted_family_label = LunarTheorySourceSummary {
        source_family_label: "Drifted family label",
        ..source_summary
    };
    let error = drifted_family_label
        .validate()
        .expect_err("drifted family label should fail validation");
    assert_eq!(
        error.to_string(),
        "the lunar source summary field `source_family_label` is out of sync with the current selection"
    );
    assert_eq!(
        format_validated_lunar_theory_source_summary_for_report(&drifted_family_label),
        "lunar source selection: unavailable (the lunar source summary field `source_family_label` is out of sync with the current selection)"
    );
    assert!(lunar_theory_source_summary_for_report().contains("lunar source selection: "));
    assert!(lunar_theory_source_summary_for_report()
        .contains("selected key: source identifier=meeus-style-truncated-lunar-baseline"));
    assert!(lunar_theory_source_summary_for_report()
        .contains("family key: source family=Meeus-style truncated analytical baseline"));
    assert!(lunar_theory_source_summary_for_report()
        .contains("aliases: Meeus-style truncated lunar baseline"));
    assert!(lunar_theory_source_summary_for_report().contains(theory.model_name));
    assert!(lunar_theory_source_summary_for_report().contains(theory.source_identifier));
    assert!(lunar_theory_source_summary_for_report()
        .contains("validation window: JD 2448724.5 (TT) → JD 2459278.5 (TT)"));
    assert!(summary.contains(source.citation));
    assert!(summary.contains("Moon, Mean Node, True Node, Mean Perigee, Mean Apogee"));
    assert!(summary.contains("unsupported bodies: True Apogee, True Perigee"));
    assert!(summary.contains("validation window: JD 2448724.5 (TT) → JD 2459278.5 (TT)"));
    assert!(summary.contains(
        "geocentric Moon RA/Dec example used for the mean-obliquity equatorial transform"
    ));
    assert!(summary.contains("mean apogee"));
    assert_eq!(
        lunar_theory_request_policy_summary(),
        theory.request_policy.summary_line()
    );
    assert_eq!(
        theory.request_policy.to_string(),
        theory.request_policy.summary_line()
    );
    assert!(theory.request_policy.validate().is_ok());
    assert_eq!(
        format_validated_lunar_theory_request_policy_for_report(&theory.request_policy),
        theory.request_policy.summary_line()
    );

    let mut drifted_request_policy = theory.request_policy;
    drifted_request_policy.supported_time_scales = &[TimeScale::Tt];
    let error = drifted_request_policy
        .validate()
        .expect_err("drifted request policy should fail validation");
    assert_eq!(
        error.to_string(),
        "the lunar theory request policy field `supported_time_scales` is out of sync with the current selection"
    );
    assert_eq!(
        format_validated_lunar_theory_request_policy_for_report(&drifted_request_policy),
        "lunar theory request policy: unavailable (the lunar theory request policy field `supported_time_scales` is out of sync with the current selection)"
    );
    assert_eq!(lunar_theory_summary_for_report(), theory.summary_line());
    let limitations_summary = lunar_theory_limitations_summary();
    assert_eq!(
        limitations_summary.summary_line(),
        format!(
            "lunar theory limitations: Compact Meeus-style truncated lunar baseline; supported bodies: Moon, Mean Node, True Node, Mean Perigee, Mean Apogee; unsupported bodies: True Apogee, True Perigee; release-grade evidence by channel: {}; {}",
            lunar_reference_evidence_envelope_for_report(),
            lunar_equatorial_reference_evidence_envelope_for_report()
        )
    );
    assert_eq!(
        format_lunar_theory_limitations_summary(&limitations_summary),
        limitations_summary.summary_line()
    );
    assert_eq!(
        lunar_theory_limitations_summary_for_report(),
        limitations_summary.summary_line()
    );
    let drifted_limitations_summary = LunarTheoryLimitationsSummary {
        unsupported_bodies: &[CelestialBody::TrueApogee],
        ..limitations_summary
    };
    let error = drifted_limitations_summary
        .validate()
        .expect_err("drifted limitations summary should fail validation");
    assert_eq!(
        error.to_string(),
        "the lunar theory limitations summary field `unsupported_bodies` is out of sync with the current baseline"
    );
    assert_eq!(
        format_validated_lunar_theory_limitations_summary_for_report(
            &drifted_limitations_summary
        ),
        "lunar theory limitations: unavailable (the lunar theory limitations summary field `unsupported_bodies` is out of sync with the current baseline)"
    );
    assert!(summary.contains("frames=Ecliptic, Equatorial"));
    assert!(summary.contains("time scales=TT, TDB"));
    assert!(summary.contains("zodiac modes=Tropical"));
    assert!(summary.contains("apparentness=Mean"));
    assert!(summary.contains("topocentric observer=false"));

    let mut drifted_spec = theory;
    drifted_spec.request_policy = LunarTheoryRequestPolicy {
        supported_time_scales: &[TimeScale::Tt],
        ..drifted_spec.request_policy
    };
    let error = drifted_spec
        .validate()
        .expect_err("drifted specification should fail validation");
    assert_eq!(
        error.to_string(),
        "the lunar theory specification field `request_policy.supported_time_scales` is out of sync with the current selection"
    );
    assert_eq!(
        format_validated_lunar_theory_specification_for_report(&drifted_spec),
        "ELP lunar theory specification: unavailable (the lunar theory specification field `request_policy.supported_time_scales` is out of sync with the current selection)"
    );

    let catalog_summary = lunar_theory_catalog_summary();
    assert_eq!(catalog.len(), 1);
    assert!(catalog[0].selected);
    assert_eq!(catalog[0].specification, theory);
    assert_eq!(catalog_summary.entry_count, 1);
    assert_eq!(catalog_summary.selected_count, 1);
    assert_eq!(
        catalog_summary.selected_source_identifier,
        theory.source_identifier
    );
    assert_eq!(catalog_summary.selected_source_family, theory.source_family);
    assert_eq!(
        catalog_summary.selected_source_family_label,
        theory.source_family.label()
    );
    assert_eq!(catalog_summary.selected_catalog_key, source.catalog_key());
    assert_eq!(catalog_summary.selected_family_key, source.family_key());
    assert_eq!(
        catalog_summary.selected_alias_count,
        theory.source_aliases.len()
    );
    assert_eq!(
        catalog_summary.selected_supported_body_count,
        theory.supported_bodies.len()
    );
    assert_eq!(
        catalog_summary.selected_unsupported_body_count,
        theory.unsupported_bodies.len()
    );
    assert_eq!(catalog_summary.summary_line(), catalog_summary.to_string());
    assert_eq!(
        catalog_summary.validated_summary_line().unwrap(),
        catalog_summary.summary_line()
    );
    assert_eq!(
        format_lunar_theory_catalog_summary(&catalog_summary),
        catalog_summary.summary_line()
    );
    assert_eq!(
        lunar_theory_catalog_summary_for_report(),
        catalog_summary.summary_line()
    );
    let catalog_entry_summary = catalog[0].summary_line();
    assert_eq!(catalog_entry_summary, catalog[0].to_string());
    assert_eq!(
        catalog[0].validated_summary_line().unwrap(),
        catalog_entry_summary
    );
    assert!(catalog_entry_summary.contains("lunar theory catalog entry: selected=true"));
    assert!(catalog_entry_summary.contains(
        "source=meeus-style-truncated-lunar-baseline [Meeus-style truncated analytical baseline]"
    ));
    assert!(catalog_entry_summary
        .contains("key=source identifier=meeus-style-truncated-lunar-baseline"));
    assert!(catalog_entry_summary.contains(&format!("aliases={}", theory.source_aliases.len())));
    assert!(catalog_entry_summary.contains(&format!(
        "supported bodies={}",
        theory.supported_bodies.len()
    )));
    assert!(catalog_entry_summary.contains(&format!(
        "unsupported bodies={}",
        theory.unsupported_bodies.len()
    )));
    let mut drifted_catalog_entry = catalog[0];
    drifted_catalog_entry.selected = false;
    let drifted_catalog_entry_summary = drifted_catalog_entry.summary_line();
    assert!(drifted_catalog_entry_summary.contains("selected=false"));
    let drifted_catalog_entry_error = drifted_catalog_entry
        .validated_summary_line()
        .expect_err("unselected catalog entry should fail validation");
    assert_eq!(
        drifted_catalog_entry_error.to_string(),
        "the lunar-theory catalog has no selected entry"
    );
    assert!(catalog[0].validate().is_ok());
    let mut drifted_specification = theory;
    drifted_specification.model_name = "Drifted lunar baseline";
    let spec_error = drifted_specification
        .validate()
        .expect_err("drifted lunar specification should fail validation");
    assert_eq!(
        spec_error.to_string(),
        "the lunar theory specification field `model_name` is out of sync with the current selection"
    );
    let drifted_catalog_validation =
        validate_lunar_theory_catalog_entries(&[LunarTheoryCatalogEntry {
            selected: true,
            specification: drifted_specification,
        }])
        .expect_err("drifted lunar catalog entry should fail validation");
    assert_eq!(
        drifted_catalog_validation.to_string(),
        "the lunar-theory catalog entry `meeus-style-truncated-lunar-baseline` has specification field `model_name` out of sync with the current catalog"
    );
    assert!(lunar_theory_catalog_summary_for_report()
        .contains("lunar theory catalog: 1 entry, 1 selected entry"));
    assert!(lunar_theory_catalog_summary_for_report()
        .contains("selected key: source identifier=meeus-style-truncated-lunar-baseline"));
    assert!(lunar_theory_catalog_summary_for_report()
        .contains("selected family key: source family=Meeus-style truncated analytical baseline"));
    assert!(catalog_summary.validate().is_ok());
    let mut drifted_catalog_summary = catalog_summary;
    drifted_catalog_summary.selected_alias_count += 1;
    let error = drifted_catalog_summary
        .validate()
        .expect_err("drifted catalog summary should fail validation");
    assert_eq!(
        error.to_string(),
        "the lunar catalog summary field `selected_alias_count` is out of sync with the current catalog"
    );
    assert_eq!(
        drifted_catalog_summary.validated_summary_line().unwrap_err().to_string(),
        "the lunar catalog summary field `selected_alias_count` is out of sync with the current catalog"
    );
    assert_eq!(
        format_validated_lunar_theory_catalog_summary_for_report(&drifted_catalog_summary),
        "lunar theory catalog: unavailable (the lunar catalog summary field `selected_alias_count` is out of sync with the current catalog)"
    );
    let drifted_catalog_key = LunarTheoryCatalogSummary {
        selected_catalog_key: LunarTheoryCatalogKey::SourceFamily(source.family),
        ..catalog_summary
    };
    let catalog_key_error = drifted_catalog_key
        .validate()
        .expect_err("drifted catalog key should fail validation");
    assert_eq!(
        catalog_key_error.to_string(),
        "the lunar catalog summary field `selected_catalog_key` is out of sync with the current catalog"
    );
    assert_eq!(
        format_validated_lunar_theory_catalog_summary_for_report(&drifted_catalog_key),
        "lunar theory catalog: unavailable (the lunar catalog summary field `selected_catalog_key` is out of sync with the current catalog)"
    );
    let drifted_family_key = LunarTheoryCatalogSummary {
        selected_family_key: LunarTheoryCatalogKey::SourceIdentifier(source.identifier),
        ..catalog_summary
    };
    let family_key_error = drifted_family_key
        .validate()
        .expect_err("drifted family key should fail validation");
    assert_eq!(
        family_key_error.to_string(),
        "the lunar catalog summary field `selected_family_key` is out of sync with the current catalog"
    );
    assert_eq!(
        format_validated_lunar_theory_catalog_summary_for_report(&drifted_family_key),
        "lunar theory catalog: unavailable (the lunar catalog summary field `selected_family_key` is out of sync with the current catalog)"
    );
    let catalog_validation_summary = lunar_theory_catalog_validation_summary();
    assert_eq!(catalog_validation_summary.entry_count, 1);
    assert_eq!(catalog_validation_summary.selected_count, 1);
    assert_eq!(catalog_validation_summary.selected_source, Some(source));
    assert!(catalog_validation_summary.validation_result.is_ok());
    assert_eq!(
        catalog_validation_summary.summary_line(),
        format_lunar_theory_catalog_validation_summary(&catalog_validation_summary)
    );
    assert_eq!(
        catalog_validation_summary.to_string(),
        catalog_validation_summary.summary_line()
    );
    assert_eq!(
        format_lunar_theory_catalog_validation_summary(&catalog_validation_summary),
        lunar_theory_catalog_validation_summary_for_report()
    );
    assert!(lunar_theory_catalog_validation_summary_for_report()
        .contains("lunar theory catalog validation: ok (1 entries, 1 selected; selected source: meeus-style-truncated-lunar-baseline [Meeus-style truncated analytical baseline]; selected key: source identifier=meeus-style-truncated-lunar-baseline; selected family key: source family=Meeus-style truncated analytical baseline; aliases=1; specification sync, round-trip, alias uniqueness, body coverage disjointness, and case-insensitive key matching verified)"));
    assert!(lunar_theory_catalog_summary_for_report()
        .contains("selected source: meeus-style-truncated-lunar-baseline"));
    assert!(lunar_theory_catalog_summary_for_report()
        .contains("aliases=1; supported bodies=5; unsupported bodies=2"));
    assert!(lunar_theory_catalog_validation_summary_for_report().contains("aliases=1"));
    assert!(lunar_theory_catalog_validation_summary_for_report()
        .contains("case-insensitive key matching verified"));
    assert_eq!(
        lunar_theory_catalog_entry_for_source_identifier(theory.source_identifier),
        Some(catalog[0])
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_model_name(theory.model_name),
        Some(catalog[0])
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_source_family(theory.source_family),
        Some(catalog[0])
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_family_label(theory.source_family.label()),
        Some(catalog[0])
    );
    assert_eq!(resolve_lunar_theory(theory.source_identifier), Some(theory));
    assert_eq!(resolve_lunar_theory(theory.model_name), Some(theory));
    assert_eq!(
        resolve_lunar_theory_by_family(theory.source_family),
        Some(theory)
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_label(theory.source_family.label()),
        Some(catalog[0])
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::SourceIdentifier(
            theory.source_identifier,
        )),
        Some(catalog[0])
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::ModelName(theory.model_name,)),
        Some(catalog[0])
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::FamilyLabel(
            theory.source_family.label(),
        )),
        Some(catalog[0])
    );
    assert_eq!(
        resolve_lunar_theory_by_key(LunarTheoryCatalogKey::SourceIdentifier(
            theory.source_identifier,
        )),
        Some(theory)
    );
    assert_eq!(
        resolve_lunar_theory_by_key(LunarTheoryCatalogKey::ModelName(theory.model_name)),
        Some(theory)
    );
    assert_eq!(
        resolve_lunar_theory_by_key(LunarTheoryCatalogKey::SourceFamily(theory.source_family)),
        Some(theory)
    );
    assert_eq!(
        resolve_lunar_theory_by_key(LunarTheoryCatalogKey::FamilyLabel(
            theory.source_family.label(),
        )),
        Some(theory)
    );
    assert_eq!(
        resolve_lunar_theory_by_key(LunarTheoryCatalogKey::Alias(
            "Meeus-style truncated lunar baseline",
        )),
        Some(theory)
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_selection(source),
        Some(catalog[0])
    );
    assert!(resolve_lunar_theory("not-a-lunar-theory").is_none());
    assert!(resolve_lunar_theory_by_family(
        LunarTheorySourceFamily::MeeusStyleTruncatedAnalyticalBaseline
    )
    .is_some());
    assert!(
        resolve_lunar_theory_by_key(LunarTheoryCatalogKey::FamilyLabel("not-a-family")).is_none()
    );
    assert!(validate_lunar_theory_catalog().is_ok());

    let empty_catalog_error = validate_lunar_theory_catalog_entries(&[])
        .expect_err("empty catalog should fail validation");
    assert_eq!(
        empty_catalog_error.to_string(),
        "the lunar-theory catalog is empty"
    );

    let no_selected_catalog_error =
        validate_lunar_theory_catalog_entries(&[LunarTheoryCatalogEntry {
            selected: false,
            specification: theory,
        }])
        .expect_err("catalog without a selected entry should fail validation");
    assert_eq!(
        no_selected_catalog_error.to_string(),
        "the lunar-theory catalog has no selected entry"
    );
}

#[test]
fn lunar_theory_catalog_resolvers_are_case_insensitive_across_key_families() {
    let theory = lunar_theory_specification();
    let catalog = lunar_theory_catalog();
    let source_identifier_upper = theory.source_identifier.to_uppercase();
    let model_name_upper = theory.model_name.to_uppercase();
    let family_label_upper = theory.source_family.label().to_uppercase();
    let alias_upper = theory.source_aliases[0].to_uppercase();

    assert_eq!(
        resolve_lunar_theory_by_key(LunarTheoryCatalogKey::SourceIdentifier(
            source_identifier_upper.as_str(),
        )),
        Some(theory)
    );
    assert_eq!(
        resolve_lunar_theory_by_key(LunarTheoryCatalogKey::ModelName(model_name_upper.as_str(),)),
        Some(theory)
    );
    assert_eq!(
        resolve_lunar_theory_by_key(LunarTheoryCatalogKey::FamilyLabel(
            family_label_upper.as_str(),
        )),
        Some(theory)
    );
    assert_eq!(
        resolve_lunar_theory_by_key(LunarTheoryCatalogKey::Alias(alias_upper.as_str())),
        Some(theory)
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::SourceIdentifier(
            source_identifier_upper.as_str(),
        )),
        Some(catalog[0])
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::ModelName(
            model_name_upper.as_str(),
        )),
        Some(catalog[0])
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::FamilyLabel(
            family_label_upper.as_str(),
        )),
        Some(catalog[0])
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::Alias(alias_upper.as_str())),
        Some(catalog[0])
    );
    assert_eq!(
        resolve_lunar_theory_by_alias(alias_upper.as_str()),
        Some(theory)
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_label(source_identifier_upper.as_str()),
        Some(catalog[0])
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_label(model_name_upper.as_str()),
        Some(catalog[0])
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_label(family_label_upper.as_str()),
        Some(catalog[0])
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_label(alias_upper.as_str()),
        Some(catalog[0])
    );
}

#[test]
fn lunar_theory_catalog_validation_rejects_duplicate_aliases_within_entry() {
    let duplicate_alias_catalog = [LunarTheoryCatalogEntry {
        selected: true,
        specification: LunarTheorySpecification {
            source_aliases: &[
                "Meeus-style truncated lunar baseline",
                "meeus-style truncated lunar baseline",
            ],
            ..LUNAR_THEORY_SPECIFICATION
        },
    }];

    assert!(matches!(
        validate_lunar_theory_catalog_entries(&duplicate_alias_catalog),
        Err(LunarTheoryCatalogValidationError::DuplicateAlias {
            alias: "Meeus-style truncated lunar baseline"
        })
    ));
}

#[test]
fn lunar_theory_catalog_validation_rejects_alias_collisions_with_core_labels() {
    let colliding_alias_catalog = [LunarTheoryCatalogEntry {
        selected: true,
        specification: LunarTheorySpecification {
            source_aliases: &["meeus-style-truncated-lunar-baseline"],
            ..LUNAR_THEORY_SPECIFICATION
        },
    }];

    assert!(matches!(
        validate_lunar_theory_catalog_entries(&colliding_alias_catalog),
        Err(LunarTheoryCatalogValidationError::DuplicateAlias {
            alias: "meeus-style-truncated-lunar-baseline"
        })
    ));
}

#[test]
fn lunar_theory_catalog_validation_rejects_duplicate_supported_bodies() {
    const DUPLICATE_SUPPORTED_BODIES: &[CelestialBody] =
        &[CelestialBody::Moon, CelestialBody::Moon];

    let duplicate_supported_catalog = [LunarTheoryCatalogEntry {
        selected: true,
        specification: LunarTheorySpecification {
            supported_bodies: DUPLICATE_SUPPORTED_BODIES,
            ..LUNAR_THEORY_SPECIFICATION
        },
    }];

    assert!(matches!(
        validate_lunar_theory_catalog_entries(&duplicate_supported_catalog),
        Err(LunarTheoryCatalogValidationError::DuplicateSupportedBody {
            body: CelestialBody::Moon
        })
    ));
}

#[test]
fn lunar_theory_catalog_validation_rejects_duplicate_unsupported_bodies() {
    const DUPLICATE_UNSUPPORTED_BODIES: &[CelestialBody] =
        &[CelestialBody::TrueApogee, CelestialBody::TrueApogee];

    let duplicate_unsupported_catalog = [LunarTheoryCatalogEntry {
        selected: true,
        specification: LunarTheorySpecification {
            unsupported_bodies: DUPLICATE_UNSUPPORTED_BODIES,
            ..LUNAR_THEORY_SPECIFICATION
        },
    }];

    assert!(matches!(
        validate_lunar_theory_catalog_entries(&duplicate_unsupported_catalog),
        Err(
            LunarTheoryCatalogValidationError::DuplicateUnsupportedBody {
                body: CelestialBody::TrueApogee
            }
        )
    ));
}

#[test]
fn lunar_theory_catalog_validation_rejects_supported_and_unsupported_overlaps() {
    const OVERLAPPING_SUPPORTED_BODIES: &[CelestialBody] = &[CelestialBody::Moon];
    const OVERLAPPING_UNSUPPORTED_BODIES: &[CelestialBody] = &[CelestialBody::Moon];

    let overlapping_catalog = [LunarTheoryCatalogEntry {
        selected: true,
        specification: LunarTheorySpecification {
            supported_bodies: OVERLAPPING_SUPPORTED_BODIES,
            unsupported_bodies: OVERLAPPING_UNSUPPORTED_BODIES,
            ..LUNAR_THEORY_SPECIFICATION
        },
    }];

    assert!(matches!(
        validate_lunar_theory_catalog_entries(&overlapping_catalog),
        Err(
            LunarTheoryCatalogValidationError::OverlappingSupportedAndUnsupportedBody {
                body: CelestialBody::Moon
            }
        )
    ));
}

#[test]
fn metadata_mentions_the_selected_lunar_theory() {
    let metadata = ElpBackend::new().metadata();
    let theory = lunar_theory_specification();

    let source = theory.source_selection();
    assert_eq!(lunar_theory_source_selection(), source);
    assert!(metadata.provenance.summary.contains(theory.model_name));
    assert!(metadata.provenance.summary.contains(source.identifier));
    assert!(metadata
        .provenance
        .summary
        .contains(theory.source_family.label()));
    assert_eq!(theory.source_family, lunar_theory_source_family());
    assert_eq!(
        theory.source_family.to_string(),
        theory.source_family.label()
    );
    assert_eq!(source.family, theory.source_family);
    assert_eq!(source.source_aliases, theory.source_aliases);
    assert_eq!(source.identifier, theory.source_identifier);
    assert_eq!(source.citation, theory.source_citation);
    assert_eq!(source.material, theory.source_material);
    assert_eq!(source.redistribution_note, theory.redistribution_note);
    assert_eq!(source.license_note, theory.license_note);
    assert_eq!(source.family_label(), theory.source_family.label());
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source == &lunar_theory_source_family_summary_for_report()));
    assert!(metadata.provenance.summary.contains(source.citation));
    assert!(metadata
        .provenance
        .summary
        .contains("true apogee/perigee unsupported"));
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source.contains("Published lunar position, node, and mean-point formulas")));
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source.contains(theory.truncation_note)));
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source.contains(theory.unit_note)));
    assert_eq!(
        metadata.supported_time_scales,
        vec![TimeScale::Tt, TimeScale::Tdb]
    );
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source.contains(theory.source_identifier)));
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source.contains(theory.source_citation)));
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source.contains("No external coefficient-file redistribution constraints")));
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source.contains("pure Rust")));
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source.contains("J2000 lunar-point anchors")));
    assert!(metadata
        .provenance
        .data_sources
        .iter()
        .any(|source| source.contains("2021-03-05 mean-perigee example")));
}

#[test]
fn lunar_theory_source_selection_has_a_stable_summary_line() {
    let selection = lunar_theory_source_selection();
    let summary = selection.summary_line();

    assert_eq!(selection.to_string(), summary);
    assert_eq!(format_lunar_theory_source_selection(&selection), summary);
    assert_eq!(lunar_theory_source_selection_summary(), summary);
    assert_eq!(
        resolve_lunar_theory_by_key(selection.catalog_key()),
        Some(lunar_theory_specification())
    );
    assert_eq!(
        resolve_lunar_theory_by_key(selection.family_key()),
        Some(lunar_theory_specification())
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_selection(selection).map(|entry| entry.specification),
        Some(lunar_theory_specification())
    );
    assert_eq!(
        lunar_theory_catalog_entry_for_current_selection().map(|entry| entry.specification),
        Some(lunar_theory_specification())
    );
    assert_eq!(
        current_lunar_theory_catalog_entry().map(|entry| entry.specification),
        Some(lunar_theory_specification())
    );
    assert!(summary.contains(selection.identifier));
    assert!(
        summary.contains("selected key: source identifier=meeus-style-truncated-lunar-baseline")
    );
    assert!(summary.contains("family key: source family=Meeus-style truncated analytical baseline"));
    assert!(summary.contains(selection.family_label()));
    assert!(summary.contains(selection.citation));
    assert!(summary.contains(selection.license_note));
    assert_eq!(selection.validated_summary_line().unwrap(), summary);
    assert_eq!(selection.validate(), Ok(()));
    assert_eq!(
        validated_lunar_theory_source_selection_summary_for_report(),
        Ok(summary)
    );
}

#[test]
fn lunar_theory_source_selection_validation_fails_closed_for_drifted_fields() {
    let selection = lunar_theory_source_selection();
    let drifted = LunarTheorySourceSelection {
        identifier: "not-the-current-selection",
        ..selection
    };

    let error = drifted
        .validate()
        .expect_err("drifted source selection should fail validation");
    assert_eq!(
        error.to_string(),
        "the lunar source selection field `identifier` is out of sync with the current selection"
    );
    assert_eq!(
        drifted.validated_summary_line().unwrap_err().to_string(),
        "the lunar source selection field `identifier` is out of sync with the current selection"
    );
    assert_eq!(
        format_lunar_theory_source_selection(&drifted),
        "lunar source selection: unavailable (the lunar source selection field `identifier` is out of sync with the current selection)"
    );
    assert_eq!(
        lunar_theory_source_selection_summary(),
        selection.summary_line()
    );
}

#[test]
fn lunar_theory_source_summary_validation_fails_closed_for_drifted_keys() {
    let source_summary = lunar_theory_source_summary();

    let drifted_catalog_key = LunarTheorySourceSummary {
        catalog_key: LunarTheoryCatalogKey::SourceFamily(source_summary.source_family),
        ..source_summary
    };
    let catalog_error = drifted_catalog_key
        .validate()
        .expect_err("drifted catalog key should fail validation");
    assert_eq!(
        catalog_error.to_string(),
        "the lunar source summary field `catalog_key` is out of sync with the current selection"
    );
    assert_eq!(
        format_validated_lunar_theory_source_summary_for_report(&drifted_catalog_key),
        "lunar source selection: unavailable (the lunar source summary field `catalog_key` is out of sync with the current selection)"
    );

    let drifted_family_key = LunarTheorySourceSummary {
        source_family_key: LunarTheoryCatalogKey::SourceIdentifier(
            source_summary.source_identifier,
        ),
        ..source_summary
    };
    let family_error = drifted_family_key
        .validate()
        .expect_err("drifted family key should fail validation");
    assert_eq!(
        family_error.to_string(),
        "the lunar source summary field `source_family_key` is out of sync with the current selection"
    );
    assert_eq!(
        format_validated_lunar_theory_source_summary_for_report(&drifted_family_key),
        "lunar source selection: unavailable (the lunar source summary field `source_family_key` is out of sync with the current selection)"
    );

    assert_eq!(
        lunar_theory_source_summary_for_report(),
        source_summary.summary_line()
    );
}

#[test]
fn backend_supports_the_moon_and_lunar_nodes() {
    let backend = ElpBackend::new();
    assert!(backend.supports_body(CelestialBody::Moon));
    assert!(!backend.supports_body(CelestialBody::Sun));
}

#[test]
fn published_moon_example_matches_reference() {
    let backend = ElpBackend::new();
    let instant = Instant::new(
        pleiades_types::JulianDay::from_days(2_448_724.5),
        TimeScale::Tt,
    );
    let result = backend
        .position(&mean_request_at(CelestialBody::Moon, instant))
        .expect("moon query should work");
    let ecliptic = result.ecliptic.expect("ecliptic result should exist");
    let motion = result.motion.expect("motion should be populated");

    // J2000 ecliptic boundary values (backend now emits J2000, not of-date).
    // Of-date values were lon=133.162_655, lat=-3.229_126; after precessing back
    // to J2000 via precess_ecliptic_date_to_j2000 they shift by ~+0.108° lon, ~+0.001° lat.
    assert!((ecliptic.longitude.degrees() - 133.270_485_958).abs() < 1e-4);
    assert!((ecliptic.latitude.degrees() - -3.228_456_673).abs() < 1e-4);
    assert!(
        (ecliptic.distance_au.expect("moon distance should exist") * 149_597_870.700 - 368_409.7)
            .abs()
            < 0.5
    );
    assert!(motion
        .longitude_deg_per_day
        .expect("longitude speed should exist")
        .is_finite());
    assert!(motion
        .latitude_deg_per_day
        .expect("latitude speed should exist")
        .is_finite());
    assert!(motion
        .distance_au_per_day
        .expect("distance speed should exist")
        .is_finite());
    assert_eq!(result.quality, QualityAnnotation::Approximate);
}

#[test]
fn published_apparent_moon_example_matches_the_shared_mean_obliquity_transform() {
    let sample = lunar_apparent_comparison_evidence()
        .iter()
        .find(|sample| (sample.epoch.julian_day.days() - 2_453_100.5).abs() < f64::EPSILON)
        .expect("NASA RP 1349 sample should exist");

    let equatorial = EquatorialCoordinates::new(
        Angle::from_degrees(sample.apparent_right_ascension_deg),
        Latitude::from_degrees(sample.apparent_declination_deg),
        Some(sample.apparent_distance_au),
    );
    let ecliptic = equatorial.to_ecliptic(sample.epoch.mean_obliquity());

    assert!((ecliptic.longitude.degrees() - sample.apparent_longitude_deg).abs() < 1e-6);
    assert!((ecliptic.latitude.degrees() - sample.apparent_latitude_deg).abs() < 1e-6);
    assert!(
        (ecliptic
            .distance_au
            .expect("apparent Moon distance should exist")
            - sample.apparent_distance_au)
            .abs()
            < 1e-12
    );
    assert!(sample.note.contains("NASA RP 1349"));
    assert!(sample.note.contains("shared mean-obliquity transform"));
}

#[test]
fn published_apparent_moon_comparison_datum_matches_the_reference_slice() {
    let sample = lunar_apparent_comparison_evidence()
        .iter()
        .find(|sample| (sample.epoch.julian_day.days() - 2_448_724.5).abs() < f64::EPSILON)
        .expect("1992 apparent Moon sample should exist");

    assert_eq!(sample.body, CelestialBody::Moon);
    assert_eq!(sample.epoch.julian_day.days(), 2_448_724.5);
    assert!((sample.apparent_longitude_deg - 133.167_264).abs() < 1e-12);
    assert!((sample.apparent_latitude_deg - (-3.229_126)).abs() < 1e-12);
    assert!((sample.apparent_distance_au - (368_409.7 / 149_597_870.700)).abs() < 1e-12);
    assert!((sample.apparent_right_ascension_deg - 134.688_469).abs() < 1e-12);
    assert!((sample.apparent_declination_deg - 13.768_367).abs() < 1e-12);
    assert!(sample
        .note
        .contains("1992-04-12 apparent geocentric Moon example"));
    assert!(sample.note.contains("mean/apparent comparison datum"));
}

#[test]
fn published_eclipsewise_apparent_moon_example_matches_the_shared_mean_obliquity_transform() {
    let sample = lunar_apparent_comparison_evidence()
        .iter()
        .find(|sample| (sample.epoch.julian_day.days() - 2_453_986.285_649).abs() < f64::EPSILON)
        .expect("EclipseWise apparent Moon sample should exist");

    let equatorial = EquatorialCoordinates::new(
        Angle::from_degrees(sample.apparent_right_ascension_deg),
        Latitude::from_degrees(sample.apparent_declination_deg),
        Some(sample.apparent_distance_au),
    );
    let ecliptic = equatorial.to_ecliptic(sample.epoch.mean_obliquity());

    assert!((ecliptic.longitude.degrees() - sample.apparent_longitude_deg).abs() < 1e-6);
    assert!((ecliptic.latitude.degrees() - sample.apparent_latitude_deg).abs() < 1e-6);
    assert!(
        (ecliptic
            .distance_au
            .expect("apparent Moon distance should exist")
            - sample.apparent_distance_au)
            .abs()
            < 1e-12
    );
    assert!(sample.note.contains("EclipseWise"));
    assert!(sample.note.contains("shared mean-obliquity transform"));
}

#[test]
fn published_true_node_example_matches_reference() {
    let backend = ElpBackend::new();
    let instant = Instant::new(
        pleiades_types::JulianDay::from_days(2_419_914.5),
        TimeScale::Tt,
    );
    let result = backend
        .position(&mean_request_at(CelestialBody::TrueNode, instant))
        .expect("true node query should work");
    let ecliptic = result.ecliptic.expect("ecliptic result should exist");
    let motion = result.motion.expect("motion should be populated");

    assert!((ecliptic.longitude.degrees() - 0.876_3).abs() < 1e-4);
    assert_eq!(ecliptic.latitude.degrees(), 0.0);
    assert_eq!(ecliptic.distance_au, None);
    assert!(motion
        .longitude_deg_per_day
        .expect("longitude speed should exist")
        .is_finite());
    assert_eq!(motion.latitude_deg_per_day, Some(0.0));
    assert_eq!(motion.distance_au_per_day, None);
    assert_eq!(result.quality, QualityAnnotation::Approximate);
}

#[test]
fn moon_samples_remain_finite_across_high_curvature_window() {
    let backend = ElpBackend::new();
    let instants = [J2000 - 1.0, J2000, J2000 + 1.0, J2000 + 2.0]
        .map(|days| Instant::new(pleiades_types::JulianDay::from_days(days), TimeScale::Tt));

    let mut previous_longitude: Option<f64> = None;
    let mut previous_distance: Option<f64> = None;

    for instant in instants {
        let result = backend
            .position(&mean_request_at(CelestialBody::Moon, instant))
            .expect("moon query should work");
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");
        let motion = result.motion.expect("motion should be populated");

        assert!(ecliptic.longitude.degrees().is_finite());
        assert!(ecliptic.latitude.degrees().is_finite());
        assert!(ecliptic
            .distance_au
            .expect("moon distance should exist")
            .is_finite());
        assert!(motion
            .longitude_deg_per_day
            .expect("longitude speed should exist")
            .is_finite());
        assert!(motion
            .latitude_deg_per_day
            .expect("latitude speed should exist")
            .is_finite());
        assert!(motion
            .distance_au_per_day
            .expect("distance speed should exist")
            .is_finite());
        assert!(motion.longitude_deg_per_day.unwrap().abs() < 20.0);
        assert!(motion.latitude_deg_per_day.unwrap().abs() < 10.0);

        if let Some(previous_longitude) = previous_longitude {
            let delta =
                signed_longitude_delta_degrees(previous_longitude, ecliptic.longitude.degrees());
            assert!(delta.abs() > 1.0);
            assert!(delta.abs() < 20.0);
        }

        if let Some(previous_distance) = previous_distance {
            assert!(
                (ecliptic.distance_au.expect("moon distance should exist") - previous_distance)
                    .abs()
                    < 0.02
            );
        }

        previous_longitude = Some(ecliptic.longitude.degrees());
        previous_distance = ecliptic.distance_au;
    }

    assert!(previous_longitude.is_some());
    assert!(previous_distance.is_some());
}

#[test]
fn lunar_high_curvature_continuity_evidence_is_rendered() {
    let envelope = lunar_high_curvature_continuity_envelope().expect("envelope should exist");
    let report = lunar_high_curvature_continuity_evidence_for_report();

    assert_eq!(report, envelope.summary_line());
    assert_eq!(envelope.validated_summary_line().unwrap(), report);
    assert!(envelope.validate().is_ok());
    assert!(report.contains("lunar high-curvature continuity evidence: 6 samples across 1 bodies"));
    assert!(report.contains("epoch range JD 2451544.0 (TT) → JD 2451547.0 (TT)"));
    assert!(report.contains("max adjacent Δlon="));
    assert!(report.contains("max adjacent Δlat="));
    assert!(report.contains("max adjacent Δdist="));
    assert!(report.contains("within regression limits=true"));
    assert!(report.contains("Δlon≤20.0°"));
    assert!(report.contains("Δlat≤10.0°"));
    assert!(report.contains("Δdist≤0.02 AU"));
}

#[test]
fn lunar_source_window_evidence_is_rendered() {
    let summary = lunar_source_window_summary().expect("source window summary should exist");
    let report = lunar_source_window_summary_for_report();

    assert_eq!(report, summary.summary_line());
    assert_eq!(report, format_lunar_source_window_summary(&summary));
    assert_eq!(
        report,
        summary
            .validated_summary_line()
            .expect("summary should validate")
    );
    assert!(report.contains(
        "lunar source windows: 7 exact Moon samples across 1 bodies in 2 exact windows; 4 reference-only apparent Moon samples across 1 bodies in 4 apparent windows"
    ));
    assert!(report.contains(
        "exact windows: published 1992-04-12 geocentric Moon example; J2000 high-curvature continuity window"
    ));
    assert!(report.contains(
        "apparent windows: published 1992-04-12 apparent geocentric Moon comparison datum; published 1968-12-24 low-accuracy Meeus-style geocentric Moon example; published 2004-04-01 NASA RP 1349 apparent Moon table row; published 2006-09-07 EclipseWise apparent Moon coordinate row"
    ));
}

#[test]
fn lunar_source_window_validation_rejects_drifted_fields() {
    let mut summary = lunar_source_window_summary().expect("source window summary should exist");
    summary.exact_sample_count += 1;

    let error = summary
        .validate()
        .expect_err("drifted summary should fail validation");
    assert_eq!(
        error,
        LunarSourceWindowSummaryValidationError::FieldOutOfSync {
            field: "exact_sample_count"
        }
    );
    assert!(error
        .to_string()
        .contains("lunar source-window summary field `exact_sample_count`"));
}

#[test]
fn lunar_source_window_request_corpus_matches_the_combined_windows() {
    let requests = lunar_source_window_request_corpus();

    assert_eq!(requests.len(), 11);
    assert!(requests
        .iter()
        .all(|request| request.body == CelestialBody::Moon));
    assert_eq!(
        requests[0].instant,
        Instant::new(
            pleiades_types::JulianDay::from_days(2_448_724.5),
            TimeScale::Tt
        )
    );
    assert_eq!(
        requests[1..7],
        lunar_high_curvature_continuity_requests()[..]
    );
    assert_eq!(requests[7..], lunar_apparent_comparison_requests()[..]);
}

#[test]
fn lunar_high_curvature_request_corpus_helpers_match_the_regression_window() {
    let ecliptic_requests = lunar_high_curvature_continuity_requests();
    let equatorial_requests = lunar_high_curvature_equatorial_continuity_requests();

    assert_eq!(
        ecliptic_requests,
        lunar_high_curvature_continuity_request_corpus()
    );
    assert_eq!(
        ecliptic_requests,
        lunar_high_curvature_continuity_batch_parity_requests()
    );
    assert_eq!(
        ecliptic_requests,
        lunar_high_curvature_continuity_batch_parity_request_corpus()
    );
    assert_eq!(
        equatorial_requests,
        lunar_high_curvature_equatorial_continuity_request_corpus()
    );
    assert_eq!(
        equatorial_requests,
        lunar_high_curvature_equatorial_continuity_batch_parity_requests()
    );
    assert_eq!(
        equatorial_requests,
        lunar_high_curvature_equatorial_continuity_batch_parity_request_corpus()
    );

    for (request, expected_epoch) in ecliptic_requests
        .iter()
        .zip(LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS)
    {
        assert_eq!(request.body, CelestialBody::Moon);
        assert_eq!(request.instant, expected_epoch);
        assert_eq!(request.frame, CoordinateFrame::Ecliptic);
        assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
        assert_eq!(request.apparent, Apparentness::Mean);
        assert!(request.observer.is_none());
    }

    for (request, expected_epoch) in equatorial_requests
        .iter()
        .zip(LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS)
    {
        assert_eq!(request.body, CelestialBody::Moon);
        assert_eq!(request.instant, expected_epoch);
        assert_eq!(request.frame, CoordinateFrame::Equatorial);
        assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
        assert_eq!(request.apparent, Apparentness::Mean);
        assert!(request.observer.is_none());
    }
}

#[test]
fn batch_query_preserves_lunar_high_curvature_continuity_order_and_values() {
    let backend = ElpBackend::new();
    let requests = lunar_high_curvature_continuity_batch_parity_requests();

    let results = backend
        .positions(&requests)
        .expect("batch query should preserve the lunar high-curvature order");

    assert_eq!(results.len(), requests.len());
    for (request, result) in requests.iter().zip(results.iter()) {
        assert_eq!(result.body, request.body);
        assert_eq!(result.instant, request.instant);
        assert_eq!(result.frame, CoordinateFrame::Ecliptic);

        let batch = result.ecliptic.expect("ecliptic result should exist");
        let single = backend
            .position(request)
            .expect("single high-curvature query should succeed")
            .ecliptic
            .expect("single-query ecliptic result should exist");

        assert_eq!(batch, single);
    }
}

#[test]
fn batch_query_preserves_lunar_high_curvature_equatorial_order_and_values() {
    let backend = ElpBackend::new();
    let requests = lunar_high_curvature_equatorial_continuity_batch_parity_requests();

    let results = backend
        .positions(&requests)
        .expect("batch query should preserve the lunar high-curvature equatorial order");

    assert_eq!(results.len(), requests.len());
    for (request, result) in requests.iter().zip(results.iter()) {
        assert_eq!(result.body, request.body);
        assert_eq!(result.instant, request.instant);
        assert_eq!(result.frame, CoordinateFrame::Equatorial);

        let batch = result.equatorial.expect("equatorial result should exist");
        let single = backend
            .position(request)
            .expect("single high-curvature equatorial query should succeed")
            .equatorial
            .expect("single-query equatorial result should exist");

        assert_eq!(batch, single);
    }
}

#[test]
fn lunar_high_curvature_continuity_validation_rejects_stale_counts() {
    let mut envelope = lunar_high_curvature_continuity_envelope().expect("envelope should exist");
    envelope.sample_count = 1;

    assert!(matches!(
        envelope.validate(),
        Err(LunarHighCurvatureEvidenceValidationError::SampleCountTooSmall { sample_count: 1 })
    ));
    assert!(matches!(
        envelope.validated_summary_line(),
        Err(LunarHighCurvatureEvidenceValidationError::SampleCountTooSmall { sample_count: 1 })
    ));
}

#[test]
fn lunar_high_curvature_continuity_validation_rejects_stale_regression_limit_flag() {
    let mut envelope = lunar_high_curvature_continuity_envelope().expect("envelope should exist");
    envelope.within_regression_limits = false;

    assert!(matches!(
        envelope.validate(),
        Err(
            LunarHighCurvatureEvidenceValidationError::RegressionLimitMismatch {
                envelope: "lunar high-curvature continuity evidence",
                within_regression_limits: false,
                expected_within_regression_limits: true,
            }
        )
    ));
}

#[test]
fn lunar_high_curvature_equatorial_continuity_evidence_is_rendered() {
    let envelope = lunar_high_curvature_equatorial_continuity_envelope()
        .expect("equatorial envelope should exist");
    let report = lunar_high_curvature_equatorial_continuity_evidence_for_report();

    assert_eq!(report, envelope.summary_line());
    assert_eq!(envelope.validated_summary_line().unwrap(), report);
    assert!(envelope.validate().is_ok());
    assert!(report.contains(
        "lunar high-curvature equatorial continuity evidence: 6 samples across 1 bodies"
    ));
    assert!(report.contains("epoch range JD 2451544.0 (TT) → JD 2451547.0 (TT)"));
    assert!(report.contains("max adjacent ΔRA="));
    assert!(report.contains("max adjacent ΔDec="));
    assert!(report.contains("max adjacent Δdist="));
    assert!(report.contains("within regression limits=true"));
    assert!(report.contains("ΔRA≤20.0°"));
    assert!(report.contains("ΔDec≤10.0°"));
    assert!(report.contains("Δdist≤0.02 AU"));
}

#[test]
fn lunar_high_curvature_equatorial_continuity_validation_rejects_stale_counts() {
    let mut envelope = lunar_high_curvature_equatorial_continuity_envelope()
        .expect("equatorial envelope should exist");
    envelope.sample_count = 1;

    assert!(matches!(
        envelope.validate(),
        Err(LunarHighCurvatureEvidenceValidationError::SampleCountTooSmall { sample_count: 1 })
    ));
    assert!(matches!(
        envelope.validated_summary_line(),
        Err(LunarHighCurvatureEvidenceValidationError::SampleCountTooSmall { sample_count: 1 })
    ));
}

#[test]
fn lunar_high_curvature_equatorial_continuity_validation_rejects_stale_regression_limit_flag() {
    let mut envelope = lunar_high_curvature_equatorial_continuity_envelope()
        .expect("equatorial envelope should exist");
    envelope.within_regression_limits = false;

    assert!(matches!(
        envelope.validate(),
        Err(
            LunarHighCurvatureEvidenceValidationError::RegressionLimitMismatch {
                envelope: "lunar high-curvature equatorial continuity evidence",
                within_regression_limits: false,
                expected_within_regression_limits: true,
            }
        )
    ));
}

#[test]
fn j2000_mean_and_true_nodes_are_available() {
    let backend = ElpBackend::new();
    let instant = Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt);

    let mean = backend
        .position(&mean_request_at(CelestialBody::MeanNode, instant))
        .expect("mean node query should work");
    let mean_ecliptic = mean.ecliptic.expect("mean node ecliptic should exist");
    assert!((mean_ecliptic.longitude.degrees() - 125.044_547_9).abs() < 1e-9);
    assert_eq!(mean_ecliptic.latitude.degrees(), 0.0);
    assert!(mean.equatorial.is_some());
    let mean_motion = mean.motion.expect("mean node motion should be populated");
    assert!(mean_motion
        .longitude_deg_per_day
        .expect("mean node longitude speed should exist")
        .is_finite());
    assert_eq!(mean_motion.latitude_deg_per_day, Some(0.0));
    assert_eq!(mean_motion.distance_au_per_day, None);

    let true_node = backend
        .position(&mean_request_at(CelestialBody::TrueNode, instant))
        .expect("true node query should work");
    let true_ecliptic = true_node.ecliptic.expect("true node ecliptic should exist");
    assert!((true_ecliptic.longitude.degrees() - 123.926_171_368_400_46).abs() < 1e-9);
    assert_eq!(true_ecliptic.latitude.degrees(), 0.0);
    assert!(true_node.equatorial.is_some());
    let true_motion = true_node
        .motion
        .expect("true node motion should be populated");
    assert!(true_motion
        .longitude_deg_per_day
        .expect("true node longitude speed should exist")
        .is_finite());
    assert_eq!(true_motion.latitude_deg_per_day, Some(0.0));
    assert_eq!(true_motion.distance_au_per_day, None);
}

#[test]
fn j2000_mean_apogee_and_perigee_are_available() {
    let backend = ElpBackend::new();
    let instant = Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt);

    let perigee = backend
        .position(&mean_request_at(CelestialBody::MeanPerigee, instant))
        .expect("mean perigee query should work");
    let perigee_ecliptic = perigee
        .ecliptic
        .expect("mean perigee ecliptic should exist");
    assert!((perigee_ecliptic.longitude.degrees() - 83.353_246_5).abs() < 1e-9);
    assert_eq!(perigee_ecliptic.latitude.degrees(), 0.0);
    assert_eq!(perigee_ecliptic.distance_au, None);
    assert!(perigee.equatorial.is_some());
    let perigee_motion = perigee
        .motion
        .expect("mean perigee motion should be populated");
    assert!(perigee_motion
        .longitude_deg_per_day
        .expect("mean perigee longitude speed should exist")
        .is_finite());
    assert_eq!(perigee_motion.latitude_deg_per_day, Some(0.0));
    assert_eq!(perigee_motion.distance_au_per_day, None);

    let apogee = backend
        .position(&mean_request_at(CelestialBody::MeanApogee, instant))
        .expect("mean apogee query should work");
    let apogee_ecliptic = apogee.ecliptic.expect("mean apogee ecliptic should exist");
    assert!((apogee_ecliptic.longitude.degrees() - 263.353_246_5).abs() < 1e-9);
    assert_eq!(apogee_ecliptic.latitude.degrees(), 0.0);
    assert_eq!(apogee_ecliptic.distance_au, None);
    assert!(apogee.equatorial.is_some());
    let apogee_motion = apogee
        .motion
        .expect("mean apogee motion should be populated");
    assert!(apogee_motion
        .longitude_deg_per_day
        .expect("mean apogee longitude speed should exist")
        .is_finite());
    assert_eq!(apogee_motion.latitude_deg_per_day, Some(0.0));
    assert_eq!(apogee_motion.distance_au_per_day, None);
}

#[test]
fn batch_query_preserves_lunar_reference_order_and_values() {
    let backend = ElpBackend::new();
    let evidence = lunar_reference_evidence();
    let requests = evidence
        .iter()
        .map(|sample| mean_request_at(sample.body.clone(), sample.epoch))
        .collect::<Vec<_>>();

    let results = backend
        .positions(&requests)
        .expect("batch query should preserve the lunar reference order");

    assert_eq!(results.len(), evidence.len());
    for (sample, result) in evidence.iter().zip(results.iter()) {
        assert_eq!(result.body, sample.body);
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");
        let longitude_tolerance = match sample.body {
            CelestialBody::MeanNode => 1e-1,
            _ => 1e-4,
        };
        assert!((ecliptic.longitude.degrees() - sample.longitude_deg).abs() < longitude_tolerance);
        assert!((ecliptic.latitude.degrees() - sample.latitude_deg).abs() < 1e-4);
        assert_eq!(ecliptic.distance_au.is_some(), sample.distance_au.is_some());
        if let (Some(actual), Some(expected)) = (ecliptic.distance_au, sample.distance_au) {
            assert!((actual - expected).abs() < 1e-8);
        }
    }
}

#[test]
fn batch_query_preserves_lunar_reference_order_for_tdb_requests() {
    let backend = ElpBackend::new();
    let evidence = lunar_reference_evidence();
    let requests = evidence
        .iter()
        .map(|sample| {
            let mut request = mean_request_at(sample.body.clone(), sample.epoch);
            request.instant.scale = TimeScale::Tdb;
            request
        })
        .collect::<Vec<_>>();

    let results = backend
        .positions(&requests)
        .expect("batch TDB query should preserve the lunar reference order");

    assert_eq!(results.len(), evidence.len());
    for (sample, result) in evidence.iter().zip(results.iter()) {
        assert_eq!(result.body, sample.body);
        assert_eq!(result.instant.scale, TimeScale::Tdb);
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");
        let longitude_tolerance = match sample.body {
            CelestialBody::MeanNode => 1e-1,
            _ => 1e-4,
        };
        assert!((ecliptic.longitude.degrees() - sample.longitude_deg).abs() < longitude_tolerance);
        assert!((ecliptic.latitude.degrees() - sample.latitude_deg).abs() < 1e-4);
        assert_eq!(ecliptic.distance_au.is_some(), sample.distance_au.is_some());
        if let (Some(actual), Some(expected)) = (ecliptic.distance_au, sample.distance_au) {
            assert!((actual - expected).abs() < 1e-8);
        }
    }
}

#[test]
fn batch_query_preserves_lunar_reference_order_for_mixed_time_scales() {
    let backend = ElpBackend::new();
    let evidence = lunar_reference_evidence();
    let requests = evidence
        .iter()
        .enumerate()
        .map(|(index, sample)| {
            let mut request = mean_request_at(sample.body.clone(), sample.epoch);
            request.instant.scale = if index % 2 == 0 {
                TimeScale::Tt
            } else {
                TimeScale::Tdb
            };
            request
        })
        .collect::<Vec<_>>();

    let results = backend
        .positions(&requests)
        .expect("batch mixed-scale query should preserve the lunar reference order");

    assert_eq!(results.len(), evidence.len());
    for (request, result) in requests.iter().zip(results.iter()) {
        assert_eq!(result.body, request.body);
        assert_eq!(result.instant.scale, request.instant.scale);
        let single = backend
            .position(request)
            .expect("single mixed-scale query should preserve the lunar reference order");
        assert_eq!(single.body, result.body);
        assert_eq!(single.instant.scale, request.instant.scale);
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");
        let single_ecliptic = single
            .ecliptic
            .expect("single-query ecliptic result should exist");
        assert_eq!(
            ecliptic.longitude.degrees(),
            single_ecliptic.longitude.degrees()
        );
        assert_eq!(
            ecliptic.latitude.degrees(),
            single_ecliptic.latitude.degrees()
        );
        assert_eq!(ecliptic.distance_au, single_ecliptic.distance_au);
    }
}

#[test]
fn batch_query_rejects_unsupported_lunar_bodies_with_structured_errors() {
    let backend = ElpBackend::new();
    let instant = Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt);
    let requests = [CelestialBody::TrueApogee, CelestialBody::TruePerigee]
        .into_iter()
        .map(|body| mean_request_at(body, instant))
        .collect::<Vec<_>>();

    let error = backend
        .positions(&requests)
        .expect_err("unsupported lunar bodies should fail explicitly through batch requests");

    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
}

#[test]
fn batch_query_preserves_equatorial_frame_and_values() {
    let backend = ElpBackend::new();
    let evidence = lunar_reference_evidence();
    let requests = evidence
        .iter()
        .map(|sample| {
            let mut request = mean_request_at(sample.body.clone(), sample.epoch);
            request.frame = CoordinateFrame::Equatorial;
            request
        })
        .collect::<Vec<_>>();

    let results = backend
        .positions(&requests)
        .expect("batch equatorial query should preserve the lunar reference order");

    assert_eq!(results.len(), evidence.len());
    for (sample, result) in evidence.iter().zip(results.iter()) {
        assert_eq!(result.body, sample.body);
        assert_eq!(result.frame, CoordinateFrame::Equatorial);

        let ecliptic = result.ecliptic.expect("ecliptic result should exist");
        let equatorial = result.equatorial.expect("equatorial result should exist");

        assert!(equatorial.right_ascension.degrees().is_finite());
        assert!(equatorial.declination.degrees().is_finite());

        if sample.body == CelestialBody::Moon {
            // Moon ecliptic is J2000 boundary; equatorial is derived from of-date ecliptic.
            // The two frames differ, so ecliptic.to_equatorial(mean_obliquity()) != equatorial.
            let j2000_derived = ecliptic.to_equatorial(sample.epoch.mean_obliquity());
            assert_ne!(
                equatorial, j2000_derived,
                "Moon equatorial must NOT be derived from J2000 ecliptic (would mix frames)"
            );
        } else {
            // Non-Moon bodies: ecliptic is of-date and equatorial is consistent.
            let expected = ecliptic.to_equatorial(sample.epoch.mean_obliquity());
            assert_eq!(equatorial, expected);
        }
    }
}

#[test]
fn batch_query_preserves_mixed_frame_requests_and_values() {
    let backend = ElpBackend::new();
    let evidence = lunar_reference_evidence();
    let requests = evidence
        .iter()
        .enumerate()
        .map(|(index, sample)| {
            let mut request = mean_request_at(sample.body.clone(), sample.epoch);
            request.frame = if index % 2 == 0 {
                CoordinateFrame::Ecliptic
            } else {
                CoordinateFrame::Equatorial
            };
            request
        })
        .collect::<Vec<_>>();

    let results = backend
        .positions(&requests)
        .expect("mixed frame batch query should preserve the lunar reference order");

    assert_eq!(results.len(), evidence.len());
    for ((request, result), sample) in requests.iter().zip(results.iter()).zip(evidence.iter()) {
        assert_eq!(result.body, sample.body);
        assert_eq!(result.instant, sample.epoch);
        assert_eq!(result.frame, request.frame);

        let ecliptic = result.ecliptic.expect("ecliptic result should exist");
        let equatorial = result.equatorial.expect("equatorial result should exist");

        assert!(equatorial.right_ascension.degrees().is_finite());
        assert!(equatorial.declination.degrees().is_finite());

        if sample.body == CelestialBody::Moon {
            // Moon ecliptic is J2000 boundary; equatorial is derived from of-date ecliptic.
            // The two frames differ, so ecliptic.to_equatorial(mean_obliquity()) != equatorial.
            let j2000_derived = ecliptic.to_equatorial(sample.epoch.mean_obliquity());
            assert_ne!(
                equatorial, j2000_derived,
                "Moon equatorial must NOT be derived from J2000 ecliptic (would mix frames)"
            );
        } else {
            // Non-Moon bodies: ecliptic is of-date and equatorial is consistent.
            let expected = ecliptic.to_equatorial(sample.epoch.mean_obliquity());
            assert_eq!(equatorial, expected);
        }
    }
}

#[test]
fn backend_supports_lunar_points() {
    let backend = ElpBackend::new();
    let theory = lunar_theory_specification();

    assert_eq!(
        theory.model_name,
        "Compact Meeus-style truncated lunar baseline"
    );
    assert_eq!(
        theory.source_identifier,
        "meeus-style-truncated-lunar-baseline"
    );
    assert_eq!(
        theory.source_citation,
        "Jean Meeus, Astronomical Algorithms, 2nd edition, truncated lunar position and lunar node/perigee/apogee formulae adapted into a compact pure-Rust baseline"
    );
    assert!(theory
        .source_material
        .contains("Published lunar position, node, and mean-point formulas"));
    assert!(theory
        .source_material
        .contains("no ELP coefficient files are bundled in the baseline"));
    assert!(theory
        .redistribution_note
        .contains("does not bundle ELP coefficient tables"));
    assert!(theory.license_note.contains("handwritten pure Rust"));
    assert!(theory.truncation_note.contains("truncated"));
    assert!(theory.unit_note.contains("astronomical units"));
    assert!(theory.date_range_note.contains("1992-04-12"));
    assert!(theory
        .date_range_note
        .contains("1968-12-24 apparent geocentric Moon comparison datum"));
    assert!(theory.date_range_note.contains("J2000 lunar-point anchors"));
    assert!(theory
        .date_range_note
        .contains("2021-03-05 mean-perigee example"));
    assert!(theory.date_range_note.contains("1913-05-27 true-node"));
    assert!(theory.frame_note.contains("mean-obliquity"));
    let frame_summary = lunar_theory_frame_treatment_summary_details();
    assert_eq!(frame_summary.to_string(), frame_summary.summary_line());
    assert_eq!(frame_summary.summary_line(), theory.frame_note);
    assert_eq!(
        lunar_theory_frame_treatment_summary_for_report(),
        frame_summary.to_string()
    );
    assert_eq!(
        lunar_theory_frame_treatment_summary(),
        frame_summary.summary_line()
    );
    assert_eq!(
        theory.validation_window,
        TimeRange::new(
            Some(Instant::new(
                pleiades_types::JulianDay::from_days(2_448_724.5),
                TimeScale::Tt,
            )),
            Some(Instant::new(
                pleiades_types::JulianDay::from_days(2_459_278.5),
                TimeScale::Tt,
            )),
        )
    );
    let capability = lunar_theory_capability_summary();
    assert_eq!(capability.model_name, theory.model_name);
    assert_eq!(capability.source_identifier, theory.source_identifier);
    assert_eq!(capability.source_family, theory.source_family);
    assert_eq!(
        capability.source_family_label,
        lunar_theory_source_family().label()
    );
    assert_eq!(capability.supported_bodies, theory.supported_bodies);
    assert_eq!(capability.unsupported_bodies, theory.unsupported_bodies);
    assert_eq!(
        capability.supported_body_count,
        theory.supported_bodies.len()
    );
    assert_eq!(
        capability.unsupported_body_count,
        theory.unsupported_bodies.len()
    );
    assert_eq!(
        capability.supported_frame_count,
        theory.supported_frames.len()
    );
    assert_eq!(
        capability.supported_time_scale_count,
        theory.supported_time_scales.len()
    );
    assert_eq!(
        capability.supported_zodiac_mode_count,
        theory.supported_zodiac_modes.len()
    );
    assert_eq!(
        capability.supported_apparentness_count,
        theory.supported_apparentness.len()
    );
    assert_eq!(
        capability.supports_topocentric_observer,
        theory.request_policy.supports_topocentric_observer
    );
    assert_eq!(capability.validation_window, theory.validation_window);
    assert_eq!(
        capability.summary_line(),
        format_lunar_theory_capability_summary(&capability)
    );
    assert_eq!(
        capability.validated_summary_line().unwrap(),
        capability.summary_line()
    );
    assert_eq!(capability.to_string(), capability.summary_line());
    assert_eq!(
        lunar_theory_capability_summary_for_report(),
        capability.summary_line()
    );
    assert_eq!(capability.validate(), Ok(()));
    let mut drifted_helper = capability;
    drifted_helper.catalog_validation_ok = !drifted_helper.catalog_validation_ok;
    assert_eq!(
        drifted_helper.validated_summary_line().unwrap_err().to_string(),
        "the lunar capability summary field `catalog_validation_ok` is out of sync with the current selection"
    );
    let mut drifted = capability;
    drifted.catalog_validation_ok = !drifted.catalog_validation_ok;
    assert_eq!(
        drifted.validate(),
        Err(
            LunarTheoryCapabilitySummaryValidationError::FieldOutOfSync {
                field: "catalog_validation_ok",
            }
        )
    );
    assert!(format_lunar_theory_capability_summary(&capability).contains("bodies=5"));
    assert!(format_lunar_theory_capability_summary(&capability)
        .contains("Moon, Mean Node, True Node, Mean Perigee, Mean Apogee"));
    assert!(format_lunar_theory_capability_summary(&capability).contains("unsupported=2"));
    assert!(
        format_lunar_theory_capability_summary(&capability).contains("True Apogee, True Perigee")
    );
    assert!(format_lunar_theory_capability_summary(&capability).contains("frames=2"));
    assert!(format_lunar_theory_capability_summary(&capability).contains("time scales=2"));
    assert!(format_lunar_theory_capability_summary(&capability).contains("zodiac modes=1"));
    assert!(format_lunar_theory_capability_summary(&capability).contains("apparentness=1"));
    assert!(
        format_lunar_theory_capability_summary(&capability).contains("topocentric observer=false")
    );
    assert!(format_lunar_theory_capability_summary(&capability)
        .contains("validation window=JD 2448724.5 (TT) → JD 2459278.5 (TT)"));
    assert!(format_lunar_theory_capability_summary(&capability).contains("catalog validation=ok"));
    assert_eq!(theory.supported_bodies, lunar_theory_supported_bodies());
    assert_eq!(theory.unsupported_bodies, lunar_theory_unsupported_bodies());
    assert_eq!(
        theory.supported_bodies,
        &[
            CelestialBody::Moon,
            CelestialBody::MeanNode,
            CelestialBody::TrueNode,
            CelestialBody::MeanPerigee,
            CelestialBody::MeanApogee,
        ]
    );
    assert_eq!(
        theory.unsupported_bodies,
        &[CelestialBody::TrueApogee, CelestialBody::TruePerigee]
    );
    assert_eq!(
        theory.request_policy.supported_frames,
        &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial]
    );
    assert_eq!(
        theory.request_policy.supported_time_scales,
        &[TimeScale::Tt, TimeScale::Tdb]
    );
    assert_eq!(
        theory.request_policy.supported_zodiac_modes,
        &[ZodiacMode::Tropical]
    );
    assert_eq!(
        theory.request_policy.supported_apparentness,
        &[Apparentness::Mean]
    );
    assert!(!theory.request_policy.supports_topocentric_observer);
    assert!(theory
        .license_note
        .contains("future source-backed lunar theory selection"));

    assert!(backend.supports_body(CelestialBody::Moon));
    assert!(backend.supports_body(CelestialBody::MeanNode));
    assert!(backend.supports_body(CelestialBody::TrueNode));
    assert!(backend.supports_body(CelestialBody::MeanApogee));
    assert!(backend.supports_body(CelestialBody::MeanPerigee));
    assert!(!backend.supports_body(CelestialBody::TrueApogee));
    assert!(!backend.supports_body(CelestialBody::TruePerigee));
    assert!(!backend.supports_body(CelestialBody::Sun));
    assert_eq!(
        backend.metadata().supported_bodies(),
        lunar_theory_supported_bodies()
    );

    let evidence = lunar_reference_evidence();
    assert_eq!(evidence.len(), 9);
    assert_eq!(evidence[0].body, CelestialBody::Moon);
    assert_eq!(evidence[0].epoch.julian_day.days(), 2_448_724.5);
    assert_eq!(evidence[1].body, CelestialBody::MeanNode);
    assert_eq!(evidence[1].epoch.julian_day.days(), J2000);
    assert_eq!(evidence[2].body, CelestialBody::TrueNode);
    assert_eq!(evidence[2].epoch.julian_day.days(), J2000);
    assert_eq!(evidence[3].body, CelestialBody::MeanNode);
    assert_eq!(evidence[3].epoch.julian_day.days(), 2_419_914.5);
    assert_eq!(evidence[4].body, CelestialBody::MeanNode);
    assert_eq!(evidence[4].epoch.julian_day.days(), 2_436_909.5);
    assert_eq!(evidence[5].body, CelestialBody::MeanPerigee);
    assert_eq!(evidence[5].epoch.julian_day.days(), 2_459_278.5);
    assert_eq!(evidence[6].body, CelestialBody::MeanPerigee);
    assert_eq!(evidence[6].epoch.julian_day.days(), J2000);
    assert_eq!(evidence[7].body, CelestialBody::MeanApogee);
    assert_eq!(evidence[7].epoch.julian_day.days(), J2000);
    assert_eq!(evidence[8].body, CelestialBody::TrueNode);
    assert_eq!(evidence[8].epoch.julian_day.days(), 2_419_914.5);
    for body in theory.supported_bodies {
        assert!(evidence.iter().any(|sample| sample.body == *body));
    }

    for sample in evidence {
        let result = backend
            .position(&mean_request_at(sample.body.clone(), sample.epoch))
            .expect("lunar reference sample should be computable");
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");
        let longitude_tolerance = match sample.body {
            CelestialBody::MeanNode => 1e-1,
            _ => 1e-4,
        };
        assert!((ecliptic.longitude.degrees() - sample.longitude_deg).abs() < longitude_tolerance);
        assert!((ecliptic.latitude.degrees() - sample.latitude_deg).abs() < 1e-4);
        assert_eq!(ecliptic.distance_au.is_some(), sample.distance_au.is_some());
        if let (Some(actual), Some(expected)) = (ecliptic.distance_au, sample.distance_au) {
            assert!((actual - expected).abs() < 1e-8);
        }
    }

    let instant = Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt);
    for body in [
        CelestialBody::TrueApogee,
        CelestialBody::TruePerigee,
        CelestialBody::Sun,
    ] {
        let error = backend
            .position(&mean_request_at(body, instant))
            .expect_err("unsupported lunar bodies should fail explicitly");
        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
    }
}

#[test]
fn lunar_reference_batch_requests_match_the_canonical_slice() {
    let requests = lunar_reference_batch_requests();
    let samples = lunar_reference_evidence();

    assert_eq!(requests.len(), samples.len());

    for (index, (request, sample)) in requests.iter().zip(samples.iter()).enumerate() {
        assert_eq!(
            request.body, sample.body,
            "request {index} body should match evidence"
        );
        assert_eq!(
            request.instant.julian_day, sample.epoch.julian_day,
            "request {index} epoch should match evidence"
        );
        assert_eq!(request.frame, CoordinateFrame::Ecliptic);
        assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
        assert_eq!(request.apparent, Apparentness::Mean);
        assert!(request.observer.is_none());
        if index % 2 == 0 {
            assert_eq!(request.instant.scale, TimeScale::Tt);
        } else {
            assert_eq!(request.instant.scale, TimeScale::Tdb);
        }
    }

    assert_eq!(
        requests
            .iter()
            .filter(|request| request.instant.scale == TimeScale::Tt)
            .count(),
        5
    );
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.instant.scale == TimeScale::Tdb)
            .count(),
        4
    );
}

#[test]
fn lunar_equatorial_reference_batch_requests_match_the_canonical_slice() {
    let requests = lunar_equatorial_reference_batch_requests();
    let samples = lunar_equatorial_reference_evidence();

    assert_eq!(requests.len(), samples.len());

    for (index, (request, sample)) in requests.iter().zip(samples.iter()).enumerate() {
        assert_eq!(
            request.body, sample.body,
            "request {index} body should match evidence"
        );
        assert_eq!(
            request.instant.julian_day, sample.epoch.julian_day,
            "request {index} epoch should match evidence"
        );
        assert_eq!(request.frame, CoordinateFrame::Equatorial);
        assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
        assert_eq!(request.apparent, Apparentness::Mean);
        assert!(request.observer.is_none());
        assert_eq!(request.instant.scale, TimeScale::Tt);
    }
}

#[test]
fn lunar_request_corpus_aliases_remain_the_canonical_slices() {
    assert_eq!(
        lunar_reference_batch_parity_requests(),
        lunar_reference_batch_requests()
    );
    assert_eq!(
        lunar_reference_batch_request_corpus(),
        lunar_reference_batch_parity_requests()
    );
    assert_eq!(
        lunar_reference_request_corpus(),
        lunar_reference_batch_requests()
    );
    assert_eq!(
        lunar_reference_batch_parity_request_corpus(),
        lunar_reference_batch_parity_requests()
    );
    assert_eq!(
        lunar_equatorial_reference_batch_parity_requests(),
        lunar_equatorial_reference_batch_requests()
    );
    assert_eq!(
        lunar_equatorial_reference_batch_request_corpus(),
        lunar_equatorial_reference_batch_parity_requests()
    );
    assert_eq!(
        lunar_equatorial_reference_request_corpus(),
        lunar_equatorial_reference_batch_requests()
    );
    assert_eq!(
        lunar_equatorial_reference_batch_parity_request_corpus(),
        lunar_equatorial_reference_batch_parity_requests()
    );
    assert_eq!(
        lunar_apparent_comparison_request_corpus(),
        lunar_apparent_comparison_requests()
    );
    assert_eq!(
        lunar_apparent_comparison_batch_parity_requests(),
        lunar_apparent_comparison_requests()
    );
    assert_eq!(
        lunar_apparent_comparison_batch_parity_request_corpus(),
        lunar_apparent_comparison_batch_parity_requests()
    );
    assert_eq!(
        lunar_apparent_comparison_equatorial_request_corpus(),
        lunar_apparent_comparison_equatorial_requests()
    );
    assert_eq!(
        lunar_apparent_comparison_equatorial_batch_parity_requests(),
        lunar_apparent_comparison_equatorial_requests()
    );
    assert_eq!(
        lunar_apparent_comparison_equatorial_batch_parity_request_corpus(),
        lunar_apparent_comparison_equatorial_batch_parity_requests()
    );
    assert_eq!(
        lunar_high_curvature_request_corpus(),
        lunar_high_curvature_continuity_requests()
    );
    assert_eq!(
        lunar_high_curvature_continuity_request_corpus(),
        lunar_high_curvature_continuity_requests()
    );
    assert_eq!(
        lunar_high_curvature_equatorial_request_corpus(),
        lunar_high_curvature_equatorial_continuity_requests()
    );
    assert_eq!(
        lunar_high_curvature_equatorial_continuity_request_corpus(),
        lunar_high_curvature_equatorial_continuity_requests()
    );
}

#[test]
fn lunar_apparent_comparison_request_corpora_match_the_canonical_slice() {
    let samples = lunar_apparent_comparison_evidence();
    let ecliptic_requests = lunar_apparent_comparison_requests();
    let equatorial_requests = lunar_apparent_comparison_equatorial_requests();

    assert_eq!(ecliptic_requests.len(), samples.len());
    assert_eq!(equatorial_requests.len(), samples.len());

    for (index, (request, sample)) in ecliptic_requests.iter().zip(samples.iter()).enumerate() {
        assert_eq!(
            request.body, sample.body,
            "ecliptic request {index} body should match evidence"
        );
        assert_eq!(
            request.instant.julian_day, sample.epoch.julian_day,
            "ecliptic request {index} epoch should match evidence"
        );
        assert_eq!(request.frame, CoordinateFrame::Ecliptic);
        assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
        assert_eq!(request.apparent, Apparentness::Mean);
        assert!(request.observer.is_none());
        assert_eq!(request.instant.scale, TimeScale::Tt);
    }

    for (index, (request, sample)) in equatorial_requests.iter().zip(samples.iter()).enumerate() {
        assert_eq!(
            request.body, sample.body,
            "equatorial request {index} body should match evidence"
        );
        assert_eq!(
            request.instant.julian_day, sample.epoch.julian_day,
            "equatorial request {index} epoch should match evidence"
        );
        assert_eq!(request.frame, CoordinateFrame::Equatorial);
        assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
        assert_eq!(request.apparent, Apparentness::Mean);
        assert!(request.observer.is_none());
        assert_eq!(request.instant.scale, TimeScale::Tt);
    }
}

#[test]
fn lunar_reference_evidence_summary_matches_the_canonical_slice() {
    let summary = lunar_reference_evidence_summary().expect("reference evidence should exist");

    assert_eq!(summary.sample_count, 9);
    assert_eq!(summary.body_count, 5);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_419_914.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_459_278.5);
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(
        format_lunar_reference_evidence_summary(&summary),
        summary.summary_line()
    );
    assert!(lunar_reference_evidence_summary_for_report().contains("9 samples across 5 bodies"));
    assert!(lunar_reference_evidence_summary_for_report()
        .contains("JD 2419914.5 (TT) → JD 2459278.5 (TT)"));

    let parity = lunar_reference_batch_parity_summary()
        .expect("mixed-scale batch parity evidence should exist");
    assert_eq!(parity.sample_count, 9);
    assert_eq!(parity.body_count, 5);
    assert_eq!(parity.tt_request_count, 5);
    assert_eq!(parity.tdb_request_count, 4);
    assert!(parity.order_preserved);
    assert!(parity.single_query_parity);
    assert_eq!(parity.summary_line(), parity.to_string());
    assert!(parity.validate().is_ok());
    assert_eq!(
        format_lunar_reference_batch_parity_summary(&parity),
        parity.summary_line()
    );
    assert!(lunar_reference_batch_parity_summary_for_report()
        .contains("lunar reference mixed TT/TDB batch parity: 9 requests across 5 bodies"));
    assert!(lunar_reference_batch_parity_summary_for_report().contains("TT requests=5"));
    assert!(lunar_reference_batch_parity_summary_for_report().contains("TDB requests=4"));
    assert!(lunar_reference_batch_parity_summary_for_report().contains("order=preserved"));
    assert!(
        lunar_reference_batch_parity_summary_for_report().contains("single-query parity=preserved")
    );

    let equatorial_parity = lunar_equatorial_reference_batch_parity_summary()
        .expect("equatorial batch parity evidence should exist");
    assert_eq!(equatorial_parity.sample_count, 3);
    assert_eq!(equatorial_parity.body_count, 1);
    assert_eq!(equatorial_parity.frame, CoordinateFrame::Equatorial);
    assert!(equatorial_parity.order_preserved);
    assert!(equatorial_parity.single_query_parity);
    assert_eq!(
        equatorial_parity.summary_line(),
        equatorial_parity.to_string()
    );
    assert!(equatorial_parity.validate().is_ok());
    assert_eq!(
        format_lunar_equatorial_reference_batch_parity_summary(&equatorial_parity),
        equatorial_parity.summary_line()
    );
    assert!(lunar_equatorial_reference_batch_parity_summary_for_report()
        .contains("lunar equatorial reference batch parity: 3 requests across 1 bodies, frame=Equatorial, order=preserved, single-query parity=preserved"));

    let envelope = lunar_reference_evidence_envelope().expect("error envelope should exist");
    assert_eq!(envelope.sample_count, summary.sample_count);
    assert_eq!(envelope.body_count, summary.body_count);
    assert_eq!(envelope.earliest_epoch, summary.earliest_epoch);
    assert_eq!(envelope.latest_epoch, summary.latest_epoch);
    assert!(envelope.max_longitude_delta_deg.is_finite());
    assert!(envelope.mean_longitude_delta_deg.is_finite());
    assert!(envelope.median_longitude_delta_deg.is_finite());
    assert!(envelope.percentile_longitude_delta_deg.is_finite());
    assert!(envelope.max_latitude_delta_deg.is_finite());
    assert!(envelope.mean_latitude_delta_deg.is_finite());
    assert!(envelope.median_latitude_delta_deg.is_finite());
    assert!(envelope.percentile_latitude_delta_deg.is_finite());
    assert_eq!(envelope.outside_current_limits_count, 0);
    assert!(envelope.within_current_limits);
    assert!(
        lunar_reference_evidence_envelope_for_report().contains("lunar reference error envelope")
    );
    assert!(lunar_reference_evidence_envelope_for_report().contains("max Δlon="));
    assert!(lunar_reference_evidence_envelope_for_report().contains("mean Δlon="));
    assert!(lunar_reference_evidence_envelope_for_report().contains("median Δlon="));
    assert!(lunar_reference_evidence_envelope_for_report().contains("p95 Δlon="));
    assert!(lunar_reference_evidence_envelope_for_report().contains("max Δlat="));
    assert!(lunar_reference_evidence_envelope_for_report().contains("mean Δlat="));
    assert!(lunar_reference_evidence_envelope_for_report().contains("median Δlat="));
    assert!(lunar_reference_evidence_envelope_for_report().contains("p95 Δlat="));
    assert!(lunar_reference_evidence_envelope_for_report().contains("outliers=none"));
    assert!(lunar_reference_evidence_envelope_for_report().contains("limits: Δlon≤1e-4°"));
    assert!(lunar_reference_evidence_envelope_for_report().contains("within current limits=true"));
    assert!(lunar_reference_evidence_envelope_for_report().contains("outside current limits=0"));
    assert_eq!(envelope.summary_line(), envelope.to_string());
    assert_eq!(
        format_lunar_reference_evidence_envelope(&envelope),
        envelope.summary_line()
    );
}

#[test]
fn lunar_reference_evidence_summary_validates_against_the_checked_in_slice() {
    let summary = lunar_reference_evidence_summary().expect("reference evidence should exist");

    assert!(summary.validate().is_ok());
    assert_eq!(
        lunar_reference_evidence_summary_for_report(),
        summary.summary_line()
    );

    let mut mutated = summary;
    mutated.sample_count += 1;
    let error = mutated
        .validate()
        .expect_err("mutated reference evidence summaries should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("lunar reference evidence summary mismatch"));
    assert!(error.message.contains("expected"));
    assert!(error.message.contains("found"));
}

#[test]
fn lunar_reference_evidence_envelope_validation_rejects_non_finite_metrics() {
    let mut envelope =
        lunar_reference_evidence_envelope().expect("reference error envelope should exist");
    envelope.mean_longitude_delta_deg = f64::NAN;

    let error = envelope
        .validate()
        .expect_err("mutated reference envelopes should fail validation");

    assert_eq!(
        error,
        LunarEvidenceEnvelopeValidationError::NonFiniteMeasure {
            envelope: "lunar reference error envelope",
            field: "mean_longitude_delta_deg",
        }
    );
}

#[test]
fn lunar_reference_batch_parity_summary_validation_rejects_count_drift() {
    let mut parity = lunar_reference_batch_parity_summary()
        .expect("mixed-scale batch parity evidence should exist");
    parity.tt_request_count -= 1;

    let error = parity
        .validate()
        .expect_err("drifted mixed-scale parity evidence should fail validation");

    assert_eq!(
        error,
        LunarReferenceBatchParitySummaryValidationError::FieldOutOfSync {
            field: "tt_request_count"
        }
    );
}

#[test]
fn lunar_equatorial_reference_batch_parity_summary_validation_rejects_count_drift() {
    let mut parity = lunar_equatorial_reference_batch_parity_summary()
        .expect("equatorial batch parity evidence should exist");
    parity.sample_count += 1;

    let error = parity
        .validate()
        .expect_err("drifted equatorial parity evidence should fail validation");

    assert_eq!(
        error,
        LunarEquatorialReferenceBatchParitySummaryValidationError::FieldOutOfSync {
            field: "sample_count"
        }
    );
}

#[test]
fn lunar_equatorial_reference_evidence_matches_the_canonical_slice() {
    let summary = lunar_equatorial_reference_evidence_summary()
        .expect("equatorial reference evidence should exist");

    assert_eq!(summary.sample_count, 3);
    assert_eq!(summary.body_count, 1);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_448_724.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_448_724.5);
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(
        format_lunar_equatorial_reference_evidence_summary(&summary),
        summary.summary_line()
    );
    assert!(lunar_equatorial_reference_evidence_summary_for_report()
        .contains("3 samples across 1 bodies"));
    assert!(lunar_equatorial_reference_evidence_summary_for_report()
        .contains("JD 2448724.5 (TT) → JD 2448724.5 (TT)"));

    let envelope = lunar_equatorial_reference_evidence_envelope()
        .expect("equatorial error envelope should exist");
    assert_eq!(envelope.sample_count, summary.sample_count);
    assert_eq!(envelope.body_count, summary.body_count);
    assert_eq!(envelope.earliest_epoch, summary.earliest_epoch);
    assert_eq!(envelope.latest_epoch, summary.latest_epoch);
    assert!(envelope.max_right_ascension_delta_deg.is_finite());
    assert!(envelope.mean_right_ascension_delta_deg.is_finite());
    assert!(envelope.median_right_ascension_delta_deg.is_finite());
    assert!(envelope.percentile_right_ascension_delta_deg.is_finite());
    assert!(envelope.max_declination_delta_deg.is_finite());
    assert!(envelope.mean_declination_delta_deg.is_finite());
    assert!(envelope.median_declination_delta_deg.is_finite());
    assert!(envelope.percentile_declination_delta_deg.is_finite());
    assert_eq!(envelope.outside_current_limits_count, 0);
    assert!(envelope.within_current_limits);
    assert!(lunar_equatorial_reference_evidence_envelope_for_report()
        .contains("lunar equatorial reference error envelope"));
    assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("max ΔRA="));
    assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("mean ΔRA="));
    assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("median ΔRA="));
    assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("p95 ΔRA="));
    assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("max ΔDec="));
    assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("mean ΔDec="));
    assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("median ΔDec="));
    assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("p95 ΔDec="));
    assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("limits: ΔRA≤1e-2°"));
    assert!(lunar_equatorial_reference_evidence_envelope_for_report()
        .contains("within current limits=true"));
    assert!(lunar_equatorial_reference_evidence_envelope_for_report()
        .contains("outside current limits=0"));
    assert_eq!(envelope.summary_line(), envelope.to_string());
    assert_eq!(
        format_lunar_equatorial_reference_evidence_envelope(&envelope),
        envelope.summary_line()
    );

    for sample in lunar_equatorial_reference_evidence() {
        assert_eq!(sample.body, CelestialBody::Moon);
        let result = ElpBackend::new()
            .position(&EphemerisRequest::new(sample.body.clone(), sample.epoch))
            .expect("equatorial reference sample should remain computable");
        let equatorial = result
            .equatorial
            .expect("equatorial reference sample should include equatorial coordinates");
        assert!(
            (equatorial.right_ascension.degrees() - sample.equatorial.right_ascension.degrees())
                .abs()
                < 1e-2
        );
        assert!(
            (equatorial.declination.degrees() - sample.equatorial.declination.degrees()).abs()
                < 1e-2
        );
        assert_eq!(
            equatorial.distance_au.is_some(),
            sample.equatorial.distance_au.is_some()
        );
        if let (Some(actual), Some(expected)) =
            (equatorial.distance_au, sample.equatorial.distance_au)
        {
            assert!((actual - expected).abs() < 1e-8);
        }
    }
}

#[test]
fn lunar_equatorial_reference_evidence_summary_validates_against_the_checked_in_slice() {
    let summary = lunar_equatorial_reference_evidence_summary()
        .expect("equatorial reference evidence should exist");

    assert!(summary.validate().is_ok());
    assert_eq!(
        lunar_equatorial_reference_evidence_summary_for_report(),
        summary.summary_line()
    );

    let mut mutated = summary;
    mutated.body_count += 1;
    let error = mutated
        .validate()
        .expect_err("mutated equatorial reference summaries should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("lunar equatorial reference evidence summary mismatch"));
    assert!(error.message.contains("expected"));
    assert!(error.message.contains("found"));
}

#[test]
fn lunar_equatorial_reference_evidence_envelope_validation_rejects_outlier_drift() {
    let mut envelope = lunar_equatorial_reference_evidence_envelope()
        .expect("equatorial error envelope should exist");
    envelope.outside_current_limits_count = 1;
    envelope.within_current_limits = false;
    envelope.outlier_bodies.clear();

    let error = envelope
        .validate()
        .expect_err("mutated equatorial envelopes should fail validation");

    assert_eq!(
        error,
        LunarEvidenceEnvelopeValidationError::OutlierCountMismatch {
            envelope: "lunar equatorial reference error envelope",
            outside_current_limits_count: 1,
            sample_count: envelope.sample_count,
            outlier_bodies_len: 0,
        }
    );
}

#[test]
fn lunar_apparent_comparison_evidence_documents_the_mean_gap() {
    let summary =
        lunar_apparent_comparison_summary().expect("apparent comparison evidence should exist");

    assert_eq!(summary.sample_count, 4);
    assert_eq!(summary.body_count, 1);
    assert!((summary.earliest_epoch.julian_day.days() - 2_440_214.916_7).abs() < 1e-9);
    assert!((summary.latest_epoch.julian_day.days() - 2_453_986.285_649).abs() < 1e-9);
    assert!(summary.max_ecliptic_longitude_delta_deg.is_finite());
    assert!(summary.mean_ecliptic_longitude_delta_deg.is_finite());
    assert!(summary.median_ecliptic_longitude_delta_deg.is_finite());
    assert!(summary.percentile_ecliptic_longitude_delta_deg.is_finite());
    assert!(summary.max_ecliptic_latitude_delta_deg.is_finite());
    assert!(summary.mean_ecliptic_latitude_delta_deg.is_finite());
    assert!(summary.median_ecliptic_latitude_delta_deg.is_finite());
    assert!(summary.percentile_ecliptic_latitude_delta_deg.is_finite());
    assert!(summary.max_ecliptic_distance_delta_au.is_finite());
    assert!(summary.mean_ecliptic_distance_delta_au.is_finite());
    assert!(summary.median_ecliptic_distance_delta_au.is_finite());
    assert!(summary.percentile_ecliptic_distance_delta_au.is_finite());
    assert!(summary.max_right_ascension_delta_deg.is_finite());
    assert!(summary.mean_right_ascension_delta_deg.is_finite());
    assert!(summary.median_right_ascension_delta_deg.is_finite());
    assert!(summary.percentile_right_ascension_delta_deg.is_finite());
    assert!(summary.max_declination_delta_deg.is_finite());
    assert!(summary.mean_declination_delta_deg.is_finite());
    assert!(summary.median_declination_delta_deg.is_finite());
    assert!(summary.percentile_declination_delta_deg.is_finite());
    let known_epochs = [2_440_214.916_7, 2_448_724.5, 2_453_100.5, 2_453_986.285_649];
    assert!(known_epochs.contains(&summary.max_ecliptic_longitude_epoch.julian_day.days()));
    assert!(known_epochs.contains(&summary.max_ecliptic_latitude_epoch.julian_day.days()));
    assert!(known_epochs.contains(&summary.max_ecliptic_distance_epoch.julian_day.days()));
    assert!(known_epochs.contains(&summary.max_right_ascension_epoch.julian_day.days()));
    assert!(known_epochs.contains(&summary.max_declination_epoch.julian_day.days()));
    assert!(lunar_apparent_comparison_summary_for_report()
        .contains("lunar apparent comparison evidence: 4 reference-only samples across 1 bodies"));
    assert!(lunar_apparent_comparison_summary_for_report()
        .contains("mean-only gap against the published apparent Moon examples"));
    assert!(lunar_apparent_comparison_summary_for_report().contains("|Δlon| mean/median/p95="));
    assert!(lunar_apparent_comparison_summary_for_report().contains("|ΔDec| mean/median/p95="));
    assert!(lunar_apparent_comparison_summary_for_report().contains("@ JD"));
    assert!(lunar_apparent_comparison_summary_for_report()
        .contains("apparent requests remain unsupported"));
    assert_eq!(summary.summary_line(), summary.to_string());

    let samples = lunar_apparent_comparison_evidence();
    assert_eq!(samples.len(), 4);
    assert_eq!(samples[0].body, CelestialBody::Moon);
    assert_eq!(samples[0].epoch.julian_day.days(), 2_448_724.5);
    assert!(samples[0]
        .note
        .contains("reference-only mean/apparent comparison datum"));
    assert_eq!(samples[1].body, CelestialBody::Moon);
    assert_eq!(samples[1].epoch.julian_day.days(), 2_440_214.916_7);
    assert!(samples[1]
        .note
        .contains("second reference-only mean/apparent comparison datum"));
    assert_eq!(samples[2].body, CelestialBody::Moon);
    assert_eq!(samples[2].epoch.julian_day.days(), 2_453_100.5);
    assert!(samples[2].note.contains("NASA RP 1349"));
    assert!(samples[2].note.contains("shared mean-obliquity transform"));
    assert_eq!(samples[3].body, CelestialBody::Moon);
    assert_eq!(samples[3].epoch.julian_day.days(), 2_453_986.285_649);
    assert!(samples[3].note.contains("EclipseWise"));
    assert!(samples[3].note.contains("shared mean-obliquity transform"));
    assert!(samples[3].summary_line().contains("body=Moon"));
    assert!(samples[3].summary_line().contains("EclipseWise"));
    assert!(samples[3]
        .to_string()
        .contains("shared mean-obliquity transform"));
}

#[test]
fn lunar_evidence_sample_validators_reject_drifted_metadata() {
    let mut reference = lunar_reference_evidence()[0].clone();
    reference.body = CelestialBody::Sun;
    assert_eq!(
        reference.validate().unwrap_err().kind,
        EphemerisErrorKind::InvalidRequest
    );

    reference = lunar_reference_evidence()[0].clone();
    reference.epoch = Instant::new(
        pleiades_types::JulianDay::from_days(reference.epoch.julian_day.days()),
        TimeScale::Tdb,
    );
    assert_eq!(
        reference.validate().unwrap_err().kind,
        EphemerisErrorKind::InvalidRequest
    );

    let mut equatorial = lunar_equatorial_reference_evidence()[0].clone();
    equatorial.body = CelestialBody::Sun;
    assert_eq!(
        equatorial.validate().unwrap_err().kind,
        EphemerisErrorKind::InvalidRequest
    );

    equatorial = lunar_equatorial_reference_evidence()[0].clone();
    equatorial.note = " ";
    assert_eq!(
        equatorial.validate().unwrap_err().kind,
        EphemerisErrorKind::InvalidRequest
    );

    let mut apparent = lunar_apparent_comparison_evidence()[0].clone();
    apparent.body = CelestialBody::Sun;
    assert_eq!(
        apparent.validate().unwrap_err().kind,
        EphemerisErrorKind::InvalidRequest
    );

    apparent = lunar_apparent_comparison_evidence()[0].clone();
    apparent.apparent_distance_au = f64::NAN;
    assert_eq!(
        apparent.validate().unwrap_err().kind,
        EphemerisErrorKind::InvalidRequest
    );
}

#[test]
fn lunar_reference_rows_expose_compact_summary_lines() {
    let reference = lunar_reference_evidence()[0].clone();
    assert_eq!(reference.summary_line(), reference.to_string());
    assert!(reference
        .summary_line()
        .contains("Published 1992-04-12 geocentric Moon example"));
    // Updated to J2000 boundary ecliptic longitude (was 133.162655000000° of-date).
    assert!(reference.summary_line().contains("lon=133.270485958000°"));

    let equatorial = lunar_equatorial_reference_evidence()[0].clone();
    assert_eq!(equatorial.summary_line(), equatorial.to_string());
    assert!(equatorial
        .summary_line()
        .contains("Published 1992-04-12 geocentric Moon RA/Dec example"));
    assert!(equatorial.summary_line().contains("ra=134.688470000000°"));
}

#[test]
fn lunar_apparent_comparison_summary_validates_against_the_checked_in_slice() {
    let summary =
        lunar_apparent_comparison_summary().expect("apparent comparison evidence should exist");

    assert!(summary.validate().is_ok());
    assert_eq!(
        lunar_apparent_comparison_summary_for_report(),
        summary.summary_line()
    );

    let mut mutated = summary;
    mutated.sample_count += 1;
    let error = mutated
        .validate()
        .expect_err("mutated apparent comparison summaries should fail validation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(error
        .message
        .contains("lunar apparent comparison evidence summary mismatch"));
    assert!(error.message.contains("expected"));
    assert!(error.message.contains("found"));
}

#[test]
fn apparent_requests_are_rejected_explicitly() {
    let backend = ElpBackend::new();
    let mut request = EphemerisRequest::new(
        CelestialBody::Moon,
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
    );
    request.apparent = Apparentness::Apparent;

    let error = backend
        .position(&request)
        .expect_err("apparent requests should be unsupported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
}

#[test]
fn batch_query_rejects_apparent_requests_explicitly() {
    let backend = ElpBackend::new();
    let mut request = mean_request(CelestialBody::Moon);
    request.apparent = Apparentness::Apparent;

    let error = backend
        .positions(&[request])
        .expect_err("apparent batch requests should be unsupported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
}

#[test]
fn unsupported_time_scales_are_rejected_explicitly() {
    let backend = ElpBackend::new();
    let request = EphemerisRequest::new(
        CelestialBody::Moon,
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Utc),
    );

    let error = backend
        .position(&request)
        .expect_err("UTC requests should be unsupported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedTimeScale);
}

#[test]
fn batch_query_rejects_unsupported_time_scales_explicitly() {
    let backend = ElpBackend::new();
    let request = EphemerisRequest::new(
        CelestialBody::Moon,
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Utc),
    );

    let error = backend
        .positions(&[request])
        .expect_err("UTC batch requests should be unsupported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedTimeScale);
}

#[test]
fn tdb_requests_are_accepted_like_tt_requests() {
    let backend = ElpBackend::new();
    let tt_request = mean_request(CelestialBody::Moon);
    let tdb_request = EphemerisRequest::new(
        CelestialBody::Moon,
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tdb),
    );

    let tt_result = backend
        .position(&tt_request)
        .expect("TT request should be supported");
    let tdb_result = backend
        .position(&tdb_request)
        .expect("TDB request should be supported");

    assert_eq!(tt_result.body, tdb_result.body);
    assert_eq!(tt_result.instant.scale, TimeScale::Tt);
    assert_eq!(tdb_result.instant.scale, TimeScale::Tdb);
    assert_eq!(tt_result.ecliptic, tdb_result.ecliptic);
    assert_eq!(tt_result.equatorial, tdb_result.equatorial);
    assert_eq!(tt_result.motion, tdb_result.motion);
}

#[test]
fn topocentric_requests_are_rejected_explicitly() {
    let backend = ElpBackend::new();
    let mut request = mean_request(CelestialBody::Moon);
    request.observer = Some(pleiades_types::ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(0.0),
        None,
    ));

    let error = backend
        .position(&request)
        .expect_err("topocentric requests should be unsupported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
}

#[test]
fn batch_query_rejects_topocentric_requests_explicitly() {
    let backend = ElpBackend::new();
    let mut request = mean_request(CelestialBody::Moon);
    request.observer = Some(pleiades_types::ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(0.0),
        None,
    ));

    let error = backend
        .positions(&[request])
        .expect_err("topocentric batch requests should be unsupported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
}

#[test]
fn elp_claims_lunar_constrained_true_apsides_unsupported() {
    use pleiades_backend::{BodyClaimTier, CelestialBody, EphemerisBackend};
    let meta = ElpBackend::new().metadata();
    assert_eq!(
        meta.claim_for(&CelestialBody::Moon).map(|c| c.tier),
        Some(BodyClaimTier::Constrained)
    );
    assert_eq!(
        meta.claim_for(&CelestialBody::TrueApogee).map(|c| c.tier),
        Some(BodyClaimTier::Unsupported)
    );
    assert!(!meta.supported_bodies().contains(&CelestialBody::TrueApogee));
    assert!(meta.release_grade_bodies().is_empty());
}

fn mean_request(body: CelestialBody) -> EphemerisRequest {
    mean_request_at(
        body,
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
    )
}

fn mean_request_at(body: CelestialBody, instant: Instant) -> EphemerisRequest {
    let mut request = EphemerisRequest::new(body, instant);
    request.apparent = Apparentness::Mean;
    request
}

#[test]
fn moon_boundary_longitude_is_j2000_not_of_date() {
    use pleiades_backend::{EphemerisBackend, EphemerisRequest};
    use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
    let backend = crate::ElpBackend::new();
    let inst = Instant::new(JulianDay::from_days(2_415_025.5), TimeScale::Tt);
    let res = backend
        .position(&EphemerisRequest::new(CelestialBody::Moon, inst))
        .unwrap();
    let lon = res.ecliptic.unwrap().longitude.degrees();
    // J2000 longitude differs from the raw of-date series by ~+1.4° (precession).
    let of_date = crate::data::moonposition::position(2_415_025.5).0.degrees();
    assert!(
        (lon - of_date).abs() > 1.0,
        "ELP Moon not precessed to J2000: {lon} vs of-date {of_date}"
    );
}

#[test]
fn elp_moon_round_trips_to_of_date_through_the_pipeline() {
    let jd = 2_415_025.5;
    let days = jd - crate::J2000;
    let j2000 = crate::backend::ElpBackend::moon_ecliptic_coordinates(days);
    let redate = pleiades_apparent::precess_ecliptic_j2000_to_date(
        j2000.longitude.degrees(),
        j2000.latitude.degrees(),
        jd,
    )
    .unwrap();
    let (od_lon, od_lat, _) = crate::data::moonposition::position(jd);
    assert!(
        (redate.longitude_deg - od_lon.degrees()).abs() * 3600.0 < 1e-3,
        "lon residual arcsec: {}",
        (redate.longitude_deg - od_lon.degrees()).abs() * 3600.0
    );
    assert!(
        (redate.latitude_deg - od_lat.degrees()).abs() * 3600.0 < 1e-3,
        "lat residual arcsec: {}",
        (redate.latitude_deg - od_lat.degrees()).abs() * 3600.0
    );
}

