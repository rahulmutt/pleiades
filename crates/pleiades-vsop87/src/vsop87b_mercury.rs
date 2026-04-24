//! Truncated VSOP87B Mercury coefficient tables.
//!
//! Coefficients are the leading terms from the public IMCCE/CELMECH
//! `VSOP87B.mer` file (Mercury heliocentric spherical variables, J2000
//! ecliptic/equinox). This mirrors the Earth slice used for the Sun path
//! while complete generated tables are still planned.
use crate::vsop87b_earth::{evaluate, SphericalLbr, Vsop87Term};
pub(crate) fn mercury_lbr(julian_day_tt: f64) -> SphericalLbr {
    let t = (julian_day_tt - 2_451_545.0) / 365_250.0;
    SphericalLbr {
        longitude_rad: evaluate(
            &[
                MERCURY_L_0,
                MERCURY_L_1,
                MERCURY_L_2,
                MERCURY_L_3,
                MERCURY_L_4,
                MERCURY_L_5,
            ],
            t,
        )
        .rem_euclid(core::f64::consts::TAU),
        latitude_rad: evaluate(
            &[
                MERCURY_B_0,
                MERCURY_B_1,
                MERCURY_B_2,
                MERCURY_B_3,
                MERCURY_B_4,
                MERCURY_B_5,
            ],
            t,
        ),
        radius_au: evaluate(
            &[
                MERCURY_R_0,
                MERCURY_R_1,
                MERCURY_R_2,
                MERCURY_R_3,
                MERCURY_R_4,
                MERCURY_R_5,
            ],
            t,
        ),
    }
}
const MERCURY_L_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 4.40250710144000e+00,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 4.09894149770000e-01,
        phase: 1.48302034195000e+00,
        frequency: 2.60879031415742e+04,
    },
    Vsop87Term {
        amplitude: 5.04629420000000e-02,
        phase: 4.47785489551000e+00,
        frequency: 5.21758062831484e+04,
    },
    Vsop87Term {
        amplitude: 8.55346844000000e-03,
        phase: 1.16520322459000e+00,
        frequency: 7.82637094247226e+04,
    },
    Vsop87Term {
        amplitude: 1.65590362000000e-03,
        phase: 4.11969163423000e+00,
        frequency: 1.04351612566297e+05,
    },
    Vsop87Term {
        amplitude: 3.45618970000000e-04,
        phase: 7.79307684430000e-01,
        frequency: 1.30439515707871e+05,
    },
    Vsop87Term {
        amplitude: 7.58347600000000e-05,
        phase: 3.71348404924000e+00,
        frequency: 1.56527418849445e+05,
    },
    Vsop87Term {
        amplitude: 3.55974500000000e-05,
        phase: 1.51202675145000e+00,
        frequency: 1.10937855209340e+03,
    },
    Vsop87Term {
        amplitude: 1.72601100000000e-05,
        phase: 3.58322670960000e-01,
        frequency: 1.82615321991019e+05,
    },
    Vsop87Term {
        amplitude: 1.80346400000000e-05,
        phase: 4.10333184211000e+00,
        frequency: 5.66133204915220e+03,
    },
    Vsop87Term {
        amplitude: 1.36468100000000e-05,
        phase: 4.59918328256000e+00,
        frequency: 2.71972816936676e+04,
    },
    Vsop87Term {
        amplitude: 1.58992300000000e-05,
        phase: 2.99510423560000e+00,
        frequency: 2.50285212113850e+04,
    },
    Vsop87Term {
        amplitude: 1.01733200000000e-05,
        phase: 8.80313938240000e-01,
        frequency: 3.17492351907264e+04,
    },
    Vsop87Term {
        amplitude: 7.14182000000000e-06,
        phase: 1.54144862493000e+00,
        frequency: 2.49785245894808e+04,
    },
    Vsop87Term {
        amplitude: 6.43759000000000e-06,
        phase: 5.30266166599000e+00,
        frequency: 2.15359496445154e+04,
    },
    Vsop87Term {
        amplitude: 4.04200000000000e-06,
        phase: 3.28228953196000e+00,
        frequency: 2.08703225132594e+05,
    },
    Vsop87Term {
        amplitude: 3.52442000000000e-06,
        phase: 5.24156372447000e+00,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 3.43312000000000e-06,
        phase: 5.76531703870000e+00,
        frequency: 9.55599741608600e+02,
    },
    Vsop87Term {
        amplitude: 3.39215000000000e-06,
        phase: 5.86327825226000e+00,
        frequency: 2.55582121764796e+04,
    },
    Vsop87Term {
        amplitude: 4.51137000000000e-06,
        phase: 6.04989282259000e+00,
        frequency: 5.11164243529592e+04,
    },
];

const MERCURY_L_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.60879031368553e+04,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.13119981100000e-02,
        phase: 6.21874197797000e+00,
        frequency: 2.60879031415742e+04,
    },
    Vsop87Term {
        amplitude: 2.92242298000000e-03,
        phase: 3.04449355541000e+00,
        frequency: 5.21758062831484e+04,
    },
    Vsop87Term {
        amplitude: 7.57750810000000e-04,
        phase: 6.08568821653000e+00,
        frequency: 7.82637094247226e+04,
    },
    Vsop87Term {
        amplitude: 1.96765250000000e-04,
        phase: 2.80965111777000e+00,
        frequency: 1.04351612566297e+05,
    },
    Vsop87Term {
        amplitude: 5.11988300000000e-05,
        phase: 5.79432353574000e+00,
        frequency: 1.30439515707871e+05,
    },
    Vsop87Term {
        amplitude: 1.33632400000000e-05,
        phase: 2.47909947012000e+00,
        frequency: 1.56527418849445e+05,
    },
    Vsop87Term {
        amplitude: 3.52230000000000e-06,
        phase: 3.05246348628000e+00,
        frequency: 1.10937855209340e+03,
    },
    Vsop87Term {
        amplitude: 3.50236000000000e-06,
        phase: 5.43397743985000e+00,
        frequency: 1.82615321991019e+05,
    },
    Vsop87Term {
        amplitude: 9.34440000000000e-07,
        phase: 6.11761855456000e+00,
        frequency: 2.71972816936676e+04,
    },
    Vsop87Term {
        amplitude: 9.05880000000000e-07,
        phase: 5.37330310000000e-04,
        frequency: 2.49785245894808e+04,
    },
    Vsop87Term {
        amplitude: 9.22590000000000e-07,
        phase: 2.09530377053000e+00,
        frequency: 2.08703225132594e+05,
    },
    Vsop87Term {
        amplitude: 5.19430000000000e-07,
        phase: 5.62157845897000e+00,
        frequency: 5.66133204915220e+03,
    },
    Vsop87Term {
        amplitude: 4.43430000000000e-07,
        phase: 4.57417248957000e+00,
        frequency: 2.50285212113850e+04,
    },
    Vsop87Term {
        amplitude: 2.76510000000000e-07,
        phase: 3.03660330131000e+00,
        frequency: 5.10664277310550e+04,
    },
    Vsop87Term {
        amplitude: 2.19940000000000e-07,
        phase: 8.64751821600000e-01,
        frequency: 9.55599741608600e+02,
    },
    Vsop87Term {
        amplitude: 2.03780000000000e-07,
        phase: 3.71392682666000e+00,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 2.02260000000000e-07,
        phase: 5.20206496310000e-01,
        frequency: 2.15359496445154e+04,
    },
    Vsop87Term {
        amplitude: 2.44450000000000e-07,
        phase: 5.03171884876000e+00,
        frequency: 2.34791128274168e+05,
    },
    Vsop87Term {
        amplitude: 1.75070000000000e-07,
        phase: 5.72782246025000e+00,
        frequency: 4.55195349705880e+03,
    },
];

