use serde::Serialize;

#[derive(Clone)]
pub struct PolynomialModel {
    coefficients: Vec<f64>,
}

/// Result of the isolated VM power estimation.
#[derive(Debug, Serialize)]
pub struct PowerEstimate {
    pub static_power_watts: f64,
    pub dynamic_power_watts: f64,
    pub total_power_watts: f64,
}

impl PolynomialModel {
    pub fn new(coefficients: Vec<f64>) -> Self {
        PolynomialModel { coefficients }
    }

    /// Evaluate the polynomial f(x) = c0 + c1*x + c2*x^2 + ...
    pub fn estimate(&self, x: f64) -> f64 {
        self.coefficients
            .iter()
            .enumerate()
            .map(|(i, &coef)| coef * x.powi(i as i32))
            .sum()
    }

    /// Estimate isolated VM power consumption using the cinergy ratio model.
    ///
    /// - static  = f(0) * (vm_cores / server_cores)
    /// - dynamic = (f(cpu_usage) - f(0)) * cinergy_ratio
    /// - total   = static + dynamic
    pub fn estimate_vm_power(
        &self,
        cpu_usage: f64,
        vm_cores: u32,
        server_cores: u32,
        cinergy_ratio: f64,
    ) -> PowerEstimate {
        let f_0 = self.estimate(0.0);
        let f_x = self.estimate(cpu_usage);

        let static_power = f_0 * (vm_cores as f64 / server_cores as f64);
        let dynamic_power = (f_x - f_0) * cinergy_ratio;

        PowerEstimate {
            static_power_watts: static_power,
            dynamic_power_watts: dynamic_power,
            total_power_watts: static_power + dynamic_power,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_constant() {
        let model = PolynomialModel::new(vec![5.0]);
        assert!((model.estimate(0.0) - 5.0).abs() < 1e-9);
        assert!((model.estimate(10.0) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_estimate_linear() {
        // f(x) = 2.0 + 3.0x
        let model = PolynomialModel::new(vec![2.0, 3.0]);
        assert!((model.estimate(0.0) - 2.0).abs() < 1e-9);
        assert!((model.estimate(1.0) - 5.0).abs() < 1e-9);
        assert!((model.estimate(2.0) - 8.0).abs() < 1e-9);
    }

    #[test]
    fn test_estimate_polynomial() {
        // f(x) = 1.0 + 2.0x + 3.0x^2
        let model = PolynomialModel::new(vec![1.0, 2.0, 3.0]);
        assert!((model.estimate(0.0) - 1.0).abs() < 1e-9);
        // f(1) = 1 + 2 + 3 = 6
        assert!((model.estimate(1.0) - 6.0).abs() < 1e-9);
        // f(2) = 1 + 4 + 12 = 17
        assert!((model.estimate(2.0) - 17.0).abs() < 1e-9);
    }

    #[test]
    fn test_f_zero_is_constant_term() {
        let model = PolynomialModel::new(vec![42.0, 1.0, 2.0, 3.0]);
        assert!((model.estimate(0.0) - 42.0).abs() < 1e-9);
    }

    #[test]
    fn test_vm_power_at_zero_cpu() {
        // f(x) = 10 + 5x; f(0)=10
        let model = PolynomialModel::new(vec![10.0, 5.0]);
        let result = model.estimate_vm_power(0.0, 4, 16, 0.8);

        // static = 10 * (4/16) = 2.5
        assert!((result.static_power_watts - 2.5).abs() < 1e-9);
        // dynamic = (10 - 10) * 0.8 = 0
        assert!((result.dynamic_power_watts).abs() < 1e-9);
        assert!((result.total_power_watts - 2.5).abs() < 1e-9);
    }

    #[test]
    fn test_vm_power_with_cinergy_ratio() {
        // f(x) = 10 + 5x; f(0)=10, f(2)=20
        let model = PolynomialModel::new(vec![10.0, 5.0]);
        let result = model.estimate_vm_power(2.0, 4, 16, 0.5);

        // static = 10 * (4/16) = 2.5
        assert!((result.static_power_watts - 2.5).abs() < 1e-9);
        // dynamic = (20 - 10) * 0.5 = 5.0
        assert!((result.dynamic_power_watts - 5.0).abs() < 1e-9);
        assert!((result.total_power_watts - 7.5).abs() < 1e-9);
    }

    #[test]
    fn test_vm_power_ratio_zero_means_static_only() {
        let model = PolynomialModel::new(vec![10.0, 5.0]);
        let result = model.estimate_vm_power(2.0, 2, 8, 0.0);

        // static = 10 * (2/8) = 2.5
        assert!((result.static_power_watts - 2.5).abs() < 1e-9);
        assert!((result.dynamic_power_watts).abs() < 1e-9);
        assert!((result.total_power_watts - 2.5).abs() < 1e-9);
    }

    #[test]
    fn test_vm_power_ratio_one_means_full_dynamic() {
        // f(x) = 10 + 5x; f(0)=10, f(3)=25
        let model = PolynomialModel::new(vec![10.0, 5.0]);
        let result = model.estimate_vm_power(3.0, 4, 4, 1.0);

        // static = 10 * (4/4) = 10
        assert!((result.static_power_watts - 10.0).abs() < 1e-9);
        // dynamic = (25 - 10) * 1.0 = 15
        assert!((result.dynamic_power_watts - 15.0).abs() < 1e-9);
        assert!((result.total_power_watts - 25.0).abs() < 1e-9);
    }
}
