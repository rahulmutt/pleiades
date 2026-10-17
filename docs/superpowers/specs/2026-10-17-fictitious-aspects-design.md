# Fictitious-body aspects in the mean-of-date frame (design)

**Status:** design approved, spec under review · **Opened:** 2026-10-17 ·
**Issue:** #168 (f), fictitious bodies ·
**Crates/tools:** `tools/se-aspects-reference`, `pleiades-validate`, docs

## Context

`validate-aspects` and `validate-aspects-lunar` gate exact aspects of the
planets, asteroids and lunar points against Swiss Ephemeris (#251, #256,
#269, #272). The 19 fictitious bodies (SE 40–58, `seorbel.txt`, served by
`pleiades-fict`) are accepted by the aspect finders but are not gated.
`validate-fictitious` holds their positions within 1.26″ of Swiss
Ephemeris, but only geometric J2000, in 570 rows.

## Measurements (spike, 2026-10-17)

The spike was throwaway. It compared:
- **SE side:** SWIEPH with the pinned `sepl_18`/`semo_18`, and the repo's
  `tools/se-fictitious-reference/data/seorbel.txt`;
- **engine side:** `RoutingBackend[PackagedDataBackend,
  FictitiousBackend(PackagedDataBackend)]`, the CLI chart's composition.

It ran every 5 days over 1900–2100, 14,608 samples per body.

**Apparent of date (SE default flags): the reference is wrong.** The engine
agrees with Swiss Ephemeris at 0.2–0.5″ rms, but most bodies show isolated
4–5″ spikes. Zeus has 23 samples above 1.5″, about 0.16%.
- **The spikes are in SE's series:** its third differences show the
  [1, −3, 3, −1] signature of a single-sample jump of about 4″. The engine's
  series is smooth at the same instants (median max |Δ³| of 2.7″ for both).
- **They are not in the ephemeris:** they are identical with SWIEPH or
  MOSEPH, with or without `SEFLG_SPEED`, and with `SEFLG_NOABERR`.
- **They are in SE's light-time correction for fictitious bodies:** they
  vanish with `SEFLG_TRUEPOS`.
- **The packaged Sun is clean** against SE: 0.17″ rms, 0.33″ max, no spikes.

