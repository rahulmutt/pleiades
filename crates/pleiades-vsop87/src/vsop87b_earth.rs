//! Truncated VSOP87B Earth coefficient tables.
//!
//! Coefficients are the leading terms from the public IMCCE/CELMECH
//! `VSOP87B.ear` file (Earth heliocentric spherical variables, J2000
//! ecliptic/equinox). The full source file is not yet vendored; this module
//! intentionally carries a small first slice so the production coefficient
//! evaluation path is tested before adding generated complete tables.

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

pub(crate) fn earth_lbr(julian_day_tt: f64) -> SphericalLbr {
    // VSOP87 uses Julian millennia from J2000.0.
    let t = (julian_day_tt - 2_451_545.0) / 365_250.0;
    SphericalLbr {
        longitude_rad: evaluate(
            &[
                EARTH_L_0, EARTH_L_1, EARTH_L_2, EARTH_L_3, EARTH_L_4, EARTH_L_5,
            ],
            t,
        )
        .rem_euclid(core::f64::consts::TAU),
        latitude_rad: evaluate(
            &[
                EARTH_B_0, EARTH_B_1, EARTH_B_2, EARTH_B_3, EARTH_B_4, EARTH_B_5,
            ],
            t,
        ),
        radius_au: evaluate(
            &[
                EARTH_R_0, EARTH_R_1, EARTH_R_2, EARTH_R_3, EARTH_R_4, EARTH_R_5,
            ],
            t,
        ),
    }
}

pub(crate) fn evaluate(series_by_power: &[&[Vsop87Term]], t: f64) -> f64 {
    let mut t_power = 1.0;
    let mut value = 0.0;
    for terms in series_by_power {
        let subtotal: f64 = terms
            .iter()
            .map(|term| term.amplitude * (term.phase + term.frequency * t).cos())
            .sum();
        value += subtotal * t_power;
        t_power *= t;
    }
    value
}

const EARTH_L_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.75347045673000e+00,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 3.34165645300000e-02,
        phase: 4.66925680415000e+00,
        frequency: 6.28307584999140e+03,
    },
    Vsop87Term {
        amplitude: 3.48942750000000e-04,
        phase: 4.62610242189000e+00,
        frequency: 1.25661516999828e+04,
    },
    Vsop87Term {
        amplitude: 3.41757200000000e-05,
        phase: 2.82886579754000e+00,
        frequency: 3.52311834900000e+00,
    },
    Vsop87Term {
        amplitude: 3.49705600000000e-05,
        phase: 2.74411783405000e+00,
        frequency: 5.75338488489680e+03,
    },
    Vsop87Term {
        amplitude: 3.13589900000000e-05,
        phase: 3.62767041756000e+00,
        frequency: 7.77137714681205e+04,
    },
    Vsop87Term {
        amplitude: 2.67621800000000e-05,
        phase: 4.41808345438000e+00,
        frequency: 7.86041939243920e+03,
    },
    Vsop87Term {
        amplitude: 2.34269100000000e-05,
        phase: 6.13516214446000e+00,
        frequency: 3.93020969621960e+03,
    },
    Vsop87Term {
        amplitude: 1.27316500000000e-05,
        phase: 2.03709657878000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 1.32429400000000e-05,
        phase: 7.42463416730000e-01,
        frequency: 1.15067697697936e+04,
    },
    Vsop87Term {
        amplitude: 9.01854000000000e-06,
        phase: 2.04505446477000e+00,
        frequency: 2.62983197998000e+01,
    },
    Vsop87Term {
        amplitude: 1.19916700000000e-05,
        phase: 1.10962946234000e+00,
        frequency: 1.57734354244780e+03,
    },
    Vsop87Term {
        amplitude: 8.57223000000000e-06,
        phase: 3.50849152283000e+00,
        frequency: 3.98149003408200e+02,
    },
    Vsop87Term {
        amplitude: 7.79786000000000e-06,
        phase: 1.17882681962000e+00,
        frequency: 5.22369391980220e+03,
    },
    Vsop87Term {
        amplitude: 9.90250000000000e-06,
        phase: 5.23268072088000e+00,
        frequency: 5.88492684658320e+03,
    },
    Vsop87Term {
        amplitude: 7.53141000000000e-06,
        phase: 2.53339052847000e+00,
        frequency: 5.50755323866740e+03,
    },
    Vsop87Term {
        amplitude: 5.05267000000000e-06,
        phase: 4.58292599973000e+00,
        frequency: 1.88492275499742e+04,
    },
    Vsop87Term {
        amplitude: 4.92392000000000e-06,
        phase: 4.20505711826000e+00,
        frequency: 7.75522611324000e+02,
    },
    Vsop87Term {
        amplitude: 3.56672000000000e-06,
        phase: 2.91954114478000e+00,
        frequency: 6.73103028000000e-02,
    },
    Vsop87Term {
        amplitude: 2.84125000000000e-06,
        phase: 1.89869240932000e+00,
        frequency: 7.96298006816400e+02,
    },
];

