# Per-Crate Versioning Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every publishable crate carries its own version and is released by release-plz only when it, or an internal dependency it pins exactly, changes.

**Architecture:** The lockstep invariant lives in three places today (workspace version inheritance, `version_group` in `release-plz.toml`, and two workspace-audit rules). This plan first rewrites the audit rules to a per-crate invariant (TDD), then switches the manifests and release-plz config so the real tree satisfies the new rules, then updates the cargo-release fallback and the docs. The CI workflow is untouched.

**Tech Stack:** Rust workspace, release-plz 0.3.169 (`aqua:release-plz/release-plz` in `mise.toml`), git-cliff config (`cliff.toml`), cargo-release 1.1.6 (fallback), `pleiades-validate` workspace audit.

**Spec:** `docs/superpowers/specs/2026-09-28-per-crate-versioning-design.md`

## Global Constraints

- All 16 publishable crates start from `0.5.2`; no renumbering.
- Internal dependency pins in `[workspace.dependencies]` stay **exact** (`version = "0.5.2"`); release-plz updates them.
- `[workspace.dependencies]` is retained; crate manifests keep `pleiades-* = { workspace = true }` for runtime deps and path-only internal dev-deps.
- Per-crate `crates/<name>/CHANGELOG.md` files are **not** seeded by hand; release-plz creates them on first release.
- `.github/workflows/release-plz.yml` is not modified.
- No new dependencies. Pure Rust.
- Every commit must be `cargo fmt --all --check` clean (CI fmt gate).
- Commit messages follow Conventional Commits. Use `chore(release):` for config/manifest changes and `docs(release):` for docs so none of them land in a crate changelog.
- Work happens on branch `per-crate-versioning` (already created; the spec is its first commit).

## Review Focus

Inputs the spec implies but which are easy to get wrong; each has a pinning test in the owning task.

1. A `[workspace.dependencies]` pin for a crate whose manifest still inherits its version: the pin check must not panic or emit a spurious mismatch; the per-crate `publish.version-not-explicit` rule is what flags it. (Task 1, test `workspace_audit_skips_pin_comparison_for_unknown_crate_version`.)
2. A pin present but different from the crate's declared version: exactly one `publish.workspace-dependency-version` violation naming the crate and both versions. (Task 1, test `workspace_audit_flags_workspace_dependency_pin_drift`.)
3. A `[workspace.dependencies]` entry with `path` but no `version`: still a violation, since the published manifest would carry no registry version. (Task 1, test `workspace_audit_flags_missing_workspace_dependency_pin`.)
4. `version.workspace = true` in a crate manifest must not be mistaken for a literal version by the helper that reads `[package] version`. (Task 1, extension of `workspace_audit_identifies_publishable_packages`.)
5. After the manifest switch, `cargo metadata` must still resolve every workspace crate at `0.5.2` with an unchanged `Cargo.lock`. (Task 2, Step 7.)

---

### Task 1: Per-crate version rules in the workspace audit

**Files:**
- Modify: `crates/pleiades-validate/src/release/workspace_audit.rs` (functions `audit_workspace_manifest_publish_text` ~687-810, `manifest_package_name` ~834, `audit_publishable_manifest_text` ~850-990, `workspace_audit_report_uncached` ~1058-1128)
- Test: `crates/pleiades-validate/src/tests/release_workspace_audit.rs` (tests at ~248-345 and ~342-560)

**Interfaces:**
- Consumes: existing helpers `manifest_has_assignment(line, key)`, `manifest_assignment_value(line)`, `extract_inline_table_string(text, key)`, `manifest_is_package(text)`, `manifest_package_name(text)`, and the `WorkspaceAuditViolation { path, rule, detail }` struct.
- Produces:
  - `pub(crate) fn manifest_package_version(text: &str) -> Option<String>` — literal `[package] version`, `None` when inherited or absent.
  - `pub(crate) fn audit_workspace_manifest_publish_text(path: &Path, text: &str, crate_versions: &BTreeMap<String, String>) -> Vec<WorkspaceAuditViolation>` — new third parameter, crate name → declared version.
  - Rule `publish.version-not-explicit` (new), rule `publish.workspace-dependency-version` (per-crate semantics), rule `publish.workspace-version-missing` (removed).

Note on the interim state: until Task 2 switches the manifests, the two tests that audit the real tree (`workspace_audit_reports_a_clean_workspace`, `workspace_audit_summary_reports_a_clean_workspace`) will fail with 16 `publish.version-not-explicit` violations. That is expected. Run this task's tests by name; Task 2 turns the whole file green.

