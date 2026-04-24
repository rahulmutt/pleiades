//! Truncated VSOP87B Saturn coefficient tables.
//!
//! Coefficients are the leading terms from the public IMCCE/CELMECH
//! `VSOP87B.sat` file (Saturn heliocentric spherical variables, J2000
//! ecliptic/equinox). This mirrors the other checked-in planetary slices
//! while complete generated tables are still planned.

use crate::vsop87b_earth::{evaluate, SphericalLbr, Vsop87Term};

pub(crate) fn saturn_lbr(julian_day_tt: f64) -> SphericalLbr {
    let t = (julian_day_tt - 2_451_545.0) / 365_250.0;
    SphericalLbr {
        longitude_rad: evaluate(
            &[
                SATURN_L_0, SATURN_L_1, SATURN_L_2, SATURN_L_3, SATURN_L_4, SATURN_L_5,
            ],
            t,
        )
        .rem_euclid(core::f64::consts::TAU),
        latitude_rad: evaluate(
            &[
                SATURN_B_0, SATURN_B_1, SATURN_B_2, SATURN_B_3, SATURN_B_4, SATURN_B_5,
            ],
            t,
        ),
        radius_au: evaluate(
            &[
                SATURN_R_0, SATURN_R_1, SATURN_R_2, SATURN_R_3, SATURN_R_4, SATURN_R_5,
            ],
            t,
        ),
    }
}

const SATURN_L_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 8.74013540250000e-01,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.11076597620000e-01,
        phase: 3.96205090159000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 1.41415095700000e-02,
        phase: 4.58581516874000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 3.98379389000000e-03,
        phase: 5.21120326990000e-01,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 3.50769243000000e-03,
        phase: 3.30329907896000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 2.06816305000000e-03,
        phase: 2.46583720020000e-01,
        frequency: 1.03092774218600e+02,
    },
    Vsop87Term {
        amplitude: 7.92713000000000e-04,
        phase: 3.84007056878000e+00,
        frequency: 2.20412642438800e+02,
    },
    Vsop87Term {
        amplitude: 2.39903550000000e-04,
        phase: 4.66976924553000e+00,
        frequency: 1.10206321219400e+02,
    },
    Vsop87Term {
        amplitude: 1.65735880000000e-04,
        phase: 4.37192282960000e-01,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 1.49069950000000e-04,
        phase: 5.76903183869000e+00,
        frequency: 3.16391869656600e+02,
    },
    Vsop87Term {
        amplitude: 1.58202900000000e-04,
        phase: 9.38091552350000e-01,
        frequency: 6.32783739313200e+02,
    },
    Vsop87Term {
        amplitude: 1.46095590000000e-04,
        phase: 1.56518472000000e+00,
        frequency: 3.93215326310000e+00,
    },
    Vsop87Term {
        amplitude: 1.31603010000000e-04,
        phase: 4.44891291899000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 1.50535430000000e-04,
        phase: 2.71669915667000e+00,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 1.30052990000000e-04,
        phase: 5.98119023644000e+00,
        frequency: 1.10457002639000e+01,
    },
    Vsop87Term {
        amplitude: 1.07250670000000e-04,
        phase: 3.12939523827000e+00,
        frequency: 2.02253395174100e+02,
    },
    Vsop87Term {
        amplitude: 5.86320600000000e-05,
        phase: 2.36569385240000e-01,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 5.22775700000000e-05,
        phase: 4.20783365759000e+00,
        frequency: 3.18139373770000e+00,
    },
    Vsop87Term {
        amplitude: 6.12631700000000e-05,
        phase: 1.76328667907000e+00,
        frequency: 2.77034993741400e+02,
    },
    Vsop87Term {
        amplitude: 5.01968700000000e-05,
        phase: 3.17787728405000e+00,
        frequency: 4.33711737876800e+02,
    },
];

const SATURN_L_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.13299095216900e+02,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.29737086200000e-02,
        phase: 1.82834923978000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 5.64345393000000e-03,
        phase: 2.88499717272000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 9.37343690000000e-04,
        phase: 1.06311793502000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 1.07674962000000e-03,
        phase: 2.27769131009000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 4.02444550000000e-04,
        phase: 2.04108104671000e+00,
        frequency: 2.20412642438800e+02,
    },
    Vsop87Term {
        amplitude: 1.99417740000000e-04,
        phase: 1.27954390470000e+00,
        frequency: 1.03092774218600e+02,
    },
    Vsop87Term {
        amplitude: 1.05116780000000e-04,
        phase: 2.74880342130000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 6.41610600000000e-05,
        phase: 3.82382950410000e-01,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 4.84899400000000e-05,
        phase: 2.43037610229000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 4.05689200000000e-05,
        phase: 2.92133209468000e+00,
        frequency: 1.10206321219400e+02,
    },
    Vsop87Term {
        amplitude: 3.76863500000000e-05,
        phase: 3.64965330780000e+00,
        frequency: 3.93215326310000e+00,
    },
    Vsop87Term {
        amplitude: 3.38469100000000e-05,
        phase: 2.41694503459000e+00,
        frequency: 3.18139373770000e+00,
    },
    Vsop87Term {
        amplitude: 3.23169300000000e-05,
        phase: 1.26149969158000e+00,
        frequency: 4.33711737876800e+02,
    },
    Vsop87Term {
        amplitude: 3.07140500000000e-05,
        phase: 2.32739504783000e+00,
        frequency: 1.99072001436400e+02,
    },
    Vsop87Term {
        amplitude: 1.95317900000000e-05,
        phase: 3.56378136497000e+00,
        frequency: 1.10457002639000e+01,
    },
    Vsop87Term {
        amplitude: 1.24946800000000e-05,
        phase: 2.62810757084000e+00,
        frequency: 9.59792272178000e+01,
    },
    Vsop87Term {
        amplitude: 9.21350000000000e-06,
        phase: 1.96069472334000e+00,
        frequency: 2.27526189439600e+02,
    },
    Vsop87Term {
        amplitude: 7.01524000000000e-06,
        phase: 4.43097553887000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 6.49591000000000e-06,
        phase: 6.17410622073000e+00,
        frequency: 2.02253395174100e+02,
    },
];