const EARTH_L_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 6.28307584999140e+03,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 2.06058863000000e-03,
        phase: 2.67823455808000e+00,
        frequency: 6.28307584999140e+03,
    },
    Vsop87Term {
        amplitude: 4.30341900000000e-05,
        phase: 2.63512233481000e+00,
        frequency: 1.25661516999828e+04,
    },
    Vsop87Term {
        amplitude: 4.25264000000000e-06,
        phase: 1.59046982018000e+00,
        frequency: 3.52311834900000e+00,
    },
    Vsop87Term {
        amplitude: 1.09017000000000e-06,
        phase: 2.96631010675000e+00,
        frequency: 1.57734354244780e+03,
    },
    Vsop87Term {
        amplitude: 9.34790000000000e-07,
        phase: 2.59211109542000e+00,
        frequency: 1.88492275499742e+04,
    },
    Vsop87Term {
        amplitude: 1.19305000000000e-06,
        phase: 5.79555765566000e+00,
        frequency: 2.62983197998000e+01,
    },
    Vsop87Term {
        amplitude: 7.21210000000000e-07,
        phase: 1.13840581212000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 6.77840000000000e-07,
        phase: 1.87453300345000e+00,
        frequency: 3.98149003408200e+02,
    },
    Vsop87Term {
        amplitude: 6.73500000000000e-07,
        phase: 4.40932832004000e+00,
        frequency: 5.50755323866740e+03,
    },
    Vsop87Term {
        amplitude: 5.90450000000000e-07,
        phase: 2.88815790631000e+00,
        frequency: 5.22369391980220e+03,
    },
    Vsop87Term {
        amplitude: 5.59760000000000e-07,
        phase: 2.17471740035000e+00,
        frequency: 1.55420399434200e+02,
    },
    Vsop87Term {
        amplitude: 4.54110000000000e-07,
        phase: 3.97995028960000e-01,
        frequency: 7.96298006816400e+02,
    },
    Vsop87Term {
        amplitude: 3.62980000000000e-07,
        phase: 4.68754372270000e-01,
        frequency: 7.75522611324000e+02,
    },
    Vsop87Term {
        amplitude: 2.89620000000000e-07,
        phase: 2.64732254645000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 1.90970000000000e-07,
        phase: 1.84628376049000e+00,
        frequency: 5.48677784317500e+03,
    },
    Vsop87Term {
        amplitude: 2.08440000000000e-07,
        phase: 5.34138275149000e+00,
        frequency: 9.80321068200000e-01,
    },
    Vsop87Term {
        amplitude: 1.85080000000000e-07,
        phase: 4.96855179468000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 1.62330000000000e-07,
        phase: 3.21658731500000e-02,
        frequency: 2.54431441988340e+03,
    },
    Vsop87Term {
        amplitude: 1.72930000000000e-07,
        phase: 2.99116760630000e+00,
        frequency: 6.27596230299060e+03,
    },
];

const EARTH_L_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 8.72185900000000e-05,
        phase: 1.07253635559000e+00,
        frequency: 6.28307584999140e+03,
    },
    Vsop87Term {
        amplitude: 9.90990000000000e-06,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 2.94833000000000e-06,
        phase: 4.37173502560000e-01,
        frequency: 1.25661516999828e+04,
    },
    Vsop87Term {
        amplitude: 2.73380000000000e-07,
        phase: 5.29563614700000e-02,
        frequency: 3.52311834900000e+00,
    },
    Vsop87Term {
        amplitude: 1.63330000000000e-07,
        phase: 5.18820215724000e+00,
        frequency: 2.62983197998000e+01,
    },
    Vsop87Term {
        amplitude: 1.57450000000000e-07,
        phase: 3.68504712183000e+00,
        frequency: 1.55420399434200e+02,
    },
    Vsop87Term {
        amplitude: 9.42500000000000e-08,
        phase: 2.96671146940000e-01,
        frequency: 1.88492275499742e+04,
    },
    Vsop87Term {
        amplitude: 8.93800000000000e-08,
        phase: 2.05706319592000e+00,
        frequency: 7.77137714681205e+04,
    },
    Vsop87Term {
        amplitude: 6.94000000000000e-08,
        phase: 8.26915410380000e-01,
        frequency: 7.75522611324000e+02,
    },
    Vsop87Term {
        amplitude: 5.06100000000000e-08,
        phase: 4.66243231680000e+00,
        frequency: 1.57734354244780e+03,
    },
    Vsop87Term {
        amplitude: 4.06000000000000e-08,
        phase: 1.03067032318000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 3.46400000000000e-08,
        phase: 5.14021224609000e+00,
        frequency: 7.96298006816400e+02,
    },
    Vsop87Term {
        amplitude: 3.17200000000000e-08,
        phase: 6.05479318507000e+00,
        frequency: 5.50755323866740e+03,
    },
    Vsop87Term {
        amplitude: 3.02000000000000e-08,
        phase: 1.19240008524000e+00,
        frequency: 2.42728603974000e+02,
    },
    Vsop87Term {
        amplitude: 2.88500000000000e-08,
        phase: 6.11705865396000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 3.80900000000000e-08,
        phase: 3.44043369494000e+00,
        frequency: 5.57314280143310e+03,
    },
    Vsop87Term {
        amplitude: 2.71900000000000e-08,
        phase: 3.03632481640000e-01,
        frequency: 3.98149003408200e+02,
    },
    Vsop87Term {
        amplitude: 2.36500000000000e-08,
        phase: 4.37666117992000e+00,
        frequency: 5.22369391980220e+03,
    },
    Vsop87Term {
        amplitude: 2.53800000000000e-08,
        phase: 2.27966434314000e+00,
        frequency: 5.53569402842400e+02,
    },
    Vsop87Term {
        amplitude: 2.07800000000000e-08,
        phase: 3.75435095487000e+00,
        frequency: 9.80321068200000e-01,
    },
];

