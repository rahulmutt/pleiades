# Release process

`pleiades` publishes nine library crates to crates.io with lockstep versions
managed by `cargo-release` (pinned exactly in `mise.toml`; the 0.25 line is the series this configuration was validated against — bump deliberately, rerun the dry-run rehearsal after upgrading). `pleiades-cli`,
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
5. **First-release note:** crates.io rate-limits brand-new crate names (burst
   of 5). Publishing all nine crates for the first time will trip it, and
   cargo-release's preflight refuses to start while the plan exceeds the
   limit. Either ask crates.io support (help@crates.io) to raise the limit
   for the initial release, or publish in two batches a while apart —
   re-running the same `cargo release ... --execute` skips crates already
   published at that version. Subsequent releases update existing crates,
   which have a much higher limit, and are unaffected.

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
