# EVM Vanity Address Generator

A Rust application that generates Ethereum vanity addresses with custom prefixes or suffixes.

## Features

- Generate EVM addresses with custom prefix or suffix patterns
- Display wallet address, private key, and mnemonic phrase
- Progress logging every 10,000 attempts
- Graceful shutdown with Ctrl+C
- High-performance address generation

## Installation

Make sure you have Rust installed. If not, install it from [rustup.rs](https://rustup.rs/).

```bash
cargo build --release
```

## Usage

### Command Line Options

- `-p, --pattern <PATTERN>`: Target pattern to match (prefix or suffix)
- `-s, --suffix`: Whether to match as suffix (default is prefix)
- `-c, --case-sensitive`: Whether to match case-sensitively (default is case-insensitive)
- `-t, --threads <NUM>`: Number of threads to use (default is number of CPU cores)
- `-h, --help`: Print help information

### Examples

```bash
# Generate address with prefix (case-insensitive by default)
cargo run -- --pattern "dead"

# Generate address with suffix (case-insensitive by default)
cargo run -- --pattern "beef" --suffix

# Generate address with case-sensitive prefix
cargo run -- --pattern "ABC" --case-sensitive

# Generate address with case-sensitive suffix
cargo run -- --pattern "DEF" --suffix --case-sensitive

# Using release build (faster)
./target/release/evm-vanity -p dead
./target/release/evm-vanity -p beef -s
./target/release/evm-vanity -p ABC -c
./target/release/evm-vanity -p DEF -s -c

# Using specific number of threads
./target/release/evm-vanity -p dead -t 4
./target/release/evm-vanity -p beef -s -t 16
```

### Help:
```bash
cargo run -- --help
```

## Example Output

```
🔍 Searching for EVM vanity address...
Pattern: dead (prefix)
Case sensitive: false
Threads: 8
Press Ctrl+C to stop

⏳ Attempts: 10000 | Rate: 15234 addr/sec | Elapsed: 656.78ms
⏳ Attempts: 20000 | Rate: 15456 addr/sec | Elapsed: 1.29s
🎉 Found vanity address after 23456 attempts in 1.52s!
📍 Address: 0xdead1234567890abcdef1234567890abcdef1234
🔐 Private Key: 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
📝 Mnemonic: abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
```

## Security Warning

⚠️ **Never share your private key or mnemonic phrase with anyone!** Store them securely and use them only for legitimate purposes.

## Performance

The application now uses multi-threading to maximize performance:
- **Multi-threaded**: Utilizes all CPU cores by default for parallel address generation
- **Optimized algorithms**: Fast address generation without unnecessary computations
- **Expected rates**: 100,000+ addresses per second on modern hardware (8+ cores)
- **Scalability**: Performance scales with CPU cores using the `-t` option

Longer patterns will take exponentially more time to find. The multi-threaded approach provides significant speedup over single-threaded generation.