const EARTH_L_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.89058000000000e-06,
        phase: 5.84173149732000e+00,
        frequency: 6.28307584999140e+03,
    },
    Vsop87Term {
        amplitude: 2.07120000000000e-07,
        phase: 6.04983939020000e+00,
        frequency: 1.25661516999828e+04,
    },
    Vsop87Term {
        amplitude: 2.96200000000000e-08,
        phase: 5.19560579570000e+00,
        frequency: 1.55420399434200e+02,
    },
    Vsop87Term {
        amplitude: 2.52700000000000e-08,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.28800000000000e-08,
        phase: 4.72197611970000e+00,
        frequency: 3.52311834900000e+00,
    },
    Vsop87Term {
        amplitude: 6.35000000000000e-09,
        phase: 5.96904899168000e+00,
        frequency: 2.42728603974000e+02,
    },
    Vsop87Term {
        amplitude: 5.70000000000000e-09,
        phase: 5.54182903238000e+00,
        frequency: 1.88492275499742e+04,
    },
    Vsop87Term {
        amplitude: 4.02000000000000e-09,
        phase: 3.78606612895000e+00,
        frequency: 5.53569402842400e+02,
    },
    Vsop87Term {
        amplitude: 7.20000000000000e-10,
        phase: 4.37131884946000e+00,
        frequency: 6.28659896834040e+03,
    },
    Vsop87Term {
        amplitude: 6.70000000000000e-10,
        phase: 9.11338989670000e-01,
        frequency: 6.12765545055720e+03,
    },
    Vsop87Term {
        amplitude: 3.70000000000000e-10,
        phase: 5.28611190997000e+00,
        frequency: 6.43849624942560e+03,
    },
    Vsop87Term {
        amplitude: 2.10000000000000e-10,
        phase: 2.94917211527000e+00,
        frequency: 6.30937416979120e+03,
    },
    Vsop87Term {
        amplitude: 1.50000000000000e-10,
        phase: 3.63037493932000e+00,
        frequency: 7.14306956181291e+04,
    },
    Vsop87Term {
        amplitude: 1.10000000000000e-10,
        phase: 4.83261533939000e+00,
        frequency: 2.51323033999656e+04,
    },
    Vsop87Term {
        amplitude: 1.10000000000000e-10,
        phase: 5.84259014283000e+00,
        frequency: 6.52580445396540e+03,
    },
    Vsop87Term {
        amplitude: 1.20000000000000e-10,
        phase: 3.82296977522000e+00,
        frequency: 7.05859846131540e+03,
    },
    Vsop87Term {
        amplitude: 1.30000000000000e-10,
        phase: 2.39991715131000e+00,
        frequency: 5.72950644714900e+03,
    },
    Vsop87Term {
        amplitude: 8.00000000000000e-11,
        phase: 5.53903320940000e-01,
        frequency: 6.04034724601740e+03,
    },
    Vsop87Term {
        amplitude: 8.00000000000000e-11,
        phase: 1.46298993048000e+00,
        frequency: 1.18562186514245e+04,
    },
    Vsop87Term {
        amplitude: 8.00000000000000e-11,
        phase: 5.07535888338000e+00,
        frequency: 6.25677753019160e+03,
    },
];

const EARTH_L_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 7.71400000000000e-08,
        phase: 4.14117321449000e+00,
        frequency: 6.28307584999140e+03,
    },
    Vsop87Term {
        amplitude: 1.01600000000000e-08,
        phase: 3.27573644241000e+00,
        frequency: 1.25661516999828e+04,
    },
    Vsop87Term {
        amplitude: 4.20000000000000e-09,
        phase: 4.18928514150000e-01,
        frequency: 1.55420399434200e+02,
    },
    Vsop87Term {
        amplitude: 4.70000000000000e-10,
        phase: 3.50591071186000e+00,
        frequency: 1.88492275499742e+04,
    },
    Vsop87Term {
        amplitude: 4.10000000000000e-10,
        phase: 3.14032562331000e+00,
        frequency: 3.52311834900000e+00,
    },
    Vsop87Term {
        amplitude: 3.50000000000000e-10,
        phase: 5.01110770000000e+00,
        frequency: 5.57314280143310e+03,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-10,
        phase: 5.64816633449000e+00,
        frequency: 6.12765545055720e+03,
    },
    Vsop87Term {
        amplitude: 1.30000000000000e-10,
        phase: 4.86092407740000e-01,
        frequency: 7.77137714681205e+04,
    },
    Vsop87Term {
        amplitude: 7.00000000000000e-11,
        phase: 2.84139222289000e+00,
        frequency: 1.61000685737674e+05,
    },
    Vsop87Term {
        amplitude: 4.00000000000000e-11,
        phase: 3.65509047070000e+00,
        frequency: 2.51323033999656e+04,
    },
    Vsop87Term {
        amplitude: 2.00000000000000e-11,
        phase: 5.48806034870000e-01,
        frequency: 6.43849624942560e+03,
    },
];

const EARTH_L_5: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.72000000000000e-09,
        phase: 2.74854172392000e+00,
        frequency: 6.28307584999140e+03,
    },
    Vsop87Term {
        amplitude: 5.00000000000000e-10,
        phase: 2.01352986713000e+00,
        frequency: 1.55420399434200e+02,
    },
    Vsop87Term {
        amplitude: 2.80000000000000e-10,
        phase: 2.93369985477000e+00,
        frequency: 1.25661516999828e+04,
    },
    Vsop87Term {
        amplitude: 5.00000000000000e-11,
        phase: 1.93829214518000e+00,
        frequency: 1.88492275499742e+04,
    },
];

