#!/usr/bin/env python3
"""
gen_eclipse_corpus.py — one-off generator for the NASA-canon eclipse fixture.

Fetches six NASA HTML eclipse catalog pages (1801-1900 + 1901-2100), parses
every solar and lunar eclipse row within the supported JD window
[2_415_020.5, 2_488_069.5] (1900-01-01 … 2100-01-01), computes
greatest_eclipse_jd_tt from the NASA TD time (TD ≈ TT), and derives
eclipsed_longitude_deg using Skyfield 1.54 + DE440.

The 1801-1900 pages supply the year-1900 eclipses that the 1901-2000 pages
omit.  Rows with jd_tt < JD_WINDOW_START (pre-1900) are discarded.

Outputs (in the same directory as this script):
  eclipses.csv        — the committed fixture (909 rows)
  saros_anchors.txt   — one anchor per active Saros series for saros.rs

NOTE: JD_WINDOW_END = 2_488_069.5 is the packaged ephemeris's last Sun/Moon
segment (2100-01-01 TT).  The NASA canon contains 4 additional eclipses in
mid/late 2100 (JD 2_488_124 … 2_488_315) that are intentionally excluded here
because the backend has no ephemeris data beyond JD_WINDOW_END and cannot
recompute them.

Reproduction:
  cd <repo>/crates/pleiades-validate/data/eclipses-corpus/
  pip install skyfield
  python3 gen_eclipse_corpus.py

Dependencies: skyfield >= 1.54 (the HTML fetch + parsing use only the stdlib
urllib.request + re — no requests/beautifulsoup4), and /workspace/.kernels/de440.bsp
(NOT committed: the .kernels/ dir is gitignored; download de440.bsp from
https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440.bsp).
"""

import re
import sys
import os
import math
import urllib.request
from pathlib import Path

# ---------------------------------------------------------------------------
# Supported JD window  (1900-01-01 TT … 2100-01-01 TT)
# ---------------------------------------------------------------------------
JD_WINDOW_START = 2_415_020.5   # 1900-01-01 00:00:00 TT
# Upper bound is the packaged ephemeris's last Sun/Moon segment (2100-01-01 TT).
# The NASA canon's mid/late-2100 eclipses (JD > 2_488_069.5) are intentionally
# excluded: the backend has no data beyond this instant, so they are uncomputable.
JD_WINDOW_END = 2_488_069.5     # 2100-01-01 00:00:00 TT (packaged ephemeris last segment)

# ---------------------------------------------------------------------------
# NASA source URLs  (1801-1900 pages first, to cover all year-1900 eclipses)
# ---------------------------------------------------------------------------
SOLAR_URLS = [
    "https://eclipse.gsfc.nasa.gov/SEcat5/SE1801-1900.html",
    "https://eclipse.gsfc.nasa.gov/SEcat5/SE1901-2000.html",
    "https://eclipse.gsfc.nasa.gov/SEcat5/SE2001-2100.html",
]
LUNAR_URLS = [
    "https://eclipse.gsfc.nasa.gov/LEcat5/LE1801-1900.html",
    "https://eclipse.gsfc.nasa.gov/LEcat5/LE1901-2000.html",
    "https://eclipse.gsfc.nasa.gov/LEcat5/LE2001-2100.html",
]

MONTH_MAP = {
    "Jan": 1, "Feb": 2, "Mar": 3, "Apr": 4, "May": 5, "Jun": 6,
    "Jul": 7, "Aug": 8, "Sep": 9, "Oct": 10, "Nov": 11, "Dec": 12,
}

