//! Fail-closed gate over the checked-in production corpus slices.

use pleiades_backend::CelestialBody;
use pleiades_jpl::parse_snapshot_entries;
use pleiades_jpl::spk::corpus_manifest::{corpus_checksum64, CorpusManifest};
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

/// Validates schema header presence, finite numeric fields, and that the
/// `#Kernel-SHA256` is not the unfilled placeholder.
pub fn validate_schema_and_provenance(slices: &[LoadedSlice]) -> Result<(), String> {
    for s in slices {
        if !s.csv.contains("#Columns:epoch_jd,body,x_km,y_km,z_km") {
            return Err(format!("slice {} missing column header", s.role));
        }
        if s.csv.contains("#Kernel-SHA256: <pinned-after-download>")
            || s.csv.contains("#Kernel-SHA256: <run shasum")
        {
            return Err(format!("slice {} has placeholder kernel SHA-256", s.role));
        }
        for line in s
            .csv
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
        {
            let fields: Vec<&str> = line.split(',').collect();
            if fields.len() != 5 {
                return Err(format!(
                    "malformed row in {} (expected 5 fields, found {}): {line}",
                    s.role,
                    fields.len()
                ));
            }
            for field in &fields[2..] {
                let v: f64 = field
                    .parse()
                    .map_err(|_| format!("non-numeric field in {}", s.role))?;
                if !v.is_finite() {
                    return Err(format!("non-finite field in {}", s.role));
                }
            }
        }
    }
    Ok(())
}

/// Validates each slice's content checksum against the manifest.
pub fn validate_drift(slices: &[LoadedSlice], manifest: &CorpusManifest) -> Result<(), String> {
    for s in slices {
        let entry = manifest
            .slices
            .iter()
            .find(|e| e.role == s.role)
            .ok_or(format!("manifest missing slice for role {}", s.role))?;
        let actual = corpus_checksum64(&s.csv);
        if actual != entry.checksum {
            return Err(format!(
                "checksum drift for {}: manifest {} != actual {}",
                s.role, entry.checksum, actual
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod drift_tests {
    use super::*;
    use pleiades_jpl::spk::corpus_manifest::SliceEntry;

    fn slice(role: &str, sha: &str) -> LoadedSlice {
        LoadedSlice {
            role: role.to_string(),
            csv: format!(
                "#Kernel-SHA256: {sha}\n#Columns:epoch_jd,body,x_km,y_km,z_km\n2451545,Sun,1.0,2.0,3.0\n"
            ),
        }
    }

    #[test]
    fn placeholder_sha_fails() {
        let s = vec![slice("boundary", "<pinned-after-download>")];
        assert!(validate_schema_and_provenance(&s).is_err());
    }

    #[test]
    fn real_sha_passes_schema() {
        let s = vec![slice("boundary", "deadbeef")];
        assert!(validate_schema_and_provenance(&s).is_ok());
    }

    #[test]
    fn checksum_mismatch_fails() {
        let s = slice("boundary", "deadbeef");
        let manifest = CorpusManifest {
            kernel: "de440.bsp".to_string(),
            kernel_sha256: "deadbeef".to_string(),
            slices: vec![SliceEntry {
                name: "boundary".to_string(),
                file: "boundary.csv".to_string(),
                role: "boundary".to_string(),
                rows: 1,
                checksum: 12345, // deliberately wrong
            }],
        };
        assert!(validate_drift(&[s], &manifest).is_err());
    }

    #[test]
    fn matching_checksum_passes() {
        let s = slice("boundary", "deadbeef");
        let checksum = corpus_checksum64(&s.csv);
        let manifest = CorpusManifest {
            kernel: "de440.bsp".to_string(),
            kernel_sha256: "deadbeef".to_string(),
            slices: vec![SliceEntry {
                name: "boundary".to_string(),
                file: "boundary.csv".to_string(),
                role: "boundary".to_string(),
                rows: 1,
                checksum,
            }],
        };
        assert!(validate_drift(&[s], &manifest).is_ok());
    }

    #[test]
    fn malformed_short_row_fails() {
        let s = vec![LoadedSlice {
            role: "boundary".to_string(),
            csv: "#Columns:epoch_jd,body,x_km,y_km,z_km\n2451545,Sun\n".to_string(),
        }];
        assert!(validate_schema_and_provenance(&s).is_err());
    }

    #[test]
    fn non_numeric_coordinate_fails() {
        let s = vec![LoadedSlice {
            role: "boundary".to_string(),
            csv: "#Columns:epoch_jd,body,x_km,y_km,z_km\n2451545,Sun,1.0,abc,3.0\n".to_string(),
        }];
        assert!(validate_schema_and_provenance(&s).is_err());
    }

    #[test]
    fn run_shasum_placeholder_form_fails() {
        let s = vec![LoadedSlice {
            role: "boundary".to_string(),
            csv: "#Kernel-SHA256: <run shasum -a 256 de440.bsp>\n#Columns:epoch_jd,body,x_km,y_km,z_km\n2451545,Sun,1.0,2.0,3.0\n".to_string(),
        }];
        assert!(validate_schema_and_provenance(&s).is_err());
    }
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