const EARTH_B_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.79620000000000e-06,
        phase: 3.19870156017000e+00,
        frequency: 8.43346615813083e+04,
    },
    Vsop87Term {
        amplitude: 1.01643000000000e-06,
        phase: 5.42248619256000e+00,
        frequency: 5.50755323866740e+03,
    },
    Vsop87Term {
        amplitude: 8.04450000000000e-07,
        phase: 3.88013204458000e+00,
        frequency: 5.22369391980220e+03,
    },
    Vsop87Term {
        amplitude: 4.38060000000000e-07,
        phase: 3.70444689759000e+00,
        frequency: 2.35286615377180e+03,
    },
    Vsop87Term {
        amplitude: 3.19330000000000e-07,
        phase: 4.00026369781000e+00,
        frequency: 1.57734354244780e+03,
    },
    Vsop87Term {
        amplitude: 2.27240000000000e-07,
        phase: 3.98473831560000e+00,
        frequency: 1.04774731175470e+03,
    },
    Vsop87Term {
        amplitude: 1.63920000000000e-07,
        phase: 3.56456119782000e+00,
        frequency: 5.85647765911540e+03,
    },
    Vsop87Term {
        amplitude: 1.81410000000000e-07,
        phase: 4.98367470262000e+00,
        frequency: 6.28307584999140e+03,
    },
    Vsop87Term {
        amplitude: 1.44430000000000e-07,
        phase: 3.70275614915000e+00,
        frequency: 9.43776293488700e+03,
    },
    Vsop87Term {
        amplitude: 1.43040000000000e-07,
        phase: 3.41117857526000e+00,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 1.12460000000000e-07,
        phase: 4.82820690527000e+00,
        frequency: 1.41434952424306e+04,
    },
    Vsop87Term {
        amplitude: 1.09000000000000e-07,
        phase: 2.08574562329000e+00,
        frequency: 6.81276681508600e+03,
    },
    Vsop87Term {
        amplitude: 9.71400000000000e-08,
        phase: 3.47303947751000e+00,
        frequency: 4.69400295470760e+03,
    },
    Vsop87Term {
        amplitude: 1.03670000000000e-07,
        phase: 4.05663927945000e+00,
        frequency: 7.10928813549327e+04,
    },
    Vsop87Term {
        amplitude: 8.77500000000000e-08,
        phase: 4.44016515666000e+00,
        frequency: 5.75338488489680e+03,
    },
    Vsop87Term {
        amplitude: 8.36600000000000e-08,
        phase: 4.99251512183000e+00,
        frequency: 7.08489678111520e+03,
    },
    Vsop87Term {
        amplitude: 6.92100000000000e-08,
        phase: 4.32559054073000e+00,
        frequency: 6.27596230299060e+03,
    },
    Vsop87Term {
        amplitude: 9.14500000000000e-08,
        phase: 1.14182646613000e+00,
        frequency: 6.62089011318780e+03,
    },
    Vsop87Term {
        amplitude: 7.19400000000000e-08,
        phase: 3.60193205744000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 7.69800000000000e-08,
        phase: 5.55425745881000e+00,
        frequency: 1.67621575850862e+05,
    },
];

const EARTH_B_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.27777722000000e-03,
        phase: 3.41376620530000e+00,
        frequency: 6.28307584999140e+03,
    },
    Vsop87Term {
        amplitude: 3.80567800000000e-05,
        phase: 3.37063423795000e+00,
        frequency: 1.25661516999828e+04,
    },
    Vsop87Term {
        amplitude: 3.61958900000000e-05,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 7.15420000000000e-07,
        phase: 3.32777549735000e+00,
        frequency: 1.88492275499742e+04,
    },
    Vsop87Term {
        amplitude: 7.65500000000000e-08,
        phase: 1.79489607186000e+00,
        frequency: 5.22369391980220e+03,
    },
    Vsop87Term {
        amplitude: 8.10700000000000e-08,
        phase: 3.89190403643000e+00,
        frequency: 5.50755323866740e+03,
    },
    Vsop87Term {
        amplitude: 6.45600000000000e-08,
        phase: 5.19789424750000e+00,
        frequency: 2.35286615377180e+03,
    },
    Vsop87Term {
        amplitude: 3.89400000000000e-08,
        phase: 2.15568517178000e+00,
        frequency: 6.27955273164240e+03,
    },
    Vsop87Term {
        amplitude: 3.89200000000000e-08,
        phase: 1.53021064904000e+00,
        frequency: 6.28659896834040e+03,
    },
    Vsop87Term {
        amplitude: 3.89700000000000e-08,
        phase: 4.87293945629000e+00,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 3.81200000000000e-08,
        phase: 1.43523182316000e+00,
        frequency: 1.20364607348882e+04,
    },
    Vsop87Term {
        amplitude: 3.57700000000000e-08,
        phase: 2.32913869227000e+00,
        frequency: 8.39968473181119e+04,
    },
    Vsop87Term {
        amplitude: 3.57000000000000e-08,
        phase: 4.92637739003000e+00,
        frequency: 7.14306956181291e+04,
    },
    Vsop87Term {
        amplitude: 3.49400000000000e-08,
        phase: 2.20864641831000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 2.42100000000000e-08,
        phase: 6.22876183393000e+00,
        frequency: 7.86041939243920e+03,
    },
    Vsop87Term {
        amplitude: 2.05600000000000e-08,
        phase: 3.06747139741000e+00,
        frequency: 1.41434952424306e+04,
    },
    Vsop87Term {
        amplitude: 1.39900000000000e-08,
        phase: 5.01078779090000e-01,
        frequency: 6.30937416979120e+03,
    },
    Vsop87Term {
        amplitude: 1.41700000000000e-08,
        phase: 3.28454570977000e+00,
        frequency: 2.51323033999656e+04,
    },
    Vsop87Term {
        amplitude: 1.54400000000000e-08,
        phase: 1.82062047625000e+00,
        frequency: 5.85647765911540e+03,
    },
    Vsop87Term {
        amplitude: 1.45700000000000e-08,
        phase: 1.75339303307000e+00,
        frequency: 5.88492684658320e+03,
    },
];