const SATURN_L_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.16441330000000e-03,
        phase: 1.17988132879000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 9.18418370000000e-04,
        phase: 7.32519584000000e-02,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 3.66617280000000e-04,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.52744960000000e-04,
        phase: 4.06493179167000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 1.09872590000000e-04,
        phase: 5.44479188310000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 1.06298300000000e-04,
        phase: 2.57643061890000e-01,
        frequency: 2.20412642438800e+02,
    },
    Vsop87Term {
        amplitude: 4.26540400000000e-05,
        phase: 1.04596041482000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 1.21544700000000e-05,
        phase: 2.91866579609000e+00,
        frequency: 1.03092774218600e+02,
    },
    Vsop87Term {
        amplitude: 1.14259500000000e-05,
        phase: 4.63711665368000e+00,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 1.06149400000000e-05,
        phase: 5.68896768215000e+00,
        frequency: 4.33711737876800e+02,
    },
    Vsop87Term {
        amplitude: 1.02010200000000e-05,
        phase: 6.33684572500000e-01,
        frequency: 3.18139373770000e+00,
    },
    Vsop87Term {
        amplitude: 1.04475900000000e-05,
        phase: 4.04202827818000e+00,
        frequency: 1.99072001436400e+02,
    },
    Vsop87Term {
        amplitude: 6.48857000000000e-06,
        phase: 4.33990455509000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 5.49320000000000e-06,
        phase: 5.57301151406000e+00,
        frequency: 3.93215326310000e+00,
    },
    Vsop87Term {
        amplitude: 4.56767000000000e-06,
        phase: 1.26896848480000e+00,
        frequency: 1.10206321219400e+02,
    },
    Vsop87Term {
        amplitude: 4.24918000000000e-06,
        phase: 2.09087865190000e-01,
        frequency: 2.27526189439600e+02,
    },
    Vsop87Term {
        amplitude: 2.73782000000000e-06,
        phase: 4.28857061190000e+00,
        frequency: 9.59792272178000e+01,
    },
    Vsop87Term {
        amplitude: 1.61533000000000e-06,
        phase: 1.38145587317000e+00,
        frequency: 1.10457002639000e+01,
    },
    Vsop87Term {
        amplitude: 1.29502000000000e-06,
        phase: 1.56592444783000e+00,
        frequency: 3.09278322655800e+02,
    },
    Vsop87Term {
        amplitude: 1.08829000000000e-06,
        phase: 3.89769392463000e+00,
        frequency: 8.53196381752000e+02,
    },
];

const SATURN_L_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.60387320000000e-04,
        phase: 5.73945573267000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 4.25473700000000e-05,
        phase: 4.58877599687000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 1.90637900000000e-05,
        phase: 4.76070843570000e+00,
        frequency: 2.20412642438800e+02,
    },
    Vsop87Term {
        amplitude: 1.46495900000000e-05,
        phase: 5.91328884284000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 1.16206200000000e-05,
        phase: 5.61974313217000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 1.04476500000000e-05,
        phase: 3.57813061587000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 2.36068000000000e-06,
        phase: 3.85849798708000e+00,
        frequency: 4.33711737876800e+02,
    },
    Vsop87Term {
        amplitude: 2.37009000000000e-06,
        phase: 5.76820709729000e+00,
        frequency: 1.99072001436400e+02,
    },
    Vsop87Term {
        amplitude: 1.65645000000000e-06,
        phase: 5.11642167451000e+00,
        frequency: 3.18139373770000e+00,
    },
    Vsop87Term {
        amplitude: 1.31328000000000e-06,
        phase: 4.74306126145000e+00,
        frequency: 2.27526189439600e+02,
    },
    Vsop87Term {
        amplitude: 1.50882000000000e-06,
        phase: 2.72695802283000e+00,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 6.16070000000000e-07,
        phase: 4.74260728276000e+00,
        frequency: 1.03092774218600e+02,
    },
    Vsop87Term {
        amplitude: 6.38990000000000e-07,
        phase: 8.67262376200000e-02,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 4.04050000000000e-07,
        phase: 5.47280316518000e+00,
        frequency: 2.13406410024000e+01,
    },
    Vsop87Term {
        amplitude: 4.02220000000000e-07,
        phase: 5.96343977224000e+00,
        frequency: 9.59792272178000e+01,
    },
    Vsop87Term {
        amplitude: 3.88070000000000e-07,
        phase: 5.83309187434000e+00,
        frequency: 1.10206321219400e+02,
    },
    Vsop87Term {
        amplitude: 2.69490000000000e-07,
        phase: 3.00877360899000e+00,
        frequency: 6.47010833314800e+02,
    },
    Vsop87Term {
        amplitude: 2.50170000000000e-07,
        phase: 9.86755764910000e-01,
        frequency: 3.93215326310000e+00,
    },
    Vsop87Term {
        amplitude: 3.26920000000000e-07,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.80510000000000e-07,
        phase: 1.02181794600000e+00,
        frequency: 4.12371096874400e+02,
    },
];

