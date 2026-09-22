// Verilog generator: produces synthesizable SystemVerilog from PTM transition tables.
// Targets Intel Cyclone V / Cyclone 10 LP FPGAs via Quartus.

use ptm_engine::transition_table::{TransitionTable, StateType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Verilog2001,
    SystemVerilog,
}

#[derive(Debug, Clone)]
pub struct VerilogGenerator {
    pub format: OutputFormat,
    pub module_name: String,
    pub clock_name: String,
    pub reset_name: String,
    pub include_comments: bool,
}

impl Default for VerilogGenerator {
    fn default() -> Self {
        VerilogGenerator {
            format: OutputFormat::SystemVerilog,
            module_name: "ptm_top".into(),
            clock_name: "clk".into(),
            reset_name: "rst_n".into(),
            include_comments: true,
        }
    }
}

impl VerilogGenerator {
    pub fn generate(&self, table: &TransitionTable) -> String {
        let mut o = String::with_capacity(4096);
        let m = &self.module_name;
        let clk = &self.clock_name;
        let rst = &self.reset_name;

        if self.include_comments {
            o.push_str("// Auto-generated PTM FPGA top-level module.\n");
            o.push_str("// Polynomial Turing Machine - CPL/Cyclone synthesis target.\n");
            o.push_str("// DO NOT EDIT MANUALLY.\n\n");
        }

        o.push_str(&format!("module {} (\n", m));
        o.push_str(&format!("    input  logic        {},\n", clk));
        o.push_str(&format!("    input  logic        {},\n", rst));
        o.push_str(&format!("    input  logic        {}_start,\n", m));
        o.push_str(&format!("    input  logic [31:0] {}_input_len,\n", m));
        o.push_str(&format!("    output logic        {}_done,\n", m));
        o.push_str(&format!("    output logic        {}_accept,\n", m));
        o.push_str(&format!("    output logic        {}_reject,\n", m));
        o.push_str(&format!("    output logic [31:0] {}_step_count\n", m));
        o.push_str(");\n\n");

        o.push_str(&format!("    parameter NUM_STATES = {};\n", table.metadata.num_states));
        o.push_str(&format!("    parameter NUM_TRANSITIONS = {};\n", table.metadata.num_transitions));
        o.push_str(&format!("    parameter INITIAL_STATE = {};\n", table.metadata.initial_state));
        o.push_str(&format!("    parameter ACCEPTING_STATE = {};\n", table.metadata.accepting_state));
        o.push_str(&format!("    parameter INPUT_ALPHABET = {};\n", table.metadata.input_alphabet_size));
        o.push_str(&format!("    parameter WORK_ALPHABET = {};\n", table.metadata.work_alphabet_size));
        o.push('\n');

        // State encoding
        o.push_str("    typedef enum logic [31:0] {\n");
        for (i, state) in table.states.iter().enumerate() {
            let suffix = match state.state_type {
                StateType::Initial => " // INITIAL",
                StateType::Accepting => " // ACCEPTING",
                StateType::Rejecting => " // REJECTING",
                StateType::Halting => " // HALTING",
                _ => "",
            };
            let comma = if i < table.states.len() - 1 { "," } else { "" };
            o.push_str(&format!("        S_{} = 32'd{}{}\n", state.index, state.index, suffix));
            if !comma.is_empty() {
                o.push_str(comma);
            }
        }
        o.push_str("    } state_t;\n\n");

        o.push_str("    state_t current_state, next_state;\n");
        o.push_str("    logic [31:0] step_counter;\n");
        o.push_str("    logic [7:0]  tape_input [0:1023];\n");
        o.push_str("    logic [7:0]  tape_work [0:1023];\n");
        o.push_str("    logic [7:0]  tape_control [0:1023];\n");
        o.push_str("    logic [7:0]  tape_output [0:1023];\n");
        o.push_str("    logic [31:0] head_input, head_work, head_control, head_output;\n");
        o.push_str("    logic        input_done;\n\n");

        // State register
        o.push_str(&format!("    always_ff @(posedge {} or negedge {}) begin\n", clk, rst));
        o.push_str(&format!("        if (!{}) begin\n", rst));
        o.push_str(&format!("            current_state <= S_{};\n", table.metadata.initial_state));
        o.push_str("            step_counter <= 32'd0;\n");
        o.push_str("            head_input <= 32'd0;\n");
        o.push_str("            head_work <= 32'd0;\n");
        o.push_str("            head_control <= 32'd0;\n");
        o.push_str("            head_output <= 32'd0;\n");
        o.push_str("            input_done <= 1'b0;\n");
        o.push_str(&format!("        end else if ({}_start && current_state == S_{}) begin\n", m, table.metadata.initial_state));
        o.push_str("            current_state <= next_state;\n");
        o.push_str("            step_counter <= step_counter + 1;\n");
        o.push_str("        end\n");
        o.push_str("    end\n\n");

        // Next state logic
        o.push_str("    always_comb begin\n");
        o.push_str("        next_state = current_state;\n");
        o.push_str(&format!("        {}_done = 1'b0;\n", m));
        o.push_str(&format!("        {}_accept = 1'b0;\n", m));
        o.push_str(&format!("        {}_reject = 1'b0;\n", m));

        for state in &table.states {
            let state_transitions: Vec<_> = table.transitions.iter()
                .filter(|t| t.source_state == state.index)
                .collect();
            if state_transitions.is_empty() {
                match state.state_type {
                    StateType::Accepting => {
                        o.push_str(&format!("        if (current_state == S_{}) begin\n", state.index));
                        o.push_str(&format!("            {}_done = 1'b1;\n", m));
                        o.push_str(&format!("            {}_accept = 1'b1;\n", m));
                        o.push_str("        end\n");
                    }
                    StateType::Rejecting | StateType::Halting => {
                        o.push_str(&format!("        if (current_state == S_{}) begin\n", state.index));
                        o.push_str(&format!("            {}_done = 1'b1;\n", m));
                        o.push_str(&format!("            {}_reject = 1'b1;\n", m));
                        o.push_str("        end\n");
                    }
                    _ => {}
                }
                continue;
            }

            o.push_str(&format!("        if (current_state == S_{}) begin\n", state.index));
            for t in &state_transitions {
                let input_delta = if t.move_input == 0x01 { "1" } else { "0" };
                let work_delta = if t.move_work == 0x01 { "1" } else { "32'hFFFFFFFF" };
                o.push_str(&format!(
                    "            if (tape_input[head_input] == 8'h{:02X} && tape_work[head_work] == 8'h{:02X}) begin\n",
                    t.read_input, t.read_work
                ));
                o.push_str(&format!("                tape_input[head_input] <= 8'h{:02X};\n", t.write_input));
                o.push_str(&format!("                tape_work[head_work] <= 8'h{:02X};\n", t.write_work));
                o.push_str(&format!("                head_input <= head_input + 32'd{};\n", input_delta));
                o.push_str(&format!("                head_work <= head_work + {};\n", work_delta));
                o.push_str(&format!("                next_state <= S_{};\n", t.target_state));
                o.push_str("            end\n");
            }
            o.push_str("        end\n");
        }

        o.push_str("    end\n\n");
        o.push_str(&format!("    assign {}_step_count = step_counter;\n\n", m));
        o.push_str("endmodule\n");
        o
    }

