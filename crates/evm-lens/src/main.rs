use clap::Parser;
use colored::*;
use evm_lens_core::disassemble;

#[derive(Parser)]
struct Args {
    hex: String,
}

fn categorize_opcode(opcode_str: &str) -> ColoredString {
    match opcode_str {
        // Stack operations - Green
        op if op.starts_with("PUSH") => op.bright_green().bold(),
        op if op.starts_with("POP") => op.green(),
        op if op.starts_with("DUP") => op.green(),
        op if op.starts_with("SWAP") => op.green(),

        // Arithmetic - Yellow
        "ADD" | "SUB" | "MUL" | "DIV" | "MOD" | "ADDMOD" | "MULMOD" => {
            opcode_str.bright_yellow().bold()
        }
        "LT" | "GT" | "SLT" | "SGT" | "EQ" | "ISZERO" => opcode_str.yellow(),

        // Memory operations - Blue
        "MLOAD" | "MSTORE" | "MSTORE8" | "MSIZE" | "MCOPY" => opcode_str.bright_blue().bold(),

        // Storage operations - Magenta
        "SLOAD" | "SSTORE" => opcode_str.bright_magenta().bold(),

        // Crypto/Hash - Cyan
        "KECCAK256" => opcode_str.bright_cyan().bold(),

        // Control flow - Red
        "JUMP" | "JUMPI" | "JUMPDEST" => opcode_str.bright_red().bold(),
        "CALL" | "CALLCODE" | "DELEGATECALL" | "STATICCALL" => opcode_str.red().bold(),
        "CREATE" | "CREATE2" => opcode_str.red(),

        // End operations - White
        "STOP" | "RETURN" | "REVERT" | "SELFDESTRUCT" => opcode_str.bright_white().bold(),

        // Default - Normal
        _ => opcode_str.normal(),
    }
}

fn print_header() {
    println!("{}", "EVM BYTECODE DISASSEMBLY".bright_blue().bold());
    println!("{}", "=".repeat(50).bright_black());
}

fn print_footer(total_opcodes: usize) {
    println!("{}", "=".repeat(50).bright_black());
    println!(
        "{} {}",
        total_opcodes.to_string().bright_green().bold(),
        "opcodes total".bright_black()
    );
}

fn print_opcode(position: usize, opcode: &str) {
    let colored_opcode = categorize_opcode(opcode);

    println!(
        "{} {} {}",
        format!("{:04x}", position).bright_black(),
        "│".bright_black(),
        colored_opcode
    );
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let cleaned = args.hex.trim_start_matches("0x");

    let bytes = hex::decode(cleaned)?;
    let ops = disassemble(&bytes).map_err(|e| color_eyre::eyre::eyre!(e))?;

    print_header();

    for (position, opcode) in ops.iter() {
        print_opcode(*position, opcode.as_str());
    }

    print_footer(ops.len());

    Ok(())
}
