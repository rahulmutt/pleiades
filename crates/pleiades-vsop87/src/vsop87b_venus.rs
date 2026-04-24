//! Truncated VSOP87B Venus coefficient tables.
//!
//! Coefficients are the leading terms from the public IMCCE/CELMECH
//! `VSOP87B.ven` file (Venus heliocentric spherical variables, J2000
//! ecliptic/equinox). This mirrors the Earth and Mercury slices while
//! complete generated tables are still planned.

use crate::vsop87b_earth::{evaluate, SphericalLbr, Vsop87Term};

pub(crate) fn venus_lbr(julian_day_tt: f64) -> SphericalLbr {
    let t = (julian_day_tt - 2_451_545.0) / 365_250.0;
    SphericalLbr {
        longitude_rad: evaluate(
            [
                VENUS_L_0, VENUS_L_1, VENUS_L_2, VENUS_L_3, VENUS_L_4, VENUS_L_5,
            ],
            t,
        )
        .rem_euclid(core::f64::consts::TAU),
        latitude_rad: evaluate(
            [
                VENUS_B_0, VENUS_B_1, VENUS_B_2, VENUS_B_3, VENUS_B_4, VENUS_B_5,
            ],
            t,
        ),
        radius_au: evaluate(
            [
                VENUS_R_0, VENUS_R_1, VENUS_R_2, VENUS_R_3, VENUS_R_4, VENUS_R_5,
            ],
            t,
        ),
    }
}

const VENUS_L_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 3.17614666774000e+00,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.35396841900000e-02,
        phase: 5.59313319619000e+00,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 8.98916450000000e-04,
        phase: 5.30650047764000e+00,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 5.47719400000000e-05,
        phase: 4.41630661466000e+00,
        frequency: 7.86041939243920e+03,
    },
    Vsop87Term {
        amplitude: 3.45574100000000e-05,
        phase: 2.69964447820000e+00,
        frequency: 1.17906290886588e+04,
    },
    Vsop87Term {
        amplitude: 2.37206100000000e-05,
        phase: 2.99377542079000e+00,
        frequency: 3.93020969621960e+03,
    },
    Vsop87Term {
        amplitude: 1.31716800000000e-05,
        phase: 5.18668228402000e+00,
        frequency: 2.62983197998000e+01,
    },
    Vsop87Term {
        amplitude: 1.66414600000000e-05,
        phase: 4.25018630147000e+00,
        frequency: 1.57734354244780e+03,
    },
    Vsop87Term {
        amplitude: 1.43838700000000e-05,
        phase: 4.15745084182000e+00,
        frequency: 9.68359458111640e+03,
    },
    Vsop87Term {
        amplitude: 1.20052100000000e-05,
        phase: 6.15357116043000e+00,
        frequency: 3.06398566386330e+04,
    },
    Vsop87Term {
        amplitude: 7.61380000000000e-06,
        phase: 1.95014701047000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 7.07676000000000e-06,
        phase: 1.06466702668000e+00,
        frequency: 7.75522611324000e+02,
    },
    Vsop87Term {
        amplitude: 5.84836000000000e-06,
        phase: 3.99839888230000e+00,
        frequency: 1.91448266111600e+02,
    },
    Vsop87Term {
        amplitude: 7.69314000000000e-06,
        phase: 8.16296151960000e-01,
        frequency: 9.43776293488700e+03,
    },
    Vsop87Term {
        amplitude: 4.99915000000000e-06,
        phase: 4.12340212820000e+00,
        frequency: 1.57208387848784e+04,
    },
    Vsop87Term {
        amplitude: 3.26221000000000e-06,
        phase: 4.59056477038000e+00,
        frequency: 1.04047338123226e+04,
    },
    Vsop87Term {
        amplitude: 4.29498000000000e-06,
        phase: 3.58642858577000e+00,
        frequency: 1.93671891622328e+04,
    },
    Vsop87Term {
        amplitude: 3.26967000000000e-06,
        phase: 5.67736584311000e+00,
        frequency: 5.50755323866740e+03,
    },
    Vsop87Term {
        amplitude: 2.31937000000000e-06,
        phase: 3.16251059356000e+00,
        frequency: 9.15390361602180e+03,
    },
    Vsop87Term {
        amplitude: 1.79695000000000e-06,
        phase: 4.65337908917000e+00,
        frequency: 1.10937855209340e+03,
    },
];

