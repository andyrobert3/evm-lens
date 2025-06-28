use clap::Parser;
use colored::*;
use evm_lens_core::{disassemble, get_stats, Stats};
use io::Source;

mod abi_resolver;
mod io;
mod opcode;

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
    evm-lens 60FF61ABCD00 --abi                # Show disassembly + 4byte signatures

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

    #[arg(long, help = "Resolve 4-byte selectors to function signatures")]
    abi: bool,
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
    eprintln!("  {} 60FF61ABCD00 --abi", "evm-lens".bright_green());
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

async fn disassemble_and_display(bytes: &[u8], args: &Args) -> color_eyre::Result<()> {
    let ops = disassemble(bytes).map_err(|e| {
        color_eyre::eyre::eyre!("Failed to disassemble bytecode: {}", e)
    })?;

    if ops.is_empty() {
        return Err(color_eyre::eyre::eyre!("No opcodes found in the provided bytecode"));
    }

    let resolved_sigs = if args.abi {
        Some(abi_resolver::resolve_selectors(&ops, bytes).await?)
    } else {
        None
    };

    print_header();
    for (position, opcode) in ops.iter() {
        opcode::print_opcode(*position, *opcode, bytes, resolved_sigs.as_ref());
    }
    print_footer(ops.len());

    if args.stats {
        print_stats(bytes)?;
    }

    Ok(())
}

fn print_stats(bytes: &[u8]) -> color_eyre::Result<()> {
    match get_stats(bytes) {
        Ok(Stats {
            byte_len,
            opcode_count,
            max_stack_depth,
        }) => {
            println!();
            println!("{}", "BYTECODE STATISTICS".bright_blue().bold());
            println!("{}", "=".repeat(50).bright_black());
            println!("Byte length: {}", byte_len);
            println!("Number of opcodes: {}", opcode_count);
            println!("Max stack depth: {}", max_stack_depth);
            Ok(())
        }
        Err(e) => {
            print_error(&format!("Failed to compute bytecode statistics: {}", e));
            Err(color_eyre::eyre::eyre!("Statistics computation failed"))
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

    if let Err(e) = disassemble_and_display(&bytes, &args).await {
        print_error(&format!("{}", e));
        print_usage_hint();
        std::process::exit(1);
    }

    Ok(())
}