const EARTH_B_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 9.72142400000000e-05,
        phase: 5.15192809920000e+00,
        frequency: 6.28307584999140e+03,
    },
    Vsop87Term {
        amplitude: 2.33002000000000e-06,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.34188000000000e-06,
        phase: 6.44062129770000e-01,
        frequency: 1.25661516999828e+04,
    },
    Vsop87Term {
        amplitude: 6.50400000000000e-08,
        phase: 1.07333397797000e+00,
        frequency: 1.88492275499742e+04,
    },
    Vsop87Term {
        amplitude: 1.66200000000000e-08,
        phase: 1.62746869551000e+00,
        frequency: 8.43346615813083e+04,
    },
    Vsop87Term {
        amplitude: 6.35000000000000e-09,
        phase: 3.51985338656000e+00,
        frequency: 6.27955273164240e+03,
    },
    Vsop87Term {
        amplitude: 4.92000000000000e-09,
        phase: 2.41382223971000e+00,
        frequency: 1.04774731175470e+03,
    },
    Vsop87Term {
        amplitude: 3.07000000000000e-09,
        phase: 6.10181422085000e+00,
        frequency: 5.22369391980220e+03,
    },
    Vsop87Term {
        amplitude: 3.22000000000000e-09,
        phase: 3.76608973890000e-01,
        frequency: 6.28659896834040e+03,
    },
    Vsop87Term {
        amplitude: 3.26000000000000e-09,
        phase: 2.35727931602000e+00,
        frequency: 5.50755323866740e+03,
    },
    Vsop87Term {
        amplitude: 2.74000000000000e-09,
        phase: 1.65307581765000e+00,
        frequency: 7.86041939243920e+03,
    },
    Vsop87Term {
        amplitude: 2.28000000000000e-09,
        phase: 1.14082932988000e+00,
        frequency: 2.51323033999656e+04,
    },
    Vsop87Term {
        amplitude: 2.02000000000000e-09,
        phase: 4.98366825300000e-01,
        frequency: 2.35286615377180e+03,
    },
    Vsop87Term {
        amplitude: 2.01000000000000e-09,
        phase: 1.55527656000000e-01,
        frequency: 1.02132855462110e+04,
    },
    Vsop87Term {
        amplitude: 1.67000000000000e-09,
        phase: 3.98005254015000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 1.70000000000000e-09,
        phase: 5.28668290523000e+00,
        frequency: 6.25677753019160e+03,
    },
    Vsop87Term {
        amplitude: 1.66000000000000e-09,
        phase: 3.04613930284000e+00,
        frequency: 1.20364607348882e+04,
    },
    Vsop87Term {
        amplitude: 1.53000000000000e-09,
        phase: 4.06779216239000e+00,
        frequency: 8.39968473181119e+04,
    },
    Vsop87Term {
        amplitude: 1.50000000000000e-09,
        phase: 3.18772213951000e+00,
        frequency: 7.14306956181291e+04,
    },
    Vsop87Term {
        amplitude: 1.20000000000000e-09,
        phase: 3.13558669517000e+00,
        frequency: 5.88492684658320e+03,
    },
];

const EARTH_B_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.75993000000000e-06,
        phase: 5.94800970920000e-01,
        frequency: 6.28307584999140e+03,
    },
    Vsop87Term {
        amplitude: 1.70340000000000e-07,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 3.61700000000000e-08,
        phase: 1.17505753250000e-01,
        frequency: 1.25661516999828e+04,
    },
    Vsop87Term {
        amplitude: 3.39000000000000e-09,
        phase: 5.66087461682000e+00,
        frequency: 1.88492275499742e+04,
    },
    Vsop87Term {
        amplitude: 5.60000000000000e-10,
        phase: 5.02765554835000e+00,
        frequency: 6.27955273164240e+03,
    },
    Vsop87Term {
        amplitude: 1.90000000000000e-10,
        phase: 5.99007646261000e+00,
        frequency: 6.25677753019160e+03,
    },
    Vsop87Term {
        amplitude: 1.80000000000000e-10,
        phase: 3.80004734567000e+00,
        frequency: 6.30937416979120e+03,
    },
    Vsop87Term {
        amplitude: 1.80000000000000e-10,
        phase: 1.21049250774000e+00,
        frequency: 6.12765545055720e+03,
    },
    Vsop87Term {
        amplitude: 1.80000000000000e-10,
        phase: 2.29734567137000e+00,
        frequency: 6.43849624942560e+03,
    },
    Vsop87Term {
        amplitude: 1.50000000000000e-10,
        phase: 4.72881467263000e+00,
        frequency: 6.28659896834040e+03,
    },
    Vsop87Term {
        amplitude: 1.20000000000000e-10,
        phase: 4.14816718080000e-01,
        frequency: 8.39968473181119e+04,
    },
    Vsop87Term {
        amplitude: 1.30000000000000e-10,
        phase: 5.54637369296000e+00,
        frequency: 2.51323033999656e+04,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-10,
        phase: 2.91937214232000e+00,
        frequency: 7.14306956181291e+04,
    },
    Vsop87Term {
        amplitude: 6.00000000000000e-11,
        phase: 2.14173241210000e+00,
        frequency: 1.18562186514245e+04,
    },
];

