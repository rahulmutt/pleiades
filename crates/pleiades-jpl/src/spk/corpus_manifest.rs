//! Corpus manifest: provenance plus per-slice FNV content checksums and row
//! counts, rendered in the repo's manifest-text style and parsed back.

/// Deterministic 64-bit content checksum (FNV-1a) used to detect drift between
/// a checked-in slice file and the manifest. Not cryptographic; pairs with the
/// gated regenerate-and-value-compare path for kernel-identity assurance.
///
/// Kept byte-identical to `reference_summary::...::checksum64`; if you change
/// one, change both.
pub fn corpus_checksum64(text: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0001_0000_01b3;
    let mut hash = FNV_OFFSET_BASIS;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_is_deterministic_and_sensitive() {
        assert_eq!(corpus_checksum64("abc"), corpus_checksum64("abc"));
        assert_ne!(corpus_checksum64("abc"), corpus_checksum64("abd"));
    }
}

/// One slice row in the manifest.
#[derive(Clone, Debug, PartialEq)]
pub struct SliceEntry {
    pub name: String,
    pub file: String,
    pub role: String,
    pub rows: usize,
    pub checksum: u64,
}

/// Parsed corpus manifest.
#[derive(Clone, Debug, PartialEq)]
pub struct CorpusManifest {
    pub kernel: String,
    pub kernel_sha256: String,
    pub slices: Vec<SliceEntry>,
}

impl CorpusManifest {
    /// Renders the manifest in the repo's `#`-comment + keyword-line style.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str("#Pleiades SPK Reference Corpus Manifest\n");
        out.push_str(&format!("#Kernel: {}\n", self.kernel));
        out.push_str(&format!("#Kernel-SHA256: {}\n", self.kernel_sha256));
        for s in &self.slices {
            out.push_str(&format!(
                "slice {} file={} role={} rows={} checksum={}\n",
                s.name, s.file, s.role, s.rows, s.checksum
            ));
        }
        out
    }

    /// Parses the rendered form. Unknown `#` lines are ignored; malformed
    /// `slice` lines are an error.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut kernel = String::new();
        let mut kernel_sha256 = String::new();
        let mut slices = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("#Kernel-SHA256:") {
                kernel_sha256 = rest.trim().to_string();
            } else if let Some(rest) = line.strip_prefix("#Kernel:") {
                kernel = rest.trim().to_string();
            } else if let Some(rest) = line.strip_prefix("slice ") {
                slices.push(parse_slice_line(rest)?);
            }
        }
        Ok(CorpusManifest {
            kernel,
            kernel_sha256,
            slices,
        })
    }
}

fn parse_slice_line(rest: &str) -> Result<SliceEntry, String> {
    let mut parts = rest.split_whitespace();
    let name = parts.next().ok_or("slice line missing name")?.to_string();
    let mut file = String::new();
    let mut role = String::new();
    let mut rows = 0usize;
    let mut checksum = 0u64;
    for kv in parts {
        let (k, v) = kv.split_once('=').ok_or(format!("bad token: {kv}"))?;
        match k {
            "file" => file = v.to_string(),
            "role" => role = v.to_string(),
            "rows" => rows = v.parse().map_err(|_| format!("bad rows: {v}"))?,
            "checksum" => checksum = v.parse().map_err(|_| format!("bad checksum: {v}"))?,
            _ => return Err(format!("unknown key: {k}")),
        }
    }
    if file.is_empty() || role.is_empty() {
        return Err(format!("slice {name} missing file/role"));
    }
    Ok(SliceEntry {
        name,
        file,
        role,
        rows,
        checksum,
    })
}

#[cfg(test)]
mod manifest_tests {
    use super::*;

    fn sample() -> CorpusManifest {
        CorpusManifest {
            kernel: "de440.bsp".to_string(),
            kernel_sha256: "abc123".to_string(),
            slices: vec![SliceEntry {
                name: "boundary".to_string(),
                file: "boundary.csv".to_string(),
                role: "boundary".to_string(),
                rows: 96,
                checksum: 42,
            }],
        }
    }

    #[test]
    fn render_parse_round_trips() {
        let m = sample();
        let parsed = CorpusManifest::parse(&m.render()).unwrap();
        assert_eq!(parsed, m);
    }

    #[test]
    fn malformed_slice_line_errors() {
        assert!(CorpusManifest::parse("slice boundary file=b.csv\n").is_err());
    }
}
