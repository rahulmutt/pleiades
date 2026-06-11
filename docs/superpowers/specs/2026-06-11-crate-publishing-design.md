# Crate Publishing Design

Date: 2026-06-11
Status: Approved design, pending implementation plan

## Goal

Publish the `pleiades` library crates to crates.io as an explicitly experimental
0.1.x release, and make publishability a fail-closed, repeatable invariant of the
workspace rather than a one-time manual effort.

This design does **not** change the maturity posture in `PLAN.md`: the published
crates carry the same documented limits (mean geometric coordinates, draft data
artifact, Pluto fallback, etc.) as the repository today.

## Decisions

| Question | Decision |
| --- | --- |
| Release posture | Early 0.1.x with documented limits (not name-grab, not production-gated) |
| Crates published | The 9 library crates: `pleiades-types`, `pleiades-backend`, `pleiades-core`, `pleiades-houses`, `pleiades-ayanamsa`, `pleiades-vsop87`, `pleiades-elp`, `pleiades-jpl`, `pleiades-compression` |
| Crates kept unpublished | `pleiades-cli`, `pleiades-data`, `pleiades-validate` (contributor tooling and draft artifact; keep `publish = false`) |
| Versioning | Lockstep: all crates share `workspace.package.version`, bumped together |
| Release tooling | `cargo-release`, run locally, pinned in `mise.toml` |
| License | Dual `MIT OR Apache-2.0`; add `LICENSE-MIT`, rename existing `LICENSE` to `LICENSE-APACHE` |
| Regression protection | Static publish-metadata rules in the existing `pleiades-validate` workspace audit, plus a `cargo package --no-verify` size/content check as a `mise` task in `ci` and `release-gate` |

Name availability was verified on 2026-06-11 via the crates.io sparse index: all
nine `pleiades-*` names are unregistered. (The bare name `pleiades` is taken by an
unrelated crate and is not used by this design.)

## 1. Cargo manifest changes

### Workspace root `Cargo.toml`

- `[workspace.package]` gains `repository = "https://github.com/rahulmutt/pleiades"`,
  `homepage` (same URL), `keywords`, and `categories` (crates.io caps both at 5;
  final picks chosen during implementation, e.g. `["astronomy", "astrology",
  "ephemeris"]` and `["science"]`).
- `[workspace.dependencies]` gains one entry per publishable internal crate with
  both `path` and `version`, e.g.
  `pleiades-types = { path = "crates/pleiades-types", version = "0.1.0" }`.
  This keeps the crates.io requirement (internal deps must carry a version)
  DRY under lockstep versioning. The `version` here must stay equal to
  `workspace.package.version`; cargo-release maintains it on bumps and the
  audit enforces it.

### Each publishable crate `Cargo.toml`

- Remove `publish = false`.
- Add a one-line `description` (required by crates.io).
- Add `repository.workspace = true`, `homepage.workspace = true`,
  `keywords.workspace = true`, `categories.workspace = true`,
  `readme = "README.md"`.
- Replace plain path deps on internal crates with `{ workspace = true }`
  references.
- Crates with a `serde` feature (`types`, `backend`, `compression`) add
  `[package.metadata.docs.rs] all-features = true`.

### Unpublished crates

`pleiades-cli`, `pleiades-data`, `pleiades-validate` keep `publish = false`.
They may also adopt `{ workspace = true }` internal deps for consistency, but
nothing about them is required to change.

## 2. Licensing

- Rename `LICENSE` → `LICENSE-APACHE` (content is already the Apache-2.0 text).
- Add `LICENSE-MIT` at the repo root with the project copyright line.
- Place byte-identical copies of both files in each of the 9 publishable crate
  directories so they ship inside the `.crate` packages (cargo does not package
  files from outside the crate directory). The workspace audit verifies the
  copies exist and match the root files, so they cannot drift.

## 3. Package contents

- `pleiades-vsop87` ships as-is: generated `src/` tables, `data/*.bin`
  (compile-time `include_bytes!`), raw `data/VSOP87B.*` source tables
  (test-referenced, provenance), and the `regenerate-vsop87b-tables` bin.
  Estimated well under the 10 MB compressed crates.io limit.