const EARTH_B_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 5.74500000000000e-08,
        phase: 2.26734029843000e+00,
        frequency: 6.28307584999140e+03,
    },
    Vsop87Term {
        amplitude: 8.70000000000000e-09,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.19000000000000e-09,
        phase: 4.26807972611000e+00,
        frequency: 1.25661516999828e+04,
    },
    Vsop87Term {
        amplitude: 1.70000000000000e-10,
        phase: 4.07422620440000e+00,
        frequency: 1.88492275499742e+04,
    },
    Vsop87Term {
        amplitude: 5.00000000000000e-11,
        phase: 8.43087052030000e-01,
        frequency: 1.04774731175470e+03,
    },
    Vsop87Term {
        amplitude: 5.00000000000000e-11,
        phase: 5.71157230300000e-02,
        frequency: 8.43346615813083e+04,
    },
];

const EARTH_B_5: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.14000000000000e-09,
        phase: 4.31455980099000e+00,
        frequency: 6.28307584999140e+03,
    },
    Vsop87Term {
        amplitude: 2.40000000000000e-10,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
];

const EARTH_R_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.00013988784000e+00,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.67069963200000e-02,
        phase: 3.09846350258000e+00,
        frequency: 6.28307584999140e+03,
    },
    Vsop87Term {
        amplitude: 1.39560240000000e-04,
        phase: 3.05524609456000e+00,
        frequency: 1.25661516999828e+04,
    },
    Vsop87Term {
        amplitude: 3.08372000000000e-05,
        phase: 5.19846674381000e+00,
        frequency: 7.77137714681205e+04,
    },
    Vsop87Term {
        amplitude: 1.62846300000000e-05,
        phase: 1.17387558054000e+00,
        frequency: 5.75338488489680e+03,
    },
    Vsop87Term {
        amplitude: 1.57557200000000e-05,
        phase: 2.84685214877000e+00,
        frequency: 7.86041939243920e+03,
    },
    Vsop87Term {
        amplitude: 9.24799000000000e-06,
        phase: 5.45292236722000e+00,
        frequency: 1.15067697697936e+04,
    },
    Vsop87Term {
        amplitude: 5.42439000000000e-06,
        phase: 4.56409151453000e+00,
        frequency: 3.93020969621960e+03,
    },
    Vsop87Term {
        amplitude: 4.72110000000000e-06,
        phase: 3.66100022149000e+00,
        frequency: 5.88492684658320e+03,
    },
    Vsop87Term {
        amplitude: 3.28780000000000e-06,
        phase: 5.89983686142000e+00,
        frequency: 5.22369391980220e+03,
    },
    Vsop87Term {
        amplitude: 3.45969000000000e-06,
        phase: 9.63686272720000e-01,
        frequency: 5.50755323866740e+03,
    },
    Vsop87Term {
        amplitude: 3.06784000000000e-06,
        phase: 2.98671395120000e-01,
        frequency: 5.57314280143310e+03,
    },
    Vsop87Term {
        amplitude: 1.74844000000000e-06,
        phase: 3.01193636733000e+00,
        frequency: 1.88492275499742e+04,
    },
    Vsop87Term {
        amplitude: 2.43181000000000e-06,
        phase: 4.27349530790000e+00,
        frequency: 1.17906290886588e+04,
    },
    Vsop87Term {
        amplitude: 2.11836000000000e-06,
        phase: 5.84714461348000e+00,
        frequency: 1.57734354244780e+03,
    },
    Vsop87Term {
        amplitude: 1.85740000000000e-06,
        phase: 5.02199710705000e+00,
        frequency: 1.09770788046990e+04,
    },
    Vsop87Term {
        amplitude: 1.09835000000000e-06,
        phase: 5.05510635860000e+00,
        frequency: 5.48677784317500e+03,
    },
    Vsop87Term {
        amplitude: 9.83160000000000e-07,
        phase: 8.86813112780000e-01,
        frequency: 6.06977675455340e+03,
    },
    Vsop87Term {
        amplitude: 8.65000000000000e-07,
        phase: 5.68956418946000e+00,
        frequency: 1.57208387848784e+04,
    },
    Vsop87Term {
        amplitude: 8.58310000000000e-07,
        phase: 1.27079125277000e+00,
        frequency: 1.61000685737674e+05,
    },
];

