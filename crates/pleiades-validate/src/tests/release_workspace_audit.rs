//! workspace audit and provenance tests (white-box; moved verbatim from the former `tests.rs`).

use super::test_support::*;
use super::*;
use std::path::Path;

#[test]
fn workspace_audit_reports_a_clean_workspace() {
    let report = workspace_audit_report().expect("workspace audit should render");
    assert!(report.is_clean());
    assert!(report
        .to_string()
        .contains("no workspace policy violations detected"));
    assert!(report.to_string().contains("Checked manifests:"));
    assert!(report.to_string().contains("Checked tool manifest:"));
}

#[test]
fn workspace_audit_summary_reports_a_clean_workspace() {
    let report = workspace_audit_report().expect("workspace audit should render");
    let summary = workspace_audit_summary(&report);
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert!(summary.summary_line().contains("violations: 0"));
    assert!(summary
        .summary_line()
        .contains("no workspace policy violations detected"));

    let rendered =
        render_cli(&["workspace-audit-summary"]).expect("workspace audit summary should render");
    let native_dependency_rendered = render_native_dependency_audit_summary()
        .expect("native dependency audit summary should render");
    let alias = render_cli(&["native-dependency-audit-summary"])
        .expect("native dependency audit summary should render");
    assert_eq!(rendered, native_dependency_rendered);
    assert_eq!(rendered, alias);
    assert!(rendered.contains("Workspace audit summary"));
    assert!(rendered.contains("Summary: workspace root:"));
    assert!(rendered.contains("Checked manifests:"));
    assert!(rendered.contains("Checked tool manifest:"));
    assert!(rendered.contains("Checked lockfile:"));
    assert!(rendered.contains("Result: no workspace policy violations detected"));

    assert_eq!(
        render_cli(&["workspace-audit", "extra"]).unwrap_err(),
        "workspace-audit does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["audit", "extra"]).unwrap_err(),
        "audit does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["native-dependency-audit", "extra"]).unwrap_err(),
        "native-dependency-audit does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["workspace-audit-summary", "extra"]).unwrap_err(),
        "workspace-audit-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["native-dependency-audit-summary", "extra"]).unwrap_err(),
        "native-dependency-audit-summary does not accept extra arguments"
    );
}

#[test]
fn workspace_provenance_summary_reports_workspace_tool_versions() {
    let summary = workspace_provenance_summary_for_report();
    assert!(summary.contains("Workspace provenance"));
    assert!(summary.contains("source revision:"));
    assert!(summary.contains("workspace status:"));
    assert!(summary.contains("rustc version:"));
    assert!(summary.contains("cargo version:"));
    assert!(summary.contains("rustfmt version:"));
    assert!(summary.contains("clippy version:"));

    let rendered = render_cli(&["workspace-provenance-summary"])
        .expect("workspace provenance summary should render");
    let alias =
        render_cli(&["workspace-provenance"]).expect("workspace provenance alias should render");
    assert_eq!(rendered, summary);
    assert_eq!(rendered, alias);
    assert_eq!(
        render_cli(&["workspace-provenance-summary", "extra"]).unwrap_err(),
        "workspace-provenance-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["workspace-provenance", "extra"]).unwrap_err(),
        "workspace-provenance does not accept extra arguments"
    );
}

