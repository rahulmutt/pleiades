use pleiades_types::{Angle, Ayanamsa, Instant, JulianDay, TimeScale};

use crate::{
    baseline_ayanamsas, built_in_ayanamsas, descriptor, release_ayanamsas, resolve_ayanamsa,
    sidereal_offset,
};

#[test]
fn baseline_catalog_includes_required_milestone_entries() {
    let names: Vec<_> = baseline_ayanamsas()
        .iter()
        .map(|entry| entry.canonical_name)
        .collect();

    for expected in [
        "Lahiri",
        "Raman",
        "Krishnamurti",
        "Fagan/Bradley",
        "True Chitra",
    ] {
        assert!(names.contains(&expected), "missing {expected}");
    }
}

#[test]
fn reference_epoch_offsets_match_the_documented_baseline_values() {
    for body in [
        Ayanamsa::Lahiri,
        Ayanamsa::Raman,
        Ayanamsa::Krishnamurti,
        Ayanamsa::FaganBradley,
        Ayanamsa::TrueChitra,
    ] {
        let d = descriptor(&body).expect("baseline ayanamsa should resolve");
        let epoch = Instant::new(
            d.epoch.expect("baseline ayanamsa should carry an epoch"),
            TimeScale::Tt,
        );
        let offset = d
            .offset_at(epoch)
            .expect("baseline ayanamsa should carry an offset");
        assert!(
            (offset.degrees() - d.offset_degrees.unwrap().degrees()).abs() < 1.0e-12,
            "expected {}, got {}",
            d.offset_degrees.unwrap().degrees(),
            offset.degrees()
        );
    }
}

#[test]
fn release_catalog_includes_stage_six_ayanamsa_variants() {
    let names: Vec<_> = release_ayanamsas()
        .iter()
        .map(|entry| entry.canonical_name)
        .collect();

    for expected in [
        "True Citra",
        "J2000",
        "J1900",
        "B1950",
        "True Revati",
        "True Mula",
        "Suryasiddhanta (Revati)",
        "Suryasiddhanta (Citra)",
        "Lahiri (ICRC)",
        "Lahiri (1940)",
        "Usha Shashi",
        "Suryasiddhanta (499 CE)",
        "Aryabhata (499 CE)",
        "Sassanian",
        "DeLuce",
        "Yukteshwar",
        "PVR Pushya-paksha",
        "Sheoran",
        "Hipparchus",
        "Babylonian (Kugler 1)",
        "Babylonian (Kugler 2)",
        "Babylonian (Kugler 3)",
        "Babylonian (Huber)",
        "Babylonian (Eta Piscium)",
        "Babylonian (Aldebaran)",
        "Babylonian (House)",
        "Babylonian (Sissy)",
        "Babylonian (True Geoc)",
        "Babylonian (True Topc)",
        "Babylonian (True Obs)",
        "Babylonian (House Obs)",
        "True Pushya",
        "Udayagiri",
        "Djwhal Khul",
        "JN Bhasin",
        "Suryasiddhanta (Mean Sun)",
        "Aryabhata (Mean Sun)",
        "Babylonian (Britton)",
        "Aryabhata (522 CE)",
        "Lahiri (VP285)",
        "Krishnamurti (VP291)",
        "True Sheoran",
        "Galactic Center",
        "Galactic Center (Rgilbrand)",
        "Galactic Center (Mardyks)",
        "Galactic Center (Mula/Wilhelm)",
        "Dhruva Galactic Center (Middle Mula)",
        "Galactic Center (Cochrane)",
        "Galactic Equator",
        "Galactic Equator (IAU 1958)",
        "Galactic Equator (True)",
        "Galactic Equator (Mula)",
        "Galactic Equator (Fiorenza)",
        "Valens Moon",
    ] {
        assert!(names.contains(&expected), "missing {expected}");
    }
}

#[test]
fn release_descriptor_aliases_do_not_repeat_canonical_labels() {
    assert!(built_in_ayanamsas()
        .iter()
        .all(|entry| { !entry.aliases.contains(&entry.canonical_name) }));
}

