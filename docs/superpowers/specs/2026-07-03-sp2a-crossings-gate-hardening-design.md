# SP-2a-FU · `validate-crossings` Gate Hardening — Design

Status: **draft — 2026-07-03**. Design proposed and reviewed in brainstorming;
awaiting written-spec review before handing to writing-plans.

This is a follow-up to **SP-2a** (longitude crossings, landed and merged
2026-07-03, `docs/sp2-longitude-crossings-design`). SP-2a shipped the
`pleiades-events` crate, the `CrossingEngine`, and a fail-closed
`validate-crossings` gate — but the gate landed with **loose per-body time
ceilings flagged "CONTROLLER-CALIBRATED pending maintainer review"** in three
code comments and the `PLAN.md` status line, and with spec §7's corpus
checksum-drift test intentionally omitted. This slice closes both.

## Context

The landed gate (`crates/pleiades-validate/src/crossings_validation.rs`)
recomputes 41 Swiss Ephemeris reference crossings with the packaged backend and
compares **crossing times** against per-frame/body ceilings:
`GEO_SUN_MOON_TOL_S = 60`, `GEO_PLANET_TOL_S = 2800`, `HELIO_TOL_S = 7200`
(seconds). Two problems, both verified in code:

1. **The ceilings are loose placeholders, not teeth.** The SE reference corpus is
   generated with Swiss Ephemeris *Moshier* theory
   (`tools/se-crossings-reference`, `SEFLG_MOSEPH`), while `packaged_backend()`
   uses VSOP87 (planets) + compact ELP/Meeus (Moon). That cross-theory difference
   — **not** engine error — dominates the residual and forces the ceilings up.
   The comparison metric makes it worse: a crossing time near a retrograde
   station (where dλ/dt → 0) amplifies a few-arcsecond longitude disagreement
   into ~2000 s of time disagreement (Mars measured 1949 s). The gate as shipped
   only catches gross (~1000×) regressions; a subtle drift under the ceiling
   passes silently.

2. **Spec §7's checksum-drift test is missing.** The manifest records a SHA-256,
   but the workspace has no `sha2` dependency, so the test was deferred with a
   comment. Meanwhile the sibling gates (`validate-lilith`, `validate-ayanamsa`,
   `validate-houses`) already verify their corpora with the in-tree
   `pleiades_apparent::fnv1a64` helper — which `crossings_validation.rs`'s own
   crate already imports for the lilith gate.

3. **Coverage gap.** The corpus exercises only **Mars** geocentrically and
   **Jupiter/Saturn** heliocentrically, though the engine claims general
   geocentric bodies and heliocentric planets Mercury–Pluto.

The fix reuses patterns already in the repo: the golden-snapshot idea generalizes
the position gates' approach, the fnv1a64 manifest check mirrors
`lilith_validation.rs` exactly, and corpus regeneration mirrors the
`generate-packaged-artifact` regenerate/`--check` command.

**Key enabling fact (verified):** the `tools/se-crossings-reference` binary
**runs in the dev environment** — SE Moshier needs no external ephemeris data
files — so both regenerating and *expanding* the SE corpus are in-repo steps, not
maintainer-offline steps.

## Decisions captured

| Topic | Decision |
| --- | --- |
| Gate structure | **Two independent fail-closed tiers** per corpus row: Tier 1 self-consistency (tight), Tier 2 SE parity (arcsecond). Replaces the single amplified time-ceiling check. |
| Tier 1 metric | `|recomputed_crossing − committed pleiades golden|`, one **sub-second** ceiling (`SELF_CONSISTENCY_TOL_S`, ≈ root-finder tolerance) for **every** body/frame. The regression teeth. |
| Tier 2 metric | Engine longitude evaluated **at the SE crossing time** vs the target longitude: `|wrap180(λ_engine(t_SE) − target)|` in **arcseconds**. Unamplified; comparable to `validate-lilith` (~306″) and the apparent gates. Per-body ceilings at ~1.4× measured. |
| Old ceilings | `GEO_SUN_MOON_TOL_S` / `GEO_PLANET_TOL_S` / `HELIO_TOL_S` (time seconds) **deleted**. |
| Golden column | New `pleiades_jd_tdb` column in `crossings.csv`, committed once from a known-good engine state, regenerated only by a deliberate `crossings-golden --regenerate` CLI action (mirrors `generate-packaged-artifact`). Not recomputed silently at gate time — that is what makes drift detectable. |
| Corpus coverage | **Expanded** to geocentric Mercury–Pluto and heliocentric Mercury–Pluto (plus Sun/Moon geo, and the retained Mars retrograde triple-crossing fixtures). ~3 fixtures/body/frame; corpus grows 41 → ~80 rows. |
| Coverage-boundary honesty | Any body the engine cannot hold to a defensible arcsecond ceiling (candidate: **Pluto**, bounded by its documented backend fallback) gets a ceiling consistent with its existing body-claim tier and is **documented as a coverage boundary** — never silently excluded. |
| New public API | `CrossingEngine::longitude_at(body, frame, instant) -> Result<Longitude, EventError>`, wrapping the private `longitude_deg` with the same window + heliocentric-Sun/Moon guards. Additive surface; drives the compat-profile bump. |
| Checksum-drift (spec §7) | Manifest gains a parseable `checksum=<fnv1a64 decimal>` line (replacing `sha256(...)`); the gate `include_str!`s the manifest, parses it, and compares `fnv1a64(CORPUS_CSV)` — mirroring `lilith_validation.rs`. No new dependency. |
| Engine behavior | **Unchanged.** No algorithm, root-finder, or convention change. |
| Versioning | New public `longitude_at` → compatibility profile `0.7.5 → 0.7.6`; API-stability profile `0.2.1` unaffected (purely additive). |

