use std::{env, fs, path::PathBuf, process::ExitCode};

use pleiades_vsop87::{generated_vsop87b_table_bytes_for_source_file, source_specifications};

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
    let output_dir = parse_output_dir(env::args().skip(1))?;
    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("failed to create {}: {error}", output_dir.display()))?;

    for spec in source_specifications() {
        let bytes = generated_vsop87b_table_bytes_for_source_file(spec.source_file)
            .ok_or_else(|| format!("no vendored source text found for {}", spec.source_file))?;
        let output_path = output_dir.join(format!("{}.bin", spec.source_file));
        fs::write(&output_path, &bytes)
            .map_err(|error| format!("failed to write {}: {error}", output_path.display()))?;
        println!("wrote {} from {}", output_path.display(), spec.source_file);
    }

    Ok(())
}

fn parse_output_dir(mut args: impl Iterator<Item = String>) -> Result<PathBuf, String> {
    let Some(first) = args.next() else {
        return Err(usage().to_string());
    };

    let output_dir = if first == "--out" {
        let Some(path) = args.next() else {
            return Err(usage().to_string());
        };
        PathBuf::from(path)
    } else if first == "--help" || first == "-h" {
        return Err(usage().to_string());
    } else {
        PathBuf::from(first)
    };

    if args.next().is_some() {
        return Err(usage().to_string());
    }

    Ok(output_dir)
}

fn usage() -> &'static str {
    "Usage: cargo run -q -p pleiades-vsop87 --bin regenerate-vsop87b-tables -- --out <dir>\n       cargo run -q -p pleiades-vsop87 --bin regenerate-vsop87b-tables -- <dir>"
}
