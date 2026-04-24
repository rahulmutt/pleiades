//! VSOP87B Earth coefficient tables backed by the full public IMCCE/CELMECH
//! source file.
//!
//! The Earth file is vendored verbatim from the public `VSOP87B.ear` source.
//! The runtime path now prefers a generated binary table derived from that
//! vendored input so the Sun path has a reproducible coefficient artifact while
//! the text parser remains available for validation and future generator work.

use std::sync::OnceLock;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Vsop87Term {
    pub(crate) amplitude: f64,
    pub(crate) phase: f64,
    pub(crate) frequency: f64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SphericalLbr {
    pub longitude_rad: f64,
    pub latitude_rad: f64,
    pub radius_au: f64,
}

#[derive(Debug)]
pub(crate) struct Vsop87SeriesTables {
    pub(crate) longitude: Vec<Vec<Vsop87Term>>,
    pub(crate) latitude: Vec<Vec<Vsop87Term>>,
    pub(crate) radius: Vec<Vec<Vsop87Term>>,
}

static EARTH_TABLES: OnceLock<Vsop87SeriesTables> = OnceLock::new();

pub(crate) fn earth_lbr(julian_day_tt: f64) -> SphericalLbr {
    // VSOP87 uses Julian millennia from J2000.0.
    let t = (julian_day_tt - 2_451_545.0) / 365_250.0;
    let tables = earth_tables();
    SphericalLbr {
        longitude_rad: evaluate(tables.longitude.iter().map(Vec::as_slice), t)
            .rem_euclid(core::f64::consts::TAU),
        latitude_rad: evaluate(tables.latitude.iter().map(Vec::as_slice), t),
        radius_au: evaluate(tables.radius.iter().map(Vec::as_slice), t),
    }
}

pub(crate) fn evaluate<I, T>(series_by_power: I, t: f64) -> f64
where
    I: IntoIterator<Item = T>,
    T: AsRef<[Vsop87Term]>,
{
    let mut t_power = 1.0;
    let mut value = 0.0;
    for terms in series_by_power {
        let terms = terms.as_ref();
        let subtotal: f64 = terms
            .iter()
            .map(|term| term.amplitude * (term.phase + term.frequency * t).cos())
            .sum();
        value += subtotal * t_power;
        t_power *= t;
    }
    value
}

const GENERATED_EARTH_TABLE_MAGIC: &[u8; 8] = b"PVSBTAB1";
const GENERATED_EARTH_TABLE_VERSION: u32 = 1;

fn earth_tables() -> &'static Vsop87SeriesTables {
    EARTH_TABLES
        .get_or_init(|| parse_generated_vsop87b_tables(include_bytes!("../data/VSOP87B.ear.bin")))
}

