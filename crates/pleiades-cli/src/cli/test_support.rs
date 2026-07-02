//! Shared test setup for the CLI command-output suites.

pub(crate) fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
    let unique = format!(
        "{}-{}-{}",
        prefix,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after UNIX_EPOCH")
            .as_nanos()
    );
    let path = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&path).expect("temp dir should be creatable");
    path
}

pub(crate) fn packaged_artifact_access_report_line() -> String {
    format!(
        "Packaged-artifact access: {}",
        pleiades_data::packaged_artifact_access_summary()
    )
}

pub(crate) fn help_command_names(help: &str) -> std::collections::BTreeSet<String> {
    help.lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            if !line.starts_with("  ") {
                return None;
            }
            let command = trimmed.split_whitespace().next()?;
            if command.starts_with('-')
                || !command
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
            {
                return None;
            }
            Some(command.to_string())
        })
        .collect()
}

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
