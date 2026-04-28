use std::{env, fs, path::PathBuf, process::ExitCode};

use pleiades_vsop87::{
    checked_in_generated_vsop87b_table_bytes_for_source_file, format_source_manifest_summary,
    source_manifest, source_manifest_summary, try_generated_vsop87b_table_bytes_for_source_file,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    match parse_command(env::args().skip(1))? {
        Command::Write { output_dir } => write_regenerated_tables(output_dir),
        Command::Check => check_regenerated_tables(),
    }
}

fn write_regenerated_tables(output_dir: PathBuf) -> Result<(), String> {
    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("failed to create {}: {error}", output_dir.display()))?;

    let manifest = source_manifest();
    let summary = source_manifest_summary(&manifest);
    summary.validate().map_err(|error| error.to_string())?;
    println!("{}", format_source_manifest_summary(&summary));

    for (body, source_file) in manifest {
        let bytes = try_generated_vsop87b_table_bytes_for_source_file(source_file)
            .map_err(|error| error.to_string())?;
        let output_path = output_dir.join(format!("{}.bin", source_file));
        fs::write(&output_path, &bytes)
            .map_err(|error| format!("failed to write {}: {error}", output_path.display()))?;
        println!(
            "wrote {} from {} ({})",
            output_path.display(),
            body,
            source_file
        );
    }

    Ok(())
}

fn check_regenerated_tables() -> Result<(), String> {
    let mut mismatches = Vec::new();
    let manifest = source_manifest();
    let summary = source_manifest_summary(&manifest);
    summary.validate().map_err(|error| error.to_string())?;
    println!("{}", format_source_manifest_summary(&summary));
    let manifest_len = manifest.len();
    let supported_source_files = manifest
        .iter()
        .map(|(_, source_file)| *source_file)
        .collect::<Vec<_>>()
        .join(", ");

    for (body, source_file) in &manifest {
        let regenerated = try_generated_vsop87b_table_bytes_for_source_file(source_file)
            .map_err(|error| error.to_string())?;
        let committed = checked_in_generated_vsop87b_table_bytes_for_source_file(source_file)
            .ok_or_else(|| {
                format!(
                    "no checked-in VSOP87B blob found for {} ({source_file}); supported source files: {}",
                    body,
                    supported_source_files
                )
            })?;

        if regenerated.as_slice() != committed {
            mismatches.push(format!(
                "{} ({source_file}) (generated {} bytes, committed {} bytes)",
                body,
                regenerated.len(),
                committed.len()
            ));
        }
    }

    if mismatches.is_empty() {
        println!(
            "checked {} regenerated VSOP87B blobs against the committed artifacts",
            manifest_len
        );
        Ok(())
    } else {
        Err(format!(
            "VSOP87B regeneration check failed for: {}",
            mismatches.join(", ")
        ))
    }
}

enum Command {
    Write { output_dir: PathBuf },
    Check,
}

fn parse_command(mut args: impl Iterator<Item = String>) -> Result<Command, String> {
    let Some(first) = args.next() else {
        return Err(usage().to_string());
    };

    if first == "--help" || first == "-h" {
        return Err(usage().to_string());
    }

    let command = if first == "--check" {
        if args.next().is_some() {
            return Err(usage().to_string());
        }
        Command::Check
    } else {
        let output_dir = if first == "--out" {
            let Some(path) = args.next() else {
                return Err(usage().to_string());
            };
            PathBuf::from(path)
        } else {
            PathBuf::from(first)
        };

        if args.next().is_some() {
            return Err(usage().to_string());
        }

        Command::Write { output_dir }
    };

    Ok(command)
}

fn usage() -> &'static str {
    "Usage: cargo run -q -p pleiades-vsop87 --bin regenerate-vsop87b-tables -- --out <dir>\n       cargo run -q -p pleiades-vsop87 --bin regenerate-vsop87b-tables -- <dir>\n       cargo run -q -p pleiades-vsop87 --bin regenerate-vsop87b-tables -- --check"
}

#[cfg(test)]
mod tests {
    use super::{
        check_regenerated_tables, format_source_manifest_summary, parse_command, source_manifest,
        source_manifest_summary, Command,
    };
    use pleiades_vsop87::validate_source_manifest;

    #[test]
    fn parse_command_accepts_check_mode() {
        let command =
            parse_command(["--check".to_string()].into_iter()).expect("check mode should parse");

        assert!(matches!(command, Command::Check));
    }

    #[test]
    fn parse_command_accepts_output_directory() {
        let command = parse_command(["--out".to_string(), "/tmp/vsop87".to_string()].into_iter())
            .expect("output directory should parse");

        assert!(matches!(command, Command::Write { .. }));
    }

    #[test]
    fn parse_command_rejects_extra_arguments() {
        assert!(parse_command(["--check".to_string(), "extra".to_string()].into_iter()).is_err());
        assert!(parse_command(
            [
                "--out".to_string(),
                "/tmp/vsop87".to_string(),
                "extra".to_string()
            ]
            .into_iter()
        )
        .is_err());
    }

    #[test]
    fn source_manifest_pairs_bodies_with_source_files_in_release_order() {
        let manifest = source_manifest();

        validate_source_manifest(&manifest).expect("source manifest should match the catalog");
        assert_eq!(manifest.len(), 8);
        assert_eq!(
            manifest
                .iter()
                .map(|(body, _)| body.to_string())
                .collect::<Vec<_>>(),
            vec![
                "Sun".to_string(),
                "Mercury".to_string(),
                "Venus".to_string(),
                "Mars".to_string(),
                "Jupiter".to_string(),
                "Saturn".to_string(),
                "Uranus".to_string(),
                "Neptune".to_string(),
            ]
        );
        assert_eq!(
            manifest
                .iter()
                .map(|(_, source_file)| *source_file)
                .collect::<Vec<_>>(),
            vec![
                "VSOP87B.ear",
                "VSOP87B.mer",
                "VSOP87B.ven",
                "VSOP87B.mar",
                "VSOP87B.jup",
                "VSOP87B.sat",
                "VSOP87B.ura",
                "VSOP87B.nep",
            ]
        );
    }

    #[test]
    fn source_manifest_summary_reports_the_release_order() {
        let manifest = source_manifest();
        let summary = source_manifest_summary(&manifest);

        summary
            .validate()
            .expect("source manifest summary should match the catalog");
        let rendered = format_source_manifest_summary(&summary);
        assert_eq!(rendered, summary.to_string());
        assert_eq!(
            rendered,
            "VSOP87 source manifest: 8 entries (Sun / VSOP87B.ear, Mercury / VSOP87B.mer, Venus / VSOP87B.ven, Mars / VSOP87B.mar, Jupiter / VSOP87B.jup, Saturn / VSOP87B.sat, Uranus / VSOP87B.ura, Neptune / VSOP87B.nep)"
        );
    }

    #[test]
    fn check_mode_matches_the_committed_artifacts() {
        check_regenerated_tables().expect("committed artifacts should match regenerated tables");
    }
}
