//! Truncated VSOP87B Neptune coefficient tables.
//!
//! Coefficients are the leading terms from the public IMCCE/CELMECH
//! `VSOP87B.nep` file (Neptune heliocentric spherical variables, J2000
//! ecliptic/equinox). This mirrors the other checked-in planetary slices
//! while complete generated tables are still planned.

use crate::vsop87b_earth::{evaluate, SphericalLbr, Vsop87Term};

pub(crate) fn neptune_lbr(julian_day_tt: f64) -> SphericalLbr {
    let t = (julian_day_tt - 2_451_545.0) / 365_250.0;
    SphericalLbr {
        longitude_rad: evaluate(
            [
                NEPTUNE_L_0,
                NEPTUNE_L_1,
                NEPTUNE_L_2,
                NEPTUNE_L_3,
                NEPTUNE_L_4,
                NEPTUNE_L_5,
            ],
            t,
        )
        .rem_euclid(core::f64::consts::TAU),
        latitude_rad: evaluate(
            [
                NEPTUNE_B_0,
                NEPTUNE_B_1,
                NEPTUNE_B_2,
                NEPTUNE_B_3,
                NEPTUNE_B_4,
                NEPTUNE_B_5,
            ],
            t,
        ),
        radius_au: evaluate(
            [
                NEPTUNE_R_0,
                NEPTUNE_R_1,
                NEPTUNE_R_2,
                NEPTUNE_R_3,
                NEPTUNE_R_4,
                NEPTUNE_R_5,
            ],
            t,
        ),
    }
}

const NEPTUNE_L_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 5.31188633046000e+00,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.79847553000000e-02,
        phase: 2.90101273890000e+00,
        frequency: 3.81330356378000e+01,
    },
    Vsop87Term {
        amplitude: 1.01972765200000e-02,
        phase: 4.85809228670000e-01,
        frequency: 1.48447270830000e+00,
    },
    Vsop87Term {
        amplitude: 1.24531845000000e-03,
        phase: 4.83008090676000e+00,
        frequency: 3.66485629295000e+01,
    },
    Vsop87Term {
        amplitude: 4.20644660000000e-04,
        phase: 5.41054993053000e+00,
        frequency: 2.96894541660000e+00,
    },
    Vsop87Term {
        amplitude: 3.77145840000000e-04,
        phase: 6.09221808686000e+00,
        frequency: 3.51640902212000e+01,
    },
    Vsop87Term {
        amplitude: 3.37847380000000e-04,
        phase: 1.24488874087000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 1.64827410000000e-04,
        phase: 7.72799800000000e-05,
        frequency: 4.91557929456800e+02,
    },
    Vsop87Term {
        amplitude: 9.19858400000000e-05,
        phase: 4.93747051954000e+00,
        frequency: 3.96175083461000e+01,
    },
    Vsop87Term {
        amplitude: 8.99425000000000e-05,
        phase: 2.74621718060000e-01,
        frequency: 1.75166059800200e+02,
    },
    Vsop87Term {
        amplitude: 4.21624200000000e-05,
        phase: 1.98711875978000e+00,
        frequency: 7.32971258590000e+01,
    },
    Vsop87Term {
        amplitude: 3.36480700000000e-05,
        phase: 1.03590060915000e+00,
        frequency: 3.36796175129000e+01,
    },
    Vsop87Term {
        amplitude: 2.28480000000000e-05,
        phase: 4.20606949415000e+00,
        frequency: 4.45341812490000e+00,
    },
    Vsop87Term {
        amplitude: 1.43351600000000e-05,
        phase: 2.78339802539000e+00,
        frequency: 7.47815985673000e+01,
    },
    Vsop87Term {
        amplitude: 9.00236000000000e-06,
        phase: 2.07607168714000e+00,
        frequency: 1.09945688788500e+02,
    },
    Vsop87Term {
        amplitude: 7.44997000000000e-06,
        phase: 3.19032509437000e+00,
        frequency: 7.18126531507000e+01,
    },
    Vsop87Term {
        amplitude: 5.06217000000000e-06,
        phase: 5.74786069680000e+00,
        frequency: 1.14399106913400e+02,
    },
    Vsop87Term {
        amplitude: 3.99552000000000e-06,
        phase: 3.49723428360000e-01,
        frequency: 1.02124889455140e+03,
    },
    Vsop87Term {
        amplitude: 3.45189000000000e-06,
        phase: 3.46185292806000e+00,
        frequency: 4.11019810544000e+01,
    },
    Vsop87Term {
        amplitude: 3.06338000000000e-06,
        phase: 4.96840529340000e-01,
        frequency: 5.21264861800000e-01,
    },
];

