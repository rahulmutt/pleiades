# Asteroid event gates: Swiss Ephemeris reference rows for stations, aspects and osculating nodes/apsides

Issues: #167 (d) asteroid stations, #168 (f) asteroid aspects, #160 asteroid
`nod_aps` rows. Status: design approved 2026-10-10.

## Goal

Since #233 the packaged backend serves Ceres, Pallas, Juno, Vesta and
asteroid:433-Eros densely over 1900–2100. Their positions are gated against
`asteroid_reference.csv` (sb441-n373s), but nothing gates what is derived from
them: station instants (which depend on the packaged *speeds*, the subject of
#158), exact-aspect instants, and osculating nodes and apsides. The event
finders accept asteroids and return answers no reference checks.

This work adds Swiss Ephemeris reference rows for those three event families
and gates them, so a regression in the packaged asteroid fits, their speeds,
or the finders' handling of slow, loop-making bodies fails a gate.

## Decisions

| Question | Decision | Why |
|---|---|---|
| Reference authority | Swiss Ephemeris, `SEFLG_SWIEPH` with `seas_18.se1` | Same authority as every other event gate; #160 names SE rows as the intended fix. Pleiades' own finders on JPL kernels would gate the ephemeris, not the finders, and break the convention. |
| Bodies | Ceres, Pallas, Juno, Vesta | Covered by `seas_18.se1`. Eros needs SE's separate per-asteroid file; it stays ungated for events (positions remain gated against sb441). |
| Aspect pairs | Ceres–Sun, Pallas–Sun, Juno–Sun, Vesta–Sun, Ceres–Moon | Sun pairs cover conjunction/opposition and the retrograde loops; Ceres–Moon adds a fast partner that stresses the scan step. |
| Structure | Separate `asteroids.csv` per corpus, read by the existing gates | Planet corpora and checksums cannot drift; asteroid rows inherit each gate's tiering; one gate per event family. |

Rejected: appending asteroid rows to the existing CSVs (a regeneration would mix
Moshier and SWIEPH rows and could move gated planet rows), and a standalone
`validate-asteroid-events` gate (duplicates three gates' comparison and
tiering code).

## 1. Swiss Ephemeris data files

- `seas_18.se1` (asteroids, 1800–2400 CE) and `sepl_18.se1` (planets; SE needs
  the Earth for a geocentric asteroid) from
  `https://raw.githubusercontent.com/aloistr/swisseph/master/ephe/`, the source
  `tools/se-nodaps-reference` already pins `sepl_18.se1`/`semo_18.se1` from.
- Each tool that needs them pins their SHA-256 as constants and verifies them
  fail-closed before writing any asteroid row, using the nod-aps tool's existing
  verifier pattern (`verify_swieph_files`). The tools are standalone crates
  (each has its own `[workspace]`), so each carries its own copy of the pins.
- The files are not committed: each tool's `data/` directory is gitignored, as
  `tools/se-nodaps-reference/.gitignore` does. The ephe directory resolves as in
  the nod-aps tool: `$SE_EPHE_PATH`, else `--ephe`, else `<tool>/data`.

## 2. Spike: SE versus sb441 (step 0, before any corpus)

A throwaway probe calls `swe_calc(SE_CERES..SE_VESTA, SEFLG_SWIEPH|SEFLG_SPEED|
SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL|SEFLG_NONUT|SEFLG_J2000)`, the
geometric J2000 geocentric place `asteroid_reference.csv` holds, at its 407
epochs per body. It reports, per body, the maximum disagreement with the sb441
rows in longitude × cos β, latitude, and (against a central difference of the
rows' neighbours) longitude speed.

- The result is recorded in
  `docs/superpowers/specs/notes/2026-10-10-se-asteroid-agreement.md`.
- Gate ceilings are never set below this floor converted to the gated quantity:
  an angle for the aspects separation and longitude ceilings and for nod-aps; a
  time (the position floor ÷ the relative speed near the event) for the aspects
  time ceiling; a time (the speed floor ÷ the longitude acceleration at
  station) for the stations time ceiling.
- **Stop condition:** if SE disagrees with sb441 by more than 5″ in position
  for any of the four bodies, work stops and the numbers go back to the
  maintainer before any corpus is built, because the ceilings would then mostly
  measure SE-versus-JPL disagreement.

## 3. Generators: an `--asteroids` mode per tool

Each tool gains an `--asteroids` flag that writes only the asteroid CSV and its
manifest (`rows=` and an fnv1a64 checksum, the format the planet manifests use).
The planet code paths, flags and output are unchanged.

### Stations (`tools/se-stations-reference`)

- Series `geo,<Ceres|Pallas|Juno|Vesta>`: the planet `geo` group's flags
  (apparent, tropical, true equinox of date) with `SEFLG_SWIEPH` in place of
  `SEFLG_MOSEPH`.
- `FULL_SPAN` (JD 2415025.5–2488064.5), the 0.25-day `PLANET_GRID_DAYS` grid,
  and the existing 1e-7-day bisection.
- Output: `crates/pleiades-validate/data/stations-corpus/asteroids.csv` and
  `asteroids-manifest.txt`, columns `group,body,jd_tt,lon_deg,kind` (the
  planet file's).

### Aspects (`tools/se-aspects-reference`)

- `geo` pairs Ceres–Sun, Pallas–Sun, Juno–Sun, Vesta–Sun over `FULL_SPAN`;
  Ceres–Moon over `SHORT_SPAN` (JD 2447892.5–2462502.5) with the Moon grid,
  as Sun–Moon uses.
- Angles 0, 60, 90, 120 and 180°; the existing refusal of a turning point
  within 30″ of an angle level applies unchanged.
- Output: `crates/pleiades-validate/data/aspects-corpus/asteroids.csv` and
  `asteroids-manifest.txt`, the planet file's columns.

### Nod-aps (`tools/se-nodaps-reference`)

- Osculating rows (`SE_NODBIT_OSCU`, `SEFLG_SWIEPH|SEFLG_SPEED|SEFLG_NOGDEFL`)
  for SE ids 17–20 at the planets' 8 `EPOCHS`: 32 rows.
- No mean rows (SE has no mean elements for asteroids, and
  `EventEngine::nod_aps` returns `UnsupportedNodAps` for them) and no
  barycentric rows (the four bodies stay inside ~6 AU).
- Output: `crates/pleiades-validate/data/nod-aps-corpus/asteroids.csv` and
  `asteroids-manifest.txt`, the planet file's 29 columns.

## 4. Gates

Each gate reads `asteroids.csv` beside its existing corpus and verifies it
against its manifest, exactly as the planet file is verified.

- **Body parsing.** `body_from_name` (stations, aspects) accepts Ceres, Pallas,
  Juno and Vesta; `body_from_se` (nod-aps) accepts ids 17–20. The nod-aps test
  that asserts ids 15 and 20 are rejected changes: 15 (Chiron) stays rejected,
  20 (Vesta) is accepted.
- **Ceilings.** Per asteroid (stations: time and longitude in
  `stations_thresholds.rs`), per pair (aspects: separation and longitude in
  `aspects_thresholds.rs`, and `PAIRS` grows from 11 to 16 entries), and a new
  `ASTEROID` category beside PLANET and MOON (nod-aps, `nod_aps_thresholds.rs`).
  Each ceiling is the measured maximum × 1.4, the #233 convention, never below
  the §2 floor, with a comment giving the measured maximum.
- **Row floors.** A separate asteroid floor per gate, so a truncated asteroid
  file fails on its own; the planet floors (`MIN_ROWS_VALIDATED` 5542 and
  10359, `EXPECTED_ROWS` 184) are unchanged. Nod-aps adds an asteroid constant
  of 32.
- **Engine.** Unchanged. Asteroids are already scanned (`never_stations` and
  `separation_bound` return "scan" for them). A defect the gate exposes becomes
  its own issue; ceilings are not widened around it.

## 5. Tiers

- Stations and aspects asteroid rows run in the full gates only: `mise run
  gate-stations` / `gate-aspects`, the nightly `stations-gate` and
  `aspects-gate` jobs, and so `release-gate`. The `release-smoke` subsets are
  unchanged.
- Nod-aps's 32 rows join its existing full run in the release battery.

## 6. Documentation

- Each tool's module docs and README (or `LICENSE-NOTES.md` where that is the
  tool's notes file): the `--asteroids` mode, the data-file download and SHA-256
  pin step, and the regeneration command.
- The gates' module docs and summary lines: asteroid rows, their ceilings and
  their source.
- A short "asteroid event gating" paragraph in the relevant `docs/` validation
  page: which asteroids are gated for events, against what, and that Eros is
  not (no pinned SE file), its positions being gated against sb441.

## 7. Delivery

Three PRs, in order, each closing its item:

1. **Stations** (#167 (d)): this spec, the plan, the §2 spike note, the data-file
   pinning in the stations tool, the stations corpus and gate.
2. **Aspects** (#168 (f)): pinning in the aspects tool, the aspects corpus and
   gate.
3. **Nod-aps** (#160, asteroid subset; fictitious bodies stay documented as
   unreferenced, since SE's `swe_nod_aps` does not implement them).

After each merge, the issue gets a comment with the measured maxima.

## 8. Validation per PR

- `mise run ci` and `mise run test-full`.
- The relevant full gate: `mise run gate-stations`, `mise run gate-aspects`, or
  `cargo run -p pleiades-validate -- validate-nod-aps`.
- The planet corpora and their manifests are byte-unchanged
  (`git diff --exit-code` on the existing CSVs and manifests).
- The asteroid CSV regenerates byte-identically from the pinned data files
  (two runs, `cmp`).
- Gate unit tests: an asteroid row parses; a truncated asteroid file fails its
  floor; a row moved past its ceiling fails; the manifest checksum catches an
  edited row.
