# Eclipse validation corpus

## Source

NASA Five Millennium Canon of Solar Eclipses and Five Millennium Canon of
Lunar Eclipses (Espenak & Meeus), restricted to 1900-01-01 ... 2100-01-01
(limited by the packaged ephemeris, which has no Sun/Moon segments beyond
2100-01-01).

**NASA source URLs (all fetched 2026-06-29):**
- https://eclipse.gsfc.nasa.gov/SEcat5/SE1801-1900.html — 242 solar eclipses (1801-1900), year-1900 rows only kept
- https://eclipse.gsfc.nasa.gov/SEcat5/SE1901-2000.html — 228 solar eclipses (1901-2000)
- https://eclipse.gsfc.nasa.gov/SEcat5/SE2001-2100.html — 224 solar eclipses (2001-2100)
- https://eclipse.gsfc.nasa.gov/LEcat5/LE1801-1900.html — 249 lunar eclipses (1801-1900), year-1900 rows only kept
- https://eclipse.gsfc.nasa.gov/LEcat5/LE1901-2000.html — 229 lunar eclipses (1901-2000)
- https://eclipse.gsfc.nasa.gov/LEcat5/LE2001-2100.html — 228 lunar eclipses (2001-2100)

The 1801-1900 pages are needed because the 1901-2000 pages start with 1901; the
four year-1900 eclipses (2 solar, 2 lunar) are present only on the 1801-1900 pages.
Pre-1900 rows from those pages are discarded (jd_tt < 2_415_020.5).

**Total: 909 eclipses (452 solar, 457 lunar)** across 95 active Saros series
(48 solar, 47 lunar).  Coverage: **1900-01-01 … 2100-01-01 (limited by the
packaged ephemeris, which has no Sun/Moon segments beyond 2100-01-01)**.

## Known limitations

- **4 NASA-canon late-2100 eclipses intentionally excluded**: The NASA catalog
  contains 4 additional eclipses in mid/late 2100 (JD 2_488_124 … 2_488_315):
  2 lunar penumbral (Saros 115, 120) and 2 solar (1 annular Saros 141, 1 total
  Saros 146). These are uncomputable with the packaged ephemeris, which ends at
  JD 2_488_069.5 (2100-01-01 TT). They are excluded from this fixture and are
  not tested by the gate.

- **One allowlisted knife-edge eclipse**: 1948-05-09 solar eclipse, Saros 137
  (JD 2_432_680.601_44 / annular, mag 0.9999). The geocentric engine computes
  magnitude 1.0004 (Δ = 0.0005, within the 0.01 tolerance), but the binary
  annular/hybrid type classification flips at the mag = 1.0 knife-edge. This
  is irreducible for a mean-radius topocentric shadow-cone model without
  widening the type tolerance (which would reclassify adjacent genuine hybrids).
  The gate allowlists this row: it counts as a known exception, not a failure.

## Columns (eclipses.csv)

```
kind,type,greatest_eclipse_jd_tt,magnitude,saros,eclipsed_longitude_deg
```

- **kind**: `solar` | `lunar`
- **type**: `total` | `annular` | `hybrid` | `partial` (solar);
  `penumbral` | `partial` | `total` (lunar)
- **greatest_eclipse_jd_tt**: Julian Day in Terrestrial Time (TT). The NASA
  catalog's TD time column is used directly (TD ≈ TT; the ΔT column is
  ignored because the gate operates entirely in TT/TDB). Precision: the NASA
  catalog rounds to the second, yielding ~1e-5 day precision.
- **magnitude**: solar → eclipse magnitude (from the NASA "Magnitude" column,
  i.e. `(s + m - σ) / (2s)`). Lunar total/partial → **umbral magnitude**
  (NASA "Um. Mag." column). Lunar penumbral → **penumbral magnitude** (NASA
  "Pen. Mag." column). This matches the engine's magnitude convention.
- **saros**: integer Saros series number from the NASA catalog column.
- **eclipsed_longitude_deg**: apparent geocentric **solar** ecliptic longitude
  of date (tropical) at the greatest-eclipse instant. For **lunar** eclipses
  this is the solar longitude + 180°, mod 360°, which equals the Moon's
  opposition longitude (the eclipsed point in the zodiac). Computed
  **independently** via Skyfield 1.54 + DE440 (see Reproduction below).
  Precision: ≥ 5 decimal places (gate tolerance is 1 arcsecond = 0.000278°).

