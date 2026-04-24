//! Truncated VSOP87B Jupiter coefficient tables.
//!
//! Coefficients are the leading terms from the public IMCCE/CELMECH
//! `VSOP87B.jup` file (Jupiter heliocentric spherical variables, J2000
//! ecliptic/equinox). This mirrors the inner-planet slices while complete
//! generated tables are still planned.

use crate::vsop87b_earth::{evaluate, SphericalLbr, Vsop87Term};

pub(crate) fn jupiter_lbr(julian_day_tt: f64) -> SphericalLbr {
    let t = (julian_day_tt - 2_451_545.0) / 365_250.0;
    SphericalLbr {
        longitude_rad: evaluate(
            [
                JUPITER_L_0,
                JUPITER_L_1,
                JUPITER_L_2,
                JUPITER_L_3,
                JUPITER_L_4,
                JUPITER_L_5,
            ],
            t,
        )
        .rem_euclid(core::f64::consts::TAU),
        latitude_rad: evaluate(
            [
                JUPITER_B_0,
                JUPITER_B_1,
                JUPITER_B_2,
                JUPITER_B_3,
                JUPITER_B_4,
                JUPITER_B_5,
            ],
            t,
        ),
        radius_au: evaluate(
            [
                JUPITER_R_0,
                JUPITER_R_1,
                JUPITER_R_2,
                JUPITER_R_3,
                JUPITER_R_4,
                JUPITER_R_5,
            ],
            t,
        ),
    }
}

const JUPITER_L_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 5.99546914940000e-01,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 9.69589871900000e-02,
        phase: 5.06191793158000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 5.73610142000000e-03,
        phase: 1.44406205629000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 3.06389205000000e-03,
        phase: 5.41734730184000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 9.71782960000000e-04,
        phase: 4.14264726552000e+00,
        frequency: 6.32783739313200e+02,
    },
    Vsop87Term {
        amplitude: 7.29030780000000e-04,
        phase: 3.64042916389000e+00,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 6.42639750000000e-04,
        phase: 3.41145165351000e+00,
        frequency: 1.03092774218600e+02,
    },
    Vsop87Term {
        amplitude: 3.98060640000000e-04,
        phase: 2.29376740788000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 3.88577670000000e-04,
        phase: 1.27231755835000e+00,
        frequency: 3.16391869656600e+02,
    },
    Vsop87Term {
        amplitude: 2.79646290000000e-04,
        phase: 1.78454591820000e+00,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 1.35897300000000e-04,
        phase: 5.77481040790000e+00,
        frequency: 1.58907289528380e+03,
    },
    Vsop87Term {
        amplitude: 8.24634900000000e-05,
        phase: 3.58227925840000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 8.76870400000000e-05,
        phase: 3.63000308199000e+00,
        frequency: 9.49175608969800e+02,
    },
    Vsop87Term {
        amplitude: 7.36804200000000e-05,
        phase: 5.08101194270000e+00,
        frequency: 7.35876513531800e+02,
    },
    Vsop87Term {
        amplitude: 6.26315000000000e-05,
        phase: 2.49762880700000e-02,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 6.11406200000000e-05,
        phase: 4.51319998626000e+00,
        frequency: 1.16247470440780e+03,
    },
    Vsop87Term {
        amplitude: 4.90539600000000e-05,
        phase: 1.32084470588000e+00,
        frequency: 1.10206321219400e+02,
    },
    Vsop87Term {
        amplitude: 5.30528500000000e-05,
        phase: 1.30671216791000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 5.30544100000000e-05,
        phase: 4.18625634012000e+00,
        frequency: 1.05226838318840e+03,
    },
    Vsop87Term {
        amplitude: 4.64724800000000e-05,
        phase: 4.69958103684000e+00,
        frequency: 3.93215326310000e+00,
    },
];

