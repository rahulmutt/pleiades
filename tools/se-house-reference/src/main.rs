use swisseph::swe2::houses2;
use swisseph::{AscMc, Cusp, HouseSystemKind};

/// Mapping from pleiades HouseSystem identifier string to swisseph HouseSystemKind.
/// Meridian and Axial both map to AxialRotationSystemMeridianSystemZariel ('X') —
/// they are identical in SE. Both still get separate corpus rows.
const HSYS: &[(&str, HouseSystemKind)] = &[
    ("Placidus", HouseSystemKind::Placidus),
    ("Koch", HouseSystemKind::Koch),
    ("Porphyry", HouseSystemKind::Porphyrius),
    ("Regiomontanus", HouseSystemKind::Regiomontanus),
    ("Campanus", HouseSystemKind::Campanus),
    ("Equal", HouseSystemKind::Equal),
    ("WholeSign", HouseSystemKind::WholeSign),
    ("Alcabitius", HouseSystemKind::Alcabitus),
    (
        "Meridian",
        HouseSystemKind::AxialRotationSystemMeridianSystemZariel,
    ),
    (
        "Axial",
        HouseSystemKind::AxialRotationSystemMeridianSystemZariel,
    ),
    ("Topocentric", HouseSystemKind::PolichPageTopocentricSystem),
    ("Morinus", HouseSystemKind::Morinus),
    ("EqualMidheaven", HouseSystemKind::EqualMc),
    ("EqualAries", HouseSystemKind::Equal1Aries),
    ("Vehlow", HouseSystemKind::VehlowEqual),
    ("Sripati", HouseSystemKind::Sripati),
    ("Carter", HouseSystemKind::CarterPoliEquatorial),
    ("Horizon", HouseSystemKind::AzimuthalHorizontalSystem),
    ("Apc", HouseSystemKind::ApcHouses),
    (
        "KrusinskiPisaGoelzer",
        HouseSystemKind::KrusinskiPisaGoelzer,
    ),
    (
        "Sunshine",
        HouseSystemKind::SunshineMakranskySolutionTreindl,
    ),
    (
        "PullenSd",
        HouseSystemKind::PullenSdSinusoidalDeltaExNeoPorphyry,
    ),
    ("PullenSr", HouseSystemKind::PullenSrSinusoidalRatio),
];

/// SE writes 37 doubles for Gauquelin (`G`): index 0 is unused, 1..=36 are the
/// sector cusps. The safe `swisseph` wrapper uses a 13-element buffer and would
/// overflow, so call the raw FFI directly with a 37-element buffer.
fn gauquelin_sectors(jd: f64, lat: f64, lon: f64) -> ([f64; 36], f64, f64) {
    let mut cusps = [0.0f64; 37];
    let mut ascmc = [0.0f64; 10];
    unsafe {
        libswisseph_sys::swe_houses(
            jd,
            lat,
            lon,
            b'G' as i32,
            cusps.as_mut_ptr(),
            ascmc.as_mut_ptr(),
        );
    }
    let mut sectors = [0.0f64; 36];
    sectors.copy_from_slice(&cusps[1..=36]);
    (sectors, ascmc[0], ascmc[1])
}

fn main() {
    let out_dir = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());

    // 6 fixtures × 23 systems = 138 data rows.
    // In-band latitudes only (northern 0, 40, 55, 66 and southern -33).
    // Strict-rejection latitudes (70, 80) are NOT included here — they are
    // asserted by the gate (Tasks 8/9).
    let fixtures: &[(&str, f64, f64, f64, f64)] = &[
        ("c0_lat00", 2_451_545.0, 0.0, 0.0, 0.0),
        ("c1_lat40", 2_451_545.0, 40.0, 0.0, 0.0),
        ("c2_lat55", 2_451_545.0, 55.0, 0.0, 0.0),
        ("c3_lat66", 2_451_545.0, 66.0, 0.0, 0.0),
        ("c4_lat40_e2", 2_433_283.0, 40.0, 30.0, 0.0),
        ("c5_lat33s", 2_451_545.0, -33.0, 20.0, 0.0),
    ];

    // Write cusps.csv
    let mut cusps_out = String::new();
    cusps_out.push_str(
        "chart_id,jd_ut,lat_deg,lon_deg,elev_m,system_code,\
         c1,c2,c3,c4,c5,c6,c7,c8,c9,c10,c11,c12,asc,mc\n",
    );
    for &(id, jd, lat, lon, elev) in fixtures {
        for &(name, ref hsys) in HSYS {
            let (cusp, ascmc): (Cusp, AscMc) = houses2(jd, lat, lon, hsys.clone());
            cusps_out.push_str(&format!(
                "{},{},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}\n",
                id, jd, lat, lon, elev, name,
                cusp.first, cusp.second, cusp.third, cusp.fourth,
                cusp.fifth, cusp.sixth, cusp.seventh, cusp.eighth,
                cusp.ninth, cusp.tenth, cusp.eleventh, cusp.twelfth,
                ascmc.ascendant, ascmc.mc,
            ));
        }
    }
    std::fs::write(format!("{out_dir}/cusps.csv"), cusps_out).expect("write cusps.csv");

    // Write sectors.csv
    let mut sectors_out = String::new();
    sectors_out.push_str("chart_id,jd_ut,lat_deg,lon_deg,elev_m,system_code,n_sectors");
    for i in 1..=36 {
        sectors_out.push_str(&format!(",s{i}"));
    }
    sectors_out.push('\n');
    for &(id, jd, lat, lon, elev) in fixtures {
        let (sectors, _asc, _mc) = gauquelin_sectors(jd, lat, lon);
        sectors_out.push_str(&format!("{id},{jd},{lat},{lon},{elev},Gauquelin,36"));
        for s in sectors {
            sectors_out.push_str(&format!(",{s:.6}"));
        }
        sectors_out.push('\n');
    }
    std::fs::write(format!("{out_dir}/sectors.csv"), sectors_out).expect("write sectors.csv");

    // Write angles.csv — one row per fixture (house-system-independent values).
    //
    // API choice: swisseph::AscMc (returned by the safe `houses2` wrapper) cleanly
    // exposes all 8 fields including armc, vertex, equatorial_ascendant,
    // co_ascendant_wk (Koch), co_ascendant_mm (Munkasey), and polar_ascendant.
    // We call houses2 once per fixture with Placidus (ascmc extras are house-system-
    // independent); we use libswisseph_sys::safe::swe_sidtime (safe wrapper, no
    // unsafe needed) for GAST in hours.
    let mut angles_out = String::new();
    angles_out.push_str(
        "chart_id,jd_ut,lat_deg,lon_deg,\
         armc,vertex,equatorial_ascendant,coasc_koch,coasc_munkasey,polar_asc,\
         sidtime_gast_hours\n",
    );
    for &(id, jd, lat, lon, _elev) in fixtures {
        let (_cusp, ascmc): (Cusp, AscMc) =
            houses2(jd, lat, lon, HouseSystemKind::Placidus);
        let sidtime = libswisseph_sys::safe::swe_sidtime(jd);
        angles_out.push_str(&format!(
            "{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.9}\n",
            id,
            jd,
            lat,
            lon,
            ascmc.armc,
            ascmc.vertex,
            ascmc.equatorial_ascendant,
            ascmc.co_ascendant_wk,
            ascmc.co_ascendant_mm,
            ascmc.polar_ascendant,
            sidtime,
        ));
    }
    std::fs::write(format!("{out_dir}/angles.csv"), angles_out).expect("write angles.csv");
}
