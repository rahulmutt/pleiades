# License notes — `se-stations-reference`

This crate is a **build-time verification harness only**. It is **not shipped**
and is deliberately kept **outside the Cargo workspace** (its own `Cargo.lock`,
`publish = false`, listed in the root `[workspace].exclude`). Nothing in the
shipped `pleiades-*` crates depends on it, and the workspace lockfile therefore
stays pure-Rust (no `-sys`/FFI), which the `workspace-audit` gate enforces.

Its sole purpose is to link Swiss Ephemeris (via `swisseph` / `libswisseph-sys`)
to **generate a planetary station reference corpus** (the instants at which
Swiss Ephemeris's own longitude speed changes sign, for Mercury–Pluto and the
true lunar node) used to validate the pure-Rust engine's
`EventEngine::stations_in_range`. It runs the Moshier ephemeris (`SEFLG_MOSEPH`), so no Swiss
Ephemeris data files are bundled or distributed.

## Swiss Ephemeris licensing

Swiss Ephemeris (© Astrodienst AG) is dual-licensed: AGPL, or a separate
commercial/professional license. Because this tool is used **only internally to
produce verification fixtures** and is **never distributed as part of the
product**, no Swiss Ephemeris code, binaries, or data files enter the shipped
artifacts. The generated CSV corpus contains numeric reference values only, not
Swiss Ephemeris source or data.

Anyone building this tool locally must have libclang available
(`LIBCLANG_PATH`) and is responsible for their own compliance with the Swiss
Ephemeris license terms for their use. See the sibling `se-lilith-reference`
tool, which follows the same isolated, verification-only posture.
