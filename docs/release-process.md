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
   trigger CI (see [Creating the `RELEASE_PLZ_TOKEN`](#creating-the-release_plz_token)
   below). The default `GITHUB_TOKEN` cannot trigger downstream workflows, so it
   is not sufficient.

**For the manual fallback**, create a crates.io token as above, then run
`cargo login` and paste it.

### Creating the `RELEASE_PLZ_TOKEN`

`RELEASE_PLZ_TOKEN` is not a special token type — it is just the secret name the
workflow reads (`GITHUB_TOKEN: ${{ secrets.RELEASE_PLZ_TOKEN }}` in both jobs of
`.github/workflows/release-plz.yml`). It holds a GitHub token that is **not** the
built-in `GITHUB_TOKEN`.

Why a separate token: GitHub deliberately prevents the built-in `GITHUB_TOKEN`
from triggering other workflows (a loop-prevention safeguard). So if release-plz
opened its Release PR or pushed the `v{version}` tag with the default token, your
CI would not run on that PR and no tag-triggered workflow would fire. A
non-default token makes those PRs and tags trigger CI normally.

**Fine-grained personal access token (simplest path):**

1. Go to GitHub → **Settings → Developer settings → Personal access tokens →
   Fine-grained tokens → Generate new token**
   (<https://github.com/settings/tokens?type=beta>).
2. **Token name:** e.g. `pleiades-release-plz`. Set an **Expiration** (GitHub
   requires one; pick a cadence you are willing to rotate on, e.g. 90 days).
3. **Resource owner:** `rahulmutt` (the account that owns the repo).
4. **Repository access:** select **Only select repositories** → choose
   `rahulmutt/pleiades`.
5. **Repository permissions** — grant exactly these two (leave everything else at
   *No access*):
   - **Contents:** *Read and write* (lets release-plz commit the version bump,
     push the `v{version}` tag, and create the GitHub Release).
   - **Pull requests:** *Read and write* (lets release-plz open and update the
     Release PR).
6. Click **Generate token** and **copy it now** — GitHub shows the value only
   once.
7. Add it to the repository as a secret: **repo → Settings → Secrets and
   variables → Actions → New repository secret**. Name it exactly
   `RELEASE_PLZ_TOKEN` and paste the token as the value.

When the token expires, regenerate it (or a new one) with the same two
permissions and update the `RELEASE_PLZ_TOKEN` secret.

**GitHub App token (recommended long-term, more secure):** instead of a PAT tied
to your account, create a GitHub App with the same *Contents: read & write* and
*Pull requests: read & write* permissions, install it on the repo, store its App
ID and private key as secrets, and mint a short-lived installation token in the
workflow with `actions/create-github-app-token`. The token auto-rotates and is
not tied to a personal account. For a solo repo the fine-grained PAT above is
fine to start; you can migrate to an App later.

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