const MERCURY_L_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.63951290000000e-04,
        phase: 4.67759555504000e+00,
        frequency: 2.60879031415742e+04,
    },
    Vsop87Term {
        amplitude: 8.12386500000000e-05,
        phase: 1.40305644134000e+00,
        frequency: 5.21758062831484e+04,
    },
    Vsop87Term {
        amplitude: 3.20817000000000e-05,
        phase: 4.49577853102000e+00,
        frequency: 7.82637094247226e+04,
    },
    Vsop87Term {
        amplitude: 1.12820900000000e-05,
        phase: 1.27901273779000e+00,
        frequency: 1.04351612566297e+05,
    },
    Vsop87Term {
        amplitude: 8.77186000000000e-06,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 3.71058000000000e-06,
        phase: 4.31735787338000e+00,
        frequency: 1.30439515707871e+05,
    },
    Vsop87Term {
        amplitude: 1.16931000000000e-06,
        phase: 1.04943307731000e+00,
        frequency: 1.56527418849445e+05,
    },
    Vsop87Term {
        amplitude: 3.58020000000000e-07,
        phase: 4.04587257390000e+00,
        frequency: 1.82615321991019e+05,
    },
    Vsop87Term {
        amplitude: 1.48970000000000e-07,
        phase: 4.63345988506000e+00,
        frequency: 1.10937855209340e+03,
    },
    Vsop87Term {
        amplitude: 1.07470000000000e-07,
        phase: 7.43529251790000e-01,
        frequency: 2.08703225132594e+05,
    },
    Vsop87Term {
        amplitude: 5.24400000000000e-08,
        phase: 4.71804553686000e+00,
        frequency: 2.49785245894808e+04,
    },
    Vsop87Term {
        amplitude: 3.18200000000000e-08,
        phase: 3.71128464182000e+00,
        frequency: 2.34791128274168e+05,
    },
    Vsop87Term {
        amplitude: 2.54700000000000e-08,
        phase: 1.43801901419000e+00,
        frequency: 2.71972816936676e+04,
    },
    Vsop87Term {
        amplitude: 2.03300000000000e-08,
        phase: 1.49538090708000e+00,
        frequency: 5.10664277310550e+04,
    },
    Vsop87Term {
        amplitude: 9.72000000000000e-09,
        phase: 1.80406148095000e+00,
        frequency: 9.55599741608600e+02,
    },
    Vsop87Term {
        amplitude: 9.33000000000000e-09,
        phase: 3.85080640820000e-01,
        frequency: 2.60879031415742e+05,
    },
    Vsop87Term {
        amplitude: 6.28000000000000e-09,
        phase: 6.18336027299000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 6.28000000000000e-09,
        phase: 4.84993612548000e+00,
        frequency: 2.44988302462904e+04,
    },
    Vsop87Term {
        amplitude: 7.48000000000000e-09,
        phase: 4.53886632656000e+00,
        frequency: 7.71543308726292e+04,
    },
    Vsop87Term {
        amplitude: 6.52000000000000e-09,
        phase: 9.82441036230000e-01,
        frequency: 5.66133204915220e+03,
    },
];

const MERCURY_L_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.69496000000000e-06,
        phase: 3.20221586818000e+00,
        frequency: 2.60879031415742e+04,
    },
    Vsop87Term {
        amplitude: 1.55725000000000e-06,
        phase: 6.23814315369000e+00,
        frequency: 5.21758062831484e+04,
    },
    Vsop87Term {
        amplitude: 9.05550000000000e-07,
        phase: 2.96712953186000e+00,
        frequency: 7.82637094247226e+04,
    },
    Vsop87Term {
        amplitude: 4.27690000000000e-07,
        phase: 6.01870391709000e+00,
        frequency: 1.04351612566297e+05,
    },
    Vsop87Term {
        amplitude: 1.77600000000000e-07,
        phase: 2.78750960026000e+00,
        frequency: 1.30439515707871e+05,
    },
    Vsop87Term {
        amplitude: 6.77400000000000e-08,
        phase: 5.82756176337000e+00,
        frequency: 1.56527418849445e+05,
    },
    Vsop87Term {
        amplitude: 3.48600000000000e-08,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 2.43500000000000e-08,
        phase: 2.56963684564000e+00,
        frequency: 1.82615321991019e+05,
    },
    Vsop87Term {
        amplitude: 8.38000000000000e-09,
        phase: 5.58026725886000e+00,
        frequency: 2.08703225132594e+05,
    },
    Vsop87Term {
        amplitude: 2.79000000000000e-09,
        phase: 2.29386373858000e+00,
        frequency: 2.34791128274168e+05,
    },
    Vsop87Term {
        amplitude: 1.17000000000000e-09,
        phase: 3.16711243445000e+00,
        frequency: 2.49785245894808e+04,
    },
    Vsop87Term {
        amplitude: 9.10000000000000e-10,
        phase: 5.27797094839000e+00,
        frequency: 2.60879031415742e+05,
    },
    Vsop87Term {
        amplitude: 7.70000000000000e-10,
        phase: 6.24118948880000e+00,
        frequency: 5.10664277310550e+04,
    },
    Vsop87Term {
        amplitude: 5.90000000000000e-10,
        phase: 6.13122855286000e+00,
        frequency: 2.71972816936676e+04,
    },
    Vsop87Term {
        amplitude: 2.90000000000000e-10,
        phase: 1.96984426125000e+00,
        frequency: 2.86966934557316e+05,
    },
    Vsop87Term {
        amplitude: 3.80000000000000e-10,
        phase: 3.01313871348000e+00,
        frequency: 7.71543308726292e+04,
    },
    Vsop87Term {
        amplitude: 2.60000000000000e-10,
        phase: 3.07099212705000e+00,
        frequency: 5.32851848352418e+04,
    },
    Vsop87Term {
        amplitude: 1.60000000000000e-10,
        phase: 6.05402853609000e+00,
        frequency: 1.03242234014203e+05,
    },
    Vsop87Term {
        amplitude: 9.00000000000000e-11,
        phase: 5.87614052800000e-01,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 8.00000000000000e-11,
        phase: 4.91858279103000e+00,
        frequency: 3.13054837698890e+05,
    },
];

