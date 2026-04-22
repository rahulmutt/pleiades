//! Command-line entry point for inspection, chart queries, and data tooling.
//!
//! The CLI currently exposes the compatibility profile so contributors can see
//! the baseline house and ayanamsa coverage without needing to inspect source
//! files directly. More chart-oriented commands will be added as the algorithmic
//! backends land in later slices.

#![forbid(unsafe_code)]

use pleiades_core::current_compatibility_profile;

fn banner() -> &'static str {
    "pleiades-cli bootstrap stub"
}

fn render_cli(args: &[&str]) -> String {
    match args.first().copied() {
        Some("compatibility-profile") | Some("profile") => {
            current_compatibility_profile().to_string()
        }
        Some("help") | Some("--help") | Some("-h") => {
            format!(
                "{}\n\nCommands:\n  compatibility-profile  Print the current compatibility profile\n  profile                Alias for compatibility-profile\n  help                   Show this help text",
                banner()
            )
        }
        _ => banner().to_string(),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    println!("{}", render_cli(&arg_refs));
}

#[cfg(test)]
mod tests {
    use super::{banner, render_cli};

    #[test]
    fn banner_mentions_package() {
        assert!(banner().contains("pleiades-cli"));
    }

    #[test]
    fn profile_command_renders_catalogs() {
        let rendered = render_cli(&["compatibility-profile"]);
        assert!(rendered.contains("Built-in house systems:"));
        assert!(rendered.contains("Topocentric"));
        assert!(rendered.contains("Built-in ayanamsas:"));
        assert!(rendered.contains("Lahiri"));
    }
}