const NEPTUNE_L_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 3.81330356395700e+01,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.66041720000000e-04,
        phase: 4.86323329249000e+00,
        frequency: 1.48447270830000e+00,
    },
    Vsop87Term {
        amplitude: 1.57440450000000e-04,
        phase: 2.27887427527000e+00,
        frequency: 3.81330356378000e+01,
    },
    Vsop87Term {
        amplitude: 1.30626100000000e-05,
        phase: 3.67285209620000e+00,
        frequency: 2.96894541660000e+00,
    },
    Vsop87Term {
        amplitude: 6.04842000000000e-06,
        phase: 1.50483042790000e+00,
        frequency: 3.51640902212000e+01,
    },
    Vsop87Term {
        amplitude: 1.82909000000000e-06,
        phase: 3.45225794434000e+00,
        frequency: 3.96175083461000e+01,
    },
    Vsop87Term {
        amplitude: 1.95106000000000e-06,
        phase: 8.86603260880000e-01,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 1.06410000000000e-06,
        phase: 2.44986610969000e+00,
        frequency: 4.45341812490000e+00,
    },
    Vsop87Term {
        amplitude: 1.05590000000000e-06,
        phase: 2.75516054635000e+00,
        frequency: 3.36796175129000e+01,
    },
    Vsop87Term {
        amplitude: 7.27570000000000e-07,
        phase: 5.49395347003000e+00,
        frequency: 3.66485629295000e+01,
    },
    Vsop87Term {
        amplitude: 5.70690000000000e-07,
        phase: 5.21649804970000e+00,
        frequency: 5.21264861800000e-01,
    },
    Vsop87Term {
        amplitude: 2.98710000000000e-07,
        phase: 3.67043294114000e+00,
        frequency: 3.88465155238200e+02,
    },
    Vsop87Term {
        amplitude: 2.88660000000000e-07,
        phase: 5.16877538898000e+00,
        frequency: 9.56122755560000e+00,
    },
    Vsop87Term {
        amplitude: 2.87420000000000e-07,
        phase: 5.16732589024000e+00,
        frequency: 2.44768055480000e+00,
    },
    Vsop87Term {
        amplitude: 2.55070000000000e-07,
        phase: 5.24526281928000e+00,
        frequency: 1.68052512799400e+02,
    },
    Vsop87Term {
        amplitude: 2.48690000000000e-07,
        phase: 4.73193067879000e+00,
        frequency: 1.82279606801000e+02,
    },
    Vsop87Term {
        amplitude: 2.02050000000000e-07,
        phase: 5.78945415677000e+00,
        frequency: 1.02124889455140e+03,
    },
    Vsop87Term {
        amplitude: 1.90220000000000e-07,
        phase: 1.82981144269000e+00,
        frequency: 4.84444382456000e+02,
    },
    Vsop87Term {
        amplitude: 1.86610000000000e-07,
        phase: 1.31606255521000e+00,
        frequency: 4.98671476457600e+02,
    },
    Vsop87Term {
        amplitude: 1.50390000000000e-07,
        phase: 4.94966181697000e+00,
        frequency: 1.37033024162400e+02,
    },
];