const JUPITER_L_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 5.29690965088140e+02,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 4.89503243000000e-03,
        phase: 4.22082939470000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 2.28917222000000e-03,
        phase: 6.02646855621000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 3.00994790000000e-04,
        phase: 4.54540782858000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 2.07209200000000e-04,
        phase: 5.45943156902000e+00,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 1.21036530000000e-04,
        phase: 1.69948160980000e-01,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 6.06798700000000e-05,
        phase: 4.42422292017000e+00,
        frequency: 1.03092774218600e+02,
    },
    Vsop87Term {
        amplitude: 5.43396800000000e-05,
        phase: 3.98480737746000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 4.23774400000000e-05,
        phase: 5.89008707199000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 2.21197400000000e-05,
        phase: 5.26766687382000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 1.98350200000000e-05,
        phase: 4.88600705699000e+00,
        frequency: 1.58907289528380e+03,
    },
    Vsop87Term {
        amplitude: 1.29576900000000e-05,
        phase: 5.55132752171000e+00,
        frequency: 3.18139373770000e+00,
    },
    Vsop87Term {
        amplitude: 1.16341600000000e-05,
        phase: 5.14506348730000e-01,
        frequency: 3.93215326310000e+00,
    },
    Vsop87Term {
        amplitude: 1.00716700000000e-05,
        phase: 4.64746900330000e-01,
        frequency: 7.35876513531800e+02,
    },
    Vsop87Term {
        amplitude: 1.17409400000000e-05,
        phase: 5.84238857133000e+00,
        frequency: 1.05226838318840e+03,
    },
    Vsop87Term {
        amplitude: 8.47762000000000e-06,
        phase: 5.75765726863000e+00,
        frequency: 1.10206321219400e+02,
    },
    Vsop87Term {
        amplitude: 8.27250000000000e-06,
        phase: 4.80311857692000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 8.29822000000000e-06,
        phase: 5.93454816950000e-01,
        frequency: 1.06649547719000e+03,
    },
    Vsop87Term {
        amplitude: 1.00386400000000e-05,
        phase: 3.14841622246000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 1.09873000000000e-05,
        phase: 5.30705242117000e+00,
        frequency: 5.15463871093000e+02,
    },
];

const JUPITER_L_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 4.72336010000000e-04,
        phase: 4.32148536482000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 3.06494360000000e-04,
        phase: 2.92977788700000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 1.48376050000000e-04,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 3.18935900000000e-05,
        phase: 1.05515491122000e+00,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 2.72890100000000e-05,
        phase: 4.84555421873000e+00,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 2.54744000000000e-05,
        phase: 3.42720888976000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 1.72104600000000e-05,
        phase: 4.18734600902000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 3.83277000000000e-06,
        phase: 5.76794364868000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 3.67514000000000e-06,
        phase: 6.05520169517000e+00,
        frequency: 1.03092774218600e+02,
    },
    Vsop87Term {
        amplitude: 3.77503000000000e-06,
        phase: 7.60508390600000e-01,
        frequency: 5.15463871093000e+02,
    },
    Vsop87Term {
        amplitude: 3.37386000000000e-06,
        phase: 3.78644856157000e+00,
        frequency: 3.18139373770000e+00,
    },
    Vsop87Term {
        amplitude: 3.08194000000000e-06,
        phase: 6.93682837900000e-01,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 2.14121000000000e-06,
        phase: 3.82958181430000e+00,
        frequency: 1.58907289528380e+03,
    },
    Vsop87Term {
        amplitude: 2.03945000000000e-06,
        phase: 5.34259263233000e+00,
        frequency: 1.06649547719000e+03,
    },
    Vsop87Term {
        amplitude: 1.97456000000000e-06,
        phase: 2.48351071790000e+00,
        frequency: 3.93215326310000e+00,
    },
    Vsop87Term {
        amplitude: 1.46156000000000e-06,
        phase: 3.81335105293000e+00,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 1.56209000000000e-06,
        phase: 1.36162315686000e+00,
        frequency: 1.05226838318840e+03,
    },
    Vsop87Term {
        amplitude: 1.29577000000000e-06,
        phase: 5.83745710707000e+00,
        frequency: 4.12371096874400e+02,
    },
    Vsop87Term {
        amplitude: 1.41825000000000e-06,
        phase: 1.63491733107000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 1.17324000000000e-06,
        phase: 1.41441723025000e+00,
        frequency: 6.25670192312400e+02,
    },
];

const JUPITER_L_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 6.50167300000000e-05,
        phase: 2.59862923650000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 1.35501200000000e-05,
        phase: 1.34692775915000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 4.70691000000000e-06,
        phase: 2.47502798748000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 4.16933000000000e-06,
        phase: 3.24456258569000e+00,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 3.52870000000000e-06,
        phase: 2.97380410245000e+00,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 1.65699000000000e-06,
        phase: 2.09182221854000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 8.67690000000000e-07,
        phase: 2.51454300081000e+00,
        frequency: 5.15463871093000e+02,
    },
    Vsop87Term {
        amplitude: 3.44580000000000e-07,
        phase: 3.82181443085000e+00,
        frequency: 1.06649547719000e+03,
    },
    Vsop87Term {
        amplitude: 2.26710000000000e-07,
        phase: 2.98178645046000e+00,
        frequency: 5.43918059096200e+02,
    },
    Vsop87Term {
        amplitude: 2.37600000000000e-07,
        phase: 1.27416115958000e+00,
        frequency: 4.12371096874400e+02,
    },
    Vsop87Term {
        amplitude: 2.85010000000000e-07,
        phase: 2.44538595164000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 1.97220000000000e-07,
        phase: 2.10936654685000e+00,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 1.77780000000000e-07,
        phase: 2.59019838502000e+00,
        frequency: 1.58907289528380e+03,
    },
    Vsop87Term {
        amplitude: 1.97090000000000e-07,
        phase: 1.40149363982000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 1.87670000000000e-07,
        phase: 1.58683219668000e+00,
        frequency: 1.03092774218600e+02,
    },
    Vsop87Term {
        amplitude: 1.70150000000000e-07,
        phase: 2.29975384867000e+00,
        frequency: 2.13406410024000e+01,
    },
    Vsop87Term {
        amplitude: 1.61790000000000e-07,
        phase: 3.15438287420000e+00,
        frequency: 6.25670192312400e+02,
    },
    Vsop87Term {
        amplitude: 1.59020000000000e-07,
        phase: 3.25713655347000e+00,
        frequency: 1.05226838318840e+03,
    },
    Vsop87Term {
        amplitude: 1.34210000000000e-07,
        phase: 2.76078519881000e+00,
        frequency: 9.59792272178000e+01,
    },
    Vsop87Term {
        amplitude: 1.32330000000000e-07,
        phase: 2.53761666317000e+00,
        frequency: 1.99072001436400e+02,
    },
];

