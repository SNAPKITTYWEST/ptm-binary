// Transition table: TMTT binary format per Section 0004.
// 16-byte standard header + 48 bytes PTM metadata.
// State records: 16 bytes each. Transition records: 48 bytes each.

use ptm_core::header::FileHeader;
use ptm_core::magic::Magic;

pub const TMTT_HEADER_SIZE: usize = 16 + 48; // standard + PTM metadata

#[derive(Debug, Clone)]
pub struct TMTTMetadata {
    pub num_states: u32,
    pub num_transitions: u32,
    pub initial_state: u32,
    pub accepting_state: u32,
    pub rejecting_state: u32,
    pub halting_state: u32,
    pub input_alphabet_size: u32,
    pub work_alphabet_size: u32,
    pub control_alphabet_size: u32,
    pub output_alphabet_size: u32,
    pub max_tapes: u32,
    pub poly_degree_bound: u32,
}

impl TMTTMetadata {
    pub fn default_for_ptm() -> Self {
        TMTTMetadata {
            num_states: 0,
            num_transitions: 0,
            initial_state: 0,
            accepting_state: 0,
            rejecting_state: 0,
            halting_state: 0,
            input_alphabet_size: 3,
            work_alphabet_size: 6,
            control_alphabet_size: 3,
            output_alphabet_size: 3,
            max_tapes: 4,
            poly_degree_bound: 1,
        }
    }

    pub fn encode(&self) -> [u8; 48] {
        let mut buf = [0u8; 48];
        buf[0..4].copy_from_slice(&self.num_states.to_be_bytes());
        buf[4..8].copy_from_slice(&self.num_transitions.to_be_bytes());
        buf[8..12].copy_from_slice(&self.initial_state.to_be_bytes());
        buf[12..16].copy_from_slice(&self.accepting_state.to_be_bytes());
        buf[16..20].copy_from_slice(&self.rejecting_state.to_be_bytes());
        buf[20..24].copy_from_slice(&self.halting_state.to_be_bytes());
        buf[24..28].copy_from_slice(&self.input_alphabet_size.to_be_bytes());
        buf[28..32].copy_from_slice(&self.work_alphabet_size.to_be_bytes());
        buf[32..36].copy_from_slice(&self.control_alphabet_size.to_be_bytes());
        buf[36..40].copy_from_slice(&self.output_alphabet_size.to_be_bytes());
        buf[40..44].copy_from_slice(&self.max_tapes.to_be_bytes());
        buf[44..48].copy_from_slice(&self.poly_degree_bound.to_be_bytes());
        buf
    }

    pub fn decode(buf: &[u8; 48]) -> Self {
        TMTTMetadata {
            num_states: u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]),
            num_transitions: u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]),
            initial_state: u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
            accepting_state: u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]),
            rejecting_state: u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]),
            halting_state: u32::from_be_bytes([buf[20], buf[21], buf[22], buf[23]]),
            input_alphabet_size: u32::from_be_bytes([buf[24], buf[25], buf[26], buf[27]]),
            work_alphabet_size: u32::from_be_bytes([buf[28], buf[29], buf[30], buf[31]]),
            control_alphabet_size: u32::from_be_bytes([buf[32], buf[33], buf[34], buf[35]]),
            output_alphabet_size: u32::from_be_bytes([buf[36], buf[37], buf[38], buf[39]]),
            max_tapes: u32::from_be_bytes([buf[40], buf[41], buf[42], buf[43]]),
            poly_degree_bound: u32::from_be_bytes([buf[44], buf[45], buf[46], buf[47]]),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum StateType {
    Normal = 0x01,
    Initial = 0x02,
    Accepting = 0x03,
    Rejecting = 0x04,
    Halting = 0x05,
}

impl StateType {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            0x01 => Some(Self::Normal),
            0x02 => Some(Self::Initial),
            0x03 => Some(Self::Accepting),
            0x04 => Some(Self::Rejecting),
            0x05 => Some(Self::Halting),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StateRecord {
    pub index: u32,
    pub state_type: StateType,
    pub num_outgoing: u32,
    pub outgoing_offset: u32,
}

impl StateRecord {
    pub fn encode(&self) -> [u8; 16] {
        let mut buf = [0u8; 16];
        buf[0..4].copy_from_slice(&self.index.to_be_bytes());
        buf[4..8].copy_from_slice(&(self.state_type as u32).to_be_bytes());
        buf[8..12].copy_from_slice(&self.num_outgoing.to_be_bytes());
        buf[12..16].copy_from_slice(&self.outgoing_offset.to_be_bytes());
        buf
    }

