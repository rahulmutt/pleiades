//! Packaged-artifact command: generate and check the packaged data artifact.

use pleiades_data::packaged_artifact_generation_manifest;
use pleiades_validate::{
    packaged_artifact_normalized_intermediate_summary_for_report,
    packaged_artifact_regeneration_summary_for_report,
};

pub(crate) enum PackagedArtifactCommand {
    Write {
        output_path: String,
        manifest_path: Option<String>,
        manifest_summary_path: Option<String>,
        manifest_checksum_path: Option<String>,
        artifact_checksum_path: Option<String>,
        normalized_intermediate_path: Option<String>,
    },
    Check,
}

pub(crate) fn parse_packaged_artifact_command(
    args: &[&str],
) -> Result<PackagedArtifactCommand, String> {
    if args.is_empty() {
        return Err(
            "missing required output path argument; pass a file path, --out <file>, --output <file>, --manifest-out <file>, --manifest-summary-out <file>, --manifest-checksum-out <file>, --artifact-checksum-out <file>, --normalized-intermediate-summary-out <file>, or --check"
                .to_string(),
        );
    }

    let mut output_path = None;
    let mut manifest_path = None;
    let mut manifest_summary_path = None;
    let mut manifest_checksum_path = None;
    let mut artifact_checksum_path = None;
    let mut normalized_intermediate_path = None;
    let mut check = false;
    let mut iter = args.iter().copied();

    while let Some(arg) = iter.next() {
        match arg {
            "--check" => {
                check = true;
            }
            "--out" | "--output" => {
                let path = iter
                    .next()
                    .ok_or_else(|| format!("missing value for {arg}"))?;
                if output_path.replace(path.to_string()).is_some() {
                    return Err(format!("duplicate output path argument: {arg}"));
                }
            }
            "--manifest-out" => {
                let path = iter
                    .next()
                    .ok_or_else(|| "missing value for --manifest-out".to_string())?;
                if manifest_path.replace(path.to_string()).is_some() {
                    return Err("duplicate manifest path argument: --manifest-out".to_string());
                }
            }
            "--manifest-summary-out" => {
                let path = iter
                    .next()
                    .ok_or_else(|| "missing value for --manifest-summary-out".to_string())?;
                if manifest_summary_path.replace(path.to_string()).is_some() {
                    return Err(
                        "duplicate manifest summary path argument: --manifest-summary-out"
                            .to_string(),
                    );
                }
            }
            "--manifest-checksum-out" => {
                let path = iter
                    .next()
                    .ok_or_else(|| "missing value for --manifest-checksum-out".to_string())?;
                if manifest_checksum_path.replace(path.to_string()).is_some() {
                    return Err(
                        "duplicate manifest checksum path argument: --manifest-checksum-out"
                            .to_string(),
                    );
                }
            }
            "--artifact-checksum-out" => {
                let path = iter
                    .next()
                    .ok_or_else(|| "missing value for --artifact-checksum-out".to_string())?;
                if artifact_checksum_path.replace(path.to_string()).is_some() {
                    return Err(
                        "duplicate artifact checksum path argument: --artifact-checksum-out"
                            .to_string(),
                    );
                }
            }
            "--normalized-intermediate-summary-out" => {
                let path = iter.next().ok_or_else(|| {
                    "missing value for --normalized-intermediate-summary-out".to_string()
                })?;
                if normalized_intermediate_path
                    .replace(path.to_string())
                    .is_some()
                {
                    return Err(
                        "duplicate normalized intermediate path argument: --normalized-intermediate-summary-out"
                            .to_string(),
                    );
                }
            }
            other if other.starts_with('-') => return Err(format!("unknown argument: {other}")),
            path => {
                if output_path.replace(path.to_string()).is_some() {
                    return Err(format!("unexpected positional output path: {path}"));
                }
            }
        }
    }

    if check {
        if output_path.is_some()
            || manifest_path.is_some()
            || manifest_summary_path.is_some()
            || manifest_checksum_path.is_some()
            || artifact_checksum_path.is_some()
            || normalized_intermediate_path.is_some()
        {
            return Err("the --check flag cannot be combined with output paths".to_string());
        }
        return Ok(PackagedArtifactCommand::Check);
    }

    let output_path = output_path.ok_or_else(|| {
        "missing required output path argument; pass a file path, --out <file>, --output <file>, --manifest-out <file>, --manifest-summary-out <file>, --manifest-checksum-out <file>, --artifact-checksum-out <file>, --normalized-intermediate-summary-out <file>, or --check"
            .to_string()
    })?;

    Ok(PackagedArtifactCommand::Write {
        output_path,
        manifest_path,
        manifest_summary_path,
        manifest_checksum_path,
        artifact_checksum_path,
        normalized_intermediate_path,
    })
}