const NEPTUNE_L_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.86136000000000e-06,
        phase: 1.18985661922000e+00,
        frequency: 3.81330356378000e+01,
    },
    Vsop87Term {
        amplitude: 2.95650000000000e-06,
        phase: 1.85520880574000e+00,
        frequency: 1.48447270830000e+00,
    },
    Vsop87Term {
        amplitude: 1.02284000000000e-06,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 2.29870000000000e-07,
        phase: 1.21060882957000e+00,
        frequency: 2.96894541660000e+00,
    },
    Vsop87Term {
        amplitude: 7.33200000000000e-08,
        phase: 5.39827180120000e-01,
        frequency: 2.44768055480000e+00,
    },
    Vsop87Term {
        amplitude: 9.11200000000000e-08,
        phase: 4.42541280638000e+00,
        frequency: 3.51640902212000e+01,
    },
    Vsop87Term {
        amplitude: 5.22300000000000e-08,
        phase: 6.74222375270000e-01,
        frequency: 1.68052512799400e+02,
    },
    Vsop87Term {
        amplitude: 5.20100000000000e-08,
        phase: 3.02334762854000e+00,
        frequency: 1.82279606801000e+02,
    },
    Vsop87Term {
        amplitude: 3.92500000000000e-08,
        phase: 3.53215364421000e+00,
        frequency: 4.84444382456000e+02,
    },
    Vsop87Term {
        amplitude: 3.74100000000000e-08,
        phase: 5.90239568618000e+00,
        frequency: 4.98671476457600e+02,
    },
    Vsop87Term {
        amplitude: 3.05400000000000e-08,
        phase: 2.88982692370000e-01,
        frequency: 4.45341812490000e+00,
    },
    Vsop87Term {
        amplitude: 3.38200000000000e-08,
        phase: 5.91086982903000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 3.28900000000000e-08,
        phase: 1.84550132467000e+00,
        frequency: 1.75166059800200e+02,
    },
    Vsop87Term {
        amplitude: 2.15700000000000e-08,
        phase: 1.89134644831000e+00,
        frequency: 3.88465155238200e+02,
    },
    Vsop87Term {
        amplitude: 2.21100000000000e-08,
        phase: 4.37947574774000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 1.95500000000000e-08,
        phase: 5.15138892758000e+00,
        frequency: 3.36796175129000e+01,
    },
    Vsop87Term {
        amplitude: 1.84700000000000e-08,
        phase: 3.48560457075000e+00,
        frequency: 9.56122755560000e+00,
    },
    Vsop87Term {
        amplitude: 2.43600000000000e-08,
        phase: 4.68322560973000e+00,
        frequency: 4.91557929456800e+02,
    },
    Vsop87Term {
        amplitude: 1.67400000000000e-08,
        phase: 2.55582666306000e+00,
        frequency: 3.66485629295000e+01,
    },
    Vsop87Term {
        amplitude: 1.30900000000000e-08,
        phase: 4.52441960698000e+00,
        frequency: 1.02124889455140e+03,
    },
];

const NEPTUNE_L_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.24720000000000e-07,
        phase: 6.04427218715000e+00,
        frequency: 1.48447270830000e+00,
    },
    Vsop87Term {
        amplitude: 1.12570000000000e-07,
        phase: 6.11436681584000e+00,
        frequency: 3.81330356378000e+01,
    },
    Vsop87Term {
        amplitude: 4.35400000000000e-08,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.39000000000000e-08,
        phase: 4.95198243861000e+00,
        frequency: 2.96894541660000e+00,
    },
    Vsop87Term {
        amplitude: 7.45000000000000e-09,
        phase: 2.37751238105000e+00,
        frequency: 1.68052512799400e+02,
    },
    Vsop87Term {
        amplitude: 7.10000000000000e-09,
        phase: 1.29216892369000e+00,
        frequency: 1.82279606801000e+02,
    },
    Vsop87Term {
        amplitude: 5.40000000000000e-09,
        phase: 5.25465584672000e+00,
        frequency: 4.84444382456000e+02,
    },
    Vsop87Term {
        amplitude: 5.20000000000000e-09,
        phase: 4.18622893104000e+00,
        frequency: 4.98671476457600e+02,
    },
    Vsop87Term {
        amplitude: 6.64000000000000e-09,
        phase: 5.58714358770000e-01,
        frequency: 3.10194886370000e+01,
    },
    Vsop87Term {
        amplitude: 3.01000000000000e-09,
        phase: 2.69253200796000e+00,
        frequency: 7.11354700080000e+00,
    },
    Vsop87Term {
        amplitude: 1.92000000000000e-09,
        phase: 2.01562375989000e+00,
        frequency: 1.37033024162400e+02,
    },
    Vsop87Term {
        amplitude: 1.68000000000000e-09,
        phase: 6.26790496230000e+00,
        frequency: 3.57445666601200e+02,
    },
    Vsop87Term {
        amplitude: 1.83000000000000e-09,
        phase: 4.12898383544000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 1.44000000000000e-09,
        phase: 2.84518337934000e+00,
        frequency: 4.60538440819800e+02,
    },
    Vsop87Term {
        amplitude: 1.44000000000000e-09,
        phase: 3.93173694550000e+00,
        frequency: 4.46311346818200e+02,
    },
    Vsop87Term {
        amplitude: 1.08000000000000e-09,
        phase: 3.45112519708000e+00,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 1.08000000000000e-09,
        phase: 2.36456446266000e+00,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 8.50000000000000e-10,
        phase: 4.34146183645000e+00,
        frequency: 4.33711737876800e+02,
    },
];

const NEPTUNE_L_4: &[Vsop87Term] = &[];

const NEPTUNE_L_5: &[Vsop87Term] = &[];

