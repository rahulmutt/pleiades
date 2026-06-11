# Crate Publishing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the 9 pleiades library crates publishable to crates.io as experimental 0.1.x releases, with publishability enforced by the existing fail-closed workspace audit plus a package-size gate.

**Architecture:** Manifest/license/README prep lands first so the workspace is publish-ready, then new audit rules in `pleiades-validate` (following its existing line-based TOML-scanning style) lock the invariants in, then a `package-check` mise task and cargo-release config make releases one command. Spec: `docs/superpowers/specs/2026-06-11-crate-publishing-design.md`.

**Tech Stack:** Rust 1.95 workspace, mise tasks, cargo-release, crates.io. No new library dependencies — the audit additions are pure-std, matching `pleiades-validate`'s zero-dependency policy.

---

## Context for the implementer

- **Publishable crates (9):** `pleiades-types`, `pleiades-backend`, `pleiades-compression`, `pleiades-houses`, `pleiades-ayanamsa`, `pleiades-vsop87`, `pleiades-elp`, `pleiades-jpl`, `pleiades-core`. **Stay unpublished (3):** `pleiades-cli`, `pleiades-data`, `pleiades-validate` (these keep `publish = false` and their plain path deps — do not touch their manifests).
- The workspace audit lives in `crates/pleiades-validate/src/lib.rs`. It deliberately scans manifest text line-by-line with helpers like `manifest_has_assignment` / `manifest_dependency_name` / `extract_inline_table_string` (around line 19745) instead of using a TOML parser — follow that style exactly. Audit functions return `Vec<WorkspaceAuditViolation>` (struct at ~line 3323: `path: PathBuf`, `rule: &'static str`, `detail: String`). The orchestrator is `workspace_audit_report_uncached()` (~line 20097). Tests live in the same file's `#[cfg(test)]` module; the existing audit tests are `workspace_audit_detects_native_hooks_in_manifests_and_lockfile` (~line 35158) and `workspace_audit_detects_tool_manifest_provenance_drift` (~line 35219) — add new tests right after the latter. A `unique_temp_dir(prefix)` test helper already exists.
- Line numbers above are from the state at plan time; re-locate with `grep -n` if they have drifted.
- Repo commit style: imperative sentence, no conventional-commit prefix (e.g. "Audit pinned toolchain provenance in workspace checks"). End commit messages with the `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>` trailer.
- Verification commands assume the repo root `/workspace` as working directory. `cargo test --workspace` takes several minutes; only run it where a step says to.

### Task overview

1. Dual-license files (root + per-crate copies)
2. Per-crate READMEs
3. Workspace manifest publish metadata + pinned internal deps
4. Make the 9 crate manifests publishable
5. Audit: workspace-manifest publish rules (TDD)
6. Audit: publishable-crate manifest rules (TDD)
7. Audit: per-crate license/README file checks (TDD)
8. Wire the new rules into the workspace audit
9. `package-check` mise task wired into `ci` and `release-gate`
10. cargo-release pin + `release.toml`
11. README "Published crates" section + `docs/release-process.md`
12. Final verification

---

### Task 1: Dual-license files

**Files:**
- Rename: `LICENSE` → `LICENSE-APACHE`
- Create: `LICENSE-MIT`
- Create: `crates/<each of the 9 publishable crates>/LICENSE-APACHE` and `LICENSE-MIT` (copies)

- [ ] **Step 1: Rename the Apache license**

```bash
git mv LICENSE LICENSE-APACHE
```

- [ ] **Step 2: Create `LICENSE-MIT`** with exactly this content:

```text
MIT License

Copyright (c) 2026 Rahul Muttineni

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

- [ ] **Step 3: Copy both files into each publishable crate** (cargo only packages files inside the crate directory):

```bash
for crate in pleiades-types pleiades-backend pleiades-compression pleiades-houses pleiades-ayanamsa pleiades-vsop87 pleiades-elp pleiades-jpl pleiades-core; do
  cp LICENSE-APACHE LICENSE-MIT "crates/$crate/"