    pub fn decode(buf: &[u8; 16]) -> Option<Self> {
        let st = StateType::from_u32(u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]))?;
        Some(StateRecord {
            index: u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]),
            state_type: st,
            num_outgoing: u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
            outgoing_offset: u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TransitionRecord {
    pub id: u32,
    pub source_state: u32,
    pub target_state: u32,
    pub read_input: u8,
    pub read_work: u8,
    pub read_control: u8,
    pub read_output: u8,
    pub write_input: u8,
    pub write_work: u8,
    pub write_control: u8,
    pub write_output: u8,
    pub move_input: u8,
    pub move_work: u8,
    pub move_control: u8,
    pub move_output: u8,
}

impl TransitionRecord {
    pub fn encode(&self) -> [u8; 48] {
        let mut buf = [0u8; 48];
        buf[0..4].copy_from_slice(&self.id.to_be_bytes());
        buf[4..8].copy_from_slice(&self.source_state.to_be_bytes());
        buf[8..12].copy_from_slice(&self.target_state.to_be_bytes());
        buf[12] = self.read_input;
        buf[13] = self.read_work;
        buf[14] = self.read_control;
        buf[15] = self.read_output;
        buf[16] = self.write_input;
        buf[17] = self.write_work;
        buf[18] = self.write_control;
        buf[19] = self.write_output;
        buf[20] = self.move_input;
        buf[21] = self.move_work;
        buf[22] = self.move_control;
        buf[23] = self.move_output;
        // pad to 48 bytes
        buf
    }

    pub fn decode(buf: &[u8; 48]) -> Self {
        TransitionRecord {
            id: u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]),
            source_state: u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]),
            target_state: u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
            read_input: buf[12],
            read_work: buf[13],
            read_control: buf[14],
            read_output: buf[15],
            write_input: buf[16],
            write_work: buf[17],
            write_control: buf[18],
            write_output: buf[19],
            move_input: buf[20],
            move_work: buf[21],
            move_control: buf[22],
            move_output: buf[23],
        }
    }

    pub fn is_valid(&self) -> bool {
        // Input tape head cannot move left (0x00)
        if self.move_input == 0x00 { return false; }
        // Output tape head cannot move left (0x00)
        if self.move_output == 0x00 { return false; }
        true
    }
}

#[derive(Debug, Clone)]
pub struct TransitionTable {
    pub header: FileHeader,
    pub metadata: TMTTMetadata,
    pub states: Vec<StateRecord>,
    pub transitions: Vec<TransitionRecord>,
}

impl TransitionTable {
    pub fn new(metadata: TMTTMetadata) -> Self {
        let payload_len = 48 // metadata
            + 0 // states (filled later)
            + 0; // transitions (filled later)
        let header = FileHeader::new(Magic::TMTT, payload_len as u32);
        TransitionTable {
            header,
            metadata,
            states: Vec::new(),
            transitions: Vec::new(),
        }
    }

    pub fn add_state(&mut self, index: u32, state_type: StateType) {
        self.states.push(StateRecord {
            index,
            state_type,
            num_outgoing: 0,
            outgoing_offset: 0,
        });
        self.metadata.num_states = self.states.len() as u32;
    }

    pub fn add_transition(&mut self, t: TransitionRecord) {
        self.transitions.push(t);
        self.metadata.num_transitions = self.transitions.len() as u32;
    }

    pub fn encode(&self) -> Vec<u8> {
        let states_bytes = self.states.len() * 16;
        let trans_bytes = self.transitions.len() * 48;
        let payload_len = 48 + states_bytes + trans_bytes;
        let header = FileHeader::new(Magic::TMTT, payload_len as u32);
        let mut buf = Vec::with_capacity(16 + payload_len);
        buf.extend_from_slice(&header.encode());
        buf.extend_from_slice(&self.metadata.encode());
        for s in &self.states {
            buf.extend_from_slice(&s.encode());
        }
        for t in &self.transitions {
            buf.extend_from_slice(&t.encode());
        }
        buf
    }