pub(crate) fn parse_generated_vsop87b_tables(bytes: &[u8]) -> Vsop87SeriesTables {
    let mut cursor = 0usize;

    fn take<'a>(bytes: &'a [u8], cursor: &mut usize, len: usize) -> &'a [u8] {
        let end = cursor
            .checked_add(len)
            .expect("VSOP87B generated table length overflow");
        assert!(end <= bytes.len(), "truncated generated VSOP87B table");
        let slice = &bytes[*cursor..end];
        *cursor = end;
        slice
    }

    fn take_u32(bytes: &[u8], cursor: &mut usize) -> u32 {
        let mut raw = [0u8; 4];
        raw.copy_from_slice(take(bytes, cursor, 4));
        u32::from_le_bytes(raw)
    }

    fn take_u8(bytes: &[u8], cursor: &mut usize) -> u8 {
        take(bytes, cursor, 1)[0]
    }

    fn take_f64(bytes: &[u8], cursor: &mut usize) -> f64 {
        let mut raw = [0u8; 8];
        raw.copy_from_slice(take(bytes, cursor, 8));
        f64::from_le_bytes(raw)
    }

    assert!(
        bytes.len() >= GENERATED_EARTH_TABLE_MAGIC.len() + 8,
        "generated VSOP87B table is too small"
    );
    assert_eq!(
        take(bytes, &mut cursor, GENERATED_EARTH_TABLE_MAGIC.len()),
        GENERATED_EARTH_TABLE_MAGIC,
        "generated VSOP87B table has an invalid magic header"
    );
    assert_eq!(
        take_u32(bytes, &mut cursor),
        GENERATED_EARTH_TABLE_VERSION,
        "generated VSOP87B table has an unsupported version"
    );
    let section_count = take_u32(bytes, &mut cursor) as usize;
    assert!(
        section_count > 0,
        "generated VSOP87B table should contain at least one coefficient section"
    );

    let mut longitude = vec![Vec::new(); 6];
    let mut latitude = vec![Vec::new(); 6];
    let mut radius = vec![Vec::new(); 6];

    for _ in 0..section_count {
        let series = take_u8(bytes, &mut cursor) as usize;
        let power = take_u8(bytes, &mut cursor) as usize;
        let term_count = take_u32(bytes, &mut cursor) as usize;
        assert!(
            matches!(series, 1..=3),
            "generated VSOP87B table has an invalid series index {series}"
        );
        assert!(
            power < 6,
            "generated VSOP87B table has an invalid power index {power}"
        );

        let target = match series {
            1 => &mut longitude,
            2 => &mut latitude,
            3 => &mut radius,
            _ => unreachable!("series index was validated above"),
        };
        let terms = &mut target[power];
        terms.reserve(term_count);
        for _ in 0..term_count {
            terms.push(Vsop87Term {
                amplitude: take_f64(bytes, &mut cursor),
                phase: take_f64(bytes, &mut cursor),
                frequency: take_f64(bytes, &mut cursor),
            });
        }
    }

    assert_eq!(
        cursor,
        bytes.len(),
        "generated VSOP87B table contained trailing bytes"
    );

    Vsop87SeriesTables {
        longitude,
        latitude,
        radius,
    }
}

pub(crate) fn parse_vsop87b_tables(source: &str) -> Vsop87SeriesTables {
    let mut longitude = vec![Vec::new(); 6];
    let mut latitude = vec![Vec::new(); 6];
    let mut radius = vec![Vec::new(); 6];

    let mut current_series: Option<usize> = None;
    let mut current_power: Option<usize> = None;
    let mut expected_terms = 0usize;
    let mut parsed_terms = 0usize;

    let finalize_section = |current_series: Option<usize>,
                            current_power: Option<usize>,
                            expected_terms: usize,
                            parsed_terms: usize| {
        if let (Some(series), Some(power)) = (current_series, current_power) {
            assert_eq!(parsed_terms, expected_terms, "malformed VSOP87B Earth section: series {series} power {power} expected {expected_terms} terms but parsed {parsed_terms}");
        }
    };

    for line in source.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Some((series, power, terms)) = parse_header_line(line) {
            finalize_section(current_series, current_power, expected_terms, parsed_terms);
            current_series = Some(series);
            current_power = Some(power);
            expected_terms = terms;
            parsed_terms = 0;
            continue;
        }

        let (series, power) = match (current_series, current_power) {
            (Some(series), Some(power)) => (series, power),
            _ => panic!("encountered VSOP87B coefficient line before a header: {line}"),
        };

        let term = parse_term_line(line);
        match series {
            1 => longitude[power].push(term),
            2 => latitude[power].push(term),
            3 => radius[power].push(term),
            _ => panic!("unexpected VSOP87B series index {series}"),
        }
        parsed_terms += 1;
    }

    finalize_section(current_series, current_power, expected_terms, parsed_terms);

    assert_eq!(
        longitude.len(),
        6,
        "Earth longitude series should have six powers"
    );
    assert_eq!(
        latitude.len(),
        6,
        "Earth latitude series should have six powers"
    );
    assert_eq!(
        radius.len(),
        6,
        "Earth radius series should have six powers"
    );

    Vsop87SeriesTables {
        longitude,
        latitude,
        radius,
    }
}