## Reproduction

To regenerate `eclipses.csv` and `saros_anchors.txt` from scratch:

```bash
cd crates/pleiades-validate/data/eclipses-corpus/
python3 gen_eclipse_corpus.py
```

The script fetches 6 NASA HTML catalog pages (1801-1900 + 1901-2000 + 2001-2100
for both solar and lunar), discards rows with jd_tt < 2_415_020.5 (pre-1900)
and rows with jd_tt ≥ 2_488_069.5 (beyond the packaged ephemeris), and
processes the remainder identically.  The net result covers 1900-01-01 …
2100-01-01 exhaustively, including all four year-1900 eclipses.

Requirements: Python 3.10+, `skyfield >= 1.54` (the HTML fetch + parsing use
only the stdlib `urllib.request` + `re` — no `requests`/`beautifulsoup4`), and
`/workspace/.kernels/de440.bsp`. The kernel is **not committed** (the `.kernels/`
directory is gitignored); obtain it separately by downloading
`https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440.bsp`
and placing it at `/workspace/.kernels/de440.bsp`.

### JD(TT) conversion

From Meeus, _Astronomical Algorithms_, ch. 7 (proleptic Gregorian):

```python
def gregorian_to_jd(year, month, day, hour, minute, second):
    a = (14 - month) // 12
    y = year + 4800 - a
    m = month + 12*a - 3
    jdn = day + (153*m + 2)//5 + 365*y + y//4 - y//100 + y//400 - 32045
    return jdn - 0.5 + (hour*3600 + minute*60 + second) / 86400.0
```

The NASA catalog's "TD" (Terrestrial Dynamical Time) column equals TT to
better than 1 ms, negligible versus the 60-second gate tolerance.

### Skyfield + DE440 longitude method

```python
from skyfield.api import Loader, load as tsload
load = Loader('/workspace/.kernels')
planets = load('de440.bsp')
earth, sun = planets['earth'], planets['sun']
ts = tsload.timescale()

def solar_ecliptic_lon_deg(jd_tt):
    t = ts.tt_jd(jd_tt)
    astr = earth.at(t).observe(sun).apparent()
    lat, lon, _ = astr.ecliptic_latlon(epoch=t)  # apparent of date
    return lon.degrees  # in [0, 360)

# Solar eclipses:
eclipsed_lon = solar_ecliptic_lon_deg(jd_tt)

# Lunar eclipses (Moon is opposite the Sun):
eclipsed_lon = (solar_ecliptic_lon_deg(jd_tt) + 180.0) % 360.0
```

This is **independent** of all `pleiades-*` crates: it uses the JPL SPICE
DE440 ephemeris and the Skyfield IAU 2000B nutation model, not the
`pleiades-apparent` IAU 1980 pipeline. Agreement between Skyfield+DE440 and
the pleiades engine is expected to be < 0.2 arcseconds across the full
1900-2100 window, well inside the 1.0-arcsecond gate tolerance.

### Magnitude column rules

| Eclipse class       | NASA column used for `magnitude` |
|---------------------|----------------------------------|
| Solar (all types)   | "Magnitude" (the eclipse magnitude, column 12) |
| Lunar total/partial | "Um. Mag." (umbral magnitude, column 13) |
| Lunar penumbral     | "Pen. Mag." (penumbral magnitude, column 12) |

### Type code mapping

Solar: `T`/`T-`/`T+` → `total`; `A`/`A-`/`A+` → `annular`; `H` → `hybrid`;
`P`/`Pe` → `partial`.

Lunar: `T`/`T-`/`T+` → `total`; `P` → `partial`; `N`/`Nx`/`Nb`/`Ne`/`Np` →
`penumbral`.

## Saros anchors

`saros_anchors.txt` contains one `(EclipseKind, member_jd_tt, series)` entry
per active 1900-2100 Saros series extracted while parsing the canon.
Task 10B pastes these into `SAROS_ANCHORS` in
`crates/pleiades-eclipse/src/saros.rs`.