const MERCURY_L_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.67100000000000e-08,
        phase: 4.76418299344000e+00,
        frequency: 5.21758062831484e+04,
    },
    Vsop87Term {
        amplitude: 2.07900000000000e-08,
        phase: 2.01782765964000e+00,
        frequency: 2.60879031415742e+04,
    },
    Vsop87Term {
        amplitude: 2.07100000000000e-08,
        phase: 1.47603650163000e+00,
        frequency: 7.82637094247226e+04,
    },
    Vsop87Term {
        amplitude: 1.24800000000000e-08,
        phase: 4.50170414847000e+00,
        frequency: 1.04351612566297e+05,
    },
    Vsop87Term {
        amplitude: 6.41000000000000e-09,
        phase: 1.26049541246000e+00,
        frequency: 1.30439515707871e+05,
    },
    Vsop87Term {
        amplitude: 2.93000000000000e-09,
        phase: 4.30408398706000e+00,
        frequency: 1.56527418849445e+05,
    },
    Vsop87Term {
        amplitude: 1.24000000000000e-09,
        phase: 1.05833043353000e+00,
        frequency: 1.82615321991019e+05,
    },
    Vsop87Term {
        amplitude: 4.90000000000000e-10,
        phase: 4.08707632054000e+00,
        frequency: 2.08703225132594e+05,
    },
    Vsop87Term {
        amplitude: 4.70000000000000e-10,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.80000000000000e-10,
        phase: 8.23737292080000e-01,
        frequency: 2.34791128274168e+05,
    },
    Vsop87Term {
        amplitude: 6.00000000000000e-11,
        phase: 3.82469204536000e+00,
        frequency: 2.60879031415742e+05,
    },
    Vsop87Term {
        amplitude: 2.00000000000000e-11,
        phase: 5.00741161560000e-01,
        frequency: 2.86966934557316e+05,
    },
    Vsop87Term {
        amplitude: 2.00000000000000e-11,
        phase: 4.68954881838000e+00,
        frequency: 5.10664277310550e+04,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-11,
        phase: 1.63693177813000e+00,
        frequency: 2.49785245894808e+04,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-11,
        phase: 3.50676934033000e+00,
        frequency: 3.13054837698890e+05,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-11,
        phase: 4.59925122335000e+00,
        frequency: 2.71972816936676e+04,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-11,
        phase: 4.34568926758000e+00,
        frequency: 7.93730879768160e+04,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-11,
        phase: 1.32214662513000e+00,
        frequency: 5.32851848352418e+04,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-11,
        phase: 1.23275471446000e+00,
        frequency: 1.29330137155778e+05,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-11,
        phase: 1.52702783547000e+00,
        frequency: 7.71543308726292e+04,
    },
];

const MERCURY_L_5: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 3.90000000000000e-10,
        phase: 6.22596606829000e+00,
        frequency: 7.82637094247226e+04,
    },
    Vsop87Term {
        amplitude: 3.50000000000000e-10,
        phase: 3.08442751462000e+00,
        frequency: 5.21758062831484e+04,
    },
    Vsop87Term {
        amplitude: 3.60000000000000e-10,
        phase: 5.58268731752000e+00,
        frequency: 2.60879031415742e+04,
    },
    Vsop87Term {
        amplitude: 3.00000000000000e-10,
        phase: 2.98600396234000e+00,
        frequency: 1.04351612566297e+05,
    },
    Vsop87Term {
        amplitude: 1.90000000000000e-10,
        phase: 6.02016521976000e+00,
        frequency: 1.30439515707871e+05,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-10,
        phase: 2.77948950109000e+00,
        frequency: 1.56527418849445e+05,
    },
    Vsop87Term {
        amplitude: 5.00000000000000e-11,
        phase: 5.82587725191000e+00,
        frequency: 1.82615321991019e+05,
    },
    Vsop87Term {
        amplitude: 2.00000000000000e-11,
        phase: 2.57971347645000e+00,
        frequency: 2.08703225132594e+05,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-11,
        phase: 5.62474344959000e+00,
        frequency: 2.34791128274168e+05,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-11,
        phase: 2.30011142231000e+00,
        frequency: 2.60879031415742e+05,
    },
    Vsop87Term {
        amplitude: 0.00000000000000e+00,
        phase: 5.32570213434000e+00,
        frequency: 2.86966934557316e+05,
    },
    Vsop87Term {
        amplitude: 0.00000000000000e+00,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
];

