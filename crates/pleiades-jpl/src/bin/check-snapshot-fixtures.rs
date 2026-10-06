//! Checks every row of the two snapshot fixtures against a fresh JPL Horizons
//! fetch (issue #200).
//!
//! `data/reference_snapshot.csv` and `data/independent_holdout_snapshot.csv`
//! are documented as Horizons geocentric ecliptic J2000 vectors. A row at an
//! isolated epoch has no neighbour to be checked against offline, so this tool
//! fetches each body's rows again and reports every row that Horizons places
//! more than [`TOLERANCE_ARCSEC`] away, as seen from the Earth.
//!
//! Requires the `horizons-fetch` feature (pure-Rust rustls + graviola TLS — no
//! C toolchain) and network access. Run it from the repository root:
//!
//!   cargo run -p pleiades-jpl --features horizons-fetch \
//!       --bin check-snapshot-fixtures
//!
//! It exits non-zero when a row disagrees. Small differences are expected for
//! the asteroids, because Horizons replaces a small body's orbit solution as
//! observations accumulate.

#[cfg(not(feature = "horizons-fetch"))]
fn main() -> Result<(), String> {
    Err("rebuild with --features horizons-fetch (this tool hits JPL Horizons)".to_string())
}

/// Largest separation, as seen from the Earth, between a fixture row and the
/// Horizons vector for the same body and instant. Measured 2026-10-06: the
/// worst row is asteroid:433-Eros at JD 2634167.0, 0.0203″ (an orbit-solution
/// update, 34 km at 2.3 AU); every major-body row agrees to the printed digits.
#[cfg(feature = "horizons-fetch")]
const TOLERANCE_ARCSEC: f64 = 0.05;

#[cfg(feature = "horizons-fetch")]
fn main() -> Result<(), String> {
    use pleiades_jpl::ingest::HttpHorizonsSource;
    use pleiades_jpl::{independent_holdout_snapshot_entries, reference_snapshot};

    let fixtures = [
        ("reference_snapshot.csv", reference_snapshot()),
        (
            "independent_holdout_snapshot.csv",
            independent_holdout_snapshot_entries().ok_or("the hold-out snapshot failed to load")?,
        ),
    ];

    let mut disagreements = 0_usize;
    for (label, entries) in fixtures {
        if entries.is_empty() {
            return Err(format!("{label} holds no rows"));
        }
        let mut bodies = Vec::new();
        for entry in entries {
            if !bodies.contains(&entry.body) {
                bodies.push(entry.body.clone());
            }
        }
        let mut worst = 0.0_f64;
        let mut rows = 0_usize;
        for body in &bodies {
            let command = horizons_command(body)?;
            let body_rows = entries
                .iter()
                .filter(|entry| &entry.body == body)
                .collect::<Vec<_>>();
            let epochs = body_rows
                .iter()
                .map(|entry| entry.epoch.julian_day.days())
                .collect::<Vec<_>>();
            let bytes = HttpHorizonsSource
                .fetch_url(&vectors_url(&command, &epochs))
                .map_err(|error| format!("{label}: fetching {body}: {error}"))?;
            let text =
                String::from_utf8(bytes).map_err(|error| format!("{label}: {body}: {error}"))?;
            let fetched =
                parse_vectors(&text).map_err(|error| format!("{label}: {body}: {error}"))?;
            for entry in body_rows {
                let jd = entry.epoch.julian_day.days();
                let truth = fetched
                    .iter()
                    .find(|(fetched_jd, _)| (fetched_jd - jd).abs() < 1e-7)
                    .map(|(_, position)| *position)
                    .ok_or_else(|| {
                        format!("{label}: Horizons returned no {body} row at JD {jd}")
                    })?;
                let separation = separation_arcsec([entry.x_km, entry.y_km, entry.z_km], truth);
                rows += 1;
                worst = worst.max(separation);
                if separation > TOLERANCE_ARCSEC {
                    disagreements += 1;
                    println!("{label}: {body} at JD {jd} is {separation:.4}″ from Horizons");
                }
            }
            // Be polite to the Horizons API between objects.
            std::thread::sleep(std::time::Duration::from_millis(600));
        }
        println!(
            "{label}: {rows} rows across {} bodies, worst {worst:.4}″",
            bodies.len()
        );
    }

    if disagreements > 0 {
        return Err(format!(
            "{disagreements} rows differ from Horizons by more than {TOLERANCE_ARCSEC}″"
        ));
    }
    Ok(())
}

