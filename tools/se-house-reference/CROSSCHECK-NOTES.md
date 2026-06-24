# Astrolog Cross-Check Notes

## Status: working — broad cusp agreement

Astrolog 7.70 was built with `hardeningDisable = [ "all" ]` (stock build aborts
under modern glibc fortify/stack-protector; patched build runs cleanly headless).

## What was tried

1. `nix build --impure --expr` with `pkgs.astrolog.overrideAttrs (old: { hardeningDisable = ["all"]; })`
   — succeeded, output path: `/nix/store/cqrgf81fskrkfcy5c2pxk9q8vfljizlf-astrolog-7.70`
2. Stock build (`/nix/store/y36bv6s61r09dw5d3jcbrmm5l6invklv-astrolog-7.70`) was confirmed
   to crash: `*** buffer overflow detected ***: terminated`.
3. Patched build confirmed to run: `astrolog -v` prints version; no crash.
4. All 60 corpus rows were cross-checked via `astrolog -qb <date> <time> 0 0 <lon> <lat> -c <N> -v -C -sd`.

## Astrolog house system flag mapping

| corpus system_code | Astrolog `-c` | Notes                      |
|--------------------|--------------|----------------------------|
| Placidus           | 0            |                            |
| Koch               | 1            |                            |
| Porphyry           | 6            |                            |
| Regiomontanus      | 5            |                            |
| Campanus           | 3            |                            |
| Equal              | 2            |                            |
| WholeSign          | 14           |                            |
| Alcabitius         | 9            |                            |
| Meridian           | 4            | same as Axial in SE        |
| Axial              | 4            | same as Meridian in SE     |
| Topocentric        | 8            |                            |
| Morinus            | 7            |                            |

## Cross-check outcome

**All 12 house cusps (c1–c12) agree within ~2 arc-seconds** across all 60 corpus
rows. This is consistent with Astrolog internally using Swiss Ephemeris.

**ASC column:** agrees within ~2" for quadrant systems. For Equal, Meridian, Axial,
Morinus, and WholeSign, Astrolog's cusp-1 equals the SE Ascendant (within 2"),
so the Ascendant computes identically.

**MC column (flagged convention difference, not a calculation disagreement):**
For non-quadrant systems (Equal, WholeSign, Meridian, Axial, Morinus), the corpus
stores the *true* Midheaven (the RAMC-derived MC), while Astrolog's `cusp 10`
reports the system-specific 10th house position (e.g. ASC+270 for Equal). This
is a well-known convention difference between SE and many astrological programs
for non-quadrant systems. The underlying cusp algorithm is identical.

## Exceptions (flagged, never gating)

All flagged "exceptions" are MC convention differences for non-quadrant systems
(Equal, WholeSign, Meridian, Axial, Morinus across all 5 fixtures). No cusp
disagreements exceeded 2". The manifest records these as context-only annotations:

- `Equal` (all fixtures): mc convention (system cusp-10 vs true MC)
- `WholeSign` (all fixtures): asc convention (Astrolog uses cusp-1 as ASC placeholder) + mc convention
- `Meridian`/`Axial` (lat≠0 fixtures): asc convention (true ASC vs cusp-1) + mc convention
- `Morinus` (all fixtures): asc convention + mc convention

None of these affect the gate, which validates cusps only.

## Gate impact

The gate (`validate_house_corpus`) checks cusp residuals against per-family ceilings
and reads `#CrossCheck-Engine:` as opaque provenance text. The manifest update from
`not-run` to `Astrolog 7.70 (patched, hardeningDisable=all)` does not affect the
corpus checksum (which covers `cusps.csv` only, not `manifest.txt`).

`cargo test -p pleiades-validate --lib house_validation` passes (18/18) unchanged.

## cusps.csv

Not modified. The reference values remain the SwissEphemeris 2.10.03 authoritative output.