done
```

- [ ] **Step 4: Verify** — `ls crates/*/LICENSE-MIT | wc -l` prints `9`; `git status --short` shows the rename plus 19 new files.

- [ ] **Step 5: Commit**

```bash
git add -A LICENSE-APACHE LICENSE-MIT crates/*/LICENSE-APACHE crates/*/LICENSE-MIT
git commit -m "Add dual-license files for publishable crates

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 2: Per-crate READMEs

**Files:**
- Create: `crates/<each of the 9>/README.md`

- [ ] **Step 1: Create the 9 README files** with exactly the content below. They share a status/license footer; the first paragraph and layering line differ per crate.

`crates/pleiades-types/README.md`:

```markdown
# pleiades-types

Shared typed vocabulary for the [pleiades](https://github.com/rahulmutt/pleiades) astrology workspace: angles, bodies, time scales, observers, coordinates, zodiac modes, house systems, and ayanamsas.

This crate sits at the base of the `pleiades-*` layering and depends on no other pleiades crates. Enable the `serde` feature for serialization support.

## Status

Experimental `0.1.x`. First-party backends expose mean geometric coordinates only, and broader accuracy claims are still gated; see the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

## License

MIT OR Apache-2.0
```

`crates/pleiades-backend/README.md`:

```markdown
# pleiades-backend

Backend traits, request/result contracts, capability metadata, policy summaries, and routing/composite helpers for [pleiades](https://github.com/rahulmutt/pleiades) ephemeris backends.

Depends only on `pleiades-types`. Enable the `serde` feature for serialization support.

## Status

Experimental `0.1.x`. First-party backends expose mean geometric coordinates only, and broader accuracy claims are still gated; see the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

## License

MIT OR Apache-2.0
```

`crates/pleiades-compression/README.md`:

```markdown
# pleiades-compression

Compressed ephemeris artifact data structures and codec helpers for the [pleiades](https://github.com/rahulmutt/pleiades) astrology workspace.

Depends only on `pleiades-types`. Enable the `serde` feature for serialization support.

## Status

Experimental `0.1.x`. The packaged-data artifact built on these structures is draft-grade; see the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

## License

MIT OR Apache-2.0
```

`crates/pleiades-houses/README.md`:

```markdown
# pleiades-houses

House-system catalog, aliases, formula-family metadata, and baseline house calculations for the [pleiades](https://github.com/rahulmutt/pleiades) astrology workspace.

Depends only on `pleiades-types`.

## Status

Experimental `0.1.x`. Formula, provenance, and interoperability audits still gate stronger compatibility claims; see the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

## License

MIT OR Apache-2.0
```

`crates/pleiades-ayanamsa/README.md`:

```markdown
# pleiades-ayanamsa

Ayanamsa catalog, aliases, reference offset metadata, and sidereal offset helpers for the [pleiades](https://github.com/rahulmutt/pleiades) astrology workspace.

Depends only on `pleiades-types`.

## Status

Experimental `0.1.x`. Formula, provenance, and interoperability audits still gate stronger compatibility claims; see the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

## License

MIT OR Apache-2.0
```

`crates/pleiades-vsop87/README.md`:

```markdown
# pleiades-vsop87

Pure-Rust VSOP87B planetary position backend for the [pleiades](https://github.com/rahulmutt/pleiades) astrology workspace, with generated binary coefficient tables and an approximate Pluto fallback.

Depends on `pleiades-types` and `pleiades-backend`. The crate ships its generated coefficient tables plus the raw VSOP87B source tables and the `regenerate-vsop87b-tables` tool used to rebuild them.

## Status

Experimental `0.1.x`. Output is mean geometric heliocentric-derived geocentric positions; apparent-place corrections and topocentric requests are rejected. Pluto is approximate and excluded from release-grade claims. See the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

## License

MIT OR Apache-2.0
```

`crates/pleiades-elp/README.md`:

```markdown
# pleiades-elp

Compact Meeus-style lunar baseline backend for the [pleiades](https://github.com/rahulmutt/pleiades) astrology workspace: Moon, mean/true node, and mean apogee/perigee channels.

Depends on `pleiades-types` and `pleiades-backend`. This is a compact baseline, not a full ELP coefficient implementation.

## Status

Experimental `0.1.x`. Output is mean geometric coordinates; apparent-place corrections and topocentric requests are rejected. See the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

## License

MIT OR Apache-2.0
```

`crates/pleiades-jpl/README.md`:

```markdown
# pleiades-jpl

Checked-in JPL Horizons reference snapshots and snapshot-backed validation helpers for the [pleiades](https://github.com/rahulmutt/pleiades) astrology workspace, plus pure-Rust CSV parsing entry points for JPL-style manifest/row corpora.

Depends on `pleiades-types` and `pleiades-backend`. This is a reference/validation fixture crate, not a broad public-data reader.

## Status

Experimental `0.1.x`. The checked-in corpus is regression evidence, sparse relative to production-coverage goals; see the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

## License

MIT OR Apache-2.0
```

`crates/pleiades-core/README.md`:

```markdown
# pleiades-core

High-level chart façade for the [pleiades](https://github.com/rahulmutt/pleiades) astrology workspace: typed tropical/sidereal chart requests, request validation, compatibility and API-stability profiles, and re-exports for common consumers.

Sits at the top of the published `pleiades-*` library layering (types, backend, houses, ayanamsa, compression). Pair it with a backend crate such as `pleiades-vsop87` and `pleiades-elp` to compute positions.

## Status

Experimental `0.1.x`. First-party backends expose mean geometric coordinates; UTC/UT1 need caller-supplied conversion offsets, and apparent/topocentric requests are rejected. See the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

## License

MIT OR Apache-2.0
```

- [ ] **Step 2: Verify** — `ls crates/*/README.md | wc -l` prints `9` (only the 9 publishable crates get READMEs).

- [ ] **Step 3: Commit**

```bash
git add crates/*/README.md
git commit -m "Add per-crate readmes for publishable crates

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 3: Workspace manifest publish metadata

**Files:**
- Modify: `Cargo.toml` (workspace root)

- [ ] **Step 1: Replace the root `Cargo.toml`** with exactly:

```toml
[workspace]
members = [
    "crates/pleiades-ayanamsa",
    "crates/pleiades-backend",
    "crates/pleiades-cli",
    "crates/pleiades-compression",
    "crates/pleiades-core",
    "crates/pleiades-data",
    "crates/pleiades-elp",
    "crates/pleiades-houses",
    "crates/pleiades-jpl",
    "crates/pleiades-types",
    "crates/pleiades-validate",
    "crates/pleiades-vsop87",
]
resolver = "2"

[workspace.package]
edition = "2021"
license = "MIT OR Apache-2.0"
version = "0.1.0"
rust-version = "1.95.0"
repository = "https://github.com/rahulmutt/pleiades"
homepage = "https://github.com/rahulmutt/pleiades"
keywords = ["astrology", "astronomy", "ephemeris"]
categories = ["science"]

[workspace.dependencies]
serde = { version = "1", default-features = false, features = ["derive", "std"] }
pleiades-ayanamsa = { path = "crates/pleiades-ayanamsa", version = "0.1.0" }
pleiades-backend = { path = "crates/pleiades-backend", version = "0.1.0" }
pleiades-compression = { path = "crates/pleiades-compression", version = "0.1.0" }
pleiades-core = { path = "crates/pleiades-core", version = "0.1.0" }
pleiades-elp = { path = "crates/pleiades-elp", version = "0.1.0" }
pleiades-houses = { path = "crates/pleiades-houses", version = "0.1.0" }
pleiades-jpl = { path = "crates/pleiades-jpl", version = "0.1.0" }
pleiades-types = { path = "crates/pleiades-types", version = "0.1.0" }
pleiades-vsop87 = { path = "crates/pleiades-vsop87", version = "0.1.0" }
```

(The `members` list is the same 12 entries as before; `keywords`/`categories` must respect crates.io limits — max 5 each, category slugs must exist on crates.io; `science` is a valid slug.)

- [ ] **Step 2: Verify the workspace still builds**

Run: `cargo check --workspace --quiet`
Expected: exits 0, no output.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "Add workspace publish metadata and pinned internal dependencies

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

(If `Cargo.lock` is unchanged, `git add` of it is a no-op — fine.)

---

### Task 4: Make the 9 crate manifests publishable

**Files:**
- Modify: `crates/pleiades-types/Cargo.toml`, `crates/pleiades-backend/Cargo.toml`, `crates/pleiades-compression/Cargo.toml`, `crates/pleiades-houses/Cargo.toml`, `crates/pleiades-ayanamsa/Cargo.toml`, `crates/pleiades-vsop87/Cargo.toml`, `crates/pleiades-elp/Cargo.toml`, `crates/pleiades-jpl/Cargo.toml`, `crates/pleiades-core/Cargo.toml`

Do **not** modify `pleiades-cli`, `pleiades-data`, or `pleiades-validate`.

- [ ] **Step 1: Replace each manifest** with exactly the content below.

`crates/pleiades-types/Cargo.toml`:

```toml
[package]
name = "pleiades-types"
description = "Shared typed vocabulary for the pleiades astrology workspace: angles, bodies, time scales, observers, coordinates, zodiac modes, house systems, and ayanamsas."
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
repository.workspace = true
homepage.workspace = true
keywords.workspace = true
categories.workspace = true
readme = "README.md"

[features]
serde = ["dep:serde"]

[dependencies]
serde = { workspace = true, optional = true }

[package.metadata.docs.rs]
all-features = true
```

`crates/pleiades-backend/Cargo.toml`:

```toml
[package]
name = "pleiades-backend"
description = "Backend traits, request/result contracts, capability metadata, policy summaries, and routing helpers for pleiades ephemeris backends."
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
repository.workspace = true
homepage.workspace = true
keywords.workspace = true
categories.workspace = true
readme = "README.md"

[features]
serde = ["dep:serde", "pleiades-types/serde"]

[dependencies]
pleiades-types = { workspace = true }
serde = { workspace = true, optional = true }

[dev-dependencies]
serde_json = "1"

[package.metadata.docs.rs]
all-features = true
```

`crates/pleiades-compression/Cargo.toml`:

```toml
[package]
name = "pleiades-compression"
description = "Compressed ephemeris artifact data structures and codec helpers for the pleiades astrology workspace."
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
repository.workspace = true
homepage.workspace = true
keywords.workspace = true
categories.workspace = true
readme = "README.md"

[features]
serde = ["dep:serde", "pleiades-types/serde"]

[dependencies]
pleiades-types = { workspace = true }
serde = { workspace = true, optional = true }

[dev-dependencies]
serde_json = "1"

[package.metadata.docs.rs]
all-features = true
```

`crates/pleiades-houses/Cargo.toml`:

```toml
[package]
name = "pleiades-houses"
description = "House-system catalog, aliases, formula-family metadata, and baseline house calculations for the pleiades astrology workspace."
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
repository.workspace = true
homepage.workspace = true
keywords.workspace = true
categories.workspace = true
readme = "README.md"

[dependencies]
pleiades-types = { workspace = true }
```

`crates/pleiades-ayanamsa/Cargo.toml`:

```toml
[package]
name = "pleiades-ayanamsa"
description = "Ayanamsa catalog, aliases, reference offset metadata, and sidereal offset helpers for the pleiades astrology workspace."
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
repository.workspace = true
homepage.workspace = true
keywords.workspace = true
categories.workspace = true
readme = "README.md"

[dependencies]
pleiades-types = { workspace = true }
```

`crates/pleiades-vsop87/Cargo.toml`:

```toml
[package]
name = "pleiades-vsop87"
description = "Pure-Rust VSOP87B planetary position backend with generated coefficient tables and an approximate Pluto fallback, for the pleiades astrology workspace."
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
repository.workspace = true
homepage.workspace = true
keywords.workspace = true
categories.workspace = true
readme = "README.md"

[dependencies]
pleiades-backend = { workspace = true }
pleiades-types = { workspace = true }
```

`crates/pleiades-elp/Cargo.toml`:

```toml
[package]
name = "pleiades-elp"
description = "Compact Meeus-style lunar baseline backend (Moon, nodes, apogee/perigee) for the pleiades astrology workspace."
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
repository.workspace = true
homepage.workspace = true
keywords.workspace = true
categories.workspace = true
readme = "README.md"

[dependencies]
pleiades-backend = { workspace = true }
pleiades-types = { workspace = true }
```

`crates/pleiades-jpl/Cargo.toml`:

```toml
[package]
name = "pleiades-jpl"
description = "Checked-in JPL Horizons reference snapshots and snapshot-backed validation helpers for the pleiades astrology workspace."
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
repository.workspace = true
homepage.workspace = true
keywords.workspace = true
categories.workspace = true
readme = "README.md"

[dependencies]
pleiades-backend = { workspace = true }
pleiades-types = { workspace = true }
```

`crates/pleiades-core/Cargo.toml`:

```toml
[package]
name = "pleiades-core"
description = "High-level chart facade, request validation, and compatibility profiles for the pleiades astrology workspace."
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
repository.workspace = true
homepage.workspace = true
keywords.workspace = true
categories.workspace = true
readme = "README.md"

[dependencies]
pleiades-backend = { workspace = true }
pleiades-types = { workspace = true }
pleiades-houses = { workspace = true }
pleiades-ayanamsa = { workspace = true }
pleiades-compression = { workspace = true }
```

- [ ] **Step 2: Verify the workspace builds with all features**

Run: `cargo check --workspace --all-features --quiet`
Expected: exits 0.

- [ ] **Step 3: Run the full test suite once** (manifests changed for every crate)

Run: `cargo test --workspace --quiet`
Expected: all tests pass (takes several minutes).

- [ ] **Step 4: Commit**

```bash
git add crates/*/Cargo.toml Cargo.lock
git commit -m "Make library crates publishable

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 5: Audit — workspace-manifest publish rules (TDD)