    pub fn generate_testbench(&self, table: &TransitionTable) -> String {
        let mut o = String::with_capacity(2048);
        let m = &self.module_name;
        let clk = &self.clock_name;
        let rst = &self.reset_name;

        o.push_str("// Auto-generated PTM testbench.\n\n");
        o.push_str(&format!("module {}_tb;\n\n", m));
        o.push_str(&format!("    logic        {};\n", clk));
        o.push_str(&format!("    logic        {};\n", rst));
        o.push_str(&format!("    logic        {}_start;\n", m));
        o.push_str(&format!("    logic [31:0] {}_input_len;\n", m));
        o.push_str(&format!("    logic        {}_done;\n", m));
        o.push_str(&format!("    logic        {}_accept;\n", m));
        o.push_str(&format!("    logic        {}_reject;\n", m));
        o.push_str(&format!("    logic [31:0] {}_step_count;\n\n", m));

        o.push_str(&format!("    {} #(.NUM_STATES({})) uut (\n", m, table.metadata.num_states));
        o.push_str(&format!("        .{}({}),\n", clk, clk));
        o.push_str(&format!("        .{}({}),\n", rst, rst));
        o.push_str(&format!("        .{}_start({}_start),\n", m, m));
        o.push_str(&format!("        .{}_input_len({}_input_len),\n", m, m));
        o.push_str(&format!("        .{}_done({}_done),\n", m, m));
        o.push_str(&format!("        .{}_accept({}_accept),\n", m, m));
        o.push_str(&format!("        .{}_reject({}_reject),\n", m, m));
        o.push_str(&format!("        .{}_step_count({}_step_count)\n", m, m));
        o.push_str("    );\n\n");

        o.push_str(&format!("    always #5 {} = ~{};\n\n", clk, clk));

        o.push_str("    initial begin\n");
        o.push_str(&format!("        {} = 0;\n", clk));
        o.push_str(&format!("        {} = 0;\n", rst));
        o.push_str(&format!("        {}_start = 0;\n", m));
        o.push_str(&format!("        {}_input_len = 32'd8;\n", m));
        o.push_str("        #10;\n");
        o.push_str(&format!("        {} = 1;\n", rst));
        o.push_str("        #10;\n");
        o.push_str(&format!("        {}_start = 1;\n", m));
        o.push_str("        #10;\n");
        o.push_str(&format!("        {}_start = 0;\n", m));
        o.push_str(&format!("        wait({}_done);\n", m));
        o.push_str("        #10;\n");
        o.push_str(&format!("        if ({}_accept) $display(\"ACCEPTED\");\n", m));
        o.push_str(&format!("        else if ({}_reject) $display(\"REJECTED\");\n", m));
        o.push_str(&format!("        $display(\"Steps: %d\", {}_step_count);\n", m));
        o.push_str("        $finish;\n");
        o.push_str("    end\n\n");
        o.push_str("endmodule\n");
        o
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ptm_engine::transition_table::{TMTTMetadata, TransitionRecord};

    fn build_test_table() -> TransitionTable {
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
            write_work: 0x02,
            write_control: 0x02,
            write_output: 0x02,
            move_input: 0x01,
            move_work: 0x02,
            move_control: 0x02,
            move_output: 0x02,
        });
        table
    }

    #[test]
    fn generates_valid_verilog() {
        let table = build_test_table();
        let gen = VerilogGenerator::default();
        let verilog = gen.generate(&table);
        assert!(verilog.contains("module ptm_top"));
        assert!(verilog.contains("typedef enum"));
        assert!(verilog.contains("S_0"));
        assert!(verilog.contains("S_1"));
    }

    #[test]
    fn generates_testbench() {
        let table = build_test_table();
        let gen = VerilogGenerator::default();
        let tb = gen.generate_testbench(&table);
        assert!(tb.contains("module ptm_top_tb"));
        assert!(tb.contains("ACCEPTED"));
    }
}
