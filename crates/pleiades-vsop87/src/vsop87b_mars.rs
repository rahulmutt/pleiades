//! Truncated VSOP87B Mars coefficient tables.
//!
//! Coefficients are the leading terms from the public IMCCE/CELMECH
//! `VSOP87B.mar` file (Mars heliocentric spherical variables, J2000
//! ecliptic/equinox). This mirrors the inner-planet slices while complete
//! generated tables are still planned.

use crate::vsop87b_earth::{evaluate, SphericalLbr, Vsop87Term};

pub(crate) fn mars_lbr(julian_day_tt: f64) -> SphericalLbr {
    let t = (julian_day_tt - 2_451_545.0) / 365_250.0;
    SphericalLbr {
        longitude_rad: evaluate(
            [MARS_L_0, MARS_L_1, MARS_L_2, MARS_L_3, MARS_L_4, MARS_L_5],
            t,
        )
        .rem_euclid(core::f64::consts::TAU),
        latitude_rad: evaluate(
            [MARS_B_0, MARS_B_1, MARS_B_2, MARS_B_3, MARS_B_4, MARS_B_5],
            t,
        ),
        radius_au: evaluate(
            [MARS_R_0, MARS_R_1, MARS_R_2, MARS_R_3, MARS_R_4, MARS_R_5],
            t,
        ),
    }
}

const MARS_L_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 6.20347711581000e+00,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.86563680930000e-01,
        phase: 5.05037100270000e+00,
        frequency: 3.34061242669980e+03,
    },
    Vsop87Term {
        amplitude: 1.10821681600000e-02,
        phase: 5.40099836344000e+00,
        frequency: 6.68122485339960e+03,
    },
    Vsop87Term {
        amplitude: 9.17984060000000e-04,
        phase: 5.75478744667000e+00,
        frequency: 1.00218372800994e+04,
    },
    Vsop87Term {
        amplitude: 2.77449870000000e-04,
        phase: 5.97049513147000e+00,
        frequency: 3.52311834900000e+00,
    },
    Vsop87Term {
        amplitude: 1.06102350000000e-04,
        phase: 2.93958560338000e+00,
        frequency: 2.28123049651060e+03,
    },
    Vsop87Term {
        amplitude: 1.23158970000000e-04,
        phase: 8.49560940020000e-01,
        frequency: 2.81092146160520e+03,
    },
    Vsop87Term {
        amplitude: 8.92678400000000e-05,
        phase: 4.15697846427000e+00,
        frequency: 1.72536522000000e-02,
    },
    Vsop87Term {
        amplitude: 8.71569100000000e-05,
        phase: 6.11005153139000e+00,
        frequency: 1.33624497067992e+04,
    },
    Vsop87Term {
        amplitude: 6.79755600000000e-05,
        phase: 3.64622296570000e-01,
        frequency: 3.98149003408200e+02,
    },
    Vsop87Term {
        amplitude: 7.77487200000000e-05,
        phase: 3.33968761376000e+00,
        frequency: 5.62184292321040e+03,
    },
    Vsop87Term {
        amplitude: 3.57507800000000e-05,
        phase: 1.66186505710000e+00,
        frequency: 2.54431441988340e+03,
    },
    Vsop87Term {
        amplitude: 4.16110800000000e-05,
        phase: 2.28149713270000e-01,
        frequency: 2.94246342329160e+03,
    },
    Vsop87Term {
        amplitude: 3.07525200000000e-05,
        phase: 8.56966141320000e-01,
        frequency: 1.91448266111600e+02,
    },
    Vsop87Term {
        amplitude: 2.62811700000000e-05,
        phase: 6.48061244650000e-01,
        frequency: 3.33708930835080e+03,
    },
    Vsop87Term {
        amplitude: 2.93754600000000e-05,
        phase: 6.07893711402000e+00,
        frequency: 6.73103028000000e-02,
    },
    Vsop87Term {
        amplitude: 2.38941400000000e-05,
        phase: 5.03896442664000e+00,
        frequency: 7.96298006816400e+02,
    },
    Vsop87Term {
        amplitude: 2.57984400000000e-05,
        phase: 2.99673615600000e-02,
        frequency: 3.34413554504880e+03,
    },
    Vsop87Term {
        amplitude: 1.52814100000000e-05,
        phase: 1.14979301996000e+00,
        frequency: 6.15153388830500e+03,
    },
    Vsop87Term {
        amplitude: 1.79880600000000e-05,
        phase: 6.56340574450000e-01,
        frequency: 5.29690965094600e+02,
    },
];

const MARS_L_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 3.34061242700512e+03,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.45755452300000e-02,
        phase: 3.60433733236000e+00,
        frequency: 3.34061242669980e+03,
    },
    Vsop87Term {
        amplitude: 1.68414711000000e-03,
        phase: 3.92318567804000e+00,
        frequency: 6.68122485339960e+03,
    },
    Vsop87Term {
        amplitude: 2.06229750000000e-04,
        phase: 4.26108844583000e+00,
        frequency: 1.00218372800994e+04,
    },
    Vsop87Term {
        amplitude: 3.45239200000000e-05,
        phase: 4.73210393190000e+00,
        frequency: 3.52311834900000e+00,
    },
    Vsop87Term {
        amplitude: 2.58633200000000e-05,
        phase: 4.60670058555000e+00,
        frequency: 1.33624497067992e+04,
    },
    Vsop87Term {
        amplitude: 8.41535000000000e-06,
        phase: 4.45864030426000e+00,
        frequency: 2.28123049651060e+03,
    },
    Vsop87Term {
        amplitude: 5.37567000000000e-06,
        phase: 5.01581256923000e+00,
        frequency: 3.98149003408200e+02,
    },
    Vsop87Term {
        amplitude: 5.20948000000000e-06,
        phase: 4.99428054039000e+00,
        frequency: 3.34413554504880e+03,
    },
    Vsop87Term {
        amplitude: 4.32635000000000e-06,
        phase: 2.56070853083000e+00,
        frequency: 1.91448266111600e+02,
    },
    Vsop87Term {
        amplitude: 4.29655000000000e-06,
        phase: 5.31645299471000e+00,
        frequency: 1.55420399434200e+02,
    },
    Vsop87Term {
        amplitude: 3.81751000000000e-06,
        phase: 3.53878166043000e+00,
        frequency: 7.96298006816400e+02,
    },
    Vsop87Term {
        amplitude: 3.28530000000000e-06,
        phase: 4.95632685192000e+00,
        frequency: 1.67030621334990e+04,
    },
    Vsop87Term {
        amplitude: 2.82795000000000e-06,
        phase: 3.15966768785000e+00,
        frequency: 2.54431441988340e+03,
    },
    Vsop87Term {
        amplitude: 2.05657000000000e-06,
        phase: 4.56889279932000e+00,
        frequency: 2.14616541647520e+03,
    },
    Vsop87Term {
        amplitude: 1.68866000000000e-06,
        phase: 1.32936559060000e+00,
        frequency: 3.33708930835080e+03,
    },
    Vsop87Term {
        amplitude: 1.57593000000000e-06,
        phase: 4.18519540728000e+00,
        frequency: 1.75153953141600e+03,
    },
    Vsop87Term {
        amplitude: 1.33686000000000e-06,
        phase: 2.23327245555000e+00,
        frequency: 9.80321068200000e-01,
    },
    Vsop87Term {
        amplitude: 1.16965000000000e-06,
        phase: 2.21414273762000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 1.17503000000000e-06,
        phase: 6.02411290806000e+00,
        frequency: 6.15153388830500e+03,
    },
];