const VENUS_L_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.02132855462164e+04,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 9.56178130000000e-04,
        phase: 2.46406511110000e+00,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 7.78720100000000e-05,
        phase: 6.24784822200000e-01,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 1.51666000000000e-06,
        phase: 6.10638559291000e+00,
        frequency: 1.57734354244780e+03,
    },
    Vsop87Term {
        amplitude: 1.41694000000000e-06,
        phase: 2.12362986036000e+00,
        frequency: 3.06398566386330e+04,
    },
    Vsop87Term {
        amplitude: 1.73908000000000e-06,
        phase: 2.65539499463000e+00,
        frequency: 2.62983197998000e+01,
    },
    Vsop87Term {
        amplitude: 8.22350000000000e-07,
        phase: 5.70231469551000e+00,
        frequency: 1.91448266111600e+02,
    },
    Vsop87Term {
        amplitude: 6.97320000000000e-07,
        phase: 2.68128549229000e+00,
        frequency: 9.43776293488700e+03,
    },
    Vsop87Term {
        amplitude: 5.22920000000000e-07,
        phase: 3.60270736876000e+00,
        frequency: 7.75522611324000e+02,
    },
    Vsop87Term {
        amplitude: 3.83130000000000e-07,
        phase: 1.03371309443000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 2.96300000000000e-07,
        phase: 1.25050823203000e+00,
        frequency: 5.50755323866740e+03,
    },
    Vsop87Term {
        amplitude: 2.50560000000000e-07,
        phase: 6.10650638660000e+00,
        frequency: 1.04047338123226e+04,
    },
    Vsop87Term {
        amplitude: 1.77720000000000e-07,
        phase: 6.19369679929000e+00,
        frequency: 1.10937855209340e+03,
    },
    Vsop87Term {
        amplitude: 1.65100000000000e-07,
        phase: 2.64360813203000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 1.42310000000000e-07,
        phase: 5.45125927817000e+00,
        frequency: 9.15390361602180e+03,
    },
    Vsop87Term {
        amplitude: 1.16270000000000e-07,
        phase: 4.97604433638000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 1.25630000000000e-07,
        phase: 1.88122194951000e+00,
        frequency: 3.82896532223200e+02,
    },
    Vsop87Term {
        amplitude: 8.87700000000000e-08,
        phase: 9.52453934570000e-01,
        frequency: 1.33679726311066e+04,
    },
    Vsop87Term {
        amplitude: 7.37400000000000e-08,
        phase: 4.39476352550000e+00,
        frequency: 1.02061719992102e+04,
    },
    Vsop87Term {
        amplitude: 6.55000000000000e-08,
        phase: 2.28168331756000e+00,
        frequency: 2.35286615377180e+03,
    },
];