const MERCURY_B_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.17375289610000e-01,
        phase: 1.98357498767000e+00,
        frequency: 2.60879031415742e+04,
    },
    Vsop87Term {
        amplitude: 2.38807699600000e-02,
        phase: 5.03738959686000e+00,
        frequency: 5.21758062831484e+04,
    },
    Vsop87Term {
        amplitude: 1.22283953200000e-02,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 5.43251810000000e-03,
        phase: 1.79644363964000e+00,
        frequency: 7.82637094247226e+04,
    },
    Vsop87Term {
        amplitude: 1.29778770000000e-03,
        phase: 4.83232503958000e+00,
        frequency: 1.04351612566297e+05,
    },
    Vsop87Term {
        amplitude: 3.18669270000000e-04,
        phase: 1.58088495658000e+00,
        frequency: 1.30439515707871e+05,
    },
    Vsop87Term {
        amplitude: 7.96330100000000e-05,
        phase: 4.60972126127000e+00,
        frequency: 1.56527418849445e+05,
    },
    Vsop87Term {
        amplitude: 2.01418900000000e-05,
        phase: 1.35324164377000e+00,
        frequency: 1.82615321991019e+05,
    },
    Vsop87Term {
        amplitude: 5.13953000000000e-06,
        phase: 4.37835406663000e+00,
        frequency: 2.08703225132594e+05,
    },
    Vsop87Term {
        amplitude: 2.07674000000000e-06,
        phase: 4.91772567908000e+00,
        frequency: 2.71972816936676e+04,
    },
    Vsop87Term {
        amplitude: 2.08584000000000e-06,
        phase: 2.02020295489000e+00,
        frequency: 2.49785245894808e+04,
    },
    Vsop87Term {
        amplitude: 1.32013000000000e-06,
        phase: 1.11908482553000e+00,
        frequency: 2.34791128274168e+05,
    },
    Vsop87Term {
        amplitude: 1.00454000000000e-06,
        phase: 5.65684757892000e+00,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 1.21395000000000e-06,
        phase: 1.81271747279000e+00,
        frequency: 5.32851848352418e+04,
    },
    Vsop87Term {
        amplitude: 9.15660000000000e-07,
        phase: 2.28163127292000e+00,
        frequency: 2.50285212113850e+04,
    },
    Vsop87Term {
        amplitude: 9.92140000000000e-07,
        phase: 9.39188789700000e-02,
        frequency: 5.11164243529592e+04,
    },
    Vsop87Term {
        amplitude: 9.45740000000000e-07,
        phase: 1.24184920920000e+00,
        frequency: 3.17492351907264e+04,
    },
    Vsop87Term {
        amplitude: 7.87850000000000e-07,
        phase: 4.40725881159000e+00,
        frequency: 5.78371383323006e+04,
    },
    Vsop87Term {
        amplitude: 7.77470000000000e-07,
        phase: 5.25570744330000e-01,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 8.42640000000000e-07,
        phase: 5.08510405853000e+00,
        frequency: 5.10664277310550e+04,
    },
];

const MERCURY_B_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.74646065000000e-03,
        phase: 3.95008450011000e+00,
        frequency: 2.60879031415742e+04,
    },
    Vsop87Term {
        amplitude: 9.97377130000000e-04,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.87720470000000e-04,
        phase: 5.14128888700000e-02,
        frequency: 7.82637094247226e+04,
    },
    Vsop87Term {
        amplitude: 2.39707260000000e-04,
        phase: 2.53272082947000e+00,
        frequency: 5.21758062831484e+04,
    },
    Vsop87Term {
        amplitude: 8.09750800000000e-05,
        phase: 3.20946389315000e+00,
        frequency: 1.04351612566297e+05,
    },
    Vsop87Term {
        amplitude: 2.89072900000000e-05,
        phase: 9.43621371000000e-03,
        frequency: 1.30439515707871e+05,
    },
    Vsop87Term {
        amplitude: 9.49669000000000e-06,
        phase: 3.06780459575000e+00,
        frequency: 1.56527418849445e+05,
    },
    Vsop87Term {
        amplitude: 2.98013000000000e-06,
        phase: 6.11414444304000e+00,
        frequency: 1.82615321991019e+05,
    },
    Vsop87Term {
        amplitude: 9.08630000000000e-07,
        phase: 2.87023913203000e+00,
        frequency: 2.08703225132594e+05,
    },
    Vsop87Term {
        amplitude: 2.71630000000000e-07,
        phase: 5.90488705529000e+00,
        frequency: 2.34791128274168e+05,
    },
    Vsop87Term {
        amplitude: 2.46770000000000e-07,
        phase: 3.72101766080000e-01,
        frequency: 2.71972816936676e+04,
    },
    Vsop87Term {
        amplitude: 1.60010000000000e-07,
        phase: 3.74996854220000e-01,
        frequency: 2.49785245894808e+04,
    },
    Vsop87Term {
        amplitude: 1.10350000000000e-07,
        phase: 3.48855329110000e+00,
        frequency: 5.32851848352418e+04,
    },
    Vsop87Term {
        amplitude: 8.00400000000000e-08,
        phase: 2.65315026358000e+00,
        frequency: 2.60879031415742e+05,
    },
    Vsop87Term {
        amplitude: 8.81700000000000e-08,
        phase: 3.46732763537000e+00,
        frequency: 5.10664277310550e+04,
    },
    Vsop87Term {
        amplitude: 4.86300000000000e-08,
        phase: 3.13533089859000e+00,
        frequency: 3.17492351907264e+04,
    },
    Vsop87Term {
        amplitude: 4.41900000000000e-08,
        phase: 2.23904465242000e+00,
        frequency: 5.11164243529592e+04,
    },
    Vsop87Term {
        amplitude: 3.81400000000000e-08,
        phase: 2.55486935220000e-01,
        frequency: 7.71543308726292e+04,
    },
    Vsop87Term {
        amplitude: 2.81500000000000e-08,
        phase: 2.62022898590000e-01,
        frequency: 7.93730879768160e+04,
    },
    Vsop87Term {
        amplitude: 2.53000000000000e-08,
        phase: 9.38224403760000e-01,
        frequency: 2.50285212113850e+04,
    },
];