const MARS_L_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 5.81525770000000e-04,
        phase: 2.04961712429000e+00,
        frequency: 3.34061242669980e+03,
    },
    Vsop87Term {
        amplitude: 1.34595790000000e-04,
        phase: 2.45738706163000e+00,
        frequency: 6.68122485339960e+03,
    },
    Vsop87Term {
        amplitude: 2.43257500000000e-05,
        phase: 2.79737979284000e+00,
        frequency: 1.00218372800994e+04,
    },
    Vsop87Term {
        amplitude: 4.01065000000000e-06,
        phase: 3.13581149963000e+00,
        frequency: 1.33624497067992e+04,
    },
    Vsop87Term {
        amplitude: 4.51384000000000e-06,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 2.22025000000000e-06,
        phase: 3.19437046607000e+00,
        frequency: 3.52311834900000e+00,
    },
    Vsop87Term {
        amplitude: 1.20954000000000e-06,
        phase: 5.43271286070000e-01,
        frequency: 1.55420399434200e+02,
    },
    Vsop87Term {
        amplitude: 6.29710000000000e-07,
        phase: 3.47765178989000e+00,
        frequency: 1.67030621334990e+04,
    },
    Vsop87Term {
        amplitude: 5.36440000000000e-07,
        phase: 3.54171478781000e+00,
        frequency: 3.34413554504880e+03,
    },
    Vsop87Term {
        amplitude: 3.42730000000000e-07,
        phase: 6.00208464365000e+00,
        frequency: 2.28123049651060e+03,
    },
    Vsop87Term {
        amplitude: 3.16590000000000e-07,
        phase: 4.14001980084000e+00,
        frequency: 1.91448266111600e+02,
    },
    Vsop87Term {
        amplitude: 2.98390000000000e-07,
        phase: 1.99838739380000e+00,
        frequency: 7.96298006816400e+02,
    },
    Vsop87Term {
        amplitude: 2.31720000000000e-07,
        phase: 4.33401932281000e+00,
        frequency: 2.42728603974000e+02,
    },
    Vsop87Term {
        amplitude: 2.16630000000000e-07,
        phase: 3.44500841809000e+00,
        frequency: 3.98149003408200e+02,
    },
    Vsop87Term {
        amplitude: 1.60500000000000e-07,
        phase: 6.11000263211000e+00,
        frequency: 2.14616541647520e+03,
    },
    Vsop87Term {
        amplitude: 2.03690000000000e-07,
        phase: 5.42202383442000e+00,
        frequency: 5.53569402842400e+02,
    },
    Vsop87Term {
        amplitude: 1.49240000000000e-07,
        phase: 6.09549588012000e+00,
        frequency: 3.18519202726560e+03,
    },
    Vsop87Term {
        amplitude: 1.62290000000000e-07,
        phase: 6.56851054220000e-01,
        frequency: 9.80321068200000e-01,
    },
    Vsop87Term {
        amplitude: 1.43170000000000e-07,
        phase: 2.61898820749000e+00,
        frequency: 1.34986740965880e+03,
    },
    Vsop87Term {
        amplitude: 1.44110000000000e-07,
        phase: 4.01941740099000e+00,
        frequency: 9.51718406250600e+02,
    },
];

const MARS_L_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.46786700000000e-05,
        phase: 4.44298394600000e-01,
        frequency: 3.34061242669980e+03,
    },
    Vsop87Term {
        amplitude: 6.92668000000000e-06,
        phase: 8.86798871230000e-01,
        frequency: 6.68122485339960e+03,
    },
    Vsop87Term {
        amplitude: 1.89478000000000e-06,
        phase: 1.28336839921000e+00,
        frequency: 1.00218372800994e+04,
    },
    Vsop87Term {
        amplitude: 4.16150000000000e-07,
        phase: 1.64210455567000e+00,
        frequency: 1.33624497067992e+04,
    },
    Vsop87Term {
        amplitude: 2.26600000000000e-07,
        phase: 2.05278956965000e+00,
        frequency: 1.55420399434200e+02,
    },
    Vsop87Term {
        amplitude: 8.12600000000000e-08,
        phase: 1.99049724299000e+00,
        frequency: 1.67030621334990e+04,
    },
    Vsop87Term {
        amplitude: 1.04550000000000e-07,
        phase: 1.57992093693000e+00,
        frequency: 3.52311834900000e+00,
    },
    Vsop87Term {
        amplitude: 4.90200000000000e-08,
        phase: 2.82516875010000e+00,
        frequency: 2.42728603974000e+02,
    },
    Vsop87Term {
        amplitude: 5.37900000000000e-08,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 3.78200000000000e-08,
        phase: 2.01848153986000e+00,
        frequency: 3.34413554504880e+03,
    },
    Vsop87Term {
        amplitude: 3.18100000000000e-08,
        phase: 4.59108786647000e+00,
        frequency: 3.18519202726560e+03,
    },
    Vsop87Term {
        amplitude: 3.13300000000000e-08,
        phase: 6.51413195170000e-01,
        frequency: 5.53569402842400e+02,
    },
    Vsop87Term {
        amplitude: 1.69800000000000e-08,
        phase: 5.53803382831000e+00,
        frequency: 9.51718406250600e+02,
    },
    Vsop87Term {
        amplitude: 1.52500000000000e-08,
        phase: 5.71698515888000e+00,
        frequency: 1.91448266111600e+02,
    },
    Vsop87Term {
        amplitude: 1.45100000000000e-08,
        phase: 4.60684902200000e-01,
        frequency: 7.96298006816400e+02,
    },
    Vsop87Term {
        amplitude: 1.47300000000000e-08,
        phase: 2.33727441522000e+00,
        frequency: 2.00436745601988e+04,
    },
    Vsop87Term {
        amplitude: 1.31400000000000e-08,
        phase: 5.36403056955000e+00,
        frequency: 9.80321068200000e-01,
    },
    Vsop87Term {
        amplitude: 1.17800000000000e-08,
        phase: 4.14644990348000e+00,
        frequency: 1.34986740965880e+03,
    },
    Vsop87Term {
        amplitude: 1.13800000000000e-08,
        phase: 2.37914351932000e+00,
        frequency: 6.68474797174860e+03,
    },
    Vsop87Term {
        amplitude: 1.04600000000000e-08,
        phase: 1.76915268602000e+00,
        frequency: 3.82896532223200e+02,
    },
];

