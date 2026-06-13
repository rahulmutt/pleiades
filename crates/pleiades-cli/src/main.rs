//! Command-line entry point for inspection, chart queries, and data tooling.
//!
//! The CLI now exposes the compatibility profile and a small chart report
//! command so contributors can exercise the first end-to-end workflow without
//! leaving the repository. The chart report keeps the mean/apparent position
//! choice explicit so report consumers can see which backend mode was used.

#![forbid(unsafe_code)]

mod cli;
mod commands;
mod help;
mod parse;
mod render;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    match cli::render_cli(&arg_refs) {
        Ok(rendered) => println!("{}", rendered),
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }
}
