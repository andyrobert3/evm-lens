use clap::Parser;
use colored::*;
use evm_lens_core::disassemble;

#[derive(Parser)]
#[command(
    name = "evm-lens",
    version,
    about = "A colorful EVM bytecode disassembler",
    after_help = "EXAMPLES:
    evm-lens 60FF                    # Simple PUSH1 instruction
    evm-lens 0x60FF61ABCD00          # Multiple instructions with 0x prefix
    evm-lens 602060005260005100      # Memory operations (MSTORE/MLOAD)
    evm-lens 6001600280900100        # Stack operations (DUP/SWAP/ADD)

For more information, visit: https://github.com/andyrobert3/evm-lens"
)]
struct Args {
    #[arg(
        help = "Hexadecimal EVM bytecode to disassemble",
        value_name = "BYTECODE"
    )]
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

fn print_error(message: &str) {
    eprintln!("{} {}", "Error:".bright_red().bold(), message);
}

fn print_usage_hint() {
    eprintln!();
    eprintln!("{}", "Usage examples:".bright_blue().bold());
    eprintln!("  {} 60FF61ABCD00", "evm-lens".bright_green());
    eprintln!("  {} 0x60FF61ABCD00", "evm-lens".bright_green());
    eprintln!();
    eprintln!(
        "{}",
        "The input should be valid hexadecimal EVM bytecode.".bright_black()
    );
}

fn validate_and_decode_hex(hex_input: &str) -> Result<Vec<u8>, String> {
    let cleaned = hex_input.trim_start_matches("0x");

    if cleaned.is_empty() {
        return Err("Empty hex string provided".to_string());
    }

    if cleaned.len() % 2 != 0 {
        return Err(format!(
            "Invalid hex string length ({}). Hex strings must have an even number of characters",
            cleaned.len()
        ));
    }

    if !cleaned.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid hex characters found. Only 0-9, a-f, and A-F are allowed".to_string());
    }

    // Decode the hex string
    hex::decode(cleaned).map_err(|e| format!("Failed to decode hex string: {}", e))
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let bytes = match validate_and_decode_hex(&args.hex) {
        Ok(bytes) => bytes,
        Err(e) => {
            print_error(&e);
            print_usage_hint();
            std::process::exit(1);
        }
    };

    let ops = match disassemble(&bytes) {
        Ok(ops) => ops,
        Err(e) => {
            print_error(&format!("Failed to disassemble bytecode: {}", e));
            eprintln!();
            eprintln!("{}", "This could happen if:".bright_blue().bold());
            eprintln!("  • The bytecode is malformed or incomplete");
            eprintln!("  • The bytecode contains invalid opcodes");
            eprintln!("  • The bytecode structure is corrupted");
            print_usage_hint();
            std::process::exit(1);
        }
    };

    if ops.is_empty() {
        print_error("No opcodes found in the provided bytecode");
        print_usage_hint();
        std::process::exit(1);
    }

    print_header();

    for (position, opcode) in ops.iter() {
        print_opcode(*position, opcode.as_str());
    }

    print_footer(ops.len());

    Ok(())
}