#[test]
fn ayanamsa_catalog_round_trips_all_built_ins_and_aliases() {
    use std::collections::HashSet;

    let built_in = built_in_ayanamsas();
    let mut unique_names = HashSet::new();

    assert_eq!(
        built_in.len(),
        baseline_ayanamsas().len() + release_ayanamsas().len()
    );

    for entry in baseline_ayanamsas()
        .iter()
        .chain(release_ayanamsas().iter())
    {
        assert!(
            unique_names.insert(entry.canonical_name),
            "duplicate canonical ayanamsa name {}",
            entry.canonical_name
        );
        assert_eq!(
            descriptor(&entry.ayanamsa).map(|d| d.canonical_name),
            Some(entry.canonical_name)
        );
        assert_eq!(
            resolve_ayanamsa(entry.canonical_name),
            Some(entry.ayanamsa.clone())
        );
        for alias in entry.aliases {
            assert_eq!(resolve_ayanamsa(alias), Some(entry.ayanamsa.clone()));
        }
    }

    for entry in built_in {
        assert!(unique_names.contains(entry.canonical_name));
    }
}

#[test]
fn selected_release_ayanamsas_carry_reference_metadata() {
    let hipparchus = descriptor(&Ayanamsa::Hipparchus).expect("Hipparchus descriptor");
    assert_eq!(hipparchus.epoch, Some(JulianDay::from_days(1_674_484.0)));
    assert_eq!(
        hipparchus.offset_degrees,
        Some(Angle::from_degrees(-9.333_333_333_333_334))
    );

    let jn_bhasin = descriptor(&Ayanamsa::JnBhasin).expect("JN Bhasin descriptor");
    assert_eq!(jn_bhasin.epoch, Some(JulianDay::from_days(2_415_020.0)));
    assert_eq!(
        jn_bhasin.offset_degrees,
        Some(Angle::from_degrees(360.0 - 338.634444))
    );

    let true_citra = descriptor(&Ayanamsa::TrueCitra).expect("True Citra descriptor");
    assert_eq!(
        true_citra.epoch,
        Some(JulianDay::from_days(1_825_182.872_330))
    );
    assert_eq!(
        true_citra.offset_degrees,
        Some(Angle::from_degrees(50.256_748_3))
    );
    // True-star modes (TrueChitra/TrueCitra) compute sidereal_offset from the committed
    // SE cubic fit (valid 1900–2100); the descriptor (epoch, offset) above is reference
    // metadata only and is intentionally NOT reproduced by sidereal_offset at the ancient
    // reference epoch (see truestar.rs).

    let kugler1 = descriptor(&Ayanamsa::BabylonianKugler1).expect("Babylonian Kugler 1 descriptor");
    assert_eq!(kugler1.epoch, Some(JulianDay::from_days(1_833_923.577_692)));
    assert_eq!(kugler1.offset_degrees, Some(Angle::from_degrees(0.0)));

    let kugler2 = descriptor(&Ayanamsa::BabylonianKugler2).expect("Babylonian Kugler 2 descriptor");
    assert_eq!(kugler2.epoch, Some(JulianDay::from_days(1_797_039.206_820)));
    assert_eq!(kugler2.offset_degrees, Some(Angle::from_degrees(0.0)));

    let eta_piscium =
        descriptor(&Ayanamsa::BabylonianEtaPiscium).expect("Babylonian Eta Piscium descriptor");
    assert_eq!(
        eta_piscium.epoch,
        Some(JulianDay::from_days(1_807_871.964_797))
    );
    assert_eq!(eta_piscium.offset_degrees, Some(Angle::from_degrees(0.0)));

    let aldebaran =
        descriptor(&Ayanamsa::BabylonianAldebaran).expect("Babylonian Aldebaran descriptor");
    assert_eq!(
        aldebaran.epoch,
        Some(JulianDay::from_days(1_801_643.133_503))
    );
    assert_eq!(aldebaran.offset_degrees, Some(Angle::from_degrees(0.0)));

    let galactic_true =
        descriptor(&Ayanamsa::GalacticEquatorTrue).expect("Galactic Equator (True) descriptor");
    assert_eq!(
        galactic_true.epoch,
        Some(JulianDay::from_days(1_665_728.603_158))
    );
    assert_eq!(galactic_true.offset_degrees, Some(Angle::from_degrees(0.0)));

    let galactic = descriptor(&Ayanamsa::GalacticEquatorIau1958)
        .expect("Galactic Equator (IAU 1958) descriptor");
    assert_eq!(
        galactic.epoch,
        Some(JulianDay::from_days(1_667_118.376_332))
    );
    assert_eq!(galactic.offset_degrees, Some(Angle::from_degrees(0.0)));

    let galactic_true =
        descriptor(&Ayanamsa::GalacticEquatorTrue).expect("Galactic Equator (True) descriptor");
    assert_eq!(
        galactic_true.epoch,
        Some(JulianDay::from_days(1_665_728.603_158))
    );
    assert_eq!(galactic_true.offset_degrees, Some(Angle::from_degrees(0.0)));

    let galactic_mula =
        descriptor(&Ayanamsa::GalacticEquatorMula).expect("Galactic Equator (Mula) descriptor");
    assert_eq!(
        galactic_mula.epoch,
        Some(JulianDay::from_days(1_840_527.426_262))
    );
    assert_eq!(galactic_mula.offset_degrees, Some(Angle::from_degrees(0.0)));

    let valens = descriptor(&Ayanamsa::ValensMoon).expect("Valens Moon descriptor");
    assert_eq!(valens.epoch, Some(JulianDay::from_days(1_775_845.5)));
    assert_eq!(valens.offset_degrees, Some(Angle::from_degrees(-2.942_2)));

    let fiorenza = descriptor(&Ayanamsa::GalacticEquatorFiorenza)
        .expect("Galactic Equator (Fiorenza) descriptor");
    assert_eq!(fiorenza.epoch, Some(JulianDay::from_days(2_451_544.5)));
    assert_eq!(fiorenza.offset_degrees, Some(Angle::from_degrees(25.0)));
    // GalacticEquatorFiorenza is now routed through the committed cubic polynomial.
    // The polynomial value at the epoch JD differs from 25.0 by < 1e-7 degrees
    // (sub-arcsecond), which is well within the gate ceiling.
    let fiorenza_offset = sidereal_offset(
        &Ayanamsa::GalacticEquatorFiorenza,
        Instant::new(JulianDay::from_days(2_451_544.5), TimeScale::Tt),
    );
    assert!(
        fiorenza_offset.is_some(),
        "GalacticEquatorFiorenza at epoch should return Some"
    );
    assert!(
        (fiorenza_offset.unwrap().degrees() - 25.0).abs() < 1e-6,
        "GalacticEquatorFiorenza at epoch should be ~25.0 degrees (polynomial); got {:?}",
        fiorenza_offset
    );

    let udayagiri = descriptor(&Ayanamsa::Udayagiri).expect("Udayagiri descriptor");
    assert_eq!(
        udayagiri.epoch,
        Some(JulianDay::from_days(1_825_235.164_583))
    );
    assert_eq!(udayagiri.offset_degrees, Some(Angle::from_degrees(0.0)));

    let vp285 = descriptor(&Ayanamsa::LahiriVP285).expect("Lahiri VP285 descriptor");
    assert_eq!(vp285.epoch, Some(JulianDay::from_days(1_825_235.164_583)));
    assert_eq!(vp285.offset_degrees, Some(Angle::from_degrees(0.0)));

    let kugler3 = descriptor(&Ayanamsa::BabylonianKugler3).expect("Babylonian Kugler 3 descriptor");
    assert_eq!(kugler3.epoch, Some(JulianDay::from_days(1_774_637.420_172)));
    assert_eq!(kugler3.offset_degrees, Some(Angle::from_degrees(0.0)));

    let britton = descriptor(&Ayanamsa::BabylonianBritton).expect("Babylonian Britton descriptor");
    assert_eq!(britton.epoch, Some(JulianDay::from_days(1_805_415.712_776)));
    assert_eq!(britton.offset_degrees, Some(Angle::from_degrees(0.0)));

    let cochrane = descriptor(&Ayanamsa::GalacticCenterCochrane)
        .expect("Galactic Center (Cochrane) descriptor");
    assert_eq!(
        cochrane.epoch,
        Some(JulianDay::from_days(1_662_951.794_251))
    );
    assert_eq!(cochrane.offset_degrees, Some(Angle::from_degrees(0.0)));

    let mardyks =
        descriptor(&Ayanamsa::GalacticCenterMardyks).expect("Galactic Center (Mardyks) descriptor");
    assert_eq!(mardyks.epoch, Some(JulianDay::from_days(1_662_951.794_251)));
    assert_eq!(mardyks.offset_degrees, Some(Angle::from_degrees(0.0)));

    let true_pushya = descriptor(&Ayanamsa::TruePushya).expect("True Pushya descriptor");
    assert_eq!(
        true_pushya.epoch,
        Some(JulianDay::from_days(1_855_769.248_315))
    );
    assert_eq!(true_pushya.offset_degrees, Some(Angle::from_degrees(0.0)));

    let ss_revati =
        descriptor(&Ayanamsa::SuryasiddhantaRevati).expect("Suryasiddhanta Revati descriptor");
    assert_eq!(
        ss_revati.epoch,
        Some(JulianDay::from_days(1_903_396.8128654))
    );
    assert_eq!(
        ss_revati.offset_degrees,
        Some(Angle::from_degrees(-0.79167046))
    );

    let ss_citra =
        descriptor(&Ayanamsa::SuryasiddhantaCitra).expect("Suryasiddhanta Citra descriptor");
    assert_eq!(
        ss_citra.epoch,
        Some(JulianDay::from_days(1_903_396.8128654))
    );
    assert_eq!(
        ss_citra.offset_degrees,
        Some(Angle::from_degrees(2.11070444))
    );

    let djwhal = descriptor(&Ayanamsa::DjwhalKhul).expect("Djwhal Khul descriptor");
    assert_eq!(djwhal.epoch, Some(JulianDay::from_days(2_415_020.0)));
    assert_eq!(
        djwhal.offset_degrees,
        Some(Angle::from_degrees(360.0 - 333.0369024))
    );

    let sheoran = descriptor(&Ayanamsa::Sheoran).expect("Sheoran descriptor");
    assert_eq!(sheoran.epoch, Some(JulianDay::from_days(1_789_947.090_881)));
    assert_eq!(sheoran.offset_degrees, Some(Angle::from_degrees(0.0)));

    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    assert!(sidereal_offset(&Ayanamsa::BabylonianHuber, instant)
        .expect("Huber offset should exist")
        .degrees()
        .is_finite());
    assert!(sidereal_offset(&Ayanamsa::GalacticEquatorIau1958, instant)
        .expect("Galactic Equator offset should exist")
        .degrees()
        .is_finite());
    // TruePushya is now a TrueStar mode (Task 2). The cubic fit window is 1900-2100
    // (JD ≥ 2_415_020.0); the reference epoch JD 1_855_769 is outside the window,
    // so sidereal_offset correctly returns None. Descriptor metadata is still present
    // (verified above at lines 299-304).
    assert_eq!(
        sidereal_offset(
            &Ayanamsa::TruePushya,
            Instant::new(JulianDay::from_days(1_855_769.248_315), TimeScale::Tt),
        ),
        None
    );
    assert_eq!(
        sidereal_offset(
            &Ayanamsa::DjwhalKhul,
            Instant::new(JulianDay::from_days(2_415_020.0), TimeScale::Tt),
        ),
        Some(Angle::from_degrees(360.0 - 333.0369024))
    );
    assert_eq!(
        sidereal_offset(
            &Ayanamsa::Sheoran,
            Instant::new(JulianDay::from_days(1_789_947.090_881), TimeScale::Tt),
        ),
        Some(Angle::from_degrees(0.0))
    );
    assert!(sidereal_offset(&Ayanamsa::ValensMoon, instant)
        .expect("Valens Moon offset should exist")
        .degrees()
        .is_finite());
    // GalacticCenterCochrane is now a Galactic mode routed via the committed cubic
    // polynomial (fit window: JD 2_415_020–2_488_070). The reference epoch JD
    // 1_662_951 is outside the window, so sidereal_offset correctly returns None.
    // Descriptor metadata is still present (verified above at lines 286-292).
    assert_eq!(
        sidereal_offset(
            &Ayanamsa::GalacticCenterCochrane,
            Instant::new(JulianDay::from_days(1_662_951.794_251), TimeScale::Tt),
        ),
        None
    );
}