const SATURN_L_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.66187700000000e-05,
        phase: 3.99824447634000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 2.57094000000000e-06,
        phase: 2.98422287887000e+00,
        frequency: 2.20412642438800e+02,
    },
    Vsop87Term {
        amplitude: 2.36328000000000e-06,
        phase: 3.90248844320000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 1.49520000000000e-06,
        phase: 2.73191135434000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 1.09412000000000e-06,
        phase: 1.51564560686000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 6.91190000000000e-07,
        phase: 1.74804093636000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 3.76800000000000e-07,
        phase: 1.23800346661000e+00,
        frequency: 1.99072001436400e+02,
    },
    Vsop87Term {
        amplitude: 3.96780000000000e-07,
        phase: 2.04527339062000e+00,
        frequency: 4.33711737876800e+02,
    },
    Vsop87Term {
        amplitude: 3.11720000000000e-07,
        phase: 3.01055217526000e+00,
        frequency: 2.27526189439600e+02,
    },
    Vsop87Term {
        amplitude: 1.50260000000000e-07,
        phase: 8.32497806160000e-01,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 9.42400000000000e-08,
        phase: 3.71267465225000e+00,
        frequency: 2.13406410024000e+01,
    },
    Vsop87Term {
        amplitude: 5.13100000000000e-08,
        phase: 2.14278851183000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 4.37900000000000e-08,
        phase: 1.44314873951000e+00,
        frequency: 9.59792272178000e+01,
    },
    Vsop87Term {
        amplitude: 5.39100000000000e-08,
        phase: 1.15849076251000e+00,
        frequency: 6.47010833314800e+02,
    },
    Vsop87Term {
        amplitude: 4.31500000000000e-08,
        phase: 2.11844568875000e+00,
        frequency: 4.40825284877600e+02,
    },
    Vsop87Term {
        amplitude: 3.21500000000000e-08,
        phase: 4.10085180982000e+00,
        frequency: 1.10206321219400e+02,
    },
    Vsop87Term {
        amplitude: 2.86600000000000e-08,
        phase: 3.03604951200000e+00,
        frequency: 8.88656802170000e+01,
    },
    Vsop87Term {
        amplitude: 2.82500000000000e-08,
        phase: 2.76965112625000e+00,
        frequency: 4.12371096874400e+02,
    },
    Vsop87Term {
        amplitude: 2.58400000000000e-08,
        phase: 6.28047035280000e+00,
        frequency: 8.53196381752000e+02,
    },
    Vsop87Term {
        amplitude: 2.61600000000000e-08,
        phase: 3.85760382180000e-01,
        frequency: 1.03092774218600e+02,
    },
];

const SATURN_L_5: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.23607000000000e-06,
        phase: 2.25923420203000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 3.41760000000000e-07,
        phase: 2.16278773143000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 2.75390000000000e-07,
        phase: 1.19822164604000e+00,
        frequency: 2.20412642438800e+02,
    },
    Vsop87Term {
        amplitude: 5.76300000000000e-08,
        phase: 1.21171444884000e+00,
        frequency: 2.27526189439600e+02,
    },
    Vsop87Term {
        amplitude: 5.28400000000000e-08,
        phase: 2.35208912950000e-01,
        frequency: 4.33711737876800e+02,
    },
    Vsop87Term {
        amplitude: 3.65000000000000e-08,
        phase: 6.20014021207000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 3.06100000000000e-08,
        phase: 2.96839870592000e+00,
        frequency: 1.99072001436400e+02,
    },
    Vsop87Term {
        amplitude: 2.86500000000000e-08,
        phase: 4.29470838129000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 1.49900000000000e-08,
        phase: 6.21044685389000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 1.26200000000000e-08,
        phase: 5.25209851911000e+00,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 7.56000000000000e-09,
        phase: 6.17670364645000e+00,
        frequency: 1.91958454435600e+02,
    },
    Vsop87Term {
        amplitude: 7.59000000000000e-09,
        phase: 6.91270923290000e-01,
        frequency: 3.02164775655000e+02,
    },
    Vsop87Term {
        amplitude: 8.20000000000000e-09,
        phase: 5.59433772118000e+00,
        frequency: 6.47010833314800e+02,
    },
    Vsop87Term {
        amplitude: 9.42000000000000e-09,
        phase: 2.45840205430000e-01,
        frequency: 4.40825284877600e+02,
    },
    Vsop87Term {
        amplitude: 5.47000000000000e-09,
        phase: 4.87451203466000e+00,
        frequency: 8.88656802170000e+01,
    },
    Vsop87Term {
        amplitude: 5.03000000000000e-09,
        phase: 4.63319665449000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 3.57000000000000e-09,
        phase: 4.73247835262000e+00,
        frequency: 8.60309928752800e+02,
    },
    Vsop87Term {
        amplitude: 3.43000000000000e-09,
        phase: 5.70825898673000e+00,
        frequency: 6.54124380315600e+02,
    },
    Vsop87Term {
        amplitude: 2.43000000000000e-09,
        phase: 2.03429529667000e+00,
        frequency: 3.23505416657400e+02,
    },
    Vsop87Term {
        amplitude: 3.05000000000000e-09,
        phase: 1.06249794404000e+00,
        frequency: 2.34639736440400e+02,
    },
];

