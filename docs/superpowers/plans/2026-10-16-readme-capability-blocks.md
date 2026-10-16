# README Capability Blocks Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Release-backend READMEs carry capability blocks rendered from each backend's `BackendMetadata`. `workspace-audit` fails when a block is missing, stale, unknown or malformed, and `pleiades-validate readme-capabilities` regenerates the blocks.

**Architecture:** A new module tree `crates/pleiades-validate/src/release/readme_capabilities/`, split into:
- `render.rs`: metadata → lines;
- `registry.rs`: which README holds which backend;
- `mod.rs`: marker parsing, the four audit rules, and the in-place rewrite;
- `tests.rs`.

`workspace_audit.rs` calls the audit once per README it already reads. `render/cli.rs` gains the command.

**Tech Stack:** Rust; the `pleiades-backend` metadata types; `pleiades-data`/`-vsop87`/`-elp`/`-jpl` backends (already dependencies of `pleiades-validate`).

**Spec:** `docs/superpowers/specs/2026-10-16-readme-capability-blocks-design.md`

## Global Constraints

- Markers: open `<!-- audit:capabilities <backend-id> -->`, close `<!-- /audit:capabilities -->`. The text between them is `"\n"` followed by one `- <line>\n` per rendered line.
- Rule ids, exactly: `readme.capabilities-block-missing`, `readme.capabilities-block-stale`, `readme.capabilities-block-unknown`, `readme.capabilities-block-malformed`.
- Registry: `crates/pleiades-data/README.md` → `pleiades-data`; `crates/pleiades-vsop87/README.md` → `pleiades-vsop87`; `crates/pleiades-elp/README.md` → `pleiades-elp`; `crates/pleiades-jpl/README.md` → `jpl-snapshot` and `jpl-spk` (runtime bodies: `those of the loaded SPK kernels (none without one)`).
- Registry metadata functions never read the environment or load kernels.
- Command: `pleiades-validate readme-capabilities` rewrites blocks in place; `--check` writes nothing and fails on any violation. A missing block is reported, never created. Text outside blocks is never changed. mise task: `readme-capabilities`.
- No README outside data, vsop87, elp, jpl and core changes.
- `cargo fmt --all` before every commit. `-D warnings` for clippy and rustdoc.
- Test `pleiades-validate` with libtest, `cargo test -p pleiades-validate --lib --bins --tests -- --include-ignored`, not nextest.

## Review Focus

1. **CRLF line endings in a README:** a block written with `\r\n` must be reported stale (and rewritten), not silently accepted or crash the parser. Pinned in Task 2 (`a_crlf_block_is_stale_and_rewrites_clean`).
2. **The same backend's block twice in one README** is malformed, not silently double-checked. Pinned in Task 2.
3. **Extra whitespace in the opening marker** (`<!--  audit:capabilities  pleiades-data  -->`) is malformed or unknown, never treated as a match. The parser matches the exact prefix `<!-- audit:capabilities ` and trims the id. Pinned in Task 2 (`an_id_with_padding_is_trimmed`, and a double-space prefix found as nothing so the block is missing).
4. **Rewriting is byte-exact outside blocks**, including trailing whitespace and no final newline. Pinned in Task 3.
5. **A kernel variable set in the environment** must not change the `jpl-spk` block. Pinned in Task 1 (`the_spk_block_never_loads_kernels`).

---

### Task 1: Renderer and registry

**Files:**
- Create: `crates/pleiades-validate/src/release/readme_capabilities/mod.rs` (module doc + `mod render; mod registry;` + re-exports; parsing comes in Task 2)
- Create: `crates/pleiades-validate/src/release/readme_capabilities/render.rs`
- Create: `crates/pleiades-validate/src/release/readme_capabilities/registry.rs`
- Create: `crates/pleiades-validate/src/release/readme_capabilities/tests.rs`
- Modify: `crates/pleiades-validate/src/release/mod.rs` (add `pub(crate) mod readme_capabilities;` after `readme_audit`)

**Interfaces:**
- Produces:
  - `pub(crate) enum BodySource { Metadata, Runtime(&'static str) }`
  - `pub(crate) fn render_capability_lines(metadata: &BackendMetadata, bodies: &BodySource) -> Vec<String>`
  - `pub(crate) fn render_block_body(lines: &[String]) -> String`, giving `"\n- a\n- b\n"`
  - `pub(crate) struct CapabilityBlock { pub readme: &'static str, pub backend_id: &'static str, pub metadata: fn() -> BackendMetadata, pub bodies: BodySource }`
  - `pub(crate) fn capability_blocks() -> Vec<CapabilityBlock>`
  - `pub(crate) struct RenderedBlock { pub readme: &'static str, pub backend_id: &'static str, pub body: String }`
  - `pub(crate) fn rendered_blocks() -> Vec<RenderedBlock>`
  - `pub(crate) const SPK_BODIES: &str = "those of the loaded SPK kernels (none without one)";`

- [ ] **Step 1: Write the failing tests** in `readme_capabilities/tests.rs`. The expected text is written by hand from the spec's table, never captured from the renderer:

