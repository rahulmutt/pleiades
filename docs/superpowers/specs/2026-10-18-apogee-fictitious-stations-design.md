# Stations of the osculating apogee and the fictitious bodies (design)

**Status:** design approved, spec under review · **Opened:** 2026-10-18 ·
**Issue:** #167 (d) ·
**Crates/tools:** `tools/se-stations-reference`, `pleiades-validate`, docs

## Context

`validate-stations` gates the stations of the planets, the true node and
Ceres–Vesta against Swiss Ephemeris (#85, #198, #250). The osculating apogee
(`TrueApogee`) and the 19 fictitious bodies (SE 40–58, `seorbel.txt`, served
by `pleiades-fict`) are accepted by `EventEngine::stations_in_range` but have
no reference corpus. The apogee was blocked by the wrong Earth–Moon μ, which
#271 fixed (the apogee now matches SE to 1.8″).

The existing corpora take a station as a sign change of the speed that
`swe_calc(…|SEFLG_SPEED)` reports. This design takes it as a sign change of
the central difference of SE's longitude instead, for the reasons measured
below.

## Measurements (spike, 2026-10-18)

The spike was throwaway. It compared:
- **SE side:** SWIEPH with the pinned `sepl_18`/`semo_18` (and the repo's
  `seorbel.txt` for the fictitious bodies), 1900–2100 (JD 2415025.5–2488064.5);
- **engine side:** `stations_in_range` on `PackagedDataBackend` for the apogee
  (geocentric apparent), and on `RoutingBackend[PackagedDataBackend,
  FictitiousBackend(PackagedDataBackend)]` for the fictitious bodies
  (`GeocentricMeanOfDate`, the frame of the fictitious aspect gate, #273).

**The osculating apogee, against SE's reported speed (0.01-day grid).** SE
finds 4792 stations and the engine 4776. Every engine station matches an SE
station of the same kind: median |Δt| 3.2 s, 99th percentile 146 s, maximum
663 s, mean −0.9 s. The 16 SE stations without a match are 8 pairs about 17 s
apart. At JD 2424889.42 SE's apogee speed is +0.081°/day between samples of
−0.898 and −0.899°/day 0.0005 day either side, and its longitude is 0.7″ off
the line through its neighbours: an isolated discontinuity in SE, not a
station. The engine rightly reports none of the 16.

**The osculating apogee, against the central difference of SE's longitude**
(h = 0.001 day, grid offset 0.00137 day from the span start). SE finds 4786
stations; the engine still matches all of them (median 3.2 s, mean −1.5 s,
maximum 586 s at the 99.9th percentile). Nine SE stations remain unmatched:
four pairs exactly 2h = 0.002 day apart and one more, which is how the same
isolated jumps look to a central difference that steps over them at t ± h.

**A real apogee near-graze:** both methods find a speed extremum of
6e-5°/day (0.21″/day) at JD 2485441.43, the closest of the span. Whether a
station pair exists there can depend on the ephemeris.

**The fictitious bodies, against SE's reported speed (0.25-day grid).** Counts
and kinds match for all 19 (WhiteMoon and Waldemath never station; Vulcan has
7461). Fourteen bodies are within 13–26 s in time and 0.75″ in longitude, but
Nibiru reaches 195 s, Vulcan 209 s, Harrington 60 s and PlutoPickering 46 s,
and Proserpina's times differ by 4.3 h rms while its longitudes at the
stations agree to 0.013″. SE's own Proserpina speed root lies 4.4 h from the
root of the central difference of its own longitude (+15981 s turning direct,
−15185 s turning retrograde); for Zeus the same check gives 0.0 s.

**The fictitious bodies, against the central difference of SE's longitude**
(h = 0.01 day):

| Body | Stations | Δt max | Δt mean | lon max |
|---|---|---|---|---|
| Cupido, Hades, Zeus, Kronos, Apollon, Admetos, Vulkanus, Poseidon | 398–400 | 17–23 s | +7.0 to +7.7 s | 0.60″ |
| Transpluto | 400 | 13.1 s | +4.3 s | 0.47″ |
| Harrington | 400 | 12.9 s | +0.4 s | 0.31″ |
| NeptuneLeverrier, NeptuneAdams | 398 | 26.2 s | +11.1 s | 0.75″ |
| Nibiru | 400 | 40.6 s | +30.9 s | 1.57″ |
| PlutoLowell, PlutoPickering | 399 | 19.7 s, 15.8 s | +5.3 s, +5.5 s | 0.51″ |
| Vulcan | 7461 | 94.3 s | −0.1 s | 0.32″ |
| Proserpina | 399 | 19.0 s | +0.5 s | 0.001″ |
| WhiteMoon, Waldemath | 0 | — | — | — |

No count or kind mismatch. The closest speed extremum is Nibiru's, 2.3e-3°/day.

**Runtime, local, one thread:** the engine scans the apogee in about 120 s and
each fictitious body in 2–11 s (about 60 s for all 19).

## Goals

- Gate apogee and fictitious-body stations station for station against SWIEPH.
- Keep SE's positions as the reference while avoiding the faults of its
  reported speed.
- Handle the apogee's near-graze by rule rather than by luck.

## Non-goals

- Changing the planet, true-node or asteroid corpora, which keep SE's
  reported speed (their agreement with the engine is already measured and
  gated; the convention difference is documented).
- The apparent frame for the fictitious bodies (SE's light-time correction
  jumps there, #273).
- The perigees: each is its apogee's antipode, so its stations are the
  apogee's (#276).

## Design

### 1. Reference corpora: `tools/se-stations-reference`

Two new modes, each writing its files into `--out <dir>` like `--asteroids`:

- `--true-apogee [--ephe <dir>]`: `geo` group, SE_OSCU_APOG (id 13), default
  flags with `SEFLG_SWIEPH`, full span, 0.01-day grid, h = 0.001 day. Writes
  `true-apogee.csv`, `true-apogee-manifest.txt`, `true-apogee-windows.csv`,
  `true-apogee-windows-manifest.txt`.
- `--fictitious [--ephe <dir>]`: `mean` group,
  `SEFLG_SWIEPH|SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL|SEFLG_NONUT`, the 19
  bodies 40–58, full span, 0.25-day grid, h = 0.01 day. Writes
  `fictitious.csv`, `fictitious-manifest.txt`, `fictitious-windows.csv`,
  `fictitious-windows-manifest.txt`.

The corpus CSVs keep the existing format, `group,body,jd_tt,lon_deg,kind`,
under a header that states the central-difference definition.

**Station definition.** The speed is `cd(t) = wrap180(lon(t + h) − lon(t −
h)) / 2h` from `swe_calc` longitudes without `SEFLG_SPEED`. A station is a
sign change of `cd` on the grid, bisected to 1e-7 day. The grid starts
0.00137 day after the span start, so its samples do not fall on the
regular instants where SE's apogee jumps were seen.

**Artefact rejection.** Each body is scanned twice, with h and 1.3 h. A
station is kept when the other scan has a station of the same kind within
1 s; the kept instant is the h scan's. A station found in only one scan must
belong to an R/D pair less than 3 × h apart in that scan (the signature of
an isolated jump stepped over at t ± h); the pair is dropped and logged to
stderr with its instants. Anything else found in only one scan aborts the
run. The header records the number of dropped pairs per body.

**Graze windows.** A sampled extremum of `cd` that stays on one side of zero
and lies within the margin of zero becomes a window: from the first grid
instant on each side at which `|cd| ≥ 2 × margin`. Overlapping windows merge.
Each row records the station count of the corpus inside it, the sign of `cd`
at both edges, the margin and the miss:

```
group,body,turn_jd_tt,miss_deg_per_day,lo_jd_tt,hi_jd_tt,se_stations,lo_side,hi_side,margin_deg_per_day
```

`se_stations` is odd exactly when `lo_side ≠ hi_side`; a window whose count
contradicts its sides, or that reaches the span's edge, aborts the run. A
station inside a window keeps its corpus row; the window only changes how the
gate compares it.

**Margins.** The margin is 3 times the largest engine-minus-SE difference of
the speed measured at the SE speed extrema below 0.1°/day, rounded up to a
round figure, following the 3″ rule of the lunar-point aspect windows (3.6
times the measured difference). Task 1 measures that difference (engine speed
by the same central difference of `EventEngine::longitude_at`) and fixes the
two constants, `TRUE_APOGEE_SPEED_MARGIN_DEG_PER_DAY` and
`FICTITIOUS_SPEED_MARGIN_DEG_PER_DAY`, in the tool. The apogee's 6e-5°/day
extremum becomes a window if the margin exceeds it; the fictitious bodies'
closest extremum (2.3e-3°/day) is expected to stay outside.

**Pins.** `sepl_18`/`semo_18` are already pinned in `pins.rs`; the
`seorbel.txt` pin (SHA-256 97b454ff…, from `tools/se-aspects-reference`) is
added, and `--fictitious` refuses to run without it.

### 2. Gate: `pleiades-validate` `stations_validation`

- **Scopes.** `Scope::TrueApogee` (packaged engine, `geo`) and
  `Scope::Fictitious` (`RoutingBackend[PackagedDataBackend,
  FictitiousBackend(PackagedDataBackend)]`, `mean`). Both run in the full gate
  (`validate-stations`, `mise run gate-stations`) after the asteroids, and
  each has its own floor: its corpus rows outside windows.
- **Generic scan.** `scan_series_chunked` and `scan_chunk` become generic over
  `B: EphemerisBackend` (every backend is `Send + Sync`); the existing scopes
  keep `EventEngine<PackagedDataBackend>`.
- **Comparison outside windows:** station for station, as for the asteroids:
  equal counts and kinds, time and longitude within the body's ceilings.
- **Windows** (`stations_validation/windows.rs`): parse and check the windows
  file against its manifest and the corpus (`se_stations` equals the corpus
  rows inside). Inside a window each side's station count must have the
  window's parity; a window whose miss is at least half its margin must hold
  equal counts; equal counts are paired and reported without ceilings. At
  each edge the engine's speed, by the tool's central difference of
  `longitude_at`, must have the recorded sign.
- **Ceilings** (`stations_thresholds`): per body, the measured maximum times
  1.4, with a 2.3″ floor on longitude, the asteroid rule (#250). Expected from
  the spike: the apogee about 930 s, the fictitious bodies 25–135 s, every
  longitude ceiling 2.3″.
- **Blocking tier.** `validate_stations_corpus_subset` also checks both
  corpora and both windows files against their manifests, with no scan, like
  the asteroid corpus (#254), with tamper tests.

### 3. Where it runs

The pass is part of `gate-stations`, which runs under a minute on the nightly
runner. Before merging, a nightly is dispatched on the branch. If the
`stations-gate` step stays under 15 minutes the pass stays there; otherwise it
moves to its own job and mise task, as `aspects-lunar-gate` did (#272).

### 4. Docs and issues

`docs/status.md`, the `pleiades-events` README and the stations gate module
docs say what is gated and why these two corpora use the longitude derivative
(SE's apogee jumps; SE's Proserpina speed is inconsistent with its own
positions). #167 gets a comment and (d) is ticked.

## Error handling

New `StationsError` variants for a malformed windows file, a window whose
recorded count disagrees with the corpus, a parity violation, a strict
window with unequal counts, and an edge whose engine speed has the wrong
sign; each names the series and the window.

## Testing

- **Reference tool:** unit tests for the artefact filter (a synthetic speed
  with an isolated jump gives a 2h pair that is dropped; a real station
  survives both h; a lone unmatched station aborts) and for the window edges
  and parity.
- **Gate, blocking:** manifest and windows checks for both corpora, with
  tamper tests (a changed row, a drifted manifest, a window count that
  disagrees with the corpus).
- **Gate, unit:** window parity, strict counts and edge sides on synthetic
  stations.
- **Full gate:** `validate-stations` passes; the report lists both new scopes.
- `mise run test-full` and a branch nightly before merging.

## Risks

- **A margin that is too small** leaves an ephemeris-dependent station pair
  in the strict comparison. Mitigation: the margin comes from a measured
  speed difference, and the window file is regenerated with it.
- **An artefact that is not a 2h pair** aborts generation instead of
  silently entering the corpus.
- **Runtime:** measured on a branch nightly before deciding where the pass
  runs.