- [ ] **Step 1: Write the failing tests for the version helper and the crate-level rule**

In `crates/pleiades-validate/src/tests/release_workspace_audit.rs`, extend `workspace_audit_identifies_publishable_packages` by appending these assertions at the end of the function body:

```rust
    assert_eq!(
        manifest_package_version("[package]\nname = \"a\"\nversion = \"0.1.0\"\n"),
        Some("0.1.0".to_string())
    );
    assert_eq!(
        manifest_package_version("[package]\nname = \"a\"\nversion.workspace = true\n"),
        None
    );
    assert_eq!(
        manifest_package_version("[workspace.package]\nversion = \"0.1.0\"\n"),
        None
    );
```

In `workspace_audit_detects_publishable_crate_manifest_gaps` (its manifest already has `version.workspace = true`), add this assertion alongside the existing `publish.description-missing` one:

```rust
    assert!(violations
        .iter()
        .any(|violation| violation.rule == "publish.version-not-explicit"));
```

In `workspace_audit_accepts_publish_ready_crate_manifest`, change the line `version.workspace = true` in the manifest literal to:

```rust
version = "0.1.0"
```

- [ ] **Step 2: Run the three tests to verify they fail**

Run:
```bash
cargo nextest run -p pleiades-validate -E 'test(workspace_audit_identifies_publishable_packages) | test(workspace_audit_detects_publishable_crate_manifest_gaps) | test(workspace_audit_accepts_publish_ready_crate_manifest)'
```
Expected: compile error `cannot find function `manifest_package_version``. (The gaps test would fail on the missing rule once it compiles.)

- [ ] **Step 3: Add the helper and the crate-level rule**

In `crates/pleiades-validate/src/release/workspace_audit.rs`, directly after `manifest_package_name`, add:

```rust
/// The literal `version = "x.y.z"` declared in a crate manifest's `[package]`
/// table. `None` when the crate inherits its version (`version.workspace =
/// true`) or declares none. Only a literal version can be bumped for one
/// crate at a time, which per-crate releasing requires.
pub(crate) fn manifest_package_version(text: &str) -> Option<String> {
    let mut in_package = false;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_package = line == "[package]";
            continue;
        }
        if in_package && manifest_has_assignment(line, "version") {
            return manifest_assignment_value(line)
                .map(|value| value.trim_matches('"').to_string());
        }
    }
    None
}
```

(`manifest_has_assignment(line, "version")` is false for `version.workspace = true` because the character after the `version` prefix is `.`, not `=`.)

In `audit_publishable_manifest_text`, in the block of trailing checks (just before `if !saw_description {`), add:

```rust
    if manifest_package_version(text).is_none() {
        violations.push(WorkspaceAuditViolation {
            path: path.to_path_buf(),
            rule: "publish.version-not-explicit",
            detail: "publishable crate must declare a literal `version = \"x.y.z\"` in `[package]`; an inherited `version.workspace = true` returns the crate to lockstep and release-plz cannot bump it on its own"
                .to_string(),
        });
    }
```

- [ ] **Step 4: Run the three tests to verify they pass**

Run the same command as Step 2. Expected: 3 passed.

- [ ] **Step 5: Write the failing tests for the workspace-manifest rule**

In `crates/pleiades-validate/src/tests/release_workspace_audit.rs`, add `use std::collections::BTreeMap;` next to the existing `use std::path::Path;`.

Add this helper after the `use` lines:

```rust
fn crate_versions(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
    entries
        .iter()
        .map(|(name, version)| (name.to_string(), version.to_string()))
        .collect()
}
```

Replace the call in `workspace_audit_detects_workspace_publish_metadata_drift` with:

```rust
    let violations = audit_workspace_manifest_publish_text(
        Path::new("/tmp/Cargo.toml"),
        manifest,
        &crate_versions(&[("pleiades-types", "0.1.0")]),
    );
```

and add, after the existing assertions in that test:

```rust
    assert!(!violations
        .iter()
        .any(|violation| violation.rule == "publish.workspace-version-missing"));
```

In `workspace_audit_accepts_publish_ready_workspace_manifest`, delete the line `version = "0.1.0"` from the manifest literal and replace the call with:

```rust
    let violations = audit_workspace_manifest_publish_text(
        Path::new("/tmp/Cargo.toml"),
        manifest,
        &crate_versions(&[("pleiades-types", "0.1.0")]),
    );
```

Delete `workspace_audit_reports_missing_workspace_version_once` entirely and put these three tests in its place:

```rust
#[test]
fn workspace_audit_flags_workspace_dependency_pin_drift() {
    let manifest = r#"[workspace.package]
license = "MIT OR Apache-2.0"
repository = "https://github.com/rahulmutt/pleiades"
homepage = "https://github.com/rahulmutt/pleiades"
keywords = ["astrology", "astronomy", "ephemeris"]
categories = ["science"]

[workspace.dependencies]
pleiades-types = { path = "crates/pleiades-types", version = "0.1.0" }
pleiades-backend = { path = "crates/pleiades-backend", version = "0.1.0" }
"#;
    let violations = audit_workspace_manifest_publish_text(
        Path::new("/tmp/Cargo.toml"),
        manifest,
        &crate_versions(&[("pleiades-types", "0.1.1"), ("pleiades-backend", "0.1.0")]),
    );
    let drift: Vec<_> = violations
        .iter()
        .filter(|violation| violation.rule == "publish.workspace-dependency-version")
        .collect();
    assert_eq!(drift.len(), 1, "violations: {violations:?}");
    assert!(drift[0].detail.contains("pleiades-types"));
    assert!(drift[0].detail.contains("0.1.0"));
    assert!(drift[0].detail.contains("0.1.1"));
    assert_eq!(
        violations
            .iter()
            .filter(|violation| violation.rule != "publish.workspace-dependency-version")
            .count(),
        0,
        "violations: {violations:?}"
    );
}

#[test]
fn workspace_audit_flags_missing_workspace_dependency_pin() {
    let manifest = r#"[workspace.package]
license = "MIT OR Apache-2.0"
repository = "https://github.com/rahulmutt/pleiades"
homepage = "https://github.com/rahulmutt/pleiades"
keywords = ["astrology", "astronomy", "ephemeris"]
categories = ["science"]

[workspace.dependencies]
pleiades-types = { path = "crates/pleiades-types" }
"#;
    let violations = audit_workspace_manifest_publish_text(
        Path::new("/tmp/Cargo.toml"),
        manifest,
        &crate_versions(&[("pleiades-types", "0.1.0")]),
    );
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.workspace-dependency-version"
        && violation.detail.contains("pleiades-types")));
}

