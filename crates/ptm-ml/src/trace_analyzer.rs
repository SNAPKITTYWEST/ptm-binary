// Trace analyzer: ML-based analysis of PTM execution traces.

use ptm_engine::snapshot::Snapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TracePattern {
    Linear,
    Periodic,
    Divergent,
    Convergent,
    Chaotic,
    Constant,
}

#[derive(Debug, Clone)]
pub struct TraceEvent {
    pub step: u64,
    pub state: u32,
    pub input_head: u32,
    pub work_head: u32,
    pub output_head: u32,
}

#[derive(Debug, Clone)]
pub struct TraceAnalyzer {
    pub events: Vec<TraceEvent>,
    state_histogram: Vec<u64>,
    head_movements: Vec<(i32, i32, i32, i32)>,
}

impl TraceAnalyzer {
    pub fn new() -> Self {
        TraceAnalyzer {
            events: Vec::new(),
            state_histogram: Vec::new(),
            head_movements: Vec::new(),
        }
    }

    pub fn from_snapshots(snapshots: &[Snapshot]) -> Self {
        let mut analyzer = TraceAnalyzer::new();
        for snap in snapshots {
            analyzer.events.push(TraceEvent {
                step: snap.metadata.step_number as u64,
                state: snap.metadata.current_state,
                input_head: snap.metadata.input_head,
                work_head: snap.metadata.work_head,
                output_head: snap.metadata.output_head,
            });
        }
        analyzer.compute_histograms();
        analyzer
    }

    fn compute_histograms(&mut self) {
        let max_state = self.events.iter().map(|e| e.state).max().unwrap_or(0) + 1;
        self.state_histogram = vec![0; max_state as usize];
        for e in &self.events {
            self.state_histogram[e.state as usize] += 1;
        }

        self.head_movements = self.events.windows(2).map(|w| {
            (
                w[1].input_head as i32 - w[0].input_head as i32,
                w[1].work_head as i32 - w[0].work_head as i32,
                0,
                w[1].output_head as i32 - w[0].output_head as i32,
            )
        }).collect();
    }

    pub fn detect_pattern(&self) -> TracePattern {
        if self.events.len() < 4 {
            return TracePattern::Constant;
        }

        // Check if all states are the same (constant)
        let first_state = self.events[0].state;
        if self.events.iter().all(|e| e.state == first_state) {
            return TracePattern::Constant;
        }

        // Check for periodicity
        if self.is_periodic() {
            return TracePattern::Periodic;
        }

        // Check head movement trends
        let work_drift: i64 = self.head_movements.iter()
            .map(|m| m.1 as i64)
            .sum();

        let total_steps = self.head_movements.len() as i64;
        if total_steps == 0 {
            return TracePattern::Constant;
        }

        let avg_drift = work_drift as f64 / total_steps as f64;

        if avg_drift.abs() < 0.01 {
            TracePattern::Constant
        } else if avg_drift > 0.5 {
            TracePattern::Divergent
        } else if avg_drift < -0.5 {
            TracePattern::Convergent
        } else {
            TracePattern::Linear
        }
    }

    fn is_periodic(&self) -> bool {
        let states: Vec<u32> = self.events.iter().map(|e| e.state).collect();
        for period in 2..=states.len() / 2 {
            let mut is_period = true;
            for i in period..states.len() {
                if states[i] != states[i % period] {
                    is_period = false;
                    break;
                }
            }
            if is_period && period <= 32 {
                return true;
            }
        }
        false
    }

    pub fn state_distribution(&self) -> Vec<f64> {
        let total = self.events.len() as f64;
        if total == 0.0 {
            return Vec::new();
        }
        self.state_histogram.iter().map(|&c| c as f64 / total).collect()
    }

    pub fn entropy(&self) -> f64 {
        let dist = self.state_distribution();
        -dist.iter()
            .filter(|&&p| p > 0.0)
            .map(|&p| p * p.log2())
            .sum::<f64>()
    }

    pub fn head_variance(&self) -> (f64, f64) {
        if self.events.is_empty() {
            return (0.0, 0.0);
        }
        let n = self.events.len() as f64;
        let input_mean = self.events.iter().map(|e| e.input_head as f64).sum::<f64>() / n;
        let work_mean = self.events.iter().map(|e| e.work_head as f64).sum::<f64>() / n;
        let input_var = self.events.iter().map(|e| (e.input_head as f64 - input_mean).powi(2)).sum::<f64>() / n;
        let work_var = self.events.iter().map(|e| (e.work_head as f64 - work_mean).powi(2)).sum::<f64>() / n;
        (input_var, work_var)
    }

    pub fn summary(&self) -> String {
        let pattern = self.detect_pattern();
        let entropy = self.entropy();
        let (input_var, work_var) = self.head_variance();
        format!(
            "Trace Analysis:\n  Events: {}\n  Pattern: {:?}\n  State Entropy: {:.3} bits\n  Input Head Variance: {:.2}\n  Work Head Variance: {:.2}\n  Unique States Visited: {}",
            self.events.len(),
            pattern,
            entropy,
            input_var,
            work_var,
            self.state_histogram.iter().filter(|&&c| c > 0).count()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_trace() {
        let analyzer = TraceAnalyzer::new();
        assert_eq!(analyzer.detect_pattern(), TracePattern::Constant);
        assert_eq!(analyzer.entropy(), 0.0);
    }

    #[test]
    fn constant_pattern() {
        let mut analyzer = TraceAnalyzer::new();
        for i in 0..10 {
            analyzer.events.push(TraceEvent {
                step: i,
                state: 0,
                input_head: 5,
                work_head: 3,
                output_head: 0,
            });
        }
        assert_eq!(analyzer.detect_pattern(), TracePattern::Constant);
    }
}
