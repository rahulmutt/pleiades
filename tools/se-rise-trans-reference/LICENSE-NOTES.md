# License notes — `se-rise-trans-reference`

This crate is a **build-time verification harness only**. It is **not shipped**
and is deliberately kept **outside the Cargo workspace** (its own `Cargo.lock`,
`publish = false`, listed in the root `[workspace].exclude`). Nothing in the
shipped `pleiades-*` crates depends on it, and the workspace lockfile therefore
stays pure-Rust (no `-sys`/FFI), which the `workspace-audit` gate enforces.

Its sole purpose is to link Swiss Ephemeris (via `swisseph` / `libswisseph-sys`)
to **generate a reference corpus of rise / set / transit times and horizontal
(azimuth / altitude) coordinates** used to validate the pure-Rust events engine.
It runs the Moshier ephemeris (`SEFLG_MOSEPH`), so no Swiss Ephemeris `.se1`
data files are bundled or distributed.

The fixed-star fixtures (`swe_fixstar` via `swe_rise_trans`) require the Swiss
Ephemeris star catalog `sefstars.txt` to be present in the ephemeris path
supplied at generation time (`SE_EPHE_PATH` / `--ephe`). That file is **not**
committed to this repository; it is read from the local machine only while the
corpus is generated, and does not enter the shipped artifacts.

## Swiss Ephemeris licensing

Swiss Ephemeris (© Astrodienst AG) is dual-licensed: AGPL, or a separate
commercial/professional license. Because this tool is used **only internally to
produce verification fixtures** and is **never distributed as part of the
product**, no Swiss Ephemeris code, binaries, or data files enter the shipped
artifacts. The generated CSV corpus contains numeric reference values only, not
Swiss Ephemeris source or data.

Anyone building this tool locally must have libclang available
(`LIBCLANG_PATH`) and is responsible for their own compliance with the Swiss
Ephemeris license terms for their use. See the sibling `se-crossings-reference`
tool, which follows the same isolated, verification-only posture.