**Mean of date** (`SEFLG_TRUEPOS|NOABERR|NOGDEFL|NONUT`, the engine's
`GeocentricMeanOfDate`, already the planet corpus's `mean` group): there are
no spikes.

| Body | rms | max |
|---|---|---|
| Cupido–Poseidon (SE 40–47) | 0.343″ | 0.607″ |
| Transpluto | 0.240″ | 0.470″ |
| Nibiru | 1.272″ | 1.570″ (steady +1.26″ offset, matching `validate-fictitious`'s 1.262″) |
| Harrington | 0.173″ | 0.306″ |
| Neptune Leverrier / Adams | 0.480″ | 0.763″ |
| Pluto Lowell / Pickering | 0.27″ | 0.52″ |
| Vulcan | 0.174″ | 0.354″ |
| White Moon | 0.000″ | 0.000″ |
| Proserpina | 0.002″ | 0.004″ |
| Waldemath | 0.348″ | 0.606″ |

The maxima fall near 2100: the difference drifts slowly.

**Turning, SE side, mean frame, coarse 5-day grid:**
- **The Sun pairs:** the separation from the Sun never turns, except for
  Vulcan, which lies inside Mercury's orbit and has about 7,460 turns. There
  are about 1,595 events per pair, and about 7,460 for Vulcan–Sun.
- **The Saturn pairs:** each Uranian body and Transpluto has 386 turns with
  Saturn and about 107–126 events.
- **Closest turn to a level** on the coarse grid: Poseidon about 3″,
  Transpluto 22″, Admetos 23″, Zeus 82″, the rest 120″ or more. The Jupiter
  pairs behave alike (366 turns, 80″ and up), so they would add little.

## Goal

Gate, event for event against SWIEPH in the mean-of-date frame, in a new
`fictitious.csv` with graze windows in `fictitious-grazes.csv`:
- each of the 19 fictitious bodies with the Sun (19 pairs);
- Cupido, Hades, Zeus, Kronos, Apollon, Admetos, Vulkanus, Poseidon and
  Transpluto with Saturn (9 pairs).

That is 28 pairs over JD 2415025.5–2488064.5, at angles 0/60/90/120/180.

Non-goals:

- **The apparent frame for fictitious bodies.** It stays ungated, because the
  reference is defective there. The defect is documented on #168 with the
  numbers above.
- **Fictitious-body stations** (#167 (d)), the other fictitious pairs, and
  the lunar perigees.
- **Any engine change.**

## Reference tool (`tools/se-aspects-reference --fictitious`)

- **Flags:** a new `MEAN_SWIEPH = MEAN | SEFLG_SWIEPH | SEFLG_SPEED`,
  following the existing `MEAN` constant, which is Moshier, of the planet
  `mean` group. SWIEPH keeps the Sun and Saturn on the same files as the
  other SWIEPH corpora.
- **Files:**
  - the pinned `sepl_18` and `semo_18`, verified by `pins::verify_swieph_files`;
  - `seorbel.txt`, which SE reads from the ephemeris path. It gets its own
    SHA-256 pin, of the committed copy at
    `tools/se-fictitious-reference/data/seorbel.txt`, and is verified
    fail-closed the same way. The run's ephemeris directory must hold all
    three; the tool's error says so.
- **Bodies:** the 19 SE numbers 40–58, written with the engine's
  `CelestialBody` names: Cupido, Hades, Zeus, Kronos, Apollon, Admetos,
  Vulkanus, Poseidon, Transpluto, Nibiru, Harrington, NeptuneLeverrier,
  NeptuneAdams, PlutoLowell, PlutoPickering, Vulcan, WhiteMoon, Proserpina,
  Waldemath.
- **Grid:** 0.05 d for every pair, except 0.01 d for Vulcan–Sun. The plan
  confirms that halving each grid changes no event count, the way #253
  checked the asteroid grid.
- **Graze windows:**
  - the `Graze::Window` mechanism of #272, unchanged;
  - margin 3″ for every pair, which is ≥ 1.9 × Nibiru's 1.57″ and
    ≥ 3.9 × everyone else's 0.76″ engine-to-SE difference;
  - output: `fictitious.csv`, `fictitious-manifest.txt`,
    `fictitious-grazes.csv` and `fictitious-grazes-manifest.txt`, all in
    `crates/pleiades-validate/data/aspects-corpus/`;
  - corpus group: `mean`.
- **Unchanged:** the planet, asteroid and lunar modes. The plan regenerates
  `aspects.csv`, `asteroids.csv` and `lunar-points.csv` (the last from the
  committed SE files, about 65 minutes, run by the controller) and `cmp`s
  each against the committed copy.

## Gate (`crates/pleiades-validate`, `Scope::Fictitious`)

- **Data:** the four files via `include_str!`, checked like the lunar ones
  (checksum, rows, window consistency).
- **Plan:** `FICTITIOUS_PAIRS: [Pair; 28]` in the tool's order, all
  `Group::Mean`, `FULL_SPAN`. `body_from_name` gains the 19 names.
- **Engine:** `validate_scoped` today always builds
  `EventEngine::new(packaged_backend())`, which does not serve fictitious
  bodies. The scope decides the backend instead: `Scope::Fictitious` uses
  `RoutingBackend::new(vec![packaged, FictitiousBackend::new(packaged)])`,
  the same composition as `pleiades-cli`'s chart. The other scopes are
  unchanged. `pleiades-validate` already depends on `pleiades-fict`
  (`fictitious_validation.rs`).
- **Windows:** the existing `split_windows` and parity rule.
- **Ceilings:** ×1.4 of the measured outside-window maximum, rounded up to
  two significant figures, floored at 2.3″. Nibiru may land at about 3″. A
  pair above 10″ outside windows is a finding, not a ceiling.
- **Floor:** `MIN_ROWS_VALIDATED_FICTITIOUS` = the corpus rows outside
  windows.
- **Entry point and tier:**
  - `validate_fictitious_aspects_corpus()`;
  - the subcommand `validate-aspects-fictitious` (alias
    `aspects-fictitious-gate`);
  - the mise task `gate-aspects-fictitious`.
  - Where it runs is decided by measurement. If the planet-and-asteroid gate
    plus the fictitious pass, measured by a nightly run dispatched on the
    branch, stays under 15 minutes, the fictitious pass joins
    `validate-aspects`' job: `validate_aspects_corpus` merges it and the
    subcommand stays as a standalone entry. Otherwise it gets its own nightly
    job `aspects-fictitious-gate` and a `release-gate` dependency, like the
    lunar job. The plan dispatches the run, reads the gate step's timestamps,
    and records the figure.
- **Blocking tier:** `validate_aspects_corpus_subset()` gains the
  fictitious manifest, the grazes manifest and the window-consistency check,
  with no scan.
- **Tests:** the pair-plan pin, the tamper and drift tests for both files, the
  floor, the window-consistency test on the committed data, and an opt-in
  full-gate test like the lunar one.

## Documentation

- `crates/pleiades-events/README.md`: fictitious bodies are gated in the
  mean-of-date frame for the 28 pairs. The apparent frame is not gated,
  because of the SE light-time defect.
- `AGENTS.md`, `docs/development.md`, `docs/status.md`, `mise.toml`, the CLI
  help and the gate-failure issue template, wherever they list gated pairs.
- **#168:**
  - (f): fictitious bodies gated in the mean frame;
  - the finding on SE's fictitious light-time spikes.
- No compatibility-profile change.

## Testing summary

- **Tool:** the three existing corpora regenerate byte-identically, and
  halving the grid leaves every fictitious pair's counts unchanged.
- **Gate:**
  - the fictitious pass passes;
  - the Sun-pair counts are within the spike's coarse estimates;
  - the blocking-tier checks and tamper tests pass.
- `mise run test-full`, `mise run audit` and `mise run docs`, and the
  nightly dispatch on the branch for the runtime figure.