const MARS_L_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.72420000000000e-07,
        phase: 5.63997742320000e+00,
        frequency: 6.68122485339960e+03,
    },
    Vsop87Term {
        amplitude: 2.55110000000000e-07,
        phase: 5.13956279086000e+00,
        frequency: 3.34061242669980e+03,
    },
    Vsop87Term {
        amplitude: 1.11470000000000e-07,
        phase: 6.03556608878000e+00,
        frequency: 1.00218372800994e+04,
    },
    Vsop87Term {
        amplitude: 3.19000000000000e-08,
        phase: 3.56206901204000e+00,
        frequency: 1.55420399434200e+02,
    },
    Vsop87Term {
        amplitude: 3.25100000000000e-08,
        phase: 1.29156164600000e-01,
        frequency: 1.33624497067992e+04,
    },
    Vsop87Term {
        amplitude: 7.90000000000000e-09,
        phase: 4.89791148610000e-01,
        frequency: 1.67030621334990e+04,
    },
    Vsop87Term {
        amplitude: 7.83000000000000e-09,
        phase: 1.31770747646000e+00,
        frequency: 2.42728603974000e+02,
    },
    Vsop87Term {
        amplitude: 5.00000000000000e-09,
        phase: 3.08200921500000e+00,
        frequency: 3.18519202726560e+03,
    },
    Vsop87Term {
        amplitude: 3.74000000000000e-09,
        phase: 2.15499052115000e+00,
        frequency: 5.53569402842400e+02,
    },
    Vsop87Term {
        amplitude: 3.38000000000000e-09,
        phase: 6.23352693699000e+00,
        frequency: 3.52311834900000e+00,
    },
    Vsop87Term {
        amplitude: 1.99000000000000e-09,
        phase: 4.53483441090000e-01,
        frequency: 3.34413554504880e+03,
    },
    Vsop87Term {
        amplitude: 1.68000000000000e-09,
        phase: 3.76870889622000e+00,
        frequency: 3.49603282613400e+03,
    },
    Vsop87Term {
        amplitude: 1.71000000000000e-09,
        phase: 8.41351336040000e-01,
        frequency: 2.00436745601988e+04,
    },
    Vsop87Term {
        amplitude: 1.15000000000000e-09,
        phase: 1.67425271260000e+00,
        frequency: 3.58334103067380e+03,
    },
    Vsop87Term {
        amplitude: 8.00000000000000e-10,
        phase: 8.57064843350000e-01,
        frequency: 6.68474797174860e+03,
    },
    Vsop87Term {
        amplitude: 9.10000000000000e-10,
        phase: 3.43800003890000e+00,
        frequency: 6.52580445396540e+03,
    },
    Vsop87Term {
        amplitude: 6.20000000000000e-10,
        phase: 4.48712111677000e+00,
        frequency: 2.78704302385740e+03,
    },
    Vsop87Term {
        amplitude: 4.30000000000000e-10,
        phase: 5.18700577258000e+00,
        frequency: 3.09788382272579e+03,
    },
    Vsop87Term {
        amplitude: 4.70000000000000e-10,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 3.70000000000000e-10,
        phase: 1.18296722277000e+00,
        frequency: 2.33842869868986e+04,
    },
];

const MARS_L_5: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 7.62000000000000e-09,
        phase: 4.03556368806000e+00,
        frequency: 6.68122485339960e+03,
    },
    Vsop87Term {
        amplitude: 5.11000000000000e-09,
        phase: 4.48770393640000e+00,
        frequency: 1.00218372800994e+04,
    },
    Vsop87Term {
        amplitude: 3.60000000000000e-09,
        phase: 5.07296615717000e+00,
        frequency: 1.55420399434200e+02,
    },
    Vsop87Term {
        amplitude: 2.02000000000000e-09,
        phase: 4.88361321440000e+00,
        frequency: 1.33624497067992e+04,
    },
    Vsop87Term {
        amplitude: 1.65000000000000e-09,
        phase: 3.48592135141000e+00,
        frequency: 3.34061242669980e+03,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-09,
        phase: 6.09065293613000e+00,
        frequency: 2.42728603974000e+02,
    },
    Vsop87Term {
        amplitude: 6.20000000000000e-10,
        phase: 5.25625355993000e+00,
        frequency: 1.67030621334990e+04,
    },
    Vsop87Term {
        amplitude: 6.40000000000000e-10,
        phase: 1.57192173495000e+00,
        frequency: 3.18519202726560e+03,
    },
    Vsop87Term {
        amplitude: 3.50000000000000e-10,
        phase: 3.68331841812000e+00,
        frequency: 5.53569402842400e+02,
    },
    Vsop87Term {
        amplitude: 1.60000000000000e-10,
        phase: 5.23951751422000e+00,
        frequency: 3.49603282613400e+03,
    },
    Vsop87Term {
        amplitude: 1.70000000000000e-10,
        phase: 5.60862454832000e+00,
        frequency: 2.00436745601988e+04,
    },
    Vsop87Term {
        amplitude: 1.60000000000000e-10,
        phase: 1.67238548050000e-01,
        frequency: 3.58334103067380e+03,
    },
    Vsop87Term {
        amplitude: 1.10000000000000e-10,
        phase: 1.93419597527000e+00,
        frequency: 6.52580445396540e+03,
    },
    Vsop87Term {
        amplitude: 7.00000000000000e-11,
        phase: 5.17525588419000e+00,
        frequency: 3.34413554504880e+03,
    },
    Vsop87Term {
        amplitude: 7.00000000000000e-11,
        phase: 2.97300501488000e+00,
        frequency: 2.78704302385740e+03,
    },
    Vsop87Term {
        amplitude: 5.00000000000000e-11,
        phase: 3.90005845640000e-01,
        frequency: 3.09788382272579e+03,
    },
    Vsop87Term {
        amplitude: 5.00000000000000e-11,
        phase: 5.60123387975000e+00,
        frequency: 6.68474797174860e+03,
    },
    Vsop87Term {
        amplitude: 4.00000000000000e-11,
        phase: 5.97749583912000e+00,
        frequency: 2.33842869868986e+04,
    },
    Vsop87Term {
        amplitude: 3.00000000000000e-11,
        phase: 2.29064254081000e+00,
        frequency: 9.86641688066520e+03,
    },
    Vsop87Term {
        amplitude: 3.00000000000000e-11,
        phase: 5.27166841040000e-01,
        frequency: 6.92395345737360e+03,
    },
];

