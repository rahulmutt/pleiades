# True-node aspect corpus against Swiss Ephemeris SWIEPH (design)

**Status:** design approved, spec under review · **Opened:** 2026-10-16 ·
**Issue:** #168 (f), the lunar-point half, and part of (c), near-grazes ·
**Crates/tools:** `tools/se-aspects-reference`, `pleiades-validate`, docs

## Context

`validate-aspects` gates exact aspects event for event against Swiss
Ephemeris:
- 11 planet pairs from Moshier, in `aspects.csv`;
- 7 asteroid pairs from SWIEPH `seas_18`, in `asteroids.csv` (#251, #256).

The lunar points are accepted by the aspect finders but not gated. The events
README says "Aspects of the lunar points and fictitious bodies are found but
not gated".

#168 (c) notes that no corpus pair has a graze. The tool refuses any pair
whose separation turns within 30″ of an angle, so near-tangent events, whose
existence depends on the ephemeris, are never compared. The closest turn in
the corpus is 102″.

## Measurements (spike, 2026-10-16)

The spike was throwaway and nothing from it is kept. It used the pinned
`sepl_18`/`semo_18`/`seas_18` files (SHA-256 verified), an SE grid of 0.005 d,
and the pleiades engine on the packaged backend at its default step. It covered
JD 2415025.5–2488064.5, the angles 0/60/90/120/180, and the geocentric apparent
ecliptic of date.

**Point residuals.** Engine minus SE SWIEPH, daily over 1900–2100:

| Point | Max | p99 | Non-finite SE speeds |
|---|---|---|---|
| TrueNode (SE body 11) | 0.94″ | 0.73″ | 0 |
| TrueApogee (SE body 13) | 85.6″ | 79.9″ | 0 |

**True node against each planet:**

| Pair | Events | SE turning points | Closest turn to a level | Engine vs SE counts | Max separation residual | Max \|Δt\| |
|---|---|---|---|---|---|---|
| TrueNode–Sun | 1686 | 0 | n/a | equal | 0.53″ | 13 s |
| TrueNode–Mars | 990 | 354 | 810″ | equal | 0.86″ | 107 s |
| TrueNode–Jupiter | 247 | 2392 | **3.6″** | equal | 0.60″ | 297 s |
| TrueNode–Saturn | 166 | 3472 | 54″ | equal | 0.64″ | 540 s |
| TrueNode–Uranus | 119 | 4786 | 26.9″ | equal | 0.51″ | 507 s |
| TrueNode–Neptune | 109 | 5778 | 17.4″ | equal | 0.72″ | 1033 s |
| TrueNode–Pluto | 107 | 6600 | **4.1″** | equal | 0.53″ | 930 s |

All 3424 events match one to one. At SE's turning points the
engine-minus-SE separation is at most 0.82″, so even the 3.6″ near-graze is
the same event on both sides. The longitude residual of either body at the
matched instants is at most 0.95″. No two events of one angle are closer
than 1.1 d.

**The osculating apogee is not gateable now.** Its point residual against
SWIEPH reaches 86″, with a cycle of about 27.5 d. That is a model difference,
not noise. Of about 11000 events per pair, 16 exist in only one ephemeris
(6 SE-only, 10 engine-only), each beside a turning point 1.8–46.5″ from a
level. One pair's equal count hides a cancelling SE-only and engine-only
pair. Gating it would need about 120″ ceilings and an optional-event rule, a
weak check for the cost, and its residual needs investigating first.

## Goal

Add the seven true-node pairs to `validate-aspects` as a third corpus,
`lunar-points.csv`. They are compared event for event against SWIEPH, with
2.3″ ceilings, a graze margin that admits the measured near-grazes, and the
same manifest discipline as the asteroid corpus.

Non-goals:

- **The osculating apogee.** The 86″ disagreement is reported on #167/#168 for
  a separate investigation, with the numbers above.
- **Mean lunar points, fictitious bodies, and the sidereal and mean-of-date
  frames.**
- **Any engine change.** The engine already agrees event for event.
- **#168 (a).** The node pairs never put two events of one angle inside one
  step, so this corpus does not exercise the turning-point split.

## Reference tool (`tools/se-aspects-reference`)

**A new mode, `--lunar-points --out <dir> [--ephe <dir>]`,** next to
`--asteroids` and built the same way:

- flags `GEO_SWIEPH` (`SEFLG_SWIEPH|SEFLG_SPEED`, apparent geocentric of date);
- the existing fail-closed `verify_swieph_files`, extended to the `sepl_18`
  and `semo_18` pins it already lists;
- body `(11, "TrueNode")`;
- `LUNAR_POINT_PAIRS`: TrueNode with the Sun, Mars, Jupiter, Saturn, Uranus,
  Neptune and Pluto, each over `FULL_SPAN`, on a 0.005 d grid;
- it writes `lunar-points.csv` and `lunar-points-manifest.txt`
  (`slice aspects-lunar-points file=lunar-points.csv role=aspects rows=N checksum=<fnv1a64>`),
  with the same CSV schema as the other two corpora and `group=geo`.

**Graze margin by group.** `GRAZE_MARGIN_DEG` (30″) becomes a parameter of the
scan. The planet and asteroid modes keep 30″, so their output stays
byte-identical, and the lunar-point mode uses **3″**:

- 3″ is 3.6× the measured 0.82″ engine-minus-SE separation at turning points;
- it is below the closest measured turn (TrueNode–Jupiter, 3.6″), so it
  refuses nothing today;
- a future pair that turns within 3″ is still refused, by the same assert.

Each turn's miss is logged on stderr, as now.

**Non-finite values.** SWIEPH returned no non-finite true-node longitude or
speed anywhere over the span, including the scan and bisection calls. So the
tool keeps its existing fail-closed assert in `state()`: a non-finite value
aborts the run. Moshier's isolated-NaN skip from `se-stations-reference` would
be dead code here.

**Building.** As for the other SE tools: `devenv shell -- env
CFLAGS=-std=gnu17 cargo build --release --offline --manifest-path
tools/se-aspects-reference/Cargo.toml`. In an environment where `devenv shell`
is unavailable, the spike built the same way with `LIBCLANG_PATH` pointing
at the nix-store libclang. The PR records the command used. `devenv.nix` is
unchanged.

## Gate (`crates/pleiades-validate`)

- **Corpus files:** `data/aspects-corpus/lunar-points.csv` and
  `lunar-points-manifest.txt`, both generated by the tool and committed.
- **`aspects_validation.rs`:**
  - `LUNAR_POINT_CSV` and `LUNAR_POINT_MANIFEST` constants, read with
    `include_str!` like the asteroid ones;
  - `LUNAR_POINT_PAIRS: [Pair; 7]`;
  - a `TrueNode` arm in `body_from_name`;
  - a `Scope::LunarPoints` variant with its floor;
  - `validate_aspects_corpus()` merges the lunar-point report after the
    asteroid one;
  - `validate_aspects_corpus_subset()`, the blocking tier (#254), adds
    `check_manifest(LUNAR_POINT_CSV, LUNAR_POINT_MANIFEST, &LUNAR_POINT_PAIRS)`.
- **`aspects_thresholds.rs`:**
  - `ceilings_for` arms for the seven `TrueNode-<planet>` names, all
    separation 2.3″ and longitude 2.3″. That is ×1.4 of the measured maxima
    (0.86″, 0.95″), floored at 2.3″, as for the asteroids.
  - `MIN_ROWS_VALIDATED_LUNAR_POINTS = 3424`, set to the regenerated count,
    which the plan checks equals the spike's 3424. Any difference is
    investigated, not accepted.
  - The ceiling comment records the measured maxima, per pair.
- **Engine and frame:** the same `EventEngine::new(packaged_backend())` and
  `Group::Geo` reference as the other corpora.
- **Tests:**
  - pin the new pair list;
  - the tampered-CSV and row-drift tests for the lunar manifest;
  - the floor;
  - the total-rows and pair-line counts, which become planets + asteroids +
    lunar points.

Follow the asteroid tests at `tests.rs` 479–625.

## Runtime and tiers

The full gate runs in the nightly `aspects-gate` job. The spike measured
71–123 s per pair on a loaded machine, running eleven in parallel. The plan
measures the full gate before and after on the same machine, and records the
figures in the `mise.toml` `gate-aspects` comment. The job's 80-minute step
timeout leaves room. If the addition takes the full gate past 15 minutes on
the nightly runner, the PR says so and the lunar pass gets its own
`Scope`-filtered subcommand, so it can be split out later. That is a
contingency, not a plan step.

The blocking tier gains only the manifest check (milliseconds).

## Documentation

- `crates/pleiades-events/README.md`: "Aspects of the lunar points …
  are found but not gated" becomes: the true node is gated against SWIEPH
  for seven pairs; the other lunar points and the fictitious bodies are not.
- The `aspects_validation.rs` module header, the `validate_aspects_corpus`
  doc, the `mise.toml` `gate-aspects` comment, `docs/status.md`,
  `docs/development.md` and the `AGENTS.md` CI-tier paragraph, wherever they
  list the gate's pairs.
- No compatibility-profile change. The asteroid aspect gate (#251) added a
  corpus without touching `pleiades-core`'s profile or notes, because a
  validation corpus is not part of the published API. This change follows it.
- **#168:**
  - (f) gains its lunar-point half: the true node is gated.
  - (c) gains the measured near-grazes, 3.6″ and 4.1″, now compared.
  - A finding on the osculating apogee's 86″ residual against SWIEPH.

## Testing summary

- **Tool:** the planet and asteroid outputs are unchanged by the
  graze-margin refactor. Check this by regenerating both and comparing their
  manifests' checksums with the committed ones.
- **Corpus:** the regenerated row counts equal the spike's per-pair counts.
- **Gate:**
  - the full `mise run gate-aspects` passes, with the lunar-point lines
    reporting at most the measured maxima;
  - the blocking-tier manifest check passes;
  - the tamper and drift tests pass.
- The full validate libtest, including the slow tier.