const VENUS_L_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 3.89420900000000e-05,
        phase: 3.48236507210000e-01,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 5.95403000000000e-06,
        phase: 2.01456107998000e+00,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 2.87868000000000e-06,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 2.38380000000000e-07,
        phase: 2.04588223604000e+00,
        frequency: 2.62983197998000e+01,
    },
    Vsop87Term {
        amplitude: 9.96400000000000e-08,
        phase: 3.97089333901000e+00,
        frequency: 7.75522611324000e+02,
    },
    Vsop87Term {
        amplitude: 7.19600000000000e-08,
        phase: 3.65730119531000e+00,
        frequency: 3.06398566386330e+04,
    },
    Vsop87Term {
        amplitude: 7.04300000000000e-08,
        phase: 1.52107808192000e+00,
        frequency: 1.57734354244780e+03,
    },
    Vsop87Term {
        amplitude: 6.01400000000000e-08,
        phase: 1.00039990357000e+00,
        frequency: 1.91448266111600e+02,
    },
    Vsop87Term {
        amplitude: 3.16700000000000e-08,
        phase: 4.36138169912000e+00,
        frequency: 9.43776293488700e+03,
    },
    Vsop87Term {
        amplitude: 1.93400000000000e-08,
        phase: 3.39260216059000e+00,
        frequency: 3.82896532223200e+02,
    },
    Vsop87Term {
        amplitude: 1.45900000000000e-08,
        phase: 6.05311371882000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 1.34500000000000e-08,
        phase: 2.94746266562000e+00,
        frequency: 5.50755323866740e+03,
    },
    Vsop87Term {
        amplitude: 1.02400000000000e-08,
        phase: 1.40825326249000e+00,
        frequency: 1.04047338123226e+04,
    },
    Vsop87Term {
        amplitude: 1.22400000000000e-08,
        phase: 3.73276078401000e+00,
        frequency: 3.15468708489560e+03,
    },
    Vsop87Term {
        amplitude: 1.03300000000000e-08,
        phase: 3.52850062173000e+00,
        frequency: 1.10151064773348e+04,
    },
    Vsop87Term {
        amplitude: 7.67000000000000e-09,
        phase: 2.69606070058000e+00,
        frequency: 4.08531421848440e+04,
    },
    Vsop87Term {
        amplitude: 9.54000000000000e-09,
        phase: 5.11160150203000e+00,
        frequency: 8.01820931123800e+02,
    },
    Vsop87Term {
        amplitude: 7.42000000000000e-09,
        phase: 1.49195106907000e+00,
        frequency: 1.10937855209340e+03,
    },
    Vsop87Term {
        amplitude: 5.25000000000000e-09,
        phase: 3.31953730020000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 5.74000000000000e-09,
        phase: 9.22868993350000e-01,
        frequency: 1.02395838660108e+04,
    },
];

const VENUS_L_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.36328000000000e-06,
        phase: 4.79698723753000e+00,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 3.06610000000000e-07,
        phase: 3.71663788064000e+00,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 3.04100000000000e-08,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 5.06000000000000e-09,
        phase: 5.34186957078000e+00,
        frequency: 3.06398566386330e+04,
    },
    Vsop87Term {
        amplitude: 7.10000000000000e-10,
        phase: 4.27707588774000e+00,
        frequency: 4.08531421848440e+04,
    },
    Vsop87Term {
        amplitude: 8.00000000000000e-11,
        phase: 1.76653383282000e+00,
        frequency: 1.01869872264112e+04,
    },
    Vsop87Term {
        amplitude: 6.00000000000000e-11,
        phase: 5.61707828538000e+00,
        frequency: 1.02395838660108e+04,
    },
];

const VENUS_L_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.63600000000000e-08,
        phase: 2.50540811485000e+00,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 1.08000000000000e-08,
        phase: 5.10106236574000e+00,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 1.80000000000000e-10,
        phase: 8.83158567390000e-01,
        frequency: 3.06398566386330e+04,
    },
    Vsop87Term {
        amplitude: 5.00000000000000e-11,
        phase: 5.76650226003000e+00,
        frequency: 4.08531421848440e+04,
    },
];

const VENUS_L_5: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.22000000000000e-09,
        phase: 1.88711724630000e+00,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 4.30000000000000e-10,
        phase: 4.21259092900000e-01,
        frequency: 2.04265710924220e+04,
    },
];

