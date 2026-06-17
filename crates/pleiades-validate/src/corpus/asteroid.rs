//! Fail-closed validation for the asteroid corpus slices: every row sits in the
//! 1900-2100 window, has finite coordinates, and names a body the curated
//! roster expects. Tier B (`asteroid_constrained`) is provenance-validated
//! here; it is never put behind the kernel regen gate.

use pleiades_jpl::spk::corpus_spec::{AST_RANGE_END_JD, AST_RANGE_START_JD};

use crate::corpus::production::LoadedSlice;

const ASTEROID_ROLES: [&str; 2] = ["asteroid_reference", "asteroid_constrained"];

/// Validates all asteroid slices, failing closed on the first breach.
pub fn validate_asteroid_slices(slices: &[LoadedSlice]) -> Result<(), String> {
    for s in slices
        .iter()
        .filter(|s| ASTEROID_ROLES.contains(&s.role.as_str()))
    {
        for (i, line) in s.csv.lines().enumerate() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let f: Vec<&str> = line.split(',').collect();
            if f.len() != 5 {
                return Err(format!(
                    "{}: row {i} has {} fields, expected 5",
                    s.role,
                    f.len()
                ));
            }
            let jd: f64 = f[0]
                .parse()
                .map_err(|_| format!("{}: row {i} bad epoch {:?}", s.role, f[0]))?;
            if !(AST_RANGE_START_JD..=AST_RANGE_END_JD).contains(&jd) {
                return Err(format!(
                    "{}: row {i} epoch {jd} outside asteroid window [{AST_RANGE_START_JD}, {AST_RANGE_END_JD}]",
                    s.role
                ));
            }
            for (col, raw) in [("x", f[2]), ("y", f[3]), ("z", f[4])] {
                let v: f64 = raw
                    .trim()
                    .parse()
                    .map_err(|_| format!("{}: row {i} bad {col} {raw:?}", s.role))?;
                if !v.is_finite() {
                    return Err(format!("{}: row {i} non-finite {col}", s.role));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::production::LoadedSlice;

    fn slice(role: &str, csv: &str) -> LoadedSlice {
        LoadedSlice {
            role: role.to_string(),
            csv: csv.to_string(),
        }
    }

    #[test]
    fn rejects_epoch_outside_window() {
        // 2200-01-01 is past AST_RANGE_END_JD.
        let s = slice("asteroid_reference", "2524594.5,Ceres,1.0,2.0,3.0\n");
        assert!(validate_asteroid_slices(&[s]).is_err());
    }

    #[test]
    fn rejects_non_finite_coord() {
        let s = slice("asteroid_reference", "2451545.0,Ceres,nan,2.0,3.0\n");
        assert!(validate_asteroid_slices(&[s]).is_err());
    }

    #[test]
    fn accepts_in_window_row() {
        let s = slice("asteroid_reference", "2451545.0,Ceres,1.0,2.0,3.0\n");
        assert!(validate_asteroid_slices(&[s]).is_ok());
    }

    #[test]
    fn ignores_non_asteroid_slices() {
        let s = slice("boundary", "9999999.0,Sun,1.0,2.0,3.0\n");
        assert!(validate_asteroid_slices(&[s]).is_ok());
    }
}
