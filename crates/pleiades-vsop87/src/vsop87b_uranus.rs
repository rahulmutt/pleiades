//! Truncated VSOP87B Uranus coefficient tables.
//!
//! Coefficients are the leading terms from the public IMCCE/CELMECH
//! `VSOP87B.ura` file (Uranus heliocentric spherical variables, J2000
//! ecliptic/equinox). This mirrors the other checked-in planetary slices
//! while complete generated tables are still planned.

use crate::vsop87b_earth::{evaluate, SphericalLbr, Vsop87Term};

pub(crate) fn uranus_lbr(julian_day_tt: f64) -> SphericalLbr {
    let t = (julian_day_tt - 2_451_545.0) / 365_250.0;
    SphericalLbr {
        longitude_rad: evaluate(
            [
                URANUS_L_0, URANUS_L_1, URANUS_L_2, URANUS_L_3, URANUS_L_4, URANUS_L_5,
            ],
            t,
        )
        .rem_euclid(core::f64::consts::TAU),
        latitude_rad: evaluate(
            [
                URANUS_B_0, URANUS_B_1, URANUS_B_2, URANUS_B_3, URANUS_B_4, URANUS_B_5,
            ],
            t,
        ),
        radius_au: evaluate(
            [
                URANUS_R_0, URANUS_R_1, URANUS_R_2, URANUS_R_3, URANUS_R_4, URANUS_R_5,
            ],
            t,
        ),
    }
}

const URANUS_L_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 5.48129294297000e+00,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 9.26040823400000e-02,
        phase: 8.91064215070000e-01,
        frequency: 7.47815985673000e+01,
    },
    Vsop87Term {
        amplitude: 1.50424789800000e-02,
        phase: 3.62719260920000e+00,
        frequency: 1.48447270830000e+00,
    },
    Vsop87Term {
        amplitude: 3.65981674000000e-03,
        phase: 1.89962179044000e+00,
        frequency: 7.32971258590000e+01,
    },
    Vsop87Term {
        amplitude: 2.72328168000000e-03,
        phase: 3.35823706307000e+00,
        frequency: 1.49563197134600e+02,
    },
    Vsop87Term {
        amplitude: 7.03284610000000e-04,
        phase: 5.39254450063000e+00,
        frequency: 6.37358983034000e+01,
    },
    Vsop87Term {
        amplitude: 6.88926780000000e-04,
        phase: 6.09292483287000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 6.19986150000000e-04,
        phase: 2.26952066061000e+00,
        frequency: 2.96894541660000e+00,
    },
    Vsop87Term {
        amplitude: 6.19507190000000e-04,
        phase: 2.85098872691000e+00,
        frequency: 1.10457002639000e+01,
    },
    Vsop87Term {
        amplitude: 2.64687700000000e-04,
        phase: 3.14152083966000e+00,
        frequency: 7.18126531507000e+01,
    },
    Vsop87Term {
        amplitude: 2.57104760000000e-04,
        phase: 6.11379840493000e+00,
        frequency: 4.54909366527300e+02,
    },
    Vsop87Term {
        amplitude: 2.10788500000000e-04,
        phase: 4.36059339067000e+00,
        frequency: 1.48078724426300e+02,
    },
    Vsop87Term {
        amplitude: 1.78186470000000e-04,
        phase: 1.74436930289000e+00,
        frequency: 3.66485629295000e+01,
    },
    Vsop87Term {
        amplitude: 1.46135070000000e-04,
        phase: 4.73732166022000e+00,
        frequency: 3.93215326310000e+00,
    },
    Vsop87Term {
        amplitude: 1.11625090000000e-04,
        phase: 5.82681796350000e+00,
        frequency: 2.24344795701900e+02,
    },
    Vsop87Term {
        amplitude: 1.09979100000000e-04,
        phase: 4.88650040180000e-01,
        frequency: 1.38517496870700e+02,
    },
    Vsop87Term {
        amplitude: 9.52747800000000e-05,
        phase: 2.95516862826000e+00,
        frequency: 3.51640902212000e+01,
    },
    Vsop87Term {
        amplitude: 7.54560100000000e-05,
        phase: 5.23626582400000e+00,
        frequency: 1.09945688788500e+02,
    },
    Vsop87Term {
        amplitude: 4.22024100000000e-05,
        phase: 3.23328220918000e+00,
        frequency: 7.08494453042000e+01,
    },
    Vsop87Term {
        amplitude: 4.05190000000000e-05,
        phase: 2.27755017300000e+00,
        frequency: 1.51047669842900e+02,
    },
];

