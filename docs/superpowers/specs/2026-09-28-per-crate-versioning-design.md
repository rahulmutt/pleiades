# Per-Crate Versioning — Design

**Date:** 2026-09-28
**Status:** Approved (design)

## Summary

Move the workspace from unified (lock-step) versioning, where all 16
publishable crates share one version and bump together, to per-crate
versioning, where each crate carries its own version and is released only when
it, or an internal dependency it pins exactly, changes.

release-plz stays the release tool, Conventional Commits stay the bump signal,
and the review-and-merge-a-Release-PR flow stays. The change is configuration,
manifests, the workspace audit that enforces the versioning invariant, the
manual fallback, and docs. The CI workflow is untouched.

## Context (current state)

- 18 workspace crates. 16 publish to crates.io; `pleiades-cli` and
  `pleiades-validate` are `publish = false`.
- Lockstep is encoded in three places:
  1. `[workspace.package] version = "0.5.2"` in the root `Cargo.toml`, with
     `version.workspace = true` in every crate manifest.
  2. `[workspace.dependencies]` pins every `pleiades-*` crate at the same
     exact version (`{ path = "crates/<name>", version = "0.5.2" }`).
  3. `release-plz.toml` puts all 16 publishable crates in
     `version_group = "pleiades"`.
- The changelog is a single root `CHANGELOG.md` with `## [x.y.z]` headers.
  Because several packages writing one file clobbered each other (the v0.5.0
  empty-section bug), `pleiades-core` owns the file via `changelog_include`
  and the other 15 set `changelog_update = false`.
- Tags and GitHub Releases are already per crate (`<crate>-v<version>`),
  release-plz's default. Nothing changes there.
- The workspace audit (`pleiades-validate`, `mise run audit`, blocking CI)
  asserts lockstep with two rules: `publish.workspace-version-missing` and
  `publish.workspace-dependency-version` (each pin must equal the workspace
  package version).
- The cargo-release fallback `release.toml` has `shared-version = true` and a
  `v{{version}}` tag name.

## Decisions (locked)

| Decision | Choice |
|---|---|
| Changelog layout | Per-crate `crates/<name>/CHANGELOG.md` (release-plz default). Root `CHANGELOG.md` frozen as history through 0.5.2. |
| Internal dependency pins | Exact (`version = "0.5.2"`), release-plz's default. A change in a crate releases it and its transitive dependents. |
| Manifest layout | Explicit `version = "x.y.z"` in every crate. `[workspace.dependencies]` retained; release-plz updates its pins. |
| Unpublished crates | `pleiades-cli` and `pleiades-validate` carry a static explicit version. release-plz ignores `publish = false` crates and nothing reads their version. |
| Starting point | All 16 crates start from `0.5.2` and diverge from there. No renumbering. |

### Why exact pins

Verified in a scratch copy of the repository with release-plz 0.3.169:

- With exact pins, a `fix` in `pleiades-time` rewrote the pins in
  `[workspace.dependencies]` to the new version and bumped all nine
  transitive dependents (patch). A change in `pleiades-types` or
  `pleiades-backend` therefore re-publishes all 16 crates.
- With minor-precision pins (`"0.5"`), the same fix bumped only
  `pleiades-time`; a breaking change (`feat!`) went to 0.6.0, rewrote the
  pin to `"0.6"`, and bumped only the four direct dependents.

Exact pins were chosen for safety: every published manifest names the exact
sibling version it was built against, so a consumer with a stale lockfile can
never resolve a dependent against an older sibling that lacks an API it uses.
The cost is cascade churn for widely used crates, accepted knowingly. Leaf
crates (events, eclipse, houses, ayanamsa, vsop87, elp, jpl, fict, data, core)
now release alone.

### Why the workspace dependency table stays

release-plz updates `version` inside `[workspace.dependencies]` entries
(verified). Moving internal deps into per-crate `{ path, version }` tables
would rewrite every dependency table and invert the audit's
`publish.internal-dependency-not-workspace` rule for no gain.

## Design

### 1. Manifests

- Root `Cargo.toml`: remove `version` from `[workspace.package]`. The
  `[workspace.dependencies]` `pleiades-*` entries keep their exact pins.
- Every crate manifest (all 18): replace `version.workspace = true` with
  `version = "0.5.2"`.

The `package-check` task in `mise.toml` reads the workspace version from the
root `Cargo.toml` to locate `target/package/<crate>-<version>.crate`. With no
workspace version it must read each crate's version from
`crates/<crate>/Cargo.toml` instead. Its hard-coded crate list is also two
crates short (`pleiades-events`, `pleiades-fict` are publishable but not
checked); they are added while the block is being edited.

### 2. release-plz configuration

`release-plz.toml` shrinks to the `[workspace]` table:

```toml
[workspace]
changelog_config = "cliff.toml"
git_release_enable = true
dependencies_update = false
```

