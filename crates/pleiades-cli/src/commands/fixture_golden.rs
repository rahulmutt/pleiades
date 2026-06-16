//! `generate-fixture-golden` command: derive the de440-independent
//! `fixture_golden.csv` from the checked-in Horizons reference snapshot.

use pleiades_jpl::spk::corpus_spec;

/// Writes `fixture_golden.csv` into `out_dir`, containing the reference-snapshot
/// (Horizons/DE441) entries at the corpus anchor epochs, in the geocentric
/// ecliptic corpus schema. This is the independent cross-check source; it is NOT
/// generated from the de440 kernel.
pub fn render_fixture_golden(args: &[&str]) -> Result<String, String> {
    let out_dir = args
        .first()
        .ok_or("generate-fixture-golden requires an output directory")?;

    let anchors = corpus_spec::anchor_epochs();
    let mut csv = String::new();
    csv.push_str("#Pleiades SPK Reference Corpus\n");
    csv.push_str(
        "#Source: NASA/JPL Horizons reference snapshot (independent of the de440 corpus)\n",
    );
    csv.push_str(&format!("#Kernel-SHA256: {}\n", corpus_spec::KERNEL_SHA256));
    csv.push_str("#Coverage: geocentric ecliptic (mean geometric), TDB epochs\n");
    csv.push_str("#Redistribution: derived from public-domain JPL Horizons fixture; corpus is redistributable\n");
    csv.push_str("#Slice-Role: fixture_golden\n");
    csv.push_str("#Columns:epoch_jd,body,x_km,y_km,z_km\n");

    let mut rows = 0usize;
    for entry in pleiades_jpl::reference_snapshot() {
        let jd = entry.epoch.julian_day.days();
        if anchors.iter().any(|a| (jd - a).abs() < 1e-6) {
            csv.push_str(&format!(
                "{},{},{:.6},{:.6},{:.6}\n",
                jd, entry.body, entry.x_km, entry.y_km, entry.z_km
            ));
            rows += 1;
        }
    }
    if rows == 0 {
        return Err("no reference-snapshot entries at the corpus anchor epochs".to_string());
    }

    std::fs::write(format!("{out_dir}/fixture_golden.csv"), &csv)
        .map_err(|e| format!("write fixture_golden.csv: {e}"))?;
    Ok(format!(
        "wrote fixture_golden.csv ({rows} rows) to {out_dir}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_out_dir_errors() {
        assert!(render_fixture_golden(&[]).is_err());
    }

    #[test]
    fn writes_anchor_rows_to_dir() {
        let dir = std::env::temp_dir().join(format!(
            "pleiades_fg_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let dir_str = dir.to_string_lossy();
        let msg = render_fixture_golden(&[&dir_str]).unwrap();
        assert!(msg.contains("fixture_golden.csv"));
        let written = std::fs::read_to_string(dir.join("fixture_golden.csv")).unwrap();
        assert!(written.contains("#Slice-Role: fixture_golden"));
        assert!(written.contains("#Columns:epoch_jd,body,x_km,y_km,z_km"));
        // At least one J2000 data row.
        assert!(written.lines().any(|l| l.starts_with("2451545")));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
