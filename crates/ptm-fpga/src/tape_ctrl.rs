// Tape controller: FPGA tape control logic generation.

#[derive(Debug, Clone)]
pub struct TapeController {
    pub tape_index: usize,
    pub cell_width: u32,
    pub max_cells: u32,
}

impl TapeController {
    pub fn new(tape_index: usize, max_cells: u32) -> Self {
        TapeController { tape_index, cell_width: 8, max_cells }
    }

    pub fn generate_verilog(&self) -> String {
        let prefix = format!("tape{}", self.tape_index);
        let max = self.max_cells - 1;
        let mut o = String::with_capacity(2048);
        o.push_str(&format!("// Tape {} controller\n", self.tape_index));
        o.push_str(&format!("module {}_ctrl (\n", prefix));
        o.push_str("    input  logic        clk,\n");
        o.push_str("    input  logic        rst_n,\n");
        o.push_str("    input  logic [7:0]  write_data,\n");
        o.push_str("    input  logic        write_en,\n");
        o.push_str("    input  logic [1:0]  head_move,  // 00=L, 01=R, 10=S\n");
        o.push_str("    output logic [7:0]  read_data,\n");
        o.push_str("    output logic [31:0] head_pos\n");
        o.push_str(");\n\n");
        o.push_str(&format!("    logic [7:0] mem [0:{}];\n", max));
        o.push_str("    logic [31:0] head;\n\n");
        o.push_str("    always_ff @(posedge clk or negedge rst_n) begin\n");
        o.push_str("        if (!rst_n) begin\n");
        o.push_str("            head <= 32'd0;\n");
        o.push_str(&format!("            for (int i = 0; i <= {}; i++)\n", max));
        o.push_str("                mem[i] <= 8'h02; // blank\n");
        o.push_str("        end else begin\n");
        o.push_str("            if (write_en)\n");
        o.push_str("                mem[head] <= write_data;\n");
        o.push_str("            case (head_move)\n");
        o.push_str("                2'b00: if (head > 0) head <= head - 1; // L\n");
        o.push_str("                2'b01: head <= head + 1;              // R\n");
        o.push_str("                default: ;                              // S\n");
        o.push_str("            endcase\n");
        o.push_str("        end\n");
        o.push_str("    end\n\n");
        o.push_str("    assign read_data = mem[head];\n");
        o.push_str("    assign head_pos = head;\n\n");
        o.push_str("endmodule\n");
        o
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_tape_controller() {
        let ctrl = TapeController::new(0, 256);
        let verilog = ctrl.generate_verilog();
        assert!(verilog.contains("module tape0_ctrl"));
        assert!(verilog.contains("head_move"));
        assert!(verilog.contains("read_data"));
    }
}