const VENUS_B_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 5.92363847200000e-02,
        phase: 2.67027758120000e-01,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 4.01079780000000e-04,
        phase: 1.14737178112000e+00,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 3.28149180000000e-04,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.01139200000000e-05,
        phase: 1.08946119730000e+00,
        frequency: 3.06398566386330e+04,
    },
    Vsop87Term {
        amplitude: 1.49458000000000e-06,
        phase: 6.25390268112000e+00,
        frequency: 1.80737049386502e+04,
    },
    Vsop87Term {
        amplitude: 1.37788000000000e-06,
        phase: 8.60200955860000e-01,
        frequency: 1.57734354244780e+03,
    },
    Vsop87Term {
        amplitude: 1.29973000000000e-06,
        phase: 3.67152480061000e+00,
        frequency: 9.43776293488700e+03,
    },
    Vsop87Term {
        amplitude: 1.19507000000000e-06,
        phase: 3.70468787104000e+00,
        frequency: 2.35286615377180e+03,
    },
    Vsop87Term {
        amplitude: 1.07971000000000e-06,
        phase: 4.53903678347000e+00,
        frequency: 2.20039146348698e+04,
    },
    Vsop87Term {
        amplitude: 9.20290000000000e-07,
        phase: 1.53954519783000e+00,
        frequency: 9.15390361602180e+03,
    },
    Vsop87Term {
        amplitude: 5.29820000000000e-07,
        phase: 2.28138198002000e+00,
        frequency: 5.50755323866740e+03,
    },
    Vsop87Term {
        amplitude: 4.56170000000000e-07,
        phase: 7.23196462890000e-01,
        frequency: 1.02395838660108e+04,
    },
    Vsop87Term {
        amplitude: 3.88550000000000e-07,
        phase: 2.93437865147000e+00,
        frequency: 1.01869872264112e+04,
    },
    Vsop87Term {
        amplitude: 4.34910000000000e-07,
        phase: 6.14015779106000e+00,
        frequency: 1.17906290886588e+04,
    },
    Vsop87Term {
        amplitude: 4.17000000000000e-07,
        phase: 5.99126840013000e+00,
        frequency: 1.98968801273274e+04,
    },
    Vsop87Term {
        amplitude: 3.96440000000000e-07,
        phase: 3.86842103668000e+00,
        frequency: 8.63594200376320e+03,
    },
    Vsop87Term {
        amplitude: 3.91750000000000e-07,
        phase: 3.94960158566000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 3.33200000000000e-07,
        phase: 4.83194901518000e+00,
        frequency: 1.41434952424306e+04,
    },
    Vsop87Term {
        amplitude: 2.37110000000000e-07,
        phase: 2.90647469167000e+00,
        frequency: 1.09888081575350e+04,
    },
    Vsop87Term {
        amplitude: 2.35010000000000e-07,
        phase: 2.00771051056000e+00,
        frequency: 1.33679726311066e+04,
    },
];

const VENUS_B_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.87821243000000e-03,
        phase: 1.88964962838000e+00,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 3.49957800000000e-05,
        phase: 3.71117560516000e+00,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 1.25784400000000e-05,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 9.61520000000000e-07,
        phase: 2.74240664188000e+00,
        frequency: 3.06398566386330e+04,
    },
    Vsop87Term {
        amplitude: 1.30510000000000e-07,
        phase: 2.27549606211000e+00,
        frequency: 9.43776293488700e+03,
    },
    Vsop87Term {
        amplitude: 8.05200000000000e-08,
        phase: 5.55049163175000e+00,
        frequency: 1.57734354244780e+03,
    },
    Vsop87Term {
        amplitude: 6.79200000000000e-08,
        phase: 1.60704519868000e+00,
        frequency: 1.80737049386502e+04,
    },
    Vsop87Term {
        amplitude: 7.52100000000000e-08,
        phase: 2.89319163420000e-01,
        frequency: 9.15390361602180e+03,
    },
    Vsop87Term {
        amplitude: 5.61200000000000e-08,
        phase: 1.59167143282000e+00,
        frequency: 1.17906290886588e+04,
    },
    Vsop87Term {
        amplitude: 6.46000000000000e-08,
        phase: 5.23429470615000e+00,
        frequency: 1.01869872264112e+04,
    },
    Vsop87Term {
        amplitude: 5.40500000000000e-08,
        phase: 6.15384515120000e+00,
        frequency: 2.20039146348698e+04,
    },
    Vsop87Term {
        amplitude: 4.19400000000000e-08,
        phase: 5.40627767662000e+00,
        frequency: 2.35286615377180e+03,
    },
    Vsop87Term {
        amplitude: 5.09700000000000e-08,
        phase: 4.17404177420000e+00,
        frequency: 1.02395838660108e+04,
    },
    Vsop87Term {
        amplitude: 4.08500000000000e-08,
        phase: 7.36180479300000e-01,
        frequency: 5.50755323866740e+03,
    },
    Vsop87Term {
        amplitude: 3.34900000000000e-08,
        phase: 1.34464128958000e+00,
        frequency: 1.04047338123226e+04,
    },
    Vsop87Term {
        amplitude: 2.94100000000000e-08,
        phase: 4.40460505149000e+00,
        frequency: 1.96510484810980e+04,
    },
    Vsop87Term {
        amplitude: 2.67000000000000e-08,
        phase: 3.92292198547000e+00,
        frequency: 4.08531421848440e+04,
    },
    Vsop87Term {
        amplitude: 2.74600000000000e-08,
        phase: 5.04771993757000e+00,
        frequency: 1.09888081575350e+04,
    },
    Vsop87Term {
        amplitude: 2.21600000000000e-08,
        phase: 1.74777661827000e+00,
        frequency: 8.63594200376320e+03,
    },
    Vsop87Term {
        amplitude: 1.93400000000000e-08,
        phase: 1.35730576040000e+00,
        frequency: 1.98968801273274e+04,
    },
];