#[test]
fn krishnamurti_vp291_descriptor_uses_the_published_zero_point() {
    let d = descriptor(&Ayanamsa::KrishnamurtiVP291).expect("Krishnamurti VP291 descriptor");
    assert_eq!(d.epoch, Some(JulianDay::from_days(1_827_424.663_554)));
    assert_eq!(d.offset_degrees, Some(Angle::from_degrees(0.0)));
    assert_eq!(
        d.offset_at(Instant::new(
            JulianDay::from_days(1_827_424.663_554),
            TimeScale::Tt
        )),
        Some(Angle::from_degrees(0.0))
    );
}

#[test]
fn scheduled_historical_reference_modes_use_the_published_zero_points() {
    let true_sheoran = descriptor(&Ayanamsa::TrueSheoran).expect("True Sheoran descriptor");
    assert_eq!(
        true_sheoran.epoch,
        Some(JulianDay::from_days(1_789_947.090_881))
    );
    assert_eq!(true_sheoran.offset_degrees, Some(Angle::from_degrees(0.0)));
    assert_eq!(
        true_sheoran.offset_at(Instant::new(
            JulianDay::from_days(1_789_947.090_881),
            TimeScale::Tt
        )),
        Some(Angle::from_degrees(0.0))
    );

    let rgilbrand = descriptor(&Ayanamsa::GalacticCenterRgilbrand)
        .expect("Galactic Center (Rgilbrand) descriptor");
    assert_eq!(
        rgilbrand.epoch,
        Some(JulianDay::from_days(1_861_740.329_525))
    );
    assert_eq!(rgilbrand.offset_degrees, Some(Angle::from_degrees(0.0)));
    assert_eq!(
        rgilbrand.offset_at(Instant::new(
            JulianDay::from_days(1_861_740.329_525),
            TimeScale::Tt
        )),
        Some(Angle::from_degrees(0.0))
    );

    let mula_wilhelm = descriptor(&Ayanamsa::GalacticCenterMulaWilhelm)
        .expect("Galactic Center (Mula/Wilhelm) descriptor");
    assert_eq!(
        mula_wilhelm.epoch,
        Some(JulianDay::from_days(1_946_834.818_321))
    );
    assert_eq!(mula_wilhelm.offset_degrees, Some(Angle::from_degrees(0.0)));
    assert_eq!(
        mula_wilhelm.offset_at(Instant::new(
            JulianDay::from_days(1_946_834.818_321),
            TimeScale::Tt
        )),
        Some(Angle::from_degrees(0.0))
    );
}