/// The Horizons `COMMAND` for a fixture body: the planet centre for a major
/// body (as the fixtures hold, not the system barycentre), or the IAU number
/// with a trailing `;` for an asteroid, which forces a small-body lookup.
#[cfg(feature = "horizons-fetch")]
fn horizons_command(body: &pleiades_backend::CelestialBody) -> Result<String, String> {
    use pleiades_backend::CelestialBody;

    let command = match body {
        CelestialBody::Sun => "10",
        CelestialBody::Moon => "301",
        CelestialBody::Mercury => "199",
        CelestialBody::Venus => "299",
        CelestialBody::Mars => "499",
        CelestialBody::Jupiter => "599",
        CelestialBody::Saturn => "699",
        CelestialBody::Uranus => "799",
        CelestialBody::Neptune => "899",
        CelestialBody::Pluto => "999",
        CelestialBody::Ceres => "1;",
        CelestialBody::Pallas => "2;",
        CelestialBody::Juno => "3;",
        CelestialBody::Vesta => "4;",
        CelestialBody::Custom(custom) => {
            let number = custom
                .designation
                .chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>();
            if number.is_empty() {
                return Err(format!("{body} has no leading IAU number"));
            }
            return Ok(format!("{number};"));
        }
        other => return Err(format!("no Horizons target is known for {other}")),
    };
    Ok(command.to_string())
}

/// A Horizons vector-table request for one target at the given TDB Julian
/// days: geocentric, ecliptic of J2000, geometric (no light-time or
/// aberration), kilometres.
#[cfg(feature = "horizons-fetch")]
fn vectors_url(command: &str, epochs: &[f64]) -> String {
    let times = epochs
        .iter()
        .map(|jd| format!("{jd:.9}"))
        .collect::<Vec<_>>()
        .join("%20");
    format!(
        "https://ssd.jpl.nasa.gov/api/horizons.api?format=text&EPHEM_TYPE=VECTORS\
&CSV_FORMAT=YES&OBJ_DATA=NO&VEC_TABLE=1&VEC_CORR=NONE&REF_PLANE=ECLIPTIC&REF_SYSTEM=ICRF\
&OUT_UNITS=KM-S&COMMAND='{}'&CENTER='500@399'&TLIST_TYPE=JD&TIME_TYPE=TDB&TLIST='{times}'",
        command.replace(';', "%3B")
    )
}

/// The `(JD, [x, y, z])` rows between `$$SOE` and `$$EOE`.
#[cfg(feature = "horizons-fetch")]
fn parse_vectors(text: &str) -> Result<Vec<(f64, [f64; 3])>, String> {
    let mut rows = Vec::new();
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
        let fields = line.split(',').map(str::trim).collect::<Vec<_>>();
        if fields.len() < 5 {
            return Err(format!("short data row: {line}"));
        }
        let number = |index: usize| {
            fields[index]
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite())
                .ok_or_else(|| format!("bad number {:?} in row: {line}", fields[index]))
        };
        rows.push((number(0)?, [number(2)?, number(3)?, number(4)?]));
    }
    if rows.is_empty() {
        return Err("Horizons returned no $$SOE/$$EOE data rows".to_string());
    }
    Ok(rows)
}

/// Angle between two geocentric vectors, in arcseconds.
#[cfg(feature = "horizons-fetch")]
fn separation_arcsec(left: [f64; 3], right: [f64; 3]) -> f64 {
    let dot = left[0] * right[0] + left[1] * right[1] + left[2] * right[2];
    let cross = [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ];
    let cross_norm = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    cross_norm.atan2(dot).to_degrees() * 3600.0
}
