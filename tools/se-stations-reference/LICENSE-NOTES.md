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
`EventEngine::stations_in_range`. In its default planet mode it runs the Moshier ephemeris
(`SEFLG_MOSEPH`), so no Swiss Ephemeris data files are used, bundled or distributed.

The `--asteroids` mode additionally generates the Ceres, Pallas, Juno and Vesta
station corpus (`asteroids.csv` + `asteroids-manifest.txt`). It reads the Swiss
Ephemeris data files `seas_18`, `sepl_18` and `semo_18` (SWIEPH), which are
SHA-256-pinned in `src/pins.rs`. The developer downloads them into this tool's
gitignored `data/` directory (from
`https://raw.githubusercontent.com/aloistr/swisseph/master/ephe/`); they are never
committed or distributed. Regenerate with:

```sh
devenv shell -- env CFLAGS=-std=gnu17 cargo build --release --manifest-path tools/se-stations-reference/Cargo.toml
tools/se-stations-reference/target/release/se-stations-reference --asteroids --out crates/pleiades-validate/data/stations-corpus
```

(run the binary outside devenv, which prints a banner on stdout).

## Swiss Ephemeris licensing

Swiss Ephemeris (© Astrodienst AG) is dual-licensed: AGPL, or a separate
commercial/professional license. Because this tool is used **only internally to
produce verification fixtures** and is **never distributed as part of the
product**, no Swiss Ephemeris code, binaries, or data files enter the shipped
artifacts. The SWIEPH data files read by `--asteroids` are covered by the same
AGPL / professional dual licence as the library and stay local to the developer
machine. The generated CSV corpus contains numeric reference values only, not
Swiss Ephemeris source or data.

Anyone building this tool locally must have libclang available
(`LIBCLANG_PATH`) and is responsible for their own compliance with the Swiss
Ephemeris license terms for their use. See the sibling `se-lilith-reference`
tool, which follows the same isolated, verification-only posture.
