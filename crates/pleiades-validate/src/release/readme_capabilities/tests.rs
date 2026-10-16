//! README capability blocks (issue #166): rendering, the registry, marker
//! parsing, the audit rules and the rewrite.

use super::registry::{capability_blocks, SPK_BODIES};
use super::render::{render_block_body, render_capability_lines, BodySource};
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

use crate::release::workspace_audit::WorkspaceAuditViolation;
use std::path::Path;

const README: &str = "crates/pleiades-x/README.md";

fn expected(id: &'static str, readme: &'static str, body: &str) -> RenderedBlock {
    RenderedBlock {
        readme,
        backend_id: id,
        body: body.to_string(),
    }
}

fn audit(text: &str, blocks: &[RenderedBlock]) -> Vec<WorkspaceAuditViolation> {
    audit_capability_blocks(Path::new(README), README, text, blocks)
}

fn rules(violations: &[WorkspaceAuditViolation]) -> Vec<&'static str> {
    violations.iter().map(|violation| violation.rule).collect()
}

fn readme_with(id: &str, body: &str) -> String {
    format!("# x\n\n## Capabilities\n\n<!-- audit:capabilities {id} -->{body}<!-- /audit:capabilities -->\n\n## License\n")
}

#[test]
fn a_current_block_passes() {
    let blocks = [expected("x", README, "\n- a\n- b\n")];
    assert!(audit(&readme_with("x", "\n- a\n- b\n"), &blocks).is_empty());
}

#[test]
fn a_registered_block_absent_from_its_readme_is_missing() {
    let blocks = [expected("x", README, "\n- a\n")];
    let violations = audit("# x\n", &blocks);
    assert_eq!(rules(&violations), ["readme.capabilities-block-missing"]);
    assert!(
        violations[0].detail.contains("`x`"),
        "{}",
        violations[0].detail
    );
}

#[test]
fn a_changed_line_is_stale_and_named() {
    let blocks = [expected("x", README, "\n- a\n- b\n")];
    let violations = audit(&readme_with("x", "\n- a\n- c\n"), &blocks);
    assert_eq!(rules(&violations), ["readme.capabilities-block-stale"]);
    let detail = &violations[0].detail;
    assert!(detail.contains("found `- c`"), "{detail}");
    assert!(detail.contains("expected `- b`"), "{detail}");
    assert!(detail.contains("mise run readme-capabilities"), "{detail}");
}

#[test]
fn a_shorter_block_is_stale_at_its_end() {
    let blocks = [expected("x", README, "\n- a\n- b\n")];
    let violations = audit(&readme_with("x", "\n- a\n"), &blocks);
    assert_eq!(rules(&violations), ["readme.capabilities-block-stale"]);
    assert!(
        violations[0].detail.contains("(end of block)"),
        "{}",
        violations[0].detail
    );
}

#[test]
fn a_crlf_block_is_stale_and_rewrites_clean() {
    let blocks = [expected("x", README, "\n- a\n")];
    let text = readme_with("x", "\r\n- a\r\n");
    assert_eq!(
        rules(&audit(&text, &blocks)),
        ["readme.capabilities-block-stale"]
    );
}

#[test]
fn an_unregistered_id_is_unknown() {
    let violations = audit(&readme_with("nope", "\n- a\n"), &[]);
    assert_eq!(rules(&violations), ["readme.capabilities-block-unknown"]);
}

#[test]
fn a_block_in_the_wrong_readme_is_unknown_and_missing_from_its_own() {
    // Registered for another README: unknown here. Its own README is
    // audited separately and reports it missing there.
    let blocks = [expected("x", "crates/pleiades-y/README.md", "\n- a\n")];
    let violations = audit(&readme_with("x", "\n- a\n"), &blocks);
    assert_eq!(rules(&violations), ["readme.capabilities-block-unknown"]);
    assert!(
        violations[0].detail.contains("crates/pleiades-y/README.md"),
        "{}",
        violations[0].detail
    );
}

#[test]
fn malformed_markers_are_reported_once() {
    let blocks = [expected("x", README, "\n- a\n")];
    for text in [
        "<!-- audit:capabilities x -->\n- a\n".to_string(),
        "<!-- audit:capabilities x\n- a\n<!-- /audit:capabilities -->".to_string(),
        "<!-- audit:capabilities x -->\n<!-- audit:capabilities y -->\n<!-- /audit:capabilities -->".to_string(),
        "<!-- /audit:capabilities -->\n".to_string(),
        format!("{}{}", readme_with("x", "\n- a\n"), readme_with("x", "\n- a\n")),
    ] {
        assert_eq!(rules(&audit(&text, &blocks)), ["readme.capabilities-block-malformed"], "{text:?}");
    }
}

