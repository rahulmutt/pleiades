//! README capability blocks (issue #166): rendering, the registry, marker
//! parsing, the audit rules and the rewrite.

use super::*;
use pleiades_backend::{
    AccuracyClass, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
    BackendProvenance, BodyClaim, BodyClaimTier, ClaimEvidence,
};
use pleiades_types::{CelestialBody, CoordinateFrame, Instant, JulianDay, TimeRange, TimeScale};

fn claim(body: CelestialBody, tier: BodyClaimTier, accuracy: AccuracyClass) -> BodyClaim {
    BodyClaim::new(body, tier, accuracy, ClaimEvidence::None)
}

/// Mean-only, geocentric-only, open range, two tiers: the shape of
/// `pleiades-elp`.
fn mean_geocentric_metadata() -> BackendMetadata {
    BackendMetadata {
        id: BackendId::new("test-mean"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("test"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
        body_claims: vec![
            claim(
                CelestialBody::Moon,
                BodyClaimTier::Constrained,
                AccuracyClass::Moderate,
            ),
            claim(
                CelestialBody::MeanNode,
                BodyClaimTier::Constrained,
                AccuracyClass::Moderate,
            ),
            claim(
                CelestialBody::TrueApogee,
                BodyClaimTier::Unsupported,
                AccuracyClass::Unknown,
            ),
        ],
        supported_frames: vec![CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
        capabilities: BackendCapabilities {
            geocentric: true,
            topocentric: false,
            apparent: false,
            mean: true,
            batch: true,
            native_sidereal: false,
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    }
}

fn lines(metadata: &BackendMetadata, bodies: &BodySource) -> Vec<String> {
    render_capability_lines(metadata, bodies)
}

#[test]
fn a_mean_geocentric_backend_renders_its_rejections_and_tiers() {
    assert_eq!(
        lines(&mean_geocentric_metadata(), &BodySource::Metadata),
        [
            "Coordinates: mean geometric; apparent requests are rejected",
            "Observer: geocentric; topocentric requests are rejected",
            "Frames: ecliptic, equatorial · Time scales: TT, TDB",
            "Window: no nominal limit",
            "Release-grade bodies: none",
            "Constrained bodies (Moderate): Moon, Mean Node",
            "Unsupported bodies (Unknown): True Apogee",
        ]
    );
}

#[test]
fn an_apparent_topocentric_bounded_backend_renders_both_modes_and_its_window() {
    let mut metadata = mean_geocentric_metadata();
    metadata.capabilities.apparent = true;
    metadata.capabilities.topocentric = true;
    metadata.capabilities.native_sidereal = true;
    metadata.nominal_range = TimeRange::new(
        Some(Instant::new(
            JulianDay::from_days(2_415_020.5),
            TimeScale::Tt,
        )),
        Some(Instant::new(
            JulianDay::from_days(2_488_069.5),
            TimeScale::Tt,
        )),
    );
    metadata.body_claims = vec![claim(
        CelestialBody::Sun,
        BodyClaimTier::ReleaseGrade,
        AccuracyClass::High,
    )];
    let window = format!("Window: {}", metadata.nominal_range.summary_line());
    assert_eq!(
        lines(&metadata, &BodySource::Metadata),
        [
            "Coordinates: mean geometric and apparent",
            "Observer: geocentric and topocentric",
            "Frames: ecliptic, equatorial · Time scales: TT, TDB",
            window.as_str(),
            "Release-grade bodies (High): Sun",
            "Sidereal: native",
        ]
    );
}

#[test]
fn a_tier_with_mixed_accuracy_names_no_class() {
    let mut metadata = mean_geocentric_metadata();
    metadata.body_claims = vec![
        claim(
            CelestialBody::Sun,
            BodyClaimTier::Constrained,
            AccuracyClass::High,
        ),
        claim(
            CelestialBody::Moon,
            BodyClaimTier::Constrained,
            AccuracyClass::Moderate,
        ),
    ];
    let rendered = lines(&metadata, &BodySource::Metadata);
    assert!(
        rendered.contains(&"Constrained bodies: Sun, Moon".to_string()),
        "{rendered:?}"
    );
}

#[test]
fn runtime_bodies_replace_every_body_line() {
    let rendered = lines(
        &mean_geocentric_metadata(),
        &BodySource::Runtime(SPK_BODIES),
    );
    assert_eq!(
        &rendered[4..],
        ["Bodies: those of the loaded SPK kernels (none without one)"]
    );
}

#[test]
fn limits_are_listed_only_when_a_backend_has_them() {
    let mut metadata = mean_geocentric_metadata();
    metadata.capabilities.batch = false;
    metadata.deterministic = false;
    metadata.offline = false;
    let rendered = lines(&metadata, &BodySource::Metadata);
    assert_eq!(
        rendered.last().map(String::as_str),
        Some("Limits: not batch-capable; not deterministic; needs network access")
    );
    assert!(!lines(&mean_geocentric_metadata(), &BodySource::Metadata)
        .iter()
        .any(|line| line.starts_with("Limits:")));
}

#[test]
fn the_block_body_is_one_bullet_per_line_after_a_newline() {
    let body = render_block_body(&["a".to_string(), "b c".to_string()]);
    assert_eq!(body, "\n- a\n- b c\n");
}

#[test]
fn registry_entries_name_their_backends_once() {
    let blocks = capability_blocks();
    let ids: Vec<&str> = blocks.iter().map(|block| block.backend_id).collect();
    assert_eq!(
        ids,
        [
            "pleiades-data",
            "pleiades-vsop87",
            "pleiades-elp",
            "jpl-snapshot",
            "jpl-spk"
        ]
    );
    for block in &blocks {
        assert_eq!(
            (block.metadata)().id.as_str(),
            block.backend_id,
            "{}",
            block.readme
        );
    }
}

#[test]
fn every_registry_entry_renders_a_bulleted_body() {
    let rendered = rendered_blocks();
    assert_eq!(rendered.len(), capability_blocks().len());
    for block in &rendered {
        assert!(
            block.body.starts_with("\n- ") && block.body.ends_with('\n'),
            "{} in {}: {:?}",
            block.backend_id,
            block.readme,
            block.body
        );
    }
    let spk = rendered
        .iter()
        .find(|block| block.backend_id == "jpl-spk")
        .expect("jpl-spk is registered");
    assert!(
        spk.body.contains(&format!("- Bodies: {SPK_BODIES}\n")),
        "{:?}",
        spk.body
    );
}

#[test]
fn the_spk_block_never_loads_kernels() {
    // The registry builds SpkBackend without kernels, so a kernel variable in
    // the environment cannot reach the block.
    let spk = capability_blocks()
        .into_iter()
        .find(|block| block.backend_id == "jpl-spk")
        .expect("jpl-spk is registered");
    assert!(matches!(spk.bodies, BodySource::Runtime(SPK_BODIES)));
    assert!((spk.metadata)().body_claims.is_empty());
}
