//! Command-line entry point for inspection, chart queries, and data tooling.
//!
//! Stage 1 exposes a smoke-testable stub. Later stages will add the real
//! command surface.

#![forbid(unsafe_code)]

fn banner() -> &'static str {
    "pleiades-cli bootstrap stub"
}

fn main() {
    println!("{}", banner());
}

#[cfg(test)]
mod tests {
    use super::banner;

    #[test]
    fn banner_mentions_package() {
        assert!(banner().contains("pleiades-cli"));
    }
}