const VENUS_B_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.26577450000000e-04,
        phase: 3.34796457029000e+00,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 1.51225000000000e-06,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 3.74760000000000e-07,
        phase: 5.34638962141000e+00,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 1.06270000000000e-07,
        phase: 3.81894300538000e+00,
        frequency: 3.06398566386330e+04,
    },
    Vsop87Term {
        amplitude: 9.42000000000000e-09,
        phase: 1.31190556100000e-02,
        frequency: 1.01869872264112e+04,
    },
    Vsop87Term {
        amplitude: 7.47000000000000e-09,
        phase: 4.13620174126000e+00,
        frequency: 1.02395838660108e+04,
    },
    Vsop87Term {
        amplitude: 4.34000000000000e-09,
        phase: 3.26791015348000e+00,
        frequency: 1.17906290886588e+04,
    },
    Vsop87Term {
        amplitude: 4.39000000000000e-09,
        phase: 6.04066783494000e+00,
        frequency: 1.09888081575350e+04,
    },
    Vsop87Term {
        amplitude: 3.31000000000000e-09,
        phase: 6.20632270120000e+00,
        frequency: 9.43776293488700e+03,
    },
    Vsop87Term {
        amplitude: 3.32000000000000e-09,
        phase: 2.82983937642000e+00,
        frequency: 1.04047338123226e+04,
    },
    Vsop87Term {
        amplitude: 3.40000000000000e-09,
        phase: 5.14190117554000e+00,
        frequency: 9.15390361602180e+03,
    },
    Vsop87Term {
        amplitude: 2.44000000000000e-09,
        phase: 6.07029755311000e+00,
        frequency: 1.96510484810980e+04,
    },
    Vsop87Term {
        amplitude: 2.74000000000000e-09,
        phase: 5.62384076510000e-01,
        frequency: 2.35286615377180e+03,
    },
    Vsop87Term {
        amplitude: 2.73000000000000e-09,
        phase: 3.09551287480000e+00,
        frequency: 1.80737049386502e+04,
    },
    Vsop87Term {
        amplitude: 2.68000000000000e-09,
        phase: 4.02642385642000e+00,
        frequency: 1.57734354244780e+03,
    },
    Vsop87Term {
        amplitude: 2.17000000000000e-09,
        phase: 5.37498456751000e+00,
        frequency: 4.08531421848440e+04,
    },
    Vsop87Term {
        amplitude: 2.13000000000000e-09,
        phase: 1.22633958472000e+00,
        frequency: 2.20039146348698e+04,
    },
    Vsop87Term {
        amplitude: 1.41000000000000e-09,
        phase: 5.42727181668000e+00,
        frequency: 5.50755323866740e+03,
    },
    Vsop87Term {
        amplitude: 1.30000000000000e-09,
        phase: 5.26806549972000e+00,
        frequency: 1.33679726311066e+04,
    },
    Vsop87Term {
        amplitude: 1.01000000000000e-09,
        phase: 4.64089661780000e+00,
        frequency: 1.57208387848784e+04,
    },
];

