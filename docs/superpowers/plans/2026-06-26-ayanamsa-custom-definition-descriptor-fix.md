# Ayanamsa Custom-Definition Descriptor Accuracy Fix — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Correct the false "labeled BABYL_… in Swiss Ephemeris" provenance text on the six custom-definition-only Babylonian ayanamsa descriptors, and patch-bump the compatibility profile that renders them.

**Architecture:** Pure metadata/text fix. The six modes (`BabylonianHouse`, `BabylonianSissy`, `BabylonianTrueGeoc`, `BabylonianTrueTopc`, `BabylonianTrueObs`, `BabylonianHouseObs`) are an already-correct, validated custom-definition-only category; only their `description` strings are inaccurate. We replace the 12 description literals (each mode appears once in `RELEASE_AYANAMSAS` and once in `BUILT_IN_AYANAMSAS`), update the two render tests that pin the old text, and bump the rendered profile id `0.7.0 → 0.7.1` across its four hardcoded sites. No enum, API, count, classification, or runtime-behavior change.

**Tech Stack:** Rust workspace (`pleiades-*` crates). Test runner `cargo test --workspace` (`mise run test`); full gate `mise run release-gate`; format check `mise run fmt`.

## Global Constraints

- Rust toolchain pinned at `1.96.0` (mise.toml); `rustfmt` + `clippy` required, `clippy` runs `-D warnings`.
- Do **not** change the public `Ayanamsa` enum, any catalogued count (stays 59 catalogued / 48 release-grade / 11 deferred), `README.md:20`, the workspace crate version (`Cargo.toml` stays `0.2.0` — this fix is non-breaking), or the custom-definition-only classification / partition / validation.
- Corrected description string, per mode (plain ASCII, two sentences, identical in both array occurrences):
  `Custom-definition-only Babylonian sidereal label (alias <ALIAS>); not a Swiss Ephemeris sidereal mode. Swiss Ephemeris defines no SE_SIDM code for it.`
  with `<ALIAS>` ∈ {`BABYL_HOUSE`, `BABYL_SISSY`, `BABYL_TRUE_GEOC`, `BABYL_TRUE_TOPC`, `BABYL_TRUE_OBS`, `BABYL_HOUSE_OBS`}.

---

### Task 1: Correct the six false descriptor strings

**Files:**
- Modify: `crates/pleiades-ayanamsa/src/catalog.rs` (lines 288/296/304/312/320/328 in `RELEASE_AYANAMSAS`; 817/825/833/841/849/857 in `BUILT_IN_AYANAMSAS`)
- Test: `crates/pleiades-core/src/compatibility/tests.rs:1111,1113` (fn `display_lists_release_sections`)

**Interfaces:**
- Consumes: nothing from other tasks.
- Produces: corrected `AyanamsaDescriptor::description` text rendered by the compatibility profile; relied on by Task 2's version bump rationale.

- [ ] **Step 1: Update the two profile-render assertions to the corrected text (failing test first)**

In `crates/pleiades-core/src/compatibility/tests.rs`, replace line 1111:

```rust
    assert!(rendered.contains("Babylonian sidereal mode labeled BABYL_HOUSE in Swiss Ephemeris."));
```

with:

```rust
    assert!(rendered.contains(
        "Custom-definition-only Babylonian sidereal label (alias BABYL_HOUSE); not a Swiss Ephemeris sidereal mode. Swiss Ephemeris defines no SE_SIDM code for it."
    ));
```

and replace lines 1112–1114:

```rust
    assert!(
        rendered.contains("Babylonian sidereal mode labeled BABYL_HOUSE_OBS in Swiss Ephemeris.")
    );
```

with:

```rust
    assert!(rendered.contains(
        "Custom-definition-only Babylonian sidereal label (alias BABYL_HOUSE_OBS); not a Swiss Ephemeris sidereal mode. Swiss Ephemeris defines no SE_SIDM code for it."
    ));
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p pleiades-core display_lists_release_sections`
Expected: FAIL — assertion fails because the catalog still renders the old "labeled … in Swiss Ephemeris" text.

- [ ] **Step 3: Replace the 12 descriptor strings in `catalog.rs`**

Each of the six old strings appears exactly twice and is unique to its mode, so replace-all per string. Apply these six replacements (old → new), each replacing **both** occurrences:

```
"Babylonian sidereal mode labeled BABYL_HOUSE in Swiss Ephemeris."
→ "Custom-definition-only Babylonian sidereal label (alias BABYL_HOUSE); not a Swiss Ephemeris sidereal mode. Swiss Ephemeris defines no SE_SIDM code for it."

"Babylonian sidereal mode labeled BABYL_SISSY in Swiss Ephemeris."
→ "Custom-definition-only Babylonian sidereal label (alias BABYL_SISSY); not a Swiss Ephemeris sidereal mode. Swiss Ephemeris defines no SE_SIDM code for it."

"Babylonian sidereal mode labeled BABYL_TRUE_GEOC in Swiss Ephemeris."
→ "Custom-definition-only Babylonian sidereal label (alias BABYL_TRUE_GEOC); not a Swiss Ephemeris sidereal mode. Swiss Ephemeris defines no SE_SIDM code for it."

"Babylonian sidereal mode labeled BABYL_TRUE_TOPC in Swiss Ephemeris."
→ "Custom-definition-only Babylonian sidereal label (alias BABYL_TRUE_TOPC); not a Swiss Ephemeris sidereal mode. Swiss Ephemeris defines no SE_SIDM code for it."

"Babylonian sidereal mode labeled BABYL_TRUE_OBS in Swiss Ephemeris."
→ "Custom-definition-only Babylonian sidereal label (alias BABYL_TRUE_OBS); not a Swiss Ephemeris sidereal mode. Swiss Ephemeris defines no SE_SIDM code for it."

"Babylonian sidereal mode labeled BABYL_HOUSE_OBS in Swiss Ephemeris."
→ "Custom-definition-only Babylonian sidereal label (alias BABYL_HOUSE_OBS); not a Swiss Ephemeris sidereal mode. Swiss Ephemeris defines no SE_SIDM code for it."
```

(Use an editor replace-all for each of the six old literals; each touches its two occurrences in `RELEASE_AYANAMSAS` and `BUILT_IN_AYANAMSAS`.)

- [ ] **Step 4: Verify no false-provenance text remains**

Run: `grep -rn "labeled.*in Swiss Ephemeris" crates/`
Expected: no output (zero matches).

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo test -p pleiades-core display_lists_release_sections`
Expected: PASS.

- [ ] **Step 6: Run the broader catalog + ayanamsa tests**

Run: `cargo test -p pleiades-ayanamsa -p pleiades-core`
Expected: PASS (descriptor completeness, provenance-summary partition, and profile render tests all green; counts unchanged).

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-ayanamsa/src/catalog.rs crates/pleiades-core/src/compatibility/tests.rs
git commit -m "fix(ayanamsa): correct false SE-label text on 6 custom-definition Babylonian descriptors (slice 4)"
```

---

### Task 2: Patch-bump the compatibility profile `0.7.0 → 0.7.1`

**Files:**
- Modify: `crates/pleiades-core/src/compatibility/mod.rs:26`
- Modify: `README.md:18`
- Test: `crates/pleiades-cli/src/cli/tests/summary_commands.rs:433` (fn `summary_commands_render_compact_reports`)
- Test: `crates/pleiades-validate/src/tests/render_request.rs:333` (fn `cli_report_summary_lists_the_summary_command`)

**Interfaces:**
- Consumes: the corrected rendered profile from Task 1 (the reason the rendered bytes differ from archived `0.7.0`).
- Produces: profile id `pleiades-compatibility-profile/0.7.1` consistent across source, prose, and tests.

- [ ] **Step 1: Update the two render tests to expect `0.7.1` (failing test first)**

In `crates/pleiades-cli/src/cli/tests/summary_commands.rs:433`, change the expected line literal:

```rust
        line == "Release profile identifiers: v1 compatibility=pleiades-compatibility-profile/0.7.0, api-stability=pleiades-api-stability/0.2.0"
```

to:

```rust
        line == "Release profile identifiers: v1 compatibility=pleiades-compatibility-profile/0.7.1, api-stability=pleiades-api-stability/0.2.0"
```

In `crates/pleiades-validate/src/tests/render_request.rs:333`, change:

```rust
            "Release profile identifiers: v1 compatibility=pleiades-compatibility-profile/0.7.0, api-stability=pleiades-api-stability/0.2.0"
```

to:

```rust
            "Release profile identifiers: v1 compatibility=pleiades-compatibility-profile/0.7.1, api-stability=pleiades-api-stability/0.2.0"
```

- [ ] **Step 2: Run both tests to verify they fail**