## Scope & boundaries

**In:**

- Two-tier `validate-crossings` (Tier 1 self-consistency, Tier 2 arcsecond SE
  parity); deletion of the old time-second ceilings.
- New `pleiades_jd_tdb` golden column + `crossings-golden --check/--regenerate`
  CLI command.
- Corpus expansion to geocentric + heliocentric Mercury–Pluto via an extended
  `tools/se-crossings-reference`.
- `CrossingEngine::longitude_at` public method.
- fnv1a64 manifest checksum-drift check + test (closes spec §7).
- Compatibility-profile / README / PLAN status updates; design-doc Open item #1
  marked resolved.

**Out (explicitly):**

- Any engine algorithm, root-finder, or longitude-convention change.
- Migrating the eclipse crate onto the shared root-finder.
- SP-2b (rise/set/transit) and SP-2c (local eclipse circumstances).
- `swe_mooncross_node`.
- Sidereal-target variants (unchanged from SP-2a: callers convert the target).

## Architecture

Nothing new structurally; the changes are localized to four places.

```
tools/se-crossings-reference/src/main.rs   (extend body set → regenerate SE corpus)
        │  SE (Moshier) reference times
        ▼
crates/pleiades-validate/data/crossings-corpus/
        crossings.csv     (+ pleiades_jd_tdb golden column, ~80 rows)
        manifest.txt      (checksum=<fnv1a64>)
        │
        ▼
crates/pleiades-validate/src/crossings_validation.rs
        ├─ Tier 1: |recomputed − golden|        ≤ SELF_CONSISTENCY_TOL_S   (all bodies)
        ├─ Tier 2: |λ_engine(t_SE) − target|    ≤ per-body arcsec ceiling
        └─ checksum: fnv1a64(CORPUS_CSV) == manifest checksum
        │
        ▼ (uses)
crates/pleiades-events/src/crossings.rs
        └─ pub fn longitude_at(body, frame, instant) -> Result<Longitude, EventError>
```

### Component: `CrossingEngine::longitude_at`

```rust
/// Ecliptic longitude of `body` in `frame` at `instant` (TDB).
///
/// Geocentric-apparent-of-date for `GeocentricApparentOfDate`; heliocentric of
/// date for `Heliocentric`. Fails closed out of the packaged window and for
/// heliocentric Sun/Moon, matching the crossing entry points.
pub fn longitude_at(
    &self,
    body: CelestialBody,
    frame: CrossingFrame,
    instant: Instant,
) -> Result<Longitude, EventError>
```

- Wraps the existing private `longitude_deg`.
- Applies `check_window` and the heliocentric Sun/Moon guard so its failure modes
  match `longitude_crossings_in_range` (no NaN/placeholder emitted).
- Returns `Longitude` (not raw `f64`) for API consistency; the gate reads
  `.degrees()`.
- Has a doctest (the crate `#![deny(missing_docs)]`s and doctests heavily).

### Component: two-tier gate

`validate_crossings_corpus()` keeps its parse loop but, per row:

1. Parse the now-7-field row (`frame,body,target,start,direction,se_jd,pleiades_golden_jd`).
   The `direction != "fwd"` and field-count schema guards remain (field count
   6 → 7).
2. Recompute the crossing with `next_longitude_crossing` → `got`.
3. **Tier 1:** `residual_self = |got − pleiades_golden_jd| · 86400` s; fail if
   `> SELF_CONSISTENCY_TOL_S`.
4. **Tier 2:** `lambda = engine.longitude_at(body, frame, Instant(se_jd))`;
   `residual_arcsec = |wrap180(lambda − target)| · 3600`; fail if
   `> arcsec_ceiling_for(frame, body)`.
5. Track per-tier maxima for the report.

`ceiling_for` is replaced by `arcsec_ceiling_for(frame, body)` returning
documented per-body arcsecond ceilings. `SELF_CONSISTENCY_TOL_S` is a single
module constant.

### Component: golden regeneration

A `crossings-golden` render command (in `pleiades-validate`'s CLI render layer,
alongside `generate-packaged-artifact`):

- `crossings-golden --check` — recompute every row's crossing with the engine and
  assert it matches the committed `pleiades_jd_tdb` within `SELF_CONSISTENCY_TOL_S`
  (a lint that the committed golden is current); non-zero exit on drift.
- `crossings-golden --regenerate` (or `--out FILE`) — rewrite `crossings.csv` with
  freshly computed golden values and print the new fnv1a64 for the manifest.

