use std::env;

/// Application configuration loaded from environment variables.
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub cinergy_ratio: f64,
    pub server_cores: u32,
    pub dc_pue: f64,
    /// Carbon emission factor in gCO2eq/kWh
    pub emission_factor: f64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let cinergy_ratio: f64 = env::var("CINERGY_RATIO")
            .expect("CINERGY_RATIO must be set")
            .parse()
            .expect("CINERGY_RATIO must be a valid float");

        assert!(
            (0.0..=1.0).contains(&cinergy_ratio),
            "CINERGY_RATIO must be between 0.0 and 1.0, got {}",
            cinergy_ratio
        );

        let server_cores: u32 = env::var("SERVER_CORES")
            .expect("SERVER_CORES must be set")
            .parse()
            .expect("SERVER_CORES must be a valid positive integer");

        assert!(server_cores > 0, "SERVER_CORES must be greater than 0");

        let dc_pue: f64 = env::var("DC_PUE")
            .expect("DC_PUE must be set")
            .parse()
            .expect("DC_PUE must be a valid float");

        assert!(dc_pue >= 1.0, "DC_PUE must be >= 1.0, got {}", dc_pue);

        let emission_factor: f64 = env::var("EMISSION_FACTOR")
            .expect("EMISSION_FACTOR must be set")
            .parse()
            .expect("EMISSION_FACTOR must be a valid float");

        assert!(
            emission_factor >= 0.0,
            "EMISSION_FACTOR must be >= 0.0, got {}",
            emission_factor
        );

        AppConfig {
            cinergy_ratio,
            server_cores,
            dc_pue,
            emission_factor,
        }
    }

    /// Compute carbon emission rate in gCO2eq/h from instantaneous power in watts.
    ///
    /// Formula: (power_watts / 1000) * dc_pue * emission_factor
    /// where emission_factor is in gCO2eq/kWh.
    pub fn carbon_intensity_gco2_per_hour(&self, power_watts: f64) -> f64 {
        (power_watts / 1000.0) * self.dc_pue * self.emission_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(cinergy_ratio: f64, server_cores: u32, dc_pue: f64, emission_factor: f64) -> AppConfig {
        AppConfig {
            cinergy_ratio,
            server_cores,
            dc_pue,
            emission_factor,
        }
    }

    #[test]
    fn test_carbon_intensity_basic() {
        let config = make_config(0.5, 16, 1.3, 50.0);
        // 100W -> 0.1 kW * 1.3 * 50 = 6.5 gCO2eq/h
        let result = config.carbon_intensity_gco2_per_hour(100.0);
        assert!((result - 6.5).abs() < 1e-9);
    }

    #[test]
    fn test_carbon_intensity_zero_power() {
        let config = make_config(0.5, 16, 1.3, 50.0);
        assert!((config.carbon_intensity_gco2_per_hour(0.0)).abs() < 1e-9);
    }

    #[test]
    fn test_carbon_intensity_pue_one() {
        let config = make_config(1.0, 8, 1.0, 100.0);
        // 500W -> 0.5 kW * 1.0 * 100 = 50 gCO2eq/h
        let result = config.carbon_intensity_gco2_per_hour(500.0);
        assert!((result - 50.0).abs() < 1e-9);
    }
}