const NEPTUNE_B_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 3.08862293300000e-02,
        phase: 1.44104372644000e+00,
        frequency: 3.81330356378000e+01,
    },
    Vsop87Term {
        amplitude: 2.77800870000000e-04,
        phase: 5.91271884599000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 2.76236090000000e-04,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.53554890000000e-04,
        phase: 2.52123799551000e+00,
        frequency: 3.66485629295000e+01,
    },
    Vsop87Term {
        amplitude: 1.54481330000000e-04,
        phase: 3.50877079215000e+00,
        frequency: 3.96175083461000e+01,
    },
    Vsop87Term {
        amplitude: 1.99991800000000e-05,
        phase: 1.50998668632000e+00,
        frequency: 7.47815985673000e+01,
    },
    Vsop87Term {
        amplitude: 1.96754000000000e-05,
        phase: 4.37778196626000e+00,
        frequency: 1.48447270830000e+00,
    },
    Vsop87Term {
        amplitude: 1.01513700000000e-05,
        phase: 3.21560997434000e+00,
        frequency: 3.51640902212000e+01,
    },
    Vsop87Term {
        amplitude: 6.05767000000000e-06,
        phase: 2.80246592015000e+00,
        frequency: 7.32971258590000e+01,
    },
    Vsop87Term {
        amplitude: 5.94878000000000e-06,
        phase: 2.12892696997000e+00,
        frequency: 4.11019810544000e+01,
    },
    Vsop87Term {
        amplitude: 5.88806000000000e-06,
        phase: 3.18655898167000e+00,
        frequency: 2.96894541660000e+00,
    },
    Vsop87Term {
        amplitude: 4.01830000000000e-06,
        phase: 4.16883411107000e+00,
        frequency: 1.14399106913400e+02,
    },
    Vsop87Term {
        amplitude: 2.54333000000000e-06,
        phase: 3.27120475878000e+00,
        frequency: 4.53424893819000e+02,
    },
    Vsop87Term {
        amplitude: 2.61647000000000e-06,
        phase: 3.76722702982000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 2.79963000000000e-06,
        phase: 1.68165289071000e+00,
        frequency: 7.77505439839000e+01,
    },
    Vsop87Term {
        amplitude: 2.05590000000000e-06,
        phase: 4.25652269561000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 1.40455000000000e-06,
        phase: 3.52969120587000e+00,
        frequency: 1.37033024162400e+02,
    },
    Vsop87Term {
        amplitude: 9.85300000000000e-07,
        phase: 4.16774786185000e+00,
        frequency: 3.36796175129000e+01,
    },
    Vsop87Term {
        amplitude: 5.12570000000000e-07,
        phase: 1.95120897519000e+00,
        frequency: 4.45341812490000e+00,
    },
    Vsop87Term {
        amplitude: 6.79710000000000e-07,
        phase: 4.66970488716000e+00,
        frequency: 7.18126531507000e+01,
    },
];

const NEPTUNE_B_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 5.15089700000000e-05,
        phase: 2.14270496419000e+00,
        frequency: 3.81330356378000e+01,
    },
    Vsop87Term {
        amplitude: 2.58298000000000e-06,
        phase: 5.46539598920000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 2.51862000000000e-06,
        phase: 4.40444268588000e+00,
        frequency: 3.66485629295000e+01,
    },
    Vsop87Term {
        amplitude: 2.34436000000000e-06,
        phase: 1.65983511437000e+00,
        frequency: 3.96175083461000e+01,
    },
    Vsop87Term {
        amplitude: 2.08814000000000e-06,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 2.53120000000000e-07,
        phase: 6.00917621033000e+00,
        frequency: 3.51640902212000e+01,
    },
    Vsop87Term {
        amplitude: 1.77950000000000e-07,
        phase: 4.95721064558000e+00,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 1.78410000000000e-07,
        phase: 4.48210000480000e-01,
        frequency: 4.11019810544000e+01,
    },
    Vsop87Term {
        amplitude: 1.31520000000000e-07,
        phase: 1.49958304388000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 1.07290000000000e-07,
        phase: 4.39946094022000e+00,
        frequency: 7.32971258590000e+01,
    },
    Vsop87Term {
        amplitude: 8.42200000000000e-08,
        phase: 1.55833887152000e+00,
        frequency: 2.96894541660000e+00,
    },
    Vsop87Term {
        amplitude: 5.65000000000000e-08,
        phase: 2.15782280490000e+00,
        frequency: 7.47815985673000e+01,
    },
    Vsop87Term {
        amplitude: 5.16400000000000e-08,
        phase: 3.68693766150000e+00,
        frequency: 1.14399106913400e+02,
    },
    Vsop87Term {
        amplitude: 5.43900000000000e-08,
        phase: 2.67475465140000e-01,
        frequency: 7.77505439839000e+01,
    },
    Vsop87Term {
        amplitude: 4.88300000000000e-08,
        phase: 5.47281451218000e+00,
        frequency: 1.48447270830000e+00,
    },
    Vsop87Term {
        amplitude: 1.86300000000000e-08,
        phase: 5.45718239014000e+00,
        frequency: 7.18126531507000e+01,
    },
    Vsop87Term {
        amplitude: 1.95600000000000e-08,
        phase: 5.93949937080000e-01,
        frequency: 3.36796175129000e+01,
    },
    Vsop87Term {
        amplitude: 1.48600000000000e-08,
        phase: 2.69790200450000e-01,
        frequency: 4.45341812490000e+00,
    },
    Vsop87Term {
        amplitude: 1.48200000000000e-08,
        phase: 5.46695876828000e+00,
        frequency: 4.25864537627000e+01,
    },
    Vsop87Term {
        amplitude: 8.81000000000000e-09,
        phase: 4.07805940363000e+00,
        frequency: 3.76117707760000e+01,
    },
];

