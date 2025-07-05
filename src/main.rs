use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use clap::Parser;
use secp256k1::{Secp256k1, SecretKey, PublicKey};
use sha3::{Digest, Keccak256};
use hex;
use rand::rngs::OsRng;
use bip39::Mnemonic;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target pattern to match (prefix or suffix)
    #[arg(short, long)]
    pattern: String,
    
    /// Whether to match as prefix (default) or suffix
    #[arg(short, long, default_value = "false")]
    suffix: bool,
    
    /// Whether to match case-sensitively (default is case-insensitive)
    #[arg(short, long, default_value = "false")]
    case_sensitive: bool,
}

struct WalletInfo {
    address: String,
    private_key: String,
    mnemonic: Option<String>,
}

fn generate_ethereum_address(secp: &Secp256k1<secp256k1::All>) -> Result<WalletInfo, Box<dyn std::error::Error>> {
    // Generate random private key
    let private_key = SecretKey::new(&mut OsRng);
    
    // Get public key
    let public_key = PublicKey::from_secret_key(secp, &private_key);
    
    // Get uncompressed public key bytes (remove the 0x04 prefix)
    let public_key_bytes = public_key.serialize_uncompressed();
    let public_key_hash = &public_key_bytes[1..]; // Remove first byte (0x04)
    
    // Hash with Keccak256
    let mut hasher = Keccak256::new();
    hasher.update(public_key_hash);
    let hash = hasher.finalize();
    
    // Take last 20 bytes for address
    let address_bytes = &hash[12..];
    let address = format!("0x{}", hex::encode(address_bytes));
    
    // Convert private key to hex
    let private_key_hex = format!("0x{}", hex::encode(private_key.secret_bytes()));
    
    // Generate mnemonic from private key entropy
    let mnemonic = match Mnemonic::from_entropy(&private_key.secret_bytes()) {
        Ok(m) => Some(m.to_string()),
        Err(_) => None,
    };
    
    Ok(WalletInfo {
        address,
        private_key: private_key_hex,
        mnemonic,
    })
}

fn matches_pattern(address: &str, pattern: &str, is_suffix: bool, case_sensitive: bool) -> bool {
    // Remove 0x prefix for matching
    let address_without_prefix = address.strip_prefix("0x").unwrap_or(address);
    
    if case_sensitive {
        if is_suffix {
            address_without_prefix.ends_with(pattern)
        } else {
            address_without_prefix.starts_with(pattern)
        }
    } else {
        let addr_lower = address_without_prefix.to_lowercase();
        let pattern_lower = pattern.to_lowercase();
        
        if is_suffix {
            addr_lower.ends_with(&pattern_lower)
        } else {
            addr_lower.starts_with(&pattern_lower)
        }
    }
}

fn validate_pattern(pattern: &str) -> Result<(), String> {
    let invalid_chars: Vec<char> = pattern
        .chars()
        .filter(|c| !c.is_ascii_hexdigit())
        .collect();
    
    if !invalid_chars.is_empty() {
        let mut error_msg = String::from("❌ Invalid characters found in pattern:\n");
        
        for &invalid_char in &invalid_chars {
            error_msg.push_str(&format!("  • '{}' is not a valid hexadecimal character\n", invalid_char));
        }
        
        error_msg.push_str("\n💡 EVM addresses only use hexadecimal characters: 0-9, a-f, A-F\n");
        error_msg.push_str("   Valid examples: \"dead\", \"beef\", \"abc123\", \"DEF456\"");
        
        return Err(error_msg);
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Validate pattern before starting
    if let Err(error_msg) = validate_pattern(&args.pattern) {
        eprintln!("{}", error_msg);
        std::process::exit(1);
    }
    
    // Setup signal handling for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        println!("\nReceived Ctrl+C, shutting down...");
        r.store(false, Ordering::SeqCst);
    });
    
    let secp = Secp256k1::new();
    let mut attempts = 0u64;
    let start_time = Instant::now();
    
    println!("🔍 Searching for EVM vanity address...");
    println!("Pattern: {} ({})", args.pattern, if args.suffix { "suffix" } else { "prefix" });
    println!("Case sensitive: {}", args.case_sensitive);
    println!("Press Ctrl+C to stop\n");
    
    while running.load(Ordering::SeqCst) {
        attempts += 1;
        
        // Generate new wallet
        let wallet = generate_ethereum_address(&secp)?;
        
        // Check if address matches pattern
        if matches_pattern(&wallet.address, &args.pattern, args.suffix, args.case_sensitive) {
            let elapsed = start_time.elapsed();
            
            println!("🎉 Found vanity address after {} attempts in {:.2?}!", attempts, elapsed);
            println!("📍 Address: {}", wallet.address);
            println!("🔐 Private Key: {}", wallet.private_key);
            
            if let Some(mnemonic) = wallet.mnemonic {
                println!("📝 Mnemonic: {}", mnemonic);
            }
            
            break;
        }
        
        // Log progress every 500,000 attempts
        if attempts % 500000 == 0 {
            let elapsed = start_time.elapsed();
            let rate = attempts as f64 / elapsed.as_secs_f64();
            println!("⏳ Attempts: {} | Rate: {:.0} addr/sec | Elapsed: {:.2?}", 
                     attempts, rate, elapsed);
        }
    }
    
    if !running.load(Ordering::SeqCst) {
        println!("Search stopped by user after {} attempts", attempts);
    }
    
    Ok(())
}