#[test]
fn deferred_modes_stay_descriptor_only() {
    use pleiades_types::{Ayanamsa, CompatibilityClaimTier};

    // No-SE_SIDM modes — deferred (no distinct SE code, must stay DescriptorOnly).
    let deferred = [
        Ayanamsa::Udayagiri,
        Ayanamsa::PvrPushyaPaksha,
        Ayanamsa::Sheoran,
        // Slice-2: remaining non-promoted candidates.
        Ayanamsa::BabylonianTrueGeoc,
        Ayanamsa::BabylonianTrueTopc,
        Ayanamsa::BabylonianTrueObs,
        Ayanamsa::BabylonianHouse,
        Ayanamsa::BabylonianHouseObs,
        Ayanamsa::BabylonianSissy,
        // Slice-2: no-SE_SIDM deferrals (no distinct SE code, must stay DescriptorOnly).
        Ayanamsa::DhruvaGalacticCenterMula,
        Ayanamsa::GalacticEquator,
    ];

    let built_ins = crate::built_in_ayanamsas();
    for m in deferred {
        let d = built_ins
            .iter()
            .find(|d| d.ayanamsa == m)
            .unwrap_or_else(|| panic!("{m:?} not found in built_in_ayanamsas()"));
        assert_eq!(
            d.claim_tier,
            CompatibilityClaimTier::DescriptorOnly,
            "{m:?} must stay descriptor-only"
        );
    }
}