const MERCURY_B_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.74716500000000e-05,
        phase: 5.24567337999000e+00,
        frequency: 2.60879031415742e+04,
    },
    Vsop87Term {
        amplitude: 2.04725700000000e-05,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 5.16030000000000e-06,
        phase: 4.93211331540000e-01,
        frequency: 5.21758062831484e+04,
    },
    Vsop87Term {
        amplitude: 4.07309000000000e-06,
        phase: 4.32215500849000e+00,
        frequency: 7.82637094247226e+04,
    },
    Vsop87Term {
        amplitude: 2.66936000000000e-06,
        phase: 1.42744634495000e+00,
        frequency: 1.04351612566297e+05,
    },
    Vsop87Term {
        amplitude: 1.33544000000000e-06,
        phase: 4.61055165903000e+00,
        frequency: 1.30439515707871e+05,
    },
    Vsop87Term {
        amplitude: 5.69560000000000e-07,
        phase: 1.44017544018000e+00,
        frequency: 1.56527418849445e+05,
    },
    Vsop87Term {
        amplitude: 2.20490000000000e-07,
        phase: 4.52127237069000e+00,
        frequency: 1.82615321991019e+05,
    },
    Vsop87Term {
        amplitude: 8.00800000000000e-08,
        phase: 1.30182043008000e+00,
        frequency: 2.08703225132594e+05,
    },
    Vsop87Term {
        amplitude: 2.78100000000000e-08,
        phase: 4.35468456951000e+00,
        frequency: 2.34791128274168e+05,
    },
    Vsop87Term {
        amplitude: 1.30400000000000e-08,
        phase: 2.02991901716000e+00,
        frequency: 2.71972816936676e+04,
    },
    Vsop87Term {
        amplitude: 9.34000000000000e-09,
        phase: 1.11727595126000e+00,
        frequency: 2.60879031415742e+05,
    },
    Vsop87Term {
        amplitude: 5.11000000000000e-09,
        phase: 4.80027921181000e+00,
        frequency: 2.49785245894808e+04,
    },
    Vsop87Term {
        amplitude: 4.33000000000000e-09,
        phase: 5.13987059401000e+00,
        frequency: 5.32851848352418e+04,
    },
    Vsop87Term {
        amplitude: 4.13000000000000e-09,
        phase: 1.75599872832000e+00,
        frequency: 5.10664277310550e+04,
    },
    Vsop87Term {
        amplitude: 3.06000000000000e-09,
        phase: 4.15800645361000e+00,
        frequency: 2.86966934557316e+05,
    },
    Vsop87Term {
        amplitude: 2.36000000000000e-09,
        phase: 4.88575156209000e+00,
        frequency: 7.71543308726292e+04,
    },
    Vsop87Term {
        amplitude: 1.23000000000000e-09,
        phase: 3.39712849299000e+00,
        frequency: 5.11164243529592e+04,
    },
    Vsop87Term {
        amplitude: 1.13000000000000e-09,
        phase: 1.69721095056000e+00,
        frequency: 1.03242234014203e+05,
    },
    Vsop87Term {
        amplitude: 1.09000000000000e-09,
        phase: 4.54767694492000e+00,
        frequency: 3.17492351907264e+04,
    },
];

const MERCURY_B_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 6.46800000000000e-07,
        phase: 2.16518315874000e+00,
        frequency: 2.60879031415742e+04,
    },
    Vsop87Term {
        amplitude: 3.07330000000000e-07,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.89290000000000e-07,
        phase: 5.40870348072000e+00,
        frequency: 5.21758062831484e+04,
    },
    Vsop87Term {
        amplitude: 9.79700000000000e-08,
        phase: 2.41402344018000e+00,
        frequency: 7.82637094247226e+04,
    },
    Vsop87Term {
        amplitude: 6.86100000000000e-08,
        phase: 5.88312096876000e+00,
        frequency: 1.04351612566297e+05,
    },
    Vsop87Term {
        amplitude: 4.36700000000000e-08,
        phase: 2.88362764626000e+00,
        frequency: 1.30439515707871e+05,
    },
    Vsop87Term {
        amplitude: 2.34400000000000e-08,
        phase: 6.05581664620000e+00,
        frequency: 1.56527418849445e+05,
    },
    Vsop87Term {
        amplitude: 1.10500000000000e-08,
        phase: 2.89178837278000e+00,
        frequency: 1.82615321991019e+05,
    },
    Vsop87Term {
        amplitude: 4.75000000000000e-09,
        phase: 5.98256115875000e+00,
        frequency: 2.08703225132594e+05,
    },
    Vsop87Term {
        amplitude: 1.91000000000000e-09,
        phase: 2.77298018505000e+00,
        frequency: 2.34791128274168e+05,
    },
    Vsop87Term {
        amplitude: 7.30000000000000e-10,
        phase: 5.83474996935000e+00,
        frequency: 2.60879031415742e+05,
    },
    Vsop87Term {
        amplitude: 2.70000000000000e-10,
        phase: 2.60945701067000e+00,
        frequency: 2.86966934557316e+05,
    },
    Vsop87Term {
        amplitude: 2.40000000000000e-10,
        phase: 3.87337367986000e+00,
        frequency: 2.71972816936676e+04,
    },
    Vsop87Term {
        amplitude: 1.60000000000000e-10,
        phase: 1.06059146335000e+00,
        frequency: 2.49785245894808e+04,
    },
    Vsop87Term {
        amplitude: 8.00000000000000e-11,
        phase: 5.66780041298000e+00,
        frequency: 3.13054837698890e+05,
    },
    Vsop87Term {
        amplitude: 6.00000000000000e-11,
        phase: 5.58854950623000e+00,
        frequency: 5.10664277310550e+04,
    },
    Vsop87Term {
        amplitude: 7.00000000000000e-11,
        phase: 3.08546607033000e+00,
        frequency: 7.71543308726292e+04,
    },
    Vsop87Term {
        amplitude: 5.00000000000000e-11,
        phase: 2.98383171300000e-02,
        frequency: 1.03242234014203e+05,
    },
    Vsop87Term {
        amplitude: 3.00000000000000e-11,
        phase: 2.47133249668000e+00,
        frequency: 3.39142740840465e+05,
    },
    Vsop87Term {
        amplitude: 2.00000000000000e-11,
        phase: 1.54850606185000e+00,
        frequency: 1.10937855209340e+03,
    },
];