**Files:**
- Modify: `crates/pleiades-validate/src/lib.rs` (implementation after `audit_build_script_path`, ~line 20082; tests after `workspace_audit_detects_tool_manifest_provenance_drift`, ~line 35235)

- [ ] **Step 1: Write the failing tests.** Add to the `#[cfg(test)]` module:

```rust
    #[test]
    fn workspace_audit_detects_workspace_publish_metadata_drift() {
        let manifest = r#"[workspace.package]
version = "0.1.0"
license = "MIT"

[workspace.dependencies]
pleiades-types = { path = "crates/pleiades-types", version = "0.2.0" }
pleiades-backend = { version = "0.1.0" }
serde = { version = "1" }
"#;
        let violations =
            audit_workspace_manifest_publish_text(Path::new("/tmp/Cargo.toml"), manifest);

        assert!(violations
            .iter()
            .any(|violation| violation.rule == "publish.workspace-license"));
        assert!(violations
            .iter()
            .any(|violation| violation.rule == "publish.workspace-metadata-missing"
                && violation.detail.contains("repository")));
        assert!(violations
            .iter()
            .any(|violation| violation.rule == "publish.workspace-metadata-missing"
                && violation.detail.contains("keywords")));
        assert!(violations
            .iter()
            .any(|violation| violation.rule == "publish.workspace-dependency-version"
                && violation.detail.contains("pleiades-types")));
        assert!(violations
            .iter()
            .any(|violation| violation.rule == "publish.workspace-dependency-path"
                && violation.detail.contains("pleiades-backend")));
        assert!(!violations
            .iter()
            .any(|violation| violation.detail.contains("`serde`")));
    }

    #[test]
    fn workspace_audit_accepts_publish_ready_workspace_manifest() {
        let manifest = r#"[workspace.package]
version = "0.1.0"
license = "MIT OR Apache-2.0"
repository = "https://github.com/rahulmutt/pleiades"
homepage = "https://github.com/rahulmutt/pleiades"
keywords = ["astrology", "astronomy", "ephemeris"]
categories = ["science"]

[workspace.dependencies]
serde = { version = "1" }
pleiades-types = { path = "crates/pleiades-types", version = "0.1.0" }
"#;
        let violations =
            audit_workspace_manifest_publish_text(Path::new("/tmp/Cargo.toml"), manifest);
        assert!(violations.is_empty(), "unexpected violations: {violations:?}");
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p pleiades-validate --lib workspace_audit_detects_workspace_publish_metadata_drift 2>&1 | tail -5`
Expected: compile error — `audit_workspace_manifest_publish_text` not found.