#[test]
fn release_grade_numeric_ayanamsa_set_is_exactly_the_gated_modes() {
    use pleiades_types::{Ayanamsa, CompatibilityClaimTier};

    let release_grade: Vec<Ayanamsa> = crate::built_in_ayanamsas()
        .iter()
        .filter(|d| d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric)
        .map(|d| d.ayanamsa.clone())
        .collect();

    let expected = [
        Ayanamsa::Lahiri,
        Ayanamsa::Raman,
        Ayanamsa::Krishnamurti,
        Ayanamsa::FaganBradley,
        Ayanamsa::TrueChitra,
        Ayanamsa::TrueCitra,
        Ayanamsa::J2000,
        Ayanamsa::J1900,
        Ayanamsa::B1950,
        Ayanamsa::UshaShashi,
        Ayanamsa::DjwhalKhul,
        Ayanamsa::Yukteshwar,
        Ayanamsa::JnBhasin,
        Ayanamsa::Sassanian,
        Ayanamsa::LahiriIcrc,
        Ayanamsa::Lahiri1940,
        Ayanamsa::Aryabhata522,
        Ayanamsa::Suryasiddhanta499,
        Ayanamsa::Suryasiddhanta499MeanSun,
        Ayanamsa::Aryabhata499,
        Ayanamsa::Aryabhata499MeanSun,
        Ayanamsa::SuryasiddhantaRevati,
        Ayanamsa::SuryasiddhantaCitra,
        // Fitted family promoted in Phase 6 slice 2 (Task 5)
        Ayanamsa::TrueRevati,
        Ayanamsa::TruePushya,
        Ayanamsa::TrueMula,
        Ayanamsa::TrueSheoran,
        Ayanamsa::GalacticCenter,
        Ayanamsa::GalacticCenterRgilbrand,
        Ayanamsa::GalacticEquatorIau1958,
        Ayanamsa::GalacticEquatorTrue,
        Ayanamsa::GalacticEquatorMula,
        Ayanamsa::GalacticCenterMardyks,
        Ayanamsa::GalacticCenterMulaWilhelm,
        Ayanamsa::GalacticCenterCochrane,
        Ayanamsa::GalacticEquatorFiorenza,
        // Fitted-offset family promoted in Phase 6 slice 3 (Task 4)
        Ayanamsa::DeLuce,
        Ayanamsa::BabylonianKugler1,
        Ayanamsa::BabylonianKugler2,
        Ayanamsa::BabylonianKugler3,
        Ayanamsa::BabylonianHuber,
        Ayanamsa::BabylonianEtaPiscium,
        Ayanamsa::BabylonianAldebaran,
        Ayanamsa::Hipparchus,
        Ayanamsa::BabylonianBritton,
        Ayanamsa::ValensMoon,
        Ayanamsa::LahiriVP285,
        Ayanamsa::KrishnamurtiVP291,
    ];

    assert_eq!(release_grade.len(), expected.len());
    for mode in expected {
        assert!(release_grade.contains(&mode), "missing {mode:?}");
    }
}