const VENUS_B_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 3.76505000000000e-06,
        phase: 4.87650249694000e+00,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 1.25870000000000e-07,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 4.80900000000000e-08,
        phase: 4.34239180180000e-01,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 8.35000000000000e-09,
        phase: 5.57179521329000e+00,
        frequency: 3.06398566386330e+04,
    },
    Vsop87Term {
        amplitude: 4.60000000000000e-10,
        phase: 1.54914240166000e+00,
        frequency: 1.01869872264112e+04,
    },
    Vsop87Term {
        amplitude: 3.40000000000000e-10,
        phase: 5.78743368814000e+00,
        frequency: 1.02395838660108e+04,
    },
    Vsop87Term {
        amplitude: 1.90000000000000e-10,
        phase: 7.62273125800000e-01,
        frequency: 4.08531421848440e+04,
    },
    Vsop87Term {
        amplitude: 1.30000000000000e-10,
        phase: 4.63920619996000e+00,
        frequency: 1.17906290886588e+04,
    },
    Vsop87Term {
        amplitude: 1.40000000000000e-10,
        phase: 5.47124304598000e+00,
        frequency: 9.43776293488700e+03,
    },
    Vsop87Term {
        amplitude: 1.40000000000000e-10,
        phase: 8.76528536360000e-01,
        frequency: 1.09888081575350e+04,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-10,
        phase: 1.84595947891000e+00,
        frequency: 2.35286615377180e+03,
    },
    Vsop87Term {
        amplitude: 5.00000000000000e-11,
        phase: 1.36825703657000e+00,
        frequency: 1.96510484810980e+04,
    },
];

const VENUS_B_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 8.55800000000000e-08,
        phase: 1.71819720540000e-01,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 1.14000000000000e-09,
        phase: 2.50366130090000e+00,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 1.15000000000000e-09,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 4.10000000000000e-10,
        phase: 7.40614326910000e-01,
        frequency: 3.06398566386330e+04,
    },
];

const VENUS_B_5: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.49000000000000e-09,
        phase: 1.67437168506000e+00,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 2.30000000000000e-10,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 8.00000000000000e-11,
        phase: 3.73924477319000e+00,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 2.00000000000000e-11,
        phase: 2.28783748701000e+00,
        frequency: 3.06398566386330e+04,
    },
];

const VENUS_R_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 7.23348208910000e-01,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 4.89824182000000e-03,
        phase: 4.02151831717000e+00,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 1.65805800000000e-05,
        phase: 4.90206728031000e+00,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 1.63209600000000e-05,
        phase: 2.84548795207000e+00,
        frequency: 7.86041939243920e+03,
    },
    Vsop87Term {
        amplitude: 1.37804300000000e-05,
        phase: 1.12846591367000e+00,
        frequency: 1.17906290886588e+04,
    },
    Vsop87Term {
        amplitude: 4.98395000000000e-06,
        phase: 2.58682193892000e+00,
        frequency: 9.68359458111640e+03,
    },
    Vsop87Term {
        amplitude: 3.73958000000000e-06,
        phase: 1.42314832858000e+00,
        frequency: 3.93020969621960e+03,
    },
    Vsop87Term {
        amplitude: 2.63615000000000e-06,
        phase: 5.52938716941000e+00,
        frequency: 9.43776293488700e+03,
    },
    Vsop87Term {
        amplitude: 2.37454000000000e-06,
        phase: 2.55136053886000e+00,
        frequency: 1.57208387848784e+04,
    },
    Vsop87Term {
        amplitude: 2.21985000000000e-06,
        phase: 2.01346696541000e+00,
        frequency: 1.93671891622328e+04,
    },
    Vsop87Term {
        amplitude: 1.19466000000000e-06,
        phase: 3.01975080538000e+00,
        frequency: 1.04047338123226e+04,
    },
    Vsop87Term {
        amplitude: 1.25896000000000e-06,
        phase: 2.72769850819000e+00,
        frequency: 1.57734354244780e+03,
    },
    Vsop87Term {
        amplitude: 7.61760000000000e-07,
        phase: 1.59574968674000e+00,
        frequency: 9.15390361602180e+03,
    },
    Vsop87Term {
        amplitude: 8.53370000000000e-07,
        phase: 3.98598666191000e+00,
        frequency: 1.96510484810980e+04,
    },
    Vsop87Term {
        amplitude: 7.43470000000000e-07,
        phase: 4.11957779786000e+00,
        frequency: 5.50755323866740e+03,
    },
    Vsop87Term {
        amplitude: 4.19020000000000e-07,
        phase: 1.64282225331000e+00,
        frequency: 1.88374981971382e+04,
    },
    Vsop87Term {
        amplitude: 4.24940000000000e-07,
        phase: 3.81864493274000e+00,
        frequency: 1.33679726311066e+04,
    },
    Vsop87Term {
        amplitude: 3.94370000000000e-07,
        phase: 5.39018702243000e+00,
        frequency: 2.35812581773176e+04,
    },
    Vsop87Term {
        amplitude: 2.90420000000000e-07,
        phase: 5.67739528728000e+00,
        frequency: 5.66133204915220e+03,
    },
    Vsop87Term {
        amplitude: 2.75550000000000e-07,
        phase: 5.72392434415000e+00,
        frequency: 7.75522611324000e+02,
    },
];