const NEPTUNE_B_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 4.20580000000000e-07,
        phase: 1.91480759314000e+00,
        frequency: 3.81330356378000e+01,
    },
    Vsop87Term {
        amplitude: 4.35900000000000e-08,
        phase: 4.77459417163000e+00,
        frequency: 3.96175083461000e+01,
    },
    Vsop87Term {
        amplitude: 4.23000000000000e-08,
        phase: 1.12991232222000e+00,
        frequency: 3.66485629295000e+01,
    },
    Vsop87Term {
        amplitude: 4.16600000000000e-08,
        phase: 4.37185631758000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 1.23500000000000e-08,
        phase: 1.30364552320000e-01,
        frequency: 2.13299095438000e+02,
    },
    Vsop87Term {
        amplitude: 9.05000000000000e-09,
        phase: 3.05640125957000e+00,
        frequency: 5.29690965094600e+02,
    },
    Vsop87Term {
        amplitude: 7.43000000000000e-09,
        phase: 3.14159265359000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 3.80000000000000e-09,
        phase: 1.70771699530000e+00,
        frequency: 3.51640902212000e+01,
    },
    Vsop87Term {
        amplitude: 3.28000000000000e-09,
        phase: 4.23287817216000e+00,
        frequency: 4.11019810544000e+01,
    },
    Vsop87Term {
        amplitude: 1.88000000000000e-09,
        phase: 1.15191208244000e+00,
        frequency: 2.20412642438800e+02,
    },
    Vsop87Term {
        amplitude: 1.32000000000000e-09,
        phase: 4.73251124812000e+00,
        frequency: 2.06185548437200e+02,
    },
    Vsop87Term {
        amplitude: 8.40000000000000e-10,
        phase: 3.93928282212000e+00,
        frequency: 1.29919477161600e+02,
    },
    Vsop87Term {
        amplitude: 8.00000000000000e-10,
        phase: 1.15127170600000e-02,
        frequency: 1.44146571163200e+02,
    },
];

const NEPTUNE_B_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 4.13100000000000e-08,
        phase: 3.06928911462000e+00,
        frequency: 3.81330356378000e+01,
    },
    Vsop87Term {
        amplitude: 1.43000000000000e-09,
        phase: 4.00453590187000e+00,
        frequency: 3.66485629295000e+01,
    },
];

const NEPTUNE_B_4: &[Vsop87Term] = &[];

const NEPTUNE_B_5: &[Vsop87Term] = &[];