const URANUS_L_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 7.47815986091000e+01,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.54332863000000e-03,
        phase: 5.24158770553000e+00,
        frequency: 7.47815985673000e+01,
    },
    Vsop87Term {
        amplitude: 2.44564740000000e-04,
        phase: 1.71260334156000e+00,
        frequency: 1.48447270830000e+00,
    },
    Vsop87Term {
        amplitude: 9.25844200000000e-05,
        phase: 4.28297323500000e-01,
        frequency: 1.10457002639000e+01,
    },
    Vsop87Term {
        amplitude: 8.26597700000000e-05,
        phase: 1.50218091379000e+00,
        frequency: 6.37358983034000e+01,
    },
    Vsop87Term {
        amplitude: 9.15016000000000e-05,
        phase: 1.41213765216000e+00,
        frequency: 1.49563197134600e+02,
    },
    Vsop87Term {
        amplitude: 3.89910800000000e-05,
        phase: 4.64835791600000e-01,
        frequency: 3.93215326310000e+00,
    },
    Vsop87Term {
        amplitude: 2.27706500000000e-05,
        phase: 4.17199181523000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 1.92747000000000e-05,
        phase: 5.29761884790000e-01,
        frequency: 2.96894541660000e+00,
    },
    Vsop87Term {
        amplitude: 1.23272500000000e-05,
        phase: 1.58632088145000e+00,
        frequency: 7.08494453042000e+01,
    },
    Vsop87Term {
        amplitude: 7.91201000000000e-06,
        phase: 5.43640595978000e+00,
        frequency: 3.18139373770000e+00,
    },
    Vsop87Term {
        amplitude: 7.66954000000000e-06,
        phase: 1.99425624214000e+00,
        frequency: 7.32971258590000e+01,
    },
    Vsop87Term {
        amplitude: 4.81813000000000e-06,
        phase: 2.98574070918000e+00,
        frequency: 8.58272988312000e+01,
    },
    Vsop87Term {
        amplitude: 4.49635000000000e-06,
        phase: 4.14242946378000e+00,
        frequency: 1.38517496870700e+02,
    },
    Vsop87Term {
        amplitude: 5.65091000000000e-06,
        phase: 3.87400932383000e+00,
        frequency: 2.24344795701900e+02,
    },
    Vsop87Term {
        amplitude: 4.26600000000000e-06,
        phase: 4.73158166033000e+00,
        frequency: 7.18126531507000e+01,
    },
    Vsop87Term {
        amplitude: 3.47745000000000e-06,
        phase: 2.45368882357000e+00,
        frequency: 9.56122755560000e+00,
    },
    Vsop87Term {
        amplitude: 3.32699000000000e-06,
        phase: 2.55525645638000e+00,
        frequency: 1.48078724426300e+02,
    },
    Vsop87Term {
        amplitude: 3.17054000000000e-06,
        phase: 5.57858240166000e+00,
        frequency: 5.26901980395000e+01,
    },
    Vsop87Term {
        amplitude: 1.79897000000000e-06,
        phase: 5.68365861477000e+00,
        frequency: 1.25301729722000e+01,
    },
];

const URANUS_L_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.34946900000000e-05,
        phase: 2.26708640433000e+00,
        frequency: 7.47815985673000e+01,
    },
    Vsop87Term {
        amplitude: 8.48806000000000e-06,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 7.68983000000000e-06,
        phase: 4.52562378749000e+00,
        frequency: 1.10457002639000e+01,
    },
    Vsop87Term {
        amplitude: 5.51555000000000e-06,
        phase: 3.25819322040000e+00,
        frequency: 6.37358983034000e+01,
    },
    Vsop87Term {
        amplitude: 5.41559000000000e-06,
        phase: 2.27572631399000e+00,
        frequency: 3.93215326310000e+00,
    },
    Vsop87Term {
        amplitude: 5.29491000000000e-06,
        phase: 4.92336172394000e+00,
        frequency: 1.48447270830000e+00,
    },
    Vsop87Term {
        amplitude: 2.57527000000000e-06,
        phase: 3.69060540044000e+00,
        frequency: 3.18139373770000e+00,
    },
    Vsop87Term {
        amplitude: 1.82036000000000e-06,
        phase: 6.21866555925000e+00,
        frequency: 7.08494453042000e+01,
    },
    Vsop87Term {
        amplitude: 1.84429000000000e-06,
        phase: 5.05954505833000e+00,
        frequency: 1.49563197134600e+02,
    },
    Vsop87Term {
        amplitude: 4.95050000000000e-07,
        phase: 6.03085160423000e+00,
        frequency: 5.66223513026000e+01,
    },
    Vsop87Term {
        amplitude: 5.34560000000000e-07,
        phase: 1.45801353517000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 3.83340000000000e-07,
        phase: 1.78433163102000e+00,
        frequency: 5.26901980395000e+01,
    },
    Vsop87Term {
        amplitude: 4.48850000000000e-07,
        phase: 3.90644983662000e+00,
        frequency: 2.44768055480000e+00,
    },
    Vsop87Term {
        amplitude: 4.46230000000000e-07,
        phase: 8.12325397610000e-01,
        frequency: 8.58272988312000e+01,
    },
    Vsop87Term {
        amplitude: 3.73730000000000e-07,
        phase: 4.46132739805000e+00,
        frequency: 2.96894541660000e+00,
    },
    Vsop87Term {
        amplitude: 3.30440000000000e-07,
        phase: 8.64619890310000e-01,
        frequency: 9.56122755560000e+00,
    },
    Vsop87Term {
        amplitude: 2.43050000000000e-07,
        phase: 2.10670976428000e+00,
        frequency: 1.81592472647000e+01,
    },
    Vsop87Term {
        amplitude: 2.92500000000000e-07,
        phase: 5.09724793503000e+00,
        frequency: 7.32971258590000e+01,
    },
    Vsop87Term {
        amplitude: 2.23090000000000e-07,
        phase: 4.81978108793000e+00,
        frequency: 7.87137518304000e+01,
    },
    Vsop87Term {
        amplitude: 2.22830000000000e-07,
        phase: 5.99230347559000e+00,
        frequency: 1.38517496870700e+02,
    },
];