    pub fn decode(buf: &[u8]) -> Result<Self, String> {
        if buf.len() < 16 {
            return Err("buffer too short for header".into());
        }
        let mut hdr = [0u8; 16];
        hdr.copy_from_slice(&buf[0..16]);
        let header = FileHeader::decode(&hdr).map_err(|e| e.to_string())?;
        if buf.len() < 16 + 48 {
            return Err("buffer too short for metadata".into());
        }
        let mut meta_buf = [0u8; 48];
        meta_buf.copy_from_slice(&buf[16..64]);
        let metadata = TMTTMetadata::decode(&meta_buf);
        let mut states = Vec::new();
        let mut offset = 64;
        for _ in 0..metadata.num_states {
            if offset + 16 > buf.len() {
                return Err("buffer too short for state records".into());
            }
            let mut sbuf = [0u8; 16];
            sbuf.copy_from_slice(&buf[offset..offset + 16]);
            states.push(StateRecord::decode(&sbuf).ok_or("invalid state type")?);
            offset += 16;
        }
        let mut transitions = Vec::new();
        for _ in 0..metadata.num_transitions {
            if offset + 48 > buf.len() {
                return Err("buffer too short for transition records".into());
            }
            let mut tbuf = [0u8; 48];
            tbuf.copy_from_slice(&buf[offset..offset + 48]);
            transitions.push(TransitionRecord::decode(&tbuf));
            offset += 48;
        }
        Ok(TransitionTable { header, metadata, states, transitions })
    }

    pub fn find_transition(&self, state: u32, r_in: u8, r_work: u8, r_ctrl: u8, r_out: u8) -> Option<&TransitionRecord> {
        self.transitions.iter().find(|t| {
            t.source_state == state
                && t.read_input == r_in
                && t.read_work == r_work
                && t.read_control == r_ctrl
                && t.read_output == r_out
        })
    }

    pub fn is_deterministic(&self) -> bool {
        let mut sorted = self.transitions.clone();
        sorted.sort_by_key(|t| (t.source_state, t.read_input, t.read_work, t.read_control, t.read_output));
        sorted.windows(2).all(|w| {
            w[0].source_state != w[1].source_state
                || w[0].read_input != w[1].read_input
                || w[0].read_work != w[1].read_work
                || w[0].read_control != w[1].read_control
                || w[0].read_output != w[1].read_output
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_roundtrip() {
        let m = TMTTMetadata::default_for_ptm();
        let buf = m.encode();
        let m2 = TMTTMetadata::decode(&buf);
        assert_eq!(m2.input_alphabet_size, 3);
        assert_eq!(m2.work_alphabet_size, 6);
        assert_eq!(m2.max_tapes, 4);
    }

    #[test]
    fn transition_record_roundtrip() {
        let t = TransitionRecord {
            id: 1,
            source_state: 0,
            target_state: 1,
            read_input: 0x01,
            read_work: 0x02,
            read_control: 0x00,
            read_output: 0x02,
            write_input: 0x02,
            write_work: 0x03,
            write_control: 0x01,
            write_output: 0x01,
            move_input: 0x01,
            move_work: 0x01,
            move_control: 0x01,
            move_output: 0x01,
        };
        let buf = t.encode();
        let t2 = TransitionRecord::decode(&buf);
        assert_eq!(t2.id, 1);
        assert_eq!(t2.read_input, 0x01);
        assert_eq!(t2.write_work, 0x03);
    }

    #[test]
    fn table_roundtrip() {
        let mut meta = TMTTMetadata::default_for_ptm();
        meta.num_states = 2;
        meta.num_transitions = 1;
        meta.initial_state = 0;
        meta.accepting_state = 1;
        meta.rejecting_state = 2;
        meta.halting_state = 3;
        let mut table = TransitionTable::new(meta);
        table.add_state(0, StateType::Initial);
        table.add_state(1, StateType::Accepting);
        table.add_transition(TransitionRecord {
            id: 0,
            source_state: 0,
            target_state: 1,
            read_input: 0x01,
            read_work: 0x02,
            read_control: 0x00,
            read_output: 0x02,
            write_input: 0x02,
            write_work: 0x03,
            write_control: 0x01,
            write_output: 0x01,
            move_input: 0x01,
            move_work: 0x01,
            move_control: 0x01,
            move_output: 0x01,
        });
        let buf = table.encode();
        let table2 = TransitionTable::decode(&buf).unwrap();
        assert_eq!(table2.metadata.num_states, 2);
        assert_eq!(table2.transitions.len(), 1);
    }
}
