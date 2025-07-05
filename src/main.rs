use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use std::thread;
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
    pattern: Option<String>,
    
    /// Prefix pattern to match
    #[arg(long)]
    prefix: Option<String>,
    
    /// Suffix pattern to match
    #[arg(long)]
    suffix: Option<String>,
    
    /// Whether to match as suffix (default is prefix) - deprecated, use --prefix/--suffix instead
    #[arg(short, long, default_value = "false")]
    suffix_mode: bool,
    
    /// Whether to match case-sensitively (default is case-insensitive)
    #[arg(short, long, default_value = "false")]
    case_sensitive: bool,
    
    /// Number of threads to use (default is number of CPU cores)
    #[arg(short, long)]
    threads: Option<usize>,
}

struct WalletInfo {
    address: String,
    private_key: String,
    mnemonic: Option<String>,
}

// Fast address generation without mnemonic for searching
fn generate_address_fast(secp: &Secp256k1<secp256k1::All>) -> (String, SecretKey) {
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
    
    (address, private_key)
}

// Generate full wallet info only when match is found
fn generate_wallet_info(private_key: SecretKey) -> WalletInfo {
    let private_key_hex = format!("0x{}", hex::encode(private_key.secret_bytes()));
    
    // Generate mnemonic from private key entropy
    let mnemonic = match Mnemonic::from_entropy(&private_key.secret_bytes()) {
        Ok(m) => Some(m.to_string()),
        Err(_) => None,
    };
    
    // Regenerate address for the wallet info
    let secp = Secp256k1::new();
    let public_key = PublicKey::from_secret_key(&secp, &private_key);
    let public_key_bytes = public_key.serialize_uncompressed();
    let public_key_hash = &public_key_bytes[1..];
    let mut hasher = Keccak256::new();
    hasher.update(public_key_hash);
    let hash = hasher.finalize();
    let address_bytes = &hash[12..];
    let address = format!("0x{}", hex::encode(address_bytes));
    
    WalletInfo {
        address,
        private_key: private_key_hex,
        mnemonic,
    }
}