const URANUS_L_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.22192000000000e-06,
        phase: 2.11210222500000e-02,
        frequency: 7.47815985673000e+01,
    },
    Vsop87Term {
        amplitude: 6.81950000000000e-07,
        phase: 4.12138633187000e+00,
        frequency: 3.93215326310000e+00,
    },
    Vsop87Term {
        amplitude: 5.27290000000000e-07,
        phase: 2.38808499397000e+00,
        frequency: 1.10457002639000e+01,
    },
    Vsop87Term {
        amplitude: 4.37140000000000e-07,
        phase: 2.95937380925000e+00,
        frequency: 1.48447270830000e+00,
    },
    Vsop87Term {
        amplitude: 4.54050000000000e-07,
        phase: 2.04405402149000e+00,
        frequency: 3.18139373770000e+00,
    },
    Vsop87Term {
        amplitude: 2.49030000000000e-07,
        phase: 4.88680075600000e+00,
        frequency: 6.37358983034000e+01,
    },
    Vsop87Term {
        amplitude: 2.10040000000000e-07,
        phase: 4.54879176205000e+00,
        frequency: 7.08494453042000e+01,
    },
    Vsop87Term {
        amplitude: 8.98500000000000e-08,
        phase: 1.58255257968000e+00,
        frequency: 5.66223513026000e+01,
    },
    Vsop87Term {
        amplitude: 9.15800000000000e-08,
        phase: 2.57000447334000e+00,
        frequency: 1.49563197134600e+02,
    },
    Vsop87Term {
        amplitude: 1.03610000000000e-07,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 4.26100000000000e-08,
        phase: 2.27802154660000e-01,
        frequency: 1.81592472647000e+01,
    },
    Vsop87Term {
        amplitude: 3.62500000000000e-08,
        phase: 5.38367304590000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 3.24400000000000e-08,
        phase: 5.01058611704000e+00,
        frequency: 8.58272988312000e+01,
    },
    Vsop87Term {
        amplitude: 3.48800000000000e-08,
        phase: 4.13160885916000e+00,
        frequency: 5.26901980395000e+01,
    },
    Vsop87Term {
        amplitude: 3.57000000000000e-08,
        phase: 9.40650812960000e-01,
        frequency: 7.79629923050000e+01,
    },
    Vsop87Term {
        amplitude: 2.73800000000000e-08,
        phase: 4.03465355400000e-01,
        frequency: 7.87137518304000e+01,
    },
    Vsop87Term {
        amplitude: 2.23300000000000e-08,
        phase: 8.71579876760000e-01,
        frequency: 1.45631043871500e+02,
    },
    Vsop87Term {
        amplitude: 1.94800000000000e-08,
        phase: 2.67957461817000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 2.12000000000000e-08,
        phase: 5.64073933192000e+00,
        frequency: 9.56122755560000e+00,
    },
    Vsop87Term {
        amplitude: 1.56600000000000e-08,
        phase: 5.46300116637000e+00,
        frequency: 7.32971258590000e+01,
    },
];

const URANUS_L_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 5.53600000000000e-08,
        phase: 4.57721551627000e+00,
        frequency: 7.47815985673000e+01,
    },
    Vsop87Term {
        amplitude: 3.18300000000000e-08,
        phase: 3.44674601710000e-01,
        frequency: 1.10457002639000e+01,
    },
    Vsop87Term {
        amplitude: 1.20700000000000e-08,
        phase: 3.40871377105000e+00,
        frequency: 5.66223513026000e+01,
    },
    Vsop87Term {
        amplitude: 6.34000000000000e-09,
        phase: 4.65445189526000e+00,
        frequency: 1.81592472647000e+01,
    },
    Vsop87Term {
        amplitude: 3.59000000000000e-09,
        phase: 6.70241568530000e-01,
        frequency: 1.49563197134600e+02,
    },
    Vsop87Term {
        amplitude: 2.47000000000000e-09,
        phase: 2.07784257495000e+00,
        frequency: 6.37358983034000e+01,
    },
    Vsop87Term {
        amplitude: 1.09000000000000e-09,
        phase: 2.75514337970000e+00,
        frequency: 8.58272988312000e+01,
    },
    Vsop87Term {
        amplitude: 9.20000000000000e-10,
        phase: 5.02598538441000e+00,
        frequency: 1.31403949869900e+02,
    },
];

const URANUS_L_5: &[Vsop87Term] = &[];

