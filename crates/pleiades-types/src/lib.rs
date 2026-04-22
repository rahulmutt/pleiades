//! Shared primitive and domain-adjacent types used across the workspace.
//!
//! Stage 1 keeps this crate as a small, compilable placeholder so the
//! workspace boundary is visible before the real implementation lands.

#![forbid(unsafe_code)]

/// Returns the canonical package name for this crate.
pub const fn package_name() -> &'static str {
    "pleiades-types"
}

#[cfg(test)]
mod tests {
    use super::package_name;

    #[test]
    fn package_name_is_stable() {
        assert_eq!(package_name(), "pleiades-types");
    }
}