Removed: `changelog_path`, every `[[package]]` block (`version_group`,
`changelog_update`, `changelog_include`), and the comment explaining the
single-changelog ownership workaround, which no longer applies.

`cliff.toml` gains a fallback in the body template so a crate released only
because a pin changed does not get a header with no body:

```
{% if commits | length == 0 %}
_No user-facing changes; released for a dependency or manifest update._
{% endif %}
```

Verified: skipped commit types are excluded from `commits` before the template
runs, so the line renders exactly when no feat/fix/perf commit touched the
crate.

`.github/workflows/release-plz.yml` is unchanged.

### 3. Changelogs

- release-plz creates `crates/<name>/CHANGELOG.md` on each crate's first
  per-crate release, using the header from `cliff.toml`. The files are not
  seeded by hand: adding a file to a crate directory would itself count as a
  change. Each file ships inside its published crate.
- Root `CHANGELOG.md` keeps its history through 0.5.2 and gets a short note at
  the top: from 0.5.3 onward each crate has its own changelog under
  `crates/<name>/CHANGELOG.md`.

### 4. Workspace audit (`crates/pleiades-validate/src/release/workspace_audit.rs`)

The workspace-manifest publish check (`audit_workspace_manifest_publish_text`)
currently takes the root manifest text alone and compares pins against the
workspace version. It will take a map of crate name → declared version,
collected by the report builder (`workspace_audit_report_uncached`) from the
crate manifests it already walks.

Rule changes:

- **Removed:** `publish.workspace-version-missing`.
- **Changed:** `publish.workspace-dependency-version` fires when a
  `[workspace.dependencies]` pin for `pleiades-<name>` differs from the
  `[package] version` declared in `crates/pleiades-<name>/Cargo.toml`, or
  when the pin is absent. Same rule name, per-crate comparison.
- **New:** `publish.version-not-explicit` on publishable crates: `[package]`
  must declare a literal `version = "x.y.z"`. Inheriting it would silently
  return the crate to lockstep and release-plz could not bump it alone.

Untouched: `publish.internal-dependency-not-workspace`,
`publish.internal-dependency-unpublishable`,
`publish.internal-dev-dependency-not-path-only`, and all metadata/file rules.

### 5. Manual fallback (`release.toml`, cargo-release 1.1.6)

- Drop `shared-version`, `tag-name`, and `tag-message`. cargo-release's
  defaults then produce `<crate>-v<version>` tags, matching release-plz.
- `pre-release-commit-message = "chore: release"`, since `{{version}}` is
  undefined when several crates release at different versions.
- Documented command: `cargo release -p <crate> <level> --execute`. When the
  crate has exact-pin dependents that must move too, name them with extra
  `-p` flags. `dependent-version = "fix"` keeps the pins updated.

### 6. Docs

- `README.md` "Releasing": per-crate bumps and changelog location.
- `docs/release-process.md`: rewrite the unified-version statements; describe
  the per-crate flow, the exact-pin cascade, the per-crate changelogs, the
  one-time all-crates 0.5.3 after the switch (see Rollout), the updated
  fallback commands; correct the stale cargo-release pin (1.1.3 → 1.1.6).
- `docs/threat-model.md` already says "per-crate tags"; no change.

### 7. Rollout

One PR lands everything above. The first Release PR after it will bump **all
16 crates to 0.5.3**. This is expected and unavoidable: release-plz compares
each crate's source `Cargo.toml` against the `Cargo.toml.orig` inside the
crates.io package, and the switch from `version.workspace = true` to
`version = "0.5.2"` differs in every crate. Verified by replaying the real
history (0.5.2 tags → the #77 ΔT fix → the switch commit): `pleiades-time`
gets the ΔT fix in its changelog; the other 15 get the fallback line. From
then on only changed crates and their exact-pin dependents move.

## Testing

- Unit tests in `crates/pleiades-validate/src/tests/release_workspace_audit.rs`:
  pin mismatch flagged; matching pins clean; missing pin flagged; inherited
  version flagged with `publish.version-not-explicit`; the test for the
  removed rule replaced. The existing clean-workspace test runs against the
  real tree and proves the switched manifests pass.
- `mise run audit`, `mise run package-check`, `cargo fmt --all --check`,
  clippy with `-D warnings`, and the blocking test tier on the branch.
- release-plz dry run: `release-plz update` in a scratch copy of the branch
  must report exactly 16 bumps to 0.5.3, the ΔT fix under `pleiades-time`,
  and the fallback line in the other changelogs.
- cargo-release dry run: `cargo release -p pleiades-time patch` (no
  `--execute`) must plan a `pleiades-time-v0.5.3` tag.

## Out of scope

- Loosening pins to minor precision (rejected above).
- Seeding per-crate changelogs with pre-0.5.3 history.
- Any change to the release workflow, tokens, or crates.io setup.