pub(crate) fn render_packaged_artifact_regeneration(
    output_path: String,
    manifest_path: Option<String>,
    manifest_summary_path: Option<String>,
    manifest_checksum_path: Option<String>,
    artifact_checksum_path: Option<String>,
    normalized_intermediate_path: Option<String>,
) -> Result<String, String> {
    // The WRITE path is kernel-gated: regenerating artifact bytes requires the
    // de440 kernel. Kernel-free callers must use the committed artifact via the
    // `packaged-artifact` decode path (and `--check`), not this write path.
    let kernel_path = std::env::var("PLEIADES_DE_KERNEL").map_err(|_| {
        "generate-packaged-artifact requires PLEIADES_DE_KERNEL (path to de440.bsp); \
         kernel-free callers use the committed artifact via packaged-artifact decode"
            .to_string()
    })?;
    let artifact = pleiades_data::regenerate_packaged_artifact_from_kernel(&kernel_path)?;
    let encoded = artifact
        .encode()
        .map_err(|error| format!("failed to encode regenerated packaged artifact: {error}"))?;
    let encoded = encoded.as_slice();
    if let Some(parent) = std::path::Path::new(&output_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
    }
    std::fs::write(&output_path, encoded)
        .map_err(|error| format!("failed to write {}: {error}", output_path))?;

    let manifest = if manifest_path.is_some()
        || manifest_summary_path.is_some()
        || manifest_checksum_path.is_some()
    {
        Some(packaged_artifact_generation_manifest().to_string())
    } else {
        None
    };
    let normalized_intermediate = if normalized_intermediate_path.is_some() {
        Some(packaged_artifact_normalized_intermediate_summary_for_report())
    } else {
        None
    };

    let manifest_line = if let Some(manifest_path) = manifest_path {
        let manifest_text = manifest
            .as_deref()
            .expect("manifest text should be available when a manifest path is requested");
        write_text_file(&manifest_path, manifest_text)?;
        format!("\n  manifest: {}", manifest_path)
    } else {
        String::new()
    };

    let manifest_summary_line = if let Some(manifest_summary_path) = manifest_summary_path {
        let manifest_text = manifest
            .as_deref()
            .expect("manifest text should be available when a manifest summary path is requested");
        write_text_file(&manifest_summary_path, manifest_text)?;
        format!("\n  manifest summary sidecar: {}", manifest_summary_path)
    } else {
        String::new()
    };

    let manifest_checksum_line = if let Some(manifest_checksum_path) = manifest_checksum_path {
        let manifest_text = manifest
            .as_deref()
            .expect("manifest text should be available when a manifest checksum path is requested");
        let checksum_text = format!("0x{:016x}\n", checksum64(manifest_text));
        write_text_file(&manifest_checksum_path, &checksum_text)?;
        format!("\n  manifest checksum sidecar: {}", manifest_checksum_path)
    } else {
        String::new()
    };

    let artifact_checksum_line = if let Some(artifact_checksum_path) = artifact_checksum_path {
        let checksum_text = format!("0x{:016x}\n", artifact.checksum);
        write_text_file(&artifact_checksum_path, &checksum_text)?;
        format!("\n  artifact checksum sidecar: {}", artifact_checksum_path)
    } else {
        String::new()
    };

    let normalized_intermediate_line = if let Some(normalized_intermediate_path) =
        normalized_intermediate_path
    {
        let normalized_intermediate_text = normalized_intermediate
            .as_deref()
            .expect("normalized intermediate text should be available when a normalized intermediate path is requested");
        write_text_file(&normalized_intermediate_path, normalized_intermediate_text)?;
        format!(
            "\n  normalized intermediate sidecar: {}",
            normalized_intermediate_path
        )
    } else {
        String::new()
    };

    Ok(format!(
        "Packaged artifact regenerated\n  path: {}\n  label: {}\n  source: {}\n  checksum: 0x{:016x}\n  bytes: {}\n  {}{}{}{}{}{}",
        output_path,
        artifact.header.generation_label,
        artifact.header.source,
        artifact.checksum,
        encoded.len(),
        packaged_artifact_regeneration_summary_for_report(),
        manifest_line,
        manifest_summary_line,
        manifest_checksum_line,
        artifact_checksum_line,
        normalized_intermediate_line,
    ))
}

pub(crate) fn render_packaged_artifact_regeneration_check() -> Result<String, String> {
    let artifact = pleiades_data::regenerate_packaged_artifact();
    let encoded = pleiades_data::regenerate_packaged_artifact_bytes();

    Ok(format!(
        "Packaged artifact regeneration check passed\n  label: {}\n  source: {}\n  checksum: 0x{:016x}\n  bytes: {}\n  {}",
        artifact.header.generation_label,
        artifact.header.source,
        artifact.checksum,
        encoded.len(),
        packaged_artifact_regeneration_summary_for_report(),
    ))
}

pub(crate) fn write_text_file(path: &str, contents: &str) -> Result<(), String> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
    }
    std::fs::write(path, contents).map_err(|error| format!("failed to write {}: {error}", path))
}

pub(crate) fn checksum64(text: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
