# Dense asteroid fits in the packaged artifact

Date: 2026-10-08. Issue: #201 (the "densify" option). Status: design,
awaiting review.

## Problem

Since #158 (PR #202) the offline backends fail closed for asteroids. Ceres,
Pallas, Juno, Vesta and `asteroid:433-Eros` are served only by
`JplSnapshotBackend`, at or between the closely spaced rows of the sparse
Horizons fixture (a handful of isolated epochs and a nine-day cluster in
January 2001). Any other date returns `OutOfRangeInstant`. Of the 407 rows per
body in `crates/pleiades-jpl/data/corpus/asteroid_reference.csv`, each body is
served at one (JD 2453000.5) and refused at 406.

The packaged artifact still carries 17-row Eros segments that are wrong by
tens of degrees on most dates. `is_carried_but_unserved` keeps
`PackagedDataBackend` from serving them.

Asteroid stations (#167 (d)), asteroid aspects (#168 (f)) and the asteroid
half of the `validate-nod-aps` reference gap (#160) all wait on dense offline
coverage.

## Goal

`PackagedDataBackend` serves the five bodies on every date in 1900–2100, at
release grade, from fits to the JPL small-body kernel `sb441-n373s`. A blocking
gate checks them against independent rows.

## Scope

In:

- Dense fits of Ceres, Pallas, Juno, Vesta and `asteroid:433-Eros` in
  `packaged-artifact.bin`, sampled from `sb441-n373s.bsp` with `de440.bsp`.
- Removal of the 17-row Eros re-fit and of `is_carried_but_unserved`.
- Claims, metadata text, compatibility profile and documentation for the new
  bodies.
- A blocking accuracy gate against `asteroid_reference.csv`.

Out (stay open on their issues):

- #201: real Horizons rows for Apophis and Juno, the 1.0″/1.9″
  Horizons-versus-`sb441` difference at JD 2453000.5, and
  `JplSnapshotBackend`'s backend-level `AccuracyClass::Exact` and
  `nominal_range`.
- #167 (d), #168 (f), #160: asteroid stations, aspects and nod_aps reference
  rows. Each gets its own change once this lands.
- Apophis and the other 31 bodies of the asteroid corpus.

## Design

### 1. Data flow

`regenerate_packaged_artifact_from_kernel[_over]`
(`crates/pleiades-data/src/regenerate.rs`) takes a second kernel path, the
asteroid kernel. It builds one `SpkBackend` from de440 and `sb441-n373s`, the
same composition `regenerate-asteroid-corpus` uses.

In `build_packaged_artifact_from_reference_over`, the five bodies take the
major-body branch: `fitting_segment_boundaries` over the base window and then
`fit_segment_within_span` for each span. The `SelectedAsteroids` /
`CustomBodies` branch, which re-fits Eros from `reference_snapshot.csv` with
`body_segments_from_entries`, is deleted, along with any helpers it alone
uses.

`body_uses_heliocentric_frame` includes the five bodies. Their samples are
reduced against the de440 Sun and stored as `StoredFrame::Heliocentric`. Lookup
already recombines a heliocentric segment with the geocentric Sun by the stored
frame, so lookup does not change. A heliocentric asteroid orbit is smooth. A
geocentric track has a retrograde loop every synodic period and would need
much shorter segments for the same accuracy.

`packaged_bodies()` lists the five bodies in a fixed order after the ten base
bodies: Ceres, Pallas, Juno, Vesta, Eros. The body order fixes the artifact's
bytes.

Both regeneration entry points (`generate-packaged-artifact` in
`pleiades-validate` and `pleiades-cli`) require `PLEIADES_AST_KERNEL` as well
as `PLEIADES_DE_KERNEL`, and refuse to write without both. The byte-identity
test `crates/pleiades-data/tests/artifact_regen.rs` needs both and skips when
either is missing, as it does today for one. `docs/spk-kernel-sourcing.md`
gains the regeneration recipe.

### 2. Segment spans and size

`fitting_segment_span_days` (`coverage/generation_spec.rs`) gets one entry
per asteroid. The degree stays 8 and the oversampling 3, as for every body.
Initial spans, to be confirmed by measurement:

| Body | Initial span | Reason |
|---|---|---|
| Ceres, Pallas, Juno, Vesta | 64 d | heliocentric period about 4.6 years |
| Eros | 32 d | Mars-like orbit; at close approach about 0.15 AU from Earth, which magnifies a heliocentric error about 7× |

Acceptance rule per body: the largest geocentric error of the packaged
backend against the two-kernel `SpkBackend`, sampled every 0.5 day over
1900–2100, is at most 1″ in longitude and in latitude. The chosen span is the
longest power-of-two span that meets the rule.

The measurement is a maintainer test in `pleiades-data`. It is `#[ignore]` and
skips without both kernels, so it can be re-run after a regeneration, and it
prints per-body maxima. The figures go into this spec's results note and the
PR.

Size: the artifact is 10,491,298 bytes against
`PACKAGED_BUDGETS.max_encoded_bytes = 12_000_000`, and `package-check` caps
the gzipped `.crate` at 9 MiB. At about 260 bytes per segment (today's
average), the initial spans add about 1.8 MB, minus the 17-row Eros segments.
That would exceed the budget, unless slow bodies encode smaller per segment.
The real figure is measured.

If the longest spans that meet the 1″ rule do not fit the 12 MB budget, work
stops and the trade-off goes back to the maintainer: longer spans with a
looser rule, or a larger budget. The budget is not raised inside this change
without that decision.

### 3. Serving, claims and metadata

- `is_carried_but_unserved` and its call sites (`lib.rs` claims,
  `backend.rs` metadata, `supports_body` and the `UnsupportedBody` error,
  `lookup.rs` ×3) are removed. `PackagedDataBackend` serves the five bodies
  like any other packaged body: inside the window, outside it
  `OutOfRangeInstant`.
- `packaged_body_claims` gives each asteroid
  `BodyClaim::release_grade(body, AccuracyClass::High,
  ClaimEvidence::CorpusValidated { source: "sb441-n373s".into() })`. The
  planets keep `ArtifactValidated`.
- The metadata text "11 bundled bodies (…, asteroid:433-Eros)" becomes "15
  bundled bodies (…)". The tests that pin it are updated with it
  (`pleiades-data`, `pleiades-compression`, `pleiades-cli`,
  `pleiades-validate` release-bundle tests).
- In the default routing chain `PackagedDataBackend` comes first, so it
  answers the five bodies. `JplSnapshotBackend` is unchanged and still answers
  Apophis at its rows.
- The compatibility profile gets a release note and moves from 0.7.33 to
  0.7.34. A chart that asked for an asteroid away from the fixture rows
  returned an error and now returns a position. A chart inside the January
  2001 cluster moves from the snapshot interpolation to the packaged fit, by
  up to about 2″ (the Horizons-versus-`sb441` difference).
- Docs: the known limit in `docs/status.md`, the Eros storage and ceiling
  rows in `spec/data-compression.md`, the isolated-row chart note in
  `docs/cli.md`, the `pleiades-data` README's body list, and the stale
  `nod_aps` and `apparent-goldens.csv` notes that describe asteroids as
  unserved.

### 4. Gates and tests

- **New blocking gate.** A `pleiades-data` test compares
  `PackagedDataBackend` with every `asteroid_reference.csv` row of the five
  bodies (407 × 5) in geocentric ecliptic J2000, geometric. The ceiling per
  channel is the measured maximum × 1.4, rounded up, which is well inside the
  `Asteroid` class's 30″.
  - The rows are independent of the fit's sampling instants but come from
    the same kernel. The gate therefore measures fit error, not ephemeris
    error, as the planets' hold-out gate does for de440.
  - It is pinned by row count, and it reads the corpus through the existing
    checksum-guarded loader.
- **Changed on purpose.** `pleiades-cli/src/cli/tests/asteroid_gate.rs`
  pins one served and 406 refused epochs per body through `jpl-snapshot`. It
  becomes all 407 served by `PackagedDataBackend` within its 5″ tolerance. The
  #202 tests pinning the refusal of a packaged asteroid become tests that it
  is served. Tests pinning the refusal outside the window stay.
- **Removed.** The Eros self-consistency check in `accuracy_baseline.rs`,
  superseded by the gate.
- **Claims audit.** `pleiades-validate/src/claims/audit.rs` compares the
  packaged release-grade bodies with the hold-out corpus only where a body
  has hold-out rows, so it would skip the asteroids silently. It gains the
  asteroid gate as their evidence. A release-grade body with no evidence row
  fails the audit.
- **Budgets.** `encoded_artifact_within_size_budget`, `package-check`, and two
  regenerations byte-identical.
- **Downstream fixtures.** Anything else the new bodies or bytes move (artifact
  checksum pins, release-bundle manifests, the crossings golden if a longitude
  moves) is regenerated in the same change. Base-body segments must come out
  bit-identical: the change adds bodies and does not refit the base ten.
- **Tiers.** `mise run ci` and `release-smoke` before the PR. `mise run
  test-full` before the PR, since `pleiades-validate` carries the slow
  release-bundle tests.

## Risks

- **Size.** Covered by the stop rule in section 2.
- **Bit-identity of the base bodies.** The change must not refit or move
  them. A test compares each base body's segments in the new artifact with
  the committed one, so a regression shows as a named body, not just a
  changed checksum.
- **Kernel availability.** Regeneration needs about 1.1 GB of kernels that are
  not committed. CI never regenerates; it checks the committed artifact. The
  kernels' SHA-256 values are pinned in `docs/spk-kernel-sourcing.md` (and the
  asteroid kernel's in `spk::corpus_spec`). Regeneration does not hash the
  1.1 GB of input. A wrong kernel shows up as a byte-identity failure against
  the committed artifact.