const NEPTUNE_R_0: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 3.00701320582800e+01,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 2.70622596320000e-01,
        phase: 1.32999459377000e+00,
        frequency: 3.81330356378000e+01,
    },
    Vsop87Term {
        amplitude: 1.69176401400000e-02,
        phase: 3.25186135653000e+00,
        frequency: 3.66485629295000e+01,
    },
    Vsop87Term {
        amplitude: 8.07830553000000e-03,
        phase: 5.18592878704000e+00,
        frequency: 1.48447270830000e+00,
    },
    Vsop87Term {
        amplitude: 5.37760510000000e-03,
        phase: 4.52113935896000e+00,
        frequency: 3.51640902212000e+01,
    },
    Vsop87Term {
        amplitude: 4.95725141000000e-03,
        phase: 1.57105641650000e+00,
        frequency: 4.91557929456800e+02,
    },
    Vsop87Term {
        amplitude: 2.74571975000000e-03,
        phase: 1.84552258866000e+00,
        frequency: 1.75166059800200e+02,
    },
    Vsop87Term {
        amplitude: 1.35134092000000e-03,
        phase: 3.37220609835000e+00,
        frequency: 3.96175083461000e+01,
    },
    Vsop87Term {
        amplitude: 1.21801746000000e-03,
        phase: 5.79754470298000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 1.00896068000000e-03,
        phase: 3.77027249300000e-01,
        frequency: 7.32971258590000e+01,
    },
    Vsop87Term {
        amplitude: 6.97913310000000e-04,
        phase: 3.79616637768000e+00,
        frequency: 2.96894541660000e+00,
    },
    Vsop87Term {
        amplitude: 4.66878360000000e-04,
        phase: 5.74938034313000e+00,
        frequency: 3.36796175129000e+01,
    },
    Vsop87Term {
        amplitude: 2.45945310000000e-04,
        phase: 5.08017458780000e-01,
        frequency: 1.09945688788500e+02,
    },
    Vsop87Term {
        amplitude: 1.69394780000000e-04,
        phase: 1.59422512526000e+00,
        frequency: 7.18126531507000e+01,
    },
    Vsop87Term {
        amplitude: 1.42298080000000e-04,
        phase: 1.07785898723000e+00,
        frequency: 7.47815985673000e+01,
    },
    Vsop87Term {
        amplitude: 1.20123200000000e-04,
        phase: 1.92059384991000e+00,
        frequency: 1.02124889455140e+03,
    },
    Vsop87Term {
        amplitude: 8.39434900000000e-05,
        phase: 6.78182335860000e-01,
        frequency: 1.46594251718000e+02,
    },
    Vsop87Term {
        amplitude: 7.57179600000000e-05,
        phase: 1.07149207335000e+00,
        frequency: 3.88465155238200e+02,
    },
    Vsop87Term {
        amplitude: 5.72087200000000e-05,
        phase: 2.59061733345000e+00,
        frequency: 4.45341812490000e+00,
    },
    Vsop87Term {
        amplitude: 4.84021000000000e-05,
        phase: 1.90681013048000e+00,
        frequency: 4.11019810544000e+01,
    },
];

const NEPTUNE_R_1: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 2.36338618000000e-03,
        phase: 7.04979547920000e-01,
        frequency: 3.81330356378000e+01,
    },
    Vsop87Term {
        amplitude: 1.32200340000000e-04,
        phase: 3.32014387930000e+00,
        frequency: 1.48447270830000e+00,
    },
    Vsop87Term {
        amplitude: 8.62177900000000e-05,
        phase: 6.21626927537000e+00,
        frequency: 3.51640902212000e+01,
    },
    Vsop87Term {
        amplitude: 2.70158700000000e-05,
        phase: 1.88124996531000e+00,
        frequency: 3.96175083461000e+01,
    },
    Vsop87Term {
        amplitude: 2.15306000000000e-05,
        phase: 5.16877044933000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 2.15417000000000e-05,
        phase: 2.09430333390000e+00,
        frequency: 2.96894541660000e+00,
    },
    Vsop87Term {
        amplitude: 1.46331400000000e-05,
        phase: 1.18410155089000e+00,
        frequency: 3.36796175129000e+01,
    },
    Vsop87Term {
        amplitude: 1.60316400000000e-05,
        phase: 0.00000000000000e+00,
        frequency: 0.00000000000000e+00,
    },
    Vsop87Term {
        amplitude: 1.13566300000000e-05,
        phase: 3.91905853528000e+00,
        frequency: 3.66485629295000e+01,
    },
    Vsop87Term {
        amplitude: 8.97650000000000e-06,
        phase: 5.24122933533000e+00,
        frequency: 3.88465155238200e+02,
    },
    Vsop87Term {
        amplitude: 7.89359000000000e-06,
        phase: 5.32950007180000e-01,
        frequency: 1.68052512799400e+02,
    },
    Vsop87Term {
        amplitude: 7.60030000000000e-06,
        phase: 2.05103364400000e-02,
        frequency: 1.82279606801000e+02,
    },
    Vsop87Term {
        amplitude: 6.07183000000000e-06,
        phase: 1.07706500350000e+00,
        frequency: 1.02124889455140e+03,
    },
    Vsop87Term {
        amplitude: 5.71622000000000e-06,
        phase: 3.40060785432000e+00,
        frequency: 4.84444382456000e+02,
    },
    Vsop87Term {
        amplitude: 5.60790000000000e-06,
        phase: 2.88685815667000e+00,
        frequency: 4.98671476457600e+02,
    },
    Vsop87Term {
        amplitude: 4.89973000000000e-06,
        phase: 3.46822250901000e+00,
        frequency: 1.37033024162400e+02,
    },
    Vsop87Term {
        amplitude: 2.64197000000000e-06,
        phase: 8.61493686020000e-01,
        frequency: 4.45341812490000e+00,
    },
    Vsop87Term {
        amplitude: 2.70304000000000e-06,
        phase: 3.27489604455000e+00,
        frequency: 7.18126531507000e+01,
    },
    Vsop87Term {
        amplitude: 2.03512000000000e-06,
        phase: 2.41823214253000e+00,
        frequency: 3.21951448046000e+01,
    },
    Vsop87Term {
        amplitude: 1.55180000000000e-06,
        phase: 3.65150530810000e-01,
        frequency: 4.11019810544000e+01,
    },
];