const URANUS_B_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.34627764800000e-02,
        phase: 2.61877810547000e+00,
        frequency: 7.47815985673000e+01,
    },
    Vsop87Term {
        amplitude: 6.23414000000000e-04,
        phase: 5.08111189648000e+00,
        frequency: 1.49563197134600e+02,
    },
    Vsop87Term {
        amplitude: 6.16011960000000e-04,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 9.96372200000000e-05,
        phase: 1.61603805646000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 9.92616000000000e-05,
        phase: 5.76303803330000e-01,
        frequency: 7.32971258590000e+01,
    },
    Vsop87Term {
        amplitude: 3.25946600000000e-05,
        phase: 1.26119342526000e+00,
        frequency: 2.24344795701900e+02,
    },
    Vsop87Term {
        amplitude: 2.97230300000000e-05,
        phase: 2.24367206357000e+00,
        frequency: 1.48447270830000e+00,
    },
    Vsop87Term {
        amplitude: 2.01027500000000e-05,
        phase: 6.05550884547000e+00,
        frequency: 1.48078724426300e+02,
    },
    Vsop87Term {
        amplitude: 1.52216300000000e-05,
        phase: 2.79596450020000e-01,
        frequency: 6.37358983034000e+01,
    },
    Vsop87Term {
        amplitude: 9.24064000000000e-06,
        phase: 4.03822512696000e+00,
        frequency: 1.51047669842900e+02,
    },
    Vsop87Term {
        amplitude: 7.60640000000000e-06,
        phase: 6.13999362624000e+00,
        frequency: 7.18126531507000e+01,
    },
    Vsop87Term {
        amplitude: 4.20265000000000e-06,
        phase: 5.21280055515000e+00,
        frequency: 1.10457002639000e+01,
    },
    Vsop87Term {
        amplitude: 4.30661000000000e-06,
        phase: 3.55443947716000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 4.36847000000000e-06,
        phase: 3.38081057022000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 5.22314000000000e-06,
        phase: 3.32086440954000e+00,
        frequency: 1.38517496870700e+02,
    },
    Vsop87Term {
        amplitude: 4.34627000000000e-06,
        phase: 3.40631997630000e-01,
        frequency: 7.77505439839000e+01,
    },
    Vsop87Term {
        amplitude: 4.62630000000000e-06,
        phase: 7.42566876060000e-01,
        frequency: 8.58272988312000e+01,
    },
    Vsop87Term {
        amplitude: 2.32667000000000e-06,
        phase: 2.25715668168000e+00,
        frequency: 2.22860322993600e+02,
    },
    Vsop87Term {
        amplitude: 2.15848000000000e-06,
        phase: 1.59122810633000e+00,
        frequency: 3.81330356378000e+01,
    },
    Vsop87Term {
        amplitude: 2.44698000000000e-06,
        phase: 7.87951741000000e-01,
        frequency: 2.96894541660000e+00,
    },
];

const URANUS_B_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 3.41019780000000e-04,
        phase: 1.32192993600000e-02,
        frequency: 7.47815985673000e+01,
    },
    Vsop87Term {
        amplitude: 2.48011500000000e-05,
        phase: 2.73961370453000e+00,
        frequency: 1.49563197134600e+02,
    },
    Vsop87Term {
        amplitude: 1.71937700000000e-05,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 3.95276000000000e-06,
        phase: 5.49322816551000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 3.08903000000000e-06,
        phase: 3.61139770633000e+00,
        frequency: 7.32971258590000e+01,
    },
    Vsop87Term {
        amplitude: 1.81125000000000e-06,
        phase: 5.32079457105000e+00,
        frequency: 2.24344795701900e+02,
    },
    Vsop87Term {
        amplitude: 1.44520000000000e-06,
        phase: 4.22110521671000e+00,
        frequency: 6.37358983034000e+01,
    },
    Vsop87Term {
        amplitude: 7.63430000000000e-07,
        phase: 4.54620999213000e+00,
        frequency: 8.58272988312000e+01,
    },
    Vsop87Term {
        amplitude: 7.26330000000000e-07,
        phase: 5.97811706013000e+00,
        frequency: 1.48447270830000e+00,
    },
    Vsop87Term {
        amplitude: 6.54920000000000e-07,
        phase: 2.77607065171000e+00,
        frequency: 1.10457002639000e+01,
    },
    Vsop87Term {
        amplitude: 6.39310000000000e-07,
        phase: 6.15917217447000e+00,
        frequency: 1.38517496870700e+02,
    },
    Vsop87Term {
        amplitude: 5.09720000000000e-07,
        phase: 1.79457572126000e+00,
        frequency: 1.51047669842900e+02,
    },
    Vsop87Term {
        amplitude: 3.99300000000000e-07,
        phase: 3.59559614775000e+00,
        frequency: 1.48078724426300e+02,
    },
    Vsop87Term {
        amplitude: 3.66670000000000e-07,
        phase: 3.82753352893000e+00,
        frequency: 7.08494453042000e+01,
    },
    Vsop87Term {
        amplitude: 2.69690000000000e-07,
        phase: 4.71074996908000e+00,
        frequency: 7.87137518304000e+01,
    },
    Vsop87Term {
        amplitude: 2.72050000000000e-07,
        phase: 4.22769491494000e+00,
        frequency: 7.18126531507000e+01,
    },
    Vsop87Term {
        amplitude: 2.20740000000000e-07,
        phase: 4.76357435668000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 2.26550000000000e-07,
        phase: 4.40615405121000e+00,
        frequency: 7.77505439839000e+01,
    },
    Vsop87Term {
        amplitude: 1.57200000000000e-07,
        phase: 1.55930265947000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 1.00900000000000e-07,
        phase: 5.83224201984000e+00,
        frequency: 1.45631043871500e+02,
    },
];