const SATURN_B_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 4.33067803900000e-02,
        phase: 3.60284428399000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 2.40348302000000e-03,
        phase: 2.85238489373000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 8.47459390000000e-04,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 3.08633570000000e-04,
        phase: 3.48441504555000e+00,
        frequency: 2.20412642438800e+02,
    },
    Vsop87Term {
        amplitude: 3.41160620000000e-04,
        phase: 5.72973075570000e-01,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 1.47340700000000e-04,
        phase: 2.11846596715000e+00,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 9.91666700000000e-05,
        phase: 5.79003188904000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 6.99356400000000e-05,
        phase: 4.73604689720000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 4.80758800000000e-05,
        phase: 5.43305312061000e+00,
        frequency: 3.16391869656600e+02,
    },
    Vsop87Term {
        amplitude: 4.78839200000000e-05,
        phase: 4.96512926584000e+00,
        frequency: 1.10206321219400e+02,
    },
    Vsop87Term {
        amplitude: 3.43212500000000e-05,
        phase: 2.73255746600000e+00,
        frequency: 4.33711737876800e+02,
    },
    Vsop87Term {
        amplitude: 1.50612900000000e-05,
        phase: 6.01304519391000e+00,
        frequency: 1.03092774218600e+02,
    },
    Vsop87Term {
        amplitude: 1.06029800000000e-05,
        phase: 5.63099296460000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 9.69071000000000e-06,
        phase: 5.20434966293000e+00,
        frequency: 6.32783739313200e+02,
    },
    Vsop87Term {
        amplitude: 9.42050000000000e-06,
        phase: 1.39646688872000e+00,
        frequency: 8.53196381752000e+02,
    },
    Vsop87Term {
        amplitude: 7.07645000000000e-06,
        phase: 3.80302289005000e+00,
        frequency: 3.23505416657400e+02,
    },
    Vsop87Term {
        amplitude: 5.52314000000000e-06,
        phase: 5.13149119536000e+00,
        frequency: 2.02253395174100e+02,
    },
    Vsop87Term {
        amplitude: 3.99674000000000e-06,
        phase: 3.35891409671000e+00,
        frequency: 2.27526189439600e+02,
    },
    Vsop87Term {
        amplitude: 3.16063000000000e-06,
        phase: 1.99716693551000e+00,
        frequency: 6.47010833314800e+02,
    },
    Vsop87Term {
        amplitude: 3.19380000000000e-06,
        phase: 3.62571687438000e+00,
        frequency: 2.09366942174900e+02,
    },
];

const SATURN_B_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.98927992000000e-03,
        phase: 4.93901017903000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 3.69479160000000e-04,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.79669890000000e-04,
        phase: 5.19794311100000e-01,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 1.09197210000000e-04,
        phase: 1.79463271368000e+00,
        frequency: 2.20412642438800e+02,
    },
    Vsop87Term {
        amplitude: 1.33202650000000e-04,
        phase: 2.26481519893000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 3.24342800000000e-05,
        phase: 1.21094033148000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 2.90051900000000e-05,
        phase: 6.17033461979000e+00,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 1.58471200000000e-05,
        phase: 9.34163971300000e-01,
        frequency: 4.33711737876800e+02,
    },
    Vsop87Term {
        amplitude: 1.58066600000000e-05,
        phase: 3.08171717435000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 7.00659000000000e-06,
        phase: 2.05451520780000e-01,
        frequency: 3.16391869656600e+02,
    },
    Vsop87Term {
        amplitude: 3.10902000000000e-06,
        phase: 4.38351712708000e+00,
        frequency: 1.10206321219400e+02,
    },
    Vsop87Term {
        amplitude: 3.01237000000000e-06,
        phase: 1.66219956459000e+00,
        frequency: 2.27526189439600e+02,
    },
    Vsop87Term {
        amplitude: 3.03761000000000e-06,
        phase: 5.46322830151000e+00,
        frequency: 8.53196381752000e+02,
    },
    Vsop87Term {
        amplitude: 2.59878000000000e-06,
        phase: 3.93026240568000e+00,
        frequency: 1.03092774218600e+02,
    },
    Vsop87Term {
        amplitude: 2.52673000000000e-06,
        phase: 9.00209252100000e-01,
        frequency: 6.32783739313200e+02,
    },
    Vsop87Term {
        amplitude: 1.82664000000000e-06,
        phase: 1.21424381480000e-01,
        frequency: 6.47010833314800e+02,
    },
    Vsop87Term {
        amplitude: 1.57532000000000e-06,
        phase: 2.42607457234000e+00,
        frequency: 1.99072001436400e+02,
    },
    Vsop87Term {
        amplitude: 1.08184000000000e-06,
        phase: 1.39896246207000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 8.83010000000000e-07,
        phase: 2.17503185037000e+00,
        frequency: 3.23505416657400e+02,
    },
    Vsop87Term {
        amplitude: 8.68750000000000e-07,
        phase: 2.91365320786000e+00,
        frequency: 1.42270940016000e+01,
    },
];

const SATURN_B_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.38842640000000e-04,
        phase: 8.99499869100000e-02,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 3.07571300000000e-05,
        phase: 3.91610937620000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 2.08166600000000e-05,
        phase: 9.63196807700000e-02,
        frequency: 2.20412642438800e+02,
    },
    Vsop87Term {
        amplitude: 1.45257400000000e-05,
        phase: 5.48867576013000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 5.46808000000000e-06,
        phase: 2.94585826799000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 3.91398000000000e-06,
        phase: 5.43939792344000e+00,
        frequency: 4.33711737876800e+02,
    },
    Vsop87Term {
        amplitude: 3.19740000000000e-06,
        phase: 4.34820275048000e+00,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 2.03518000000000e-06,
        phase: 1.37396136744000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 2.20164000000000e-06,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.16719000000000e-06,
        phase: 6.24505924943000e+00,
        frequency: 2.27526189439600e+02,
    },
    Vsop87Term {
        amplitude: 6.76050000000000e-07,
        phase: 1.75135990376000e+00,
        frequency: 3.16391869656600e+02,
    },
    Vsop87Term {
        amplitude: 6.40440000000000e-07,
        phase: 4.10904350356000e+00,
        frequency: 1.99072001436400e+02,
    },
    Vsop87Term {
        amplitude: 5.55180000000000e-07,
        phase: 4.56815095513000e+00,
        frequency: 6.47010833314800e+02,
    },
    Vsop87Term {
        amplitude: 4.98750000000000e-07,
        phase: 3.48944345784000e+00,
        frequency: 8.53196381752000e+02,
    },
    Vsop87Term {
        amplitude: 3.89840000000000e-07,
        phase: 2.79930428520000e+00,
        frequency: 6.32783739313200e+02,
    },
    Vsop87Term {
        amplitude: 2.76430000000000e-07,
        phase: 1.22439852303000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 2.48040000000000e-07,
        phase: 4.48123972552000e+00,
        frequency: 2.10117701700300e+02,
    },
    Vsop87Term {
        amplitude: 2.14980000000000e-07,
        phase: 5.38853499774000e+00,
        frequency: 4.40825284877600e+02,
    },
    Vsop87Term {
        amplitude: 1.57040000000000e-07,
        phase: 4.28129850675000e+00,
        frequency: 2.17231248701100e+02,
    },
    Vsop87Term {
        amplitude: 1.95380000000000e-07,
        phase: 5.81992746567000e+00,
        frequency: 2.16480489175700e+02,
    },
];