const NEPTUNE_R_2: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 4.24777600000000e-05,
        phase: 5.89911844921000e+00,
        frequency: 3.81330356378000e+01,
    },
    Vsop87Term {
        amplitude: 2.17404000000000e-06,
        phase: 3.45895467130000e-01,
        frequency: 1.48447270830000e+00,
    },
    Vsop87Term {
        amplitude: 1.63025000000000e-06,
        phase: 2.23872947130000e+00,
        frequency: 1.68052512799400e+02,
    },
    Vsop87Term {
        amplitude: 1.56285000000000e-06,
        phase: 4.59414467342000e+00,
        frequency: 1.82279606801000e+02,
    },
    Vsop87Term {
        amplitude: 1.17940000000000e-06,
        phase: 5.10295026024000e+00,
        frequency: 4.84444382456000e+02,
    },
    Vsop87Term {
        amplitude: 1.12429000000000e-06,
        phase: 1.19000583596000e+00,
        frequency: 4.98671476457600e+02,
    },
    Vsop87Term {
        amplitude: 1.27836000000000e-06,
        phase: 2.84821806298000e+00,
        frequency: 3.51640902212000e+01,
    },
    Vsop87Term {
        amplitude: 9.93110000000000e-07,
        phase: 3.41592669789000e+00,
        frequency: 1.75166059800200e+02,
    },
    Vsop87Term {
        amplitude: 6.48140000000000e-07,
        phase: 3.46214064840000e+00,
        frequency: 3.88465155238200e+02,
    },
    Vsop87Term {
        amplitude: 7.67800000000000e-07,
        phase: 1.68034306500000e-02,
        frequency: 4.91557929456800e+02,
    },
    Vsop87Term {
        amplitude: 4.95110000000000e-07,
        phase: 4.06995993334000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 3.93300000000000e-07,
        phase: 6.09521855958000e+00,
        frequency: 1.02124889455140e+03,
    },
    Vsop87Term {
        amplitude: 3.64510000000000e-07,
        phase: 5.17129778081000e+00,
        frequency: 1.37033024162400e+02,
    },
    Vsop87Term {
        amplitude: 3.67090000000000e-07,
        phase: 5.97476878862000e+00,
        frequency: 2.96894541660000e+00,
    },
    Vsop87Term {
        amplitude: 2.90370000000000e-07,
        phase: 3.58135470306000e+00,
        frequency: 3.36796175129000e+01,
    },
    Vsop87Term {
        amplitude: 2.08620000000000e-07,
        phase: 7.73415684230000e-01,
        frequency: 3.66485629295000e+01,
    },
    Vsop87Term {
        amplitude: 1.38860000000000e-07,
        phase: 3.59248623971000e+00,
        frequency: 3.95578702239000e+02,
    },
    Vsop87Term {
        amplitude: 1.30010000000000e-07,
        phase: 5.12870831936000e+00,
        frequency: 9.88999885246000e+01,
    },
    Vsop87Term {
        amplitude: 1.13790000000000e-07,
        phase: 1.18060018898000e+00,
        frequency: 3.81351608237400e+02,
    },
    Vsop87Term {
        amplitude: 9.13200000000000e-08,
        phase: 2.34787658568000e+00,
        frequency: 6.01764250676200e+02,
    },
];