Run: `cargo test -p pleiades-cli summary_commands_render_compact_reports && cargo test -p pleiades-validate cli_report_summary_lists_the_summary_command`
Expected: FAIL — source still emits `0.7.0`.

- [ ] **Step 3: Bump the profile id constant**

In `crates/pleiades-core/src/compatibility/mod.rs:26`, change:

```rust
pub const CURRENT_COMPATIBILITY_PROFILE_ID: &str = "pleiades-compatibility-profile/0.7.0";
```

to:

```rust
pub const CURRENT_COMPATIBILITY_PROFILE_ID: &str = "pleiades-compatibility-profile/0.7.1";
```

- [ ] **Step 4: Update the README reference**

In `README.md:18`, change:

```
- a release compatibility profile (`pleiades-compatibility-profile/0.7.0`),
```

to:

```
- a release compatibility profile (`pleiades-compatibility-profile/0.7.1`),
```

- [ ] **Step 5: Verify no stale `0.7.0` profile id remains**

Run: `grep -rn "compatibility-profile/0.7.0" crates/ README.md`
Expected: no output (zero matches).

- [ ] **Step 6: Run both tests to verify they pass**

Run: `cargo test -p pleiades-cli summary_commands_render_compact_reports && cargo test -p pleiades-validate cli_report_summary_lists_the_summary_command`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-core/src/compatibility/mod.rs README.md crates/pleiades-cli/src/cli/tests/summary_commands.rs crates/pleiades-validate/src/tests/render_request.rs
git commit -m "chore(compat): bump compatibility profile to 0.7.1 for corrected descriptor text (slice 4)"
```

---

### Task 3: Update PLAN.md and run the full gate

**Files:**
- Modify: `PLAN.md` (Phase 6 ayanamsa note + Status line)

**Interfaces:**
- Consumes: completed Tasks 1–2.
- Produces: final green gate; plan record of slice 4.

- [ ] **Step 1: Add the slice-4 record to `PLAN.md`**

In the Phase 6 ayanamsa progress section and the trailing `Status:` line, add a sentence (keep counts unchanged):

```
Phase 6 ayanamsa slice 4 (descriptor accuracy) is done (2026-06-26): the six
custom-definition-only Babylonian descriptors (House, Sissy, True Geoc, True
Topc, True Obs, House Obs) no longer falsely claim a Swiss Ephemeris label;
compatibility profile bumped to 0.7.1. Catalogued counts unchanged (59
catalogued / 48 release-grade / 11 deferred); the six remain the validated
custom-definition-only category, not release-claimed.
```

- [ ] **Step 2: Format check**

Run: `mise run fmt`
Expected: PASS (no diff). If it reports formatting, run `cargo fmt --all` and re-check.

- [ ] **Step 3: Full workspace test**

Run: `mise run test`
Expected: PASS across the workspace.

- [ ] **Step 4: Compatibility / claims audits**

Run: `mise run compat-claims-audit && mise run claims-audit`
Expected: PASS — tier↔evidence↔profile↔prose agreement holds (no claim tiers changed).

- [ ] **Step 5: Full release gate**

Run: `mise run release-gate`
Expected: PASS (fmt, lint, test-full, benchmark, audit, package-check, release-smoke, claims-audit all green).

- [ ] **Step 6: Commit**

```bash
git add PLAN.md
git commit -m "docs(plan): record ayanamsa descriptor-accuracy fix + profile 0.7.1 (slice 4)"
```

---

## Self-Review

**Spec coverage:**
- Corrected descriptor text (spec §1) → Task 1.
- Test alignment for the two pinned assertions (spec §2) → Task 1 Steps 1–5.
- Compatibility-profile version bump across all four sites (spec §3) → Task 2.
- "No descriptor asserts 'labeled … in Swiss Ephemeris'" acceptance → Task 1 Step 4.
- "Profile id is …/0.7.1 consistently" acceptance → Task 2 Step 5.
- "Counts unchanged / enum unchanged / gates green" acceptance → Task 3 Steps 3–5 (and Global Constraints forbid count/enum/Cargo edits).
- "PLAN.md records slice 4" acceptance → Task 3 Step 1.

**Placeholder scan:** none — every step has exact paths, literals, commands, and expected output.

**Type/string consistency:** the corrected description literal in Task 1 Step 3 is byte-identical to the strings asserted in Task 1 Step 1 (per `<ALIAS>`); the `0.7.1` profile id in Task 2 source edits is byte-identical to the test expectations in Task 2 Step 1. The two-sentence, plain-ASCII description form is used uniformly.
