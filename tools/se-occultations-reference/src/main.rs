//! Emits a Swiss Ephemeris **lunar-occultation** reference corpus for the
//! upcoming `validate-occultations` gate (SP-6: a `swe_lun_occult_*` analogue).
//!
//! One CSV plus a manifest:
//!   * `occultations.csv` — two kinds of rows (`mode` column):
//!       - `loc`  — local circumstances of a lunar occultation of a target
//!                  (bright star or planet) at a given observer, from
//!                  `swe_lun_occult_when_loc` evaluated with `SE_ECL_ONE_TRY`
//!                  (the single lunar conjunction nearest `jd_tt`): maximum
//!                  instant + contact times C1..C4, plus the covered-diameter
//!                  fraction (magnitude, attr[0]) and covered-disc fraction
//!                  (obscuration, attr[2]) SE returns from the same call. When
//!                  that conjunction is NOT an occultation at the observer
//!                  (observer beyond the graze limit), SE returns 0 and the row
//!                  is `occ_type=0` (Miss) with all time/attr fields `-1`.
//!       - `glob` — the next occultation *anywhere on Earth* from
//!                  `swe_lun_occult_when_glob` (max instant, tret[0]), localized
//!                  by `swe_lun_occult_where` at that instant (sub-lunar
//!                  lon/lat + the same magnitude/obscuration attrs), with the
//!                  central-vs-noncentral classification.
//!
//! Every `loc` row is anchored to a real global event: the tool finds the event
//! with `when_glob`, localizes its central point with `where`, then evaluates
//! `when_loc` (ONE_TRY) at observers derived from that central point, scanning
//! latitude northward to deterministically capture a Total (near the central
//! line), a Grazing (at the north graze limit), and a Miss (just beyond it).
//! ONE_TRY keeps every `when_loc` bounded to that one conjunction, so the search
//! never runs off the end of the Moshier ephemeris.
//!
//!   * `manifest.txt` — row count + `fnv1a64` checksum of the CSV bytes + the
//!     SE version string + the flags/conventions below.
//!
//! **Note on `swe_lun_occult_how`.** SP-6's brief listed a fourth call,
//! `swe_lun_occult_how`, but that function **does not exist** in Swiss
//! Ephemeris 2.10.03 (the version vendored by `libswisseph-sys` 0.1.2): it is
//! declared nowhere in `swephexp.h` and defined nowhere in `swecl.c` (only the
//! *eclipse* analogues `swe_sol_eclipse_how` / `swe_lun_eclipse_how` exist).
//! It is unnecessary: the magnitude/obscuration "how" attributes are already
//! returned by `swe_lun_occult_when_loc` (for `loc` rows) and
//! `swe_lun_occult_where` (for `glob` rows), because both internally call SE's
//! private `eclipse_how()`. This tool therefore uses **three** SE occultation
//! calls, verified argument-for-argument against the vendored `swecl.c`.
//!
//! Flags (SE default apparent output minus gravitational deflection):
//! `SEFLG_MOSEPH | SEFLG_NOGDEFL` = `4 | 512` = `516`. Moshier is KERNEL-FREE —
//! no `.se1` data files, no network fetch. Bright-star targets do require SE's
//! fixed-star catalog `sefstars.txt`, bundled under this tool's `data/` dir and
//! found via the default `--ephe` path.
//!
//! `swe_lun_occult_when_loc(tjd_start, ipl, starname, ifl, geopos, tret, attr,
//! backward, serr)` — `geopos=[lon_deg,lat_deg,elev_m]` (INPUT); `tret[0]`=max,
//! `tret[1..4]`=C1..C4 (all UT); `attr[0]`=fraction of target diameter covered
//! (magnitude), `attr[2]`=fraction of target disc covered (obscuration),
//! `attr[5]`=true altitude of target above horizon.  (No `ifltype` parameter —
//! the brief's signature was wrong; verified against swecl.c:2071.)
//!
//! `swe_lun_occult_when_glob(tjd_start, ipl, starname, ifl, ifltype, tret,
//! backward, serr)` — `tret[0]`=time of maximum occultation (UT), globally.
//!
//! `swe_lun_occult_where(tjd_ut, ipl, starname, ifl, geopos, attr, serr)` —
//! `geopos[0]`=lon, `geopos[1]`=lat of the sub-lunar/central point (OUTPUT);
//! `attr` as above. `retflag & SE_ECL_CENTRAL` (=1) marks a central occultation.
//!
//! Times: every SE `_ut` instant is converted to **TDB** exactly once, at
//! emission, via `jd_tdb = jd_ut + swe_deltat(jd_ut)` (SE's committed ΔT; TT and
//! TDB agree to <2 ms, which the whole pleiades stack treats as identical).
//! Every `jd_tt`/`max_jd`/`c*_jd` column is written in TDB.
//!
//! Fail-closed: every SE return code / `serr` is checked; any error aborts the
//! run (panic with `serr`) and every emitted numeric is asserted finite before
//! the row is appended — no partial or bogus rows. Two deliberate benign
//! outcomes are NOT errors: `when_loc` returning 0 (observer beyond the graze
//! limit → Miss row), and the non-occultable-star row (Sirius), where SE
//! *correctly* returns ERR "occultation never occurs" because the star's
//! ecliptic latitude exceeds the ~7° hard limit — exactly the fast-reject the
//! gate must pin, so the tool emits an explicit `occ_type=0` "no event" row for
//! it (and panics if SE *fails* to reject it).
//!
//! Usage:
//!   se-occultations-reference --dry-run          # print CSV+manifest, no writes
//!   se-occultations-reference --out <dir>        # write the two files
//! Requires libclang + LIBCLANG_PATH to build. NOT needed to run the gate.