const URANUS_B_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 7.64663000000000e-06,
        phase: 1.74870957857000e+00,
        frequency: 7.47815985673000e+01,
    },
    Vsop87Term {
        amplitude: 5.57340000000000e-07,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 2.56410000000000e-07,
        phase: 5.67301557131000e+00,
        frequency: 1.49563197134600e+02,
    },
    Vsop87Term {
        amplitude: 1.33350000000000e-07,
        phase: 5.92348443969000e+00,
        frequency: 7.32971258590000e+01,
    },
    Vsop87Term {
        amplitude: 6.63600000000000e-08,
        phase: 2.30241577514000e+00,
        frequency: 8.58272988312000e+01,
    },
    Vsop87Term {
        amplitude: 4.92600000000000e-08,
        phase: 2.21241492976000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 4.36800000000000e-08,
        phase: 7.66494935060000e-01,
        frequency: 1.10457002639000e+01,
    },
    Vsop87Term {
        amplitude: 4.09500000000000e-08,
        phase: 1.81604424547000e+00,
        frequency: 7.08494453042000e+01,
    },
    Vsop87Term {
        amplitude: 3.55000000000000e-08,
        phase: 2.72620892642000e+00,
        frequency: 2.24344795701900e+02,
    },
    Vsop87Term {
        amplitude: 3.55600000000000e-08,
        phase: 3.68989806020000e-01,
        frequency: 7.87137518304000e+01,
    },
    Vsop87Term {
        amplitude: 3.79900000000000e-08,
        phase: 1.75732801545000e+00,
        frequency: 1.38517496870700e+02,
    },
    Vsop87Term {
        amplitude: 1.82000000000000e-08,
        phase: 1.54477121376000e+00,
        frequency: 7.79629923050000e+01,
    },
    Vsop87Term {
        amplitude: 1.65100000000000e-08,
        phase: 1.41591379356000e+00,
        frequency: 1.48447270830000e+00,
    },
    Vsop87Term {
        amplitude: 1.60800000000000e-08,
        phase: 6.22512841748000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 1.45200000000000e-08,
        phase: 3.90164387464000e+00,
        frequency: 1.45631043871500e+02,
    },
    Vsop87Term {
        amplitude: 1.24500000000000e-08,
        phase: 3.04960471697000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 1.68900000000000e-08,
        phase: 5.74226020410000e-01,
        frequency: 7.16002048296000e+01,
    },
    Vsop87Term {
        amplitude: 1.08200000000000e-08,
        phase: 5.44260490226000e+00,
        frequency: 1.51047669842900e+02,
    },
    Vsop87Term {
        amplitude: 1.03300000000000e-08,
        phase: 5.50906270157000e+00,
        frequency: 6.37358983034000e+01,
    },
    Vsop87Term {
        amplitude: 9.79000000000000e-09,
        phase: 4.45089803473000e+00,
        frequency: 3.93215326310000e+00,
    },
];

const URANUS_B_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.12010000000000e-07,
        phase: 3.16540759295000e+00,
        frequency: 7.47815985673000e+01,
    },
    Vsop87Term {
        amplitude: 1.18200000000000e-08,
        phase: 4.44441014271000e+00,
        frequency: 1.49563197134600e+02,
    },
    Vsop87Term {
        amplitude: 1.18400000000000e-08,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 4.33000000000000e-09,
        phase: 1.10371780340000e+00,
        frequency: 7.32971258590000e+01,
    },
    Vsop87Term {
        amplitude: 4.12000000000000e-09,
        phase: 4.39846579460000e-01,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 4.66000000000000e-09,
        phase: 5.92951996029000e+00,
        frequency: 7.08494453042000e+01,
    },
    Vsop87Term {
        amplitude: 4.64000000000000e-09,
        phase: 1.88752032733000e+00,
        frequency: 7.87137518304000e+01,
    },
    Vsop87Term {
        amplitude: 4.04000000000000e-09,
        phase: 6.16046303283000e+00,
        frequency: 8.58272988312000e+01,
    },
    Vsop87Term {
        amplitude: 4.01000000000000e-09,
        phase: 1.68921956710000e+00,
        frequency: 6.37358983034000e+01,
    },
    Vsop87Term {
        amplitude: 2.95000000000000e-09,
        phase: 5.96253146711000e+00,
        frequency: 7.79629923050000e+01,
    },
    Vsop87Term {
        amplitude: 2.94000000000000e-09,
        phase: 1.85767623542000e+00,
        frequency: 7.16002048296000e+01,
    },
    Vsop87Term {
        amplitude: 1.74000000000000e-09,
        phase: 4.79023778832000e+00,
        frequency: 1.10457002639000e+01,
    },
    Vsop87Term {
        amplitude: 9.90000000000000e-10,
        phase: 4.22830061350000e-01,
        frequency: 2.24344795701900e+02,
    },
    Vsop87Term {
        amplitude: 8.80000000000000e-10,
        phase: 2.27837607751000e+00,
        frequency: 1.45631043871500e+02,
    },
    Vsop87Term {
        amplitude: 7.90000000000000e-10,
        phase: 3.66485269931000e+00,
        frequency: 1.38517496870700e+02,
    },
];