const JUPITER_L_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 6.69505000000000e-06,
        phase: 8.52803781580000e-01,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 9.99650000000000e-07,
        phase: 7.42436519860000e-01,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 5.00300000000000e-07,
        phase: 1.65383477095000e+00,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 4.36900000000000e-07,
        phase: 5.81923759985000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 3.17940000000000e-07,
        phase: 4.85865051639000e+00,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 1.47350000000000e-07,
        phase: 4.29065528652000e+00,
        frequency: 5.15463871093000e+02,
    },
    Vsop87Term {
        amplitude: 8.40800000000000e-08,
        phase: 6.83861817680000e-01,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 4.92600000000000e-08,
        phase: 1.29899425511000e+00,
        frequency: 5.43918059096200e+02,
    },
    Vsop87Term {
        amplitude: 4.56300000000000e-08,
        phase: 2.31453670801000e+00,
        frequency: 1.06649547719000e+03,
    },
    Vsop87Term {
        amplitude: 4.25400000000000e-08,
        phase: 4.81933636910000e-01,
        frequency: 2.13406410024000e+01,
    },
    Vsop87Term {
        amplitude: 3.10000000000000e-08,
        phase: 3.00251285081000e+00,
        frequency: 4.12371096874400e+02,
    },
    Vsop87Term {
        amplitude: 2.05300000000000e-08,
        phase: 3.98541675610000e-01,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 1.76400000000000e-08,
        phase: 4.90551864257000e+00,
        frequency: 6.25670192312400e+02,
    },
    Vsop87Term {
        amplitude: 1.90100000000000e-08,
        phase: 4.25660977930000e+00,
        frequency: 1.99072001436400e+02,
    },
    Vsop87Term {
        amplitude: 1.69000000000000e-08,
        phase: 4.25228443627000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 1.34500000000000e-08,
        phase: 5.06309624095000e+00,
        frequency: 1.05226838318840e+03,
    },
    Vsop87Term {
        amplitude: 1.21100000000000e-08,
        phase: 4.71432598740000e+00,
        frequency: 9.59792272178000e+01,
    },
    Vsop87Term {
        amplitude: 1.09100000000000e-08,
        phase: 1.32037613765000e+00,
        frequency: 1.58907289528380e+03,
    },
    Vsop87Term {
        amplitude: 9.71000000000000e-09,
        phase: 5.67505418481000e+00,
        frequency: 7.28762966531000e+02,
    },
    Vsop87Term {
        amplitude: 9.35000000000000e-09,
        phase: 6.05626917469000e+00,
        frequency: 8.88656802170000e+01,
    },
];

const JUPITER_L_5: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 4.96390000000000e-07,
        phase: 5.25769924770000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 1.57750000000000e-07,
        phase: 5.24859620238000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 4.32600000000000e-08,
        phase: 2.66073892900000e-02,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 1.57300000000000e-08,
        phase: 1.18411087933000e+00,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 8.19000000000000e-09,
        phase: 5.86582284529000e+00,
        frequency: 5.43918059096200e+02,
    },
    Vsop87Term {
        amplitude: 7.24000000000000e-09,
        phase: 8.82779412850000e-01,
        frequency: 5.15463871093000e+02,
    },
    Vsop87Term {
        amplitude: 3.60000000000000e-09,
        phase: 7.83357495730000e-01,
        frequency: 1.06649547719000e+03,
    },
    Vsop87Term {
        amplitude: 3.19000000000000e-09,
        phase: 5.73095137303000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 1.98000000000000e-09,
        phase: 4.37256604900000e-02,
        frequency: 1.58907289528380e+03,
    },
    Vsop87Term {
        amplitude: 1.07000000000000e-09,
        phase: 9.29849995800000e-02,
        frequency: 1.07360902419080e+03,
    },
    Vsop87Term {
        amplitude: 7.90000000000000e-10,
        phase: 6.16619004945000e+00,
        frequency: 5.29690965094600e+02,
    },
];

