// State encoder: binary state encoding strategies for FPGA synthesis.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingType {
    Binary,
    OneHot,
    Gray,
}

#[derive(Debug, Clone)]
pub struct StateEncoder {
    pub encoding: EncodingType,
}

impl Default for StateEncoder {
    fn default() -> Self {
        StateEncoder { encoding: EncodingType::Binary }
    }
}

impl StateEncoder {
    pub fn encode_states(&self, num_states: u32) -> Vec<u32> {
        match self.encoding {
            EncodingType::Binary => (0..num_states).collect(),
            EncodingType::OneHot => (0..num_states).map(|i| 1u32 << i).collect(),
            EncodingType::Gray => (0..num_states).map(|i| i ^ (i >> 1)).collect(),
        }
    }

    pub fn bits_needed(&self, num_states: u32) -> u32 {
        match self.encoding {
            EncodingType::Binary => (num_states as f64).log2().ceil() as u32,
            EncodingType::OneHot => num_states,
            EncodingType::Gray => (num_states as f64).log2().ceil() as u32,
        }
    }

    pub fn generate_verilog_encoding(&self, states: &[(u32, &str)]) -> String {
        let mut out = String::with_capacity(512);
        let encodings = self.encode_states(states.len() as u32);

        match self.encoding {
            EncodingType::Binary => {
                out.push_str("    // Binary state encoding\n");
                out.push_str("    typedef enum logic [");
                let bits = self.bits_needed(states.len() as u32);
                out.push_str(&format!("{}:0", bits - 1));
                out.push_str("] {\n");
            }
            EncodingType::OneHot => {
                out.push_str("    // One-hot state encoding\n");
                out.push_str(&format!("    typedef enum logic [{}:0] {{\n", states.len() - 1));
            }
            EncodingType::Gray => {
                out.push_str("    // Gray code state encoding\n");
                let bits = self.bits_needed(states.len() as u32);
                out.push_str(&format!("    typedef enum logic [{}:0] {{\n", bits - 1));
            }
        }

        for (i, (_idx, name)) in states.iter().enumerate() {
            let enc = encodings[i];
            match self.encoding {
                EncodingType::Binary => {
                    out.push_str(&format!("        S_{} = {}'d{}", name, self.bits_needed(states.len() as u32), enc));
                }
                EncodingType::OneHot => {
                    out.push_str(&format!("        S_{} = {}'h{:X}", name, states.len(), enc));
                }
                EncodingType::Gray => {
                    out.push_str(&format!("        S_{} = {}'b{:b}", name, self.bits_needed(states.len() as u32), enc));
                }
            }
            if i < states.len() - 1 {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("    } state_t;\n");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_encoding() {
        let enc = StateEncoder { encoding: EncodingType::Binary };
        let codes = enc.encode_states(4);
        assert_eq!(codes, vec![0, 1, 2, 3]);
        assert_eq!(enc.bits_needed(4), 2);
    }

    #[test]
    fn onehot_encoding() {
        let enc = StateEncoder { encoding: EncodingType::OneHot };
        let codes = enc.encode_states(4);
        assert_eq!(codes, vec![1, 2, 4, 8]);
        assert_eq!(enc.bits_needed(4), 4);
    }

    #[test]
    fn gray_encoding() {
        let enc = StateEncoder { encoding: EncodingType::Gray };
        let codes = enc.encode_states(4);
        assert_eq!(codes, vec![0, 1, 3, 2]);
    }
}
