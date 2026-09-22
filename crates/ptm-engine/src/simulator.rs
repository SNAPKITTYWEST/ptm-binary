// Simulator: 4-tape Polynomial Turing Machine execution engine.

use ptm_core::tape::{Tape, HeadMove};
use crate::transition_table::TransitionTable;
use crate::snapshot::{Snapshot, SnapshotMetadata};

#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub max_steps: u64,
    pub snapshot_interval: u64,
    pub poly_degree: u32,
    pub poly_constant: f64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        SimulationConfig {
            max_steps: 1_000_000,
            snapshot_interval: 1000,
            poly_degree: 1,
            poly_constant: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepResult {
    Continue,
    Halted { state_type: StateTypeOutput },
    Rejected,
    MaxStepsExceeded,
    NoTransition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateTypeOutput {
    Accepting,
    Rejecting,
    Halting,
}

#[derive(Debug)]
pub struct Simulator {
    pub tape_input: Tape,
    pub tape_work: Tape,
    pub tape_control: Tape,
    pub tape_output: Tape,
    pub current_state: u32,
    pub step_count: u64,
    pub config: SimulationConfig,
    pub snapshots: Vec<Snapshot>,
}

impl Simulator {
    pub fn new(config: SimulationConfig) -> Self {
        Simulator {
            tape_input: Tape::new(),
            tape_work: Tape::new(),
            tape_control: Tape::new(),
            tape_output: Tape::new(),
            current_state: 0,
            step_count: 0,
            config,
            snapshots: Vec::new(),
        }
    }

    pub fn init_input(&mut self, bits: &[u8]) {
        self.tape_input.init_from_bits(bits);
    }

    pub fn init_control(&mut self, bits: &[u8]) {
        self.tape_control.init_from_bits(bits);
    }

    pub fn step(&mut self, table: &TransitionTable) -> StepResult {
        // Check polynomial bound
        let input_len = self.tape_input.content_len() as u64;
        let bound = (self.config.poly_constant * (input_len as f64).powi(self.config.poly_degree as i32)) as u64;
        if self.step_count >= bound && bound > 0 {
            return StepResult::MaxStepsExceeded;
        }
        if self.step_count >= self.config.max_steps {
            return StepResult::MaxStepsExceeded;
        }

        let r_in = self.tape_input.read();
        let r_work = self.tape_work.read();
        let r_ctrl = self.tape_control.read();
        let r_out = self.tape_output.read();

        let transition = match table.find_transition(self.current_state, r_in, r_work, r_ctrl, r_out) {
            Some(t) => t,
            None => return StepResult::NoTransition,
        };

        // Write symbols
        self.tape_input.write(transition.write_input);
        self.tape_work.write(transition.write_work);
        self.tape_control.write(transition.write_control);
        self.tape_output.write(transition.write_output);

        // Move heads
        self.tape_input.move_head(decode_move(transition.move_input));
        self.tape_work.move_head(decode_move(transition.move_work));
        self.tape_control.move_head(decode_move(transition.move_control));
        self.tape_output.move_head(decode_move(transition.move_output));

        // Transition state
        self.current_state = transition.target_state;
        self.step_count += 1;

        // Check if halted
        if let Some(state) = table.states.iter().find(|s| s.index == self.current_state) {
            match state.state_type {
                crate::transition_table::StateType::Accepting => {
                    return StepResult::Halted { state_type: StateTypeOutput::Accepting };
                }
                crate::transition_table::StateType::Rejecting => {
                    return StepResult::Rejected;
                }
                crate::transition_table::StateType::Halting => {
                    return StepResult::Halted { state_type: StateTypeOutput::Halting };
                }
                _ => {}
            }
        }

        StepResult::Continue
    }

    pub fn take_snapshot(&mut self) -> Snapshot {
        let meta = SnapshotMetadata {
            step_number: self.step_count as u32,
            current_state: self.current_state,
            input_head: self.tape_input.head as u32,
            work_head: self.tape_work.head as u32,
            control_head: self.tape_control.head as u32,
            output_head: self.tape_output.head as u32,
            work_tape_len: self.tape_work.content_len() as u32,
            control_tape_len: self.tape_control.content_len() as u32,
        };
        let snap = Snapshot::new(meta);
        self.snapshots.push(snap.clone());
        snap
    }

    pub fn run(&mut self, table: &TransitionTable) -> StepResult {
        loop {
            let result = self.step(table);
            if self.step_count % self.config.snapshot_interval == 0 {
                self.take_snapshot();
            }
            match result {
                StepResult::Continue => continue,
                other => return other,
            }
        }
    }
}

fn decode_move(v: u8) -> HeadMove {
    match v {
        0x00 => HeadMove::Left,
        0x01 => HeadMove::Right,
        _ => HeadMove::Stay,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transition_table::{TransitionTable, TMTTMetadata, StateType, TransitionRecord};

    fn build_simple_table() -> TransitionTable {
        let mut meta = TMTTMetadata::default_for_ptm();
        meta.num_states = 3;
        meta.num_transitions = 2;
        meta.initial_state = 0;
        meta.accepting_state = 2;
        meta.rejecting_state = 3;
        meta.halting_state = 4;
        let mut table = TransitionTable::new(meta);
        table.add_state(0, StateType::Initial);
        table.add_state(1, StateType::Normal);
        table.add_state(2, StateType::Accepting);
        // Transition: state 0, read 1 on input -> write B, move R, go to state 1
        table.add_transition(TransitionRecord {
            id: 0,
            source_state: 0,
            target_state: 1,
            read_input: 0x01,
            read_work: 0x02,
            read_control: 0x02,
            read_output: 0x02,
            write_input: 0x02,
            write_work: 0x02,
            write_control: 0x02,
            write_output: 0x02,
            move_input: 0x01,
            move_work: 0x02,
            move_control: 0x02,
            move_output: 0x02,
        });
        // Transition: state 1, read B on input -> write B, stay, go to state 2
        table.add_transition(TransitionRecord {
            id: 1,
            source_state: 1,
            target_state: 2,
            read_input: 0x02,
            read_work: 0x02,
            read_control: 0x02,
            read_output: 0x02,
            write_input: 0x02,
            write_work: 0x02,
            write_control: 0x02,
            write_output: 0x02,
            move_input: 0x02,
            move_work: 0x02,
            move_control: 0x02,
            move_output: 0x02,
        });
        table
    }

    #[test]
    fn simulator_halts() {
        let table = build_simple_table();
        let mut sim = Simulator::new(SimulationConfig::default());
        sim.init_input(&[1]); // single bit '1'
        let result = sim.run(&table);
        assert_eq!(result, StepResult::Halted { state_type: StateTypeOutput::Accepting });
        assert_eq!(sim.step_count, 2);
    }
}
