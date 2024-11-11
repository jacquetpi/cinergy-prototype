#[derive(Clone)]
pub struct PolynomialModel {
    coefficients: Vec<f64>,
}

impl PolynomialModel {
    // Create a new PolynomialModel with given coefficients
    pub fn new(coefficients: Vec<f64>) -> Self {
        PolynomialModel { coefficients }
    }

    // Estimate the polynomial value for a given x (input)
    pub fn estimate(&self, x: f64) -> f64 {
        let mut result = 0.0;

        for (i, &coef) in self.coefficients.iter().enumerate() {
            result += coef * x.powi(i as i32); // x^i * coef
        }

        result
    }
}