#[test]
fn workspace_audit_detects_native_hooks_in_manifests_and_lockfile() {
    let manifest = r#"[package]
name = "example"
build = "build.rs"
links = "example-native"

[dependencies]
cc = "1"
openssl-sys = "0.9"
[target.'cfg(unix)'.dependencies]
bindgen = { version = "0.69" }
renamed-native = { package = "zstd-sys", version = "2" }
"#;
    let manifest_violations = audit_manifest_text(Path::new("/tmp/Cargo.toml"), manifest);
    assert!(manifest_violations
        .iter()
        .any(|violation| violation.rule == "package.build"));
    assert!(manifest_violations
        .iter()
        .any(|violation| violation.rule == "package.links"));
    assert!(manifest_violations
        .iter()
        .any(|violation| violation.detail.contains("cc")));
    assert!(manifest_violations
        .iter()
        .any(|violation| violation.detail.contains("bindgen")));
    assert!(manifest_violations
        .iter()
        .any(|violation| violation.rule == "dependency.native-package"
            && violation.detail.contains("openssl-sys")));
    assert!(manifest_violations
        .iter()
        .any(|violation| violation.rule == "dependency.native-package"
            && violation.detail.contains("zstd-sys")));

    let build_script_dir = unique_temp_dir("pleiades-workspace-audit-build-script");
    let build_script_manifest = build_script_dir.join("Cargo.toml");
    let build_script_path = build_script_dir.join("build.rs");
    std::fs::write(
        &build_script_manifest,
        "[package]\nname = \"example-build-script\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be writable");
    std::fs::write(&build_script_path, "fn main() {}\n").expect("build.rs should be writable");
    let build_script_violation =
        audit_build_script_path(&build_script_manifest).expect("build.rs should be detected");
    assert_eq!(build_script_violation.rule, "package.build-script");
    assert_eq!(build_script_violation.path, build_script_path);
    assert!(build_script_violation.detail.contains("build.rs"));

    let lockfile = r#"[[package]]
name = "openssl-sys"
version = "0.9.0"
"#;
    let lockfile_violations = audit_lockfile_text(Path::new("/tmp/Cargo.lock"), lockfile);
    assert!(lockfile_violations
        .iter()
        .any(|violation| violation.rule == "lockfile.native-package"));
}

#[test]
fn workspace_audit_exempts_pure_rust_tls_phantom_lockfile_entries() {
    // The optional, default-off `horizons-fetch` TLS stack (ureq + rustls +
    // graviola) compiles no native crypto: `ring`/`aws-lc-rs` are never built
    // (verified via `cargo tree -p pleiades-jpl --features horizons-fetch -i
    // ring` -> "nothing to print"). But because `rustls-webpki` *declares*
    // `ring` as an optional dependency, `ring` (and its build-dep `cc` and
    // `windows-sys`) linger in Cargo.lock as never-compiled phantom entries.
    // They are allowlisted by name; any other native package still fails.
    let phantom = r#"[[package]]
name = "cc"
version = "1.2.0"

[[package]]
name = "ring"
version = "0.17.0"

[[package]]
name = "windows-sys"
version = "0.59.0"
"#;
    let phantom_violations = audit_lockfile_text(Path::new("/tmp/Cargo.lock"), phantom);
    assert!(
        phantom_violations.is_empty(),
        "pure-Rust TLS phantom entries should be exempt, got: {phantom_violations:?}"
    );

    // A genuinely native package (not in the phantom allowlist) must still fail.
    let real_native = r#"[[package]]
name = "openssl-sys"
version = "0.9.0"
"#;
    let real_violations = audit_lockfile_text(Path::new("/tmp/Cargo.lock"), real_native);
    assert!(real_violations
        .iter()
        .any(|violation| violation.rule == "lockfile.native-package"));
}

#[test]
fn workspace_tls_stack_stays_pure_rust() {
    // Static backstop for the `audit_lockfile_text` phantom allowlist: the
    // allowlist is only safe while the `horizons-fetch` TLS stack never
    // *activates* a native crypto provider. ureq must use `rustls-no-provider`
    // (not the `rustls` umbrella, which wires `_ring`), and `rustls` must not
    // enable its `ring`/`aws-lc-rs` features. If a future edit flips any of
    // these, `ring`/`cc` would actually compile and this test fails.
    let workspace_manifest = concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.toml");
    let text = std::fs::read_to_string(workspace_manifest)
        .expect("workspace Cargo.toml should be readable");
    let ureq_line = text
        .lines()
        .find(|line| line.trim_start().starts_with("ureq = "))
        .expect("workspace should declare ureq");
    assert!(
        ureq_line.contains("rustls-no-provider"),
        "ureq must use rustls-no-provider, got: {ureq_line}"
    );
    assert!(
        !ureq_line.contains("\"rustls\""),
        "ureq must not enable the ring-pulling `rustls` umbrella feature, got: {ureq_line}"
    );
    let rustls_line = text
        .lines()
        .find(|line| line.trim_start().starts_with("rustls = "))
        .expect("workspace should declare rustls");
    assert!(
        rustls_line.contains("default-features = false"),
        "rustls must disable default features (which pull aws-lc-rs), got: {rustls_line}"
    );
    assert!(
        !rustls_line.contains("\"ring\"") && !rustls_line.contains("\"aws-lc-rs\""),
        "rustls must not enable a native crypto provider, got: {rustls_line}"
    );
}

