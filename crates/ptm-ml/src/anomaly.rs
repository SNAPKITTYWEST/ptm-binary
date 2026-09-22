// Anomaly detector: statistical analysis of PTM execution for anomalies.

#[derive(Debug, Clone)]
pub struct AnomalyScore {
    pub step: u64,
    pub score: f64,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct AnomalyDetector {
    pub state_visit_counts: Vec<u64>,
    pub head_distances: Vec<f64>,
    pub transition_entropy: Vec<f64>,
    pub window_size: usize,
}

impl AnomalyDetector {
    pub fn new(window_size: usize) -> Self {
        AnomalyDetector {
            state_visit_counts: Vec::new(),
            head_distances: Vec::new(),
            transition_entropy: Vec::new(),
            window_size,
        }
    }

    pub fn analyze_state_visits(&self) -> Vec<AnomalyScore> {
        let mut scores = Vec::new();
        if self.state_visit_counts.is_empty() {
            return scores;
        }

        let total: u64 = self.state_visit_counts.iter().sum();
        let mean = total as f64 / self.state_visit_counts.len() as f64;
        let variance: f64 = self.state_visit_counts.iter()
            .map(|&c| (c as f64 - mean).powi(2))
            .sum::<f64>() / self.state_visit_counts.len() as f64;
        let std = variance.sqrt();

        if std < 1e-10 {
            return scores;
        }

        for (state, &count) in self.state_visit_counts.iter().enumerate() {
            let z_score = (count as f64 - mean) / std;
            if z_score.abs() > 2.0 {
                scores.push(AnomalyScore {
                    step: state as u64,
                    score: z_score.abs(),
                    reason: format!(
                        "State {} visited {} times (mean {:.1}, z={:.2})",
                        state, count, mean, z_score
                    ),
                });
            }
        }

        scores
    }

    pub fn analyze_head_movement(&self, movements: &[(i32, i32)]) -> Vec<AnomalyScore> {
        let mut scores = Vec::new();
        if movements.len() < self.window_size {
            return scores;
        }

        for window in movements.windows(self.window_size) {
            let sum: i64 = window.iter().map(|m| m.1 as i64).sum();
            let mean = sum as f64 / self.window_size as f64;

            if mean.abs() > 5.0 {
                let step = movements.len() as u64;
                scores.push(AnomalyScore {
                    step,
                    score: mean.abs(),
                    reason: format!(
                        "Work head drift: mean {:.2} over window {}",
                        mean, self.window_size
                    ),
                });
            }
        }

        scores
    }

    pub fn compute_transition_entropy(&self, window: &[(u32, u32)]) -> f64 {
        if window.is_empty() {
            return 0.0;
        }

        let mut counts = std::collections::HashMap::new();
        for &(from, to) in window {
            *counts.entry((from, to)).or_insert(0u64) += 1;
        }

        let total = window.len() as f64;
        -counts.values()
            .map(|&c| {
                let p = c as f64 / total;
                p * p.log2()
            })
            .sum::<f64>()
    }

    pub fn detect_stuck(&self, current_state: u32, recent_states: &[u32], threshold: usize) -> bool {
        if recent_states.len() < threshold {
            return false;
        }
        recent_states.iter().rev().take(threshold).all(|&s| s == current_state)
    }

    pub fn summary(&self) -> String {
        let anomalies = self.analyze_state_visits();
        format!(
            "Anomaly Detector:\n  States tracked: {}\n  Window size: {}\n  Anomalies found: {}",
            self.state_visit_counts.len(),
            self.window_size,
            anomalies.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_anomalies_uniform() {
        let mut det = AnomalyDetector::new(3);
        det.state_visit_counts = vec![10, 10, 10, 10];
        let anomalies = det.analyze_state_visits();
        assert!(anomalies.is_empty());
    }

    #[test]
    fn detects_outlier_state() {
        let mut det = AnomalyDetector::new(3);
        det.state_visit_counts = vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1000];
        let anomalies = det.analyze_state_visits();
        assert!(!anomalies.is_empty());
        assert!(anomalies[0].score > 2.0);
    }

    #[test]
    fn stuck_detection() {
        let det = AnomalyDetector::new(3);
        let recent = vec![5, 5, 5, 5, 5];
        assert!(det.detect_stuck(5, &recent, 3));
        assert!(!det.detect_stuck(3, &recent, 3));
    }
}
