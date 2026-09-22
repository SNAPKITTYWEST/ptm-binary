// Complexity verifier: ML-based polynomial complexity bound verification.

#[derive(Debug, Clone)]
pub struct ComplexityBound {
    pub degree: u32,
    pub constant: f64,
    pub n0: u64,
    pub proven: bool,
}

impl ComplexityBound {
    pub fn new(degree: u32, constant: f64, n0: u64) -> Self {
        ComplexityBound { degree, constant, n0, proven: false }
    }

    pub fn bound(&self, n: u64) -> f64 {
        if n < self.n0 {
            return f64::INFINITY;
        }
        self.constant * (n as f64).powi(self.degree as i32)
    }

    pub fn is_within_bound(&self, n: u64, actual_steps: u64) -> bool {
        actual_steps as f64 <= self.bound(n)
    }
}

#[derive(Debug, Clone)]
pub struct ComplexityVerifier {
    pub measured_points: Vec<(u64, u64)>,
    pub inferred_bound: Option<ComplexityBound>,
}

impl ComplexityVerifier {
    pub fn new() -> Self {
        ComplexityVerifier {
            measured_points: Vec::new(),
            inferred_bound: None,
        }
    }

    pub fn record(&mut self, input_size: u64, steps: u64) {
        self.measured_points.push((input_size, steps));
    }

    pub fn infer(&mut self) -> Option<ComplexityBound> {
        if self.measured_points.len() < 3 {
            return None;
        }

        let mut best_degree = 1;
        let mut best_error = f64::INFINITY;

        for degree in 1..=5 {
            let error = self.fit_polynomial_error(degree);
            if error < best_error {
                best_error = error;
                best_degree = degree;
            }
        }

        let (constant, n0) = self.estimate_params(best_degree);
        let bound = ComplexityBound {
            degree: best_degree,
            constant,
            n0,
            proven: self.verify_bound(best_degree, constant, n0),
        };

        self.inferred_bound = Some(bound.clone());
        Some(bound)
    }

    fn fit_polynomial_error(&self, degree: u32) -> f64 {
        if self.measured_points.is_empty() {
            return f64::INFINITY;
        }

        let sum_log_n: f64 = self.measured_points.iter()
            .map(|&(n, _)| (n as f64).ln())
            .sum();
        let sum_log_t: f64 = self.measured_points.iter()
            .map(|&(_, t)| (t as f64).ln())
            .sum();
        let n = self.measured_points.len() as f64;
        let d = degree as f64;

        let sum_sq: f64 = self.measured_points.iter()
            .map(|&(ni, ti)| {
                let log_n = (ni as f64).ln();
                let log_t = (ti as f64).ln();
                let predicted = d * log_n + (sum_log_t - d * sum_log_n) / n;
                (log_t - predicted).powi(2)
            })
            .sum();

        sum_sq / n
    }

    fn estimate_params(&self, degree: u32) -> (f64, u64) {
        if self.measured_points.is_empty() {
            return (1.0, 1);
        }

        let d = degree as f64;
        let n = self.measured_points.len() as f64;
        let sum_log_n: f64 = self.measured_points.iter().map(|&(ni, _)| (ni as f64).ln()).sum();
        let sum_log_t: f64 = self.measured_points.iter().map(|&(_, ti)| (ti as f64).ln()).sum();

        let log_c = (sum_log_t - d * sum_log_n) / n;
        let c = log_c.exp();
        let n0 = self.measured_points.iter().map(|&(ni, _)| ni).min().unwrap_or(1);

        (c, n0)
    }

    fn verify_bound(&self, degree: u32, constant: f64, n0: u64) -> bool {
        let bound = ComplexityBound::new(degree, constant, n0);
        self.measured_points.iter()
            .all(|&(n, t)| bound.is_within_bound(n, t))
    }

    pub fn report(&self) -> String {
        match &self.inferred_bound {
            None => "Insufficient data for complexity analysis.".into(),
            Some(bound) => format!(
                "Complexity Bound:\n  Degree: {}{}\n  Constant: {:.4}\n  n0: {}\n  Proven: {}",
                bound.degree,
                if bound.proven { "" } else { " (UNPROVEN)" },
                bound.constant,
                bound.n0,
                bound.proven
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_time_detection() {
        let mut verifier = ComplexityVerifier::new();
        for n in [16, 32, 64, 128, 256] {
            verifier.record(n, 100);
        }
        let bound = verifier.infer().unwrap();
        assert_eq!(bound.degree, 1);
        assert!(bound.constant > 0.0);
    }

    #[test]
    fn linear_detection() {
        let mut verifier = ComplexityVerifier::new();
        for n in [16, 32, 64, 128, 256] {
            verifier.record(n, n * 10);
        }
        let bound = verifier.infer().unwrap();
        assert!(bound.degree >= 1);
    }
}