const URANUS_B_4: &[Vsop87Term] = &[];

const URANUS_B_5: &[Vsop87Term] = &[];

const URANUS_R_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.92126484720600e+01,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 8.87849844130000e-01,
        phase: 5.60377527014000e+00,
        frequency: 7.47815985673000e+01,
    },
    Vsop87Term {
        amplitude: 3.44083606200000e-02,
        phase: 3.28360997060000e-01,
        frequency: 7.32971258590000e+01,
    },
    Vsop87Term {
        amplitude: 2.05565386000000e-02,
        phase: 1.78295159330000e+00,
        frequency: 1.49563197134600e+02,
    },
    Vsop87Term {
        amplitude: 6.49322410000000e-03,
        phase: 4.52247285911000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 6.02247865000000e-03,
        phase: 3.86003823674000e+00,
        frequency: 6.37358983034000e+01,
    },
    Vsop87Term {
        amplitude: 4.96404167000000e-03,
        phase: 1.40139935333000e+00,
        frequency: 4.54909366527300e+02,
    },
    Vsop87Term {
        amplitude: 3.38525369000000e-03,
        phase: 1.58002770318000e+00,
        frequency: 1.38517496870700e+02,
    },
    Vsop87Term {
        amplitude: 2.43509114000000e-03,
        phase: 1.57086606044000e+00,
        frequency: 7.18126531507000e+01,
    },
    Vsop87Term {
        amplitude: 1.90522303000000e-03,
        phase: 1.99809394714000e+00,
        frequency: 1.48447270830000e+00,
    },
    Vsop87Term {
        amplitude: 1.61858838000000e-03,
        phase: 2.79137786799000e+00,
        frequency: 1.48078724426300e+02,
    },
    Vsop87Term {
        amplitude: 1.43706183000000e-03,
        phase: 1.38368544947000e+00,
        frequency: 1.10457002639000e+01,
    },
    Vsop87Term {
        amplitude: 9.31924050000000e-04,
        phase: 1.74372204670000e-01,
        frequency: 3.66485629295000e+01,
    },
    Vsop87Term {
        amplitude: 7.14245480000000e-04,
        phase: 4.24509236074000e+00,
        frequency: 2.24344795701900e+02,
    },
    Vsop87Term {
        amplitude: 8.98060140000000e-04,
        phase: 3.66105364565000e+00,
        frequency: 1.09945688788500e+02,
    },
    Vsop87Term {
        amplitude: 3.90097230000000e-04,
        phase: 1.66971401684000e+00,
        frequency: 7.08494453042000e+01,
    },
    Vsop87Term {
        amplitude: 4.66772960000000e-04,
        phase: 1.39976401694000e+00,
        frequency: 3.51640902212000e+01,
    },
    Vsop87Term {
        amplitude: 3.90256240000000e-04,
        phase: 3.36234773834000e+00,
        frequency: 2.77034993741400e+02,
    },
    Vsop87Term {
        amplitude: 3.67552740000000e-04,
        phase: 3.88649278513000e+00,
        frequency: 1.46594251718000e+02,
    },
    Vsop87Term {
        amplitude: 3.03487230000000e-04,
        phase: 7.01008387980000e-01,
        frequency: 1.51047669842900e+02,
    },
];

