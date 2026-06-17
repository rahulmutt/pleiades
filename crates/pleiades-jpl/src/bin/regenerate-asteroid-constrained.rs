//! Maintainer tool (Tier B): generate the `asteroid_constrained` corpus slice by
//! fetching geocentric ecliptic vectors from JPL Horizons for the Tier B roster
//! (centaurs, personal asteroids, TNOs) over the 1900-2100 asteroid window.
//!
//! Requires the `horizons-fetch` feature (pure-Rust rustls + graviola TLS — no C
//! toolchain). Network-dependent and NOT reproducible byte-for-byte across
//! Horizons solution updates, which is why Tier B is provenance-validated and
//! never put behind the kernel regen gate.
//!
//! Usage:
//!   cargo run -p pleiades-jpl --features horizons-fetch \
//!     --bin regenerate-asteroid-constrained
//!
//! Writes `crates/pleiades-jpl/data/corpus/asteroid_constrained.csv` and prints
//! the `slice asteroid_constrained …` manifest line (rows + checksum).

#[cfg(not(feature = "horizons-fetch"))]
fn main() -> Result<(), String> {
    Err("rebuild with --features horizons-fetch (this tool hits JPL Horizons)".to_string())
}

#[cfg(feature = "horizons-fetch")]
fn main() -> Result<(), String> {
    use std::sync::Arc;

    use pleiades_backend::CelestialBody;
    use pleiades_jpl::spk::asteroid_roster::{asteroid_core_roster, AsteroidClass, AsteroidTier};
    use pleiades_jpl::spk::corpus_manifest::corpus_checksum64;
    use pleiades_jpl::spk::corpus_spec::{AST_RANGE_END_JD, AST_RANGE_START_JD};
    use ureq::tls::{TlsConfig, TlsProvider};

    const OUT_PATH: &str = "crates/pleiades-jpl/data/corpus/asteroid_constrained.csv";

    // Pure-Rust TLS agent, identical stack to ingest::fetch::HttpHorizonsSource.
    let provider = Arc::new(rustls_graviola::default_provider());
    let agent = ureq::Agent::config_builder()
        .tls_config(
            TlsConfig::builder()
                .provider(TlsProvider::Rustls)
                .unversioned_rustls_crypto_provider(provider)
                .build(),
        )
        .build()
        .new_agent();

    let mut out = String::new();
    out.push_str("#Pleiades SPK Reference Corpus\n");
    out.push_str("#Source: JPL Horizons VECTORS (geocentric, Ecliptic of J2000, GEOMETRIC), Tier B constrained\n");
    out.push_str("#Coverage: geocentric ecliptic (mean geometric), TDB epochs\n");
    out.push_str(
        "#Redistribution: derived from public-domain JPL Horizons; corpus is redistributable\n",
    );
    out.push_str("#Slice-Role: asteroid_constrained\n");
    out.push_str("#Columns:epoch_jd,body,x_km,y_km,z_km\n");

    let mut total_rows = 0usize;
    for entry in asteroid_core_roster()
        .iter()
        .filter(|e| e.tier == AsteroidTier::Constrained)
    {
        let designation = match &entry.body {
            CelestialBody::Custom(c) => c.designation.clone(),
            other => return Err(format!("unexpected non-Custom Tier B body: {other:?}")),
        };
        let number: String = designation
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if number.is_empty() {
            return Err(format!("Tier B body {} has no leading IAU number", entry.body));
        }
        let step = match entry.class {
            AsteroidClass::MainBelt => "180d",
            AsteroidClass::Centaur => "365d",
            AsteroidClass::Tno => "1825d",
        };
        // COMMAND='<number>;' forces an unambiguous small-body lookup by number.
        let url = format!(
            "https://ssd.jpl.nasa.gov/api/horizons.api?format=text&EPHEM_TYPE=VECTORS\
&CSV_FORMAT=YES&OBJ_DATA=NO&VEC_TABLE=1&REF_PLANE=ECLIPTIC&OUT_UNITS=KM-S\
&COMMAND='{number}%3B'&CENTER='500@399'&START_TIME='JD{:.1}'&STOP_TIME='JD{:.1}'&STEP_SIZE='{step}'",
            AST_RANGE_START_JD, AST_RANGE_END_JD
        );
        eprintln!("fetching {} (#{number}) step={step} …", entry.body);
        let mut resp = agent
            .get(&url)
            .call()
            .map_err(|e| format!("fetch #{number}: {e}"))?;
        let bytes = resp
            .body_mut()
            .read_to_vec()
            .map_err(|e| format!("read #{number}: {e}"))?;
        let text = String::from_utf8(bytes).map_err(|e| format!("utf8 #{number}: {e}"))?;

        let body_token = format!("{}", entry.body);
        let mut rows = 0usize;
        let mut in_data = false;
        for line in text.lines() {
            match line.trim() {
                "$$SOE" => {
                    in_data = true;
                    continue;
                }
                "$$EOE" => {
                    in_data = false;
                    continue;
                }
                _ => {}
            }
            if !in_data {
                continue;
            }
            // CSV columns: JDTDB, Calendar Date, X, Y, Z, (trailing comma)
            let f: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if f.len() < 5 {
                return Err(format!("#{number}: short data row: {line}"));
            }
            let jd: f64 = f[0].parse().map_err(|_| format!("#{number}: bad jd {}", f[0]))?;
            let x: f64 = f[2].parse().map_err(|_| format!("#{number}: bad x {}", f[2]))?;
            let y: f64 = f[3].parse().map_err(|_| format!("#{number}: bad y {}", f[3]))?;
            let z: f64 = f[4].parse().map_err(|_| format!("#{number}: bad z {}", f[4]))?;
            if !(x.is_finite() && y.is_finite() && z.is_finite()) {
                return Err(format!("#{number}: non-finite row: {line}"));
            }
            if !(AST_RANGE_START_JD..=AST_RANGE_END_JD).contains(&jd) {
                return Err(format!("#{number}: epoch {jd} outside asteroid window"));
            }
            out.push_str(&format!("{jd},{body_token},{x:.6},{y:.6},{z:.6}\n"));
            rows += 1;
        }
        if rows == 0 {
            return Err(format!(
                "#{number} ({}): Horizons returned no $$SOE/$$EOE data rows",
                entry.body
            ));
        }
        eprintln!("  {} rows", rows);
        total_rows += rows;
        // Be polite to the Horizons API between objects.
        std::thread::sleep(std::time::Duration::from_millis(600));
    }

    std::fs::write(OUT_PATH, &out).map_err(|e| format!("write {OUT_PATH}: {e}"))?;
    let checksum = corpus_checksum64(&out);
    eprintln!("\nwrote {OUT_PATH}: {total_rows} rows, checksum={checksum}");
    println!(
        "slice asteroid_constrained file=asteroid_constrained.csv role=asteroid_constrained rows={total_rows} checksum={checksum}"
    );
    Ok(())
}