const SATURN_B_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 4.63357000000000e-06,
        phase: 1.69194209337000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 4.87242000000000e-06,
        phase: 5.57827705588000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 2.70686000000000e-06,
        phase: 4.65445792593000e+00,
        frequency: 2.20412642438800e+02,
    },
    Vsop87Term {
        amplitude: 2.77451000000000e-06,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 6.67180000000000e-07,
        phase: 3.66337287998000e+00,
        frequency: 4.33711737876800e+02,
    },
    Vsop87Term {
        amplitude: 6.56170000000000e-07,
        phase: 4.71263096227000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 6.98460000000000e-07,
        phase: 3.33236270677000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 3.05510000000000e-07,
        phase: 4.53651131935000e+00,
        frequency: 2.27526189439600e+02,
    },
    Vsop87Term {
        amplitude: 2.97040000000000e-07,
        phase: 2.49374065388000e+00,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 1.81570000000000e-07,
        phase: 5.89401285772000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 1.75040000000000e-07,
        phase: 5.79120992263000e+00,
        frequency: 1.99072001436400e+02,
    },
    Vsop87Term {
        amplitude: 1.16840000000000e-07,
        phase: 2.74773493978000e+00,
        frequency: 6.47010833314800e+02,
    },
    Vsop87Term {
        amplitude: 6.04800000000000e-08,
        phase: 5.80237729519000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 6.24800000000000e-08,
        phase: 1.60565634016000e+00,
        frequency: 8.53196381752000e+02,
    },
    Vsop87Term {
        amplitude: 6.42000000000000e-08,
        phase: 3.63996599914000e+00,
        frequency: 4.40825284877600e+02,
    },
    Vsop87Term {
        amplitude: 4.55200000000000e-08,
        phase: 6.21266119922000e+00,
        frequency: 2.10117701700300e+02,
    },
    Vsop87Term {
        amplitude: 4.99500000000000e-08,
        phase: 3.21953122449000e+00,
        frequency: 3.16391869656600e+02,
    },
    Vsop87Term {
        amplitude: 4.16600000000000e-08,
        phase: 4.64321479214000e+00,
        frequency: 6.32783739313200e+02,
    },
    Vsop87Term {
        amplitude: 2.93800000000000e-08,
        phase: 4.64767028200000e+00,
        frequency: 4.12371096874400e+02,
    },
    Vsop87Term {
        amplitude: 2.89400000000000e-08,
        phase: 4.02023147538000e+00,
        frequency: 2.16480489175700e+02,
    },
];

const SATURN_B_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 5.85210000000000e-07,
        phase: 9.64042696720000e-01,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 2.70230000000000e-07,
        phase: 2.97511812746000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 2.73450000000000e-07,
        phase: 2.90816987834000e+00,
        frequency: 2.20412642438800e+02,
    },
    Vsop87Term {
        amplitude: 8.70900000000000e-08,
        phase: 1.88638219079000e+00,
        frequency: 4.33711737876800e+02,
    },
    Vsop87Term {
        amplitude: 6.01500000000000e-08,
        phase: 2.81931276694000e+00,
        frequency: 2.27526189439600e+02,
    },
    Vsop87Term {
        amplitude: 6.05900000000000e-08,
        phase: 2.15765624750000e-01,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 3.79600000000000e-08,
        phase: 1.19723799579000e+00,
        frequency: 1.99072001436400e+02,
    },
    Vsop87Term {
        amplitude: 3.64700000000000e-08,
        phase: 1.71327650497000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 2.05400000000000e-08,
        phase: 6.64108945530000e-01,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 2.55900000000000e-08,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.86700000000000e-08,
        phase: 9.35787199250000e-01,
        frequency: 6.47010833314800e+02,
    },
    Vsop87Term {
        amplitude: 1.25600000000000e-08,
        phase: 4.13175992780000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 1.39900000000000e-08,
        phase: 1.88853247568000e+00,
        frequency: 4.40825284877600e+02,
    },
    Vsop87Term {
        amplitude: 9.36000000000000e-09,
        phase: 4.08790738476000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 6.18000000000000e-09,
        phase: 6.23879306520000e+00,
        frequency: 8.53196381752000e+02,
    },
    Vsop87Term {
        amplitude: 3.72000000000000e-09,
        phase: 2.71498257560000e-01,
        frequency: 4.12371096874400e+02,
    },
    Vsop87Term {
        amplitude: 3.32000000000000e-09,
        phase: 2.65677859091000e+00,
        frequency: 2.34639736440400e+02,
    },
    Vsop87Term {
        amplitude: 2.47000000000000e-09,
        phase: 2.56044748980000e-01,
        frequency: 6.32783739313200e+02,
    },
    Vsop87Term {
        amplitude: 2.12000000000000e-09,
        phase: 3.06313505900000e-02,
        frequency: 8.60309928752800e+02,
    },
    Vsop87Term {
        amplitude: 1.62000000000000e-09,
        phase: 4.96348549494000e+00,
        frequency: 1.17319868220200e+02,
    },
];

