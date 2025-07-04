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

### Generate address with prefix:
```bash
cargo run -- --pattern "dead"
```

### Generate address with suffix:
```bash
cargo run -- --pattern "beef" --suffix
```

### Help:
```bash
cargo run -- --help
```

## Example Output

```
🔍 Searching for EVM vanity address...
Pattern: dead (prefix)
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

The application generates approximately 10,000-20,000 addresses per second depending on your hardware. Longer patterns will take exponentially more time to find.