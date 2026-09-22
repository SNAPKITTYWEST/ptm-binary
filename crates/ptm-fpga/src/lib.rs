// ptm-fpga: CPL/Cyclone FPGA synthesis target.
// Generates synthesizable Verilog for Intel Cyclone FPGAs from PTM transition tables.

pub mod verilog_gen;
pub mod cyclone_target;
pub mod state_machine;
pub mod tape_ctrl;

pub use verilog_gen::VerilogGenerator;
pub use cyclone_target::{CycloneDevice, ResourceEstimate};
pub use state_machine::{StateEncoder, EncodingType};
pub use tape_ctrl::TapeController;