const SATURN_B_5: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 5.44200000000000e-08,
        phase: 2.61186488264000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 1.96600000000000e-08,
        phase: 1.16969532852000e+00,
        frequency: 2.20412642438800e+02,
    },
    Vsop87Term {
        amplitude: 9.07000000000000e-09,
        phase: 1.07715583710000e-01,
        frequency: 4.33711737876800e+02,
    },
    Vsop87Term {
        amplitude: 8.29000000000000e-09,
        phase: 1.07640059707000e+00,
        frequency: 2.27526189439600e+02,
    },
    Vsop87Term {
        amplitude: 5.84000000000000e-09,
        phase: 2.88210646011000e+00,
        frequency: 1.99072001436400e+02,
    },
    Vsop87Term {
        amplitude: 7.64000000000000e-09,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.77000000000000e-09,
        phase: 2.05357076014000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 1.51000000000000e-09,
        phase: 5.41582267800000e+00,
        frequency: 6.47010833314800e+02,
    },
    Vsop87Term {
        amplitude: 9.80000000000000e-10,
        phase: 1.68550159247000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 1.20000000000000e-09,
        phase: 1.08933118790000e-01,
        frequency: 4.40825284877600e+02,
    },
    Vsop87Term {
        amplitude: 8.10000000000000e-10,
        phase: 5.11373096610000e+00,
        frequency: 6.39897286314000e+02,
    },
];

const SATURN_R_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 9.55758135486000e+00,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 5.29213828650000e-01,
        phase: 2.39226219573000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 1.87367986700000e-02,
        phase: 5.23549604660000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 1.46466392900000e-02,
        phase: 1.64763042902000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 8.21891141000000e-03,
        phase: 5.93520042303000e+00,
        frequency: 3.16391869656600e+02,
    },
    Vsop87Term {
        amplitude: 5.47506923000000e-03,
        phase: 5.01532618980000e+00,
        frequency: 1.03092774218600e+02,
    },
    Vsop87Term {
        amplitude: 3.71684650000000e-03,
        phase: 2.27114821115000e+00,
        frequency: 2.20412642438800e+02,
    },
    Vsop87Term {
        amplitude: 3.61778765000000e-03,
        phase: 3.13904301847000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 1.40617506000000e-03,
        phase: 5.70406606781000e+00,
        frequency: 6.32783739313200e+02,
    },
    Vsop87Term {
        amplitude: 1.08974848000000e-03,
        phase: 3.29313390175000e+00,
        frequency: 1.10206321219400e+02,
    },
    Vsop87Term {
        amplitude: 6.90069620000000e-04,
        phase: 5.94099540992000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 6.10533670000000e-04,
        phase: 9.40376918010000e-01,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 4.89132940000000e-04,
        phase: 1.55733638681000e+00,
        frequency: 2.02253395174100e+02,
    },
    Vsop87Term {
        amplitude: 3.41437720000000e-04,
        phase: 1.95191025970000e-01,
        frequency: 2.77034993741400e+02,
    },
    Vsop87Term {
        amplitude: 3.24017730000000e-04,
        phase: 5.47084567016000e+00,
        frequency: 9.49175608969800e+02,
    },
    Vsop87Term {
        amplitude: 2.09365960000000e-04,
        phase: 4.63492511290000e-01,
        frequency: 7.35876513531800e+02,
    },
    Vsop87Term {
        amplitude: 2.08393000000000e-04,
        phase: 1.52102476129000e+00,
        frequency: 4.33711737876800e+02,
    },
    Vsop87Term {
        amplitude: 2.07467510000000e-04,
        phase: 5.33255457763000e+00,
        frequency: 1.99072001436400e+02,
    },
    Vsop87Term {
        amplitude: 1.52984040000000e-04,
        phase: 3.05943814940000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 1.42964840000000e-04,
        phase: 2.60433479142000e+00,
        frequency: 3.23505416657400e+02,
    },
];

const SATURN_R_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 6.18298134000000e-02,
        phase: 2.58435114800000e-01,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 5.06577242000000e-03,
        phase: 7.11146252610000e-01,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 3.41394029000000e-03,
        phase: 5.79635741658000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 1.88491195000000e-03,
        phase: 4.72155896520000e-01,
        frequency: 2.20412642438800e+02,
    },
    Vsop87Term {
        amplitude: 1.86261486000000e-03,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.43891146000000e-03,
        phase: 1.40744822888000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 4.96212080000000e-04,
        phase: 6.01744279820000e+00,
        frequency: 1.03092774218600e+02,
    },
    Vsop87Term {
        amplitude: 2.09284260000000e-04,
        phase: 5.09244947411000e+00,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 1.99525640000000e-04,
        phase: 1.17560606130000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 1.88395440000000e-04,
        phase: 1.60818334043000e+00,
        frequency: 1.10206321219400e+02,
    },
    Vsop87Term {
        amplitude: 1.28928430000000e-04,
        phase: 5.94329433020000e+00,
        frequency: 4.33711737876800e+02,
    },
    Vsop87Term {
        amplitude: 1.38768490000000e-04,
        phase: 7.58849288660000e-01,
        frequency: 1.99072001436400e+02,
    },
    Vsop87Term {
        amplitude: 5.39684200000000e-05,
        phase: 1.28853589711000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 4.86928900000000e-05,
        phase: 8.67972270540000e-01,
        frequency: 3.23505416657400e+02,
    },
    Vsop87Term {
        amplitude: 4.24722100000000e-05,
        phase: 3.92949847320000e-01,
        frequency: 2.27526189439600e+02,
    },
    Vsop87Term {
        amplitude: 3.25233100000000e-05,
        phase: 1.25850154330000e+00,
        frequency: 9.59792272178000e+01,
    },
    Vsop87Term {
        amplitude: 2.85606600000000e-05,
        phase: 2.16731283870000e+00,
        frequency: 7.35876513531800e+02,
    },
    Vsop87Term {
        amplitude: 2.90954000000000e-05,
        phase: 4.60680719251000e+00,
        frequency: 2.02253395174100e+02,
    },
    Vsop87Term {
        amplitude: 3.08141000000000e-05,
        phase: 3.43662543526000e+00,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 1.98773100000000e-05,
        phase: 2.45053765034000e+00,
        frequency: 4.12371096874400e+02,
    },
];