- [ ] **Step 3: Implement.** Add immediately after `audit_build_script_path` (before `workspace_audit_report`):

```rust
const PUBLISH_WORKSPACE_INHERITED_FIELDS: [&str; 4] =
    ["repository", "homepage", "keywords", "categories"];

const PUBLISH_WORKSPACE_LICENSE: &str = "MIT OR Apache-2.0";

fn manifest_assignment_value(line: &str) -> Option<&str> {
    let (_, value) = line.split_once('=')?;
    Some(value.trim())
}

fn audit_workspace_manifest_publish_text(
    path: &Path,
    text: &str,
) -> Vec<WorkspaceAuditViolation> {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Section {
        Other,
        WorkspacePackage,
        WorkspaceDependencies,
    }

    let mut section = Section::Other;
    let mut violations = Vec::new();
    let mut workspace_version: Option<String> = None;
    let mut saw_license = false;
    let mut inherited_fields: Vec<&str> = Vec::new();
    let mut internal_dependencies: Vec<(String, String)> = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            section = match line {
                "[workspace.package]" => Section::WorkspacePackage,
                "[workspace.dependencies]" => Section::WorkspaceDependencies,
                _ => Section::Other,
            };
            continue;
        }

        match section {
            Section::WorkspacePackage => {
                if manifest_has_assignment(line, "version") {
                    workspace_version = manifest_assignment_value(line)
                        .map(|value| value.trim_matches('"').to_string());
                }
                if manifest_has_assignment(line, "license") {
                    saw_license = manifest_assignment_value(line)
                        .is_some_and(|value| value.trim_matches('"') == PUBLISH_WORKSPACE_LICENSE);
                }
                for field in PUBLISH_WORKSPACE_INHERITED_FIELDS {
                    if manifest_has_assignment(line, field) {
                        inherited_fields.push(field);
                    }
                }
            }
            Section::WorkspaceDependencies => {
                if let Some(name) = manifest_dependency_name(line) {
                    if name.starts_with("pleiades-") {
                        internal_dependencies.push((name.to_string(), line.to_string()));
                    }
                }
            }
            Section::Other => {}
        }
    }

    if !saw_license {
        violations.push(WorkspaceAuditViolation {
            path: path.to_path_buf(),
            rule: "publish.workspace-license",
            detail: format!(
                "workspace package license must be `{PUBLISH_WORKSPACE_LICENSE}` so published crates inherit the dual license"
            ),
        });
    }

    for field in PUBLISH_WORKSPACE_INHERITED_FIELDS {
        if !inherited_fields.contains(&field) {
            violations.push(WorkspaceAuditViolation {
                path: path.to_path_buf(),
                rule: "publish.workspace-metadata-missing",
                detail: format!(
                    "workspace package is missing `{field}`, which publishable crates inherit"
                ),
            });
        }
    }

    for (name, line) in &internal_dependencies {
        let expected_path = format!("path = \"crates/{name}\"");
        if !line.contains(expected_path.as_str()) {
            violations.push(WorkspaceAuditViolation {
                path: path.to_path_buf(),
                rule: "publish.workspace-dependency-path",
                detail: format!(
                    "workspace dependency `{name}` must declare `{expected_path}` so workspace builds use the local crate"
                ),
            });
        }
        match extract_inline_table_string(line, "version") {
            Some(version) => match workspace_version.as_deref() {
                Some(expected) if expected == version => {}
                Some(expected) => violations.push(WorkspaceAuditViolation {
                    path: path.to_path_buf(),
                    rule: "publish.workspace-dependency-version",
                    detail: format!(
                        "workspace dependency `{name}` pins version {version}, but the workspace package version is {expected}"
                    ),
                }),
                None => violations.push(WorkspaceAuditViolation {
                    path: path.to_path_buf(),
                    rule: "publish.workspace-version-missing",
                    detail: "workspace Cargo.toml does not declare a workspace package version to compare against pinned internal dependency versions"
                        .to_string(),
                }),
            },
            None => violations.push(WorkspaceAuditViolation {
                path: path.to_path_buf(),
                rule: "publish.workspace-dependency-version",
                detail: format!(
                    "workspace dependency `{name}` must pin a version equal to the workspace package version so published manifests carry a registry version"
                ),
            }),
        }
    }

    violations
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p pleiades-validate --lib workspace_audit 2>&1 | tail -5`
Expected: all `workspace_audit*` tests pass, including the two new ones.