- `pleiades-jpl` ships its checked-in snapshot sources as-is.
- No `include`/`exclude` lists initially; the package-check gate (section 5)
  guards size and would surface any future fixture bloat.
- Each publishable crate gets a short `README.md`: what the crate is, where it
  sits in the workspace layering, a link to the repository, and the relevant
  maturity caveats (mean geometric coordinates, draft artifact, experimental
  0.1.x status).
- The root `README.md` gains a "Published crates" section stating the
  experimental status so crates.io/GitHub visitors see the posture up front.

## 4. Release tooling

- Add `cargo-release` to `mise.toml` `[tools]` with a pinned version, keeping
  tool provenance inside the existing audit posture.
- Add a workspace-root `release.toml`:
  - `shared-version = true` (lockstep bumps from `workspace.package.version`),
  - single tag per release, `tag-name = "v{{version}}"`,
  - publish enabled; cargo-release skips `publish = false` crates and publishes
    in dependency order with index-propagation waits.
- Release flow: from a clean `main`, run `mise run release-gate`, then
  `cargo release <level-or-version> --execute`.

## 5. Publishability gate

Two layers, both fail-closed:

### Static rules in the existing workspace audit (`pleiades-validate`)

For every workspace crate **not** marked `publish = false`:

- `description` is present and non-blank;
- `license` is exactly `MIT OR Apache-2.0`;
- `repository`, `readme`, `keywords`, and `categories` resolve (directly or via
  `workspace = true`);
- the crate's `README.md`, `LICENSE-APACHE`, and `LICENSE-MIT` files exist, and
  the license files are byte-identical to the repo-root copies;
- every dependency on an internal `pleiades-*` crate carries both `path` and
  `version`, and that version equals `workspace.package.version`;
- every internal dependency of a publishable crate is itself publishable
  (no path-only or unpublished deps reachable from the published set).

Violations are reported through the existing `WorkspaceAuditViolation`
mechanism and fail `mise run audit`, hence CI and `release-gate`, with no new
CI wiring.

### Package check (`mise` task `package-check`)

- Runs `cargo package --no-verify -p <crate>` for each publishable crate.
- Asserts each produced `.crate` file is under a 9 MB budget (headroom below
  the 10 MB crates.io limit).
- Added to the `ci` and `release-gate` task dependency lists.
- `--no-verify` keeps CI fast; the full verifying build happens at release time
  via cargo-release / `cargo publish`.

## 6. First-release process (one-time)

1. Land all of the above on `main` with CI green.
2. Maintainer creates a crates.io API token and runs `cargo login` (manual,
   cannot be automated here).
3. Rehearse with `cargo release 0.1.0 --dry-run`, review the plan output.
4. Execute `cargo release 0.1.0 --execute`: publishes the 9 crates in
   dependency order, tags `v0.1.0`, pushes.
5. Post-publish verification: confirm docs.rs builds for all 9 crates, and in a
   scratch project run `cargo add pleiades-core pleiades-vsop87 pleiades-elp`
   and compile a minimal chart example against the published versions.

## 7. Error handling and testing

- New audit rules get unit tests alongside the existing workspace-audit tests,
  using fixture manifests that violate each rule individually.
- Partial-publish recovery: if publishing fails midway, re-running cargo-release
  skips already-published versions; this is documented in a short
  `docs/release-process.md` along with the token setup and release commands.
- Mistake policy: fix-forward with a patch release; published versions are
  never reused (crates.io forbids it) and yanking is reserved for
  unsound/broken releases.

## Out of scope

- Publishing `pleiades-cli`, `pleiades-data`, or `pleiades-validate`.
- Any change to backend capabilities, accuracy claims, or the PLAN.md phases.
- CI-driven publishing (release-plz et al.) — revisit if local cargo-release
  becomes friction.
- A `pleiades` umbrella/facade crate under the bare (taken) name.