```rust
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
            claim(CelestialBody::Moon, BodyClaimTier::Constrained, AccuracyClass::Moderate),
            claim(CelestialBody::MeanNode, BodyClaimTier::Constrained, AccuracyClass::Moderate),
            claim(CelestialBody::TrueApogee, BodyClaimTier::Unsupported, AccuracyClass::Unknown),
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
        Some(Instant::new(JulianDay::from_days(2_415_020.5), TimeScale::Tt)),
        Some(Instant::new(JulianDay::from_days(2_488_069.5), TimeScale::Tt)),
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
        claim(CelestialBody::Sun, BodyClaimTier::Constrained, AccuracyClass::High),
        claim(CelestialBody::Moon, BodyClaimTier::Constrained, AccuracyClass::Moderate),
    ];
    let rendered = lines(&metadata, &BodySource::Metadata);
    assert!(rendered.contains(&"Constrained bodies: Sun, Moon".to_string()), "{rendered:?}");
}

#[test]
fn runtime_bodies_replace_every_body_line() {
    let rendered = lines(&mean_geocentric_metadata(), &BodySource::Runtime(SPK_BODIES));
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
        ["pleiades-data", "pleiades-vsop87", "pleiades-elp", "jpl-snapshot", "jpl-spk"]
    );
    for block in &blocks {
        assert_eq!((block.metadata)().id.as_str(), block.backend_id, "{}", block.readme);
    }
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
```

Check `CelestialBody`'s `Display` for `MeanNode` and `TrueApogee` before running: the probe printed `Mean Node` and `True Apogee`. If the variant names differ, use the names `crates/pleiades-types` defines. The expected strings are their `Display`.

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p pleiades-validate --lib readme_capabilities`
Expected: compile errors (`render_capability_lines` not found).

- [ ] **Step 3: Implement `render.rs`**

```rust
//! Rendering of one backend's capability block from its metadata.

use pleiades_backend::{BackendMetadata, BodyClaim, BodyClaimTier};

/// Where a block's body lines come from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BodySource {
    /// The backend's `body_claims`, grouped by tier.
    Metadata,
    /// The bodies are fixed at runtime (for example by loaded kernels); the
    /// block says so in this sentence instead.
    Runtime(&'static str),
}

const TIERS: [(BodyClaimTier, &str); 4] = [
    (BodyClaimTier::ReleaseGrade, "Release-grade"),
    (BodyClaimTier::Constrained, "Constrained"),
    (BodyClaimTier::Approximate, "Approximate"),
    (BodyClaimTier::Unsupported, "Unsupported"),
];

/// One line per capability, in the spec's order.
pub(crate) fn render_capability_lines(metadata: &BackendMetadata, bodies: &BodySource) -> Vec<String> {
    let caps = &metadata.capabilities;
    let mut lines = vec![
        modes_line("Coordinates", ("mean geometric", caps.mean), ("apparent", caps.apparent), "mean", "apparent"),
        modes_line("Observer", ("geocentric", caps.geocentric), ("topocentric", caps.topocentric), "geocentric", "topocentric"),
        format!(
            "Frames: {} · Time scales: {}",
            join(metadata.supported_frames.iter().map(|frame| frame.to_string().to_lowercase())),
            join(metadata.supported_time_scales.iter().map(ToString::to_string)),
        ),
        format!("Window: {}", window(metadata)),
    ];
    match bodies {
        BodySource::Metadata => lines.extend(body_lines(&metadata.body_claims)),
        BodySource::Runtime(sentence) => lines.push(format!("Bodies: {sentence}")),
    }
    if caps.native_sidereal {
        lines.push("Sidereal: native".to_string());
    }
    let limits: Vec<&str> = [
        (!caps.batch, "not batch-capable"),
        (!metadata.deterministic, "not deterministic"),
        (!metadata.offline, "needs network access"),
    ]
    .into_iter()
    .filter_map(|(applies, text)| applies.then_some(text))
    .collect();
    if !limits.is_empty() {
        lines.push(format!("Limits: {}", limits.join("; ")));
    }
    lines
}

/// `"<label>: first and second"`, or `"<label>: first; <rejected> requests
/// are rejected"` when only one mode is supported.
fn modes_line(
    label: &str,
    first: (&str, bool),
    second: (&str, bool),
    first_request: &str,
    second_request: &str,
) -> String {
    match (first.1, second.1) {
        (true, true) => format!("{label}: {} and {}", first.0, second.0),
        (true, false) => format!("{label}: {}; {second_request} requests are rejected", first.0),
        (false, true) => format!("{label}: {}; {first_request} requests are rejected", second.0),
        (false, false) => format!("{label}: none"),
    }
}

fn window(metadata: &BackendMetadata) -> String {
    let range = &metadata.nominal_range;
    if range.start.is_none() && range.end.is_none() {
        "no nominal limit".to_string()
    } else {
        range.summary_line()
    }
}