Regeneration is a deliberate maintainer action taken **only** when the engine is
intentionally changed, after Tier 2 has re-vouched for correctness.

### Component: manifest checksum (spec §7)

Mirror `lilith_validation.rs`:

- Manifest line `checksum=<fnv1a64 decimal>` (replacing `sha256(crossings.csv): …`);
  keep the human-readable header lines.
- `include_str!` the manifest; parse the `checksum=` token; compare against
  `fnv1a64(CORPUS_CSV)`; new `CrossingsCorpusError::ChecksumMismatch { got, want }`.
- Delete the "intentionally omitted" comment block.

## Data / corpus

`crossings.csv` schema (header + comment lines retained, count updated):

```
frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb,pleiades_jd_tdb
```

- `crossing_jd_tdb` — SE (Moshier) reference time (unchanged meaning).
- `pleiades_jd_tdb` — engine golden time (new).

Rows: Sun geo, Moon geo, and **Mercury–Pluto** in **both** geo and helio frames,
~3 fixtures/body/frame (retaining Mars's retrograde triple-crossing fixtures),
~80 rows total.

`tools/se-crossings-reference/src/main.rs` extends its SE ipl body constants
(add Mercury=2, Venus=3, Uranus=7, Neptune=8, Pluto=9) and emits geo (bisection
on `swe_calc`) and helio (`swe_helio_cross`) fixtures for the full set.

`manifest.txt`: `rows:` updated to the new count; `checksum=<fnv1a64>` added;
`source`/`generator`/`frames`/`window` lines retained.

## Error handling / fail-closed conditions

- Malformed row / wrong field count / non-`fwd` direction → `Schema` (unchanged
  policy; field count now 7).
- Engine cannot reproduce a fixture SE reports → `Missing`.
- Tier-1 self-consistency residual over ceiling → `ToleranceExceeded` (self tier).
- Tier-2 arcsecond residual over ceiling → `ToleranceExceeded` (parity tier);
  the error carries which tier and the numeric residual/ceiling.
- `fnv1a64(CORPUS_CSV)` ≠ manifest checksum → `ChecksumMismatch`.
- `longitude_at` out-of-window / heliocentric Sun-Moon → `EventError`, surfaced as
  `Engine`.

Every path returns `Err` immediately; no NaN or placeholder is ever accepted.

## Testing strategy

- **Keep:** `validate_crossings_passes_over_committed_corpus` — assert the new row
  count (~80) instead of 41.
- **Add:** `manifest_checksum_matches` — `fnv1a64(CORPUS_CSV)` equals the manifest
  value (drift guard; closes spec §7).
- **Add:** Tier-1 catches injected golden drift — a crafted row whose
  `pleiades_jd_tdb` is perturbed beyond `SELF_CONSISTENCY_TOL_S` returns
  `ToleranceExceeded`.
- **Add:** Tier-2 catches injected longitude drift — a crafted target offset
  beyond the arcsec ceiling returns `ToleranceExceeded`.
- **Add:** `longitude_at` doctest + a unit test (matches the longitude implied by
  a known crossing; fails closed for heliocentric Sun/Moon and out-of-window).
- **Add:** `crossings-golden --check` passes over the committed corpus (the golden
  is current).
- **Regression:** `validate-eclipses` and `validate-angles` unchanged and green
  (proves the events/validate changes did not perturb neighboring gates).
- **Full:** `cargo test --workspace` and `validate release-gate` green.

## Versioning & bookkeeping

- Compatibility profile `0.7.5 → 0.7.6`; the SP-2a capability entry rewritten to
  state the resolved two-tier evidence and the **actual** body coverage (geo +
  helio Mercury–Pluto), removing the "placeholder ceilings pending maintainer
  review" caveat. Overclaim audit (`compat-claims-audit`) stays green.
- API-stability profile unaffected (`0.2.1`) — `longitude_at` is additive.
- README "current state" crossings sentence → two-tier gate + full body coverage.
- `PLAN.md` status → replace the placeholder-ceiling wording with the resolved
  description.
- SP-2a design-doc Open item #1 (per-body time ceilings) marked **resolved by
  SP-2a-FU**.

## Open items (confirm during implementation)

1. **Per-body arcsecond ceilings** — measured during implementation from the
   regenerated corpus; set at ~1.4× each body-class group max, documented against
   the `validate-lilith` ~306″ precedent. Pluto's ceiling reflects its backend
   fallback.
2. **`SELF_CONSISTENCY_TOL_S` value** — pin to the root-finder's own tolerance
   (sub-second; confirm the exact bisection tolerance in `root.rs` and set the
   ceiling a small factor above it).
3. **Fixture epochs/targets per new body** — choose starts/targets in-window that
   yield a clean forward crossing for each body in each frame (outer planets move
   slowly; pick spans that guarantee a crossing), mirroring the existing fixture
   density.
4. **Golden regeneration surface** — confirm the exact CLI dispatch point in the
   render layer and whether `--out`/`--check` flag names should match
   `generate-packaged-artifact` verbatim.
