# License notes — `se-occultations-reference`

This crate is a **build-time verification harness only**. It is **not shipped**
and is deliberately kept **outside the Cargo workspace** (its own `Cargo.lock`,
`publish = false`, listed in the root `[workspace].exclude`). Nothing in the
shipped `pleiades-*` crates depends on it, and the workspace lockfile therefore
stays pure-Rust (no `-sys`/FFI), which the `workspace-audit` gate enforces.

Its sole purpose is to link Swiss Ephemeris (via `libswisseph-sys`) to
**generate a reference corpus of lunar-occultation local circumstances and
global paths** (`swe_lun_occult_when_loc`, `swe_lun_occult_when_glob`,
`swe_lun_occult_where`) used to validate `EventEngine`'s occultation search
(SP-6). It runs the Moshier ephemeris (`SEFLG_MOSEPH`), so no Swiss Ephemeris
`.se1` data files are bundled, read, or distributed — Moshier is a kernel-free
analytic ephemeris.

## Fixed-star catalog (`data/sefstars.txt`)

Bright-star occultations (Aldebaran, Regulus, Antares, Sirius) require Swiss
Ephemeris' fixed-star catalog `sefstars.txt`, which is bundled here under
`data/`. It is the unmodified Astrodienst-distributed catalog (md5
`3658a5a37ef795ada934c451024801c1`); its Spica record matches, byte-for-byte
modulo whitespace, the authentic record hard-coded inside the Swiss Ephemeris C
source (`get_builtin_star`, `sweph.c`), confirming provenance. It is a **data
file** (star positions/proper motions), used only at corpus-generation time; no
Swiss Ephemeris *source* enters the shipped product.

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
`se-fictitious-reference` / `se-nodaps-reference` tools, which follow the same
isolated, verification-only posture.