fn matches_pattern(address: &str, prefix_pattern: Option<&str>, suffix_pattern: Option<&str>, case_sensitive: bool) -> bool {
    // Remove 0x prefix for matching
    let address_without_prefix = address.strip_prefix("0x").unwrap_or(address);
    
    let addr_to_check = if case_sensitive {
        address_without_prefix.to_string()
    } else {
        address_without_prefix.to_lowercase()
    };
    
    // Check prefix pattern
    if let Some(prefix) = prefix_pattern {
        let prefix_to_check = if case_sensitive {
            prefix.to_string()
        } else {
            prefix.to_lowercase()
        };
        
        if !addr_to_check.starts_with(&prefix_to_check) {
            return false;
        }
    }
    
    // Check suffix pattern
    if let Some(suffix) = suffix_pattern {
        let suffix_to_check = if case_sensitive {
            suffix.to_string()
        } else {
            suffix.to_lowercase()
        };
        
        if !addr_to_check.ends_with(&suffix_to_check) {
            return false;
        }
    }
    
    true
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
    
    // Determine prefix and suffix patterns
    let (prefix_pattern, suffix_pattern) = match (&args.pattern, &args.prefix, &args.suffix) {
        // New style: --prefix and/or --suffix
        (None, Some(prefix), Some(suffix)) => (Some(prefix.as_str()), Some(suffix.as_str())),
        (None, Some(prefix), None) => (Some(prefix.as_str()), None),
        (None, None, Some(suffix)) => (None, Some(suffix.as_str())),
        
        // Legacy style: -p pattern with -s flag
        (Some(pattern), None, None) => {
            if args.suffix_mode {
                (None, Some(pattern.as_str()))
            } else {
                (Some(pattern.as_str()), None)
            }
        }
        
        // Invalid combinations
        (Some(_), Some(_), _) | (Some(_), _, Some(_)) => {
            eprintln!("❌ Cannot use -p/--pattern with --prefix/--suffix. Use either:");
            eprintln!("  • Legacy: -p <pattern> [-s]");
            eprintln!("  • New: --prefix <pattern> and/or --suffix <pattern>");
            std::process::exit(1);
        }
        
        (None, None, None) => {
            eprintln!("❌ Must specify at least one pattern:");
            eprintln!("  • --prefix <pattern>: Match prefix");
            eprintln!("  • --suffix <pattern>: Match suffix");
            eprintln!("  • --prefix <pattern> --suffix <pattern>: Match both");
            eprintln!("  • -p <pattern>: Legacy format");
            std::process::exit(1);
        }
    };
    
    // Validate patterns
    if let Some(prefix) = prefix_pattern {
        if let Err(error_msg) = validate_pattern(prefix) {
            eprintln!("❌ Invalid prefix pattern:");
            eprintln!("{}", error_msg);
            std::process::exit(1);
        }
    }
    
    if let Some(suffix) = suffix_pattern {
        if let Err(error_msg) = validate_pattern(suffix) {
            eprintln!("❌ Invalid suffix pattern:");
            eprintln!("{}", error_msg);
            std::process::exit(1);
        }
    }
    
    // Determine number of threads
    let num_threads = args.threads.unwrap_or_else(|| thread::available_parallelism().unwrap().get());
    
    // Setup signal handling for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        println!("\nReceived Ctrl+C, shutting down...");
        r.store(false, Ordering::SeqCst);
    });
    
    let total_attempts = Arc::new(AtomicU64::new(0));
    let start_time = Instant::now();
    
    println!("🔍 Searching for EVM vanity address...");
    match (prefix_pattern, suffix_pattern) {
        (Some(prefix), Some(suffix)) => println!("Pattern: prefix '{}' AND suffix '{}'", prefix, suffix),
        (Some(prefix), None) => println!("Pattern: prefix '{}'", prefix),
        (None, Some(suffix)) => println!("Pattern: suffix '{}'", suffix),
        (None, None) => unreachable!(),
    }
    println!("Case sensitive: {}", args.case_sensitive);
    println!("Threads: {}", num_threads);
    println!("Press Ctrl+C to stop\n");
    
    // Shared data between threads
    let prefix_pattern_arc = Arc::new(prefix_pattern.map(|s| s.to_string()));
    let suffix_pattern_arc = Arc::new(suffix_pattern.map(|s| s.to_string()));
    let case_sensitive = args.case_sensitive;
    let found = Arc::new(AtomicBool::new(false));
    let result = Arc::new(std::sync::Mutex::new(None::<WalletInfo>));
    let winning_attempts = Arc::new(AtomicU64::new(0));
    
    // Spawn worker threads
    let mut handles = Vec::new();
    for _thread_id in 0..num_threads {
        let running = running.clone();
        let prefix_pattern_arc = prefix_pattern_arc.clone();
        let suffix_pattern_arc = suffix_pattern_arc.clone();
        let found = found.clone();
        let result = result.clone();
        let total_attempts = total_attempts.clone();
        let winning_attempts = winning_attempts.clone();
        
        let handle = thread::spawn(move || {
            let secp = Secp256k1::new();
            let mut local_attempts = 0u64;
            
            while running.load(Ordering::SeqCst) && !found.load(Ordering::SeqCst) {
                local_attempts += 1;
                
                // Generate new address
                let (address, private_key) = generate_address_fast(&secp);
                
                // Check if address matches pattern
                let prefix_ref = prefix_pattern_arc.as_ref().as_ref().map(|s| s.as_str());
                let suffix_ref = suffix_pattern_arc.as_ref().as_ref().map(|s| s.as_str());
                if matches_pattern(&address, prefix_ref, suffix_ref, case_sensitive) {
                    // Found match - create full wallet info
                    let wallet = generate_wallet_info(private_key);
                    
                    // Set found flag and store result
                    found.store(true, Ordering::SeqCst);
                    *result.lock().unwrap() = Some(wallet);
                    winning_attempts.store(local_attempts, Ordering::SeqCst);
                    break;
                }
                
                // Update total attempts counter periodically
                if local_attempts % 1000 == 0 {
                    total_attempts.fetch_add(1000, Ordering::SeqCst);
                }
            }
            
            // Add remaining attempts
            total_attempts.fetch_add(local_attempts % 1000, Ordering::SeqCst);
        });
        
        handles.push(handle);
    }
    
    // Progress reporting thread
    let progress_running = running.clone();
    let progress_attempts = total_attempts.clone();
    let progress_found = found.clone();
    let progress_handle = thread::spawn(move || {
        let mut last_attempts = 0u64;
        let mut last_time = Instant::now();
        
        while progress_running.load(Ordering::SeqCst) && !progress_found.load(Ordering::SeqCst) {
            thread::sleep(std::time::Duration::from_secs(5));
            
            let current_attempts = progress_attempts.load(Ordering::SeqCst);
            let current_time = Instant::now();
            
            if current_attempts > last_attempts {
                let elapsed = current_time.duration_since(last_time);
                let rate = (current_attempts - last_attempts) as f64 / elapsed.as_secs_f64();
                let total_elapsed = current_time.duration_since(start_time);
                
                if current_attempts % 500000 < last_attempts % 500000 || 
                   current_attempts - last_attempts >= 500000 {
                    println!("⏳ Attempts: {} | Rate: {:.0} addr/sec | Elapsed: {:.2?}", 
                             current_attempts, rate, total_elapsed);
                }
                
                last_attempts = current_attempts;
                last_time = current_time;
            }
        }
    });
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    progress_handle.join().unwrap();
    
    // Check results
    if found.load(Ordering::SeqCst) {
        let final_attempts = total_attempts.load(Ordering::SeqCst);
        let elapsed = start_time.elapsed();
        
        if let Some(wallet) = result.lock().unwrap().as_ref() {
            println!("🎉 Found vanity address after {} attempts in {:.2?}!", final_attempts, elapsed);
            println!("📍 Address: {}", wallet.address);
            println!("🔐 Private Key: {}", wallet.private_key);
            
            if let Some(mnemonic) = &wallet.mnemonic {
                println!("📝 Mnemonic: {}", mnemonic);
            }
        }
    } else {
        let final_attempts = total_attempts.load(Ordering::SeqCst);
        println!("Search stopped by user after {} attempts", final_attempts);
    }
    
    Ok(())
}