#[test]
fn an_id_with_padding_is_trimmed() {
    let blocks = [expected("x", README, "\n- a\n")];
    let text = "<!-- audit:capabilities   x   -->\n- a\n<!-- /audit:capabilities -->";
    assert!(audit(text, &blocks).is_empty());
    // A different prefix is not a marker at all, so the block is missing.
    // (No closing marker: a lone one would be reported as malformed instead.)
    let text = "<!--  audit:capabilities x -->\n- a\n";
    assert_eq!(
        rules(&audit(text, &blocks)),
        ["readme.capabilities-block-missing"]
    );
}

#[test]
fn a_readme_with_no_registered_block_and_no_markers_is_clean() {
    assert!(audit("# x\n\nNo blocks here.\n", &[]).is_empty());
}

#[test]
fn rewriting_fixes_a_stale_block_and_nothing_else() {
    let blocks = [expected("x", README, "\n- a\n- b\n")];
    let before = "# x  \n\ntrailing spaces kept  \n<!-- audit:capabilities x -->\n- old\n<!-- /audit:capabilities -->\nno final newline";
    let after = rewrite_capability_blocks(README, before, &blocks).expect("rewrite");
    assert_eq!(
        after,
        "# x  \n\ntrailing spaces kept  \n<!-- audit:capabilities x -->\n- a\n- b\n<!-- /audit:capabilities -->\nno final newline"
    );
    assert!(audit(&after, &blocks).is_empty());
    assert_eq!(
        rewrite_capability_blocks(README, &after, &blocks).expect("again"),
        after
    );
}

#[test]
fn rewriting_a_crlf_block_gives_a_clean_one() {
    let blocks = [expected("x", README, "\n- a\n")];
    let after = rewrite_capability_blocks(README, &readme_with("x", "\r\n- a\r\n"), &blocks)
        .expect("rewrite");
    assert_eq!(after, readme_with("x", "\n- a\n"));
}

#[test]
fn rewriting_leaves_unknown_blocks_and_refuses_malformed_markers() {
    let blocks = [expected("x", README, "\n- a\n")];
    let unknown = readme_with("nope", "\n- keep\n");
    assert_eq!(
        rewrite_capability_blocks(README, &unknown, &blocks).expect("rewrite"),
        unknown
    );
    assert!(rewrite_capability_blocks(README, "<!-- audit:capabilities x -->\n", &blocks).is_err());
}

fn temp_root(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "pleiades-readme-capabilities-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("crates/pleiades-x")).expect("temp dir");
    root
}

#[test]
fn the_command_rewrites_in_place_and_check_writes_nothing() {
    let root = temp_root("command");
    let path = root.join(README);
    let blocks = [expected("x", README, "\n- a\n")];
    std::fs::write(&path, readme_with("x", "\n- old\n")).expect("write");

    let error = readme_capabilities_at(&root, &blocks, true).expect_err("stale fails --check");
    assert!(error.contains("readme.capabilities-block-stale"), "{error}");
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        readme_with("x", "\n- old\n")
    );

    let report = readme_capabilities_at(&root, &blocks, false).expect("rewrite");
    assert!(
        report.contains("updated crates/pleiades-x/README.md"),
        "{report}"
    );
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        readme_with("x", "\n- a\n")
    );
    assert!(readme_capabilities_at(&root, &blocks, true).is_ok());
    let report = readme_capabilities_at(&root, &blocks, false).expect("idempotent");
    assert!(
        report.contains("unchanged crates/pleiades-x/README.md"),
        "{report}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_command_reports_a_missing_block_without_creating_it() {
    let root = temp_root("missing");
    let path = root.join(README);
    std::fs::write(&path, "# x\n").expect("write");
    let blocks = [expected("x", README, "\n- a\n")];
    let error = readme_capabilities_at(&root, &blocks, false).expect_err("missing fails");
    assert!(
        error.contains("readme.capabilities-block-missing"),
        "{error}"
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "# x\n");
    let _ = std::fs::remove_dir_all(&root);
}