fn parse_header_line(line: &str) -> Option<(usize, usize, usize)> {
    if !line.contains("VARIABLE") || !line.contains("*T**") {
        return None;
    }

    let tokens: Vec<&str> = line.split_whitespace().collect();
    let variable_idx = tokens.iter().position(|token| *token == "VARIABLE")?;
    let series = tokens.get(variable_idx + 1)?.parse().ok()?;
    let power_idx = tokens.iter().position(|token| token.starts_with("*T**"))?;
    let power = tokens.get(power_idx)?.strip_prefix("*T**")?.parse().ok()?;
    let terms = tokens.get(power_idx + 1)?.parse().ok()?;
    Some((series, power, terms))
}

fn parse_term_line(line: &str) -> Vsop87Term {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    assert!(tokens.len() >= 3, "malformed VSOP87B term line: {line}");
    let amplitude = tokens[tokens.len() - 3]
        .parse::<f64>()
        .unwrap_or_else(|_| panic!("invalid VSOP87B amplitude in line: {line}"));
    let phase = tokens[tokens.len() - 2]
        .parse::<f64>()
        .unwrap_or_else(|_| panic!("invalid VSOP87B phase in line: {line}"));
    let frequency = tokens[tokens.len() - 1]
        .parse::<f64>()
        .unwrap_or_else(|_| panic!("invalid VSOP87B frequency in line: {line}"));
    Vsop87Term {
        amplitude,
        phase,
        frequency,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_earth_tables_with_expected_series_counts() {
        let tables = parse_vsop87b_tables(include_str!("../data/VSOP87B.ear"));
        assert_eq!(tables.longitude.len(), 6);
        assert_eq!(tables.latitude.len(), 6);
        assert_eq!(tables.radius.len(), 6);

        let longitude_terms: Vec<usize> = tables.longitude.iter().map(Vec::len).collect();
        let latitude_terms: Vec<usize> = tables.latitude.iter().map(Vec::len).collect();
        let radius_terms: Vec<usize> = tables.radius.iter().map(Vec::len).collect();

        assert_eq!(longitude_terms, vec![623, 379, 144, 23, 11, 4]);
        assert_eq!(latitude_terms, vec![184, 134, 62, 14, 6, 2]);
        assert_eq!(radius_terms, vec![523, 290, 134, 20, 9, 2]);
    }

    #[test]
    fn parses_generated_earth_table_blob_with_expected_series_counts() {
        let tables = parse_generated_vsop87b_tables(include_bytes!("../data/VSOP87B.ear.bin"));
        assert_eq!(tables.longitude.len(), 6);
        assert_eq!(tables.latitude.len(), 6);
        assert_eq!(tables.radius.len(), 6);

        let longitude_terms: Vec<usize> = tables.longitude.iter().map(Vec::len).collect();
        let latitude_terms: Vec<usize> = tables.latitude.iter().map(Vec::len).collect();
        let radius_terms: Vec<usize> = tables.radius.iter().map(Vec::len).collect();

        assert_eq!(longitude_terms, vec![623, 379, 144, 23, 11, 4]);
        assert_eq!(latitude_terms, vec![184, 134, 62, 14, 6, 2]);
        assert_eq!(radius_terms, vec![523, 290, 134, 20, 9, 2]);
    }

    #[test]
    fn evaluates_j2000_earth_coordinates_from_the_generated_table_file() {
        let earth = earth_lbr(2_451_545.0);
        assert!((earth.longitude_rad.to_degrees() - 100.377_843_416_648_52).abs() < 1e-12);
        assert!((earth.latitude_rad.to_degrees() + 0.000_227_210_514_441_982_95).abs() < 1e-12);
        assert!((earth.radius_au - 0.983_327_682_322_294_2).abs() < 1e-15);
    }
}
