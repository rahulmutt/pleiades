# Release process

`pleiades` publishes **14 library crates** to crates.io with lockstep versions.
Only `pleiades-cli` and `pleiades-validate` are `publish = false` and are never
published. Publish metadata is enforced by `mise run audit` (workspace audit
`publish.*` rules) and `mise run package-check` (artifact size budget), both
part of `ci` and `release-gate`.

Releases are **automated with [release-plz](https://release-plz.dev)** (the
primary path). The manual `cargo-release` flow is retained as a documented
fallback for cutting a release by hand.

## Where things live

- **Release notes / changelog:** `CHANGELOG.md` at the repo root. New `## [x.y.z]`
  sections are generated from Conventional Commits (`feat`/`fix`/`perf`/breaking)
  and prepended above the curated history. There is no separate `releases/`
  directory — the changelog is the single source of release notes, and a
  **GitHub Release** is created per tag automatically by release-plz.
- **Changelog format:** `cliff.toml` (git-cliff config, shared by the bootstrap
  changelog and release-plz).
- **Automation config:** `release-plz.toml` — unified `version_group = "pleiades"`
  so every publishable crate bumps lockstep, and `changelog_path = "CHANGELOG.md"`
  so release-plz maintains the single root changelog (not per-crate files).
- **CI workflow:** `.github/workflows/release-plz.yml` (two jobs: `release-plz-pr`
  opens/updates the Release PR; `release-plz-release` publishes + tags on merge).
- **Manual fallback config:** `release.toml` (cargo-release, pinned to `1.1.2` in
  `mise.toml`).

## One-time setup

**For the automated flow**, configure two GitHub Actions secrets (Settings →
Secrets and variables → Actions):

1. `CARGO_REGISTRY_TOKEN` — a crates.io API token
   (<https://crates.io/settings/tokens>) with publish scope for the `pleiades-*`
   crates.
2. `RELEASE_PLZ_TOKEN` — a GitHub token the workflow uses so its PRs and tags
   trigger CI. Prefer a GitHub App installation token; a fine-grained PAT with
   `contents: write` + `pull-requests: write` also works. The default
   `GITHUB_TOKEN` cannot trigger downstream workflows, so it is not sufficient.

**For the manual fallback**, create a crates.io token as above, then run
`cargo login` and paste it.

## Cutting a release (automated — primary)

1. Land your `feat`/`fix`/`perf`/breaking commits on `main` as usual.
2. release-plz maintains an open **Release** pull request that bumps all 14
   publishable crates to the next unified version and updates `CHANGELOG.md`.
   Review it.
3. **Merge the Release PR.** On merge, `release-plz-release` publishes every
   publishable crate to crates.io in dependency order, tags `v{version}`, and
   creates the GitHub Release. The `pleiades-cli`/`pleiades-validate` crates are
   skipped automatically.

## Cutting a release (manual fallback — cargo-release)

Use this only if the automation is unavailable.

1. Start from a clean, pushed `main` checkout.
2. Run the release gate: `mise run release-gate`.
3. Rehearse: `cargo release <level>` (without `--execute` this is a dry run)
   and review the planned version bump, publish order, tag, and push.
4. Execute: `cargo release <level> --execute`, where `<level>` is `patch`,
   `minor`, or an explicit version such as `0.3.0`. cargo-release bumps the
   shared workspace version, updates the pinned `workspace.dependencies`
   versions, commits `Release {version}`, publishes the crates in dependency
   order (waiting for the index between publishes), tags `v{version}`, and
   pushes.

### First-release note

crates.io rate-limits brand-new crate names (burst of 5). Publishing several new
crate names for the first time will trip it, and cargo-release's preflight
refuses to start while the plan exceeds the limit (release-plz surfaces the same
crates.io limit). Either ask crates.io support (help@crates.io) to raise the
limit for the initial release, or publish in batches a while apart — re-running
the release skips crates already published at that version. Subsequent releases
update existing crates, which have a much higher limit, and are unaffected.

## After publishing

- Confirm the docs.rs build for each crate at `https://docs.rs/<crate>`.
- In a scratch project, run
  `cargo add pleiades-core pleiades-data pleiades-eclipse` and build a minimal
  chart against the published versions (`pleiades-data` ships the packaged
  offline backend, so this works without a local ephemeris).

## Recovery and mistakes

- **Publish fails midway:** fix the cause and re-run — merging the Release PR
  again (automated) or re-running `cargo release ... --execute` (manual) skips
  crates already published at that version.
- **Bad release:** fix forward with a patch release; crates.io never allows
  re-publishing a version. Reserve `cargo yank` for unsound or badly broken
  releases, and yank only after the fixed version is available.
