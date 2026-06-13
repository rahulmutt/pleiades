//! Shared white-box test fixtures and helpers for the validate crate.
//!
//! Extracted verbatim from the former `tests.rs` monolith so the
//! per-module test files can reuse the common arrange blocks.

use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    let unique = format!(
        "{}-{}-{}-{}",
        prefix,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after UNIX_EPOCH")
            .as_nanos(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed),
    );
    let path = std::env::temp_dir().join(unique);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("temporary directory should be creatable");
    path
}

pub(crate) fn assert_report_contains_exact_line(report: &str, expected: &str) {
    let expected = expected.trim_start();
    assert!(
        report.lines().any(|line| line.trim_start() == expected),
        "expected report to contain line `{expected}`\nreport:\n{report}"
    );
}

pub(crate) fn assert_release_bundle_rejects_tampered_text_file(
    bundle_dir_prefix: &str,
    file_name: &str,
    expected_fragment: &str,
) {
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

    let file_path = bundle_dir.join(file_name);
    let mut text = std::fs::read_to_string(&file_path)
        .unwrap_or_else(|error| panic!("{file_name} should exist: {error}"));
    text.push_str("\nTampered for regression coverage.\n");
    std::fs::write(&file_path, text)
        .unwrap_or_else(|error| panic!("{file_name} should be writable: {error}"));

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a tampered release bundle file");
    assert!(
        error.contains("release bundle verification failed") || error.contains(expected_fragment),
        "unexpected error: {error}"
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

pub(crate) fn assert_release_bundle_rejects_semantically_tampered_text_file_with_updated_checksum(
    bundle_dir_prefix: &str,
    file_name: &str,
    manifest_checksum_prefix: &str,
    from: &str,
    to: &str,
    expected_fragment: &str,
) {
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

    let file_path = bundle_dir.join(file_name);
    let original = std::fs::read_to_string(&file_path)
        .unwrap_or_else(|error| panic!("{file_name} should exist: {error}"));
    let tampered = original.replace(from, to);
    assert_ne!(
        original, tampered,
        "{file_name} should be changed by the regression edit"
    );
    std::fs::write(&file_path, &tampered)
        .unwrap_or_else(|error| panic!("{file_name} should be writable: {error}"));

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let old_checksum_line = manifest
        .lines()
        .find(|line| line.starts_with(manifest_checksum_prefix))
        .unwrap_or_else(|| panic!("manifest should contain the {manifest_checksum_prefix} line"));
    let new_checksum_line = format!(
        "{manifest_checksum_prefix} 0x{:016x}",
        checksum64(&tampered)
    );
    let updated_manifest = manifest.replacen(old_checksum_line, &new_checksum_line, 1);
    std::fs::write(&manifest_path, &updated_manifest).expect("manifest should be writable");

    let checksum_path = bundle_dir.join("bundle-manifest.checksum.txt");
    std::fs::write(
        &checksum_path,
        format!("0x{:016x}\n", checksum64(&updated_manifest)),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for semantic release bundle drift");
    assert!(
        error.contains("release bundle verification failed") || error.contains(expected_fragment),
        "unexpected error: {error}"
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

pub(crate) fn assert_release_bundle_rejects_symlinked_text_file(
    bundle_dir_prefix: &str,
    file_name: &str,
    link_target: &str,
    expected_fragment: &str,
) {
    use std::os::unix::fs::symlink;

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

    let file_path = bundle_dir.join(file_name);
    std::fs::remove_file(&file_path).expect("bundled text file should be removable");
    symlink(link_target, &file_path).expect("symlink should be creatable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a symlinked release bundle file");
    assert!(
        error.contains("release bundle verification failed") || error.contains(expected_fragment),
        "unexpected error: {error}"
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

pub(crate) fn assert_release_bundle_rejects_missing_manifest_entry(
    bundle_dir_prefix: &str,
    manifest_line_prefix: &str,
    expected_fragments: &[&str],
) {
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

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let filtered = manifest
        .lines()
        .filter(|line| !line.starts_with(manifest_line_prefix))
        .map(str::to_owned)
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&manifest_path, format!("{filtered}\n")).expect("manifest should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a manifest missing the requested entry");
    assert!(
        expected_fragments
            .iter()
            .any(|fragment| error.contains(fragment))
            || error.contains("unexpected release bundle manifest line count"),
        "unexpected error: {error}"
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

pub(crate) fn assert_release_bundle_rejects_blank_manifest_value(
    bundle_dir_prefix: &str,
    manifest_line_prefix: &str,
    expected_fragments: &[&str],
) {
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

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let rewritten = manifest
        .lines()
        .map(|line| {
            if line.starts_with(manifest_line_prefix) {
                manifest_line_prefix.to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&manifest_path, format!("{rewritten}\n")).expect("manifest should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a manifest with a blank requested entry");
    assert!(
        expected_fragments
            .iter()
            .any(|fragment| error.contains(fragment)),
        "unexpected error: {error}"
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

pub(crate) fn assert_release_bundle_rejects_duplicate_manifest_entry(
    bundle_dir_prefix: &str,
    manifest_line_prefix: &str,
    expected_fragments: &[&str],
) {
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

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let duplicate_line = manifest
        .lines()
        .find(|line| line.starts_with(manifest_line_prefix))
        .unwrap_or_else(|| panic!("{manifest_line_prefix} should exist"));
    let mut lines = manifest.lines().map(str::to_owned).collect::<Vec<_>>();
    lines.push(duplicate_line.to_string());
    std::fs::write(&manifest_path, format!("{}\n", lines.join("\n")))
        .expect("manifest should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a manifest with a duplicate requested entry");
    assert!(
        expected_fragments
            .iter()
            .any(|fragment| error.contains(fragment)),
        "unexpected error: {error}"
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}

pub(crate) fn assert_release_bundle_rejects_whitespace_manifest_entry(
    bundle_dir_prefix: &str,
    manifest_line_prefix: &str,
    expected_fragments: &[&str],
) {
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

    let manifest_path = bundle_dir.join("bundle-manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).expect("manifest should exist");
    let rewritten = manifest
        .lines()
        .map(|line| {
            if line.starts_with(manifest_line_prefix) {
                format!("{line} ")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&manifest_path, format!("{rewritten}\n")).expect("manifest should be writable");

    let checksum = checksum64(
        &std::fs::read_to_string(&manifest_path).expect("manifest should exist after rewrite"),
    );
    std::fs::write(
        bundle_dir.join("bundle-manifest.checksum.txt"),
        format!("0x{checksum:016x}\n"),
    )
    .expect("manifest checksum sidecar should be writable");

    let error = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
        .expect_err("verification should fail for a manifest with noncanonical whitespace");
    assert!(
        expected_fragments
            .iter()
            .any(|fragment| error.contains(fragment)),
        "unexpected error: {error}"
    );

    let _ = std::fs::remove_dir_all(&bundle_dir);
}