const JUPITER_B_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.26861570200000e-02,
        phase: 3.55852606721000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 1.09971634000000e-03,
        phase: 3.90809347197000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 1.10090358000000e-03,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 8.10142800000000e-05,
        phase: 3.60509572885000e+00,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 6.04399600000000e-05,
        phase: 4.25883108339000e+00,
        frequency: 1.58907289528380e+03,
    },
    Vsop87Term {
        amplitude: 6.43778200000000e-05,
        phase: 3.06271192150000e-01,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 1.10688000000000e-05,
        phase: 2.98534409520000e+00,
        frequency: 1.16247470440780e+03,
    },
    Vsop87Term {
        amplitude: 9.41651000000000e-06,
        phase: 2.93619073963000e+00,
        frequency: 1.05226838318840e+03,
    },
    Vsop87Term {
        amplitude: 8.94088000000000e-06,
        phase: 1.75447402715000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 7.67280000000000e-06,
        phase: 2.15473604461000e+00,
        frequency: 6.32783739313200e+02,
    },
    Vsop87Term {
        amplitude: 9.44328000000000e-06,
        phase: 1.67522315024000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 6.84219000000000e-06,
        phase: 3.67808774854000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 6.29223000000000e-06,
        phase: 6.43432900200000e-01,
        frequency: 1.06649547719000e+03,
    },
    Vsop87Term {
        amplitude: 8.35861000000000e-06,
        phase: 5.17881977810000e+00,
        frequency: 1.03092774218600e+02,
    },
    Vsop87Term {
        amplitude: 5.31671000000000e-06,
        phase: 2.70305944444000e+00,
        frequency: 1.10206321219400e+02,
    },
    Vsop87Term {
        amplitude: 5.58524000000000e-06,
        phase: 1.35483816100000e-02,
        frequency: 8.46082834751200e+02,
    },
    Vsop87Term {
        amplitude: 4.64449000000000e-06,
        phase: 1.17337267936000e+00,
        frequency: 9.49175608969800e+02,
    },
    Vsop87Term {
        amplitude: 4.31072000000000e-06,
        phase: 2.60825022780000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 3.51433000000000e-06,
        phase: 4.61062966359000e+00,
        frequency: 2.11876386037840e+03,
    },
    Vsop87Term {
        amplitude: 1.23148000000000e-06,
        phase: 3.34968047337000e+00,
        frequency: 1.69216566950240e+03,
    },
];

const JUPITER_B_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 7.82034460000000e-04,
        phase: 1.52377859742000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 7.78990500000000e-05,
        phase: 2.59734071843000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 2.78860200000000e-05,
        phase: 4.85622679819000e+00,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 2.42972800000000e-05,
        phase: 5.45947255041000e+00,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 1.98577700000000e-05,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 7.11633000000000e-06,
        phase: 3.13688338277000e+00,
        frequency: 1.58907289528380e+03,
    },
    Vsop87Term {
        amplitude: 2.92916000000000e-06,
        phase: 5.27960297214000e+00,
        frequency: 1.06649547719000e+03,
    },
    Vsop87Term {
        amplitude: 2.57804000000000e-06,
        phase: 4.76667796123000e+00,
        frequency: 1.05226838318840e+03,
    },
    Vsop87Term {
        amplitude: 2.71233000000000e-06,
        phase: 1.01549209580000e-01,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 8.62610000000000e-07,
        phase: 1.08347893125000e+00,
        frequency: 1.03092774218600e+02,
    },
    Vsop87Term {
        amplitude: 7.96830000000000e-07,
        phase: 1.04738628033000e+00,
        frequency: 1.10206321219400e+02,
    },
    Vsop87Term {
        amplitude: 8.13690000000000e-07,
        phase: 6.39012096390000e-01,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 8.16660000000000e-07,
        phase: 4.92173680920000e-01,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 7.06130000000000e-07,
        phase: 2.82219329635000e+00,
        frequency: 6.32783739313200e+02,
    },
    Vsop87Term {
        amplitude: 6.69920000000000e-07,
        phase: 5.48215719084000e+00,
        frequency: 5.15463871093000e+02,
    },
    Vsop87Term {
        amplitude: 5.84970000000000e-07,
        phase: 3.56648086507000e+00,
        frequency: 2.11876386037840e+03,
    },
    Vsop87Term {
        amplitude: 5.19760000000000e-07,
        phase: 2.85910965609000e+00,
        frequency: 9.49175608969800e+02,
    },
    Vsop87Term {
        amplitude: 4.11880000000000e-07,
        phase: 4.75217333048000e+00,
        frequency: 5.43918059096200e+02,
    },
    Vsop87Term {
        amplitude: 3.99240000000000e-07,
        phase: 3.92433787110000e+00,
        frequency: 7.35876513531800e+02,
    },
    Vsop87Term {
        amplitude: 4.02370000000000e-07,
        phase: 1.13564290140000e+00,
        frequency: 1.16247470440780e+03,
    },
];

