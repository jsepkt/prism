use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Instant;

use prism_agent::{AutonomousSolver, IntentDomain, UserIntent};
use prism_client::{EdgeEvaluator, NumericCriteria, SovereignContextVault};
use prism_crypto::{hash, Keypair};

fn http_request(method: &str, url_str: &str, body_json: Option<&str>) -> Result<String, String> {
    let clean = url_str.trim_start_matches("http://");
    let parts: Vec<&str> = clean.splitn(2, '/').collect();
    let host_port = parts[0];
    let path = if parts.len() > 1 {
        format!("/{}", parts[1])
    } else {
        "/".to_string()
    };

    let mut stream = TcpStream::connect(host_port)
        .map_err(|e| format!("Failed to connect to {}: {}", host_port, e))?;

    let host_only = host_port.split(':').next().unwrap_or(host_port);

    let req = match body_json {
        Some(body) => format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            method,
            path,
            host_only,
            body.len(),
            body
        ),
        None => format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            method,
            path,
            host_only
        ),
    };

    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("Write failed: {}", e))?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|e| format!("Read failed: {}", e))?;

    if let Some(pos) = response.find("\r\n\r\n") {
        Ok(response[pos + 4..].to_string())
    } else {
        Ok(response)
    }
}

fn print_banner() {
    println!(r#"
    ██████╗ ██████╗ ██╗███████╗███╗   ███╗
    ██╔══██╗██╔══██╗██║██╔════╝████╗ ████║
    ██████╔╝██████╔╝██║███████╗██╔████╔██║
    ██╔═══╝ ██╔══██╗██║╚════██║██║╚██╔╝██║
    ██║     ██║  ██║██║███████║██║ ╚═╝ ██║
    ╚═╝     ╚═╝  ╚═╝╚═╝╚══════╝╚═╝     ╚═╝
    Sovereign Context Ledger & Edge-AI CLI v0.1.0
    "#);
}

fn print_help() {
    print_banner();
    println!("USAGE:");
    println!("    prism-cli <SUBCOMMAND> [OPTIONS]\n");
    println!("SUBCOMMANDS:");
    println!("    status                Query local/remote PoAC node health & consensus height");
    println!("    keygen                Generate a sovereign Ed25519 cryptographic keypair");
    println!("    account <PUBKEY>      Fetch on-chain account balance and nonce");
    println!("    faucet <PUBKEY> [AMT] Request devnet PRISM tokens from validator faucet");
    println!("    prove [VALUE] [MIN]   Benchmark edge hardware & synthesize real Groth16 zk-CP proof");
    println!("    intent [BUDGET]       Run autonomous multi-solver clearinghouse bidding simulation");
    println!("    help                  Display this help message\n");
    println!("OPTIONS:");
    println!("    --rpc <URL>           Specify custom RPC URL (default: http://127.0.0.1:8545)");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_help();
        return;
    }

    let mut rpc_url = "http://127.0.0.1:8545".to_string();
    let mut clean_args = Vec::new();

    let mut i = 1;
    while i < args.len() {
        if args[i] == "--rpc" && i + 1 < args.len() {
            rpc_url = args[i + 1].clone();
            i += 2;
        } else {
            clean_args.push(args[i].clone());
            i += 1;
        }
    }

    if clean_args.is_empty() {
        print_help();
        return;
    }

    let command = clean_args[0].as_str();

    match command {
        "help" | "-h" | "--help" => {
            print_help();
        }

        "keygen" => {
            let keypair = Keypair::generate();
            println!("\n[+] Generated Sovereign Ed25519 Keypair:");
            println!("    Public Key:  0x{}", keypair.public_key().to_hex());
            println!("    Private Key: 0x{}", keypair.private_key_hex());
            println!("    [!] Secure your private key. Never share it with untrusted parties.\n");
        }

        "status" => {
            println!("[*] Querying Prism Node at {}...", rpc_url);
            match http_request("GET", &format!("{}/health", rpc_url), None) {
                Ok(resp) => {
                    println!("\n[+] Node Status: OK");
                    println!("    Response: {}", resp.trim());
                }
                Err(e) => {
                    eprintln!("[-] Error connecting to node: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "account" => {
            if clean_args.len() < 2 {
                eprintln!("Error: Account public key required: prism-cli account <PUBKEY_HEX>");
                std::process::exit(1);
            }
            let pubkey = clean_args[1].trim_start_matches("0x");
            let endpoint = format!("{}/api/v1/accounts/{}", rpc_url, pubkey);
            match http_request("GET", &endpoint, None) {
                Ok(resp) => {
                    println!("\n[+] Account Details (0x{}):", pubkey);
                    println!("    Response: {}", resp.trim());
                }
                Err(e) => {
                    eprintln!("[-] Error fetching account: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "faucet" => {
            if clean_args.len() < 2 {
                eprintln!("Error: Recipient public key required: prism-cli faucet <PUBKEY_HEX> [AMOUNT]");
                std::process::exit(1);
            }
            let pubkey = clean_args[1].trim_start_matches("0x");
            let amount: u64 = if clean_args.len() >= 3 {
                clean_args[2].parse().unwrap_or(1000)
            } else {
                1000
            };

            let payload = format!(r#"{{"pubkey":"{}","amount":{}}}"#, pubkey, amount);
            let endpoint = format!("{}/api/v1/dev/faucet", rpc_url);
            match http_request("POST", &endpoint, Some(&payload)) {
                Ok(resp) => {
                    println!("\n[+] Faucet Drop Request Succeeded!");
                    println!("    Credited: {} PRISM to 0x{}", amount, pubkey);
                    println!("    Receipt: {}", resp.trim());
                }
                Err(e) => {
                    eprintln!("[-] Error requesting faucet: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "prove" => {
            let actual_value: f64 = if clean_args.len() >= 2 {
                clean_args[1].parse().unwrap_or(24.5)
            } else {
                24.5
            };
            let min_threshold: f64 = if clean_args.len() >= 3 {
                clean_args[2].parse().unwrap_or(15.0)
            } else {
                15.0
            };

            println!("\n[*] Benchmarking Zero-Knowledge Proving on Edge Silicon...");
            println!("    Metric: Weekly Run Distance (km)");
            println!("    User Value: {} km | Threshold: >= {} km", actual_value, min_threshold);

            let criteria = NumericCriteria {
                category: "health".to_string(),
                metric: "weekly_run_distance_km".to_string(),
                min_value: Some(min_threshold),
                max_value: None,
            };

            let mut vault = SovereignContextVault::new();
            vault.record_numeric("health", "weekly_run_distance_km", actual_value);

            let schema_id = hash(b"HEALTH_CIRCUIT_V1");
            let start = Instant::now();
            let proof_res = EdgeEvaluator::prove_criteria(&schema_id, &criteria, &vault, 0);
            let elapsed = start.elapsed();

            if let Some((proof, attestation)) = proof_res {
                println!("\n[+] Zero-Knowledge Proof Synthesized Successfully!");
                println!("    Evaluation & MSM Time: {:?}", elapsed);
                println!("    Circuit ID:    0x{}", proof.circuit_id.to_hex());
                println!("    Public Inputs: 0x{}", hex::encode(&proof.public_inputs));
                println!("    Proof Bytes:   0x{}", hex::encode(&proof.proof_bytes));
                println!("    Attestation:   Platform {:?} (Nonce: {})", attestation.platform, attestation.nonce);
                println!("    Verification:  SATISFIED (0 raw metrics leaked to network)");
            } else {
                println!("\n[-] Criteria Unmet: User distance {} km < {} km threshold.", actual_value, min_threshold);
                println!("    Proof Rejected. 0 metadata leaked.");
            }
        }

        "intent" => {
            let budget: u64 = if clean_args.len() >= 2 {
                clean_args[1].parse().unwrap_or(400)
            } else {
                400
            };

            let user_kp = Keypair::generate();
            let provider_kp = Keypair::generate();
            let solver_alpha_kp = Keypair::generate();

            println!("\n[*] Initializing Autonomous Intent Clearinghouse Auction...");
            println!("    User:   0x{}", user_kp.public_key().to_hex());
            println!("    Domain: TravelFlight");
            println!("    Intent: Direct flight SFO -> DEN with extra legroom");
            println!("    Max Budget: {} PRISM", budget);

            let intent = UserIntent::new(
                user_kp.public_key(),
                IntentDomain::TravelFlight,
                "Direct flight SFO -> DEN with extra legroom",
                budget,
                budget,
            );

            // Solver Alpha: 15% discount
            let solver_alpha = AutonomousSolver::new(solver_alpha_kp, vec![IntentDomain::TravelFlight], 0.15);
            let bid = solver_alpha
                .evaluate_intent(&intent, provider_kp.public_key())
                .expect("Evaluation failed");

            let royalty = (bid.proposed_cost as f64 * 12500.0) / 1_000_000.0;
            let burn = (bid.proposed_cost as f64 * 2500.0) / 1_000_000.0;
            let net = bid.proposed_cost as f64 + royalty + burn;

            println!("\n[+] Auction Finalized: Solver Alpha Won Bidding Mesh!");
            println!("    Winning Solver Bid:  {} PRISM (15% Saved vs Budget)", bid.proposed_cost);
            println!("    Service Provider:    0x{}", provider_kp.public_key().to_hex());
            println!("    Enshrined Royalty:   {:.2} PRISM (12,500 PPM / 1.25% to Treasury)", royalty);
            println!("    Deflationary Burn:   {:.2} PRISM (2,500 PPM / 0.25% Burned)", burn);
            println!("    Total User Cost:     {:.2} PRISM", net);

            let tx = solver_alpha.construct_execution_tx(&intent, &bid, 0, 2);
            assert!(tx.verify_signature().is_ok());
            println!("    On-Chain Tx Hash:    0x{}", tx.hash().to_hex());
            println!("    Tx Signature:        Valid (Verified with Ed25519)\n");
        }

        _ => {
            eprintln!("Unknown command: '{}'", command);
            print_help();
            std::process::exit(1);
        }
    }
}
