use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use bitcoincore_rpc::{
    bitcoin::{Address, Amount, Network},
    Auth, Client as BitcoinClient, RpcApi,
};
use reqwest::blocking::Client;
use serde::de;
use serde_json::Value;

fn get_loaded_wallet(rpc: &BitcoinClient, name: &str) {
    let wallets = rpc.list_wallets().unwrap_or_else(|_| vec![]);

    if wallets.contains(&name.to_string()) {
        println!("{} is already loaded.", name);
    } else {
        match rpc.load_wallet(&name.to_string()) {
            Ok(wallet) => println!("Loaded wallet: {:?}", wallet),
            Err(e) => {
                // Create wallet, since it doesn't exist amd load it
                let _ = rpc.create_wallet(name, None, None, None, None);
                println!("Created and loaded wallet: {}", name);
            }
        }
    }
}

fn call_cln(method: &str, params: Value) -> Result<Value, Box<dyn std::error::Error>> {
    let rune = std::env::var("CLN_RUNE")?;
    let url = format!("http://localhost:3010/v1/{}", method);

    let client = Client::new();
    let response = client
        .post(&url)
        .json(&params)
        .header("Rune", rune)
        .send()?
        .json::<Value>()?;

    Ok(response)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get blockchain info
    let rpc = BitcoinClient::new(
        "http://localhost:18443",
        Auth::UserPass("alice".to_string(), "password".to_string()),
    )?;

    println!("Blockchain Info: {:?}", rpc.get_blockchain_info()?);

    // Get Lightning node info
    let ln_info = call_cln("getinfo", serde_json::json!({}))?;
    println!("Lightning Node Info: {}", ln_info);

    // Create a new address for funding using lightning-cli and store it in cln_address
    let cln_address = call_cln("newaddr", serde_json::json!({}))?
        .get("bech32")
        .and_then(|v| v.as_str())
        .ok_or("Failed to get new address")?
        .to_string();
    println!("New Lightning Address: {}", cln_address);

    // Check if wallet exists, if not Create a bitcoin wallet named 'mining_wallet' using bitcoin-cli for mining
    get_loaded_wallet(&rpc, "mining_wallet");

    // Generate a new address and mine blocks to it. How many blocks need to mined? Why?
    let btc_address = rpc.get_new_address(None, None)?.assume_checked();
    println!("New Bitcoin Address for Mining: {:?}", btc_address);

    // Mine 101 blocks to the new address to ensure the funds are mature and can be spent
    rpc.generate_to_address(101, &btc_address)?;

    // Fund the Lightning node by sending 0.1 BTC from the mining wallet to cln_address
    let txid = rpc.send_to_address(
        &Address::from_str(&cln_address)
            .unwrap()
            .require_network(Network::Regtest)
            .unwrap(),
        Amount::from_btc(0.1).unwrap(),
        None,
        None,
        None,
        None,
        None,
        None,
    )?;
    println!("Funding Transaction ID: {:?}", txid);

    // Confirm the funding transaction by mining 6 blocks
    rpc.generate_to_address(6, &btc_address)?;

    // Verify Lightning wallet balance using lightning-cli listfunds
    let balance = call_cln("listfunds", serde_json::json!({}))?;
    println!("Lightning Wallet Balance: {}", balance);

    // Create an invoice with parameters and store the invoice string:
    // - Amount: 50,000 satoshis (50000000 millisatoshis)
    // - Label: Generate unique label using timestamp (e.g., "invoice_$(date +%s)")
    // - Description: "Coffee Payment"
    // - Expiry: 3600 seconds
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let invoice = call_cln(
        "invoice",
        serde_json::json!({
            "amount_msat": 50000000,
            "label": format!("invoice_{}", timestamp),
            "description": "Coffee Payment",
            "expiry": 3600
        }),
    )?;
    println!(
        "Generated Invoice: {} with BOLT11 key, {}",
        invoice, invoice["bolt11"]
    );

    // Decode the invoice string using lightning-cli decode and verify the parameters
    // Output the invoice details in the specified format to out.txt
    // - Payment hash
    // - BOLT11 invoice string
    // - Amount
    // - Description
    // - Expiry time
    let bolt11 = invoice["bolt11"].as_str().unwrap_or("");
    let decoded_invoice = call_cln("decode", serde_json::json!({ "string": bolt11 }))?;
    println!("Decoded Invoice: {}", decoded_invoice);

    let pay_hash = decoded_invoice["payment_hash"].as_str().unwrap_or("");
    let amount = decoded_invoice["amount_msat"].as_u64().unwrap_or(0);
    let description = decoded_invoice["description"].as_str().unwrap_or("");
    let expiry = decoded_invoice["expiry"].as_u64().unwrap_or(0);

    let mut file = File::create("../out.txt")?;
    writeln!(file, "{}", pay_hash)?;
    writeln!(file, "{}", bolt11)?;
    writeln!(file, "{}", amount)?;
    writeln!(file, "{}", description)?;
    writeln!(file, "{}", expiry)?;

    println!("Invoice details written to out.txt");

    Ok(())
}