const JUPITER_B_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 5.49832000000000e-05,
        phase: 3.01596270062000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 6.02076000000000e-06,
        phase: 3.13358939436000e+00,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 5.02174000000000e-06,
        phase: 2.05202111599000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 4.53862000000000e-06,
        phase: 9.59124163880000e-01,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 1.15043000000000e-06,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 6.89110000000000e-07,
        phase: 3.65515676096000e+00,
        frequency: 1.06649547719000e+03,
    },
    Vsop87Term {
        amplitude: 6.70520000000000e-07,
        phase: 2.23363751256000e+00,
        frequency: 1.58907289528380e+03,
    },
    Vsop87Term {
        amplitude: 4.25550000000000e-07,
        phase: 5.21433658090000e-01,
        frequency: 1.05226838318840e+03,
    },
    Vsop87Term {
        amplitude: 3.93960000000000e-07,
        phase: 4.65314230657000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 2.34380000000000e-07,
        phase: 9.67258520730000e-01,
        frequency: 5.15463871093000e+02,
    },
    Vsop87Term {
        amplitude: 1.73830000000000e-07,
        phase: 3.03116251890000e+00,
        frequency: 5.43918059096200e+02,
    },
    Vsop87Term {
        amplitude: 6.65100000000000e-08,
        phase: 4.14899100562000e+00,
        frequency: 1.59618644228460e+03,
    },
    Vsop87Term {
        amplitude: 7.01300000000000e-08,
        phase: 2.58268666095000e+00,
        frequency: 2.11876386037840e+03,
    },
    Vsop87Term {
        amplitude: 5.38900000000000e-08,
        phase: 5.43989474079000e+00,
        frequency: 1.10206321219400e+02,
    },
    Vsop87Term {
        amplitude: 4.57800000000000e-08,
        phase: 6.21390672967000e+00,
        frequency: 1.04515483618760e+03,
    },
    Vsop87Term {
        amplitude: 4.22600000000000e-08,
        phase: 2.60174767485000e+00,
        frequency: 5.32872358832300e+02,
    },
    Vsop87Term {
        amplitude: 3.65300000000000e-08,
        phase: 5.49147329377000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 4.20800000000000e-08,
        phase: 4.53565061928000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 3.62000000000000e-08,
        phase: 2.16725398015000e+00,
        frequency: 1.16247470440780e+03,
    },
    Vsop87Term {
        amplitude: 4.34700000000000e-08,
        phase: 4.34610976020000e+00,
        frequency: 3.23505416657400e+02,
    },
];

const JUPITER_B_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.85332000000000e-06,
        phase: 4.79276761490000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 8.56680000000000e-07,
        phase: 1.40023038638000e+00,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 5.63590000000000e-07,
        phase: 2.81574766965000e+00,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 1.94350000000000e-07,
        phase: 6.25741008684000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 1.08580000000000e-07,
        phase: 2.04333735353000e+00,
        frequency: 1.06649547719000e+03,
    },
    Vsop87Term {
        amplitude: 1.44770000000000e-07,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 5.53500000000000e-08,
        phase: 2.75732372347000e+00,
        frequency: 5.15463871093000e+02,
    },
    Vsop87Term {
        amplitude: 4.93900000000000e-08,
        phase: 1.29727834284000e+00,
        frequency: 5.43918059096200e+02,
    },
    Vsop87Term {
        amplitude: 4.97000000000000e-08,
        phase: 2.56009290021000e+00,
        frequency: 1.05226838318840e+03,
    },
    Vsop87Term {
        amplitude: 4.11200000000000e-08,
        phase: 8.68404804280000e-01,
        frequency: 1.58907289528380e+03,
    },
    Vsop87Term {
        amplitude: 3.79800000000000e-08,
        phase: 2.86619114773000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 1.10700000000000e-08,
        phase: 2.66033381472000e+00,
        frequency: 1.59618644228460e+03,
    },
    Vsop87Term {
        amplitude: 1.09300000000000e-08,
        phase: 1.82485496219000e+00,
        frequency: 1.04515483618760e+03,
    },
    Vsop87Term {
        amplitude: 1.03100000000000e-08,
        phase: 2.82866669066000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 4.33000000000000e-09,
        phase: 4.07064796547000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 4.41000000000000e-09,
        phase: 1.38849268510000e+00,
        frequency: 2.11876386037840e+03,
    },
    Vsop87Term {
        amplitude: 4.27000000000000e-09,
        phase: 1.88306006605000e+00,
        frequency: 1.07360902419080e+03,
    },
    Vsop87Term {
        amplitude: 3.85000000000000e-09,
        phase: 3.43063155260000e+00,
        frequency: 1.10206321219400e+02,
    },
    Vsop87Term {
        amplitude: 2.70000000000000e-09,
        phase: 2.82543407370000e-01,
        frequency: 9.42062061969000e+02,
    },
    Vsop87Term {
        amplitude: 2.82000000000000e-09,
        phase: 2.56588914137000e+00,
        frequency: 3.23505416657400e+02,
    },
];

