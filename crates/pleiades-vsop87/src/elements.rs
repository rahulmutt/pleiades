/// Mean Keplerian orbital elements used for fallback body paths.
#[derive(Clone, Copy, Debug)]
pub(crate) struct OrbitalElements {
    pub(crate) ascending_node: f64,
    pub(crate) inclination: f64,
    pub(crate) argument_of_perihelion: f64,
    pub(crate) semi_major_axis: f64,
    pub(crate) eccentricity: f64,
    pub(crate) mean_anomaly: f64,
}

impl OrbitalElements {
    pub(crate) const fn new(
        ascending_node: f64,
        inclination: f64,
        argument_of_perihelion: f64,
        semi_major_axis: f64,
        eccentricity: f64,
        mean_anomaly: f64,
    ) -> Self {
        Self {
            ascending_node,
            inclination,
            argument_of_perihelion,
            semi_major_axis,
            eccentricity,
            mean_anomaly,
        }
    }
}
