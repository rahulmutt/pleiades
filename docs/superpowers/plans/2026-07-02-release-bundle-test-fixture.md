# Release-Bundle Test Fixture Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Generate the release bundle once per test process and have every release-bundle verify test work from a cheap copy, cutting the family from ~15 min wall / ~79 CPU-min to roughly 2–3 min with zero assertion changes.

**Architecture:** A `OnceLock`-backed pristine-bundle fixture in each crate's test support module (one real `bundle-release` generation per process), plus a `stage_bundle_copy` helper that copies the pristine directory into a per-test temp dir. Tamper tests copy-then-tamper; generation-asserting tests consume the fixture's cached `rendered` string and `ReleaseBundle` struct.

**Tech Stack:** Rust, std-only (`std::sync::OnceLock`, `std::fs`), cargo test.

Spec: `docs/superpowers/specs/2026-07-02-release-bundle-test-fixture-design.md`

## Global Constraints

- **Zero assertion changes**: no assertion text, expected-error fragment, or test name may change anywhere in this plan.
- The pristine fixture dir is **never mutated and never deleted**; only `stage_bundle_copy` copies may be tampered with. (Read-only direct access to the pristine dir is allowed.)
- The validate-crate fixture uses `--rounds 1` (matches every converted test today); the CLI-crate fixture uses default rounds (matches its tests today).
- Exactly two real non-fixture generations remain after this plan: the `--output`-alias front-end test in each crate.
- Run `cargo fmt` before every commit.
- Timing note for every verification run: the first release-bundle test in a process pays a ~74 s memoization warm-up (validate crate; the CLI crate's default-rounds warm-up can take ~2–4 min). This is expected — do not kill the run.

---

### Task 1: Pristine-bundle fixture in pleiades-validate + convert the seven shared helpers

**Files:**
- Modify: `crates/pleiades-validate/src/tests/test_support.rs`
- Test: existing tests in `crates/pleiades-validate/src/tests/release_bundle_verify_b.rs` (no new test files)

**Interfaces:**
- Consumes: `render_release_bundle(rounds: usize, output_dir: impl AsRef<Path>) -> Result<ReleaseBundle, ReleaseBundleError>` and `ReleaseBundle` (both in scope via the module's existing `use super::*`; `ReleaseBundle` is `Clone` and its `Display` is exactly what the CLI prints for `bundle-release`).
- Produces (used by Tasks 2–4):
  - `pub(crate) struct PristineBundle { pub(crate) dir: PathBuf, pub(crate) rendered: String, pub(crate) bundle: ReleaseBundle }`
  - `pub(crate) fn pristine_release_bundle() -> &'static PristineBundle`
  - `pub(crate) fn stage_bundle_copy(prefix: &str) -> PathBuf`

- [ ] **Step 1: Add the fixture to `test_support.rs`**

Add `OnceLock` to the existing import at the top of the file:

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
```

Append after `unique_temp_dir` (the `unique_temp_dir` body itself is unchanged):

```rust
/// One pristine release bundle per test process.
///
/// Generating a bundle costs ~74s of memoized warm-up plus ~2s of file
/// writing; regenerating it per test dominated `test-full` wall clock
/// (see docs/superpowers/specs/2026-07-02-release-bundle-test-fixture-design.md).
/// Tests must never mutate `dir`; tamper tests take a `stage_bundle_copy`.
pub(crate) struct PristineBundle {
    pub(crate) dir: std::path::PathBuf,
    pub(crate) rendered: String,
    pub(crate) bundle: ReleaseBundle,
}

pub(crate) fn pristine_release_bundle() -> &'static PristineBundle {
    static PRISTINE: OnceLock<PristineBundle> = OnceLock::new();
    PRISTINE.get_or_init(|| {
        let dir = unique_temp_dir("pleiades-release-bundle-pristine");
        let bundle = render_release_bundle(1, &dir)
            .expect("pristine release bundle fixture should render");
        // The bundle-release CLI arm prints `bundle.to_string()`, so this is
        // byte-identical to what `render_cli(["bundle-release", ...])` returns.
        let rendered = bundle.to_string();
        PristineBundle {
            dir,
            rendered,
            bundle,
        }
    })
}

pub(crate) fn stage_bundle_copy(prefix: &str) -> std::path::PathBuf {
    let source = &pristine_release_bundle().dir;
    let dest = unique_temp_dir(prefix);
    for entry in std::fs::read_dir(source).expect("pristine bundle dir should be readable") {
        let entry = entry.expect("pristine bundle dir entry should be readable");
        let file_type = entry
            .file_type()
            .expect("pristine bundle entry file type should be readable");
        assert!(
            file_type.is_file(),
            "pristine bundle should contain only flat files, found non-file: {}",
            entry.path().display()
        );
        std::fs::copy(entry.path(), dest.join(entry.file_name()))
            .expect("pristine bundle file should copy into the staged dir");
    }
    dest
}
```

- [ ] **Step 2: Convert the seven `assert_release_bundle_rejects_*` helpers**

In each of the seven helpers in `test_support.rs`
(`assert_release_bundle_rejects_tampered_text_file`,
`assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum`,
`assert_release_bundle_rejects_symlinked_text_file`,
`assert_release_bundle_rejects_missing_manifest_entry`,
`assert_release_bundle_rejects_blank_manifest_value`,
`assert_release_bundle_rejects_duplicate_manifest_entry`,
`assert_release_bundle_rejects_whitespace_manifest_entry`), replace this exact
arrange block:

```rust
    let bundle_dir = unique_temp_dir(bundle_dir_prefix);
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");
```

with:

```rust
    let bundle_dir = stage_bundle_copy(bundle_dir_prefix);
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
```

Everything below the arrange block (tamper, verify, assert, cleanup) is
unchanged in all seven helpers.

- [ ] **Step 3: Verify it compiles and two helper-based tests pass**

Run:

```bash
cargo test -p pleiades-validate --lib -- --include-ignored --exact \
  tests::release_bundle_verify_b::verify_release_bundle_rejects_tampered_release_summary_file \
  tests::release_bundle_verify_b::verify_release_bundle_rejects_semantically_tampered_release_summary_file
```

Expected: `2 passed; 0 failed` in ~80–100 s (one warm-up + one fixture generation + two copies).

- [ ] **Step 4: Commit**

```bash
cargo fmt
git add crates/pleiades-validate/src/tests/test_support.rs
git commit -m "test(validate): add pristine release-bundle fixture; convert shared tamper helpers"
```

---

### Task 2: Bulk-convert inline generations in `release_bundle_verify_b.rs`

**Files:**
- Modify: `crates/pleiades-validate/src/tests/release_bundle_verify_b.rs` (~30 inline arrange blocks)

**Interfaces:**
- Consumes: `stage_bundle_copy(prefix: &str) -> PathBuf` from Task 1 (already in scope via the file's `use super::test_support::*;`).
- Produces: nothing new.

- [ ] **Step 1: Apply the mechanical conversion**

Every inline site is the identical 10-line block (same shape as the helpers in
Task 1, with a string-literal prefix). Convert them all with this script:

```bash
python3 - <<'EOF'
import pathlib
import re

block = re.compile(
    r'    let bundle_dir = unique_temp_dir\("([^"]+)"\);\n'
    r'    let bundle_dir_string = bundle_dir\.to_string_lossy\(\)\.to_string\(\);\n'
    r'    render_cli\(&\[\n'
    r'        "bundle-release",\n'
    r'        "--out",\n'
    r'        &bundle_dir_string,\n'
    r'        "--rounds",\n'
    r'        "1",\n'
    r'    \]\)\n'
    r'    \.expect\("bundle release should render"\);\n'
)
repl = (
    '    let bundle_dir = stage_bundle_copy("\\1");\n'
    '    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();\n'
)
path = pathlib.Path("crates/pleiades-validate/src/tests/release_bundle_verify_b.rs")
text = path.read_text()
text, count = block.subn(repl, text)
path.write_text(text)
print(f"converted {count} sites")
EOF
```

Expected output: `converted 30 sites` (the exact count must equal the number of
`"bundle-release"` occurrences in the file before the script runs — check with
the grep in Step 2; if any site resisted because its formatting differs, convert
it by hand to the same two-line replacement).

- [ ] **Step 2: Verify no generation sites remain in the file**

Run:

```bash
grep -c '"bundle-release"' crates/pleiades-validate/src/tests/release_bundle_verify_b.rs
```

Expected: `0`.

- [ ] **Step 3: Run the whole verify_b module**

```bash
cargo test -p pleiades-validate --lib -- --include-ignored tests::release_bundle_verify_b
```

Expected: `122 passed; 0 failed`, total wall time roughly 2–3 minutes (vs ~10+
before). If any test fails, its tamper/assert logic depended on something in
the arrange block — inspect that one test; do not weaken assertions.

- [ ] **Step 4: Commit**

```bash
cargo fmt
git add crates/pleiades-validate/src/tests/release_bundle_verify_b.rs
git commit -m "test(validate): stage release-bundle copies instead of regenerating in verify_b"
```

---

### Task 3: Bulk-convert inline generations in `release_bundle_verify_a.rs` (tamper tests only)

**Files:**
- Modify: `crates/pleiades-validate/src/tests/release_bundle_verify_a.rs`

**Interfaces:**
- Consumes: `stage_bundle_copy` from Task 1.
- Produces: nothing new.

Two tests in this file are intentionally **not** converted here:
`release_bundle_commands_accept_output_aliases_in_the_validation_front_end`
(lines ~9–52: it exercises `bundle-release` argument parsing, so it keeps its
real generations — leave the whole test untouched) and
`release_bundle_writes_expected_artifacts` (converted in Task 4). The script
below cannot match the alias test (its `render_cli` calls use `--output` /
mixed aliases and different `.expect` messages), and it converts
`release_bundle_writes_expected_artifacts` only if its arrange block matches —
it does not (its render call binds `let rendered =`), so it is safe to run on
the whole file.

- [ ] **Step 1: Apply the same mechanical conversion as Task 2**

```bash
python3 - <<'EOF'
import pathlib
import re

block = re.compile(
    r'    let bundle_dir = unique_temp_dir\("([^"]+)"\);\n'
    r'    let bundle_dir_string = bundle_dir\.to_string_lossy\(\)\.to_string\(\);\n'
    r'    render_cli\(&\[\n'
    r'        "bundle-release",\n'
    r'        "--out",\n'
    r'        &bundle_dir_string,\n'
    r'        "--rounds",\n'
    r'        "1",\n'
    r'    \]\)\n'
    r'    \.expect\("bundle release should render"\);\n'
)
repl = (
    '    let bundle_dir = stage_bundle_copy("\\1");\n'
    '    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();\n'
)
path = pathlib.Path("crates/pleiades-validate/src/tests/release_bundle_verify_a.rs")
text = path.read_text()
text, count = block.subn(repl, text)
path.write_text(text)
print(f"converted {count} sites")
EOF
```

Expected output: `converted N sites` where N ≥ 10. Verify completeness in Step 2.

- [ ] **Step 2: Verify the only remaining `"bundle-release"` sites are the alias test and `writes_expected_artifacts`**

```bash
grep -n '"bundle-release"' crates/pleiades-validate/src/tests/release_bundle_verify_a.rs
```

Expected: exactly three matches — two inside
`release_bundle_commands_accept_output_aliases_in_the_validation_front_end`
(the `--output` generation and the mixed-alias error case) and one inside
`release_bundle_writes_expected_artifacts` (its `let rendered = render_cli(...)`
arrange, converted in Task 4). Any other match is an inline site the script
missed — convert it by hand to the two-line `stage_bundle_copy` form.

- [ ] **Step 3: Run the whole verify_a module**

```bash
cargo test -p pleiades-validate --lib -- --include-ignored tests::release_bundle_verify_a
```

Expected: `50 passed; 0 failed` (the unconverted tests still pass — they still
generate for real).

- [ ] **Step 4: Commit**

```bash
cargo fmt
git add crates/pleiades-validate/src/tests/release_bundle_verify_a.rs
git commit -m "test(validate): stage release-bundle copies instead of regenerating in verify_a"
```

---

### Task 4: Convert the generation-asserting tests to consume the fixture directly

**Files:**
- Modify: `crates/pleiades-validate/src/tests/release_bundle_verify_a.rs`

**Interfaces:**
- Consumes: `pristine_release_bundle() -> &'static PristineBundle` from Task 1 (fields `.dir`, `.rendered`, `.bundle`).
- Produces: nothing new.

- [ ] **Step 1: Convert `release_bundle_writes_expected_artifacts`**

At the top of the test (currently lines ~71–80), replace:

```rust
    let bundle_dir = unique_temp_dir("pleiades-release-bundle");
    let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
    let rendered = render_cli(&[
        "bundle-release",
        "--out",
        &bundle_dir_string,
        "--rounds",
        "1",
    ])
    .expect("bundle release should render");
```

with:

```rust
    let pristine = pristine_release_bundle();
    let bundle_dir = pristine.dir.clone();
    let rendered = pristine.rendered.clone();
```

(`bundle_dir_string` was only used by the removed `render_cli` call.) At the
**end of the same test** (currently line ~1562), delete its cleanup line —
this test now points at the pristine dir, which must survive:

```rust
    let _ = std::fs::remove_dir_all(&bundle_dir);
```

Every assertion in between is unchanged: the test only reads files.

- [ ] **Step 2: Convert the five `release_bundle_validate_*` struct tests**

For `release_bundle_validate_accepts_rendered_bundle` (line ~2330), replace:

```rust
    let bundle_dir = unique_temp_dir("pleiades-release-bundle-validate-accepts");
    let bundle = render_release_bundle(1, &bundle_dir).expect("release bundle should render");
```

with:

```rust
    let bundle = pristine_release_bundle().bundle.clone();
```

and delete its trailing `let _ = std::fs::remove_dir_all(&bundle_dir);`.

For the four tamper variants
(`release_bundle_validate_rejects_whitespace_padded_provenance`,
`release_bundle_validate_rejects_placeholder_provenance`,
`release_bundle_validate_rejects_multiline_provenance`,
`release_bundle_validate_rejects_manifest_path_drift`), replace their
two-line arrange (each has its own `unique_temp_dir` prefix) with:

```rust
    let mut bundle = pristine_release_bundle().bundle.clone();
```

and delete each trailing `let _ = std::fs::remove_dir_all(&bundle_dir);`.
All tampering is in-memory struct mutation (`bundle.source_revision`,
`bundle.rustc_version`, `bundle.workspace_status`, `bundle.manifest_path`);
no file under the pristine dir is touched. Assertions unchanged.

- [ ] **Step 3: Run the six converted tests**

```bash
cargo test -p pleiades-validate --lib -- --include-ignored \
  tests::release_bundle_verify_a::release_bundle_writes_expected_artifacts \
  tests::release_bundle_verify_a::release_bundle_validate
```

Expected: `6 passed; 0 failed`.

- [ ] **Step 4: Commit**

```bash
cargo fmt
git add crates/pleiades-validate/src/tests/release_bundle_verify_a.rs
git commit -m "test(validate): assert bundle generation against the pristine fixture"
```

---

### Task 5: Same pattern in pleiades-cli

**Files:**
- Modify: `crates/pleiades-cli/src/cli/test_support.rs`
- Modify: `crates/pleiades-cli/src/cli/tests/release.rs`

**Interfaces:**
- Consumes: `crate::cli::render_cli(args: &[&str]) -> Result<String, String>` (existing).
- Produces (CLI-crate only): `pub(crate) struct PristineBundle { pub(crate) dir: PathBuf, pub(crate) rendered: String }` and `pub(crate) fn pristine_release_bundle() -> &'static PristineBundle` in `crate::cli::test_support`. (No `bundle` field and no `stage_bundle_copy` here — no CLI test tampers with bundle files. YAGNI.)

- [ ] **Step 1: Add the fixture to `crates/pleiades-cli/src/cli/test_support.rs`**

Append:

```rust
/// One pristine release bundle per test process (default benchmark rounds,
/// matching what the release-command tests assert). Never mutate `dir`.
pub(crate) struct PristineBundle {
    pub(crate) dir: std::path::PathBuf,
    pub(crate) rendered: String,
}

pub(crate) fn pristine_release_bundle() -> &'static PristineBundle {
    static PRISTINE: std::sync::OnceLock<PristineBundle> = std::sync::OnceLock::new();
    PRISTINE.get_or_init(|| {
        let dir = unique_temp_dir("pleiades-cli-release-bundle-pristine");
        let dir_string = dir.display().to_string();
        let rendered = crate::cli::render_cli(&["bundle-release", "--out", &dir_string])
            .expect("pristine release bundle fixture should render");
        PristineBundle { dir, rendered }
    })
}
```

- [ ] **Step 2: Convert the two staged-bundle tests in `release.rs`**

Update the import at the top of the file:

```rust
use super::super::test_support::{pristine_release_bundle, unique_temp_dir};
```

In `bundle_release_command_writes_a_staged_bundle`, replace:

```rust
    let bundle_dir = unique_temp_dir("pleiades-cli-release-bundle");
    let bundle_dir_string = bundle_dir.display().to_string();

    let rendered = render_cli(&["bundle-release", "--out", &bundle_dir_string])
        .expect("bundle generation should render");
```

with:

```rust
    let pristine = pristine_release_bundle();
    let bundle_dir = pristine.dir.clone();
    let rendered = pristine.rendered.clone();
```

(the rest of the test only reads `bundle_dir` files and asserts on `rendered`;
it has no cleanup line).

In `verify_release_bundle_command_verifies_a_staged_bundle`, replace:

```rust
    let bundle_dir = unique_temp_dir("pleiades-cli-release-bundle");
    let bundle_dir_string = bundle_dir.display().to_string();

    render_cli(&["bundle-release", "--out", &bundle_dir_string])
        .expect("bundle generation should succeed");
```

with:

```rust
    let bundle_dir_string = pristine_release_bundle().dir.display().to_string();
```

(verification is read-only, so it may point at the pristine dir directly).

Leave `bundle_release_commands_reject_duplicate_output_arguments` (front-end
errors, no generation) and `bundle_release_commands_accept_output_alias` (the
CLI crate's kept real generation) untouched.

- [ ] **Step 3: Run the CLI release tests**

```bash
cargo test -p pleiades-cli -- --include-ignored cli::tests::release
```

Expected: `4 passed; 0 failed`. The default-rounds warm-up makes this run
~2–4 minutes; that is expected.

- [ ] **Step 4: Commit**

```bash
cargo fmt
git add crates/pleiades-cli/src/cli/test_support.rs crates/pleiades-cli/src/cli/tests/release.rs
git commit -m "test(cli): share one pristine release bundle across release-command tests"
```

---

### Task 6: Full-family verification, timing record, spec status

**Files:**
- Modify: `docs/superpowers/specs/2026-07-02-release-bundle-test-fixture-design.md` (status line + measured result)

**Interfaces:**
- Consumes: everything above.
- Produces: recorded before/after numbers.

- [ ] **Step 1: Audit that only the sanctioned generation sites remain**

```bash
grep -rn '"bundle-release"' crates/pleiades-validate/src/tests/ crates/pleiades-cli/src/cli/tests/ crates/pleiades-cli/src/cli/test_support.rs
grep -rn 'render_release_bundle(' crates/pleiades-validate/src/tests/
```

Expected generation sites only:
- `pleiades-validate` `test_support.rs`: the fixture's `render_release_bundle(1, &dir)`.
- `pleiades-validate` `release_bundle_verify_a.rs`: the alias front-end test (plus its mixed-alias error case, which never generates).
- `pleiades-validate` `render_request.rs`: front-end `expect_err` cases only (zero rounds / duplicate args — they never generate; leave as-is).
- `pleiades-cli` `test_support.rs`: the fixture.
- `pleiades-cli` `release.rs`: the alias test and the duplicate-arguments error test (never generates).

- [ ] **Step 2: Run and time the full validate family**

```bash
time cargo test -p pleiades-validate --lib -- --include-ignored release_bundle
```

Expected: `172 passed; 0 failed`, wall time roughly 2–3 minutes (baseline
2026-07-02 on this machine: 906 s / ~79 CPU-min). If wall time exceeds ~5
minutes, something still regenerates — go back to the Step 1 audit.

- [ ] **Step 3: Run the full test-full gate**

```bash
mise run test-full
```

Expected: passes. This is the CI-equivalent gate; it is long (tens of
minutes) — let it finish.

- [ ] **Step 4: Record the outcome and flip the spec status**

In `docs/superpowers/specs/2026-07-02-release-bundle-test-fixture-design.md`,
change the status line to:

```markdown
Status: implemented (2026-07-02). Plan: docs/superpowers/plans/2026-07-02-release-bundle-test-fixture.md
```

and append the measured before/after wall time from Step 2 under
"Expected outcome" as a short "Measured result (2026-07-02): …" line with the
actual numbers observed.

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/specs/2026-07-02-release-bundle-test-fixture-design.md
git commit -m "docs(specs): record release-bundle fixture results"
```
