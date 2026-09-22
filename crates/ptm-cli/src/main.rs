// ptm-cli: Command-line interface for Polynomial Turing Machine.

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        process::exit(1);
    }

    match args[0].as_str() {
        "validate" => cmd_validate(&args[1..]),
        "simulate" => cmd_simulate(&args[1..]),
        "synthesize" => cmd_synthesize(&args[1..]),
        "analyze" => cmd_analyze(&args[1..]),
        "info" => cmd_info(),
        _ => {
            eprintln!("Unknown command: {}", args[0]);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: ptm-cli <command> [options]\n");
    eprintln!("Commands:");
    eprintln!("  validate   <file>           Validate a PTM binary artifact");
    eprintln!("  simulate   <file> [input]   Run PTM simulation");
    eprintln!("  synthesize <file>           Generate Verilog for Cyclone FPGA");
    eprintln!("  analyze    <file>           Analyze execution trace");
    eprintln!("  info                        Show tool info");
}

fn cmd_validate(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: ptm-cli validate <file>");
        process::exit(1);
    }
    let path = &args[0];
    match fs::read(path) {
        Ok(data) => {
            if data.len() < 16 {
                eprintln!("ERROR: File too small for header");
                process::exit(1);
            }
            let mut hdr = [0u8; 16];
            hdr.copy_from_slice(&data[0..16]);
            match ptm_core::header::FileHeader::decode(&hdr) {
                Ok(header) => {
                    println!("Magic: {}", header.magic.as_str());
                    println!("Version: 0x{:04X}", header.version);
                    println!("Payload length: {}", header.payload_len);
                    println!("Total file size: {}", data.len());
                    if data.len() < 16 + header.payload_len as usize {
                        eprintln!("WARNING: File shorter than declared payload");
                    } else if data.len() > 16 + header.payload_len as usize {
                        eprintln!("WARNING: Trailing bytes detected");
                    }
                    println!("Status: VALID");
                }
                Err(e) => {
                    eprintln!("ERROR: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("ERROR reading file: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_simulate(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: ptm-cli simulate <file> [input_bits]");
        process::exit(1);
    }
    let path = &args[0];
    let input_bits: Vec<u8> = if args.len() > 1 {
        args[1].bytes().map(|b| if b == b'1' { 1 } else { 0 }).collect()
    } else {
        vec![1, 0, 1]
    };

    match fs::read(path) {
        Ok(data) => {
            match ptm_engine::transition_table::TransitionTable::decode(&data) {
                Ok(table) => {
                    println!("Loaded transition table:");
                    println!("  States: {}", table.metadata.num_states);
                    println!("  Transitions: {}", table.metadata.num_transitions);
                    println!("  Input alphabet: {}", table.metadata.input_alphabet_size);
                    println!("  Work alphabet: {}", table.metadata.work_alphabet_size);

                    let config = ptm_engine::simulator::SimulationConfig::default();
                    let mut sim = ptm_engine::simulator::Simulator::new(config);
                    sim.init_input(&input_bits);

                    println!("Running simulation with input: {:?}", input_bits);
                    let result = sim.run(&table);
                    match result {
                        ptm_engine::simulator::StepResult::Halted { state_type } => {
                            println!("Result: HALTED ({:?})", state_type);
                        }
                        ptm_engine::simulator::StepResult::Rejected => {
                            println!("Result: REJECTED");
                        }
                        ptm_engine::simulator::StepResult::MaxStepsExceeded => {
                            println!("Result: MAX STEPS EXCEEDED");
                        }
                        ptm_engine::simulator::StepResult::NoTransition => {
                            println!("Result: NO TRANSITION (stuck)");
                        }
                        _ => {}
                    }
                    println!("Steps: {}", sim.step_count);
                    println!("Snapshots taken: {}", sim.snapshots.len());
                }
                Err(e) => {
                    eprintln!("ERROR decoding transition table: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("ERROR reading file: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_synthesize(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: ptm-cli synthesize <file>");
        process::exit(1);
    }
    let path = &args[0];
    match fs::read(path) {
        Ok(data) => {
            match ptm_engine::transition_table::TransitionTable::decode(&data) {
                Ok(table) => {
                    let gen = ptm_fpga::verilog_gen::VerilogGenerator::default();
                    let verilog = gen.generate(&table);
                    let tb = gen.generate_testbench(&table);

                    let out_path = path.replace(".tmtt", "_top.sv");
                    let tb_path = path.replace(".tmtt", "_tb.sv");

                    fs::write(&out_path, &verilog).expect("write top module");
                    fs::write(&tb_path, &tb).expect("write testbench");

                    println!("Generated:");
                    println!("  {}", out_path);
                    println!("  {}", tb_path);

                    let est = ptm_fpga::cyclone_target::ResourceEstimate::from_table(
                        table.metadata.num_states,
                        table.metadata.num_transitions,
                        256,
                    );
                    let dev = ptm_fpga::cyclone_target::CycloneDevice::CycloneV;
                    println!("\n{}", est.report(&dev));
                }
                Err(e) => {
                    eprintln!("ERROR: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("ERROR reading file: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_analyze(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: ptm-cli analyze <file>");
        process::exit(1);
    }
    let path = &args[0];
    match fs::read(path) {
        Ok(data) => {
            if data.len() < 48 {
                eprintln!("File too small for analysis");
                process::exit(1);
            }
            let mut meta_buf = [0u8; 32];
            meta_buf.copy_from_slice(&data[16..48]);
            let meta = ptm_engine::snapshot::SnapshotMetadata::decode(&meta_buf);
            println!("Snapshot at step {}", meta.step_number);
            println!("  State: {}", meta.current_state);
            println!("  Input head: {}", meta.input_head);
            println!("  Work head: {}", meta.work_head);

            let mut analyzer = ptm_ml::trace_analyzer::TraceAnalyzer::new();
            analyzer.events.push(ptm_ml::trace_analyzer::TraceEvent {
                step: meta.step_number as u64,
                state: meta.current_state,
                input_head: meta.input_head,
                work_head: meta.work_head,
                output_head: meta.output_head,
            });
            println!("\n{}", analyzer.summary());
        }
        Err(e) => {
            eprintln!("ERROR: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_info() {
    println!("PTM Binary Toolkit v0.1.0");
    println!("Polynomial Turing Machine — CPL/Cyclone synthesis target");
    println!();
    println!("Crate: ptm-core     Binary foundation (headers, CRC-32, magic)");
    println!("Crate: ptm-engine   Transition table + 4-tape simulator");
    println!("Crate: ptm-fpga     Verilog generation for Intel Cyclone FPGAs");
    println!("Crate: ptm-ml       ML trace analysis + complexity verification");
    println!();
    println!("License: NLNPL-1.0 (Node-Locked Network Public License)");
    println!("Spec: BLRD-PTM-2026-001");
}
