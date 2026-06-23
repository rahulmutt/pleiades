use swisseph::{HouseSystemKind, Cusp, AscMc};
use swisseph::swe2::houses2;

/// Mapping from pleiades HouseSystem identifier string to swisseph HouseSystemKind.
/// Meridian and Axial both map to AxialRotationSystemMeridianSystemZariel ('X') —
/// they are identical in SE. Both still get separate corpus rows.
const HSYS: &[(&str, HouseSystemKind)] = &[
    ("Placidus",      HouseSystemKind::Placidus),
    ("Koch",          HouseSystemKind::Koch),
    ("Porphyry",      HouseSystemKind::Porphyrius),
    ("Regiomontanus", HouseSystemKind::Regiomontanus),
    ("Campanus",      HouseSystemKind::Campanus),
    ("Equal",         HouseSystemKind::Equal),
    ("WholeSign",     HouseSystemKind::WholeSign),
    ("Alcabitius",    HouseSystemKind::Alcabitus),
    ("Meridian",      HouseSystemKind::AxialRotationSystemMeridianSystemZariel),
    ("Axial",         HouseSystemKind::AxialRotationSystemMeridianSystemZariel),
    ("Topocentric",   HouseSystemKind::PolichPageTopocentricSystem),
    ("Morinus",       HouseSystemKind::Morinus),
];

fn main() {
    // 5 fixtures × 12 systems = 60 data rows.
    // In-band latitudes only (0, 40, 55, 66). Strict-rejection latitudes (70, 80)
    // are NOT included here — they are asserted by the gate (Tasks 8/9).
    let fixtures: &[(&str, f64, f64, f64, f64)] = &[
        ("c0_lat00",    2_451_545.0,  0.0, 0.0, 0.0),
        ("c1_lat40",    2_451_545.0, 40.0, 0.0, 0.0),
        ("c2_lat55",    2_451_545.0, 55.0, 0.0, 0.0),
        ("c3_lat66",    2_451_545.0, 66.0, 0.0, 0.0),
        ("c4_lat40_e2", 2_433_283.0, 40.0, 30.0, 0.0),
    ];

    println!(
        "chart_id,jd_ut,lat_deg,lon_deg,elev_m,system_code,\
         c1,c2,c3,c4,c5,c6,c7,c8,c9,c10,c11,c12,asc,mc"
    );

    for &(id, jd, lat, lon, elev) in fixtures {
        for &(name, ref hsys) in HSYS {
            let (cusp, ascmc): (Cusp, AscMc) = houses2(jd, lat, lon, hsys.clone());
            println!(
                "{},{},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
                id, jd, lat, lon, elev, name,
                cusp.first, cusp.second, cusp.third, cusp.fourth,
                cusp.fifth, cusp.sixth, cusp.seventh, cusp.eighth,
                cusp.ninth, cusp.tenth, cusp.eleventh, cusp.twelfth,
                ascmc.ascendant, ascmc.mc,
            );
        }
    }
}