const URANUS_R_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.47989662900000e-02,
        phase: 3.67205697578000e+00,
        frequency: 7.47815985673000e+01,
    },
    Vsop87Term {
        amplitude: 7.12121430000000e-04,
        phase: 6.22600975161000e+00,
        frequency: 6.37358983034000e+01,
    },
    Vsop87Term {
        amplitude: 6.86271600000000e-04,
        phase: 6.13411179902000e+00,
        frequency: 1.49563197134600e+02,
    },
    Vsop87Term {
        amplitude: 2.08575540000000e-04,
        phase: 5.24625848960000e+00,
        frequency: 1.10457002639000e+01,
    },
    Vsop87Term {
        amplitude: 2.14683620000000e-04,
        phase: 2.60175716374000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 2.40593690000000e-04,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.14050560000000e-04,
        phase: 1.84973801700000e-02,
        frequency: 7.08494453042000e+01,
    },
    Vsop87Term {
        amplitude: 7.49679700000000e-05,
        phase: 4.23613559550000e-01,
        frequency: 7.32971258590000e+01,
    },
    Vsop87Term {
        amplitude: 4.24360600000000e-05,
        phase: 1.41691058162000e+00,
        frequency: 8.58272988312000e+01,
    },
    Vsop87Term {
        amplitude: 3.50595100000000e-05,
        phase: 2.58348117401000e+00,
        frequency: 1.38517496870700e+02,
    },
    Vsop87Term {
        amplitude: 3.22880000000000e-05,
        phase: 5.25495561645000e+00,
        frequency: 3.93215326310000e+00,
    },
    Vsop87Term {
        amplitude: 3.92683300000000e-05,
        phase: 3.15526349399000e+00,
        frequency: 7.18126531507000e+01,
    },
    Vsop87Term {
        amplitude: 3.05989900000000e-05,
        phase: 1.53238421120000e-01,
        frequency: 1.48447270830000e+00,
    },
    Vsop87Term {
        amplitude: 3.57825400000000e-05,
        phase: 2.31157935775000e+00,
        frequency: 2.24344795701900e+02,
    },
    Vsop87Term {
        amplitude: 2.56423500000000e-05,
        phase: 9.80785491080000e-01,
        frequency: 1.48078724426300e+02,
    },
    Vsop87Term {
        amplitude: 2.42919100000000e-05,
        phase: 3.99450740432000e+00,
        frequency: 5.26901980395000e+01,
    },
    Vsop87Term {
        amplitude: 1.64483000000000e-05,
        phase: 2.65310351864000e+00,
        frequency: 1.27471796606800e+02,
    },
    Vsop87Term {
        amplitude: 1.58356900000000e-05,
        phase: 1.43049534360000e+00,
        frequency: 7.87137518304000e+01,
    },
    Vsop87Term {
        amplitude: 1.41338000000000e-05,
        phase: 4.57461623347000e+00,
        frequency: 2.02253395174100e+02,
    },
    Vsop87Term {
        amplitude: 1.48972400000000e-05,
        phase: 2.67568435302000e+00,
        frequency: 5.66223513026000e+01,
    },
];

const URANUS_R_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.24398990000000e-04,
        phase: 6.99533109030000e-01,
        frequency: 7.47815985673000e+01,
    },
    Vsop87Term {
        amplitude: 4.72683800000000e-05,
        phase: 1.69896897296000e+00,
        frequency: 6.37358983034000e+01,
    },
    Vsop87Term {
        amplitude: 1.68138300000000e-05,
        phase: 4.64842242588000e+00,
        frequency: 7.08494453042000e+01,
    },
    Vsop87Term {
        amplitude: 1.43363300000000e-05,
        phase: 3.52135281258000e+00,
        frequency: 1.49563197134600e+02,
    },
    Vsop87Term {
        amplitude: 1.64947700000000e-05,
        phase: 3.09669484042000e+00,
        frequency: 1.10457002639000e+01,
    },
    Vsop87Term {
        amplitude: 7.69974000000000e-06,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 4.61159000000000e-06,
        phase: 7.66671856720000e-01,
        frequency: 3.93215326310000e+00,
    },
    Vsop87Term {
        amplitude: 5.00193000000000e-06,
        phase: 6.17218448634000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 3.90377000000000e-06,
        phase: 4.49603136758000e+00,
        frequency: 5.66223513026000e+01,
    },
    Vsop87Term {
        amplitude: 3.89972000000000e-06,
        phase: 5.52663268311000e+00,
        frequency: 8.58272988312000e+01,
    },
    Vsop87Term {
        amplitude: 2.92283000000000e-06,
        phase: 2.03708206680000e-01,
        frequency: 5.26901980395000e+01,
    },
    Vsop87Term {
        amplitude: 2.72269000000000e-06,
        phase: 3.84735375210000e+00,
        frequency: 1.38517496870700e+02,
    },
    Vsop87Term {
        amplitude: 2.86451000000000e-06,
        phase: 3.53449822561000e+00,
        frequency: 7.32971258590000e+01,
    },
    Vsop87Term {
        amplitude: 2.05341000000000e-06,
        phase: 3.24759155116000e+00,
        frequency: 7.87137518304000e+01,
    },
    Vsop87Term {
        amplitude: 2.19349000000000e-06,
        phase: 1.96433948894000e+00,
        frequency: 1.31403949869900e+02,
    },
    Vsop87Term {
        amplitude: 2.15812000000000e-06,
        phase: 8.48209224530000e-01,
        frequency: 7.79629923050000e+01,
    },
    Vsop87Term {
        amplitude: 1.29040000000000e-06,
        phase: 2.08142441038000e+00,
        frequency: 3.18139373770000e+00,
    },
    Vsop87Term {
        amplitude: 1.48716000000000e-06,
        phase: 4.89757177249000e+00,
        frequency: 1.27471796606800e+02,
    },
    Vsop87Term {
        amplitude: 1.17642000000000e-06,
        phase: 4.93417950365000e+00,
        frequency: 4.47795819526500e+02,
    },
    Vsop87Term {
        amplitude: 1.12873000000000e-06,
        phase: 1.01358614296000e+00,
        frequency: 4.62022913528100e+02,
    },
];

