//! Validation, comparison, and benchmarking entry point for the workspace.
//!
//! Stage 1 exposes a smoke-testable stub. Later stages will add the real
//! command surface.

#![forbid(unsafe_code)]

fn banner() -> &'static str {
    "pleiades-validate bootstrap stub"
}

fn main() {
    println!("{}", banner());
}

#[cfg(test)]
mod tests {
    use super::banner;

    #[test]
    fn banner_mentions_package() {
        assert!(banner().contains("pleiades-validate"));
    }
}
