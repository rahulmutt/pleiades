//! CLI dispatch and report rendering for the validation tool.

pub(crate) mod cli;
pub(crate) mod text;

pub use cli::{banner, render_cli};
