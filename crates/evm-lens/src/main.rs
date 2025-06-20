use clap::Parser;
use colored::*;
use evm_lens_core::{Stats, disassemble, get_stats};
use io::Source;

mod io;

#[derive(Parser)]
#[command(
    name = "evm-lens",
    version,
    about = "A colorful EVM bytecode disassembler",
    after_help = "EXAMPLES:
    evm-lens 60FF                              # Simple PUSH1 instruction from arg
    echo '0x60FF61ABCD00' | evm-lens --stdin   # From stdin
    evm-lens --file bytecode.txt               # From file
    evm-lens --address 0x... --rpc http://...  # From blockchain
    evm-lens 60FF61ABCD00 --stats              # Show disassembly + statistics

For more information, visit: https://github.com/andyrobert3/evm-lens"
)]
struct Args {
    #[arg(
        help = "Hexadecimal EVM bytecode to disassemble (if no other source specified)",
        value_name = "BYTECODE",
        required = false
    )]
    hex: Option<String>,

    #[arg(
        long,
        help = "Read bytecode from stdin",
        conflicts_with_all = ["hex", "file", "address"]
    )]
    stdin: bool,

    #[arg(
        long,
        help = "Read bytecode from file",
        value_name = "FILE",
        conflicts_with_all = ["hex", "stdin", "address"]
    )]
    file: Option<String>,

    #[arg(
        long,
        help = "Ethereum address to fetch bytecode from",
        value_name = "ADDRESS",
        conflicts_with_all = ["hex", "stdin", "file"]
    )]
    address: Option<String>,

    #[arg(
        long,
        help = "RPC endpoint URL for fetching on-chain bytecode",
        value_name = "URL",
        default_value = "https://eth.llamarpc.com"
    )]
    rpc: Option<String>,

    #[arg(long, help = "Show bytecode statistics after disassembly")]
    stats: bool,
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
    eprintln!("  {} --stdin", "evm-lens".bright_green());
    eprintln!("  {} --file bytecode.txt", "evm-lens".bright_green());
    eprintln!(
        "  {} --address 0x123... --rpc https://eth.llamarpc.com",
        "evm-lens".bright_green()
    );
    eprintln!("  {} 60FF61ABCD00 --stats", "evm-lens".bright_green());
    eprintln!();
    eprintln!(
        "{}",
        "The input should be valid hexadecimal EVM bytecode.".bright_black()
    );
}

async fn get_bytes_from_args(args: &Args) -> color_eyre::Result<Vec<u8>> {
    match (&args.hex, &args.address, &args.file, args.stdin) {
        (Some(hex_string), None, None, false) => io::decode_hex(hex_string),
        (None, Some(address_str), None, false) => {
            let rpc_url = args.rpc.as_deref().unwrap_or("https://eth.llamarpc.com");

            let address = address_str
                .parse()
                .map_err(|_| color_eyre::eyre::eyre!("Invalid address: {}", address_str))?;

            let rpc = rpc_url
                .parse()
                .map_err(|_| color_eyre::eyre::eyre!("Invalid RPC URL: {}", rpc_url))?;

            let source = Source::OnChain { address, rpc };
            io::fetch_bytes(source).await
        }
        (None, None, Some(file_path), false) => {
            let source = Source::File(file_path.into());
            io::fetch_bytes(source).await
        }
        (None, None, None, true) => {
            let source = Source::Stdin;
            io::fetch_bytes(source).await
        }
        _ => {
            let source = Source::Stdin;
            io::fetch_bytes(source).await
        }
    }
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let bytes = match get_bytes_from_args(&args).await {
        Ok(bytes) => bytes,
        Err(e) => {
            print_error(&format!("{}", e));
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

    if args.stats {
        println!();
        match get_stats(&bytes) {
            Ok(Stats {
                byte_len,
                opcode_count,
                max_stack_depth,
            }) => {
                println!("{}", "BYTECODE STATISTICS".bright_blue().bold());
                println!("{}", "=".repeat(50).bright_black());
                println!("Byte length: {}", byte_len);
                println!("Number of opcodes: {}", opcode_count);
                println!("Max stack depth: {}", max_stack_depth);
            }
            Err(e) => {
                print_error(&format!("Failed to compute bytecode statistics: {}", e));
            }
        }
    }

    Ok(())
}