const JUPITER_B_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 8.96300000000000e-08,
        phase: 5.93887232380000e+00,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 5.28000000000000e-08,
        phase: 4.80778878768000e+00,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 1.16100000000000e-08,
        phase: 4.62958904380000e-01,
        frequency: 1.06649547719000e+03,
    },
    Vsop87Term {
        amplitude: 1.10400000000000e-08,
        phase: 4.53240452495000e+00,
        frequency: 5.15463871093000e+02,
    },
    Vsop87Term {
        amplitude: 1.08700000000000e-08,
        phase: 5.81789252627000e+00,
        frequency: 5.43918059096200e+02,
    },
    Vsop87Term {
        amplitude: 8.18000000000000e-09,
        phase: 1.49293156118000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 5.95000000000000e-09,
        phase: 4.58881648484000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 5.13000000000000e-09,
        phase: 4.57214361679000e+00,
        frequency: 1.05226838318840e+03,
    },
    Vsop87Term {
        amplitude: 1.79000000000000e-09,
        phase: 7.58212642800000e-01,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 1.43000000000000e-09,
        phase: 5.78292264064000e+00,
        frequency: 1.58907289528380e+03,
    },
    Vsop87Term {
        amplitude: 1.35000000000000e-09,
        phase: 1.57382028639000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 9.60000000000000e-10,
        phase: 1.04662476547000e+00,
        frequency: 1.59618644228460e+03,
    },
    Vsop87Term {
        amplitude: 7.70000000000000e-10,
        phase: 3.58780641570000e+00,
        frequency: 1.04515483618760e+03,
    },
];

const JUPITER_B_5: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 6.62000000000000e-09,
        phase: 4.10413626462000e+00,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 4.31000000000000e-09,
        phase: 8.26146637210000e-01,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 2.08000000000000e-09,
        phase: 5.49845776900000e-02,
        frequency: 5.15463871093000e+02,
    },
];

const JUPITER_R_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 5.20887429326000e+00,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 2.52093271190000e-01,
        phase: 3.49108639871000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 6.10599976000000e-03,
        phase: 3.84115365948000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 2.82029458000000e-03,
        phase: 2.57419881293000e+00,
        frequency: 6.32783739313200e+02,
    },
    Vsop87Term {
        amplitude: 1.87647346000000e-03,
        phase: 2.07590383214000e+00,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 8.67929050000000e-04,
        phase: 7.10011455450000e-01,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 7.20629740000000e-04,
        phase: 2.14657246070000e-01,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 6.55172480000000e-04,
        phase: 5.97995884790000e+00,
        frequency: 3.16391869656600e+02,
    },
    Vsop87Term {
        amplitude: 2.91345420000000e-04,
        phase: 1.67759379655000e+00,
        frequency: 1.03092774218600e+02,
    },
    Vsop87Term {
        amplitude: 3.01353350000000e-04,
        phase: 2.16132003734000e+00,
        frequency: 9.49175608969800e+02,
    },
    Vsop87Term {
        amplitude: 2.34532710000000e-04,
        phase: 3.54023522184000e+00,
        frequency: 7.35876513531800e+02,
    },
    Vsop87Term {
        amplitude: 2.22837430000000e-04,
        phase: 4.19362594399000e+00,
        frequency: 1.58907289528380e+03,
    },
    Vsop87Term {
        amplitude: 2.39472980000000e-04,
        phase: 2.74580374800000e-01,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 1.30326140000000e-04,
        phase: 2.96042965363000e+00,
        frequency: 1.16247470440780e+03,
    },
    Vsop87Term {
        amplitude: 9.70336000000000e-05,
        phase: 1.90669633585000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 1.27490230000000e-04,
        phase: 2.71550286592000e+00,
        frequency: 1.05226838318840e+03,
    },
    Vsop87Term {
        amplitude: 9.16139300000000e-05,
        phase: 4.41352953117000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 7.89451100000000e-05,
        phase: 2.47907592482000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 7.05793100000000e-05,
        phase: 2.18184839926000e+00,
        frequency: 1.26556747862640e+03,
    },
    Vsop87Term {
        amplitude: 6.13770300000000e-05,
        phase: 6.26418240033000e+00,
        frequency: 8.46082834751200e+02,
    },
];