#[test]
fn promoted_fitted_and_fitted_offset_modes_are_release_grade() {
    use crate::descriptor;
    use pleiades_types::CompatibilityClaimTier::ReleaseGradeNumeric;
    use pleiades_types::{Ayanamsa, CompatibilityClaimTier};
    for m in [
        Ayanamsa::TrueRevati,
        Ayanamsa::TruePushya,
        Ayanamsa::TrueMula,
        Ayanamsa::TrueSheoran,
        Ayanamsa::GalacticCenter,
        Ayanamsa::GalacticCenterRgilbrand,
        Ayanamsa::GalacticEquatorIau1958,
        Ayanamsa::GalacticEquatorTrue,
        Ayanamsa::GalacticEquatorMula,
        Ayanamsa::GalacticCenterMardyks,
        Ayanamsa::GalacticCenterMulaWilhelm,
        Ayanamsa::GalacticCenterCochrane,
        Ayanamsa::GalacticEquatorFiorenza,
        // Fitted-offset family promoted in Phase 6 slice 3
        Ayanamsa::DeLuce,
        Ayanamsa::BabylonianKugler1,
        Ayanamsa::BabylonianKugler2,
        Ayanamsa::BabylonianKugler3,
        Ayanamsa::BabylonianHuber,
        Ayanamsa::BabylonianEtaPiscium,
        Ayanamsa::BabylonianAldebaran,
        Ayanamsa::Hipparchus,
        Ayanamsa::BabylonianBritton,
        Ayanamsa::ValensMoon,
        Ayanamsa::LahiriVP285,
        Ayanamsa::KrishnamurtiVP291,
    ] {
        let d = descriptor(&m).expect("descriptor exists");
        assert_eq!(d.claim_tier, ReleaseGradeNumeric, "{m:?}");
    }
    // Deferred: stay descriptor-only.
    assert_eq!(
        descriptor(&Ayanamsa::DhruvaGalacticCenterMula)
            .unwrap()
            .claim_tier,
        CompatibilityClaimTier::DescriptorOnly
    );
}