const MERCURY_B_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.42700000000000e-08,
        phase: 4.97519726738000e+00,
        frequency: 2.60879031415742e+04,
    },
    Vsop87Term {
        amplitude: 4.91000000000000e-09,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 3.36000000000000e-09,
        phase: 3.19691284098000e+00,
        frequency: 5.21758062831484e+04,
    },
    Vsop87Term {
        amplitude: 2.43000000000000e-09,
        phase: 5.77399476510000e-01,
        frequency: 7.82637094247226e+04,
    },
    Vsop87Term {
        amplitude: 1.70000000000000e-09,
        phase: 4.04262780835000e+00,
        frequency: 1.04351612566297e+05,
    },
    Vsop87Term {
        amplitude: 1.20000000000000e-09,
        phase: 1.12342918082000e+00,
        frequency: 1.30439515707871e+05,
    },
    Vsop87Term {
        amplitude: 7.60000000000000e-10,
        phase: 4.36272648537000e+00,
        frequency: 1.56527418849445e+05,
    },
    Vsop87Term {
        amplitude: 4.30000000000000e-10,
        phase: 1.23406162348000e+00,
        frequency: 1.82615321991019e+05,
    },
    Vsop87Term {
        amplitude: 2.20000000000000e-10,
        phase: 4.35637189777000e+00,
        frequency: 2.08703225132594e+05,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-10,
        phase: 1.16395130801000e+00,
        frequency: 2.34791128274168e+05,
    },
    Vsop87Term {
        amplitude: 4.00000000000000e-11,
        phase: 4.27354172628000e+00,
        frequency: 2.60879031415742e+05,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-11,
        phase: 1.02393401255000e+00,
        frequency: 2.86966934557316e+05,
    },
    Vsop87Term {
        amplitude: 0.00000000000000e+00,
        phase: 4.13470035458000e+00,
        frequency: 3.13054837698890e+05,
    },
];

const MERCURY_B_5: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 9.00000000000000e-11,
        phase: 1.38311629808000e+00,
        frequency: 5.21758062831484e+04,
    },
    Vsop87Term {
        amplitude: 8.00000000000000e-11,
        phase: 5.38548752147000e+00,
        frequency: 2.60879031415742e+04,
    },
    Vsop87Term {
        amplitude: 6.00000000000000e-11,
        phase: 4.90804019263000e+00,
        frequency: 7.82637094247226e+04,
    },
    Vsop87Term {
        amplitude: 6.00000000000000e-11,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 4.00000000000000e-11,
        phase: 2.13779173141000e+00,
        frequency: 1.04351612566297e+05,
    },
    Vsop87Term {
        amplitude: 3.00000000000000e-11,
        phase: 5.58133586504000e+00,
        frequency: 1.30439515707871e+05,
    },
    Vsop87Term {
        amplitude: 2.00000000000000e-11,
        phase: 2.64274667677000e+00,
        frequency: 1.56527418849445e+05,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-11,
        phase: 5.83772628756000e+00,
        frequency: 1.82615321991019e+05,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-11,
        phase: 2.68770315459000e+00,
        frequency: 2.08703225132594e+05,
    },
    Vsop87Term {
        amplitude: 0.00000000000000e+00,
        phase: 5.84365103714000e+00,
        frequency: 2.34791128274168e+05,
    },
];

const MERCURY_R_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 3.95282716510000e-01,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 7.83413181800000e-02,
        phase: 6.19233722598000e+00,
        frequency: 2.60879031415742e+04,
    },
    Vsop87Term {
        amplitude: 7.95525558000000e-03,
        phase: 2.95989690104000e+00,
        frequency: 5.21758062831484e+04,
    },
    Vsop87Term {
        amplitude: 1.21281764000000e-03,
        phase: 6.01064153797000e+00,
        frequency: 7.82637094247226e+04,
    },
    Vsop87Term {
        amplitude: 2.19219690000000e-04,
        phase: 2.77820093972000e+00,
        frequency: 1.04351612566297e+05,
    },
    Vsop87Term {
        amplitude: 4.35406500000000e-05,
        phase: 5.82894543774000e+00,
        frequency: 1.30439515707871e+05,
    },
    Vsop87Term {
        amplitude: 9.18228000000000e-06,
        phase: 2.59650562845000e+00,
        frequency: 1.56527418849445e+05,
    },
    Vsop87Term {
        amplitude: 2.60033000000000e-06,
        phase: 3.02817753901000e+00,
        frequency: 2.71972816936676e+04,
    },
    Vsop87Term {
        amplitude: 2.89955000000000e-06,
        phase: 1.42441937278000e+00,
        frequency: 2.50285212113850e+04,
    },
    Vsop87Term {
        amplitude: 2.01855000000000e-06,
        phase: 5.64725040577000e+00,
        frequency: 1.82615321991019e+05,
    },
    Vsop87Term {
        amplitude: 2.01498000000000e-06,
        phase: 5.59227727403000e+00,
        frequency: 3.17492351907264e+04,
    },
    Vsop87Term {
        amplitude: 1.41980000000000e-06,
        phase: 6.25264206514000e+00,
        frequency: 2.49785245894808e+04,
    },
    Vsop87Term {
        amplitude: 1.00144000000000e-06,
        phase: 3.73435615066000e+00,
        frequency: 2.15359496445154e+04,
    },
    Vsop87Term {
        amplitude: 7.75610000000000e-07,
        phase: 3.66972523786000e+00,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 6.32770000000000e-07,
        phase: 4.29905566028000e+00,
        frequency: 2.55582121764796e+04,
    },
    Vsop87Term {
        amplitude: 6.29510000000000e-07,
        phase: 4.76588960835000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 6.67530000000000e-07,
        phase: 2.52520325806000e+00,
        frequency: 5.66133204915220e+03,
    },
    Vsop87Term {
        amplitude: 7.55000000000000e-07,
        phase: 4.47428643135000e+00,
        frequency: 5.11164243529592e+04,
    },
    Vsop87Term {
        amplitude: 4.82650000000000e-07,
        phase: 6.06824353565000e+00,
        frequency: 5.32851848352418e+04,
    },
    Vsop87Term {
        amplitude: 4.57480000000000e-07,
        phase: 2.41480951848000e+00,
        frequency: 2.08703225132594e+05,
    },
];