const SATURN_R_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 4.36902572000000e-03,
        phase: 4.78671677509000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 7.19224980000000e-04,
        phase: 2.50070069930000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 4.97668720000000e-04,
        phase: 4.97167777235000e+00,
        frequency: 2.20412642438800e+02,
    },
    Vsop87Term {
        amplitude: 4.32207830000000e-04,
        phase: 3.86941044212000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 2.96457660000000e-04,
        phase: 5.96309886479000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 4.14168700000000e-05,
        phase: 4.10673009419000e+00,
        frequency: 4.33711737876800e+02,
    },
    Vsop87Term {
        amplitude: 4.72082200000000e-05,
        phase: 2.47524028389000e+00,
        frequency: 1.99072001436400e+02,
    },
    Vsop87Term {
        amplitude: 3.78932100000000e-05,
        phase: 3.09771189740000e+00,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 2.96398100000000e-05,
        phase: 1.37198670946000e+00,
        frequency: 1.03092774218600e+02,
    },
    Vsop87Term {
        amplitude: 2.55640300000000e-05,
        phase: 2.85066948131000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 2.20847300000000e-05,
        phase: 6.27590108662000e+00,
        frequency: 1.10206321219400e+02,
    },
    Vsop87Term {
        amplitude: 2.18731100000000e-05,
        phase: 5.85545017140000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 1.95677900000000e-05,
        phase: 4.92451269861000e+00,
        frequency: 2.27526189439600e+02,
    },
    Vsop87Term {
        amplitude: 2.32677700000000e-05,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 9.23829000000000e-06,
        phase: 5.46389688910000e+00,
        frequency: 3.23505416657400e+02,
    },
    Vsop87Term {
        amplitude: 7.05974000000000e-06,
        phase: 2.97065900638000e+00,
        frequency: 9.59792272178000e+01,
    },
    Vsop87Term {
        amplitude: 5.45943000000000e-06,
        phase: 4.12843012325000e+00,
        frequency: 4.12371096874400e+02,
    },
    Vsop87Term {
        amplitude: 3.73763000000000e-06,
        phase: 5.83412146980000e+00,
        frequency: 1.17319868220200e+02,
    },
    Vsop87Term {
        amplitude: 3.60843000000000e-06,
        phase: 3.27730304283000e+00,
        frequency: 6.47010833314800e+02,
    },
    Vsop87Term {
        amplitude: 3.56448000000000e-06,
        phase: 3.19046275776000e+00,
        frequency: 2.10117701700300e+02,
    },
];

const SATURN_R_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.03152390000000e-04,
        phase: 3.02186068237000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 8.92367900000000e-05,
        phase: 3.19144467228000e+00,
        frequency: 2.20412642438800e+02,
    },
    Vsop87Term {
        amplitude: 6.90876800000000e-05,
        phase: 4.35175288182000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 4.08705600000000e-05,
        phase: 4.22398596149000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 3.87884800000000e-05,
        phase: 2.01051759517000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 1.07075400000000e-05,
        phase: 4.20372656114000e+00,
        frequency: 1.99072001436400e+02,
    },
    Vsop87Term {
        amplitude: 9.07379000000000e-06,
        phase: 2.28356519128000e+00,
        frequency: 4.33711737876800e+02,
    },
    Vsop87Term {
        amplitude: 6.05936000000000e-06,
        phase: 3.17456913264000e+00,
        frequency: 2.27526189439600e+02,
    },
    Vsop87Term {
        amplitude: 5.96411000000000e-06,
        phase: 4.13395467306000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 4.83108000000000e-06,
        phase: 1.17313249713000e+00,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 3.93213000000000e-06,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 2.29396000000000e-06,
        phase: 4.69783424016000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 1.87917000000000e-06,
        phase: 4.59089264920000e+00,
        frequency: 1.10206321219400e+02,
    },
    Vsop87Term {
        amplitude: 1.49326000000000e-06,
        phase: 3.20334759568000e+00,
        frequency: 1.03092774218600e+02,
    },
    Vsop87Term {
        amplitude: 1.21613000000000e-06,
        phase: 3.76751430846000e+00,
        frequency: 3.23505416657400e+02,
    },
    Vsop87Term {
        amplitude: 1.01300000000000e-06,
        phase: 5.81716272185000e+00,
        frequency: 4.12371096874400e+02,
    },
    Vsop87Term {
        amplitude: 1.02030000000000e-06,
        phase: 4.70997918436000e+00,
        frequency: 9.59792272178000e+01,
    },
    Vsop87Term {
        amplitude: 9.27370000000000e-07,
        phase: 1.43601934858000e+00,
        frequency: 6.47010833314800e+02,
    },
    Vsop87Term {
        amplitude: 7.24110000000000e-07,
        phase: 4.15100432048000e+00,
        frequency: 1.17319868220200e+02,
    },
    Vsop87Term {
        amplitude: 8.41970000000000e-07,
        phase: 2.63457296718000e+00,
        frequency: 2.16480489175700e+02,
    },
];