const MARS_B_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 3.19713498600000e-02,
        phase: 3.76832042431000e+00,
        frequency: 3.34061242669980e+03,
    },
    Vsop87Term {
        amplitude: 2.98033234000000e-03,
        phase: 4.10616996305000e+00,
        frequency: 6.68122485339960e+03,
    },
    Vsop87Term {
        amplitude: 2.89104742000000e-03,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 3.13655390000000e-04,
        phase: 4.44651053090000e+00,
        frequency: 1.00218372800994e+04,
    },
    Vsop87Term {
        amplitude: 3.48410000000000e-05,
        phase: 4.78812549260000e+00,
        frequency: 1.33624497067992e+04,
    },
    Vsop87Term {
        amplitude: 4.42999000000000e-06,
        phase: 5.65233014206000e+00,
        frequency: 3.33708930835080e+03,
    },
    Vsop87Term {
        amplitude: 4.43401000000000e-06,
        phase: 5.02642622964000e+00,
        frequency: 3.34413554504880e+03,
    },
    Vsop87Term {
        amplitude: 3.99109000000000e-06,
        phase: 5.13056816928000e+00,
        frequency: 1.67030621334990e+04,
    },
    Vsop87Term {
        amplitude: 2.92506000000000e-06,
        phase: 3.79290674178000e+00,
        frequency: 2.28123049651060e+03,
    },
    Vsop87Term {
        amplitude: 1.81982000000000e-06,
        phase: 6.13648041445000e+00,
        frequency: 6.15153388830500e+03,
    },
    Vsop87Term {
        amplitude: 1.63159000000000e-06,
        phase: 4.26399640691000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 1.59678000000000e-06,
        phase: 2.23194572851000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 1.39323000000000e-06,
        phase: 2.41796458896000e+00,
        frequency: 8.96245534991020e+03,
    },
    Vsop87Term {
        amplitude: 1.49297000000000e-06,
        phase: 2.16501221175000e+00,
        frequency: 5.62184292321040e+03,
    },
    Vsop87Term {
        amplitude: 1.42686000000000e-06,
        phase: 1.18215016908000e+00,
        frequency: 3.34059517304760e+03,
    },
    Vsop87Term {
        amplitude: 1.42685000000000e-06,
        phase: 3.21292181638000e+00,
        frequency: 3.34062968035200e+03,
    },
    Vsop87Term {
        amplitude: 8.25440000000000e-07,
        phase: 5.36667920373000e+00,
        frequency: 6.68474797174860e+03,
    },
    Vsop87Term {
        amplitude: 7.36390000000000e-07,
        phase: 5.09187695770000e+00,
        frequency: 3.98149003408200e+02,
    },
    Vsop87Term {
        amplitude: 7.26600000000000e-07,
        phase: 5.53775735826000e+00,
        frequency: 6.28307584999140e+03,
    },
    Vsop87Term {
        amplitude: 8.63770000000000e-07,
        phase: 5.74429749104000e+00,
        frequency: 3.73876143010800e+03,
    },
];

const MARS_B_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.17310991000000e-03,
        phase: 6.04472194776000e+00,
        frequency: 3.34061242669980e+03,
    },
    Vsop87Term {
        amplitude: 2.09769480000000e-04,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.28347090000000e-04,
        phase: 1.60810667915000e+00,
        frequency: 6.68122485339960e+03,
    },
    Vsop87Term {
        amplitude: 3.32098100000000e-05,
        phase: 2.62947004077000e+00,
        frequency: 1.00218372800994e+04,
    },
    Vsop87Term {
        amplitude: 6.27200000000000e-06,
        phase: 3.11898601248000e+00,
        frequency: 1.33624497067992e+04,
    },
    Vsop87Term {
        amplitude: 1.01990000000000e-06,
        phase: 3.52113557592000e+00,
        frequency: 1.67030621334990e+04,
    },
    Vsop87Term {
        amplitude: 7.51070000000000e-07,
        phase: 9.59837585150000e-01,
        frequency: 3.33708930835080e+03,
    },
    Vsop87Term {
        amplitude: 2.92640000000000e-07,
        phase: 3.40307682710000e+00,
        frequency: 3.34413554504880e+03,
    },
    Vsop87Term {
        amplitude: 2.32510000000000e-07,
        phase: 3.69342549027000e+00,
        frequency: 5.62184292321040e+03,
    },
    Vsop87Term {
        amplitude: 2.21900000000000e-07,
        phase: 2.21703408598000e+00,
        frequency: 2.28123049651060e+03,
    },
    Vsop87Term {
        amplitude: 1.54540000000000e-07,
        phase: 3.89610159362000e+00,
        frequency: 2.00436745601988e+04,
    },
    Vsop87Term {
        amplitude: 1.18670000000000e-07,
        phase: 3.83861019788000e+00,
        frequency: 6.68474797174860e+03,
    },
    Vsop87Term {
        amplitude: 1.20380000000000e-07,
        phase: 2.13866775328000e+00,
        frequency: 6.15153388830500e+03,
    },
    Vsop87Term {
        amplitude: 9.69700000000000e-08,
        phase: 5.48941186798000e+00,
        frequency: 3.34062968035200e+03,
    },
    Vsop87Term {
        amplitude: 9.69700000000000e-08,
        phase: 3.45863925102000e+00,
        frequency: 3.34059517304760e+03,
    },
    Vsop87Term {
        amplitude: 1.15370000000000e-07,
        phase: 1.90395033905000e+00,
        frequency: 3.53206069281140e+03,
    },
    Vsop87Term {
        amplitude: 9.27600000000000e-08,
        phase: 7.19413124620000e-01,
        frequency: 2.94246342329160e+03,
    },
    Vsop87Term {
        amplitude: 9.24000000000000e-08,
        phase: 2.51747952408000e+00,
        frequency: 5.88492684658320e+03,
    },
    Vsop87Term {
        amplitude: 9.87600000000000e-08,
        phase: 6.13507416822000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 9.26500000000000e-08,
        phase: 4.55759125226000e+00,
        frequency: 8.96245534991020e+03,
    },
];