fn body_lines(claims: &[BodyClaim]) -> Vec<String> {
    let mut lines = Vec::new();
    for (tier, label) in TIERS {
        let in_tier: Vec<&BodyClaim> = claims.iter().filter(|claim| claim.tier == tier).collect();
        let Some(first) = in_tier.first() else {
            if tier == BodyClaimTier::ReleaseGrade {
                lines.push("Release-grade bodies: none".to_string());
            }
            continue;
        };
        let names = join(in_tier.iter().map(|claim| claim.body.to_string()));
        if in_tier.iter().all(|claim| claim.accuracy == first.accuracy) {
            lines.push(format!("{label} bodies ({}): {names}", first.accuracy.label()));
        } else {
            lines.push(format!("{label} bodies: {names}"));
        }
    }
    lines
}

fn join(items: impl Iterator<Item = String>) -> String {
    items.collect::<Vec<_>>().join(", ")
}

/// The text between a block's markers: a newline, then one bullet per line.
pub(crate) fn render_block_body(lines: &[String]) -> String {
    let mut body = String::from("\n");
    for line in lines {
        body.push_str("- ");
        body.push_str(line);
        body.push('\n');
    }
    body
}
```

- [ ] **Step 4: Implement `registry.rs`**

```rust
//! Which README holds which backend's capability block.

use pleiades_backend::{BackendMetadata, EphemerisBackend};

use super::render::{render_block_body, render_capability_lines, BodySource};

/// `jpl-spk`'s bodies come from the kernels a caller loads, so its block
/// states that instead of listing them.
pub(crate) const SPK_BODIES: &str = "those of the loaded SPK kernels (none without one)";

/// One capability block: the README it lives in, the backend it describes,
/// and how to build that backend's metadata without the environment.
pub(crate) struct CapabilityBlock {
    /// Workspace-relative README path, with `/` separators.
    pub readme: &'static str,
    /// The backend id; also the block's key in its opening marker.
    pub backend_id: &'static str,
    /// Builds the metadata without loading kernels or reading the environment.
    pub metadata: fn() -> BackendMetadata,
    pub bodies: BodySource,
}

/// A block's expected text, rendered from the current metadata.
pub(crate) struct RenderedBlock {
    pub readme: &'static str,
    pub backend_id: &'static str,
    pub body: String,
}

pub(crate) fn capability_blocks() -> Vec<CapabilityBlock> {
    vec![
        CapabilityBlock {
            readme: "crates/pleiades-data/README.md",
            backend_id: "pleiades-data",
            metadata: || pleiades_data::PackagedDataBackend::default().metadata(),
            bodies: BodySource::Metadata,
        },
        CapabilityBlock {
            readme: "crates/pleiades-vsop87/README.md",
            backend_id: "pleiades-vsop87",
            metadata: || pleiades_vsop87::Vsop87Backend.metadata(),
            bodies: BodySource::Metadata,
        },
        CapabilityBlock {
            readme: "crates/pleiades-elp/README.md",
            backend_id: "pleiades-elp",
            metadata: || pleiades_elp::ElpBackend.metadata(),
            bodies: BodySource::Metadata,
        },
        CapabilityBlock {
            readme: "crates/pleiades-jpl/README.md",
            backend_id: "jpl-snapshot",
            metadata: || pleiades_jpl::JplSnapshotBackend::default().metadata(),
            bodies: BodySource::Metadata,
        },
        CapabilityBlock {
            readme: "crates/pleiades-jpl/README.md",
            backend_id: "jpl-spk",
            // No kernels: what an SPK backend serves depends on the caller's.
            metadata: || pleiades_jpl::SpkBackend::builder().build().metadata(),
            bodies: BodySource::Runtime(SPK_BODIES),
        },
    ]
}

pub(crate) fn rendered_blocks() -> Vec<RenderedBlock> {
    capability_blocks()
        .into_iter()
        .map(|block| RenderedBlock {
            readme: block.readme,
            backend_id: block.backend_id,
            body: render_block_body(&render_capability_lines(&(block.metadata)(), &block.bodies)),
        })
        .collect()
}
```

If `pleiades_jpl::JplSnapshotBackend::default()` or `SpkBackend::builder().build()` doesn't exist with exactly that shape, use what `crates/pleiades-validate/src/claims/` uses to build them, minus any environment reads. Keep the "never reads the environment" rule.

- [ ] **Step 5: Write `mod.rs`** (Task 2 adds the parsing and rules):

```rust
//! README capability blocks (issue #166): each release backend's README
//! carries a block, between `<!-- audit:capabilities <backend-id> -->` and
//! `<!-- /audit:capabilities -->`, rendered from that backend's
//! `BackendMetadata`. `workspace-audit` checks the blocks, and
//! `pleiades-validate readme-capabilities` regenerates them.

mod registry;
mod render;

pub(crate) use registry::{capability_blocks, rendered_blocks, CapabilityBlock, RenderedBlock, SPK_BODIES};
pub(crate) use render::{render_block_body, render_capability_lines, BodySource};