const MERCURY_R_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.17347740000000e-03,
        phase: 4.65617158665000e+00,
        frequency: 2.60879031415742e+04,
    },
    Vsop87Term {
        amplitude: 4.41418260000000e-04,
        phase: 1.42385544001000e+00,
        frequency: 5.21758062831484e+04,
    },
    Vsop87Term {
        amplitude: 1.00944790000000e-04,
        phase: 4.47466326327000e+00,
        frequency: 7.82637094247226e+04,
    },
    Vsop87Term {
        amplitude: 2.43280500000000e-05,
        phase: 1.24226083323000e+00,
        frequency: 1.04351612566297e+05,
    },
    Vsop87Term {
        amplitude: 1.62436700000000e-05,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 6.03996000000000e-06,
        phase: 4.29303116468000e+00,
        frequency: 1.30439515707871e+05,
    },
    Vsop87Term {
        amplitude: 1.52851000000000e-06,
        phase: 1.06060778072000e+00,
        frequency: 1.56527418849445e+05,
    },
    Vsop87Term {
        amplitude: 3.92020000000000e-07,
        phase: 4.11136733071000e+00,
        frequency: 1.82615321991019e+05,
    },
    Vsop87Term {
        amplitude: 1.77600000000000e-07,
        phase: 4.54424729034000e+00,
        frequency: 2.71972816936676e+04,
    },
    Vsop87Term {
        amplitude: 1.79990000000000e-07,
        phase: 4.71193597233000e+00,
        frequency: 2.49785245894808e+04,
    },
    Vsop87Term {
        amplitude: 1.01540000000000e-07,
        phase: 8.78935409820000e-01,
        frequency: 2.08703225132594e+05,
    },
    Vsop87Term {
        amplitude: 8.08600000000000e-08,
        phase: 3.00540629863000e+00,
        frequency: 2.50285212113850e+04,
    },
    Vsop87Term {
        amplitude: 4.44400000000000e-08,
        phase: 2.13638817844000e+00,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 4.39300000000000e-08,
        phase: 1.48073536997000e+00,
        frequency: 5.10664277310550e+04,
    },
    Vsop87Term {
        amplitude: 3.51000000000000e-08,
        phase: 3.21169312709000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 3.13300000000000e-08,
        phase: 5.23846816226000e+00,
        frequency: 2.15359496445154e+04,
    },
    Vsop87Term {
        amplitude: 2.65000000000000e-08,
        phase: 3.92968869319000e+00,
        frequency: 2.34791128274168e+05,
    },
    Vsop87Term {
        amplitude: 2.49800000000000e-08,
        phase: 2.02627371234000e+00,
        frequency: 2.44988302462904e+04,
    },
    Vsop87Term {
        amplitude: 2.01100000000000e-08,
        phase: 1.23910805857000e+00,
        frequency: 5.32851848352418e+04,
    },
    Vsop87Term {
        amplitude: 1.96300000000000e-08,
        phase: 4.04525058702000e+00,
        frequency: 5.66133204915220e+03,
    },
];

const MERCURY_R_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 3.11786700000000e-05,
        phase: 3.08231840294000e+00,
        frequency: 2.60879031415742e+04,
    },
    Vsop87Term {
        amplitude: 1.24539700000000e-05,
        phase: 6.15183316810000e+00,
        frequency: 5.21758062831484e+04,
    },
    Vsop87Term {
        amplitude: 4.24822000000000e-06,
        phase: 2.92583350003000e+00,
        frequency: 7.82637094247226e+04,
    },
    Vsop87Term {
        amplitude: 1.36130000000000e-06,
        phase: 5.97983927257000e+00,
        frequency: 1.04351612566297e+05,
    },
    Vsop87Term {
        amplitude: 4.21760000000000e-07,
        phase: 2.74936984182000e+00,
        frequency: 1.30439515707871e+05,
    },
    Vsop87Term {
        amplitude: 2.17590000000000e-07,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.27940000000000e-07,
        phase: 5.80143158303000e+00,
        frequency: 1.56527418849445e+05,
    },
    Vsop87Term {
        amplitude: 3.82500000000000e-08,
        phase: 2.56993470104000e+00,
        frequency: 1.82615321991019e+05,
    },
    Vsop87Term {
        amplitude: 1.04200000000000e-08,
        phase: 3.14646747795000e+00,
        frequency: 2.49785245894808e+04,
    },
    Vsop87Term {
        amplitude: 1.13100000000000e-08,
        phase: 5.62140894157000e+00,
        frequency: 2.08703225132594e+05,
    },
    Vsop87Term {
        amplitude: 4.83000000000000e-09,
        phase: 6.14311665486000e+00,
        frequency: 2.71972816936676e+04,
    },
    Vsop87Term {
        amplitude: 3.32000000000000e-09,
        phase: 2.38990569407000e+00,
        frequency: 2.34791128274168e+05,
    },
    Vsop87Term {
        amplitude: 3.20000000000000e-09,
        phase: 6.20678671907000e+00,
        frequency: 5.10664277310550e+04,
    },
    Vsop87Term {
        amplitude: 1.20000000000000e-09,
        phase: 5.67532780810000e-01,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 1.13000000000000e-09,
        phase: 3.28087063421000e+00,
        frequency: 2.44988302462904e+04,
    },
    Vsop87Term {
        amplitude: 1.05000000000000e-09,
        phase: 4.36644825356000e+00,
        frequency: 2.50285212113850e+04,
    },
    Vsop87Term {
        amplitude: 9.70000000000000e-10,
        phase: 5.44142060651000e+00,
        frequency: 2.60879031415742e+05,
    },
    Vsop87Term {
        amplitude: 9.50000000000000e-10,
        phase: 1.70261963549000e+00,
        frequency: 1.05938193018920e+03,
    },
    Vsop87Term {
        amplitude: 1.03000000000000e-09,
        phase: 2.98025866325000e+00,
        frequency: 7.71543308726292e+04,
    },
    Vsop87Term {
        amplitude: 7.40000000000000e-10,
        phase: 1.28624418497000e+00,
        frequency: 2.66175941066688e+04,
    },
];