const MARS_B_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 8.88844600000000e-05,
        phase: 1.06196052751000e+00,
        frequency: 3.34061242669980e+03,
    },
    Vsop87Term {
        amplitude: 2.59539300000000e-05,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 9.18914000000000e-06,
        phase: 1.15384311900000e-01,
        frequency: 6.68122485339960e+03,
    },
    Vsop87Term {
        amplitude: 2.67883000000000e-06,
        phase: 7.88378930630000e-01,
        frequency: 1.00218372800994e+04,
    },
    Vsop87Term {
        amplitude: 6.69110000000000e-07,
        phase: 1.39435595847000e+00,
        frequency: 1.33624497067992e+04,
    },
    Vsop87Term {
        amplitude: 1.42670000000000e-07,
        phase: 1.87268116087000e+00,
        frequency: 1.67030621334990e+04,
    },
    Vsop87Term {
        amplitude: 7.94800000000000e-08,
        phase: 2.58819177832000e+00,
        frequency: 3.33708930835080e+03,
    },
    Vsop87Term {
        amplitude: 2.70900000000000e-08,
        phase: 2.29241371893000e+00,
        frequency: 2.00436745601988e+04,
    },
    Vsop87Term {
        amplitude: 2.91100000000000e-08,
        phase: 1.36634316448000e+00,
        frequency: 3.34413554504880e+03,
    },
    Vsop87Term {
        amplitude: 2.52800000000000e-08,
        phase: 6.00423798411000e+00,
        frequency: 3.49603282613400e+03,
    },
    Vsop87Term {
        amplitude: 1.61700000000000e-08,
        phase: 5.72212771018000e+00,
        frequency: 5.62184292321040e+03,
    },
    Vsop87Term {
        amplitude: 1.62500000000000e-08,
        phase: 4.63140305669000e+00,
        frequency: 3.18519202726560e+03,
    },
    Vsop87Term {
        amplitude: 1.17300000000000e-08,
        phase: 2.04871812080000e+00,
        frequency: 6.68474797174860e+03,
    },
    Vsop87Term {
        amplitude: 1.06600000000000e-08,
        phase: 1.15825195582000e+00,
        frequency: 2.28123049651060e+03,
    },
    Vsop87Term {
        amplitude: 1.04300000000000e-08,
        phase: 3.54796199484000e+00,
        frequency: 3.53206069281140e+03,
    },
    Vsop87Term {
        amplitude: 7.24000000000000e-09,
        phase: 4.20458660775000e+00,
        frequency: 5.88492684658320e+03,
    },
    Vsop87Term {
        amplitude: 6.10000000000000e-09,
        phase: 5.22430575829000e+00,
        frequency: 2.81092146160520e+03,
    },
    Vsop87Term {
        amplitude: 5.57000000000000e-09,
        phase: 2.75698163218000e+00,
        frequency: 2.94246342329160e+03,
    },
    Vsop87Term {
        amplitude: 6.55000000000000e-09,
        phase: 2.84033212598000e+00,
        frequency: 6.67770173505060e+03,
    },
    Vsop87Term {
        amplitude: 6.41000000000000e-09,
        phase: 5.44557723446000e+00,
        frequency: 5.48677784317500e+03,
    },
];

const MARS_B_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 3.30418000000000e-06,
        phase: 2.04215300484000e+00,
        frequency: 3.34061242669980e+03,
    },
    Vsop87Term {
        amplitude: 9.30570000000000e-07,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.45460000000000e-07,
        phase: 5.38525967237000e+00,
        frequency: 1.00218372800994e+04,
    },
    Vsop87Term {
        amplitude: 8.73100000000000e-08,
        phase: 4.90252313032000e+00,
        frequency: 6.68122485339960e+03,
    },
    Vsop87Term {
        amplitude: 5.21500000000000e-08,
        phase: 5.97441462813000e+00,
        frequency: 1.33624497067992e+04,
    },
    Vsop87Term {
        amplitude: 1.42200000000000e-08,
        phase: 2.12836502260000e-01,
        frequency: 1.67030621334990e+04,
    },
    Vsop87Term {
        amplitude: 5.85000000000000e-09,
        phase: 4.14327356645000e+00,
        frequency: 3.33708930835080e+03,
    },
    Vsop87Term {
        amplitude: 4.89000000000000e-09,
        phase: 1.32053432093000e+00,
        frequency: 3.49603282613400e+03,
    },
    Vsop87Term {
        amplitude: 3.32000000000000e-09,
        phase: 6.70620285390000e-01,
        frequency: 2.00436745601988e+04,
    },
    Vsop87Term {
        amplitude: 2.81000000000000e-09,
        phase: 3.02888736938000e+00,
        frequency: 3.18519202726560e+03,
    },
    Vsop87Term {
        amplitude: 9.50000000000000e-10,
        phase: 6.81291903680000e-01,
        frequency: 5.62184292321040e+03,
    },
    Vsop87Term {
        amplitude: 1.17000000000000e-09,
        phase: 6.20178829761000e+00,
        frequency: 3.34413554504880e+03,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-09,
        phase: 2.70816972496000e+00,
        frequency: 3.09788382272579e+03,
    },
    Vsop87Term {
        amplitude: 8.60000000000000e-10,
        phase: 3.80645819220000e-01,
        frequency: 6.68474797174860e+03,
    },
    Vsop87Term {
        amplitude: 7.10000000000000e-10,
        phase: 6.24412260280000e+00,
        frequency: 3.89418182954220e+03,
    },
    Vsop87Term {
        amplitude: 7.90000000000000e-10,
        phase: 5.14539339999000e+00,
        frequency: 3.53206069281140e+03,
    },
    Vsop87Term {
        amplitude: 8.00000000000000e-10,
        phase: 3.42045199039000e+00,
        frequency: 6.52580445396540e+03,
    },
    Vsop87Term {
        amplitude: 6.10000000000000e-10,
        phase: 3.81929342821000e+00,
        frequency: 2.94246342329160e+03,
    },
    Vsop87Term {
        amplitude: 6.60000000000000e-10,
        phase: 9.52649881000000e-01,
        frequency: 2.33842869868986e+04,
    },
    Vsop87Term {
        amplitude: 6.20000000000000e-10,
        phase: 1.65751945223000e+00,
        frequency: 3.58334103067380e+03,
    },
];

const MARS_B_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 6.00700000000000e-08,
        phase: 3.37637101191000e+00,
        frequency: 3.34061242669980e+03,
    },
    Vsop87Term {
        amplitude: 6.62500000000000e-08,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 4.64000000000000e-09,
        phase: 3.77202757150000e+00,
        frequency: 1.00218372800994e+04,
    },
    Vsop87Term {
        amplitude: 4.05000000000000e-09,
        phase: 6.31074167760000e-01,
        frequency: 6.68122485339960e+03,
    },
    Vsop87Term {
        amplitude: 2.99000000000000e-09,
        phase: 4.32966790803000e+00,
        frequency: 1.33624497067992e+04,
    },
    Vsop87Term {
        amplitude: 1.09000000000000e-09,
        phase: 4.85358280107000e+00,
        frequency: 1.67030621334990e+04,
    },
    Vsop87Term {
        amplitude: 7.70000000000000e-10,
        phase: 2.88080440451000e+00,
        frequency: 3.49603282613400e+03,
    },
    Vsop87Term {
        amplitude: 3.90000000000000e-10,
        phase: 1.43197590360000e+00,
        frequency: 3.18519202726560e+03,
    },
    Vsop87Term {
        amplitude: 3.10000000000000e-10,
        phase: 5.67989354532000e+00,
        frequency: 3.33708930835080e+03,
    },
    Vsop87Term {
        amplitude: 3.10000000000000e-10,
        phase: 5.33373014736000e+00,
        frequency: 2.00436745601988e+04,
    },
    Vsop87Term {
        amplitude: 1.70000000000000e-10,
        phase: 4.32019018052000e+00,
        frequency: 3.09788382272579e+03,
    },
    Vsop87Term {
        amplitude: 1.30000000000000e-10,
        phase: 1.77444975035000e+00,
        frequency: 6.52580445396540e+03,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-10,
        phase: 4.05960272400000e-02,
        frequency: 3.58334103067380e+03,
    },
    Vsop87Term {
        amplitude: 7.00000000000000e-11,
        phase: 4.46709463778000e+00,
        frequency: 6.68474797174860e+03,
    },
    Vsop87Term {
        amplitude: 6.00000000000000e-11,
        phase: 1.85656049405000e+00,
        frequency: 2.78704302385740e+03,
    },
    Vsop87Term {
        amplitude: 6.00000000000000e-11,
        phase: 1.15875671922000e+00,
        frequency: 3.89418182954220e+03,
    },
    Vsop87Term {
        amplitude: 6.00000000000000e-11,
        phase: 5.76437007548000e+00,
        frequency: 2.33842869868986e+04,
    },
    Vsop87Term {
        amplitude: 3.00000000000000e-11,
        phase: 2.91488661915000e+00,
        frequency: 6.83664525283380e+03,
    },
];