- [ ] **Step 5: Format and commit**

```bash
cargo fmt --all
git add crates/pleiades-validate/src/lib.rs
git commit -m "Audit workspace manifest publish metadata

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 6: Audit — publishable-crate manifest rules (TDD)

**Files:**
- Modify: `crates/pleiades-validate/src/lib.rs` (implementation right after Task 5's code; tests after Task 5's tests)

- [ ] **Step 1: Write the failing tests:**

```rust
    #[test]
    fn workspace_audit_identifies_publishable_packages() {
        assert!(manifest_is_package("[package]\nname = \"a\"\n"));
        assert!(!manifest_is_package("[workspace]\nmembers = []\n"));
        assert!(manifest_declares_publish_false(
            "[package]\nname = \"a\"\npublish = false\n"
        ));
        assert!(!manifest_declares_publish_false("[package]\nname = \"a\"\n"));
        assert_eq!(
            manifest_package_name("[package]\nname = \"pleiades-types\"\n"),
            Some("pleiades-types".to_string())
        );
    }

    #[test]
    fn workspace_audit_detects_publishable_crate_manifest_gaps() {
        let manifest = r#"[package]
name = "pleiades-example"
version.workspace = true
edition.workspace = true

[dependencies]
pleiades-types = { path = "../pleiades-types" }
pleiades-data = { workspace = true }
serde = { workspace = true, optional = true }
"#;
        let publishable = vec![
            "pleiades-example".to_string(),
            "pleiades-types".to_string(),
        ];
        let violations = audit_publishable_manifest_text(
            Path::new("/tmp/Cargo.toml"),
            manifest,
            &publishable,
        );

        assert!(violations
            .iter()
            .any(|violation| violation.rule == "publish.description-missing"));
        assert!(violations
            .iter()
            .any(|violation| violation.rule == "publish.license-not-inherited"));
        assert!(violations
            .iter()
            .any(|violation| violation.rule == "publish.readme-field-missing"));
        assert!(violations
            .iter()
            .any(|violation| violation.rule == "publish.metadata-field-missing"
                && violation.detail.contains("repository")));
        assert!(violations
            .iter()
            .any(|violation| violation.rule == "publish.internal-dependency-not-workspace"
                && violation.detail.contains("pleiades-types")));
        assert!(violations
            .iter()
            .any(|violation| violation.rule == "publish.internal-dependency-unpublishable"
                && violation.detail.contains("pleiades-data")));
        assert!(!violations
            .iter()
            .any(|violation| violation.detail.contains("`serde`")));
    }

    #[test]
    fn workspace_audit_accepts_publish_ready_crate_manifest() {
        let manifest = r#"[package]
name = "pleiades-example"
description = "Example publishable crate."
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
repository.workspace = true
homepage.workspace = true
keywords.workspace = true
categories.workspace = true
readme = "README.md"

[dependencies]
pleiades-types = { workspace = true }

[dev-dependencies]
serde_json = "1"
"#;
        let publishable = vec![
            "pleiades-example".to_string(),
            "pleiades-types".to_string(),
        ];
        let violations = audit_publishable_manifest_text(
            Path::new("/tmp/Cargo.toml"),
            manifest,
            &publishable,
        );
        assert!(violations.is_empty(), "unexpected violations: {violations:?}");
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p pleiades-validate --lib workspace_audit_detects_publishable_crate_manifest_gaps 2>&1 | tail -5`
Expected: compile error — `audit_publishable_manifest_text` not found.

- [ ] **Step 3: Implement** (after Task 5's `audit_workspace_manifest_publish_text`):

```rust
fn manifest_is_package(text: &str) -> bool {
    text.lines().any(|line| line.trim() == "[package]")
}

fn manifest_declares_publish_false(text: &str) -> bool {
    let mut in_package = false;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_package = line == "[package]";
            continue;
        }
        if in_package
            && manifest_has_assignment(line, "publish")
            && manifest_assignment_value(line) == Some("false")
        {
            return true;
        }
    }
    false
}

fn manifest_package_name(text: &str) -> Option<String> {
    let mut in_package = false;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_package = line == "[package]";
            continue;
        }
        if in_package && manifest_has_assignment(line, "name") {
            return manifest_assignment_value(line)
                .map(|value| value.trim_matches('"').to_string());
        }
    }
    None
}