# ---------------------------------------------------------------------------
# JD(TT) conversion — Meeus, Astronomical Algorithms ch.7
# ---------------------------------------------------------------------------
def gregorian_to_jd(year: int, month: int, day: int,
                     hour: int, minute: int, second: int) -> float:
    """Convert a Gregorian calendar date + TD/TT time to Julian Day Number."""
    a = (14 - month) // 12
    y = year + 4800 - a
    m = month + 12 * a - 3
    jdn = (day + (153 * m + 2) // 5 + 365 * y
           + y // 4 - y // 100 + y // 400 - 32045)
    # Noon on the calendar date is JD integer; subtract 0.5 for midnight
    jd = jdn - 0.5 + (hour * 3600 + minute * 60 + second) / 86400.0
    return jd

# ---------------------------------------------------------------------------
# Solar type mapping
# ---------------------------------------------------------------------------
def map_solar_type(code: str) -> str:
    """Map NASA solar type code to CSV type string."""
    c = code.rstrip("+-").upper()
    if c == "T":
        return "total"
    if c == "A":
        return "annular"
    if c in ("H", "HT", "HA"):
        return "hybrid"
    if c in ("P", "PE"):
        return "partial"
    # fallback — try first character
    first = c[0] if c else "?"
    if first == "T":
        return "total"
    if first == "A":
        return "annular"
    if first == "H":
        return "hybrid"
    return "partial"

# ---------------------------------------------------------------------------
# Lunar type mapping
# ---------------------------------------------------------------------------
def map_lunar_type(code: str) -> str:
    """Map NASA lunar type code to CSV type string."""
    c = code.strip().upper()
    if c.startswith("T"):
        return "total"
    if c == "P":
        return "partial"
    # N, Nx, Nb, Ne, etc. are penumbral; any non-T, non-"P" code is penumbral too.
    if c.startswith("N") or not c.startswith("P"):
        return "penumbral"
    return "partial"

# ---------------------------------------------------------------------------
# HTML fetch + strip
# ---------------------------------------------------------------------------
def fetch_text(url: str) -> str:
    """Fetch a URL and return the text with HTML tags stripped."""
    print(f"  Fetching {url} ...", flush=True)
    req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(req, timeout=60) as resp:
        raw = resp.read().decode("utf-8", errors="replace")
    # Strip HTML tags
    text = re.sub(r"<[^>]+>", " ", raw)
    # Collapse whitespace runs inside lines, but keep newlines
    lines = [" ".join(line.split()) for line in text.splitlines()]
    return "\n".join(lines)

# ---------------------------------------------------------------------------
# Solar parser
# ---------------------------------------------------------------------------
def parse_solar_rows(text: str) -> list[dict]:
    """Parse solar eclipse rows from stripped NASA HTML text."""
    rows = []
    for line in text.splitlines():
        parts = line.split()
        if len(parts) < 12:
            continue
        # Row starts with 5-digit catalog number
        if not re.match(r"^\d{5}$", parts[0]):
            continue
        # Year is a 4-digit integer
        if not re.match(r"^\d{4}$", parts[1]):
            continue
        # Month is a 3-char abbreviation
        if parts[2] not in MONTH_MAP:
            continue
        try:
            year = int(parts[1])
            month = MONTH_MAP[parts[2]]
            day = int(parts[3])
            # Time in TD/TT: HH:MM:SS
            hms = parts[4]
            h, mi, s = [int(x) for x in hms.split(":")]
            # parts[5] = delta_T (ignore), parts[6] = lunation#, parts[7] = saros
            saros = int(parts[7])
            # parts[8] = type code, parts[9] = sub-flags
            # parts[10] = gamma, parts[11] = magnitude
            type_code = parts[8]
            magnitude = float(parts[11])
            eclipse_type = map_solar_type(type_code)
            jd = gregorian_to_jd(year, month, day, h, mi, s)
            rows.append({
                "kind": "solar",
                "type": eclipse_type,
                "jd_tt": jd,
                "magnitude": magnitude,
                "saros": saros,
            })
        except (ValueError, IndexError):
            continue
    return rows

# ---------------------------------------------------------------------------
# Lunar parser
# ---------------------------------------------------------------------------
def parse_lunar_rows(text: str) -> list[dict]:
    """Parse lunar eclipse rows from stripped NASA HTML text."""
    rows = []
    for line in text.splitlines():
        parts = line.split()
        if len(parts) < 13:
            continue
        if not re.match(r"^\d{5}$", parts[0]):
            continue
        if not re.match(r"^\d{4}$", parts[1]):
            continue
        if parts[2] not in MONTH_MAP:
            continue
        try:
            year = int(parts[1])
            month = MONTH_MAP[parts[2]]
            day = int(parts[3])
            hms = parts[4]
            h, mi, s = [int(x) for x in hms.split(":")]
            saros = int(parts[7])
            type_code = parts[8]
            # parts[10] = gamma, parts[11] = penumbral mag, parts[12] = umbral mag
            pen_mag = float(parts[11])
            try:
                umb_mag = float(parts[12])
            except (ValueError, IndexError):
                umb_mag = pen_mag
            eclipse_type = map_lunar_type(type_code)
            # Magnitude rule: total/partial → umbral; penumbral → penumbral mag
            if eclipse_type in ("total", "partial"):
                magnitude = umb_mag
            else:
                magnitude = pen_mag
            jd = gregorian_to_jd(year, month, day, h, mi, s)
            rows.append({
                "kind": "lunar",
                "type": eclipse_type,
                "jd_tt": jd,
                "magnitude": magnitude,
                "saros": saros,
            })
        except (ValueError, IndexError):
            continue
    return rows

# ---------------------------------------------------------------------------
# Skyfield longitude computation
# ---------------------------------------------------------------------------
def setup_skyfield():
    """Load Skyfield with DE440 from the workspace kernels directory."""
    from skyfield.api import Loader, load as tsload
    kernel_dir = "/workspace/.kernels"
    load = Loader(kernel_dir)
    planets = load("de440.bsp")
    earth = planets["earth"]
    sun = planets["sun"]
    ts = tsload.timescale()
    return earth, sun, ts

def compute_longitudes(rows: list[dict], earth, sun, ts) -> None:
    """
    Compute eclipsed_longitude_deg for every row in-place.

    Solar eclipses: apparent geocentric solar longitude of date.
    Lunar eclipses: solar longitude of date + 180°, mod 360°.

    Uses Skyfield 1.54 + DE440; independent of all pleiades crates.
    """
    print(f"  Computing {len(rows)} Skyfield+DE440 solar longitudes ...", flush=True)
    for i, row in enumerate(rows):
        t = ts.tt_jd(row["jd_tt"])
        # Apparent geocentric position of Sun from Earth, ecliptic of date
        astr = earth.at(t).observe(sun).apparent()
        lat, lon, _ = astr.ecliptic_latlon(epoch=t)
        solar_lon = lon.degrees  # already in [0, 360)
        if row["kind"] == "lunar":
            row["eclipsed_longitude_deg"] = (solar_lon + 180.0) % 360.0
        else:
            row["eclipsed_longitude_deg"] = solar_lon
        if (i + 1) % 100 == 0:
            print(f"    ... {i + 1}/{len(rows)}", flush=True)

# ---------------------------------------------------------------------------
# Saros anchors extraction
# ---------------------------------------------------------------------------
def extract_saros_anchors(rows: list[dict]) -> dict:
    """
    Return a dict mapping (kind, saros) -> first_jd_tt seen.
    Used to build saros.rs SAROS_ANCHORS.
    """
    anchors = {}
    for row in rows:
        key = (row["kind"], row["saros"])
        if key not in anchors:
            anchors[key] = row["jd_tt"]
    return anchors

# ---------------------------------------------------------------------------
# Output writers
# ---------------------------------------------------------------------------
def write_csv(rows: list[dict], path: Path) -> None:
    """Write eclipses.csv with the canonical 6-column header."""
    rows_sorted = sorted(rows, key=lambda r: r["jd_tt"])
    with open(path, "w", newline="") as f:
        f.write("kind,type,greatest_eclipse_jd_tt,magnitude,saros,eclipsed_longitude_deg\n")
        for row in rows_sorted:
            f.write(
                f"{row['kind']},{row['type']},{row['jd_tt']:.5f},"
                f"{row['magnitude']:.4f},{row['saros']},"
                f"{row['eclipsed_longitude_deg']:.5f}\n"
            )
    print(f"  Wrote {len(rows_sorted)} data rows to {path}")


def write_saros_anchors(anchors: dict, rows: list[dict], path: Path) -> None:
    """
    Write saros_anchors.txt: ready-to-paste Rust tuples + plain table.
    One entry per active 1900-2100 Saros series (both solar and lunar).
    """
    # Build a reverse index: jd -> (kind, type, saros) for display
    jd_to_row = {row["jd_tt"]: row for row in rows}

    solar_anchors = sorted(
        [(s, jd) for (k, s), jd in anchors.items() if k == "solar"],
        key=lambda x: x[0]
    )
    lunar_anchors = sorted(
        [(s, jd) for (k, s), jd in anchors.items() if k == "lunar"],
        key=lambda x: x[0]
    )

    with open(path, "w") as f:
        f.write("# Saros anchors extracted from NASA 1900-2100 eclipse catalogs\n")
        f.write("# One anchor per active series in the 1900-2100 window.\n")
        f.write("# Paste the Rust block into crates/pleiades-eclipse/src/saros.rs\n")
        f.write("# replacing the placeholder SAROS_ANCHORS constant.\n\n")

        f.write("// ---- RUST BLOCK: paste into saros.rs SAROS_ANCHORS ----\n")
        f.write("pub(crate) const SAROS_ANCHORS: &[(EclipseKind, f64, u32)] = &[\n")
        f.write("    // Solar series\n")
        for series, jd in solar_anchors:
            row = jd_to_row.get(jd, {})
            rtype = row.get("type", "?")
            f.write(f"    (EclipseKind::Solar, {jd:.5f}_f64, {series}),  // {rtype}\n")
        f.write("    // Lunar series\n")
        for series, jd in lunar_anchors:
            row = jd_to_row.get(jd, {})
            rtype = row.get("type", "?")
            f.write(f"    (EclipseKind::Lunar, {jd:.5f}_f64, {series}),  // {rtype}\n")
        f.write("];\n")
        f.write("// ---- END RUST BLOCK ----\n\n")

        f.write("# Plain table\n")
        f.write(f"{'kind':<8} {'series':>6} {'jd_tt':>14} {'type':<10}\n")
        f.write("-" * 44 + "\n")
        for series, jd in solar_anchors:
            row = jd_to_row.get(jd, {})
            f.write(f"{'solar':<8} {series:>6} {jd:>14.5f} {row.get('type','?'):<10}\n")
        for series, jd in lunar_anchors:
            row = jd_to_row.get(jd, {})
            f.write(f"{'lunar':<8} {series:>6} {jd:>14.5f} {row.get('type','?'):<10}\n")

    total = len(solar_anchors) + len(lunar_anchors)
    print(f"  Wrote {total} anchors ({len(solar_anchors)} solar, {len(lunar_anchors)} lunar) to {path}")

# ---------------------------------------------------------------------------
# Spot-check helper
# ---------------------------------------------------------------------------
def spot_check(rows: list[dict]) -> None:
    """Print spot-check rows for known eclipses."""
    checks = [
        # (description, approx_jd, kind)
        ("1999-08-11 total solar (Saros 145)", 2451401.96, "solar"),
        ("2017-08-21 total solar (Saros 145)", 2457987.27, "solar"),
        ("2024-04-08 total solar (Saros 139)", 2460408.27, "solar"),
        ("2019-01-21 total lunar (Saros 134)", 2458504.90, "lunar"),
    ]
    print("\n--- Spot-check ---")
    print(f"{'Description':<45} {'JD(TT)':>14} {'type':<12} {'saros':>6} {'lon_deg':>10}")
    print("-" * 95)
    for desc, ref_jd, kind in checks:
        best = min(
            [r for r in rows if r["kind"] == kind],
            key=lambda r: abs(r["jd_tt"] - ref_jd)
        )
        delta_s = abs(best["jd_tt"] - ref_jd) * 86400
        print(f"{desc:<45} {best['jd_tt']:>14.5f} {best['type']:<12} {best['saros']:>6}"
              f" {best['eclipsed_longitude_deg']:>10.5f}  (Δ={delta_s:.1f}s from ref)")
    print()

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def main():
    script_dir = Path(__file__).parent

    print("=== Eclipse corpus generator ===")
    print("Step 1: Fetch and parse NASA catalog pages")

    all_rows = []

    # Solar
    for url in SOLAR_URLS:
        text = fetch_text(url)
        rows = parse_solar_rows(text)
        print(f"    Solar rows parsed: {len(rows)}")
        all_rows.extend(rows)

    # Lunar
    for url in LUNAR_URLS:
        text = fetch_text(url)
        rows = parse_lunar_rows(text)
        print(f"    Lunar rows parsed: {len(rows)}")
        all_rows.extend(rows)

    # Filter to the supported window [JD_WINDOW_START, JD_WINDOW_END): discard
    # pre-1900 rows from the 1801-1900 pages and guard the upper bound so a
    # future re-run of a reformatted/extended NASA page cannot leak rows past
    # the engine window (which would break the gate's bounds).
    before = len(all_rows)
    all_rows = [
        r for r in all_rows if JD_WINDOW_START <= r["jd_tt"] < JD_WINDOW_END
    ]
    discarded = before - len(all_rows)
    if discarded:
        print(f"  Discarded {discarded} rows outside [{JD_WINDOW_START}, {JD_WINDOW_END})")

    solar_count = sum(1 for r in all_rows if r["kind"] == "solar")
    lunar_count = sum(1 for r in all_rows if r["kind"] == "lunar")
    print(f"\n  Total in window: {len(all_rows)} ({solar_count} solar, {lunar_count} lunar)")

    if len(all_rows) < 900:
        print("ERROR: fewer than 900 rows parsed — check NASA page format", file=sys.stderr)
        sys.exit(1)

    print("\nStep 2: Compute Skyfield+DE440 solar longitudes (independent of pleiades)")
    earth, sun, ts = setup_skyfield()
    compute_longitudes(all_rows, earth, sun, ts)

    print("\nStep 3: Spot-check")
    spot_check(all_rows)

    print("Step 4: Write CSV")
    csv_path = script_dir / "eclipses.csv"
    write_csv(all_rows, csv_path)

    print("\nStep 5: Extract Saros anchors")
    anchors = extract_saros_anchors(all_rows)
    anchors_path = script_dir / "saros_anchors.txt"
    write_saros_anchors(anchors, all_rows, anchors_path)

    # Final summary
    print(f"\n=== Done ===")
    print(f"  eclipses.csv:      {len(all_rows)} data rows ({solar_count} solar, {lunar_count} lunar) — window 1900-01-01…2100-01-01")
    print(f"  saros_anchors.txt: {len(anchors)} series anchors")
    print(f"  Longitude method:  Skyfield 1.54 + DE440, independent of pleiades")
    if len(all_rows) >= 900:
        print("  Row count gate:    PASS (>= 900)")
    else:
        print("  Row count gate:    FAIL")

if __name__ == "__main__":
    main()
