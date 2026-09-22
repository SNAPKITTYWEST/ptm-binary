// Pattern matcher: template-based pattern matching on PTM traces.

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub pattern_name: String,
    pub confidence: f64,
    pub start_step: u64,
    pub end_step: u64,
}

#[derive(Debug, Clone)]
pub struct PatternTemplate {
    pub name: String,
    pub state_sequence: Vec<u32>,
    pub min_occurrences: usize,
}

#[derive(Debug, Clone)]
pub struct PatternMatcher {
    pub templates: Vec<PatternTemplate>,
    pub matches: Vec<MatchResult>,
}

impl PatternMatcher {
    pub fn new() -> Self {
        PatternMatcher { templates: Vec::new(), matches: Vec::new() }
    }

    pub fn add_template(&mut self, template: PatternTemplate) {
        self.templates.push(template);
    }

    pub fn match_sequence(&mut self, sequence: &[u32]) -> Vec<MatchResult> {
        let mut results = Vec::new();

        for template in &self.templates {
            let mut occurrences = 0;
            let mut i = 0;

            while i <= sequence.len().saturating_sub(template.state_sequence.len()) {
                if sequence[i..].starts_with(&template.state_sequence) {
                    occurrences += 1;
                    if occurrences >= template.min_occurrences {
                        results.push(MatchResult {
                            pattern_name: template.name.clone(),
                            confidence: 1.0,
                            start_step: i as u64,
                            end_step: (i + template.state_sequence.len()) as u64,
                        });
                    }
                    i += template.state_sequence.len();
                } else {
                    i += 1;
                }
            }
        }

        self.matches = results.clone();
        results
    }

    pub fn detect_periodicity(&self, sequence: &[u32]) -> Option<usize> {
        if sequence.len() < 4 {
            return None;
        }

        for period in 2..=sequence.len() / 2 {
            let mut is_periodic = true;
            for i in period..sequence.len() {
                if sequence[i] != sequence[i % period] {
                    is_periodic = false;
                    break;
                }
            }
            if is_periodic {
                return Some(period);
            }
        }
        None
    }

    pub fn detect_cycle(&self, sequence: &[u32]) -> Option<(usize, usize)> {
        if sequence.len() < 2 {
            return None;
        }

        let mut slow = 0;
        let mut fast = 0;

        loop {
            slow += 1;
            fast += 2;
            if fast >= sequence.len() {
                return None;
            }
            if sequence[slow] == sequence[fast] {
                let mut start = 0;
                let mut s = 0;
                loop {
                    if sequence[s] == sequence[slow] {
                        break;
                    }
                    s += 1;
                    start += 1;
                    if s >= sequence.len() || start >= sequence.len() {
                        return None;
                    }
                }
                let mut length = 1;
                let mut pos = start + 1;
                while pos < sequence.len() && sequence[pos] != sequence[start] {
                    pos += 1;
                    length += 1;
                }
                if length < sequence.len() {
                    return Some((start, length));
                }
            }
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "Pattern Matcher:\n  Templates: {}\n  Matches found: {}",
            self.templates.len(),
            self.matches.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_matching() {
        let mut matcher = PatternMatcher::new();
        matcher.add_template(PatternTemplate {
            name: "AB".into(),
            state_sequence: vec![0, 1],
            min_occurrences: 1,
        });
        let seq = vec![0, 1, 2, 0, 1, 3];
        let results = matcher.match_sequence(&seq);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn periodicity_detection() {
        let matcher = PatternMatcher::new();
        let seq = vec![0, 1, 2, 0, 1, 2, 0, 1, 2];
        assert_eq!(matcher.detect_periodicity(&seq), Some(3));
    }
}
