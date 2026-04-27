use std::{env, fs, path::PathBuf, process::ExitCode};

use pleiades_vsop87::{
    checked_in_generated_vsop87b_table_bytes_for_source_file, source_specifications,
    try_generated_vsop87b_table_bytes_for_source_file,
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

    for spec in source_specifications() {
        let bytes = try_generated_vsop87b_table_bytes_for_source_file(spec.source_file)
            .map_err(|error| error.to_string())?;
        let output_path = output_dir.join(format!("{}.bin", spec.source_file));
        fs::write(&output_path, &bytes)
            .map_err(|error| format!("failed to write {}: {error}", output_path.display()))?;
        println!("wrote {} from {}", output_path.display(), spec.source_file);
    }

    Ok(())
}

fn check_regenerated_tables() -> Result<(), String> {
    let mut mismatches = Vec::new();

    for spec in source_specifications() {
        let regenerated = try_generated_vsop87b_table_bytes_for_source_file(spec.source_file)
            .map_err(|error| error.to_string())?;
        let committed = checked_in_generated_vsop87b_table_bytes_for_source_file(spec.source_file)
            .ok_or_else(|| {
                format!(
                    "no checked-in VSOP87B blob found for {}; supported source files: {}",
                    spec.source_file,
                    source_specifications()
                        .iter()
                        .map(|specification| specification.source_file)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;

        if regenerated.as_slice() != committed {
            mismatches.push(format!(
                "{} (generated {} bytes, committed {} bytes)",
                spec.source_file,
                regenerated.len(),
                committed.len()
            ));
        }
    }

    if mismatches.is_empty() {
        println!(
            "checked {} regenerated VSOP87B blobs against the committed artifacts",
            source_specifications().len()
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
    use super::{check_regenerated_tables, parse_command, Command};

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
    fn check_mode_matches_the_committed_artifacts() {
        check_regenerated_tables().expect("committed artifacts should match regenerated tables");
    }
}
