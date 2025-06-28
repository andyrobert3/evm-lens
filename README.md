# EVM Lens

[![CI](https://github.com/andyrobert3/evm-lens/workflows/CI/badge.svg)](https://github.com/andyrobert3/evm-lens/actions)
[![Rust](https://img.shields.io/badge/rust-1.85+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Fast and colorful EVM bytecode disassembler**

EVM Lens is a high-performance Ethereum Virtual Machine (EVM) bytecode disassembler written in Rust. It provides both a library (`evm-lens-core`) and a beautiful command-line tool (`evm-lens`) for analyzing EVM bytecode.

## 📦 Crates

This workspace contains two crates:

### [`evm-lens-core`](./evm-lens-core) - The Core Library
- Fast EVM bytecode disassembly using revm
- Position-accurate opcode extraction  
- Result-based error handling
- Zero-copy iteration where possible

### [`evm-lens`](./evm-lens) - The CLI Tool  
- Colorful terminal output with opcode categorization
- Multiple input methods: direct hex, files, stdin, and blockchain
- Support for hex strings with/without `0x` prefix
- On-chain bytecode fetching via Ethereum RPC
- Beautiful error reporting

## 🚀 Quick Start

### Install the CLI

```bash
cargo install evm-lens
```

### Use as a Library

```toml
[dependencies]
evm-lens-core = "2.0.0"
```

### Example Usage

**CLI:**
```bash
# From command line argument
evm-lens 60FF61ABCD00

# From file
evm-lens --file bytecode.txt

# From stdin
echo "0x60FF61ABCD00" | evm-lens --stdin

# From contract address (fetches from blockchain)
evm-lens --address 0x123... --rpc https://eth.llamarpc.com

# Show bytecode statistics
evm-lens 60FF61ABCD00 --stats

# Decode function selectors with ABI resolution
evm-lens 63a9059cbb00 --abi
```

**Library:**
```rust
use lens_core::disassemble;

let bytecode = hex::decode("60FF61ABCD00")?;
let ops = disassemble(&bytecode)?;
for (position, opcode) in ops {
    println!("{:04x}: {:?}", position, opcode);
}
```

## 🎨 Features

**Core Capabilities:**
- **🔍 Disassemble EVM bytecode** from multiple sources - hex strings, files, stdin, and live contract addresses
- **📊 Generate statistics summary** including bytecode length, number of opcodes, and maximum stack depth
- **🎯 Decode function selectors** - automatically resolve PUSH4 instructions to human-readable function signatures using 4byte.directory



## 📥 Input Methods

EVM Lens supports multiple ways to provide bytecode for analysis:

### Direct Hex Input
```bash
evm-lens 60FF61ABCD00         # Without 0x prefix
evm-lens 0x60FF61ABCD00       # With 0x prefix
```

### File Input
```bash
evm-lens --file bytecode.txt  # Read from file
```

### Standard Input
```bash
echo "0x60FF61ABCD00" | evm-lens --stdin
cat bytecode.txt | evm-lens --stdin
```

### Blockchain Input
```bash
# Use default RPC (eth.llamarpc.com)
evm-lens --address 0x123...

# Use custom RPC endpoint
evm-lens --address 0x123... --rpc https://mainnet.infura.io/v3/YOUR_KEY
evm-lens --address 0x123... --rpc https://eth.llamarpc.com
```

## 🎯 ABI Function Selector Decoding

EVM Lens can automatically decode 4-byte function selectors found in PUSH4 instructions to their human-readable function signatures:

```bash
# Decode function selectors using --abi flag
evm-lens 63a9059cbb00 --abi

# Works with any input method
evm-lens --file contract.hex --abi
echo "0x63a9059cbb00" | evm-lens --stdin --abi
evm-lens --address 0x123... --rpc https://eth.llamarpc.com --abi

# Combine with stats for comprehensive analysis
evm-lens 63a9059cbb00 --abi --stats
```


## 📊 Example Output

```
EVM BYTECODE DISASSEMBLY
==================================================
0000 │ PUSH1     # Stack operation (green)
0002 │ PUSH2     # Stack operation (green)  
0005 │ ADD       # Arithmetic (yellow)
0006 │ MSTORE    # Memory operation (blue)
0007 │ RETURN    # Termination (white)
==================================================
5 opcodes total
```

**With `--stats` flag:**
```
EVM BYTECODE DISASSEMBLY
==================================================
0000 │ PUSH1
0002 │ PUSH2
0005 │ STOP
==================================================
3 opcodes total
BYTECODE STATISTICS
==================================================
Byte length: 6
Number of opcodes: 3
Max stack depth: 2
```

**With `--abi` flag (function selector decoding):**
```
EVM BYTECODE DISASSEMBLY
==================================================
0000 │ PUSH4  # 0xa9059cbb → transfer(address,uint256)
0005 │ PUSH20
001a │ PUSH9
0024 │ BLOCKHASH
==================================================
4 opcodes total
```


## 🔧 Development

### Prerequisites

- Rust 1.85+ (2024 edition)
- Cargo

### Building

```bash
git clone https://github.com/andyrobert3/evm-lens
cd evm-lens
cargo build --release
```

### Testing

```bash
cargo test --workspace
```

### Running Examples

```bash
# Run the CLI with different input methods
cargo run -p evm-lens -- 60FF61ABCD00
cargo run -p evm-lens -- --file examples/bytecode.txt
echo "0x60FF61ABCD00" | cargo run -p evm-lens -- --stdin
cargo run -p evm-lens -- --address 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --rpc https://eth.llamarpc.com

# Test ABI function selector decoding
cargo run -p evm-lens -- 63a9059cbb00 --abi

# Test the library
cargo run --example basic -p evm-lens-core
```

## 📋 Supported Opcodes

All standard EVM opcodes are supported:

| Category | Examples |
|----------|----------|
| **Stack** | PUSH1-PUSH32, POP, DUP1-DUP16, SWAP1-SWAP16 |
| **Arithmetic** | ADD, SUB, MUL, DIV, MOD, ADDMOD, MULMOD |
| **Comparison** | LT, GT, SLT, SGT, EQ, ISZERO |
| **Bitwise** | AND, OR, XOR, NOT, BYTE, SHL, SHR, SAR |
| **Memory** | MLOAD, MSTORE, MSTORE8, MSIZE, MCOPY |
| **Storage** | SLOAD, SSTORE, TLOAD, TSTORE |
| **Control** | JUMP, JUMPI, JUMPDEST, PC, GAS |
| **Block Info** | BLOCKHASH, COINBASE, TIMESTAMP, NUMBER |
| **Calls** | CALL, CALLCODE, DELEGATECALL, STATICCALL |
| **Create** | CREATE, CREATE2 |
| **Termination** | STOP, RETURN, REVERT, SELFDESTRUCT |
| **Crypto** | KECCAK256, ECRECOVER |

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [revm](https://github.com/bluealloy/revm) - High-performance EVM implementation
- The Ethereum community for EVM specifications
****