#[test]
fn workspace_audit_detects_tool_manifest_provenance_drift() {
    let tool_manifest = r#"[tools]
rust = { version = "1.96.0", components = "rustfmt" }
"#;
    let violations = audit_tool_manifest_text(
        Path::new("/tmp/mise.toml"),
        tool_manifest,
        Some("1.95.0".to_string()),
    );

    assert!(violations
        .iter()
        .any(|violation| violation.rule == "tool-manifest.rust-version-mismatch"));
    assert!(violations
        .iter()
        .any(|violation| violation.rule == "tool-manifest.rust-components-missing"));
}

#[test]
fn workspace_audit_detects_workspace_publish_metadata_drift() {
    let manifest = r#"[workspace.package]
version = "0.1.0"
license = "MIT"

[workspace.dependencies]
pleiades-types = { path = "crates/pleiades-types", version = "0.2.0" }
pleiades-backend = { version = "0.1.0" }
serde = { version = "1" }
"#;
    let violations = audit_workspace_manifest_publish_text(Path::new("/tmp/Cargo.toml"), manifest);

    assert!(violations
        .iter()
        .any(|violation| violation.rule == "publish.workspace-license"));
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.workspace-metadata-missing"
        && violation.detail.contains("repository")));
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.workspace-metadata-missing"
        && violation.detail.contains("keywords")));
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.workspace-dependency-version"
        && violation.detail.contains("pleiades-types")));
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.workspace-dependency-path"
        && violation.detail.contains("pleiades-backend")));
    assert!(!violations
        .iter()
        .any(|violation| violation.detail.contains("`serde`")));
}

#[test]
fn workspace_audit_accepts_publish_ready_workspace_manifest() {
    let manifest = r#"[workspace.package]
version = "0.1.0"
license = "MIT OR Apache-2.0"
repository = "https://github.com/rahulmutt/pleiades"
homepage = "https://github.com/rahulmutt/pleiades"
keywords = ["astrology", "astronomy", "ephemeris"]
categories = ["science"]

[workspace.dependencies]
serde = { version = "1" }
pleiades-types = { path = "crates/pleiades-types", version = "0.1.0" }
"#;
    let violations = audit_workspace_manifest_publish_text(Path::new("/tmp/Cargo.toml"), manifest);
    assert!(
        violations.is_empty(),
        "unexpected violations: {violations:?}"
    );
}

#[test]
fn workspace_audit_reports_missing_workspace_version_once() {
    let manifest = r#"[workspace.package]
license = "MIT OR Apache-2.0"
repository = "https://github.com/rahulmutt/pleiades"
homepage = "https://github.com/rahulmutt/pleiades"
keywords = ["astrology", "astronomy", "ephemeris"]
categories = ["science"]

[workspace.dependencies]
pleiades-types = { path = "crates/pleiades-types", version = "0.1.0" }
pleiades-backend = { path = "crates/pleiades-backend", version = "0.1.0" }
"#;
    let violations = audit_workspace_manifest_publish_text(Path::new("/tmp/Cargo.toml"), manifest);
    let count = violations
        .iter()
        .filter(|violation| violation.rule == "publish.workspace-version-missing")
        .count();
    assert_eq!(count, 1, "violations: {violations:?}");
}

#[test]
fn workspace_audit_identifies_publishable_packages() {
    assert!(manifest_is_package("[package]\nname = \"a\"\n"));
    assert!(!manifest_is_package("[workspace]\nmembers = []\n"));
    assert!(manifest_declares_publish_false(
        "[package]\nname = \"a\"\npublish = false\n"
    ));
    assert!(manifest_declares_publish_false(
        "[package]\nname = \"a\"\npublish = []\n"
    ));
    assert!(!manifest_declares_publish_false(
        "[package]\nname = \"a\"\n"
    ));
    assert_eq!(
        manifest_package_name("[package]\nname = \"pleiades-types\"\n"),
        Some("pleiades-types".to_string())
    );
}