const JUPITER_R_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.27180152000000e-02,
        phase: 2.64937512894000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 6.16618160000000e-04,
        phase: 3.00076460387000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 5.34437130000000e-04,
        phase: 3.89717383175000e+00,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 3.11851710000000e-04,
        phase: 4.88276958012000e+00,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 4.13902690000000e-04,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.18472630000000e-04,
        phase: 2.41328764459000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 9.16645400000000e-05,
        phase: 4.75978553741000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 3.17559500000000e-05,
        phase: 2.79298354393000e+00,
        frequency: 1.03092774218600e+02,
    },
    Vsop87Term {
        amplitude: 3.20348100000000e-05,
        phase: 5.21084121495000e+00,
        frequency: 7.35876513531800e+02,
    },
    Vsop87Term {
        amplitude: 3.40357700000000e-05,
        phase: 3.34689633223000e+00,
        frequency: 1.58907289528380e+03,
    },
    Vsop87Term {
        amplitude: 2.59992500000000e-05,
        phase: 3.63439058628000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 2.41212700000000e-05,
        phase: 1.46948314626000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 2.80607000000000e-05,
        phase: 3.74227009702000e+00,
        frequency: 5.15463871093000e+02,
    },
    Vsop87Term {
        amplitude: 2.67661100000000e-05,
        phase: 4.33051702874000e+00,
        frequency: 1.05226838318840e+03,
    },
    Vsop87Term {
        amplitude: 2.10039200000000e-05,
        phase: 3.92772817188000e+00,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 1.64616000000000e-05,
        phase: 5.30947626153000e+00,
        frequency: 1.06649547719000e+03,
    },
    Vsop87Term {
        amplitude: 1.64109300000000e-05,
        phase: 4.41628521235000e+00,
        frequency: 6.25670192312400e+02,
    },
    Vsop87Term {
        amplitude: 1.04976600000000e-05,
        phase: 3.16115576687000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 1.02470300000000e-05,
        phase: 2.55437897122000e+00,
        frequency: 4.12371096874400e+02,
    },
    Vsop87Term {
        amplitude: 7.40834000000000e-06,
        phase: 2.17089042827000e+00,
        frequency: 1.16247470440780e+03,
    },
];

const JUPITER_R_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 7.96449570000000e-04,
        phase: 1.35865949884000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 8.25164500000000e-05,
        phase: 5.77774460400000e+00,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 7.02994000000000e-05,
        phase: 3.27477392111000e+00,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 5.31403100000000e-05,
        phase: 1.83835031247000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 1.86118400000000e-05,
        phase: 2.97686957956000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 8.36256000000000e-06,
        phase: 4.19892740368000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 9.64420000000000e-06,
        phase: 5.48029587251000e+00,
        frequency: 5.15463871093000e+02,
    },
    Vsop87Term {
        amplitude: 4.06408000000000e-06,
        phase: 3.78248932836000e+00,
        frequency: 1.06649547719000e+03,
    },
    Vsop87Term {
        amplitude: 4.26544000000000e-06,
        phase: 2.22743958182000e+00,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 3.77334000000000e-06,
        phase: 2.24232535935000e+00,
        frequency: 1.58907289528380e+03,
    },
    Vsop87Term {
        amplitude: 4.97914000000000e-06,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 3.39124000000000e-06,
        phase: 6.12690872435000e+00,
        frequency: 6.25670192312400e+02,
    },
    Vsop87Term {
        amplitude: 3.62961000000000e-06,
        phase: 5.36776401268000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 3.42139000000000e-06,
        phase: 6.09909325177000e+00,
        frequency: 1.05226838318840e+03,
    },
    Vsop87Term {
        amplitude: 2.79940000000000e-06,
        phase: 4.26158071104000e+00,
        frequency: 4.12371096874400e+02,
    },
    Vsop87Term {
        amplitude: 3.32558000000000e-06,
        phase: 3.32561805000000e-03,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 2.29775000000000e-06,
        phase: 7.05108404370000e-01,
        frequency: 7.35876513531800e+02,
    },
    Vsop87Term {
        amplitude: 2.00884000000000e-06,
        phase: 3.06805028347000e+00,
        frequency: 5.43918059096200e+02,
    },
    Vsop87Term {
        amplitude: 1.99660000000000e-06,
        phase: 4.42869041267000e+00,
        frequency: 1.03092774218600e+02,
    },
    Vsop87Term {
        amplitude: 2.57306000000000e-06,
        phase: 9.62674825000000e-01,
        frequency: 6.32783739313200e+02,
    },
];