const EARTH_R_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.03018607000000e-03,
        phase: 1.10748968172000e+00,
        frequency: 6.28307584999140e+03,
    },
    Vsop87Term {
        amplitude: 1.72123800000000e-05,
        phase: 1.06442300386000e+00,
        frequency: 1.25661516999828e+04,
    },
    Vsop87Term {
        amplitude: 7.02217000000000e-06,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 3.23450000000000e-07,
        phase: 1.02168583254000e+00,
        frequency: 1.88492275499742e+04,
    },
    Vsop87Term {
        amplitude: 3.08010000000000e-07,
        phase: 2.84358443952000e+00,
        frequency: 5.50755323866740e+03,
    },
    Vsop87Term {
        amplitude: 2.49780000000000e-07,
        phase: 1.31906570344000e+00,
        frequency: 5.22369391980220e+03,
    },
    Vsop87Term {
        amplitude: 1.84870000000000e-07,
        phase: 1.42428709076000e+00,
        frequency: 1.57734354244780e+03,
    },
    Vsop87Term {
        amplitude: 1.00770000000000e-07,
        phase: 5.91385248388000e+00,
        frequency: 1.09770788046990e+04,
    },
    Vsop87Term {
        amplitude: 8.63500000000000e-08,
        phase: 2.71581929450000e-01,
        frequency: 5.48677784317500e+03,
    },
    Vsop87Term {
        amplitude: 8.65400000000000e-08,
        phase: 1.42046854427000e+00,
        frequency: 6.27596230299060e+03,
    },
    Vsop87Term {
        amplitude: 5.06900000000000e-08,
        phase: 1.68613408916000e+00,
        frequency: 5.08862883976680e+03,
    },
    Vsop87Term {
        amplitude: 4.98500000000000e-08,
        phase: 6.01402338185000e+00,
        frequency: 6.28659896834040e+03,
    },
    Vsop87Term {
        amplitude: 4.66700000000000e-08,
        phase: 5.98749245692000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 4.39500000000000e-08,
        phase: 5.18004234450000e-01,
        frequency: 4.69400295470760e+03,
    },
    Vsop87Term {
        amplitude: 3.87000000000000e-08,
        phase: 4.74932206877000e+00,
        frequency: 2.54431441988340e+03,
    },
    Vsop87Term {
        amplitude: 3.75500000000000e-08,
        phase: 5.07053801166000e+00,
        frequency: 7.96298006816400e+02,
    },
    Vsop87Term {
        amplitude: 4.10000000000000e-08,
        phase: 1.08424801084000e+00,
        frequency: 9.43776293488700e+03,
    },
    Vsop87Term {
        amplitude: 3.51800000000000e-08,
        phase: 2.29021697800000e-02,
        frequency: 8.39968473181119e+04,
    },
    Vsop87Term {
        amplitude: 3.43600000000000e-08,
        phase: 9.49375038720000e-01,
        frequency: 7.14306956181291e+04,
    },
    Vsop87Term {
        amplitude: 3.22100000000000e-08,
        phase: 6.15628775321000e+00,
        frequency: 2.14616541647520e+03,
    },
];

const EARTH_R_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 4.35938500000000e-05,
        phase: 5.78455133808000e+00,
        frequency: 6.28307584999140e+03,
    },
    Vsop87Term {
        amplitude: 1.23633000000000e-06,
        phase: 5.57935427994000e+00,
        frequency: 1.25661516999828e+04,
    },
    Vsop87Term {
        amplitude: 1.23420000000000e-07,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 8.79200000000000e-08,
        phase: 3.62777893099000e+00,
        frequency: 7.77137714681205e+04,
    },
    Vsop87Term {
        amplitude: 5.68900000000000e-08,
        phase: 1.86958905084000e+00,
        frequency: 5.57314280143310e+03,
    },
    Vsop87Term {
        amplitude: 3.30200000000000e-08,
        phase: 5.47034879713000e+00,
        frequency: 1.88492275499742e+04,
    },
    Vsop87Term {
        amplitude: 1.47100000000000e-08,
        phase: 4.47964125007000e+00,
        frequency: 5.50755323866740e+03,
    },
    Vsop87Term {
        amplitude: 1.01300000000000e-08,
        phase: 2.81323115556000e+00,
        frequency: 5.22369391980220e+03,
    },
    Vsop87Term {
        amplitude: 8.54000000000000e-09,
        phase: 3.10776566900000e+00,
        frequency: 1.57734354244780e+03,
    },
    Vsop87Term {
        amplitude: 1.10200000000000e-08,
        phase: 2.84173992403000e+00,
        frequency: 1.61000685737674e+05,
    },
    Vsop87Term {
        amplitude: 6.48000000000000e-09,
        phase: 5.47348203398000e+00,
        frequency: 7.75522611324000e+02,
    },
    Vsop87Term {
        amplitude: 6.08000000000000e-09,
        phase: 1.37894173533000e+00,
        frequency: 6.43849624942560e+03,
    },
    Vsop87Term {
        amplitude: 4.99000000000000e-09,
        phase: 4.41649242250000e+00,
        frequency: 6.28659896834040e+03,
    },
    Vsop87Term {
        amplitude: 4.16000000000000e-09,
        phase: 9.03326979740000e-01,
        frequency: 1.09770788046990e+04,
    },
    Vsop87Term {
        amplitude: 4.04000000000000e-09,
        phase: 3.20567269530000e+00,
        frequency: 5.08862883976680e+03,
    },
    Vsop87Term {
        amplitude: 3.51000000000000e-09,
        phase: 1.81081728907000e+00,
        frequency: 5.48677784317500e+03,
    },
    Vsop87Term {
        amplitude: 4.66000000000000e-09,
        phase: 3.65086758149000e+00,
        frequency: 7.08489678111520e+03,
    },
    Vsop87Term {
        amplitude: 4.58000000000000e-09,
        phase: 5.38585314743000e+00,
        frequency: 1.49854400134808e+05,
    },
    Vsop87Term {
        amplitude: 3.04000000000000e-09,
        phase: 3.51015066341000e+00,
        frequency: 7.96298006816400e+02,
    },
    Vsop87Term {
        amplitude: 2.66000000000000e-09,
        phase: 6.17413982699000e+00,
        frequency: 6.83664525283380e+03,
    },
];