#[test]
fn workspace_audit_detects_publishable_crate_manifest_gaps() {
    let manifest = r#"[package]
name = "pleiades-example"
version.workspace = true
edition.workspace = true

[dependencies]
pleiades-types = { path = "../pleiades-types" }
pleiades-data = { workspace = true }
serde = { workspace = true, optional = true }
renamed = { package = "pleiades-houses", path = "../pleiades-houses" }

[build-dependencies]
pleiades-elp = { path = "../pleiades-elp" }

[dev-dependencies]
pleiades-jpl = { workspace = true }
"#;
    let publishable = vec!["pleiades-example".to_string(), "pleiades-types".to_string()];
    let violations =
        audit_publishable_manifest_text(Path::new("/tmp/Cargo.toml"), manifest, &publishable);

    assert!(violations
        .iter()
        .any(|violation| violation.rule == "publish.description-missing"));
    assert!(violations
        .iter()
        .any(|violation| violation.rule == "publish.license-not-inherited"));
    assert!(violations
        .iter()
        .any(|violation| violation.rule == "publish.readme-field-missing"));
    assert!(violations.iter().any(
        |violation| violation.rule == "publish.metadata-field-missing"
            && violation.detail.contains("repository")
    ));
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.internal-dependency-not-workspace"
        && violation.detail.contains("pleiades-types")));
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.internal-dependency-unpublishable"
        && violation.detail.contains("pleiades-data")));
    assert!(!violations
        .iter()
        .any(|violation| violation.detail.contains("`serde`")));
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.internal-dependency-not-workspace"
        && violation.detail.contains("pleiades-houses")));
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.internal-dependency-unpublishable"
        && violation.detail.contains("pleiades-elp")));
    assert!(!violations.iter().any(|violation| violation.rule
        == "publish.internal-dependency-unpublishable"
        && violation.detail.contains("pleiades-jpl")));
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.internal-dev-dependency-not-path-only"
        && violation.detail.contains("pleiades-jpl")));
    assert!(!violations.iter().any(|violation| violation.rule
        == "publish.internal-dependency-not-workspace"
        && violation.detail.contains("pleiades-jpl")));
}

#[test]
fn workspace_audit_requires_path_only_internal_dev_dependencies() {
    let manifest = r#"[package]
name = "pleiades-example"
version.workspace = true
edition.workspace = true

[dependencies]
pleiades-types = { workspace = true }

[dev-dependencies]
pleiades-data = { workspace = true }
pleiades-elp = { path = "../pleiades-elp" }
pleiades-jpl = { path = "../pleiades-jpl", version = "0.4.0" }
serde_json = "1"
"#;
    let publishable = vec!["pleiades-example".to_string(), "pleiades-types".to_string()];
    let violations =
        audit_publishable_manifest_text(Path::new("/tmp/Cargo.toml"), manifest, &publishable);

    // workspace = true dev-dep must violate the new path-only rule.
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.internal-dev-dependency-not-path-only"
        && violation.detail.contains("pleiades-data")));
    // ...and must NOT also trip the runtime-only "not-workspace" rule.
    assert!(!violations.iter().any(|violation| violation.rule
        == "publish.internal-dependency-not-workspace"
        && violation.detail.contains("pleiades-data")));

    // path-only dev-dep is compliant: no violation for pleiades-elp at all.
    assert!(!violations
        .iter()
        .any(|violation| violation.detail.contains("pleiades-elp")));

    // path + version dev-dep must still violate: version defeats path-only stripping.
    assert!(violations.iter().any(|violation| violation.rule
        == "publish.internal-dev-dependency-not-path-only"
        && violation.detail.contains("pleiades-jpl")));

    // external dev-dep is untouched.
    assert!(!violations
        .iter()
        .any(|violation| violation.detail.contains("serde_json")));
}

#[test]
fn workspace_audit_accepts_publish_ready_crate_manifest() {
    let manifest = r#"[package]
name = "pleiades-example"
description = "Example publishable crate."
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
repository.workspace = true
homepage.workspace = true
keywords.workspace = true
categories.workspace = true
readme = "README.md"

[dependencies]
pleiades-types = { workspace = true }

[dev-dependencies]
serde_json = "1"
pleiades-jpl = { path = "../pleiades-jpl" }
"#;
    let publishable = vec!["pleiades-example".to_string(), "pleiades-types".to_string()];
    let violations =
        audit_publishable_manifest_text(Path::new("/tmp/Cargo.toml"), manifest, &publishable);
    assert!(
        violations.is_empty(),
        "unexpected violations: {violations:?}"
    );
}