fn audit_publishable_manifest_text(
    path: &Path,
    text: &str,
    publishable_names: &[String],
) -> Vec<WorkspaceAuditViolation> {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Section {
        Other,
        Package,
        RuntimeDependencies,
        DevDependencies,
    }

    let mut section = Section::Other;
    let mut violations = Vec::new();
    let mut saw_description = false;
    let mut saw_license_inheritance = false;
    let mut saw_readme = false;
    let mut inherited_fields: Vec<&str> = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            section = match line {
                "[package]" => Section::Package,
                "[dependencies]" => Section::RuntimeDependencies,
                "[dev-dependencies]" => Section::DevDependencies,
                _ => Section::Other,
            };
            continue;
        }

        match section {
            Section::Package => {
                if manifest_has_assignment(line, "description") {
                    saw_description |= manifest_assignment_value(line)
                        .is_some_and(|value| !value.trim_matches('"').trim().is_empty());
                }
                if line == "license.workspace = true" {
                    saw_license_inheritance = true;
                }
                if line == "readme = \"README.md\"" {
                    saw_readme = true;
                }
                for field in PUBLISH_WORKSPACE_INHERITED_FIELDS {
                    let needle = format!("{field}.workspace = true");
                    if line == needle.as_str() {
                        inherited_fields.push(field);
                    }
                }
            }
            Section::RuntimeDependencies | Section::DevDependencies => {
                let Some(name) = manifest_dependency_name(line) else {
                    continue;
                };
                if !name.starts_with("pleiades-") {
                    continue;
                }
                if !line.contains("workspace = true") {
                    violations.push(WorkspaceAuditViolation {
                        path: path.to_path_buf(),
                        rule: "publish.internal-dependency-not-workspace",
                        detail: format!(
                            "internal dependency `{name}` must use `workspace = true` so it inherits the pinned path and version from the workspace manifest"
                        ),
                    });
                }
                if section == Section::RuntimeDependencies
                    && !publishable_names
                        .iter()
                        .any(|publishable| publishable.as_str() == name)
                {
                    violations.push(WorkspaceAuditViolation {
                        path: path.to_path_buf(),
                        rule: "publish.internal-dependency-unpublishable",
                        detail: format!(
                            "internal dependency `{name}` is not publishable, so this crate cannot list it as a runtime dependency"
                        ),
                    });
                }
            }
            Section::Other => {}
        }
    }

    if !saw_description {
        violations.push(WorkspaceAuditViolation {
            path: path.to_path_buf(),
            rule: "publish.description-missing",
            detail: "publishable crate is missing a non-blank package description".to_string(),
        });
    }
    if !saw_license_inheritance {
        violations.push(WorkspaceAuditViolation {
            path: path.to_path_buf(),
            rule: "publish.license-not-inherited",
            detail: "publishable crate must declare `license.workspace = true` so the dual license is inherited"
                .to_string(),
        });
    }
    if !saw_readme {
        violations.push(WorkspaceAuditViolation {
            path: path.to_path_buf(),
            rule: "publish.readme-field-missing",
            detail: "publishable crate must declare `readme = \"README.md\"`".to_string(),
        });
    }
    for field in PUBLISH_WORKSPACE_INHERITED_FIELDS {
        if !inherited_fields.contains(&field) {
            violations.push(WorkspaceAuditViolation {
                path: path.to_path_buf(),
                rule: "publish.metadata-field-missing",
                detail: format!("publishable crate must declare `{field}.workspace = true`"),
            });
        }
    }

    violations
}
```

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p pleiades-validate --lib workspace_audit 2>&1 | tail -5`
Expected: all pass.

- [ ] **Step 5: Format and commit**