const EARTH_R_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.44595000000000e-06,
        phase: 4.27319433901000e+00,
        frequency: 6.28307584999140e+03,
    },
    Vsop87Term {
        amplitude: 6.72900000000000e-08,
        phase: 3.91706261708000e+00,
        frequency: 1.25661516999828e+04,
    },
    Vsop87Term {
        amplitude: 7.74000000000000e-09,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 2.47000000000000e-09,
        phase: 3.73021571217000e+00,
        frequency: 1.88492275499742e+04,
    },
    Vsop87Term {
        amplitude: 3.60000000000000e-10,
        phase: 2.80081409050000e+00,
        frequency: 6.28659896834040e+03,
    },
    Vsop87Term {
        amplitude: 3.30000000000000e-10,
        phase: 5.62990083112000e+00,
        frequency: 6.12765545055720e+03,
    },
    Vsop87Term {
        amplitude: 1.80000000000000e-10,
        phase: 3.72826142555000e+00,
        frequency: 6.43849624942560e+03,
    },
    Vsop87Term {
        amplitude: 1.60000000000000e-10,
        phase: 4.26011484232000e+00,
        frequency: 6.52580445396540e+03,
    },
    Vsop87Term {
        amplitude: 1.40000000000000e-10,
        phase: 3.47817116396000e+00,
        frequency: 6.25677753019160e+03,
    },
    Vsop87Term {
        amplitude: 1.20000000000000e-10,
        phase: 3.55747379482000e+00,
        frequency: 2.51323033999656e+04,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-10,
        phase: 4.43995693209000e+00,
        frequency: 4.70573230754360e+03,
    },
    Vsop87Term {
        amplitude: 1.00000000000000e-10,
        phase: 4.28045255470000e+00,
        frequency: 8.39968473181119e+04,
    },
    Vsop87Term {
        amplitude: 9.00000000000000e-11,
        phase: 5.36457057335000e+00,
        frequency: 6.04034724601740e+03,
    },
    Vsop87Term {
        amplitude: 8.00000000000000e-11,
        phase: 1.78458957263000e+00,
        frequency: 5.50755323866740e+03,
    },
    Vsop87Term {
        amplitude: 9.00000000000000e-11,
        phase: 4.72751999300000e-01,
        frequency: 6.27955273164240e+03,
    },
    Vsop87Term {
        amplitude: 9.00000000000000e-11,
        phase: 1.34741231639000e+00,
        frequency: 6.30937416979120e+03,
    },
    Vsop87Term {
        amplitude: 9.00000000000000e-11,
        phase: 7.70929007080000e-01,
        frequency: 5.72950644714900e+03,
    },
    Vsop87Term {
        amplitude: 7.00000000000000e-11,
        phase: 3.50146897332000e+00,
        frequency: 7.05859846131540e+03,
    },
    Vsop87Term {
        amplitude: 5.00000000000000e-11,
        phase: 2.89071061700000e+00,
        frequency: 7.75522611324000e+02,
    },
    Vsop87Term {
        amplitude: 6.00000000000000e-11,
        phase: 2.36514111314000e+00,
        frequency: 6.83664525283380e+03,
    },
];

const EARTH_R_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 3.85800000000000e-08,
        phase: 2.56389016346000e+00,
        frequency: 6.28307584999140e+03,
    },
    Vsop87Term {
        amplitude: 3.06000000000000e-09,
        phase: 2.26911740541000e+00,
        frequency: 1.25661516999828e+04,
    },
    Vsop87Term {
        amplitude: 5.30000000000000e-10,
        phase: 3.44031471924000e+00,
        frequency: 5.57314280143310e+03,
    },
    Vsop87Term {
        amplitude: 1.50000000000000e-10,
        phase: 2.03136359366000e+00,
        frequency: 1.88492275499742e+04,
    },
    Vsop87Term {
        amplitude: 1.30000000000000e-10,
        phase: 2.05688873673000e+00,
        frequency: 7.77137714681205e+04,
    },
    Vsop87Term {
        amplitude: 7.00000000000000e-11,
        phase: 4.41218854480000e+00,
        frequency: 1.61000685737674e+05,
    },
    Vsop87Term {
        amplitude: 4.00000000000000e-11,
        phase: 5.33854414781000e+00,
        frequency: 6.43849624942560e+03,
    },
    Vsop87Term {
        amplitude: 6.00000000000000e-11,
        phase: 3.81514213664000e+00,
        frequency: 1.49854400134808e+05,
    },
    Vsop87Term {
        amplitude: 4.00000000000000e-11,
        phase: 4.26602478239000e+00,
        frequency: 6.12765545055720e+03,
    },
];

const EARTH_R_5: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 8.60000000000000e-10,
        phase: 1.21805304895000e+00,
        frequency: 6.28307584999140e+03,
    },
    Vsop87Term {
        amplitude: 1.20000000000000e-10,
        phase: 6.55728780440000e-01,
        frequency: 1.25661516999828e+04,
    },
];