const MARS_B_5: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 3.68000000000000e-09,
        phase: 4.63460005338000e+00,
        frequency: 3.34061242669980e+03,
    },
    Vsop87Term {
        amplitude: 4.50000000000000e-10,
        phase: 5.14206308865000e+00,
        frequency: 6.68122485339960e+03,
    },
    Vsop87Term {
        amplitude: 3.50000000000000e-10,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.20000000000000e-10,
        phase: 2.68434578055000e+00,
        frequency: 1.33624497067992e+04,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-10,
        phase: 4.43919916342000e+00,
        frequency: 3.49603282613400e+03,
    },
    Vsop87Term {
        amplitude: 6.00000000000000e-11,
        phase: 1.70177189273000e+00,
        frequency: 1.00218372800994e+04,
    },
    Vsop87Term {
        amplitude: 5.00000000000000e-11,
        phase: 3.20415104620000e+00,
        frequency: 1.67030621334990e+04,
    },
    Vsop87Term {
        amplitude: 4.00000000000000e-11,
        phase: 6.15670065734000e+00,
        frequency: 3.18519202726560e+03,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-11,
        phase: 3.71915090755000e+00,
        frequency: 2.00436745601988e+04,
    },
];

const MARS_R_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.53033488271000e+00,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.41849531600000e-01,
        phase: 3.47971283528000e+00,
        frequency: 3.34061242669980e+03,
    },
    Vsop87Term {
        amplitude: 6.60776362000000e-03,
        phase: 3.81783443019000e+00,
        frequency: 6.68122485339960e+03,
    },
    Vsop87Term {
        amplitude: 4.61791170000000e-04,
        phase: 4.15595316782000e+00,
        frequency: 1.00218372800994e+04,
    },
    Vsop87Term {
        amplitude: 8.10973300000000e-05,
        phase: 5.55958416318000e+00,
        frequency: 2.81092146160520e+03,
    },
    Vsop87Term {
        amplitude: 7.48531800000000e-05,
        phase: 1.77239078402000e+00,
        frequency: 5.62184292321040e+03,
    },
    Vsop87Term {
        amplitude: 5.52319100000000e-05,
        phase: 1.36436303770000e+00,
        frequency: 2.28123049651060e+03,
    },
    Vsop87Term {
        amplitude: 3.82516000000000e-05,
        phase: 4.49407183687000e+00,
        frequency: 1.33624497067992e+04,
    },
    Vsop87Term {
        amplitude: 2.30653700000000e-05,
        phase: 9.08157900100000e-02,
        frequency: 2.54431441988340e+03,
    },
    Vsop87Term {
        amplitude: 1.99939600000000e-05,
        phase: 5.36059617709000e+00,
        frequency: 3.33708930835080e+03,
    },
    Vsop87Term {
        amplitude: 2.48439400000000e-05,
        phase: 4.92545639920000e+00,
        frequency: 2.94246342329160e+03,
    },
    Vsop87Term {
        amplitude: 1.96019500000000e-05,
        phase: 4.74249437639000e+00,
        frequency: 3.34413554504880e+03,
    },
    Vsop87Term {
        amplitude: 1.16711900000000e-05,
        phase: 2.11260868341000e+00,
        frequency: 5.09215195811580e+03,
    },
    Vsop87Term {
        amplitude: 1.10281600000000e-05,
        phase: 5.00908403998000e+00,
        frequency: 3.98149003408200e+02,
    },
    Vsop87Term {
        amplitude: 8.99066000000000e-06,
        phase: 4.40791133207000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 9.92252000000000e-06,
        phase: 5.83861961952000e+00,
        frequency: 6.15153388830500e+03,
    },
    Vsop87Term {
        amplitude: 8.07354000000000e-06,
        phase: 2.10217065501000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 7.97915000000000e-06,
        phase: 3.44839203899000e+00,
        frequency: 7.96298006816400e+02,
    },
    Vsop87Term {
        amplitude: 7.40975000000000e-06,
        phase: 1.49906336885000e+00,
        frequency: 2.14616541647520e+03,
    },
    Vsop87Term {
        amplitude: 6.92339000000000e-06,
        phase: 2.13378874689000e+00,
        frequency: 8.96245534991020e+03,
    },
];

