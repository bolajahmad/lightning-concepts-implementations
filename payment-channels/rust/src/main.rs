use bitcoincore_rpc::{Auth, Client as BitcoinClient, RpcApi};
use reqwest::blocking::Client;
use serde_json::Value;
use std::fs;
use std::thread::sleep;
use std::time::Duration;

/// Call Alice's Lightning node via CLN REST API on port 3010
fn call_alice_ln(method: &str, params: Value) -> Result<Value, Box<dyn std::error::Error>> {
    let rune = std::env::var("ALICE_RUNE")?;
    let url = format!("http://localhost:3010/v1/{}", method);

    let client = Client::new();
    let response = client
        .post(&url)
        .json(&params)
        .header("Rune", rune)
        .send()?
        .json::<Value>()?;

    if response.get("code").is_some() {
        let msg = response
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown CLN error");
        return Err(msg.to_string().into());
    }

    Ok(response)
}

/// Call Bob's Lightning node via CLN REST API on port 3011
fn call_bob_ln(method: &str, params: Value) -> Result<Value, Box<dyn std::error::Error>> {
    let rune = std::env::var("BOB_RUNE")?;
    let url = format!("http://localhost:3011/v1/{}", method);

    let client = Client::new();
    let response = client
        .post(&url)
        .json(&params)
        .header("Rune", rune)
        .send()?
        .json::<Value>()?;

    if response.get("code").is_some() {
        let msg = response
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown CLN error");
        return Err(msg.to_string().into());
    }

    Ok(response)
}

fn msat_to_u64(value: &Value) -> Result<u64, Box<dyn std::error::Error>> {
    if let Some(n) = value.as_u64() {
        return Ok(n);
    }

    if let Some(s) = value.as_str() {
        return Ok(s.trim_end_matches("msat").parse::<u64>()?);
    }

    Err("Unsupported millisatoshi value format".into())
}

fn create_or_load_wallet(
    bitcoin_rpc: &BitcoinClient,
    wallet_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let create_result =
        bitcoin_rpc.call::<Value>("createwallet", &[serde_json::json!(wallet_name)]);

    if let Err(create_err) = create_result {
        // If wallet already exists we try loading it.
        let load_result =
            bitcoin_rpc.call::<Value>("loadwallet", &[serde_json::json!(wallet_name)]);
        if let Err(load_err) = load_result {
            let err_text = format!("{} | {}", create_err, load_err);
            if !err_text.contains("already loaded") {
                return Err(err_text.into());
            }
        }
    }

    Ok(())
}

fn get_channel_for_peer(channels_response: &Value, peer_id: &str) -> Option<Value> {
    channels_response
        .get("channels")
        .and_then(|v| v.as_array())
        .and_then(|channels| {
            channels
                .iter()
                .find(|ch| ch.get("peer_id").and_then(|v| v.as_str()) == Some(peer_id))
                .cloned()
        })
}