const JUPITER_R_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 3.51927700000000e-05,
        phase: 6.05800355513000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 1.07328100000000e-05,
        phase: 1.67319166156000e+00,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 9.15630000000000e-06,
        phase: 1.41326157617000e+00,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 3.41654000000000e-06,
        phase: 5.22945327870000e-01,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 2.54881000000000e-06,
        phase: 1.19631092831000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 2.21477000000000e-06,
        phase: 9.52343043510000e-01,
        frequency: 5.15463871093000e+02,
    },
    Vsop87Term {
        amplitude: 6.90200000000000e-07,
        phase: 2.26889455907000e+00,
        frequency: 1.06649547719000e+03,
    },
    Vsop87Term {
        amplitude: 8.97770000000000e-07,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 5.78850000000000e-07,
        phase: 1.41227055539000e+00,
        frequency: 5.43918059096200e+02,
    },
    Vsop87Term {
        amplitude: 5.77000000000000e-07,
        phase: 5.25648057040000e-01,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 5.12130000000000e-07,
        phase: 5.97994255422000e+00,
        frequency: 4.12371096874400e+02,
    },
    Vsop87Term {
        amplitude: 4.69680000000000e-07,
        phase: 1.57861666908000e+00,
        frequency: 6.25670192312400e+02,
    },
    Vsop87Term {
        amplitude: 4.27440000000000e-07,
        phase: 6.11814173992000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 3.74440000000000e-07,
        phase: 1.18048940249000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 3.38160000000000e-07,
        phase: 1.66573652907000e+00,
        frequency: 1.05226838318840e+03,
    },
    Vsop87Term {
        amplitude: 3.11660000000000e-07,
        phase: 1.04468072620000e+00,
        frequency: 1.58907289528380e+03,
    },
    Vsop87Term {
        amplitude: 2.99430000000000e-07,
        phase: 4.63498871771000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 3.35580000000000e-07,
        phase: 8.48538791700000e-01,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 2.07090000000000e-07,
        phase: 2.50340319894000e+00,
        frequency: 7.28762966531000e+02,
    },
    Vsop87Term {
        amplitude: 1.44700000000000e-07,
        phase: 9.61114605060000e-01,
        frequency: 5.08350324092200e+02,
    },
];

const JUPITER_R_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.28623000000000e-06,
        phase: 8.34760889500000e-02,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 1.13458000000000e-06,
        phase: 4.24818938180000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 8.27040000000000e-07,
        phase: 3.29801136583000e+00,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 3.78970000000000e-07,
        phase: 2.73402665560000e+00,
        frequency: 5.15463871093000e+02,
    },
    Vsop87Term {
        amplitude: 2.67130000000000e-07,
        phase: 5.68996992467000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 1.78080000000000e-07,
        phase: 5.40366594364000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 1.25640000000000e-07,
        phase: 6.00543529469000e+00,
        frequency: 5.43918059096200e+02,
    },
    Vsop87Term {
        amplitude: 9.27200000000000e-08,
        phase: 7.56192604040000e-01,
        frequency: 1.06649547719000e+03,
    },
    Vsop87Term {
        amplitude: 8.14100000000000e-08,
        phase: 5.68230705037000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 6.17400000000000e-08,
        phase: 5.10190413726000e+00,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 6.92000000000000e-08,
        phase: 1.42214334807000e+00,
        frequency: 4.12371096874400e+02,
    },
    Vsop87Term {
        amplitude: 5.32700000000000e-08,
        phase: 3.33829390777000e+00,
        frequency: 6.25670192312400e+02,
    },
    Vsop87Term {
        amplitude: 2.89500000000000e-08,
        phase: 3.38407751603000e+00,
        frequency: 1.05226838318840e+03,
    },
    Vsop87Term {
        amplitude: 2.69600000000000e-08,
        phase: 4.18310762577000e+00,
        frequency: 7.28762966531000e+02,
    },
    Vsop87Term {
        amplitude: 2.43500000000000e-08,
        phase: 2.96139551556000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 2.17600000000000e-08,
        phase: 6.21232313303000e+00,
        frequency: 1.58907289528380e+03,
    },
    Vsop87Term {
        amplitude: 2.00800000000000e-08,
        phase: 3.13891134942000e+00,
        frequency: 1.04515483618760e+03,
    },
    Vsop87Term {
        amplitude: 1.81700000000000e-08,
        phase: 2.74670205576000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 1.88300000000000e-08,
        phase: 1.87835568033000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 1.50100000000000e-08,
        phase: 1.26929907808000e+00,
        frequency: 1.59618644228460e+03,
    },
];

const JUPITER_R_5: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.11930000000000e-07,
        phase: 4.74280611863000e+00,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 4.28800000000000e-08,
        phase: 5.90497787277000e+00,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 2.00400000000000e-08,
        phase: 3.65178377123000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 2.11800000000000e-08,
        phase: 5.57290745004000e+00,
        frequency: 5.15463871093000e+02,
    },
    Vsop87Term {
        amplitude: 1.90800000000000e-08,
        phase: 4.29659647286000e+00,
        frequency: 5.43918059096200e+02,
    },
    Vsop87Term {
        amplitude: 1.53400000000000e-08,
        phase: 5.46373729640000e+00,
        frequency: 1.06649547719000e+03,
    },
    Vsop87Term {
        amplitude: 1.59600000000000e-08,
        phase: 4.11045079899000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 1.30100000000000e-08,
        phase: 3.72955393027000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 1.03300000000000e-08,
        phase: 4.50671820436000e+00,
        frequency: 5.29690965094600e+02,
    },
];