```bash
cargo fmt --all
git add crates/pleiades-validate/src/lib.rs
git commit -m "Audit publishable crate manifests

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 7: Audit — per-crate license/README file checks (TDD)

**Files:**
- Modify: `crates/pleiades-validate/src/lib.rs`

- [ ] **Step 1: Write the failing test** (uses the existing `unique_temp_dir` helper):

```rust
    #[test]
    fn workspace_audit_detects_publish_file_gaps() {
        let root = unique_temp_dir("pleiades-publish-file-audit");
        let crate_dir = root.join("crates").join("pleiades-example");
        std::fs::create_dir_all(&crate_dir).expect("crate dir should be creatable");
        std::fs::write(root.join("LICENSE-APACHE"), "apache text")
            .expect("root apache license should be writable");
        std::fs::write(root.join("LICENSE-MIT"), "mit text")
            .expect("root mit license should be writable");
        std::fs::write(crate_dir.join("LICENSE-APACHE"), "apache text")
            .expect("crate apache license should be writable");
        std::fs::write(crate_dir.join("LICENSE-MIT"), "different text")
            .expect("crate mit license should be writable");

        let violations = audit_publishable_crate_files(&crate_dir.join("Cargo.toml"), &root);
        assert!(violations
            .iter()
            .any(|violation| violation.rule == "publish.readme-file-missing"));
        assert!(violations
            .iter()
            .any(|violation| violation.rule == "publish.license-file-drift"
                && violation.detail.contains("LICENSE-MIT")));
        assert!(!violations
            .iter()
            .any(|violation| violation.rule == "publish.license-file-missing"));

        std::fs::write(crate_dir.join("README.md"), "# pleiades-example\n")
            .expect("crate readme should be writable");
        std::fs::write(crate_dir.join("LICENSE-MIT"), "mit text")
            .expect("crate mit license should be writable");
        let violations = audit_publishable_crate_files(&crate_dir.join("Cargo.toml"), &root);
        assert!(violations.is_empty(), "unexpected violations: {violations:?}");
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p pleiades-validate --lib workspace_audit_detects_publish_file_gaps 2>&1 | tail -5`
Expected: compile error — `audit_publishable_crate_files` not found.

- [ ] **Step 3: Implement** (after `audit_publishable_manifest_text`):

```rust
fn audit_publishable_crate_files(
    manifest_path: &Path,
    workspace_root: &Path,
) -> Vec<WorkspaceAuditViolation> {
    const PUBLISH_LICENSE_FILES: [&str; 2] = ["LICENSE-APACHE", "LICENSE-MIT"];

    let mut violations = Vec::new();
    let Some(crate_dir) = manifest_path.parent() else {
        return violations;
    };

    let readme_path = crate_dir.join("README.md");
    if !readme_path.is_file() {
        violations.push(WorkspaceAuditViolation {
            path: readme_path,
            rule: "publish.readme-file-missing",
            detail: "publishable crate is missing its README.md".to_string(),
        });
    }

    for license_name in PUBLISH_LICENSE_FILES {
        let crate_copy_path = crate_dir.join(license_name);
        let root_copy_path = workspace_root.join(license_name);
        let Ok(root_bytes) = fs::read(&root_copy_path) else {
            violations.push(WorkspaceAuditViolation {
                path: root_copy_path,
                rule: "publish.license-file-missing",
                detail: format!("workspace root is missing {license_name}"),
            });
            continue;
        };
        match fs::read(&crate_copy_path) {
            Ok(crate_bytes) => {
                if crate_bytes != root_bytes {
                    violations.push(WorkspaceAuditViolation {
                        path: crate_copy_path,
                        rule: "publish.license-file-drift",
                        detail: format!(
                            "crate copy of {license_name} does not match the workspace root copy"
                        ),
                    });
                }
            }
            Err(_) => violations.push(WorkspaceAuditViolation {
                path: crate_copy_path,
                rule: "publish.license-file-missing",
                detail: format!("publishable crate is missing its {license_name} copy"),
            }),
        }
    }

    violations
}
```

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p pleiades-validate --lib workspace_audit 2>&1 | tail -5`
Expected: all pass.

- [ ] **Step 5: Format and commit**

```bash
cargo fmt --all
git add crates/pleiades-validate/src/lib.rs
git commit -m "Audit publishable crate license and readme files

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 8: Wire the new rules into the workspace audit

**Files:**
- Modify: `crates/pleiades-validate/src/lib.rs` — `workspace_audit_report_uncached()` (~line 20097) and the `WorkspaceAuditReport` doc comment (~line 3305)

- [ ] **Step 1: Update the doc comment** on `WorkspaceAuditReport` from:

```rust
/// A deterministic workspace audit that checks for mandatory native build hooks
/// in the first-party crates, lockfile, and pinned tooling manifest.
```

to:

```rust
/// A deterministic workspace audit that checks for mandatory native build hooks
/// in the first-party crates, lockfile, and pinned tooling manifest, plus
/// publish metadata for publishable crates.
```

- [ ] **Step 2: Rewrite the manifest loop in `workspace_audit_report_uncached`.** Replace:

```rust
    for path in &manifest_paths {
        let text = fs::read_to_string(path)?;
        violations.extend(audit_manifest_text(path, &text));
        if let Some(violation) = audit_build_script_path(path) {
            violations.push(violation);
        }
    }
```

with:

```rust
    let mut manifests = Vec::new();
    for path in &manifest_paths {
        let text = fs::read_to_string(path)?;
        manifests.push((path.clone(), text));
    }

    let publishable_names: Vec<String> = manifests
        .iter()
        .filter(|(_, text)| manifest_is_package(text) && !manifest_declares_publish_false(text))
        .filter_map(|(_, text)| manifest_package_name(text))
        .collect();

    let root_manifest_path = workspace_root.join("Cargo.toml");
    for (path, text) in &manifests {
        violations.extend(audit_manifest_text(path, text));
        if let Some(violation) = audit_build_script_path(path) {
            violations.push(violation);
        }
        if *path == root_manifest_path {
            violations.extend(audit_workspace_manifest_publish_text(path, text));
        } else if manifest_is_package(text) && !manifest_declares_publish_false(text) {
            violations.extend(audit_publishable_manifest_text(path, text, &publishable_names));
            violations.extend(audit_publishable_crate_files(path, &workspace_root));
        }
    }
```

- [ ] **Step 3: Run the real audit against the workspace** (Tasks 1–4 made it publish-ready, so it must be clean):

Run: `cargo run -q -p pleiades-validate -- workspace-audit`
Expected: clean audit output, exit 0. If violations print, the rule pinpoints which earlier task's output drifted — fix the workspace, not the rule (unless the rule itself mis-parses a manifest line, in which case fix the rule and its test).

- [ ] **Step 4: Run the full validate test suite**

Run: `cargo test -p pleiades-validate --quiet 2>&1 | tail -5`
Expected: all pass (some existing tests render full audit reports; they must still be clean).

- [ ] **Step 5: Lint, format, commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-validate --all-targets -- -D warnings
git add crates/pleiades-validate/src/lib.rs
git commit -m "Wire publish metadata checks into the workspace audit

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 9: `package-check` mise task

**Files:**
- Modify: `mise.toml`

- [ ] **Step 1: Add the task** after `[tasks.audit]`:

```toml
[tasks.package-check]
shell = "bash -c"
run = '''
set -euo pipefail
crates="pleiades-types pleiades-backend pleiades-compression pleiades-houses pleiades-ayanamsa pleiades-vsop87 pleiades-elp pleiades-jpl pleiades-core"
version=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
budget_bytes=$((9 * 1024 * 1024))
for crate in $crates; do
  cargo package --quiet --no-verify --allow-dirty --package "$crate"
  artifact="target/package/${crate}-${version}.crate"
  size=$(wc -c < "$artifact")
  if [ "$size" -gt "$budget_bytes" ]; then
    echo "package-check: $artifact is $size bytes, over the $budget_bytes-byte budget" >&2
    exit 1
  fi
  echo "package-check: $artifact ok ($size bytes)"
done
'''
```

Notes: `--no-verify` is required before the first publish (the verify build strips `path` from internal deps and would try to fetch them from crates.io). `--allow-dirty` keeps the check usable mid-development; tree cleanliness is enforced at release time by cargo-release, not here.

- [ ] **Step 2: Wire into `ci` and `release-gate`.** Change:

```toml
[tasks.release-gate]
depends = ["fmt", "lint", "test", "benchmark", "audit", "release-smoke"]
```

to:

```toml
[tasks.release-gate]
depends = ["fmt", "lint", "test", "benchmark", "audit", "package-check", "release-smoke"]
```

and:

```toml
[tasks.ci]
depends = ["fmt", "lint", "test", "docs", "audit", "release-smoke"]
```

to:

```toml
[tasks.ci]
depends = ["fmt", "lint", "test", "docs", "audit", "package-check", "release-smoke"]
```

- [ ] **Step 3: Run it**

Run: `mise run package-check`
Expected: nine `package-check: target/package/pleiades-<name>-0.1.0.crate ok (<n> bytes)` lines, exit 0. The `pleiades-vsop87` artifact is the largest; confirm it is comfortably under the budget. If `cargo package` errors on a manifest, that is a real publishability bug — fix it in the relevant crate manifest and re-run.

- [ ] **Step 4: Commit**

```bash
git add mise.toml
git commit -m "Add package-check gate for publishable crate artifacts

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 10: cargo-release pin and release configuration

**Files:**
- Modify: `mise.toml` (`[tools]`)
- Create: `release.toml`

- [ ] **Step 1: Pin cargo-release.** In `mise.toml`, change:

```toml
[tools]
rust = { version = "1.95.0", components = "rustfmt,clippy" }
```

to:

```toml
[tools]
rust = { version = "1.95.0", components = "rustfmt,clippy" }
"cargo:cargo-release" = "0.25"
```

Then run `mise install` (this compiles cargo-release from source via the cargo backend; expect a few minutes). If the `0.25` series is no longer current, pin the latest stable minor instead and use that in the expected output below.

- [ ] **Step 2: Verify the tool**

Run: `cargo release --version`
Expected: `cargo-release 0.25.<x>`.

- [ ] **Step 3: Create `release.toml`** at the repo root:

```toml
allow-branch = ["main"]
shared-version = true
consolidate-commits = true
pre-release-commit-message = "Release {{version}}"
tag-name = "v{{version}}"
sign-commit = false
sign-tag = false
publish = true
push = true
tag = true
```

(`shared-version` + `tag-name = "v{{version}}"` gives one workspace-wide tag instead of nine per-crate tags; cargo-release skips the three `publish = false` crates automatically.)

- [ ] **Step 4: Dry-run rehearsal** (cargo-release without `--execute` is always a dry run — do NOT pass `--execute`):

Run: `cargo release 2>&1 | tail -30`
Expected: a plan that publishes exactly the nine publishable crates in dependency order (types/houses/ayanamsa-tier first, `pleiades-core` last) at version 0.1.0, one `v0.1.0` tag, no errors about manifest metadata. Warnings about the remote branch or uncommitted `release.toml` are acceptable at this stage; metadata errors are not. If network/remote checks make the command fail outright in this environment, record the output in the commit message body and rely on Task 9's `package-check` plus the real run at release time.

- [ ] **Step 5: Commit**

```bash
git add mise.toml release.toml
git commit -m "Pin cargo-release and add workspace release configuration

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 11: Documentation

**Files:**
- Modify: `README.md`
- Create: `docs/release-process.md`

- [ ] **Step 1: Add a "Published crates" section to `README.md`** immediately before the line `For the source-of-truth design and compatibility targets, read [SPEC.md](SPEC.md) and the documents in [`spec/`](spec/).`:

```markdown
## Published crates

The nine library crates (`pleiades-types`, `pleiades-backend`, `pleiades-core`,
`pleiades-houses`, `pleiades-ayanamsa`, `pleiades-vsop87`, `pleiades-elp`,
`pleiades-jpl`, `pleiades-compression`) are published to crates.io as
experimental `0.1.x` releases under `MIT OR Apache-2.0`. The limits above apply
to the published crates as well; production-accuracy claims wait on the phases
in [PLAN.md](PLAN.md). `pleiades-cli`, `pleiades-data`, and `pleiades-validate`
are contributor tooling and stay unpublished. The release procedure is
documented in [docs/release-process.md](docs/release-process.md).
```

- [ ] **Step 2: Create `docs/release-process.md`:**

```markdown
# Release process

`pleiades` publishes nine library crates to crates.io with lockstep versions
managed by `cargo-release` (pinned in `mise.toml`). `pleiades-cli`,
`pleiades-data`, and `pleiades-validate` are `publish = false` and are never
published. Publish metadata is enforced by `mise run audit` (workspace audit
`publish.*` rules) and `mise run package-check` (artifact size budget), both
part of `ci` and `release-gate`.

## One-time setup

1. Create a crates.io API token at <https://crates.io/settings/tokens> with
   publish scopes.
2. Run `cargo login` and paste the token.

## Cutting a release

1. Start from a clean, pushed `main` checkout.
2. Run the release gate: `mise run release-gate`.
3. Rehearse: `cargo release <level>` (without `--execute` this is a dry run)
   and review the planned version bump, publish order, tag, and push.
4. Execute: `cargo release <level> --execute`, where `<level>` is `patch`,
   `minor`, or an explicit version such as `0.1.0`. cargo-release bumps the
   shared workspace version, updates the pinned `workspace.dependencies`
   versions, commits `Release {version}`, publishes the crates in dependency
   order (waiting for the index between publishes), tags `v{version}`, and
   pushes.

## After publishing

- Confirm the docs.rs build for each crate at `https://docs.rs/<crate>`.
- In a scratch project, run
  `cargo add pleiades-core pleiades-vsop87 pleiades-elp` and build a minimal
  chart against the published versions.

## Recovery and mistakes

- **Publish fails midway:** fix the cause and re-run the same
  `cargo release ... --execute`; crates already published at that version are
  skipped.
- **Bad release:** fix forward with a patch release; crates.io never allows
  re-publishing a version. Reserve `cargo yank` for unsound or badly broken
  releases, and yank only after the fixed version is available.
```

- [ ] **Step 3: Commit**

```bash
git add README.md docs/release-process.md
git commit -m "Document published-crate posture and release process

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 12: Final verification

- [ ] **Step 1: Run the full CI gate locally**

Run: `mise run ci`
Expected: fmt, lint, test, docs, audit, package-check, and release-smoke all pass. This is the same command CI runs, so a green run here means a green PR.

- [ ] **Step 2: Confirm nothing is left uncommitted**

Run: `git status --short`
Expected: empty.

- [ ] **Step 3: Report the remaining manual steps** to the user (do not attempt them):
  1. `cargo login` with a crates.io token,
  2. `cargo release 0.1.0 --execute` from a clean, pushed `main`,
  3. post-publish docs.rs and `cargo add` verification per `docs/release-process.md`.

---

## Out of scope

Publishing `pleiades-cli`/`pleiades-data`/`pleiades-validate`; any behavior or accuracy changes; CI-driven publishing; an umbrella `pleiades` crate (name is taken on crates.io). `PLAN.md` is intentionally untouched — publishing posture is not one of its remaining implementation phases.