const URANUS_R_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.16466300000000e-05,
        phase: 4.73440180792000e+00,
        frequency: 7.47815985673000e+01,
    },
    Vsop87Term {
        amplitude: 2.12363000000000e-06,
        phase: 3.34268349684000e+00,
        frequency: 6.37358983034000e+01,
    },
    Vsop87Term {
        amplitude: 1.96315000000000e-06,
        phase: 2.98101237100000e+00,
        frequency: 7.08494453042000e+01,
    },
    Vsop87Term {
        amplitude: 1.04707000000000e-06,
        phase: 9.57892795550000e-01,
        frequency: 1.10457002639000e+01,
    },
    Vsop87Term {
        amplitude: 7.16810000000000e-07,
        phase: 2.52829507100000e-02,
        frequency: 5.66223513026000e+01,
    },
    Vsop87Term {
        amplitude: 7.27190000000000e-07,
        phase: 9.94798310410000e-01,
        frequency: 1.49563197134600e+02,
    },
    Vsop87Term {
        amplitude: 5.49330000000000e-07,
        phase: 2.59936585639000e+00,
        frequency: 3.93215326310000e+00,
    },
    Vsop87Term {
        amplitude: 3.40260000000000e-07,
        phase: 3.82319495878000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 3.20810000000000e-07,
        phase: 3.59825177872000e+00,
        frequency: 1.31403949869900e+02,
    },
    Vsop87Term {
        amplitude: 2.95690000000000e-07,
        phase: 3.44303690664000e+00,
        frequency: 8.58272988312000e+01,
    },
    Vsop87Term {
        amplitude: 3.63770000000000e-07,
        phase: 5.65035573026000e+00,
        frequency: 7.79629923050000e+01,
    },
    Vsop87Term {
        amplitude: 2.76250000000000e-07,
        phase: 4.28854773770000e-01,
        frequency: 3.18139373770000e+00,
    },
    Vsop87Term {
        amplitude: 2.75520000000000e-07,
        phase: 2.55709855563000e+00,
        frequency: 5.26901980395000e+01,
    },
    Vsop87Term {
        amplitude: 2.47400000000000e-07,
        phase: 5.14634979896000e+00,
        frequency: 7.87137518304000e+01,
    },
    Vsop87Term {
        amplitude: 1.93820000000000e-07,
        phase: 5.13444064222000e+00,
        frequency: 1.81592472647000e+01,
    },
    Vsop87Term {
        amplitude: 1.57670000000000e-07,
        phase: 3.71169517430000e-01,
        frequency: 4.47795819526500e+02,
    },
    Vsop87Term {
        amplitude: 1.54410000000000e-07,
        phase: 5.57271837433000e+00,
        frequency: 4.62022913528100e+02,
    },
    Vsop87Term {
        amplitude: 1.50350000000000e-07,
        phase: 3.84415419523000e+00,
        frequency: 7.32971258590000e+01,
    },
    Vsop87Term {
        amplitude: 1.54500000000000e-07,
        phase: 2.97572514360000e+00,
        frequency: 1.45631043871500e+02,
    },
    Vsop87Term {
        amplitude: 1.77880000000000e-07,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
];

const URANUS_R_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 5.32240000000000e-07,
        phase: 3.00468894529000e+00,
        frequency: 7.47815985673000e+01,
    },
    Vsop87Term {
        amplitude: 9.88700000000000e-08,
        phase: 1.91399083603000e+00,
        frequency: 5.66223513026000e+01,
    },
    Vsop87Term {
        amplitude: 7.00800000000000e-08,
        phase: 5.08677527404000e+00,
        frequency: 1.10457002639000e+01,
    },
    Vsop87Term {
        amplitude: 6.71800000000000e-08,
        phase: 5.39509675772000e+00,
        frequency: 1.49563197134600e+02,
    },
    Vsop87Term {
        amplitude: 3.85500000000000e-08,
        phase: 5.18994119112000e+00,
        frequency: 1.31403949869900e+02,
    },
    Vsop87Term {
        amplitude: 3.31600000000000e-08,
        phase: 1.22839100759000e+00,
        frequency: 8.58272988312000e+01,
    },
    Vsop87Term {
        amplitude: 2.66400000000000e-08,
        phase: 4.40645778370000e-01,
        frequency: 6.37358983034000e+01,
    },
    Vsop87Term {
        amplitude: 2.30900000000000e-08,
        phase: 9.23807209340000e-01,
        frequency: 1.45631043871500e+02,
    },
    Vsop87Term {
        amplitude: 2.38300000000000e-08,
        phase: 6.21390585593000e+00,
        frequency: 3.58930139309500e+02,
    },
    Vsop87Term {
        amplitude: 2.28800000000000e-08,
        phase: 2.23425399117000e+00,
        frequency: 4.40682272525700e+02,
    },
    Vsop87Term {
        amplitude: 2.47200000000000e-08,
        phase: 3.28269448244000e+00,
        frequency: 1.81592472647000e+01,
    },
    Vsop87Term {
        amplitude: 2.83700000000000e-08,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
];

const URANUS_R_5: &[Vsop87Term] = &[];
