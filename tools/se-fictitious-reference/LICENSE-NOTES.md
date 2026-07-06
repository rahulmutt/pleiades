# License notes — `se-fictitious-reference`

This crate is a **build-time verification harness only**. It is **not shipped**
and is deliberately kept **outside the Cargo workspace** (its own `Cargo.lock`,
`publish = false`, listed in the root `[workspace].exclude`). Nothing in the
shipped `pleiades-*` crates depends on it, and the workspace lockfile therefore
stays pure-Rust (no `-sys`/FFI), which the `workspace-audit` gate enforces.

Its sole purpose is to link Swiss Ephemeris (via `libswisseph-sys`) to
**generate a reference corpus of fictitious (hypothetical) body positions**
(geocentric J2000-ecliptic longitude/latitude/distance, geometric) used to
validate the pure-Rust `pleiades-fict` backend. It runs the Moshier ephemeris
(`SEFLG_MOSEPH`), so no Swiss Ephemeris `.se1` data files are bundled, read, or
distributed.

## `data/seorbel.txt`

`data/seorbel.txt` is the verbatim upstream Swiss Ephemeris fictitious-body
orbital-element file (© Astrodienst AG), committed as build-time input to this
tool and as provenance for the transcription in
`crates/pleiades-fict/data/fictitious-elements.csv`. It is required at build
time because SE's built-in fallback element set only covers fictitious bodies
40-54; bodies 55-58 (Vulcan, White Moon, Proserpina, Waldemath) are read from
this file. It is not shipped as part of any `pleiades-*` crate artifact.

## Swiss Ephemeris licensing

Swiss Ephemeris (© Astrodienst AG) is dual-licensed: AGPL, or a separate
commercial/professional license. Because this tool is used **only internally to
produce verification fixtures** and is **never distributed as part of the
product**, no Swiss Ephemeris code or binaries enter the shipped artifacts. The
generated CSV corpus contains numeric reference values only, not Swiss
Ephemeris source.

Anyone building this tool locally must have libclang available
(`LIBCLANG_PATH`) and is responsible for their own compliance with the Swiss
Ephemeris license terms for their use. See the sibling
`se-eclipse-local-reference` tool, which follows the same isolated,
verification-only posture.