#[cfg(test)]
mod tests;
```

Add `pub(crate) mod readme_capabilities;` to `crates/pleiades-validate/src/release/mod.rs` after `pub(crate) mod readme_audit;`. If any re-exported item stays unused outside tests until Task 2, scope a `#[allow(unused_imports)]` to that `use` line with a one-line comment ("used by the audit from Task 2"). Remove the allow in Task 2.

- [ ] **Step 6: Run the tests**

Run: `cargo fmt --all && cargo test -p pleiades-validate --lib readme_capabilities`
Expected: 8 passed. Then `cargo clippy -p pleiades-validate --all-targets --all-features -- -D warnings`: clean.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-validate/src/release/mod.rs crates/pleiades-validate/src/release/readme_capabilities/
git commit -m "feat(validate): render README capability blocks from backend metadata (#166)"
```

---

### Task 2: Marker parsing and the four audit rules

**Files:**
- Modify: `crates/pleiades-validate/src/release/readme_capabilities/mod.rs`
- Test: `crates/pleiades-validate/src/release/readme_capabilities/tests.rs`

**Interfaces:**
- Consumes: `RenderedBlock` (Task 1); `crate::release::workspace_audit::WorkspaceAuditViolation { path: PathBuf, rule: &'static str, detail: String }`.
- Produces:
  - `pub(crate) const OPEN_PREFIX: &str = "<!-- audit:capabilities ";`, `pub(crate) const CLOSE: &str = "<!-- /audit:capabilities -->";`
  - `pub(crate) struct ParsedBlock<'a> { pub id: &'a str, pub body: std::ops::Range<usize> }`
  - `pub(crate) fn parse_blocks(text: &str) -> Result<Vec<ParsedBlock<'_>>, String>` (the `Err` is a malformed-marker detail)
  - `pub(crate) fn audit_capability_blocks(path: &Path, readme: &str, text: &str, expected: &[RenderedBlock]) -> Vec<WorkspaceAuditViolation>`

- [ ] **Step 1: Write the failing tests**, appended to `tests.rs`:

```rust
use crate::release::workspace_audit::WorkspaceAuditViolation;
use std::path::Path;

const README: &str = "crates/pleiades-x/README.md";

fn expected(id: &'static str, readme: &'static str, body: &str) -> RenderedBlock {
    RenderedBlock { readme, backend_id: id, body: body.to_string() }
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
    assert!(violations[0].detail.contains("`x`"), "{}", violations[0].detail);
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
    assert!(violations[0].detail.contains("(end of block)"), "{}", violations[0].detail);
}

#[test]
fn a_crlf_block_is_stale_and_rewrites_clean() {
    let blocks = [expected("x", README, "\n- a\n")];
    let text = readme_with("x", "\r\n- a\r\n");
    assert_eq!(rules(&audit(&text, &blocks)), ["readme.capabilities-block-stale"]);
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
    assert!(violations[0].detail.contains("crates/pleiades-y/README.md"), "{}", violations[0].detail);
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
    let text = "<!--  audit:capabilities x -->\n- a\n<!-- /audit:capabilities -->";
    assert_eq!(rules(&audit(text, &blocks)), ["readme.capabilities-block-missing"]);
}

#[test]
fn a_readme_with_no_registered_block_and_no_markers_is_clean() {
    assert!(audit("# x\n\nNo blocks here.\n", &[]).is_empty());
}
```

(`a_crlf_block_is_stale_and_rewrites_clean` gets its rewrite half in Task 3.)

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p pleiades-validate --lib readme_capabilities`
Expected: compile errors (`audit_capability_blocks` not found).

- [ ] **Step 3: Implement in `mod.rs`**, after the `use` lines:

```rust
use std::ops::Range;
use std::path::Path;

use super::workspace_audit::WorkspaceAuditViolation;

pub(crate) const OPEN_PREFIX: &str = "<!-- audit:capabilities ";
const OPEN_END: &str = " -->";
pub(crate) const CLOSE: &str = "<!-- /audit:capabilities -->";
const REGENERATE: &str = "run `mise run readme-capabilities` to regenerate it";

/// One block found in a README: its backend id and the byte range between
/// its markers.
pub(crate) struct ParsedBlock<'a> {
    pub id: &'a str,
    pub body: Range<usize>,
}