const VENUS_R_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 3.45510410000000e-04,
        phase: 8.91987062760000e-01,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 2.34203000000000e-06,
        phase: 1.77224942363000e+00,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 2.33998000000000e-06,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 2.38670000000000e-07,
        phase: 1.11270233944000e+00,
        frequency: 9.43776293488700e+03,
    },
    Vsop87Term {
        amplitude: 1.05710000000000e-07,
        phase: 4.59152848465000e+00,
        frequency: 1.57734354244780e+03,
    },
    Vsop87Term {
        amplitude: 9.12400000000000e-08,
        phase: 4.53540895241000e+00,
        frequency: 1.04047338123226e+04,
    },
    Vsop87Term {
        amplitude: 6.60000000000000e-08,
        phase: 5.97725159435000e+00,
        frequency: 5.50755323866740e+03,
    },
    Vsop87Term {
        amplitude: 4.66500000000000e-08,
        phase: 3.87732289579000e+00,
        frequency: 9.15390361602180e+03,
    },
    Vsop87Term {
        amplitude: 3.84000000000000e-08,
        phase: 5.66215379445000e+00,
        frequency: 1.33679726311066e+04,
    },
    Vsop87Term {
        amplitude: 2.66200000000000e-08,
        phase: 2.82393816664000e+00,
        frequency: 1.02061719992102e+04,
    },
    Vsop87Term {
        amplitude: 2.19400000000000e-08,
        phase: 2.05324060020000e+00,
        frequency: 7.75522611324000e+02,
    },
    Vsop87Term {
        amplitude: 2.09300000000000e-08,
        phase: 2.54944827541000e+00,
        frequency: 1.88374981971382e+04,
    },
    Vsop87Term {
        amplitude: 1.78100000000000e-08,
        phase: 2.64889239766000e+00,
        frequency: 3.06398566386330e+04,
    },
    Vsop87Term {
        amplitude: 1.84400000000000e-08,
        phase: 1.87568236008000e+00,
        frequency: 1.10151064773348e+04,
    },
    Vsop87Term {
        amplitude: 1.30300000000000e-08,
        phase: 2.06130456040000e-01,
        frequency: 1.13226640983044e+04,
    },
    Vsop87Term {
        amplitude: 1.16800000000000e-08,
        phase: 7.94428928530000e-01,
        frequency: 1.72981823273262e+04,
    },
    Vsop87Term {
        amplitude: 1.00200000000000e-08,
        phase: 6.16544615317000e+00,
        frequency: 1.02395838660108e+04,
    },
    Vsop87Term {
        amplitude: 9.15000000000000e-09,
        phase: 4.59854496963000e+00,
        frequency: 1.10937855209340e+03,
    },
    Vsop87Term {
        amplitude: 8.82000000000000e-09,
        phase: 6.68005674170000e-01,
        frequency: 1.80737049386502e+04,
    },
    Vsop87Term {
        amplitude: 8.46000000000000e-09,
        phase: 5.58765716729000e+00,
        frequency: 1.25661516999828e+04,
    },
];

