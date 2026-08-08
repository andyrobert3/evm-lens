

# EVM Lens

[![CI](https://github.com/andyrobert3/evm-lens/workflows/CI/badge.svg)](https://github.com/andyrobert3/evm-lens/actions)
[![Rust](https://img.shields.io/badge/rust-1.85+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Desensamblador rápido y colorido de bytecode EVM**

EVM Lens es un desensamblador de bytecode de la Máquina Virtual de Ethereum (EVM) de alto rendimiento escrito en Rust. Proporciona tanto una biblioteca (`evm-lens-core`) como una herramienta de línea de comandos (`evm-lens`) para analizar bytecode EVM.

## 📦 Paquetes (Crates)

Este espacio de trabajo contiene dos paquetes:

### [`evm-lens-core`](./evm-lens-core) - La Biblioteca Central
- Desensamblaje rápido de bytecode EVM usando revm
- Extracción de opcodes con precisión de posición  
- Manejo de errores basado en resultados
- Iteración sin copias (zero-copy) cuando sea posible

### [`evm-lens`](./evm-lens) - La Herramienta CLI  
- Salida de terminal colorida con categorización de opcodes
- Múltiples métodos de entrada: hex directo, archivos, stdin y blockchain
- Soporte para cadenas hex con o sin prefijo `0x`
- Obtención de bytecode en cadena vía RPC de Ethereum
- Informes de errores claros y detallados

## 🚀 Inicio Rápido

### Instalar la CLI

```bash
cargo install evm-lens
```

### Usar como Biblioteca

```toml
[dependencies]
evm-lens-core = "3.0.0"
```

### Ejemplos de Uso

**CLI:**
```bash
# Desde argumento de línea de comandos
evm-lens 60FF61ABCD00

# Desde un archivo
evm-lens --file bytecode.txt

# Desde stdin
echo "0x60FF61ABCD00" | evm-lens --stdin

# Desde dirección de contrato (obtiene de la blockchain)
evm-lens --address 0x123... --rpc https://eth.llamarpc.com

# Mostrar estadísticas del bytecode
evm-lens 60FF61ABCD00 --stats

# Decodificar selectores de función con resolución de ABI
evm-lens 63a9059cbb00 --abi

# Comparar layouts de almacenamiento (solo archivos)
evm-lens storage-diff artifacts/old.hex artifacts/new.hex
evm-lens storage-diff old.hex new.hex --json target/storage.json --html target/storage.html
evm-lens storage-diff old.hex new.hex --ci
```

**Biblioteca:**
```rust
use lens_core::disassemble;

let bytecode = hex::decode("60FF61ABCD00")?;
let ops = disassemble(&bytecode)?;
for (position, opcode) in ops {
    println!("{:04x}: {:?}", position, opcode);
}
```

## 🎨 Características

**Capacidades Principales:**
- **🔍 Desensamblar bytecode EVM** desde múltiples fuentes: cadenas hex, archivos, stdin y direcciones de contrato en vivo
- **📊 Generar resumen de estadísticas** que incluye longitud del bytecode, número de opcodes y profundidad máxima de la pila
- **🎯 Decodificar selectores de función** - resuelve automáticamente instrucciones PUSH4 a firmas de función legibles usando 4byte.directory
- **🧮 Diferencias de almacenamiento (Storage diff)** - compara dos artefactos y señala cambios en el layout de almacenamiento con informes JSON/HTML y códigos de salida compatibles con CI



## 📥 Métodos de Entrada

EVM Lens admite múltiples formas de proporcionar bytecode para análisis:

### Entrada Hex Directa
```bash
evm-lens 60FF61ABCD00         # Sin prefijo 0x
evm-lens 0x60FF61ABCD00       # Con prefijo 0x
```

### Entrada desde Archivo
```bash
evm-lens --file bytecode.txt  # Leer desde archivo
```

### Entrada Estándar (stdin)
```bash
echo "0x60FF61ABCD00" | evm-lens --stdin
cat bytecode.txt | evm-lens --stdin
```

### Entrada desde Blockchain
```bash
# Usar RPC predeterminado (eth.llamarpc.com)
evm-lens --address 0x123...

# Usar endpoint RPC personalizado
evm-lens --address 0x123... --rpc https://mainnet.infura.io/v3/YOUR_KEY
evm-lens --address 0x123... --rpc https://eth.llamarpc.com
```

## 🎯 Decodificación de Selectores de Función ABI

EVM Lens puede decodificar automáticamente selectores de función de 4 bytes encontrados en instrucciones PUSH4 a sus firmas de función legibles:

```bash
# Decodificar selectores de función usando la bandera --abi
evm-lens 63a9059cbb00 --abi

# Funciona con cualquier método de entrada
evm-lens --file contract.hex --abi
echo "0x63a9059cbb00" | evm-lens --stdin --abi
evm-lens --address 0x123... --rpc https://eth.llamarpc.com --abi

# Combinar con estadísticas para un análisis completo
evm-lens 63a9059cbb00 --abi --stats
```


## 🧮 Diferencias de Almacenamiento (Storage Diff)

Compara dos artefactos compilados (archivos que contienen bytecode de ejecución codificado en hex) y señala riesgos en el layout de almacenamiento.

```bash
evm-lens storage-diff <old.hex> <new.hex> [--json out.json] [--html out.html] [--ci]
```

- Entradas:
  - `<old.hex>`, `<new.hex>`: rutas de archivos que contienen bytecode de ejecución codificado en hex (con o sin 0x).
- Cómo funciona:
  - Construye un StorageLayout para cada entrada usando un resolvedor compuesto:
    1) Metadatos del compilador (si están disponibles), 2) heurística conservadora de bytecode (PUSH… luego SLOAD/SSTORE).
  - Calcula una diferencia por slot con estados: `Same | Added | Removed | TypeChanged | PackingChanged`.
  - Asigna calificaciones: `Ok | Risk | Break` (Added=Ok, Removed/TypeChanged=Break, PackingChanged=Risk).
  - Registra la procedencia por lado: `CompilerMetadata` o `HeuristicTrace`.
- Salidas:
  - Resumen de una línea en CLI con conteos y calificación máxima.
  - Informes JSON (`--json`) y HTML (`--html`) opcionales.
- CI:
  - Con `--ci`, sale con código no cero (código 2) si `max_grade >= Risk`.

Ejemplos:
```bash
# Comparación básica
evm-lens storage-diff artifacts/old.hex artifacts/new.hex

# Escribir informes JSON/HTML
evm-lens storage-diff old.hex new.hex --json target/storage.json --html target/storage.html

# Política de CI (salida no cero en Risk/Break)
evm-lens storage-diff old.hex new.hex --ci
```


## 📊 Ejemplo de Salida

```
EVM BYTECODE DISASSEMBLY
==================================================
0000 │ PUSH1     # Operación de pila (verde)
0002 │ PUSH2     # Operación de pila (verde)  
0005 │ ADD       # Aritmética (amarillo)
0006 │ MSTORE    # Operación de memoria (azul)
0007 │ RETURN    # Terminación (blanco)
==================================================
5 opcodes en total
```

**Con la bandera `--stats`:**
```
EVM BYTECODE DISASSEMBLY
==================================================
0000 │ PUSH1
0002 │ PUSH2
0005 │ STOP
==================================================
3 opcodes en total
ESTADÍSTICAS DEL BYTECODE
==================================================
Longitud en bytes: 6
Número de opcodes: 3
Profundidad máxima de pila: 2
```

**Con la bandera `--abi` (decodificación de selectores de función):**
```
EVM BYTECODE DISASSEMBLY
==================================================
0000 │ PUSH4  # 0xa9059cbb → transfer(address,uint256)
0005 │ PUSH20
001a │ PUSH9
0024 │ BLOCKHASH
==================================================
4 opcodes en total
```


## 🔧 Desarrollo

### Prerrequisitos

- Rust 1.85+ (edición 2024)
- Cargo

### Compilación

```bash
git clone https://github.com/andyrobert3/evm-lens
cd evm-lens
cargo build --release
```

### Pruebas

```bash
cargo test --workspace
```

### Ejecutar Ejemplos

```bash
# Ejecutar la CLI con diferentes métodos de entrada
cargo run -p evm-lens -- 60FF61ABCD00
cargo run -p evm-lens -- --file examples/bytecode.txt
echo "0x60FF61ABCD00" | cargo run -p evm-lens -- --stdin
cargo run -p evm-lens -- --address 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --rpc https://eth.llamarpc.com

# Probar decodificación de selectores de función ABI
cargo run -p evm-lens -- 63a9059cbb00 --abi

# Probar la biblioteca
cargo run --example basic -p evm-lens-core
```

## 📋 Opcodes Soportados

Se soportan todos los opcodes estándar de EVM:

| Categoría | Ejemplos |
|----------|----------|
| **Pila** | PUSH1-PUSH32, POP, DUP1-DUP16, SWAP1-SWAP16 |
| **Aritméticos** | ADD, SUB, MUL, DIV, MOD, ADDMOD, MULMOD |
| **Comparación** | LT, GT, SLT, SGT, EQ, ISZERO |
| **Bit a bit** | AND, OR, XOR, NOT, BYTE, SHL, SHR, SAR |
| **Memoria** | MLOAD, MSTORE, MSTORE8, MSIZE, MCOPY |
| **Almacenamiento** | SLOAD, SSTORE, TLOAD, TSTORE |
| **Control** | JUMP, JUMPI, JUMPDEST, PC, GAS |
| **Info de Bloque** | BLOCKHASH, COINBASE, TIMESTAMP, NUMBER |
| **Llamadas** | CALL, CALLCODE, DELEGATECALL, STATICCALL |
| **Creación** | CREATE, CREATE2 |
| **Terminación** | STOP, RETURN, REVERT, SELFDESTRUCT |
| **Criptografía** | KECCAK256, ECRECOVER |

## 🤝 Contribuciones

¡Las contribuciones son bienvenidas! No dudes en enviar un Pull Request.

1. Haz un Fork del repositorio
2. Crea tu rama de características (`git checkout -b feature/amazing-feature`)
3. Confirma tus cambios (`git commit -m 'Añadir alguna característica increíble'`)
4. Sube a la rama (`git push origin feature/amazing-feature`)
5. Abre un Pull Request

## 📝 Licencia

Este proyecto está licenciado bajo la Licencia MIT - consulta el archivo [LICENSE](LICENSE) para más detalles.

## 🙏 Agradecimientos

- [revm](https://github.com/bluealloy/revm) - Implementación de EVM de alto rendimiento
- La comunidad de Ethereum por las especificaciones de EVM
****