const MARS_R_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.10743334500000e-02,
        phase: 2.03250524857000e+00,
        frequency: 3.34061242669980e+03,
    },
    Vsop87Term {
        amplitude: 1.03175887000000e-03,
        phase: 2.37071847807000e+00,
        frequency: 6.68122485339960e+03,
    },
    Vsop87Term {
        amplitude: 1.28772000000000e-04,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.08158800000000e-04,
        phase: 2.70888095665000e+00,
        frequency: 1.00218372800994e+04,
    },
    Vsop87Term {
        amplitude: 1.19455000000000e-05,
        phase: 3.04702256206000e+00,
        frequency: 1.33624497067992e+04,
    },
    Vsop87Term {
        amplitude: 4.38582000000000e-06,
        phase: 2.88835054603000e+00,
        frequency: 2.28123049651060e+03,
    },
    Vsop87Term {
        amplitude: 3.95700000000000e-06,
        phase: 3.42323670971000e+00,
        frequency: 3.34413554504880e+03,
    },
    Vsop87Term {
        amplitude: 1.82576000000000e-06,
        phase: 1.58427562964000e+00,
        frequency: 2.54431441988340e+03,
    },
    Vsop87Term {
        amplitude: 1.35851000000000e-06,
        phase: 3.38507063082000e+00,
        frequency: 1.67030621334990e+04,
    },
    Vsop87Term {
        amplitude: 1.28199000000000e-06,
        phase: 6.29917718130000e-01,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 1.27059000000000e-06,
        phase: 1.95391155885000e+00,
        frequency: 7.96298006816400e+02,
    },
    Vsop87Term {
        amplitude: 1.18443000000000e-06,
        phase: 2.99762091382000e+00,
        frequency: 2.14616541647520e+03,
    },
    Vsop87Term {
        amplitude: 1.28362000000000e-06,
        phase: 6.04343227063000e+00,
        frequency: 3.33708930835080e+03,
    },
    Vsop87Term {
        amplitude: 8.75340000000000e-07,
        phase: 3.42053385867000e+00,
        frequency: 3.98149003408200e+02,
    },
    Vsop87Term {
        amplitude: 8.30210000000000e-07,
        phase: 3.85575072018000e+00,
        frequency: 3.73876143010800e+03,
    },
    Vsop87Term {
        amplitude: 7.56040000000000e-07,
        phase: 4.45097659377000e+00,
        frequency: 6.15153388830500e+03,
    },
    Vsop87Term {
        amplitude: 7.20020000000000e-07,
        phase: 2.76443992447000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 6.65450000000000e-07,
        phase: 2.54878381470000e+00,
        frequency: 1.75153953141600e+03,
    },
    Vsop87Term {
        amplitude: 5.43050000000000e-07,
        phase: 6.77542033870000e-01,
        frequency: 8.96245534991020e+03,
    },
    Vsop87Term {
        amplitude: 5.10430000000000e-07,
        phase: 3.72584855417000e+00,
        frequency: 6.68474797174860e+03,
    },
];

const MARS_R_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 4.42422490000000e-04,
        phase: 4.79306049540000e-01,
        frequency: 3.34061242669980e+03,
    },
    Vsop87Term {
        amplitude: 8.13804200000000e-05,
        phase: 8.69983892040000e-01,
        frequency: 6.68122485339960e+03,
    },
    Vsop87Term {
        amplitude: 1.27491500000000e-05,
        phase: 1.22593985222000e+00,
        frequency: 1.00218372800994e+04,
    },
    Vsop87Term {
        amplitude: 1.87388000000000e-06,
        phase: 1.57298976045000e+00,
        frequency: 1.33624497067992e+04,
    },
    Vsop87Term {
        amplitude: 4.07450000000000e-07,
        phase: 1.97082077028000e+00,
        frequency: 3.34413554504880e+03,
    },
    Vsop87Term {
        amplitude: 5.23950000000000e-07,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 2.66170000000000e-07,
        phase: 1.91665337822000e+00,
        frequency: 1.67030621334990e+04,
    },
    Vsop87Term {
        amplitude: 1.78280000000000e-07,
        phase: 4.43491476419000e+00,
        frequency: 2.28123049651060e+03,
    },
    Vsop87Term {
        amplitude: 1.17130000000000e-07,
        phase: 4.52509926559000e+00,
        frequency: 3.18519202726560e+03,
    },
    Vsop87Term {
        amplitude: 1.02100000000000e-07,
        phase: 5.39147322060000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 9.95000000000000e-08,
        phase: 4.18656784480000e-01,
        frequency: 7.96298006816400e+02,
    },
    Vsop87Term {
        amplitude: 9.23600000000000e-08,
        phase: 4.53559625376000e+00,
        frequency: 2.14616541647520e+03,
    },
    Vsop87Term {
        amplitude: 7.29900000000000e-08,
        phase: 3.14214513120000e+00,
        frequency: 2.54431441988340e+03,
    },
    Vsop87Term {
        amplitude: 7.21400000000000e-08,
        phase: 2.29302335628000e+00,
        frequency: 6.68474797174860e+03,
    },
    Vsop87Term {
        amplitude: 6.81000000000000e-08,
        phase: 5.26707245601000e+00,
        frequency: 1.55420399434200e+02,
    },
    Vsop87Term {
        amplitude: 6.52600000000000e-08,
        phase: 2.30772456100000e+00,
        frequency: 3.73876143010800e+03,
    },
    Vsop87Term {
        amplitude: 7.78300000000000e-08,
        phase: 5.93373461009000e+00,
        frequency: 1.74801641306700e+03,
    },
    Vsop87Term {
        amplitude: 5.84000000000000e-08,
        phase: 1.05191820290000e+00,
        frequency: 1.34986740965880e+03,
    },
    Vsop87Term {
        amplitude: 6.75000000000000e-08,
        phase: 5.30191763402000e+00,
        frequency: 1.19444701022460e+03,
    },
    Vsop87Term {
        amplitude: 4.69500000000000e-08,
        phase: 7.68810328740000e-01,
        frequency: 3.09788382272579e+03,
    },
];

const MARS_R_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.11310800000000e-05,
        phase: 5.14987305093000e+00,
        frequency: 3.34061242669980e+03,
    },
    Vsop87Term {
        amplitude: 4.24447000000000e-06,
        phase: 5.61343952053000e+00,
        frequency: 6.68122485339960e+03,
    },
    Vsop87Term {
        amplitude: 1.00044000000000e-06,
        phase: 5.99727457548000e+00,
        frequency: 1.00218372800994e+04,
    },
    Vsop87Term {
        amplitude: 1.96060000000000e-07,
        phase: 7.63145378300000e-02,
        frequency: 1.33624497067992e+04,
    },
    Vsop87Term {
        amplitude: 3.47800000000000e-08,
        phase: 4.29120102110000e-01,
        frequency: 1.67030621334990e+04,
    },
    Vsop87Term {
        amplitude: 4.69300000000000e-08,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 2.87000000000000e-08,
        phase: 4.46920023930000e-01,
        frequency: 3.34413554504880e+03,
    },
    Vsop87Term {
        amplitude: 2.42800000000000e-08,
        phase: 3.02114808809000e+00,
        frequency: 3.18519202726560e+03,
    },
    Vsop87Term {
        amplitude: 6.87000000000000e-09,
        phase: 8.05604426660000e-01,
        frequency: 6.68474797174860e+03,
    },
    Vsop87Term {
        amplitude: 5.78000000000000e-09,
        phase: 7.78493094020000e-01,
        frequency: 2.00436745601988e+04,
    },
    Vsop87Term {
        amplitude: 5.40000000000000e-09,
        phase: 3.86818792995000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 4.68000000000000e-09,
        phase: 4.52509679627000e+00,
        frequency: 3.49603282613400e+03,
    },
    Vsop87Term {
        amplitude: 4.87000000000000e-09,
        phase: 1.60861075208000e+00,
        frequency: 3.58334103067380e+03,
    },
    Vsop87Term {
        amplitude: 3.62000000000000e-09,
        phase: 4.42325508671000e+00,
        frequency: 2.78704302385740e+03,
    },
    Vsop87Term {
        amplitude: 3.97000000000000e-09,
        phase: 5.72008479880000e+00,
        frequency: 3.14916416058820e+03,
    },
    Vsop87Term {
        amplitude: 2.99000000000000e-09,
        phase: 7.57084283850000e-01,
        frequency: 3.73876143010800e+03,
    },
    Vsop87Term {
        amplitude: 3.52000000000000e-09,
        phase: 5.55622002342000e+00,
        frequency: 4.13691043351620e+03,
    },
    Vsop87Term {
        amplitude: 3.16000000000000e-09,
        phase: 3.37609906858000e+00,
        frequency: 6.52580445396540e+03,
    },
    Vsop87Term {
        amplitude: 2.34000000000000e-09,
        phase: 2.13923292630000e+00,
        frequency: 3.09788382272579e+03,
    },
    Vsop87Term {
        amplitude: 2.14000000000000e-09,
        phase: 4.20457486315000e+00,
        frequency: 3.34159274776800e+03,
    },
];

