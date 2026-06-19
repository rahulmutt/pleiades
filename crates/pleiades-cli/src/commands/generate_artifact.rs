//! `generate-artifact` command: regenerate the packaged artifact from a de440
//! kernel over a chosen coverage window and write the encoded bytes to a file.
//!
//! Usage:
//!   generate-artifact <kernel.bsp> --out <path> [--start <year|JD>] [--end <year|JD>]
//!
//! `--start`/`--end` accept a calendar year (e.g. 1850) or a Julian Day (a value
//! with a decimal point, e.g. 2451545.0). Omitted bounds default to the shipped
//! 1900–2100 window. Major-body generation requires the kernel (dense de440 fit).

use pleiades_data::regenerate_packaged_artifact_from_kernel_over;
use pleiades_jpl::spk::corpus_spec::CoverageWindow;

/// Parse a `--start`/`--end` token: a value containing '.' is a JD; otherwise a
/// calendar year converted to Jan 1 00:00 TDB JD.
fn parse_bound(token: &str) -> Result<Option<i32>, String> {
    // Returns Ok(Some(year)) for a year token, Ok(None) handled by caller for JD.
    token
        .parse::<i32>()
        .map(Some)
        .map_err(|_| format!("bad year/JD bound: {token}"))
}

pub fn render_generate_artifact(args: &[&str]) -> Result<String, String> {
    let kernel = args
        .first()
        .ok_or("generate-artifact requires a kernel path")?;

    let mut out: Option<&str> = None;
    let mut start: Option<f64> = None;
    let mut end: Option<f64> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i] {
            "--out" => {
                out = Some(args.get(i + 1).ok_or("--out requires a path")?);
                i += 2;
            }
            "--start" => {
                start = Some(parse_bound_jd(args.get(i + 1).ok_or("--start requires a value")?)?);
                i += 2;
            }
            "--end" => {
                end = Some(parse_bound_jd(args.get(i + 1).ok_or("--end requires a value")?)?);
                i += 2;
            }
            other => return Err(format!("unknown generate-artifact arg: {other}")),
        }
    }

    let out = out.ok_or("generate-artifact requires --out <path>")?;
    let default = CoverageWindow::default();
    let window = CoverageWindow::new(
        start.unwrap_or(default.start_jd),
        end.unwrap_or(default.end_jd),
    );
    if window.end_jd <= window.start_jd {
        return Err("coverage window end must be after start".to_string());
    }

    let artifact = regenerate_packaged_artifact_from_kernel_over(kernel, window)?;
    let bytes = artifact.encode().map_err(|e| format!("encode: {e}"))?;
    let len = bytes.len();
    std::fs::write(out, &bytes).map_err(|e| format!("write {out}: {e}"))?;
    Ok(format!(
        "wrote {len} bytes to {out} (window {}..{} JD)",
        window.start_jd, window.end_jd
    ))
}

/// A token with a '.' is treated as a JD; otherwise as a calendar year.
fn parse_bound_jd(token: &str) -> Result<f64, String> {
    if token.contains('.') {
        token.parse::<f64>().map_err(|_| format!("bad JD: {token}"))
    } else {
        let year = parse_bound(token)?.expect("year token");
        Ok(CoverageWindow::from_years(year, year + 1).start_jd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_kernel_errors() {
        assert!(render_generate_artifact(&[]).is_err());
    }

    #[test]
    fn missing_out_errors() {
        let err = render_generate_artifact(&["/no/such/kernel.bsp"]).unwrap_err();
        assert!(err.contains("--out"), "unexpected: {err}");
    }

    #[test]
    fn year_token_parses_to_jan1_jd() {
        let jd = parse_bound_jd("2000").unwrap();
        assert!((jd - 2_451_544.5).abs() < 1e-6);
    }

    #[test]
    fn jd_token_parses_as_jd() {
        let jd = parse_bound_jd("2451545.0").unwrap();
        assert!((jd - 2_451_545.0).abs() < 1e-9);
    }

    #[test]
    fn inverted_window_errors() {
        let err = render_generate_artifact(&[
            "/no/such/kernel.bsp", "--out", "/tmp/x.bin", "--start", "2100", "--end", "1900",
        ])
        .unwrap_err();
        // The guard fires before the kernel is loaded, so the fake path is never reached.
        assert!(err.contains("coverage window end must be after start"), "expected inverted-window guard, got: {err}");
    }
}