/// Every block in `text`, in order, or the first marker defect: an opening
/// marker without ` -->` or without a closing marker, a block inside a
/// block, a closing marker with no opening one, or one backend twice.
pub(crate) fn parse_blocks(text: &str) -> Result<Vec<ParsedBlock<'_>>, String> {
    let mut blocks: Vec<ParsedBlock<'_>> = Vec::new();
    let mut position = 0;
    loop {
        let next_open = text[position..].find(OPEN_PREFIX).map(|at| position + at);
        let gap_end = next_open.unwrap_or(text.len());
        if text[position..gap_end].contains(CLOSE) {
            return Err(format!("a `{CLOSE}` marker has no opening marker"));
        }
        let Some(open) = next_open else {
            return Ok(blocks);
        };
        let id_start = open + OPEN_PREFIX.len();
        let id_end = text[id_start..]
            .find(OPEN_END)
            .map(|at| id_start + at)
            .filter(|&end| !text[id_start..end].contains('\n'))
            .ok_or_else(|| format!("an opening `{OPEN_PREFIX}` marker is not closed by `-->` on its line"))?;
        let id = text[id_start..id_end].trim();
        let body_start = id_end + OPEN_END.len();
        let close = text[body_start..]
            .find(CLOSE)
            .map(|at| body_start + at)
            .ok_or_else(|| format!("the `{id}` block has no `{CLOSE}` marker"))?;
        if text[body_start..close].contains(OPEN_PREFIX) {
            return Err(format!("the `{id}` block contains another opening marker"));
        }
        if blocks.iter().any(|block| block.id == id) {
            return Err(format!("the `{id}` block appears twice"));
        }
        blocks.push(ParsedBlock { id, body: body_start..close });
        position = close + CLOSE.len();
    }
}

/// The capability-block violations of one README. `readme` is its
/// workspace-relative path with `/` separators; `expected` is every
/// registered block, rendered.
pub(crate) fn audit_capability_blocks(
    path: &Path,
    readme: &str,
    text: &str,
    expected: &[RenderedBlock],
) -> Vec<WorkspaceAuditViolation> {
    let violation = |rule: &'static str, detail: String| WorkspaceAuditViolation {
        path: path.to_path_buf(),
        rule,
        detail,
    };
    let parsed = match parse_blocks(text) {
        Ok(parsed) => parsed,
        Err(detail) => return vec![violation("readme.capabilities-block-malformed", detail)],
    };
    let mut violations = Vec::new();
    for block in &parsed {
        match expected.iter().find(|candidate| candidate.backend_id == block.id) {
            None => violations.push(violation(
                "readme.capabilities-block-unknown",
                format!("the `{}` capability block names no registered backend", block.id),
            )),
            Some(candidate) if candidate.readme != readme => violations.push(violation(
                "readme.capabilities-block-unknown",
                format!(
                    "the `{}` capability block belongs in {}, not here",
                    block.id, candidate.readme
                ),
            )),
            Some(candidate) => {
                if let Some(detail) = first_difference(&text[block.body.clone()], &candidate.body) {
                    violations.push(violation(
                        "readme.capabilities-block-stale",
                        format!(
                            "the `{}` capability block differs from its metadata: {detail}; {REGENERATE}",
                            block.id
                        ),
                    ));
                }
            }
        }
    }
    for candidate in expected.iter().filter(|candidate| candidate.readme == readme) {
        if !parsed.iter().any(|block| block.id == candidate.backend_id) {
            violations.push(violation(
                "readme.capabilities-block-missing",
                format!(
                    "the README has no `{}` capability block; add `{OPEN_PREFIX}{} -->` and `{CLOSE}` where it belongs, then {REGENERATE}",
                    candidate.backend_id, candidate.backend_id
                ),
            ));
        }
    }
    violations
}

/// `None` when `found == expected`; otherwise where they first differ, by
/// line, as "line N: found `..`, expected `..`".
fn first_difference(found: &str, expected: &str) -> Option<String> {
    if found == expected {
        return None;
    }
    // `split('\n')` keeps a stray `\r`, so a CRLF block differs visibly.
    let mut found_lines = found.split('\n');
    let mut expected_lines = expected.split('\n');
    for line in 1.. {
        match (found_lines.next(), expected_lines.next()) {
            (Some(f), Some(e)) if f == e => continue,
            (f, e) => {
                let show = |side: Option<&str>| side.map_or("(end of block)".to_string(), |text| text.escape_debug().to_string());
                return Some(format!("line {line}: found `{}`, expected `{}`", show(f), show(e)));
            }
        }
    }
    unreachable!("the strings differ, so some line differs")
}
```

The stale test asserts `found `- c``, which this format produces as `found `- c``. If clippy objects to `unreachable!` in a library path, restructure the loop to `loop { … }` with an index counter that returns. Do not use `#[allow]`.

- [ ] **Step 4: Run the tests**

Run: `cargo fmt --all && cargo test -p pleiades-validate --lib readme_capabilities`
Expected: all pass (8 from Task 1 + 10 new). Then clippy as in Task 1: clean, with Task 1's temporary `#[allow(unused_imports)]` removed if the items are now used. If they're still only used by tests until Task 4 wires the audit, keep the allow and say so in the report.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/src/release/readme_capabilities/
git commit -m "feat(validate): audit rules for README capability blocks (#166)"
```

---

### Task 3: Rewrite and the `readme-capabilities` command

**Files:**
- Modify: `crates/pleiades-validate/src/release/readme_capabilities/mod.rs`
- Modify: `crates/pleiades-validate/src/release/workspace_audit.rs` (make `fn workspace_root()` `pub(crate)`)
- Modify: `crates/pleiades-validate/src/render/cli.rs` (dispatch arm + one help line beside `workspace-audit`)
- Modify: `mise.toml` (new `[tasks.readme-capabilities]` next to `[tasks.audit]`)
- Test: `crates/pleiades-validate/src/release/readme_capabilities/tests.rs`

**Interfaces:**
- Consumes: Task 1–2 items.
- Produces:
  - `pub(crate) fn rewrite_capability_blocks(readme: &str, text: &str, expected: &[RenderedBlock]) -> Result<String, String>`
  - `pub(crate) fn readme_capabilities_at(root: &Path, expected: &[RenderedBlock], check: bool) -> Result<String, String>`
  - CLI `readme-capabilities [--check]`

- [ ] **Step 1: Write the failing tests**, appended to `tests.rs`:

```rust
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
    assert_eq!(rewrite_capability_blocks(README, &after, &blocks).expect("again"), after);
}

