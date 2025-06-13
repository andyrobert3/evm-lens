# EVM Lens

[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
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
- Simple command-line interface
- Support for hex strings with/without `0x` prefix
- Beautiful error reporting

## 🚀 Quick Start

### Install the CLI

```bash
cargo install evm-lens
```

### Use as a Library

```toml
[dependencies]
evm-lens-core = "0.1.0"
```

### Example Usage

**CLI:**
```bash
evm-lens 60FF61ABCD00
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

- **🚀 Fast**: Built on revm's optimized EVM implementation
- **🎨 Beautiful**: Color-coded output for different opcode categories
- **✅ Accurate**: Proper handling of PUSH instruction immediates
- **📍 Precise**: Exact position tracking for all opcodes


## 🎯 Use Cases

- **Smart Contract Analysis**: Understand deployed contract behavior
- **Security Research**: Analyze suspicious or malicious contracts  
- **Bytecode Debugging**: Debug Solidity compilation issues
- **Education**: Learn EVM opcodes and instruction structure
- **Development Tools**: Build bytecode analysis into your workflow

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


## 🔧 Development

### Prerequisites

- Rust 1.70+ (2021 edition)
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
# Run the CLI
cargo run -p evm-lens -- 60FF61ABCD00

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