use std::env;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;

use libswisseph_sys::raw::{
    swe_calc, swe_deltat, swe_fixstar, swe_lun_occult_when_glob, swe_lun_occult_when_loc,
    swe_lun_occult_where, swe_pheno, swe_set_ephe_path, swe_set_topo, swe_version,
};

// SE default apparent minus gravitational deflection: SEFLG_MOSEPH=4,
// SEFLG_NOGDEFL=512.
const IFLAG: c_int = 4 | 512;

// Eclipse/occultation return-flag bits (swephexp.h:307-317).
const SE_ECL_CENTRAL: i32 = 1;
const SE_ECL_TOTAL: i32 = 4;
const SE_ECL_PARTIAL: i32 = 16;
// One-conjunction-only search flag, passed via `backward` (swephexp.h:331).
const SE_ECL_ONE_TRY: c_int = 32 * 1024;

// Output-coordinate flags (swephexp.h).
const SEFLG_EQUATORIAL: c_int = 2048;
const SEFLG_TOPCTR: c_int = 32 * 1024;

// SE body numbers.
const SE_MOON: c_int = 1;

/// FNV-1a 64-bit over the CSV bytes — identical to the sibling harnesses'
/// `fnv1a64`, so the manifest checksum matches the repo's scheme byte-for-byte.
fn fnv1a64(text: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0001_0000_01b3;
    let mut hash = FNV_OFFSET_BASIS;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn serr_string(serr: &[c_char]) -> String {
    unsafe { CStr::from_ptr(serr.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

fn se_version() -> String {
    let mut buf = [0 as c_char; 256];
    unsafe {
        swe_version(buf.as_mut_ptr());
        CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned()
    }
}

/// UT Julian day -> TDB (≈TT) via SE's committed ΔT. One conversion per instant.
fn ut_to_tdb(jd_ut: f64) -> f64 {
    jd_ut + unsafe { swe_deltat(jd_ut) }
}

/// Map an SE occultation return flag to the corpus `occ_type`:
/// 2 = Total (target fully behind the Moon), 1 = Grazing/partial, 0 = Miss.
fn occ_type_from_retflag(rf: i32) -> i32 {
    if rf & SE_ECL_TOTAL != 0 {
        2
    } else if rf & SE_ECL_PARTIAL != 0 {
        1
    } else {
        0
    }
}

/// Lunar semidiameter (degrees) at a TDB instant, computed independently via
/// `swe_pheno(Moon).attr[3]` (apparent diameter, degrees) / 2 — a wiring
/// sanity check, not written to the CSV.
fn lunar_semidiameter_deg(jd_tdb: f64) -> f64 {
    let mut attr = [0.0_f64; 20];
    let mut serr = [0 as c_char; 256];
    let ret = unsafe { swe_pheno(jd_tdb, SE_MOON, IFLAG, attr.as_mut_ptr(), serr.as_mut_ptr()) };
    if ret < 0 {
        panic!("swe_pheno(Moon) at jd_tdb={jd_tdb}: {}", serr_string(&serr));
    }
    attr[3] / 2.0
}

/// Geocentric or topocentric apparent equatorial-of-date (ra deg, dec deg,
/// dist AU) for a planet (`ipl >= 0`, `star` empty) or fixed star, at a TDB
/// instant. Caller must have called `swe_set_topo` first when passing
/// `SEFLG_TOPCTR`. Fail-closed: SE errors panic.
fn calc_radec(ipl: c_int, star: &str, jd_tdb: f64, iflag: c_int) -> (f64, f64, f64) {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0 as c_char; 256];
    let ret = if star.is_empty() {
        unsafe { swe_calc(jd_tdb, ipl, iflag, xx.as_mut_ptr(), serr.as_mut_ptr()) }
    } else {
        // swe_fixstar writes the resolved name back into its buffer.
        let mut buf = [0 as c_char; 64];
        for (i, b) in star.as_bytes().iter().enumerate().take(62) {
            buf[i] = *b as c_char;
        }
        unsafe {
            swe_fixstar(
                buf.as_mut_ptr(),
                jd_tdb,
                iflag,
                xx.as_mut_ptr(),
                serr.as_mut_ptr(),
            )
        }
    };
    if ret < 0 {
        panic!(
            "calc_radec(ipl={ipl}, star={star:?}, jd={jd_tdb}, iflag={iflag}): {}",
            serr_string(&serr)
        );
    }
    (xx[0], xx[1], xx[2])
}

/// SE Delta-T (SECONDS) at a TDB instant: evaluate swe_deltat (which takes
/// UT and returns DAYS) with one fixed-point refinement of the UT argument.
fn deltat_seconds_at_tdb(jd_tdb: f64) -> f64 {
    let dt0 = unsafe { swe_deltat(jd_tdb) };
    let dt = unsafe { swe_deltat(jd_tdb - dt0) };
    dt * 86_400.0
}

/// KNOWN GAP 3 differential fixture: for each geometric-miss loc row in the
/// committed corpus (occ_type 0 with a @center/@graze sibling sharing the
/// (se_body, star, jd_tt) key — Sirius never-rows have no sibling and are
/// skipped), emit SE's stage intermediates at the sibling's real max_jd,
/// geocentric AND topocentric at the miss observer.
fn build_diagnosis_csv(corpus: &str) -> (String, usize) {
    use std::collections::BTreeMap;
    let mut anchors: BTreeMap<(String, String, String), f64> = BTreeMap::new();
    let data_lines: Vec<&str> = corpus
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("label,"))
        .collect();
    for l in &data_lines {
        let f: Vec<&str> = l.split(',').collect();
        assert_eq!(f.len(), 19, "corpus schema drift: {l}");
        if f[1] != "loc" {
            continue;
        }
        let occ_type: i64 = f[15].parse().expect("occ_type");
        if occ_type == 0 {
            continue;
        }
        anchors.insert(
            (f[2].to_string(), f[3].to_string(), f[4].to_string()),
            f[8].parse().expect("max_jd"),
        );
    }
    let mut out = String::from(
        "# KNOWN GAP 3 differential fixture: SE stage intermediates at each geometric-miss \
         loc row's sibling conjunction (anchor_jd_tdb = sibling @center/@graze max_jd). \
         swe_calc/swe_fixstar with iflag base 516 (SEFLG_MOSEPH|SEFLG_NOGDEFL) | SEFLG_EQUATORIAL \
         (| SEFLG_TOPCTR after swe_set_topo for the *_topo columns). RA/Dec degrees \
         apparent-of-date, dist AU, deltat_sec = SE Delta-T seconds at the anchor, \
         moon_sd_topo_deg = swe_pheno topocentric attr[3]/2.\n\
         label,se_body,star,anchor_jd_tdb,lat,lon,deltat_sec,\
         moon_geo_ra,moon_geo_dec,moon_geo_dist,moon_topo_ra,moon_topo_dec,moon_topo_dist,\
         tgt_geo_ra,tgt_geo_dec,tgt_geo_dist,tgt_topo_ra,tgt_topo_dec,tgt_topo_dist,\
         moon_sd_topo_deg\n",
    );
    let mut rows = 0usize;
    for l in &data_lines {
        let f: Vec<&str> = l.split(',').collect();
        if f[1] != "loc" || f[15].parse::<i64>().expect("occ_type") != 0 {
            continue;
        }
        let key = (f[2].to_string(), f[3].to_string(), f[4].to_string());
        let Some(&anchor) = anchors.get(&key) else {
            continue; // never-occultable (Sirius) rows: no sibling, not part of GAP 3
        };
        let (label, star) = (f[0], f[3].trim());
        let ipl: c_int = f[2].parse::<i64>().expect("se_body") as c_int;
        let ipl = if star.is_empty() { ipl } else { 0 };
        let (lat, lon): (f64, f64) = (f[5].parse().expect("lat"), f[6].parse().expect("lon"));
        let dt_sec = deltat_seconds_at_tdb(anchor);
        let geo = IFLAG | SEFLG_EQUATORIAL;
        let topo = geo | SEFLG_TOPCTR;
        let mg = calc_radec(SE_MOON, "", anchor, geo);
        let tg = calc_radec(ipl, star, anchor, geo);
        unsafe { swe_set_topo(lon, lat, 0.0) };
        let mt = calc_radec(SE_MOON, "", anchor, topo);
        let tt = calc_radec(ipl, star, anchor, topo);
        let sd = {
            let mut attr = [0.0_f64; 20];
            let mut serr = [0 as c_char; 256];
            let ret =
                unsafe { swe_pheno(anchor, SE_MOON, topo, attr.as_mut_ptr(), serr.as_mut_ptr()) };
            if ret < 0 {
                panic!("swe_pheno topo Moon at {anchor}: {}", serr_string(&serr));
            }
            attr[3] / 2.0
        };
        out.push_str(&format!(
            "{label},{},{star},{anchor:.9},{lat:.6},{lon:.6},{dt_sec:.6},\
             {:.9},{:.9},{:.9},{:.9},{:.9},{:.9},\
             {:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{sd:.9}\n",
            f[2], mg.0, mg.1, mg.2, mt.0, mt.1, mt.2, tg.0, tg.1, tg.2, tt.0, tt.1, tt.2,
        ));
        rows += 1;
    }
    (out, rows)
}

/// A star target (name + SE fixstar catalog name) or a planet (SE body number).
#[derive(Clone, Copy)]
enum Target {
    Star(&'static str, &'static str), // (label, SE fixstar name)
    Planet(&'static str, c_int),      // (label, SE body number)
}

impl Target {
    fn label(&self) -> &'static str {
        match self {
            Target::Star(l, _) | Target::Planet(l, _) => l,
        }
    }
    /// (se_body column, star column) per the CSV schema.
    fn columns(&self) -> (c_int, &'static str) {
        match self {
            Target::Star(_, name) => (-1, name),
            Target::Planet(_, ipl) => (*ipl, ""),
        }
    }
    /// (ipl arg, starname CString) for the SE calls.
    fn se_args(&self) -> (c_int, CString) {
        match self {
            Target::Star(_, name) => (0, CString::new(*name).expect("star name has NUL")),
            Target::Planet(_, ipl) => (*ipl, CString::new("").unwrap()),
        }
    }
    fn is_planet(&self) -> bool {
        matches!(self, Target::Planet(..))
    }
}

/// (label, lon east deg, lat deg, elev m).
type Observer = (&'static str, f64, f64, f64);

/// Diagnostics collected during generation for the physical sanity report.
#[derive(Default)]
struct Diag {
    occ_miss: usize,
    occ_grazing: usize,
    occ_total: usize,
    lunar_sd: Vec<f64>,
    altitude: Vec<f64>,
    planet_total_mag: Vec<f64>,
    central_true: usize,
    central_false: usize,
}

/// starname pointer for the SE call: NULL for planets, the catalog name for stars.
fn star_ptr(target: &Target, cstr: &CString) -> *mut c_char {
    match target {
        Target::Planet(..) => ptr::null_mut(),
        Target::Star(..) => cstr.as_ptr() as *mut c_char,
    }
}

/// Local circumstances of one occultation at one observer (ONE_TRY conjunction).
struct LocCirc {
    occ_type: i32,
    central: i32,
    max_jd: f64,
    c: [f64; 4],
    magnitude: f64,
    obscuration: f64,
    altitude: f64,
    /// true when the conjunction IS an occultation at the observer (attrs valid);
    /// false = Miss (SE returned 0).
    hit: bool,
}

/// Evaluate `swe_lun_occult_when_loc` with `SE_ECL_ONE_TRY` at (lon,lat,elev),
/// anchored to the conjunction nearest `start_ut`. Fail-closed: ret<0 panics;
/// ret==0 is a benign Miss (observer beyond the graze limit).
fn compute_loc(target: &Target, lon: f64, lat: f64, elev: f64, start_ut: f64) -> LocCirc {
    let (ipl, star_cstr) = target.se_args();
    let mut geopos = [lon, lat, elev];
    let mut tret = [0.0_f64; 10];
    let mut attr = [0.0_f64; 20];
    let mut serr = [0 as c_char; 256];
    let ret = unsafe {
        swe_lun_occult_when_loc(
            start_ut,
            ipl,
            star_ptr(target, &star_cstr),
            IFLAG,
            geopos.as_mut_ptr(),
            tret.as_mut_ptr(),
            attr.as_mut_ptr(),
            SE_ECL_ONE_TRY, // backward=0 | ONE_TRY: evaluate just this conjunction
            serr.as_mut_ptr(),
        )
    };
    if ret < 0 {
        panic!(
            "swe_lun_occult_when_loc({}, ipl={ipl}, lat={lat:.3}) at {start_ut}: {}",
            target.label(),
            serr_string(&serr)
        );
    }
    if ret == 0 {
        // Miss: this conjunction is not an occultation at this observer.
        return LocCirc {
            occ_type: 0,
            central: 0,
            max_jd: -1.0,
            c: [-1.0; 4],
            magnitude: -1.0,
            obscuration: -1.0,
            altitude: f64::NAN,
            hit: false,
        };
    }
    let occ_type = occ_type_from_retflag(ret);
    // Contacts SE does not define for this event (e.g. C2/C3 totality contacts
    // for a grazing occultation) are left near 0; a real contact is a Julian day
    // ~2.4e6. Emit -1 for any implausible (<1e6) contact time.
    let contact = |raw: f64| -> f64 {
        if raw < 1.0e6 {
            -1.0
        } else {
            ut_to_tdb(raw)
        }
    };
    let circ = LocCirc {
        occ_type,
        central: i32::from(ret & SE_ECL_CENTRAL != 0),
        max_jd: contact(tret[0]),
        c: [
            contact(tret[1]),
            contact(tret[2]),
            contact(tret[3]),
            contact(tret[4]),
        ],
        magnitude: attr[0],
        obscuration: attr[2],
        altitude: attr[5],
        hit: true,
    };
    for (name, v) in [
        ("max_jd", circ.max_jd),
        ("magnitude", circ.magnitude),
        ("obscuration", circ.obscuration),
    ] {
        assert!(
            v.is_finite(),
            "non-finite {name} for {} lat={lat:.3}",
            target.label()
        );
    }
    circ
}

/// Append a `loc` row.
#[allow(clippy::too_many_arguments)]
fn push_loc_row(
    csv: &mut String,
    rows: &mut usize,
    diag: &mut Diag,
    target: &Target,
    kind: &str,
    lon: f64,
    lat: f64,
    elev: f64,
    start_tdb: f64,
    circ: &LocCirc,
) {
    let (se_body, star_col) = target.columns();
    let label = format!("{}@{kind}", target.label());

    match circ.occ_type {
        0 => diag.occ_miss += 1,
        1 => diag.occ_grazing += 1,
        _ => diag.occ_total += 1,
    }
    if circ.hit {
        diag.lunar_sd.push(lunar_semidiameter_deg(circ.max_jd));
        diag.altitude.push(circ.altitude);
        if target.is_planet() && circ.occ_type == 2 {
            diag.planet_total_mag.push(circ.magnitude);
        }
    }

    // schema: label,mode,se_body,star,jd_tt,lat,lon,elev,max_jd,c1_jd,c2_jd,c3_jd,c4_jd,
    //         magnitude,obscuration,occ_type,sublunar_lat,sublunar_lon,central
    let fmt = |v: f64| {
        if v < 0.0 {
            "-1".to_string()
        } else {
            format!("{v:.9}")
        }
    };
    csv.push_str(&format!(
        "{label},loc,{se_body},{star_col},{start_tdb:.9},{lat:.6},{lon:.6},{elev:.1},\
         {},{},{},{},{},{},{},{},-1,-1,{}\n",
        fmt(circ.max_jd),
        fmt(circ.c[0]),
        fmt(circ.c[1]),
        fmt(circ.c[2]),
        fmt(circ.c[3]),
        fmt(circ.magnitude),
        fmt(circ.obscuration),
        circ.occ_type,
        circ.central,
    ));
    *rows += 1;
}

/// Emit the explicit no-event `loc` row for a non-occultable star: SE returns
/// ERR "occultation never occurs" (ecliptic latitude beyond ~7°), the exact
/// fast-reject the gate pins. Panics if SE *fails* to reject it.
fn emit_loc_no_event(
    csv: &mut String,
    rows: &mut usize,
    diag: &mut Diag,
    target: &Target,
    obs: &Observer,
    start_ut: f64,
) {
    let label_t = target.label();
    let (se_body, star_col) = target.columns();
    let (ipl, star_cstr) = target.se_args();
    let (obs_name, lon, lat, elev) = *obs;
    assert!(!target.is_planet(), "no-event row is only for stars");

    let mut geopos = [lon, lat, elev];
    let mut tret = [0.0_f64; 10];
    let mut attr = [0.0_f64; 20];
    let mut serr = [0 as c_char; 256];
    let ret = unsafe {
        swe_lun_occult_when_loc(
            start_ut,
            ipl,
            star_ptr(target, &star_cstr),
            IFLAG,
            geopos.as_mut_ptr(),
            tret.as_mut_ptr(),
            attr.as_mut_ptr(),
            SE_ECL_ONE_TRY,
            serr.as_mut_ptr(),
        )
    };
    let msg = serr_string(&serr);
    assert!(
        ret < 0 && (msg.contains("never occurs") || msg.contains("ecl. lat")),
        "expected SE to reject non-occultable star {label_t} (ret<0, 'never occurs'), \
         got ret={ret} serr='{msg}'"
    );
    diag.occ_miss += 1;
    let start_tdb = ut_to_tdb(start_ut);
    let label = format!("{label_t}@{obs_name}");
    // occ_type=0 (Miss); all time/attr fields -1 (no event).
    csv.push_str(&format!(
        "{label},loc,{se_body},{star_col},{start_tdb:.9},{lat:.6},{lon:.6},{elev:.1},\
         -1,-1,-1,-1,-1,-1,-1,0,-1,-1,0\n"
    ));
    *rows += 1;
}

/// Emit one `glob` row and return the localized event `(max_ut, lon0, lat0)`.
/// `swe_lun_occult_when_glob` -> max; `swe_lun_occult_where` at max -> sub-lunar
/// point + attrs + central. Fail-closed on any SE error.
fn emit_glob(
    csv: &mut String,
    rows: &mut usize,
    diag: &mut Diag,
    target: &Target,
    start_ut: f64,
) -> (f64, f64, f64) {
    let label_t = target.label();
    let (se_body, star_col) = target.columns();
    let (ipl, star_cstr) = target.se_args();

    let mut tret = [0.0_f64; 10];
    let mut serr = [0 as c_char; 256];
    let ret_g = unsafe {
        swe_lun_occult_when_glob(
            start_ut,
            ipl,
            star_ptr(target, &star_cstr),
            IFLAG,
            0, // ifltype=0: any occultation type
            tret.as_mut_ptr(),
            0, // backward=0
            serr.as_mut_ptr(),
        )
    };
    if ret_g < 0 {
        panic!(
            "swe_lun_occult_when_glob({label_t}, ipl={ipl}, star='{star_col}') from {start_ut}: {}",
            serr_string(&serr)
        );
    }
    let max_ut = tret[0];
    let max_jd = ut_to_tdb(max_ut);

    let mut geopos = [0.0_f64; 20];
    let mut attr = [0.0_f64; 20];
    let mut serr2 = [0 as c_char; 256];
    let ret_w = unsafe {
        swe_lun_occult_where(
            max_ut,
            ipl,
            star_ptr(target, &star_cstr),
            IFLAG,
            geopos.as_mut_ptr(),
            attr.as_mut_ptr(),
            serr2.as_mut_ptr(),
        )
    };
    if ret_w < 0 {
        panic!(
            "swe_lun_occult_where({label_t}, ipl={ipl}) at {max_ut}: {}",
            serr_string(&serr2)
        );
    }
    let occ_type = occ_type_from_retflag(ret_w);
    let central = i32::from(ret_w & SE_ECL_CENTRAL != 0);
    let sublunar_lon = geopos[0];
    let sublunar_lat = geopos[1];
    let magnitude = attr[0];
    let obscuration = attr[2];
    let start_tdb = ut_to_tdb(start_ut);

    for (name, v) in [
        ("max_jd", max_jd),
        ("sublunar_lat", sublunar_lat),
        ("sublunar_lon", sublunar_lon),
        ("magnitude", magnitude),
        ("obscuration", obscuration),
    ] {
        assert!(v.is_finite(), "non-finite {name} for glob {label_t}");
    }

    match occ_type {
        0 => diag.occ_miss += 1,
        1 => diag.occ_grazing += 1,
        _ => diag.occ_total += 1,
    }
    diag.lunar_sd.push(lunar_semidiameter_deg(max_jd));
    if central == 1 {
        diag.central_true += 1;
    } else {
        diag.central_false += 1;
    }
    if target.is_planet() && occ_type == 2 {
        diag.planet_total_mag.push(magnitude);
    }

    // glob rows: observer/contact columns (lat,lon,elev,c1..c4) are -1.
    csv.push_str(&format!(
        "{label_t},glob,{se_body},{star_col},{start_tdb:.9},-1,-1,-1,\
         {max_jd:.9},-1,-1,-1,-1,{magnitude:.9},{obscuration:.9},\
         {occ_type},{sublunar_lat:.6},{sublunar_lon:.6},{central}\n"
    ));
    *rows += 1;
    (max_ut, sublunar_lon, sublunar_lat)
}

/// Find, localize, and characterize one occultation event: emit its `glob` row,
/// then scan observer latitude northward from the central point to emit up to
/// three `loc` rows — a central Total, a Grazing at the north limit, and a Miss
/// just beyond it. Every `when_loc` is ONE_TRY-bounded to this event.
fn process_event(
    csv: &mut String,
    rows: &mut usize,
    diag: &mut Diag,
    target: &Target,
    start_ut: f64,
) {
    let (max_ut, lon0, lat0) = emit_glob(csv, rows, diag, target, start_ut);
    let loc_start = max_ut - 0.5; // half a day before max: the ONE_TRY anchor
    let start_tdb = ut_to_tdb(loc_start);

    let mut got_center = false;
    let mut got_graze = false;
    let mut got_miss = false;
    // Scan equator-ward from the central line (away from the pole, so the graze
    // limit is always reachable): 0.25° steps up to 60°. Crossing the limit
    // yields a Grazing (occ_type 1) then a Miss (occ_type 0).
    let step = if lat0 >= 0.0 { -0.25 } else { 0.25 };
    for i in 0..=240 {
        if got_center && got_graze && got_miss {
            break;
        }
        let lat = lat0 + f64::from(i) * step;
        if !(-89.0..=89.0).contains(&lat) {
            break;
        }
        let circ = compute_loc(target, lon0, lat, 0.0, loc_start);
        if i == 0 {
            // Central line: the deepest occultation (expected Total).
            push_loc_row(
                csv, rows, diag, target, "center", lon0, lat, 0.0, start_tdb, &circ,
            );
            got_center = true;
        } else if !got_graze && circ.occ_type == 1 {
            push_loc_row(
                csv, rows, diag, target, "graze", lon0, lat, 0.0, start_tdb, &circ,
            );
            got_graze = true;
        } else if !got_miss && circ.occ_type == 0 {
            push_loc_row(
                csv, rows, diag, target, "miss", lon0, lat, 0.0, start_tdb, &circ,
            );
            got_miss = true;
        }
    }
}

// ---- Curated target x mode table ------------------------------------------

/// Occultable bright stars (near-ecliptic; all classic occultation targets).
const STARS: [Target; 4] = [
    Target::Star("Aldebaran", "Aldebaran"),
    Target::Star("Regulus", "Regulus"),
    Target::Star("Spica", "Spica"),
    Target::Star("Antares", "Antares"),
];

/// Planets whose discs give both Total and Grazing occultations.
const PLANETS: [Target; 3] = [
    Target::Planet("Venus", 3),
    Target::Planet("Jupiter", 5),
    Target::Planet("Saturn", 6),
];

/// Named observers for the non-occultable-star (Sirius) fast-reject rows.
const SIRIUS_OBSERVERS: [Observer; 3] = [
    ("Greenwich", 0.0, 51.4779, 0.0),
    ("Sydney", 151.21, -33.87, 40.0),
    ("Tokyo", 139.69, 35.69, 40.0),
];

/// Search-start epochs (UT JD): 2000-01-01, 2010-01-01, 2020-01-01.
const STAR_STARTS: [f64; 3] = [2_451_544.5, 2_455_197.5, 2_458_849.5];
const PLANET_STARTS: [f64; 2] = [2_451_544.5, 2_458_849.5];
const SIRIUS_START: f64 = 2_451_544.5;

fn build_csv(version: &str) -> (String, usize, Diag) {
    let mut csv = String::new();
    let mut rows = 0usize;
    let mut diag = Diag::default();

    csv.push_str(&format!(
        "# Source: Swiss Ephemeris {version} (libswisseph-sys 0.1.2), \
         swe_lun_occult_when_loc / swe_lun_occult_when_glob / swe_lun_occult_where, \
         iflag={IFLAG} (SEFLG_MOSEPH|SEFLG_NOGDEFL).\n"
    ));
    csv.push_str(
        "# Lunar occultations of bright stars & planets. mode=loc: local circumstances at \
         one observer from swe_lun_occult_when_loc evaluated with SE_ECL_ONE_TRY (the single \
         lunar conjunction nearest jd_tt): max + contacts C1..C4, magnitude=attr[0] (covered \
         diameter fraction), obscuration=attr[2] (covered disc fraction). occ_type=0 with all \
         time/attr fields -1 means that conjunction is not an occultation at that observer \
         (beyond the graze limit). mode=glob: next occultation anywhere \
         (swe_lun_occult_when_glob, max only) localized by swe_lun_occult_where (sub-lunar \
         lon/lat + attrs). occ_type: 2=Total, 1=Grazing, 0=Miss/no-event. central: 1 if SE \
         flags SE_ECL_CENTRAL else 0. Unused numeric fields are -1: for loc rows \
         sublunar_lat/lon=-1; for glob rows lat/lon/elev and c1..c4=-1. jd_tt is the \
         search-start epoch; jd_tt/max_jd/c*_jd are ALL in TDB (jd_ut + swe_deltat). \
         Non-occultable star (Sirius) -> occ_type=0 no-event row (SE rejects it: ecliptic \
         latitude beyond ~7 deg).\n",
    );
    csv.push_str(
        "label,mode,se_body,star,jd_tt,lat,lon,elev,max_jd,c1_jd,c2_jd,c3_jd,c4_jd,\
         magnitude,obscuration,occ_type,sublunar_lat,sublunar_lon,central\n",
    );

    // Stars: each start epoch yields one event -> 1 glob + up to 3 loc rows.
    for star in &STARS {
        for &start in &STAR_STARTS {
            process_event(&mut csv, &mut rows, &mut diag, star, start);
        }
    }
    // Planets: same, with disc geometry giving Total + Grazing.
    for planet in &PLANETS {
        for &start in &PLANET_STARTS {
            process_event(&mut csv, &mut rows, &mut diag, planet, start);
        }
    }
    // Non-occultable star Sirius (ecl. lat ~-39.6 deg): explicit no-event rows.
    let sirius = Target::Star("Sirius", "Sirius");
    for obs in &SIRIUS_OBSERVERS {
        emit_loc_no_event(&mut csv, &mut rows, &mut diag, &sirius, obs, SIRIUS_START);
    }

    (csv, rows, diag)
}

fn build_manifest(version: &str, csv: &str, rows: usize) -> String {
    let mut m = String::new();
    m.push_str("corpus: occultations\n");
    m.push_str(&format!(
        "source: Swiss Ephemeris {version} (Moshier, SEFLG_MOSEPH)\n"
    ));
    m.push_str("generator: tools/se-occultations-reference\n");
    m.push_str(&format!(
        "file: occultations.csv rows={rows} checksum={}\n",
        fnv1a64(csv)
    ));
    m.push_str(&format!("iflag: {IFLAG} (SEFLG_MOSEPH|SEFLG_NOGDEFL)\n"));
    m.push_str(
        "calls: swe_lun_occult_when_loc+SE_ECL_ONE_TRY (loc), swe_lun_occult_when_glob + \
         swe_lun_occult_where (glob)\n",
    );
    m.push_str(
        "attrs: magnitude=attr[0] (covered diameter fraction), \
         obscuration=attr[2] (covered disc fraction)\n",
    );
    m.push_str("occ_type: 2=Total, 1=Grazing, 0=Miss/no-event\n");
    m.push_str("central: 1 if SE_ECL_CENTRAL else 0\n");
    m.push_str("times: TDB (Julian days); jd_ut converted via swe_deltat at emission\n");
    m.push_str(
        "fixstars: data/sefstars.txt (Astrodienst catalog, md5 3658a5a37ef795ada934c451024801c1)\n",
    );
    m
}

struct Config {
    dry_run: bool,
    out_dir: String,
    ephe_dir: String,
    diagnosis: Option<String>,
}

/// Default ephemeris directory: this tool's own `data/` folder (holds the
/// fixed-star catalog `sefstars.txt`), resolved at COMPILE time from the
/// manifest directory so it is correct regardless of the process's cwd.
const DEFAULT_EPHE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data");

fn parse_args() -> Config {
    let mut dry_run = false;
    let mut out_dir = ".".to_string();
    let mut ephe_dir = env::var("SE_EPHE_PATH").unwrap_or_else(|_| DEFAULT_EPHE_DIR.to_string());
    let mut diagnosis = None;
    let mut args = env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--dry-run" => dry_run = true,
            "--out" => out_dir = args.next().expect("--out needs a directory"),
            "--ephe" => ephe_dir = args.next().expect("--ephe needs a directory"),
            "--diagnosis" => {
                diagnosis = Some(
                    args.next()
                        .expect("--diagnosis needs the occultations.csv path"),
                )
            }
            other if !other.starts_with("--") => out_dir = other.to_string(),
            other => panic!("unknown argument: {other}"),
        }
    }
    Config {
        dry_run,
        out_dir,
        ephe_dir,
        diagnosis,
    }
}

fn report_diag(diag: &Diag, rows: usize, version: &str, ephe: &str) {
    let sd_min = diag.lunar_sd.iter().cloned().fold(f64::INFINITY, f64::min);
    let sd_max = diag
        .lunar_sd
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let alt_min = diag.altitude.iter().cloned().fold(f64::INFINITY, f64::min);
    let alt_max = diag
        .altitude
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let below = diag.altitude.iter().filter(|a| **a < 0.0).count();
    let above = diag.altitude.iter().filter(|a| **a >= 0.0).count();
    let (pt_min, pt_max) = if diag.planet_total_mag.is_empty() {
        (f64::NAN, f64::NAN)
    } else {
        (
            diag.planet_total_mag
                .iter()
                .cloned()
                .fold(f64::INFINITY, f64::min),
            diag.planet_total_mag
                .iter()
                .cloned()
                .fold(f64::NEG_INFINITY, f64::max),
        )
    };
    eprintln!("=== occultations corpus sanity (SE {version}, ephe={ephe}) ===");
    eprintln!("rows: {rows}");
    eprintln!(
        "occ_type: Total={} Grazing={} Miss={}",
        diag.occ_total, diag.occ_grazing, diag.occ_miss
    );
    eprintln!(
        "central (glob): true={} false={}",
        diag.central_true, diag.central_false
    );
    eprintln!("lunar semidiameter deg: min={sd_min:.5} max={sd_max:.5} (expect ~0.24-0.28)");
    eprintln!(
        "target altitude deg: min={alt_min:.3} max={alt_max:.3} (above_horizon={above} below={below})"
    );
    eprintln!(
        "planet Total magnitude (attr[0]): count={} min={pt_min:.6} max={pt_max:.6}",
        diag.planet_total_mag.len()
    );
}

fn main() {
    let cfg = parse_args();

    let ephe = CString::new(cfg.ephe_dir.clone()).expect("ephe path has NUL");
    unsafe { swe_set_ephe_path(ephe.as_ptr()) };

    if let Some(corpus_path) = &cfg.diagnosis {
        let corpus = std::fs::read_to_string(corpus_path)
            .unwrap_or_else(|e| panic!("read {corpus_path}: {e}"));
        let (csv, rows) = build_diagnosis_csv(&corpus);
        assert_eq!(
            rows, 18,
            "expected 18 sibling-anchored miss rows, got {rows}"
        );
        std::fs::create_dir_all(&cfg.out_dir)
            .unwrap_or_else(|e| panic!("create_dir_all {}: {e}", cfg.out_dir));
        let path = format!("{}/graze-diagnosis.csv", cfg.out_dir);
        std::fs::write(&path, &csv).unwrap_or_else(|e| panic!("write {path}: {e}"));
        eprintln!("wrote {path} ({rows} rows); fnv1a64={}", fnv1a64(&csv));
        return;
    }

    let version = se_version();
    let (csv, rows, diag) = build_csv(&version);
    let manifest = build_manifest(&version, &csv, rows);

    if cfg.dry_run {
        print!("{csv}");
        println!("# ---- manifest.txt ----");
        print!("{manifest}");
        report_diag(&diag, rows, &version, &cfg.ephe_dir);
        eprintln!("dry-run: occultations rows={rows}");
        return;
    }

    std::fs::create_dir_all(&cfg.out_dir)
        .unwrap_or_else(|e| panic!("create_dir_all {}: {e}", cfg.out_dir));
    let csv_path = format!("{}/occultations.csv", cfg.out_dir);
    let mf_path = format!("{}/manifest.txt", cfg.out_dir);
    std::fs::write(&csv_path, &csv).unwrap_or_else(|e| panic!("write {csv_path}: {e}"));
    std::fs::write(&mf_path, &manifest).unwrap_or_else(|e| panic!("write {mf_path}: {e}"));
    report_diag(&diag, rows, &version, &cfg.ephe_dir);
    eprintln!("wrote {csv_path} ({rows} rows), {mf_path}; SE {version}");
}
