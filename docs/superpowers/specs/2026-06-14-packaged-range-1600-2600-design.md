# Packaged Range Realignment: 1500–2500 CE → 1600–2600 CE

Date: 2026-06-14
Status: Approved for Layers 1+2; Layer 3 deferred

## Problem

The spec, plan, docs, and parts of the code claim the packaged compressed
ephemeris is optimized for **1500–2500 CE**. The first-party reference kernel is
`de440.bsp`, whose advertised coverage is approximately **1550-01-01 to
2650-01-01**. The 1500 floor therefore sits *below* de440, which forced a
documented "1500–1550 CE known gap" that would only close with the ~3 GB
`de441`. That gap is pure accidental complexity for the first release.

## Decision

Realign the packaged-range requirement to **1600–2600 CE**, a round window that
sits comfortably inside de440 with margin at both ends. This:

- eliminates the 1500–1550 known gap entirely (the floor moves *into* de440), and
- keeps de441 out of the first-release critical path.

The supported range is a **claim/requirement**, asserted in prose and in a few
code-level labels. Realigning it is mostly a wording change; the one place it is
*not* mechanical is the boundary reference fixtures, which carry real
astronomical numbers. We therefore split the work.

## Scope

### In scope now — Layer 1 (docs/spec prose)

Replace the `1500-2500 CE` range claim with `1600-2600 CE` in every prose
location, and retire the now-obsolete known-gap narrative.

Files (range claim):
- `SPEC.md` (lines ~12, 23, 36, 67, 106, 118)
- `spec/requirements.md` (~101, 107)
- `spec/roadmap.md` (~28)
- `spec/data-compression.md` (~7, 107)
- `spec/backends.md` (~45)
- `spec/vision-and-scope.md` (~31)
- `spec/architecture.md` (~31)
- `README.md` (~23)
- `PLAN.md` (~42, 70)
- `AGENTS.md` (~341)
- `prompts/bootstrap.md` (~6)
- `plan/overview.md` (~11)
- `plan/status/01-current-execution-frontier.md` (~36)
- `plan/status/02-next-slice-candidates.md` (~19)
- `plan/tracks/03-backends-and-distribution.md` (~19)
- `plan/stages/01-production-reference-corpus.md` (~6, 33)
- `plan/stages/02-production-compressed-ephemeris.md` (~6, 36)
- `plan/appendices/02-phase-workable-state-matrix.md` (~6)

Files (known-gap retirement):
- `docs/spk-kernel-sourcing.md`: rewrite the "Coverage and the 1500 CE known
  gap" section. de440 (~1550–2650) now *contains* the 1600–2600 target, so there
  is no floor gap. Keep the de440 source/URL/license/SHA placeholder; note the
  de441 escalation is no longer needed for the target range.
- `docs/superpowers/specs/2026-06-13-spk-reference-backend-design.md` and
  `docs/superpowers/plans/2026-06-13-spk-reference-backend.md`: these are
  historical, already-shipped artifacts. **Leave them unchanged** — they record
  what was true when written. Dated design/plan artifacts are historical records,
  not live docs.

### In scope now — Layer 2 (code range-labels + their tests)

These are display/label strings only; no behavior or numeric data changes.

Source strings:
- `crates/pleiades-data/src/lib.rs:1` — module doc "…for the common 1500-2500 range."
- `crates/pleiades-validate/src/corpus/mod.rs` — `:188` doc comment, `:206` and
  `:336` corpus name `"Representative 1500-2500 window"`, `:337` description
  `"Reduced timing subset of the representative 1500-2500 benchmark corpus."`
- `crates/pleiades-validate/src/render/summary/release.rs:130` — `"…advertised 1500-2500 CE window…"`
- `crates/pleiades-validate/src/render/summary/writers.rs:839` — `"…generated 1500-2500 production artifacts are Phase 2 work"`
- `crates/pleiades-validate/src/render/cli.rs:2062` — help banner `"…representative 1500-2500 window corpus…"`

Tests asserting the above (must change in lockstep):
- `crates/pleiades-validate/src/tests/corpus.rs:92`
- `crates/pleiades-validate/src/tests/report.rs:14, 302`
- `crates/pleiades-validate/src/tests/release_bundle_verify_b.rs:926, 927`
- `crates/pleiades-validate/src/tests/snapshot_render.rs:1264, 1403, 2462, 2496`
- `crates/pleiades-cli/src/cli/tests/help.rs:819`
- `crates/pleiades-cli/src/cli/tests/summary_commands.rs:206`

### Deferred — Layer 3 (boundary fixtures)

Not in this change. Tracked as a follow-up (it is genuine Phase-1 reference-corpus
work, not a label edit).

The boundary reference fixtures are checked-in CSV rows
(`epoch_jd,body,x_km,y_km,z_km`) in `crates/pleiades-jpl/data/*.csv`, pinned by
checksums and `validate()` methods, with year-labelled epoch constants in
`crates/pleiades-jpl/src/backend.rs` (e.g. `…_1500_…EPOCH_JD = 2_268_932.5`,
`…_2500_…EPOCH_JD = 2_634_167.0`) and summary types in
`crates/pleiades-jpl/src/reference_summary/reference_snapshot/boundaries/`.

After this change the **claim** is 1600–2600 CE while the **fixtures** still
bracket 1500–2500 (floor 1499-12 / ceiling year-2500). Reconciling them requires:
- A new **2600 CE ceiling** boundary fixture (JD ≈ 2_670_000). This row does not
  exist; the current ceiling is year-2500. Its `x/y/z` must be correct
  geocentric-ecliptic-**J2000**, **TDB**, **mean geometric** values sourced from
  de440 (preferred, deterministic) or the JPL Horizons API (reachable here), with
  the reduction matched exactly and all checksums re-pinned.
- A decision on the now-below-range **1500 floor** fixture: retire it, or keep it
  as historical-margin evidence labelled as outside the claimed range.

Layer 3 must not fabricate placeholder numbers — that would corrupt the
fail-closed regression evidence the project depends on.

## Non-goals

- No change to de440 as the source kernel, nor adoption of de441.
- No change to body/channel/frame claims, accuracy thresholds, or artifact format.
- No fixture-data regeneration, epoch-constant renames, or CLI fixture-command
  renames (all Layer 3).

## Verification

- `cargo build` and `cargo test` across the workspace pass with the updated
  label strings and tests.
- `cargo fmt --check` and `cargo clippy` stay clean.
- `grep -rn "1500-2500"` over tracked sources returns only the intentionally
  retained historical superpowers artifacts from 2026-06-13.
- A manual read confirms no remaining prose claims the 1500–1550 known gap.

## Risk / known follow-up

The Layer 2 posture string in `release.rs:130` ("coverage … aligned across the
advertised 1600-2600 CE window") will, post-change, describe a 1600–2600 *claim*
over fixtures that still physically span 1500–2500. This is the deliberate,
documented Layer 3 gap. If any test cross-checks that label against the corpus's
actual min/max epoch (rather than asserting a fixed string), it will fail and
surface real coupling — at which point Layer 3 is a hard prerequisite and we stop
and reassess rather than loosen the check.