#[test]
fn workspace_audit_skips_pin_comparison_for_unknown_crate_version() {
    // A crate that still inherits its version has no entry in the map. The
    // pin cannot be compared, and the crate-level
    // `publish.version-not-explicit` rule is what reports that case.
    let manifest = r#"[workspace.package]
license = "MIT OR Apache-2.0"
repository = "https://github.com/rahulmutt/pleiades"
homepage = "https://github.com/rahulmutt/pleiades"
keywords = ["astrology", "astronomy", "ephemeris"]
categories = ["science"]

[workspace.dependencies]
pleiades-types = { path = "crates/pleiades-types", version = "0.1.0" }
"#;
    let violations = audit_workspace_manifest_publish_text(
        Path::new("/tmp/Cargo.toml"),
        manifest,
        &crate_versions(&[]),
    );
    assert!(
        violations.is_empty(),
        "unexpected violations: {violations:?}"
    );
}
```

- [ ] **Step 6: Run the workspace-manifest tests to verify they fail**

Run:
```bash
cargo nextest run -p pleiades-validate -E 'test(workspace_audit_detects_workspace_publish_metadata_drift) | test(workspace_audit_accepts_publish_ready_workspace_manifest) | test(workspace_audit_flags_workspace_dependency_pin_drift) | test(workspace_audit_flags_missing_workspace_dependency_pin) | test(workspace_audit_skips_pin_comparison_for_unknown_crate_version)'
```
Expected: compile error, `audit_workspace_manifest_publish_text` takes 2 arguments but 3 were supplied.

- [ ] **Step 7: Rewrite the workspace-manifest rule and the report builder**

In `crates/pleiades-validate/src/release/workspace_audit.rs`:

Add `use std::collections::BTreeMap;` to the imports at the top of the file (keep the existing imports; put it in alphabetical position among the `std` imports).

Change the signature of `audit_workspace_manifest_publish_text` to:

```rust
pub(crate) fn audit_workspace_manifest_publish_text(
    path: &Path,
    text: &str,
    crate_versions: &BTreeMap<String, String>,
) -> Vec<WorkspaceAuditViolation> {
```

Delete the local `let mut workspace_version: Option<String> = None;` and, in the `Section::WorkspacePackage` arm, delete the block:

```rust
                if manifest_has_assignment(line, "version") {
                    workspace_version = manifest_assignment_value(line)
                        .map(|value| value.trim_matches('"').to_string());
                }
```

Delete the whole `if workspace_version.is_none() && !internal_dependencies.is_empty() { ... }` block (the `publish.workspace-version-missing` rule).

Replace the `match extract_inline_table_string(line, "version") { ... }` inside `for (name, line) in &internal_dependencies` with:

```rust
        match extract_inline_table_string(line, "version") {
            Some(version) => {
                if let Some(expected) = crate_versions.get(name.as_str()) {
                    if expected != version {
                        violations.push(WorkspaceAuditViolation {
                            path: path.to_path_buf(),
                            rule: "publish.workspace-dependency-version",
                            detail: format!(
                                "workspace dependency `{name}` pins version {version}, but `crates/{name}/Cargo.toml` declares version {expected}"
                            ),
                        });
                    }
                }
            }
            None => violations.push(WorkspaceAuditViolation {
                path: path.to_path_buf(),
                rule: "publish.workspace-dependency-version",
                detail: format!(
                    "workspace dependency `{name}` must pin a version equal to the crate's declared package version so published manifests carry a registry version"
                ),
            }),
        }
```

In `workspace_audit_report_uncached`, after the `publishable_names` binding, add:

```rust
    let crate_versions: BTreeMap<String, String> = manifests
        .iter()
        .filter(|(_, text)| manifest_is_package(text))
        .filter_map(|(_, text)| {
            Some((manifest_package_name(text)?, manifest_package_version(text)?))
        })
        .collect();
```

and change the root-manifest call to:

```rust
            violations.extend(audit_workspace_manifest_publish_text(
                path,
                text,
                &crate_versions,
            ));
```

- [ ] **Step 8: Run the workspace-manifest tests to verify they pass**

Run the Step 6 command. Expected: 5 passed.

- [ ] **Step 9: Run the whole audit test file, lint, and format**

Run:
```bash
cargo nextest run -p pleiades-validate -E 'test(workspace_audit)'
cargo clippy -p pleiades-validate --all-targets --all-features -- -D warnings
cargo fmt --all
```
Expected: every test passes **except** `workspace_audit_reports_a_clean_workspace` and `workspace_audit_summary_reports_a_clean_workspace`, which fail with `publish.version-not-explicit` on 16 crates (Task 2 fixes the tree). Clippy clean. `cargo fmt` may reformat the long `filter_map`; keep whatever it produces.

- [ ] **Step 10: Commit**

```bash
git add crates/pleiades-validate/src/release/workspace_audit.rs crates/pleiades-validate/src/tests/release_workspace_audit.rs
git commit -m "chore(release): audit per-crate version pins instead of a workspace version"
```

---

### Task 2: Switch manifests, release-plz config, changelog template, root changelog note

**Files:**
- Modify: `Cargo.toml` (root, `[workspace.package]` line `version = "0.5.2"`)
- Modify: all 18 `crates/*/Cargo.toml` (line `version.workspace = true`)
- Modify: `release-plz.toml` (whole file)
- Modify: `cliff.toml` (`body` template)
- Modify: `CHANGELOG.md` (note under the header)
- Modify: `mise.toml` (`[tasks.package-check]`, lines ~75-99)
- Test: `crates/pleiades-validate/src/tests/release_workspace_audit.rs` (existing clean-workspace tests), `mise run audit`, `mise run package-check`

**Interfaces:**
- Consumes: the Task 1 rules (`publish.version-not-explicit`, per-crate `publish.workspace-dependency-version`).
- Produces: a tree where every crate declares `version = "0.5.2"`, `[workspace.package]` has no `version`, and release-plz runs with default per-crate behaviour.

- [ ] **Step 1: Confirm the clean-workspace test currently fails**

Run:
```bash
cargo nextest run -p pleiades-validate -E 'test(workspace_audit_reports_a_clean_workspace)'
```
Expected: FAIL, report lists `publish.version-not-explicit` for the 16 publishable crates.

- [ ] **Step 2: Switch every crate to an explicit version and drop the workspace version**

```bash
for m in crates/*/Cargo.toml; do sed -i 's/^version\.workspace = true$/version = "0.5.2"/' "$m"; done
sed -i '/^version = "0.5.2"$/d' Cargo.toml
grep -c '^version = "0.5.2"$' crates/*/Cargo.toml | grep -v ':1$' ; echo "---"; grep -n '^version' Cargo.toml; echo "---"; grep -c 'version = "0.5.2" }' Cargo.toml
```
Expected: the first grep prints nothing (every crate manifest has exactly one such line). The second prints nothing (no bare `version` key left in `[workspace.package]`; `rust-version` does not match `^version`). The third prints `16` (the `[workspace.dependencies]` pins are untouched).

- [ ] **Step 2b: Make `package-check` read each crate's own version**

In `mise.toml`, `[tasks.package-check]` currently extracts one version from the root `Cargo.toml` (`version=$(sed -n 's/^version = ...' Cargo.toml ...)`) and fails once that line is gone. Replace the whole `run = '''...'''` block of that task with:

```bash
set -euo pipefail
crates="pleiades-types pleiades-apparent pleiades-eclipse pleiades-time pleiades-backend pleiades-apsides pleiades-compression pleiades-houses pleiades-ayanamsa pleiades-vsop87 pleiades-elp pleiades-jpl pleiades-data pleiades-core pleiades-events pleiades-fict"
budget_bytes=$((9 * 1024 * 1024))
# Package all publishable crates together so cargo can resolve intra-workspace
# path dependencies without hitting crates.io (required before first publish).
pkg_args=""
for crate in $crates; do
  pkg_args="$pkg_args --package $crate"
done
cargo package --quiet --no-verify --allow-dirty $pkg_args
for crate in $crates; do
  # Per-crate versioning: each crate declares its own version in its manifest.
  version=$(sed -n 's/^version = "\(.*\)"$/\1/p' "crates/${crate}/Cargo.toml" | head -n 1)
  [ -n "$version" ] || { echo "package-check: could not extract version from crates/${crate}/Cargo.toml" >&2; exit 1; }
  artifact="target/package/${crate}-${version}.crate"
  size=$(wc -c < "$artifact")
  if [ "$size" -gt "$budget_bytes" ]; then
    echo "package-check: $artifact is $size bytes, over the $budget_bytes-byte budget" >&2
    exit 1
  fi
  echo "package-check: $artifact ok ($size bytes)"
done
```

Two deliberate differences from the old block: the version is read per crate, and `pleiades-events` and `pleiades-fict` are added to the list (they are publishable but were missing). Keep the `shell = "bash -c"` line and the `'''` delimiters as they are.

- [ ] **Step 3: Rewrite `release-plz.toml`**

Replace the whole file with:

```toml
[workspace]
# Changelog format shared with the 0.3.0 bootstrap (git-cliff).
changelog_config = "cliff.toml"
# Create a GitHub Release for each `<crate>-v<version>` tag.
git_release_enable = true
# Don't churn Cargo.lock as part of the release commit.
dependencies_update = false

# Per-crate versioning (since 0.5.3): release-plz's defaults apply. Each
# publishable crate is bumped from its own Conventional Commits, gets its own
# `crates/<name>/CHANGELOG.md`, tag, and GitHub Release, and the exact pins in
# `[workspace.dependencies]` are rewritten so exact-pin dependents are released
# too. The two `publish = false` crates are skipped automatically.
```

- [ ] **Step 4: Add the empty-section fallback to `cliff.toml`**

In `cliff.toml`, change the `body` template so the `{% endif %}` after the `## [Unreleased]` line is followed by the fallback. The full `body` value becomes:

```
body = """
{% if version %}\
## [{{ version | trim_start_matches(pat="v") }}] - {{ timestamp | date(format="%Y-%m-%d") }}
{% else %}\
## [Unreleased]
{% endif %}\
{% if commits | length == 0 %}
_No user-facing changes; released for a dependency or manifest update._
{% endif %}\
{% for group, commits in commits | group_by(attribute="group") %}
### {{ group | striptags | trim | upper_first }}
{% for commit in commits %}
- {{ commit.message | split(pat="\\n") | first | upper_first }} \
([{{ commit.id | truncate(length=7, end="") }}](https://github.com/rahulmutt/pleiades/commit/{{ commit.id }}))\
{% endfor %}
{% endfor %}
"""
```

Also update the first comment line of `cliff.toml` from `all future releases (release-plz)` wording to mention per-crate files, e.g.:

```toml
# Changelog config shared by the 0.3.0 bootstrap (git-cliff CLI) and all
# release-plz releases (one `crates/<name>/CHANGELOG.md` per crate since 0.5.3).
# Scope: feat / fix / perf / breaking only; a crate released only because a
# dependency pin changed gets a one-line fallback instead of an empty section.
```

- [ ] **Step 5: Freeze the root changelog with a pointer**

In `CHANGELOG.md`, insert after the line `and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).` and before `## [0.5.2] - 2026-09-27`:

```markdown

> **Since 0.5.3 each crate is versioned and released on its own.** Release
> notes from that point live in `crates/<name>/CHANGELOG.md` and on each
> crate's GitHub Release. This file is the unified-version history through
> 0.5.2 and is no longer updated.
```

- [ ] **Step 6: Run the audit tests, the audit task, and the package check**

Run:
```bash
cargo nextest run -p pleiades-validate -E 'test(workspace_audit)'
mise run audit
mise run package-check
```
Expected: all audit tests pass, including the two clean-workspace tests. `mise run audit` reports `no workspace policy violations detected`. `package-check` passes.

- [ ] **Step 7: Confirm cargo still resolves every crate at 0.5.2 with an unchanged lockfile**

Run:
```bash
cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; ms=json.load(sys.stdin)["packages"]; print(sorted({m["version"] for m in ms}), len(ms))'
git status --short Cargo.lock
```
Expected: `['0.5.2'] 18` and no output from `git status` (the lockfile did not change).

- [ ] **Step 8: Format, lint, and commit**

```bash
cargo fmt --all --check
git add Cargo.toml crates/*/Cargo.toml release-plz.toml cliff.toml CHANGELOG.md mise.toml
git commit -m "chore(release): switch to per-crate versions and changelogs"
```

---

### Task 3: cargo-release fallback for per-crate releases

**Files:**
- Modify: `release.toml` (whole file)
- Test: dry run `cargo release -p pleiades-time patch`

**Interfaces:**
- Produces: a fallback config whose tags match release-plz's `<crate>-v<version>`.

- [ ] **Step 1: Rewrite `release.toml`**

Replace the whole file with:

```toml
# Manual-release fallback (cargo-release). The default release path is
# release-plz (see .github/workflows/release-plz.yml and release-plz.toml).
# Use this only to cut a release by hand, one crate at a time:
#   cargo release -p <crate> <level> --execute
# Name each exact-pin dependent that must move too with extra `-p` flags.
# Tags use cargo-release's workspace default, `<crate>-v<version>`, which is
# the same scheme release-plz uses.
allow-branch = ["main"]
consolidate-commits = true
dependent-version = "fix"
pre-release-commit-message = "chore: release"
sign-commit = false
sign-tag = false
publish = true
push = true
tag = true
```

(Removed: `shared-version = true`, `tag-name = "v{{version}}"`, `tag-message = "Release {{version}}"`.)

- [ ] **Step 2: Dry-run a single-crate release and check the planned tag**

Run (dry run; no `--execute`; it packages and compiles the crate, ~1 minute). `release.toml` only allows `main`, so permit the feature branch for the rehearsal on the command line:
```bash
cargo release -p pleiades-time patch --allow-branch per-crate-versioning 2>&1 | grep -E 'Upgrading|Pushing|aborting release'
```
Expected output contains:
```
   Upgrading pleiades-time from 0.5.2 to 0.5.3
     Pushing Pushing per-crate-versioning, pleiades-time-v0.5.3 to origin
warning: aborting release due to dry run; re-run with `--execute`
```
The tag must read `pleiades-time-v0.5.3`, not `v0.5.3`. Warnings of the form `disabled by user, skipping <crate> which has files changed since <crate>-v0.5.2` for the other crates are expected after the manifest switch and harmless.

- [ ] **Step 3: Confirm the dry run left the tree untouched, then commit**

```bash
git status --short
```
Expected: only `release.toml` modified.

```bash
git add release.toml
git commit -m "chore(release): per-crate tags for the cargo-release fallback"
```

---

### Task 4: Documentation

**Files:**
- Modify: `README.md` (section `## Releasing`, lines ~246-267)
- Modify: `docs/release-process.md` (intro ~1-12, "Where things live" ~14-28, "Cutting a release (automated)" ~138-148, "Cutting a release (manual fallback)" ~150-165)

**Interfaces:**
- Consumes: the behaviour established in Tasks 2 and 3.

- [ ] **Step 1: Update the README `Releasing` section**

Replace the first paragraph of `## Releasing` (the one beginning `Releases are automated with`) with:

```markdown
Releases are automated with [release-plz](https://release-plz.dev). Each
publishable crate has its own version. On every push to `main`, release-plz
maintains a **Release** pull request that bumps every crate with releasable
Conventional Commits (`feat`/`fix`/`perf`/breaking), plus the crates that pin
it exactly, and updates that crate's `crates/<name>/CHANGELOG.md`. Merge that
PR to tag each bumped crate (`<crate>-v<version>`), publish it to crates.io,
and create its GitHub Release. The root `CHANGELOG.md` is the unified-version
history through 0.5.2.
```

Replace the `### Manual fallback` paragraph with:

```markdown
To cut a release by hand (e.g. if crates.io automation is unavailable), use the
retained `release.toml` config, one crate at a time:
`cargo release -p <crate> <level> --execute`.
```

- [ ] **Step 2: Update `docs/release-process.md`**

Replace the opening paragraph (`\`pleiades\` publishes **16 library crates** ... lockstep versions.`) with:

```markdown
`pleiades` publishes **16 library crates** to crates.io, each with its own
version (per-crate versioning since 0.5.3; 0.5.2 and earlier were unified).
```

In "Where things live", replace the **Release notes / changelog** bullet with:

```markdown
- **Release notes / changelog:** one `crates/<name>/CHANGELOG.md` per
  publishable crate, created by release-plz on the crate's first per-crate
  release and shipped inside the published crate. New `## [x.y.z]` sections
  are generated from Conventional Commits (`feat`/`fix`/`perf`/breaking). A
  crate released only because an internal dependency pin changed gets a
  one-line "no user-facing changes" note instead of an empty section. The
  root `CHANGELOG.md` is the unified-version history through 0.5.2 and is no
  longer updated. A **GitHub Release** is created per tag automatically.
```

Replace the **Automation config** bullet with:

```markdown
- **Automation config:** `release-plz.toml` — release-plz defaults: per-crate
  bumps, changelogs, tags, and GitHub Releases. There is no `version_group`;
  the crates are versioned independently.
```

In the **Manual fallback config** bullet, change `1.1.3` to `1.1.6`.

Replace the three numbered steps under `## Cutting a release (automated — primary)` with:

```markdown
1. Land your `feat`/`fix`/`perf`/breaking commits on `main` as usual.
2. release-plz maintains an open **Release** pull request. For each crate
   with releasable commits since its last tag it bumps that crate and
   prepends a section to `crates/<name>/CHANGELOG.md`. Because the internal
   pins in `[workspace.dependencies]` are exact, every crate that depends on
   a bumped crate is bumped too (patch) so its published manifest names the
   new version; those dependents get the one-line "no user-facing changes"
   note. A change in a widely used crate such as `pleiades-types` therefore
   still releases most of the workspace; a change in a leaf crate releases
   only that crate. Review the PR.
3. **Merge the Release PR.** On merge, `release-plz-release` publishes the
   bumped crates to crates.io in dependency order, tags each one
   (`<crate>-v{version}`), and creates their GitHub Releases. Crates that
   did not bump are untouched. `pleiades-cli`/`pleiades-validate` are skipped
   automatically.

> **One-time note (0.5.2 → 0.5.3):** the switch from a shared workspace
> version to explicit per-crate versions changed every crate's `Cargo.toml`,
> which release-plz counts as a change. The first Release PR after the switch
> bumps all 16 crates to 0.5.3; only crates with real commits (e.g. the ΔT
> fix in `pleiades-time`) have changelog entries, the rest carry the
> one-line note. Releases after that are per crate.
```

Replace step 4 under `## Cutting a release (manual fallback — cargo-release)` with:

```markdown
4. Execute: `cargo release -p <crate> <level> --execute`, where `<crate>` is
   the crate to release and `<level>` is `patch`, `minor`, or an explicit
   version such as `0.6.0`. Name every crate that pins `<crate>` exactly with
   an additional `-p` flag so their manifests are republished with the new
   pin (`dependent-version = "fix"` rewrites the pins). cargo-release bumps
   the selected crates, commits `chore: release`, publishes them in
   dependency order (waiting for the index between publishes), tags each
   one `<crate>-v{version}`, and pushes.
```

Also in that section, the rehearsal step 3 reads `cargo release <level>`; change it to `cargo release -p <crate> <level>`.

- [ ] **Step 3: Check for leftover unified-version wording**

Run:
```bash
grep -n -i "unified\|lockstep\|lock-step\|version_group\|shared-version\|tags \`v{version}\`" README.md docs/release-process.md
```
Expected: only the intentional historical mentions (the "0.5.2 and earlier were unified" sentence, the "unified-version history through 0.5.2" sentences, the one-time note, and the historical 0.3.0 bootstrap section, which stays as-is). Anything else describing current behaviour as unified is a miss; fix it.

- [ ] **Step 4: Commit**

```bash
git add README.md docs/release-process.md
git commit -m "docs(release): describe per-crate versioning and the updated fallback"
```

---

### Task 5: End-to-end verification of the release behaviour

**Files:**
- None modified. Scratch copy under the session scratchpad directory.
- Test: `release-plz update` on a copy of the branch; the blocking CI gates.

**Interfaces:**
- Consumes: the finished branch.

- [ ] **Step 1: Replay the real history in a scratch copy**

Reproduce what release-plz will see on `main` after the branch merges: the 0.5.2 tags on the release commit, then the current `main` commits, then the branch's changes. Use the session scratchpad directory (`$SCRATCH` below stands for it).

Three scratch commits, so the ΔT fix is attributed to `pleiades-time` alone and the manifest switch is a separate `chore` commit:

```bash
S="$SCRATCH/rp-verify"; rm -rf "$S"; mkdir -p "$S"
git archive e87921d66 | tar -x -C "$S"
( cd "$S" && git init -q && git add -A && git -c user.name=t -c user.email=t@t commit -qm "chore: release v0.5.2" && for c in $(ls crates); do git tag -a -m x "$c-v0.5.2"; done )
git archive main | tar -x -C "$S"
( cd "$S" && git add -A && git -c user.name=t -c user.email=t@t commit -qm "$(git -C "$OLDPWD" log -1 --format=%s main)" )
git archive HEAD | tar -x -C "$S"
( cd "$S" && git add -A && git -c user.name=t -c user.email=t@t commit -qm "chore(release): switch to per-crate versions" )
( cd "$S" && git log --oneline && release-plz update --allow-dirty 2>&1 | grep -E '^\* ' | sort )
```
Expected: three commits listed, then exactly 16 lines, one per publishable crate, each `0.5.2 -> 0.5.3`. No line for `pleiades-cli` or `pleiades-validate`.

This assumes `main` still has exactly one commit since the 0.5.2 release (the ΔT fix, #77, which touches only `pleiades-time` and docs). If `main` has moved on, the middle scratch commit squashes several real commits under the newest subject; the 16-bump count still holds, but adjust the Step 2 expectation for which crates carry a real entry.

- [ ] **Step 2: Check the generated changelogs**

```bash
( cd "$S" && sed -n 7,12p crates/pleiades-time/CHANGELOG.md; echo ---; sed -n 7,10p crates/pleiades-types/CHANGELOG.md; echo ---; git diff --stat Cargo.toml | tail -1 )
```
Expected: `pleiades-time` shows `## [0.5.3]` with a `### Fixed` entry for the ΔT fix. `pleiades-types` shows `## [0.5.3]` followed by the line `_No user-facing changes; released for a dependency or manifest update._`. The root `Cargo.toml` diff touches the 16 pin lines.

- [ ] **Step 3: Run the blocking gates on the branch**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
mise run audit
mise run package-check
cargo nextest run --workspace -E 'not package(pleiades-validate)'
cargo nextest run -p pleiades-validate -E 'test(workspace_audit)'
```
Expected: all pass.

- [ ] **Step 4: Clean up and record**

```bash
rm -rf "$S"
git status --short
git log --oneline main..HEAD
```
Expected: clean tree; six commits on the branch (spec, plan, audit, manifests/config, fallback, docs). No commit to make in this task.

---

## Self-review notes

- **Spec coverage:** §1 Manifests → Task 2 Steps 2, 7. §2 release-plz config + cliff fallback → Task 2 Steps 3–4. §3 Changelogs → Task 2 Step 5 (root note), no seeding by design. §4 Audit → Task 1. §5 Fallback → Task 3. §6 Docs → Task 4. §7 Rollout → documented in Task 4 Step 2 and verified in Task 5 Step 1. Spec testing list → Task 1 (unit tests), Task 2 Step 6 (audit, package-check), Task 3 Step 2 (cargo-release dry run), Task 5 (release-plz dry run, blocking gates).
- **Type consistency:** `audit_workspace_manifest_publish_text(path, text, &BTreeMap<String, String>)` is used identically in Task 1 Steps 5, 7 and by the report builder. `manifest_package_version` is defined in Task 1 Step 3 and used in Steps 1, 3, 7.
- **Review Focus:** each of the five lines names its pinning test or step.
