// PTM state: finite control for the Polynomial Turing Machine.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StateType {
    Normal = 0x01,
    Initial = 0x02,
    Accepting = 0x03,
    Rejecting = 0x04,
    Halting = 0x05,
}

impl StateType {
    pub fn from_u8(v: u8) -> Option<Self> {
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
pub struct PTMState {
    pub index: u32,
    pub state_type: StateType,
    pub transitions: Vec<u32>, // transition indices
}

impl PTMState {
    pub fn new(index: u32, state_type: StateType) -> Self {
        PTMState { index, state_type, transitions: Vec::new() }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self.state_type, StateType::Accepting | StateType::Rejecting | StateType::Halting)
    }
}

#[derive(Debug, Clone)]
pub struct Transition {
    pub id: u32,
    pub source_state: u32,
    pub target_state: u32,
    // Read symbols (4 tapes)
    pub read_input: u8,
    pub read_work: u8,
    pub read_control: u8,
    pub read_output: u8,
    // Write symbols (4 tapes)
    pub write_input: u8,
    pub write_work: u8,
    pub write_control: u8,
    pub write_output: u8,
    // Head movements (4 tapes)
    pub move_input: u8,
    pub move_work: u8,
    pub move_control: u8,
    pub move_output: u8,
}

impl Transition {
    pub fn is_valid(&self) -> bool {
        // Input tape head cannot move left
        if self.move_input == 0x00 { return false; }
        // Output tape head cannot move left
        if self.move_output == 0x00 { return false; }
        true
    }
}