const VENUS_R_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.40658700000000e-05,
        phase: 5.06366395112000e+00,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 1.55290000000000e-07,
        phase: 5.47321056992000e+00,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 1.30590000000000e-07,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.09600000000000e-08,
        phase: 2.78919545899000e+00,
        frequency: 9.43776293488700e+03,
    },
    Vsop87Term {
        amplitude: 4.87000000000000e-09,
        phase: 6.27655636902000e+00,
        frequency: 1.57734354244780e+03,
    },
    Vsop87Term {
        amplitude: 3.61000000000000e-09,
        phase: 6.11959389556000e+00,
        frequency: 1.04047338123226e+04,
    },
    Vsop87Term {
        amplitude: 3.10000000000000e-09,
        phase: 1.39073645837000e+00,
        frequency: 5.50755323866740e+03,
    },
    Vsop87Term {
        amplitude: 3.85000000000000e-09,
        phase: 1.95564555688000e+00,
        frequency: 1.10151064773348e+04,
    },
    Vsop87Term {
        amplitude: 3.71000000000000e-09,
        phase: 2.33232050485000e+00,
        frequency: 7.75522611324000e+02,
    },
    Vsop87Term {
        amplitude: 2.07000000000000e-09,
        phase: 5.63406721595000e+00,
        frequency: 1.02395838660108e+04,
    },
    Vsop87Term {
        amplitude: 1.67000000000000e-09,
        phase: 1.11639732890000e+00,
        frequency: 1.33679726311066e+04,
    },
    Vsop87Term {
        amplitude: 1.75000000000000e-09,
        phase: 6.16674649733000e+00,
        frequency: 3.06398566386330e+04,
    },
    Vsop87Term {
        amplitude: 1.68000000000000e-09,
        phase: 3.64042170990000e+00,
        frequency: 7.08489678111520e+03,
    },
    Vsop87Term {
        amplitude: 1.20000000000000e-09,
        phase: 5.85861348966000e+00,
        frequency: 9.15390361602180e+03,
    },
    Vsop87Term {
        amplitude: 1.61000000000000e-09,
        phase: 2.21564443685000e+00,
        frequency: 3.15468708489560e+03,
    },
    Vsop87Term {
        amplitude: 1.18000000000000e-09,
        phase: 2.62362056521000e+00,
        frequency: 8.63594200376320e+03,
    },
    Vsop87Term {
        amplitude: 1.12000000000000e-09,
        phase: 2.36235956804000e+00,
        frequency: 1.05961820784342e+04,
    },
    Vsop87Term {
        amplitude: 9.30000000000000e-10,
        phase: 7.42511930580000e-01,
        frequency: 1.25661516999828e+04,
    },
    Vsop87Term {
        amplitude: 6.70000000000000e-10,
        phase: 3.76089669118000e+00,
        frequency: 1.88374981971382e+04,
    },
    Vsop87Term {
        amplitude: 6.50000000000000e-10,
        phase: 2.47990173302000e+00,
        frequency: 1.17906290886588e+04,
    },
];

const VENUS_R_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 4.95820000000000e-07,
        phase: 3.22264415899000e+00,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 8.31000000000000e-09,
        phase: 3.21255590531000e+00,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 1.12000000000000e-09,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.30000000000000e-10,
        phase: 3.77454760284000e+00,
        frequency: 3.06398566386330e+04,
    },
    Vsop87Term {
        amplitude: 8.00000000000000e-11,
        phase: 4.29674209391000e+00,
        frequency: 1.02395838660108e+04,
    },
    Vsop87Term {
        amplitude: 6.00000000000000e-11,
        phase: 3.91335213680000e-01,
        frequency: 1.01869872264112e+04,
    },
    Vsop87Term {
        amplitude: 4.00000000000000e-11,
        phase: 4.77456526708000e+00,
        frequency: 9.43776293488700e+03,
    },
    Vsop87Term {
        amplitude: 3.00000000000000e-11,
        phase: 1.46047829450000e-01,
        frequency: 1.09888081575350e+04,
    },
];

const VENUS_R_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 5.73000000000000e-09,
        phase: 9.22535255920000e-01,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 3.90000000000000e-10,
        phase: 9.56967873030000e-01,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 6.00000000000000e-11,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
];

const VENUS_R_5: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 4.50000000000000e-10,
        phase: 3.00370148080000e-01,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 2.00000000000000e-11,
        phase: 5.33215705373000e+00,
        frequency: 2.04265710924220e+04,
    },
];
