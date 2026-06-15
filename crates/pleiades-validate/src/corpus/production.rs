//! Fail-closed gate over the checked-in production corpus slices.

use pleiades_backend::CelestialBody;
use pleiades_jpl::parse_snapshot_entries;
use pleiades_jpl::spk::corpus_spec;

/// A loaded slice: its role token and CSV text.
pub struct LoadedSlice {
    pub role: String,
    pub csv: String,
}

/// Validates that every release-claimed body appears in the corpus and that the
/// boundary, interior, and hold-out roles are all present and non-empty.
/// Returns the first violation as `Err`, fail-closed.
pub fn validate_completeness(slices: &[LoadedSlice]) -> Result<(), String> {
    let required_roles = ["boundary", "interior", "holdout"];
    for role in required_roles {
        let slice = slices
            .iter()
            .find(|s| s.role == role)
            .ok_or(format!("missing required slice role: {role}"))?;
        let entries = parse_snapshot_entries(&slice.csv)
            .map_err(|e| format!("slice {role} failed to parse: {e:?}"))?;
        if entries.is_empty() {
            return Err(format!("slice {role} has no data rows"));
        }
    }

    // Every release body must appear somewhere in the corpus.
    let mut seen: Vec<CelestialBody> = Vec::new();
    for slice in slices {
        if let Ok(entries) = parse_snapshot_entries(&slice.csv) {
            for e in entries {
                if !seen.contains(&e.body) {
                    seen.push(e.body);
                }
            }
        }
    }
    for body in corpus_spec::release_bodies() {
        if !seen.contains(&body) {
            return Err(format!(
                "release-claimed body missing from corpus: {body:?}"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(role: &str) -> String {
        format!("#Slice-Role: {role}\n#Columns:epoch_jd,body,x_km,y_km,z_km\n")
    }

    fn full_corpus() -> Vec<LoadedSlice> {
        let bodies = corpus_spec::release_bodies();
        let mut rows = String::new();
        for b in &bodies {
            rows.push_str(&format!("2451545,{b},1.0,2.0,3.0\n"));
        }
        ["boundary", "interior", "holdout"]
            .iter()
            .map(|r| LoadedSlice {
                role: r.to_string(),
                csv: format!("{}{}", header(r), rows),
            })
            .collect()
    }

    #[test]
    fn full_corpus_passes() {
        assert!(validate_completeness(&full_corpus()).is_ok());
    }

    #[test]
    fn missing_role_fails() {
        let mut corpus = full_corpus();
        corpus.retain(|s| s.role != "holdout");
        assert!(validate_completeness(&corpus).is_err());
    }

    #[test]
    fn missing_body_fails() {
        let mut corpus = full_corpus();
        // Strip Mars from every slice.
        for s in &mut corpus {
            s.csv = s
                .csv
                .lines()
                .filter(|l| !l.contains("Mars"))
                .collect::<Vec<_>>()
                .join("\n");
        }
        assert!(validate_completeness(&corpus).is_err());
    }

    #[test]
    fn empty_slice_fails() {
        let mut corpus = full_corpus();
        corpus[0].csv = header("boundary");
        assert!(validate_completeness(&corpus).is_err());
    }
}