#[test]
fn rewriting_a_crlf_block_gives_a_clean_one() {
    let blocks = [expected("x", README, "\n- a\n")];
    let after = rewrite_capability_blocks(README, &readme_with("x", "\r\n- a\r\n"), &blocks).expect("rewrite");
    assert_eq!(after, readme_with("x", "\n- a\n"));
}

#[test]
fn rewriting_leaves_unknown_blocks_and_refuses_malformed_markers() {
    let blocks = [expected("x", README, "\n- a\n")];
    let unknown = readme_with("nope", "\n- keep\n");
    assert_eq!(rewrite_capability_blocks(README, &unknown, &blocks).expect("rewrite"), unknown);
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
    assert_eq!(std::fs::read_to_string(&path).unwrap(), readme_with("x", "\n- old\n"));

    let report = readme_capabilities_at(&root, &blocks, false).expect("rewrite");
    assert!(report.contains("updated crates/pleiades-x/README.md"), "{report}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), readme_with("x", "\n- a\n"));
    assert!(readme_capabilities_at(&root, &blocks, true).is_ok());
    let report = readme_capabilities_at(&root, &blocks, false).expect("idempotent");
    assert!(report.contains("unchanged crates/pleiades-x/README.md"), "{report}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_command_reports_a_missing_block_without_creating_it() {
    let root = temp_root("missing");
    let path = root.join(README);
    std::fs::write(&path, "# x\n").expect("write");
    let blocks = [expected("x", README, "\n- a\n")];
    let error = readme_capabilities_at(&root, &blocks, false).expect_err("missing fails");
    assert!(error.contains("readme.capabilities-block-missing"), "{error}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "# x\n");
    let _ = std::fs::remove_dir_all(&root);
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p pleiades-validate --lib readme_capabilities`
Expected: compile errors (`rewrite_capability_blocks` not found).

- [ ] **Step 3: Implement in `mod.rs`**

```rust
/// `text` with each block registered for `readme` replaced by its rendering.
/// Blocks of other ids are left for the audit to report. Malformed markers
/// are an error: nothing is rewritten.
pub(crate) fn rewrite_capability_blocks(
    readme: &str,
    text: &str,
    expected: &[RenderedBlock],
) -> Result<String, String> {
    let parsed = parse_blocks(text)?;
    let mut out = String::with_capacity(text.len());
    let mut copied = 0;
    for block in &parsed {
        let Some(candidate) = expected
            .iter()
            .find(|candidate| candidate.backend_id == block.id && candidate.readme == readme)
        else {
            continue;
        };
        out.push_str(&text[copied..block.body.start]);
        out.push_str(&candidate.body);
        copied = block.body.end;
    }
    out.push_str(&text[copied..]);
    Ok(out)
}

/// The `readme-capabilities` command over the workspace at `root`.
/// Rewrites every registered README's blocks, or with `check` writes nothing
/// and fails on any violation. Either way a missing block fails.
pub(crate) fn readme_capabilities_at(
    root: &Path,
    expected: &[RenderedBlock],
    check: bool,
) -> Result<String, String> {
    let mut readmes: Vec<&str> = expected.iter().map(|block| block.readme).collect();
    readmes.dedup();
    let mut report = Vec::new();
    let mut violations = Vec::new();
    for readme in readmes {
        let path = root.join(readme);
        let text = std::fs::read_to_string(&path).map_err(|error| format!("{readme}: {error}"))?;
        if check {
            violations.extend(audit_capability_blocks(&path, readme, &text, expected));
            continue;
        }
        let rewritten = rewrite_capability_blocks(readme, &text, expected)
            .map_err(|detail| format!("{readme}: {detail}"))?;
        if rewritten == text {
            report.push(format!("unchanged {readme}"));
        } else {
            std::fs::write(&path, &rewritten).map_err(|error| format!("{readme}: {error}"))?;
            report.push(format!("updated {readme}"));
        }
        violations.extend(audit_capability_blocks(&path, readme, &rewritten, expected));
    }
    if violations.is_empty() {
        report.push(format!("README capability blocks: OK ({} blocks)", expected.len()));
        Ok(report.join("\n"))
    } else {
        let lines: Vec<String> = violations
            .iter()
            .map(|violation| format!("{} [{}]: {}", violation.path.display(), violation.rule, violation.detail))
            .collect();
        Err(format!("README capability blocks failed:\n{}", lines.join("\n")))
    }
}
```

`readmes.dedup()` relies on the registry listing a README's blocks next to each other. The registry does (the two `jpl` entries are adjacent). Note this in a comment on `capability_blocks()`.

- [ ] **Step 4: Wire the CLI.** In `crates/pleiades-validate/src/release/workspace_audit.rs`, change `fn workspace_root()` to `pub(crate) fn workspace_root()`. In `crates/pleiades-validate/src/render/cli.rs`, beside the `Some("workspace-audit")` arm, add:

```rust
        Some("readme-capabilities") => {
            let check = match &args[1..] {
                [] => false,
                ["--check"] => true,
                other => {
                    return Err(format!(
                        "readme-capabilities accepts only --check, got: {}",
                        other.join(" ")
                    ))
                }
            };
            let root = std::fs::canonicalize(crate::release::workspace_audit::workspace_root())
                .map_err(|error| error.to_string())?;
            crate::release::readme_capabilities::readme_capabilities_at(
                &root,
                &crate::release::readme_capabilities::rendered_blocks(),
                check,
            )
        }
```

Adjust `args` element types to whatever this dispatcher uses (`&[&str]` or `&[String]`; follow the `claims-audit` arm). In the help string, immediately after the `workspace-audit` line, add this line, matching the spacing style:

```
  readme-capabilities [--check]  Regenerate (or --check) the README capability blocks from backend metadata
```

If a test checks that every dispatched command appears in the help (or the reverse), make it pass by this addition alone.

In `mise.toml`, after the `[tasks.audit]` block, add:

```toml
[tasks.readme-capabilities]
description = "Regenerate the README capability blocks from backend metadata (issue #166); pass --check to verify only."
run = "cargo run -q -p pleiades-validate -- readme-capabilities"
```

- [ ] **Step 5: Run the tests and one CLI smoke**

Run: `cargo fmt --all && cargo test -p pleiades-validate --lib readme_capabilities`
Expected: all pass (23).

Run: `cargo run -q -p pleiades-validate -- readme-capabilities --check`
Expected: exits non-zero listing five `readme.capabilities-block-missing` violations (the READMEs get their markers in Task 4). That's correct at this point. Paste the output in the report.

Run: `cargo test -p pleiades-validate --lib --bins --tests -- --include-ignored` and `cargo clippy -p pleiades-validate --all-targets --all-features -- -D warnings`
Expected: pass and clean.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-validate/src/release/readme_capabilities/ crates/pleiades-validate/src/release/workspace_audit.rs crates/pleiades-validate/src/render/cli.rs mise.toml
git commit -m "feat(validate): readme-capabilities regenerates the README capability blocks (#166)"
```

---

### Task 4: Fill the READMEs, wire the audit, document

**Files:**
- Modify: `crates/pleiades-data/README.md`, `crates/pleiades-vsop87/README.md`, `crates/pleiades-elp/README.md`, `crates/pleiades-jpl/README.md`, `crates/pleiades-core/README.md`
- Modify: `crates/pleiades-validate/src/release/workspace_audit.rs` (call the audit for every README it reads)
- Modify: `docs/development.md`, `docs/cli.md`

**Interfaces:**
- Consumes: `audit_capability_blocks`, `rendered_blocks`, `readme_capabilities_at` (Tasks 1–3).

- [ ] **Step 1: Add the markers and the prose cleanup.** Use the Edit tool for each README.

`crates/pleiades-data/README.md`:
- In the first paragraph, replace "(precomputed positions for the Sun, the Moon,\nMercury through Pluto and five asteroids, derived from JPL public-domain ephemerides)" with "(precomputed positions derived from JPL public-domain ephemerides)".
- In the second paragraph, delete the sentence "The backend serves the Sun, the Moon, Mercury\nthrough Pluto, and Ceres, Pallas, Juno, Vesta and `asteroid:433-Eros`, and\nfalls back to other providers when callers request bodies outside the packaged\nslice." and replace it with "A routing chain falls back to other providers for bodies outside the packaged slice."
- Before `## Status`, insert:

```markdown
## Capabilities

Generated from the backend's metadata (`mise run readme-capabilities`):

<!-- audit:capabilities pleiades-data -->
<!-- /audit:capabilities -->

```

- In `## Status`, replace "This backend returns mean\ngeometric coordinates and rejects apparent requests; the chart layer in" with "The chart layer in", so the sentence reads "The chart layer in `pleiades-core` and the event engines in `pleiades-events` apply apparent place on top of it."

`crates/pleiades-vsop87/README.md`: insert the same `## Capabilities` section (id `pleiades-vsop87`) before `## Status`. In Status, replace "This backend returns mean geometric heliocentric-derived geocentric positions and rejects apparent and topocentric requests; the chart layer in `pleiades-core` applies those corrections on top of a backend." with "Positions are geocentric, derived from the heliocentric VSOP87B series; the chart layer in `pleiades-core` applies apparent place and the topocentric correction on top."

`crates/pleiades-elp/README.md`: insert the section (id `pleiades-elp`) before `## Status`. In Status, replace "This backend returns mean geometric coordinates and rejects apparent and topocentric requests; the chart layer in `pleiades-core` applies those corrections on top of a backend." with "The chart layer in `pleiades-core` applies apparent place and the topocentric correction on top."

`crates/pleiades-jpl/README.md`: before `## License`, insert:

```markdown
## Capabilities

Generated from each backend's metadata (`mise run readme-capabilities`).

`JplSnapshotBackend`:

<!-- audit:capabilities jpl-snapshot -->
<!-- /audit:capabilities -->

`SpkBackend`:

<!-- audit:capabilities jpl-spk -->
<!-- /audit:capabilities -->

```

`crates/pleiades-core/README.md`: in Status, replace "The first-party backends themselves return mean geometric coordinates and reject apparent requests: the chart layer applies the corrections on top of them." with "Each backend crate's README lists what that backend returns; the chart layer applies apparent place and the topocentric correction on top."

- [ ] **Step 2: Fill the blocks**

Run: `cargo run -q -p pleiades-validate -- readme-capabilities`
Expected: "updated" for the four backend READMEs and `README capability blocks: OK (5 blocks)`. Read each filled block and check it against the metadata the spec's Context table lists: data has 21 release-grade (High) bodies and the 1900–2100 window; vsop87 has 9 constrained (Moderate); elp has 5 constrained (Moderate) and 2 unsupported (Unknown); jpl-snapshot has 16 constrained (Unknown); jpl-spk has the runtime sentence. Paste the five blocks into the report.

- [ ] **Step 3: Wire the audit** in `crates/pleiades-validate/src/release/workspace_audit.rs`.
- Import `use super::readme_capabilities::{audit_capability_blocks, rendered_blocks};`.
- In `workspace_audit_report_uncached`, compute `let capability_blocks = rendered_blocks();` once, before the README checks.
- Then, wherever a README's text is read:
  - the workspace README: `"README.md"` relative;
  - each crate README in the manifest loop: `crates/<dir>/README.md` relative, from the path minus `workspace_root` with `/` separators;
- extend `violations` with `audit_capability_blocks(&path, &relative, &text, &capability_blocks)`.
- Derive `relative` with `path.strip_prefix(&workspace_root)` and `components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/")`.

Then run:

Run: `mise run audit`
Expected: `Result: no workspace policy violations detected`.

Prove the wiring catches drift: change one character inside the vsop87 block, run `mise run audit` (expect a `readme.capabilities-block-stale` violation for `crates/pleiades-vsop87/README.md`), then restore it with `mise run readme-capabilities` and re-run `mise run audit` (clean). Paste both outputs into the report. Do not commit the edited state.

- [ ] **Step 4: Docs.**

In `docs/development.md`, after the paragraph that follows the `mise run audit` task list, add:

```markdown
`mise run audit` also checks the capability blocks in the backend READMEs (`<!-- audit:capabilities <backend-id> -->`): each must match what that backend's metadata renders. After a change to a backend's capabilities, body claims or window, run `mise run readme-capabilities` to regenerate them.
```

In `docs/cli.md`, in the `pleiades-validate` bash block, after the `workspace-audit` lines, add:

```bash
# README capability blocks: regenerate from backend metadata, or --check
cargo run -q -p pleiades-validate -- readme-capabilities --check
```

- [ ] **Step 5: Run the checks**

Run: `cargo fmt --all && mise run audit && cargo run -q -p pleiades-validate -- readme-capabilities --check && cargo test -p pleiades-validate --lib --bins --tests -- --include-ignored && cargo clippy -p pleiades-validate --all-targets --all-features -- -D warnings && mise run docs`
Expected: all pass. If a validate test pins the workspace-audit report text (for example a count of rules, or the clean report), update that pin only where it changed because of the new rules, and say which in the report.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-data/README.md crates/pleiades-vsop87/README.md crates/pleiades-elp/README.md crates/pleiades-jpl/README.md crates/pleiades-core/README.md crates/pleiades-validate/src/release/workspace_audit.rs docs/development.md docs/cli.md
git commit -m "docs: generate the backend READMEs' capability blocks and audit them (#166)"
```

---

### Task 5: Whole-branch verification

**Files:** none changed unless a check fails.

- [ ] **Step 1:** `cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && mise run docs`
- [ ] **Step 2:** `mise run test-full` (no edits or commits while it runs)
- [ ] **Step 3:** `mise run audit && mise run release-smoke && mise run package-check && cargo run -q -p pleiades-validate -- readme-capabilities --check`
- [ ] **Step 4:** With `PLEIADES_DE_KERNEL=/nonexistent/de440.bsp` exported for that one command, run `cargo run -q -p pleiades-validate -- readme-capabilities --check`. Expected: still OK (the registry never reads it).
- [ ] **Step 5:** If anything fails, fix it in the owning task's files, commit, and repeat from Step 1.
