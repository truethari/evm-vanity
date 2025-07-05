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

**New Pattern Matching:**
- `--prefix <PATTERN>`: Match prefix pattern
- `--suffix <PATTERN>`: Match suffix pattern
- `--prefix <PATTERN> --suffix <PATTERN>`: Match both prefix AND suffix (dual pattern)

**Legacy Options:**
- `-p, --pattern <PATTERN>`: Target pattern to match (prefix or suffix)
- `-s, --suffix`: Whether to match as suffix (default is prefix)

**Other Options:**
- `-c, --case-sensitive`: Whether to match case-sensitively (default is case-insensitive)
- `-t, --threads <NUM>`: Number of threads to use (default is number of CPU cores)
- `-h, --help`: Print help information

### Examples

```bash
# New dual pattern syntax
# Generate address with prefix only
cargo run -- --prefix "dead"

# Generate address with suffix only
cargo run -- --suffix "beef"

# Generate address with BOTH prefix AND suffix (dual pattern)
cargo run -- --prefix "dead" --suffix "beef"

# Case-sensitive dual pattern
cargo run -- --prefix "ABC" --suffix "DEF" --case-sensitive

# Using release build (faster)
./target/release/evm-vanity --prefix dead
./target/release/evm-vanity --suffix beef
./target/release/evm-vanity --prefix dead --suffix beef
./target/release/evm-vanity --prefix ABC --suffix DEF --case-sensitive

# Using specific number of threads with dual pattern
./target/release/evm-vanity --prefix dead --suffix beef -t 16

# Legacy syntax (still supported)
cargo run -- --pattern "dead"
cargo run -- --pattern "beef" --suffix
./target/release/evm-vanity -p dead
./target/release/evm-vanity -p beef -s
```

### Help:
```bash
cargo run -- --help
```

## Example Output

```
🔍 Searching for EVM vanity address...
Pattern: prefix 'dead' AND suffix 'beef'
Case sensitive: false
Threads: 8
Press Ctrl+C to stop

⏳ Attempts: 10000 | Rate: 15234 addr/sec | Elapsed: 656.78ms
⏳ Attempts: 20000 | Rate: 15456 addr/sec | Elapsed: 1.29s
🎉 Found vanity address after 23456 attempts in 1.52s!
📍 Address: 0xdead1234567890abcdef1234567890abcdefbeef
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