#[test]
fn workspace_audit_detects_publish_file_gaps() {
    let root = unique_temp_dir("pleiades-publish-file-audit");
    let crate_dir = root.join("crates").join("pleiades-example");
    std::fs::create_dir_all(&crate_dir).expect("crate dir should be creatable");
    std::fs::write(root.join("LICENSE-APACHE"), "apache text")
        .expect("root apache license should be writable");
    std::fs::write(root.join("LICENSE-MIT"), "mit text")
        .expect("root mit license should be writable");
    std::fs::write(crate_dir.join("LICENSE-APACHE"), "apache text")
        .expect("crate apache license should be writable");
    std::fs::write(crate_dir.join("LICENSE-MIT"), "different text")
        .expect("crate mit license should be writable");

    let violations = audit_publishable_crate_files(&crate_dir.join("Cargo.toml"), &root);
    assert!(violations
        .iter()
        .any(|violation| violation.rule == "publish.readme-file-missing"));
    assert!(violations
        .iter()
        .any(|violation| violation.rule == "publish.license-file-drift"
            && violation.detail.contains("LICENSE-MIT")));
    assert!(!violations
        .iter()
        .any(|violation| violation.rule == "publish.license-file-missing"));

    std::fs::write(crate_dir.join("README.md"), "# pleiades-example\n")
        .expect("crate readme should be writable");
    std::fs::write(crate_dir.join("LICENSE-MIT"), "mit text")
        .expect("crate mit license should be writable");
    let violations = audit_publishable_crate_files(&crate_dir.join("Cargo.toml"), &root);
    assert!(
        violations.is_empty(),
        "unexpected violations: {violations:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn workspace_audit_summary_groups_rule_counts_for_violations() {
    let report = WorkspaceAuditReport {
            workspace_root: PathBuf::from("/workspace"),
            manifest_paths: vec![PathBuf::from("/workspace/Cargo.toml")],
            tool_manifest_path: PathBuf::from("/workspace/mise.toml"),
            lockfile_path: PathBuf::from("/workspace/Cargo.lock"),
            violations: vec![
                WorkspaceAuditViolation {
                    path: PathBuf::from("/workspace/Cargo.toml"),
                    rule: "package.build",
                    detail: "package declares a build script".to_string(),
                },
                WorkspaceAuditViolation {
                    path: PathBuf::from("/workspace/Cargo.toml"),
                    rule: "package.build",
                    detail: "package declares a build script".to_string(),
                },
                WorkspaceAuditViolation {
                    path: PathBuf::from("/workspace/Cargo.lock"),
                    rule: "lockfile.native-package",
                    detail: "lockfile package `openssl-sys` suggests a native build dependency and should be reviewed".to_string(),
                },
            ],
        };

    let summary = workspace_audit_summary(&report);
    assert!(summary.summary_line().contains("violations: 3"));
    assert_eq!(
        summary.rule_counts,
        vec![("lockfile.native-package", 1), ("package.build", 2)]
    );
    summary
        .validate()
        .expect("workspace audit summary should validate");

    let rendered = render_workspace_audit_summary_text(&report);
    assert!(rendered.contains("Summary: workspace root:"));
    assert!(rendered.contains("tool manifest:"));
    assert!(rendered.contains("Violations: 3"));
    assert!(rendered.contains("Rule counts:"));
    assert!(rendered.contains("package.build: 2"));
    assert!(rendered.contains("lockfile.native-package: 1"));
    assert!(rendered.contains("Result: violations found"));

    let display = report.to_string();
    assert!(display.contains("Rule counts:"));
    assert!(display.contains("package.build: 2"));
    assert!(display.contains("lockfile.native-package: 1"));
}

#[test]
fn workspace_audit_summary_validate_rejects_incoherent_counts() {
    let summary = WorkspaceAuditSummary {
        workspace_root: PathBuf::from("/workspace"),
        manifest_count: 1,
        tool_manifest_path: PathBuf::from("/workspace/mise.toml"),
        lockfile_path: PathBuf::from("/workspace/Cargo.lock"),
        violation_count: 2,
        rule_counts: vec![("package.build", 1)],
        clean: false,
    };

    let error = summary
        .validate()
        .expect_err("incoherent workspace audit summary should fail validation");
    assert!(error
        .to_string()
        .contains("workspace audit summary violation count mismatch"));
}