fn get_available_confirmed_msat(listfunds: &Value) -> Result<u64, Box<dyn std::error::Error>> {
    let mut total: u64 = 0;
    let outputs = listfunds
        .get("outputs")
        .and_then(|v| v.as_array())
        .ok_or("Missing outputs in listfunds")?;

    for output in outputs {
        let status = output.get("status").and_then(|v| v.as_str()).unwrap_or("");
        let reserved = output
            .get("reserved")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if status == "confirmed" && !reserved {
            let amount = msat_to_u64(
                output
                    .get("amount_msat")
                    .ok_or("Missing output amount_msat")?,
            )?;
            total = total.saturating_add(amount);
        }
    }

    Ok(total)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bitcoin RPC client
    let bitcoin_rpc = BitcoinClient::new(
        "http://localhost:18443",
        Auth::UserPass("alice".to_string(), "password".to_string()),
    )?;

    // Get Alice's node info
    let alice_info = call_alice_ln("getinfo", serde_json::json!({}))?;

    // Get Bob's node info
    let bob_info = call_bob_ln("getinfo", serde_json::json!({}))?;

    // Get Alice's node ID
    let alice_node_id = alice_info
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("Failed to get Alice's node ID")?;

    // Get Bob's node ID
    let bob_node_id = bob_info
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("Failed to get Bob's node ID")?;

    // Connect Alice to Bob as a peer
    let connect_result = call_alice_ln(
        "connect",
        serde_json::json!({
            "id": bob_node_id,
            "host": "bob",
            "port": 9735
        }),
    );
    if let Err(err) = connect_result {
        let err_msg = err.to_string();
        if !err_msg.contains("already connected") {
            return Err(err);
        }
    }

    // Verify peer connection from both Alice's and Bob's perspectives
    let alice_peers = call_alice_ln("listpeers", serde_json::json!({}))?;
    let bob_peers = call_bob_ln("listpeers", serde_json::json!({}))?;

    let alice_peer_count = alice_peers
        .get("peers")
        .and_then(|v| v.as_array())
        .map(|v| v.len())
        .ok_or("Failed to read Alice peer count")?;
    let bob_peer_count = bob_peers
        .get("peers")
        .and_then(|v| v.as_array())
        .map(|v| v.len())
        .ok_or("Failed to read Bob peer count")?;

    // Create or load a mining wallet
    create_or_load_wallet(&bitcoin_rpc, "miner")?;

    // Use wallet-scoped RPC endpoint for all wallet commands.
    let miner_wallet_rpc = BitcoinClient::new(
        "http://localhost:18443/wallet/miner",
        Auth::UserPass("alice".to_string(), "password".to_string()),
    )?;

    // Generate a new mining address from the mining wallet
    let mining_address: String = miner_wallet_rpc.call("getnewaddress", &[])?;

    // Mine 101 blocks: coinbase outputs need 100 confirmations before they are spendable.
    let _: Value = bitcoin_rpc.call(
        "generatetoaddress",
        &[serde_json::json!(101), serde_json::json!(mining_address)],
    )?;

    // Verify wallet balance
    let wallet_balance: f64 = miner_wallet_rpc.call("getbalance", &[])?;
    if wallet_balance < 1.0 {
        return Err("Mining wallet has insufficient balance after mining".into());
    }

    // Create an on-chain address for Alice and send 1 BTC from mining wallet to this address
    let alice_newaddr = call_alice_ln("newaddr", serde_json::json!({}))?;
    let alice_chain_address = alice_newaddr
        .get("bech32")
        .and_then(|v| v.as_str())
        .ok_or("Failed to get Alice on-chain address")?;

    let _: String = miner_wallet_rpc.call(
        "sendtoaddress",
        &[
            serde_json::json!(alice_chain_address),
            serde_json::json!(1.0),
        ],
    )?;

    // Mine blocks to confirm Alice's on-chain funding.
    let _: Value = bitcoin_rpc.call(
        "generatetoaddress",
        &[serde_json::json!(6), serde_json::json!(mining_address)],
    )?;

    // Wait until Alice sees confirmed, unreserved UTXOs before attempting fundchannel.
    let min_required_msat = 600_000_000u64;
    let mut available_msat = 0u64;
    for _ in 0..30 {
        let listfunds = call_alice_ln("listfunds", serde_json::json!({}))?;
        available_msat = get_available_confirmed_msat(&listfunds)?;
        if available_msat >= min_required_msat {
            break;
        }
        sleep(Duration::from_secs(2));
    }
    if available_msat < min_required_msat {
        return Err("Alice does not have enough confirmed spendable funds for fundchannel".into());
    }

    // Open a payment channel from Alice to Bob with 500,000 satoshis capacity
    let _fundchannel_result = call_alice_ln(
        "fundchannel",
        serde_json::json!({
            "id": bob_node_id,
            "amount": 500000
        }),
    )?;

    // Mine blocks to confirm the channel opening transaction.
    let _: Value = bitcoin_rpc.call(
        "generatetoaddress",
        &[serde_json::json!(6), serde_json::json!(mining_address)],
    )?;

    // Wait a few seconds for nodes to recognize the confirmed channel.
    let mut alice_channel: Option<Value> = None;
    let mut bob_channel: Option<Value> = None;

    for _ in 0..30 {
        let alice_channels =
            call_alice_ln("listpeerchannels", serde_json::json!({ "id": bob_node_id }))?;
        let bob_channels = call_bob_ln(
            "listpeerchannels",
            serde_json::json!({ "id": alice_node_id }),
        )?;

        let alice_ch = get_channel_for_peer(&alice_channels, bob_node_id);
        let bob_ch = get_channel_for_peer(&bob_channels, alice_node_id);

        let alice_state = alice_ch
            .as_ref()
            .and_then(|ch| ch.get("state"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let bob_state = bob_ch
            .as_ref()
            .and_then(|ch| ch.get("state"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if alice_state == "CHANNELD_NORMAL" && bob_state == "CHANNELD_NORMAL" {
            alice_channel = alice_ch;
            bob_channel = bob_ch;
            break;
        }

        sleep(Duration::from_secs(2));
    }

    // Verify channel is active on both Alice's side and Bob's side
    let alice_channel = alice_channel.ok_or("Alice channel did not reach CHANNELD_NORMAL")?;
    let bob_channel = bob_channel.ok_or("Bob channel did not reach CHANNELD_NORMAL")?;

    // Get channel details from both Alice's and Bob's perspectives
    let channel_id = alice_channel
        .get("channel_id")
        .and_then(|v| v.as_str())
        .ok_or("Missing channel_id")?;
    let funding_txid = alice_channel
        .get("funding_txid")
        .and_then(|v| v.as_str())
        .ok_or("Missing funding_txid")?;
    let alice_channel_state = alice_channel
        .get("state")
        .and_then(|v| v.as_str())
        .ok_or("Missing Alice channel state")?;
    let bob_channel_state = bob_channel
        .get("state")
        .and_then(|v| v.as_str())
        .ok_or("Missing Bob channel state")?;
    let total_msat = msat_to_u64(
        alice_channel
            .get("total_msat")
            .ok_or("Missing total_msat")?,
    )?;
    let alice_balance_msat = msat_to_u64(
        alice_channel
            .get("to_us_msat")
            .ok_or("Missing Alice to_us_msat")?,
    )?;
    let bob_balance_msat = msat_to_u64(
        bob_channel
            .get("to_us_msat")
            .ok_or("Missing Bob to_us_msat")?,
    )?;

    // Write to out.txt
    let output = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
        alice_node_id,
        bob_node_id,
        alice_peer_count,
        bob_peer_count,
        channel_id,
        funding_txid,
        alice_channel_state,
        bob_channel_state,
        total_msat,
        alice_balance_msat,
        bob_balance_msat
    );

    fs::write("../out.txt", output)?;

    Ok(())
}