const NEPTUNE_R_3: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 1.66556000000000e-06,
        phase: 4.55393495836000e+00,
        frequency: 3.81330356378000e+01,
    },
    Vsop87Term {
        amplitude: 2.23800000000000e-07,
        phase: 3.94830879358000e+00,
        frequency: 1.68052512799400e+02,
    },
    Vsop87Term {
        amplitude: 2.13480000000000e-07,
        phase: 2.86296778794000e+00,
        frequency: 1.82279606801000e+02,
    },
    Vsop87Term {
        amplitude: 1.62330000000000e-07,
        phase: 5.42267258720000e-01,
        frequency: 4.84444382456000e+02,
    },
    Vsop87Term {
        amplitude: 1.56230000000000e-07,
        phase: 5.75702251906000e+00,
        frequency: 4.98671476457600e+02,
    },
    Vsop87Term {
        amplitude: 1.14120000000000e-07,
        phase: 4.38291384655000e+00,
        frequency: 1.48447270830000e+00,
    },
    Vsop87Term {
        amplitude: 6.44800000000000e-08,
        phase: 5.19003066847000e+00,
        frequency: 3.10194886370000e+01,
    },
    Vsop87Term {
        amplitude: 3.65500000000000e-08,
        phase: 5.91335292846000e+00,
        frequency: 1.00702180054980e+03,
    },
    Vsop87Term {
        amplitude: 3.68100000000000e-08,
        phase: 1.62865545676000e+00,
        frequency: 3.88465155238200e+02,
    },
    Vsop87Term {
        amplitude: 3.19800000000000e-08,
        phase: 7.01971185750000e-01,
        frequency: 1.55805340664680e+03,
    },
    Vsop87Term {
        amplitude: 3.24300000000000e-08,
        phase: 1.88035665980000e+00,
        frequency: 5.22577418093800e+02,
    },
    Vsop87Term {
        amplitude: 2.68800000000000e-08,
        phase: 1.87062743473000e+00,
        frequency: 4.02692249239800e+02,
    },
    Vsop87Term {
        amplitude: 3.24600000000000e-08,
        phase: 7.93813561930000e-01,
        frequency: 5.36804512095400e+02,
    },
    Vsop87Term {
        amplitude: 2.65000000000000e-08,
        phase: 5.76858449026000e+00,
        frequency: 3.43218572599600e+02,
    },
    Vsop87Term {
        amplitude: 2.64400000000000e-08,
        phase: 4.64542905401000e+00,
        frequency: 5.00155949165900e+02,
    },
    Vsop87Term {
        amplitude: 2.54100000000000e-08,
        phase: 4.79217120822000e+00,
        frequency: 4.82959909747700e+02,
    },
    Vsop87Term {
        amplitude: 2.52300000000000e-08,
        phase: 1.72869889780000e+00,
        frequency: 3.95578702239000e+02,
    },
    Vsop87Term {
        amplitude: 3.04000000000000e-08,
        phase: 2.90934098363000e+00,
        frequency: 7.62660712756000e+01,
    },
    Vsop87Term {
        amplitude: 2.69000000000000e-08,
        phase: 2.21096415618000e+00,
        frequency: 4.46311346818200e+02,
    },
    Vsop87Term {
        amplitude: 2.35500000000000e-08,
        phase: 5.77381398401000e+00,
        frequency: 4.85928855164300e+02,
    },
];

const NEPTUNE_R_4: &[Vsop87Term] = &[
    Vsop87Term {
        amplitude: 4.22700000000000e-08,
        phase: 2.40375758563000e+00,
        frequency: 4.77330835455200e+02,
    },
    Vsop87Term {
        amplitude: 4.33300000000000e-08,
        phase: 1.04594845450000e-01,
        frequency: 3.95578702239000e+02,
    },
    Vsop87Term {
        amplitude: 3.54500000000000e-08,
        phase: 4.78431259422000e+00,
        frequency: 1.02836244155220e+03,
    },
    Vsop87Term {
        amplitude: 3.15400000000000e-08,
        phase: 3.88192942366000e+00,
        frequency: 5.05785023458400e+02,
    },
    Vsop87Term {
        amplitude: 3.01600000000000e-08,
        phase: 1.03609346831000e+00,
        frequency: 1.89393153801800e+02,
    },
    Vsop87Term {
        amplitude: 2.29400000000000e-08,
        phase: 1.10879658603000e+00,
        frequency: 1.82279606801000e+02,
    },
    Vsop87Term {
        amplitude: 2.29500000000000e-08,
        phase: 5.67776133184000e+00,
        frequency: 1.68052512799400e+02,
    },
];

const NEPTUNE_R_5: &[Vsop87Term] = &[];