const MARS_R_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.95510000000000e-07,
        phase: 3.58210746512000e+00,
        frequency: 3.34061242669980e+03,
    },
    Vsop87Term {
        amplitude: 1.63220000000000e-07,
        phase: 4.05115851142000e+00,
        frequency: 6.68122485339960e+03,
    },
    Vsop87Term {
        amplitude: 5.84800000000000e-08,
        phase: 4.46381646580000e+00,
        frequency: 1.00218372800994e+04,
    },
    Vsop87Term {
        amplitude: 1.53300000000000e-08,
        phase: 4.84332951095000e+00,
        frequency: 1.33624497067992e+04,
    },
    Vsop87Term {
        amplitude: 3.75000000000000e-09,
        phase: 1.50951652931000e+00,
        frequency: 3.18519202726560e+03,
    },
    Vsop87Term {
        amplitude: 3.40000000000000e-09,
        phase: 5.20519444932000e+00,
        frequency: 1.67030621334990e+04,
    },
    Vsop87Term {
        amplitude: 1.52000000000000e-09,
        phase: 5.16376141170000e+00,
        frequency: 3.34413554504880e+03,
    },
    Vsop87Term {
        amplitude: 1.26000000000000e-09,
        phase: 2.19183723061000e+00,
        frequency: 3.49603282613400e+03,
    },
    Vsop87Term {
        amplitude: 1.48000000000000e-09,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 8.80000000000000e-10,
        phase: 1.04303598620000e-01,
        frequency: 3.58334103067380e+03,
    },
    Vsop87Term {
        amplitude: 7.10000000000000e-10,
        phase: 5.56060980870000e+00,
        frequency: 2.00436745601988e+04,
    },
    Vsop87Term {
        amplitude: 5.00000000000000e-10,
        phase: 5.56346631885000e+00,
        frequency: 6.68474797174860e+03,
    },
    Vsop87Term {
        amplitude: 4.80000000000000e-10,
        phase: 2.91926298132000e+00,
        frequency: 2.78704302385740e+03,
    },
    Vsop87Term {
        amplitude: 5.70000000000000e-10,
        phase: 1.86870004051000e+00,
        frequency: 6.52580445396540e+03,
    },
    Vsop87Term {
        amplitude: 3.40000000000000e-10,
        phase: 3.63370917313000e+00,
        frequency: 3.09788382272579e+03,
    },
    Vsop87Term {
        amplitude: 2.10000000000000e-10,
        phase: 2.30993179224000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 2.00000000000000e-10,
        phase: 4.04278354592000e+00,
        frequency: 4.13691043351620e+03,
    },
    Vsop87Term {
        amplitude: 1.80000000000000e-10,
        phase: 4.17438755890000e+00,
        frequency: 3.14916416058820e+03,
    },
    Vsop87Term {
        amplitude: 1.70000000000000e-10,
        phase: 5.90833786763000e+00,
        frequency: 2.33842869868986e+04,
    },
    Vsop87Term {
        amplitude: 1.40000000000000e-10,
        phase: 1.98185426243000e+00,
        frequency: 1.55420399434200e+02,
    },
];

const MARS_R_5: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 4.75000000000000e-09,
        phase: 2.47621038205000e+00,
        frequency: 6.68122485339960e+03,
    },
    Vsop87Term {
        amplitude: 2.70000000000000e-09,
        phase: 2.90961348988000e+00,
        frequency: 1.00218372800994e+04,
    },
    Vsop87Term {
        amplitude: 1.16000000000000e-09,
        phase: 1.76766655427000e+00,
        frequency: 3.34061242669980e+03,
    },
    Vsop87Term {
        amplitude: 9.70000000000000e-10,
        phase: 3.31315004582000e+00,
        frequency: 1.33624497067992e+04,
    },
    Vsop87Term {
        amplitude: 4.90000000000000e-10,
        phase: 6.28284800757000e+00,
        frequency: 3.18519202726560e+03,
    },
    Vsop87Term {
        amplitude: 2.70000000000000e-10,
        phase: 3.69297278191000e+00,
        frequency: 1.67030621334990e+04,
    },
    Vsop87Term {
        amplitude: 1.30000000000000e-10,
        phase: 3.67011956659000e+00,
        frequency: 3.49603282613400e+03,
    },
    Vsop87Term {
        amplitude: 1.20000000000000e-10,
        phase: 4.88179002689000e+00,
        frequency: 3.58334103067380e+03,
    },
    Vsop87Term {
        amplitude: 1.20000000000000e-10,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 9.00000000000000e-11,
        phase: 4.05335755845000e+00,
        frequency: 2.00436745601988e+04,
    },
    Vsop87Term {
        amplitude: 9.00000000000000e-11,
        phase: 3.55758014570000e-01,
        frequency: 6.52580445396540e+03,
    },
    Vsop87Term {
        amplitude: 6.00000000000000e-11,
        phase: 3.95186839136000e+00,
        frequency: 6.68474797174860e+03,
    },
    Vsop87Term {
        amplitude: 5.00000000000000e-11,
        phase: 1.40424074649000e+00,
        frequency: 2.78704302385740e+03,
    },
    Vsop87Term {
        amplitude: 5.00000000000000e-11,
        phase: 3.60327227648000e+00,
        frequency: 3.34413554504880e+03,
    },
    Vsop87Term {
        amplitude: 4.00000000000000e-11,
        phase: 5.11981331029000e+00,
        frequency: 3.09788382272579e+03,
    },
    Vsop87Term {
        amplitude: 4.00000000000000e-11,
        phase: 5.24603068590000e+00,
        frequency: 6.92395345737360e+03,
    },
    Vsop87Term {
        amplitude: 2.00000000000000e-11,
        phase: 4.09544260110000e-01,
        frequency: 9.86641688066520e+03,
    },
];