const MERCURY_R_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 3.26760000000000e-07,
        phase: 1.67971641967000e+00,
        frequency: 2.60879031415742e+04,
    },
    Vsop87Term {
        amplitude: 2.41660000000000e-07,
        phase: 4.63403168878000e+00,
        frequency: 5.21758062831484e+04,
    },
    Vsop87Term {
        amplitude: 1.21330000000000e-07,
        phase: 1.38983777816000e+00,
        frequency: 7.82637094247226e+04,
    },
    Vsop87Term {
        amplitude: 5.14100000000000e-08,
        phase: 4.43915486864000e+00,
        frequency: 1.04351612566297e+05,
    },
    Vsop87Term {
        amplitude: 1.98100000000000e-08,
        phase: 1.20734065292000e+00,
        frequency: 1.30439515707871e+05,
    },
    Vsop87Term {
        amplitude: 1.46000000000000e-08,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 7.19000000000000e-09,
        phase: 4.25914225052000e+00,
        frequency: 1.56527418849445e+05,
    },
    Vsop87Term {
        amplitude: 2.50000000000000e-09,
        phase: 1.02794489584000e+00,
        frequency: 1.82615321991019e+05,
    },
    Vsop87Term {
        amplitude: 8.40000000000000e-10,
        phase: 4.08003393556000e+00,
        frequency: 2.08703225132594e+05,
    },
    Vsop87Term {
        amplitude: 2.80000000000000e-10,
        phase: 8.49761298690000e-01,
        frequency: 2.34791128274168e+05,
    },
    Vsop87Term {
        amplitude: 2.30000000000000e-10,
        phase: 1.60056693427000e+00,
        frequency: 2.49785245894808e+04,
    },
    Vsop87Term {
        amplitude: 1.10000000000000e-10,
        phase: 4.57906901820000e+00,
        frequency: 2.71972816936676e+04,
    },
    Vsop87Term {
        amplitude: 1.20000000000000e-10,
        phase: 4.66186402631000e+00,
        frequency: 5.10664277310550e+04,
    },
    Vsop87Term {
        amplitude: 9.00000000000000e-11,
        phase: 3.90351105399000e+00,
        frequency: 2.60879031415742e+05,
    },
    Vsop87Term {
        amplitude: 5.00000000000000e-11,
        phase: 1.44899194699000e+00,
        frequency: 7.71543308726292e+04,
    },
    Vsop87Term {
        amplitude: 4.00000000000000e-11,
        phase: 1.37039350951000e+00,
        frequency: 5.32851848352418e+04,
    },
    Vsop87Term {
        amplitude: 3.00000000000000e-11,
        phase: 6.52161708360000e-01,
        frequency: 2.86966934557316e+05,
    },
    Vsop87Term {
        amplitude: 2.00000000000000e-11,
        phase: 5.31647339795000e+00,
        frequency: 2.04265710924220e+04,
    },
    Vsop87Term {
        amplitude: 2.00000000000000e-11,
        phase: 1.54948310012000e+00,
        frequency: 1.10937855209340e+03,
    },
    Vsop87Term {
        amplitude: 2.00000000000000e-11,
        phase: 4.46650154841000e+00,
        frequency: 1.03242234014203e+05,
    },
];

const MERCURY_R_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 3.94000000000000e-09,
        phase: 3.67367388360000e-01,
        frequency: 2.60879031415742e+04,
    },
    Vsop87Term {
        amplitude: 3.87000000000000e-09,
        phase: 3.18568894140000e+00,
        frequency: 5.21758062831484e+04,
    },
    Vsop87Term {
        amplitude: 2.70000000000000e-09,
        phase: 6.16979809593000e+00,
        frequency: 7.82637094247226e+04,
    },
    Vsop87Term {
        amplitude: 1.49000000000000e-09,
        phase: 2.91591472142000e+00,
        frequency: 1.04351612566297e+05,
    },
    Vsop87Term {
        amplitude: 7.10000000000000e-10,
        phase: 5.95888916295000e+00,
        frequency: 1.30439515707871e+05,
    },
    Vsop87Term {
        amplitude: 3.10000000000000e-10,
        phase: 2.72386331553000e+00,
        frequency: 1.56527418849445e+05,
    },
    Vsop87Term {
        amplitude: 1.70000000000000e-10,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.20000000000000e-10,
        phase: 5.77758679438000e+00,
        frequency: 1.82615321991019e+05,
    },
    Vsop87Term {
        amplitude: 5.00000000000000e-11,
        phase: 2.54442235521000e+00,
        frequency: 2.08703225132594e+05,
    },
    Vsop87Term {
        amplitude: 2.00000000000000e-11,
        phase: 5.59215484513000e+00,
        frequency: 2.34791128274168e+05,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-11,
        phase: 2.31734413223000e+00,
        frequency: 2.60879031415742e+05,
    },
    Vsop87Term {
        amplitude: 0.00000000000000e+00,
        phase: 5.37041038965000e+00,
        frequency: 2.86966934557316e+05,
    },
    Vsop87Term {
        amplitude: 0.00000000000000e+00,
        phase: 3.05050417438000e+00,
        frequency: 5.10664277310550e+04,
    },
    Vsop87Term {
        amplitude: 0.00000000000000e+00,
        phase: 7.05285459100000e-02,
        frequency: 2.49785245894808e+04,
    },
    Vsop87Term {
        amplitude: 0.00000000000000e+00,
        phase: 5.93192891756000e+00,
        frequency: 5.32851848352418e+04,
    },
    Vsop87Term {
        amplitude: 0.00000000000000e+00,
        phase: 6.03840913462000e+00,
        frequency: 7.71543308726292e+04,
    },
    Vsop87Term {
        amplitude: 0.00000000000000e+00,
        phase: 3.02089425425000e+00,
        frequency: 2.71972816936676e+04,
    },
];

const MERCURY_R_5: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 6.00000000000000e-11,
        phase: 3.98812118954000e+00,
        frequency: 2.60879031415742e+04,
    },
    Vsop87Term {
        amplitude: 6.00000000000000e-11,
        phase: 1.55172409309000e+00,
        frequency: 5.21758062831484e+04,
    },
    Vsop87Term {
        amplitude: 5.00000000000000e-11,
        phase: 4.65488347662000e+00,
        frequency: 7.82637094247226e+04,
    },
    Vsop87Term {
        amplitude: 4.00000000000000e-11,
        phase: 1.40628214181000e+00,
        frequency: 1.04351612566297e+05,
    },
    Vsop87Term {
        amplitude: 2.00000000000000e-11,
        phase: 4.44423794944000e+00,
        frequency: 1.30439515707871e+05,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-11,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-11,
        phase: 1.21235041448000e+00,
        frequency: 1.56527418849445e+05,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-11,
        phase: 4.24238056507000e+00,
        frequency: 1.82615321991019e+05,
    },
    Vsop87Term {
        amplitude: 0.00000000000000e+00,
        phase: 9.51401529370000e-01,
        frequency: 2.08703225132594e+05,
    },
    Vsop87Term {
        amplitude: 0.00000000000000e+00,
        phase: 4.00511196914000e+00,
        frequency: 2.34791128274168e+05,
    },
];