const SATURN_R_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.20211700000000e-05,
        phase: 1.41498340225000e+00,
        frequency: 2.20412642438800e+02,
    },
    Vsop87Term {
        amplitude: 7.07794000000000e-06,
        phase: 1.16151449537000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 5.16224000000000e-06,
        phase: 6.24049105350000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 4.26107000000000e-06,
        phase: 2.46891791825000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 2.67495000000000e-06,
        phase: 1.86447168750000e-01,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 1.70055000000000e-06,
        phase: 5.96000580678000e+00,
        frequency: 1.99072001436400e+02,
    },
    Vsop87Term {
        amplitude: 1.44813000000000e-06,
        phase: 1.44265291294000e+00,
        frequency: 2.27526189439600e+02,
    },
    Vsop87Term {
        amplitude: 1.50056000000000e-06,
        phase: 4.79681863810000e-01,
        frequency: 4.33711737876800e+02,
    },
    Vsop87Term {
        amplitude: 1.21067000000000e-06,
        phase: 2.40476128629000e+00,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 4.75030000000000e-07,
        phase: 5.56874777537000e+00,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 1.56510000000000e-07,
        phase: 2.89076603229000e+00,
        frequency: 1.10206321219400e+02,
    },
    Vsop87Term {
        amplitude: 1.63640000000000e-07,
        phase: 5.39287924150000e-01,
        frequency: 4.40825284877600e+02,
    },
    Vsop87Term {
        amplitude: 1.89730000000000e-07,
        phase: 5.85514753020000e+00,
        frequency: 6.47010833314800e+02,
    },
    Vsop87Term {
        amplitude: 1.44900000000000e-07,
        phase: 1.31356947305000e+00,
        frequency: 4.12371096874400e+02,
    },
    Vsop87Term {
        amplitude: 1.23100000000000e-07,
        phase: 2.10618416544000e+00,
        frequency: 3.23505416657400e+02,
    },
    Vsop87Term {
        amplitude: 1.46880000000000e-07,
        phase: 2.96859659490000e-01,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 1.08930000000000e-07,
        phase: 2.45288604864000e+00,
        frequency: 1.17319868220200e+02,
    },
    Vsop87Term {
        amplitude: 1.13480000000000e-07,
        phase: 1.74903122780000e-01,
        frequency: 9.59792272178000e+01,
    },
    Vsop87Term {
        amplitude: 9.21000000000000e-08,
        phase: 2.30108004686000e+00,
        frequency: 2.13406410024000e+01,
    },
    Vsop87Term {
        amplitude: 9.00700000000000e-08,
        phase: 1.55461797818000e+00,
        frequency: 8.88656802170000e+01,
    },
];

const SATURN_R_5: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.28668000000000e-06,
        phase: 5.91279864289000e+00,
        frequency: 2.20412642438800e+02,
    },
    Vsop87Term {
        amplitude: 3.21960000000000e-07,
        phase: 6.95582843840000e-01,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 2.67370000000000e-07,
        phase: 5.91270395039000e+00,
        frequency: 2.27526189439600e+02,
    },
    Vsop87Term {
        amplitude: 1.98370000000000e-07,
        phase: 6.73968529600000e-01,
        frequency: 1.42270940016000e+01,
    },
    Vsop87Term {
        amplitude: 1.99940000000000e-07,
        phase: 4.95031713518000e+00,
        frequency: 4.33711737876800e+02,
    },
    Vsop87Term {
        amplitude: 1.36270000000000e-07,
        phase: 1.47747814594000e+00,
        frequency: 1.99072001436400e+02,
    },
    Vsop87Term {
        amplitude: 1.37060000000000e-07,
        phase: 4.59824754628000e+00,
        frequency: 4.26598190876000e+02,
    },
    Vsop87Term {
        amplitude: 1.40680000000000e-07,
        phase: 2.63892426573000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 7.32400000000000e-08,
        phase: 4.64667642371000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 4.91600000000000e-08,
        phase: 3.63019930267000e+00,
        frequency: 6.39897286314000e+02,
    },
    Vsop87Term {
        amplitude: 2.98500000000000e-08,
        phase: 4.64378755577000e+00,
        frequency: 1.91958454435600e+02,
    },
    Vsop87Term {
        amplitude: 2.67500000000000e-08,
        phase: 5.17420576470000e-01,
        frequency: 3.23505416657400e+02,
    },
    Vsop87Term {
        amplitude: 3.42000000000000e-08,
        phase: 4.91489841099000e+00,
        frequency: 4.40825284877600e+02,
    },
    Vsop87Term {
        amplitude: 3.17100000000000e-08,
        phase: 4.10118061147000e+00,
        frequency: 6.47010833314800e+02,
    },
    Vsop87Term {
        amplitude: 2.88500000000000e-08,
        phase: 3.24108476164000e+00,
        frequency: 4.19484643875200e+02,
    },
    Vsop87Term {
        amplitude: 2.17300000000000e-08,
        phase: 5.39877301813000e+00,
        frequency: 3.02164775655000e+02,
    },
    Vsop87Term {
        amplitude: 1.87300000000000e-08,
        phase: 3.22101902976000e+00,
        frequency: 9.59792272178000e+01,
    },
    Vsop87Term {
        amplitude: 2.05500000000000e-08,
        phase: 3.60842101774000e+00,
        frequency: 8.88656802170000e+01,
    },
    Vsop87Term {
        amplitude: 1.50900000000000e-08,
        phase: 2.68946095921000e+00,
        frequency: 8.53196381752000e+02,
    },
    Vsop87Term {
        amplitude: 1.51800000000000e-08,
        phase: 8.96924314390000e-01,
        frequency: 5.15463871093000e+02,
    },
];
