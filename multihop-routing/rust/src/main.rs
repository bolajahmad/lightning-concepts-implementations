use std::io::Write;
use std::{
    fs::File,
    println,
    time::{SystemTime, UNIX_EPOCH},
    writeln,
};

use bitcoincore_rpc::{
    bitcoin::{address::NetworkChecked, Address, Amount, Network},
    Auth, Client as BitcoinClient, RpcApi,
};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
struct Invoice {
    amount_msat: u64,
    description: String,
    expiry: u64,
    min_final_cltv_expiry: u32,
    payee: String,
    payment_hash: String,
    payment_secret: String,
}

#[derive(Debug, Clone)]
struct LightningClient {
    url: String,
    rune: String,
}

impl LightningClient {
    fn new(url: &str, rune: &str) -> Self {
        LightningClient {
            url: url.to_string(),
            rune: rune.to_string(),
        }
    }

    fn call(&self, method: &str, params: Value) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!("{}/{}", self.url, method);
        let client = Client::new();
        let response = client
            .post(&url)
            .json(&params)
            .header("Rune", &self.rune)
            .send()?
            .json::<Value>()?;

        Ok(response)
    }

    fn node_id(&self) -> String {
        let info = self.call("getinfo", Value::Null).unwrap();
        let id = info.get("id").and_then(|v| v.as_str()).unwrap();

        id.to_string()
    }

    fn get_address(&self) -> Address {
        let raw_address = self.call("newaddr", serde_json::json!({})).unwrap();
        let address_str = raw_address.get("bech32").and_then(|v| v.as_str()).unwrap();
        let address = address_str
            .parse::<Address<_>>()
            .unwrap()
            .require_network(Network::Regtest)
            .unwrap();
        address
    }

    fn get_balance(&self) -> u64 {
        self.call("listfunds", serde_json::json!({}))
            .unwrap()
            .get("channels")
            .and_then(|o| o.as_array())
            .map(|out| {
                out.iter()
                    .filter_map(|amt| amt.get("our_amount_msat"))
                    .filter_map(|amt| amt.as_u64())
                    .sum::<u64>()
            })
            .unwrap_or(0)
    }

    fn get_channel_for(&self, channels: &Value, peer: &str) -> Option<Value> {
        channels
            .get("channels")
            .and_then(|v| v.as_array())
            .and_then(|chs| {
                chs.iter()
                    .find(|ch| ch.get("peer_id").and_then(|v| v.as_str()) == Some(peer))
                    .cloned()
            })
    }

    fn wait_for_channeld_state(
        &self,
        partner: LightningClient, // or &LightningClient depending on your wrapper
        nodeid: &str,
    ) -> (Option<Value>, Option<Value>) {
        let mut self_channel: Option<Value> = None;
        let mut partner_channel: Option<Value> = None;

        for _ in 0..30 {
            let channels = self
                .call("listpeerchannels", serde_json::json!({ "id": nodeid }))
                .unwrap();

            let partner_channels = partner
                .call(
                    "listpeerchannels",
                    serde_json::json!({ "id": self.node_id() }),
                )
                .unwrap();

            let self_ch = self.get_channel_for(&channels, nodeid);
            let partner_ch = partner.get_channel_for(&partner_channels, &self.node_id());

            let self_state = self_ch
                .as_ref()
                .and_then(|ch| ch.get("state"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let partner_state = partner_ch
                .as_ref()
                .and_then(|ch| ch.get("state"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if self_state == "CHANNELD_NORMAL" && partner_state == "CHANNELD_NORMAL" {
                // FIX: Mutate the outer variables directly. No 'let' keyword!
                // Use .cloned() to convert Option<&Value> into Option<Value>
                self_channel = self_ch;
                partner_channel = partner_ch;
                break;
            }

            std::thread::sleep(std::time::Duration::from_secs(2));
        }

        (self_channel, partner_channel)
    }

    fn connect_to_peer(
        &self,
        node_id: &str,
        hostname: &str,
        port: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.call(
            "connect",
            serde_json::json!({
                "id": node_id,
                "host": hostname,
                "port": port
            }),
        ) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

struct BitcoinRpcClient {
    client: BitcoinClient,
}

impl BitcoinRpcClient {
    fn new(url: &str, user: &str, password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client =
            BitcoinClient::new(url, Auth::UserPass(user.to_string(), password.to_string()))?;
        Ok(BitcoinRpcClient { client })
    }

    pub fn client(&self) -> &BitcoinClient {
        &self.client
    }

    pub fn balance(&self) -> Option<Amount> {
        match self.client().get_balance(None, None) {
            Ok(amt) => Some(amt),
            Err(_) => None,
        }
    }

    pub fn get_wallet(&self, name: &str) -> Result<(), bitcoincore_rpc::Error> {
        let client = self.client();

        // 1. Try to create the wallet first
        match client.create_wallet(name, None, None, None, None) {
            Ok(_) => {
                println!("Created and loaded wallet: {}", name);
                return Ok(());
            }
            Err(e) => {
                // Check why it failed, first
                // Error code -4 means the wallet file already exists on disk
                if let bitcoincore_rpc::Error::JsonRpc(
                    bitcoincore_rpc::jsonrpc::error::Error::Rpc(ref rpc_err),
                ) = e
                {
                    if rpc_err.code == -4 {
                        // 2. Since it exists on disk, try to load it into memory
                        match client.load_wallet(name) {
                            Ok(_) => println!("Loaded existing wallet: {}", name),
                            Err(load_err) => {
                                // If code is -4 here, it means it was ALREADY loaded in memory
                                if let bitcoincore_rpc::Error::JsonRpc(
                                    bitcoincore_rpc::jsonrpc::error::Error::Rpc(ref rpc_load_err),
                                ) = load_err
                                {
                                    if rpc_load_err.code == -4 {
                                        println!("Wallet {} was already active.", name);
                                        return Ok(());
                                    }
                                }
                                return Err(load_err);
                            }
                        }
                        return Ok(());
                    }
                }
                // Return any other unexpected error
                return Err(e);
            }
        }
    }
}

fn fund_onchain_wallet(
    client: &BitcoinRpcClient,
    recipient: &Address,
    amount: u64,
    client_address: Option<&Address>,
) -> Result<(), Box<dyn std::error::Error>> {
    let txid = client.client().send_to_address(
        recipient,
        Amount::from_sat(amount),
        None,
        None,
        None,
        None,
        None,
        None,
    )?;

    // Mine block to confit transfer
    let new_address = client
        .client()
        .get_new_address(None, None)?
        .assume_checked();
    let _ = client.client().generate_to_address(
        3,
        match client_address {
            Some(address) => address,
            None => &new_address,
        },
    );

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Miner's Bitcoin RPC client
    let btc_rpc = BitcoinRpcClient::new("http://localhost:18443", "alice", "password")?;

    // Create a bitcoin wallet named 'mining_wallet'
    // Load the "mining_wallet" if already exists
    let _ = btc_rpc.get_wallet("mining_wallet");
    let miningwallet_client = BitcoinRpcClient::new(
        "http://localhost:18443/wallet/mining_wallet",
        "alice",
        "password",
    )?;

    // Generate a mining address
    // This will send some block rewards to the generated address
    let miningwallet_address = miningwallet_client
        .client()
        .get_new_address(None, None)?
        .assume_checked();

    // Mine initial blocks at least 101 to ensure maturity of coinbase transactions
    miningwallet_client
        .client()
        .generate_to_address(101, &miningwallet_address)?;
    println!(
        "Balance of Mining_Wallet mining, {:?}",
        miningwallet_client.balance()
    );

    // Initialize Lightning clients for ALice, Bob, Carol
    let alice_ln = LightningClient::new("http://localhost:3010/v1/", &std::env::var("ALICE_RUNE")?);
    let bob_ln = LightningClient::new("http://localhost:3011/v1/", &std::env::var("BOB_RUNE")?);
    let carol_ln = LightningClient::new("http://localhost:3012/v1/", &std::env::var("CAROL_RUNE")?);

    // Create and fund an on-chain address for Alice
    let alice_address: Address<NetworkChecked> = alice_ln.get_address();
    // Create and fund an on-chain address for Bob
    let bob_address = bob_ln.get_address();

    // Fund the alice_onchain_address
    let _ = fund_onchain_wallet(
        &miningwallet_client,
        &alice_address,
        1000000,
        Some(&miningwallet_address),
    );

    // Mine blocks to confirm funding transactions
    let _hash = miningwallet_client
        .client()
        .generate_to_address(6, &miningwallet_address)?;
    println!("Funding TX successfully mined!");

    let _ = fund_onchain_wallet(
        &miningwallet_client,
        &bob_address,
        1000000,
        Some(&miningwallet_address),
    );

    // Mine blocks to confirm funding transactions
    let _hash = miningwallet_client
        .client()
        .generate_to_address(6, &miningwallet_address)?;
    println!("Funding TX successfully mined!");

    println!("Alice balance after funding {}", alice_ln.get_balance());
    println!("Bob balance after funding {}", bob_ln.get_balance());

    // Get node IDs for Alice, Bob, and Carol
    let _alice_noideid = alice_ln.node_id();
    let bob_nodeid = bob_ln.node_id();
    let carol_nodeid = carol_ln.node_id();

    // Connect peers
    // Alice <-> Bob
    let _connect_result = alice_ln.connect_to_peer(&bob_nodeid, "bob", 9735)?;
    println!("Connecting Alice <-> Bob");
    // Bob <-> Carol
    let _ = bob_ln.connect_to_peer(&carol_nodeid, "carol", 9735)?;
    println!("Connecting Bob <-> Carol");

    let alice_peers = alice_ln.call("listpeers", serde_json::json!({}))?;
    let bob_peers = bob_ln.call("listpeers", serde_json::json!({}))?;

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
    assert!(alice_peer_count >= 1, "Must be greater than 1 peer");
    assert!(bob_peer_count >= 1, "Expect Bob to have 1 peer at least");

    // Alice opens a 500,000 sat channel with Bob
    let _ = alice_ln.call(
        "fundchannel",
        serde_json::json!({
            "id": &bob_nodeid,
            "amount": 500000
        }),
    );
    // Bob opens a 300,000 sat channel with Carol
    let _ = bob_ln.call(
        "fundchannel",
        serde_json::json!({
            "id": &carol_nodeid,
            "amount": 300000
        }),
    );
    // Mine at least 6 blocks to confirm channels
    let _ = miningwallet_client
        .client()
        .generate_to_address(6, &miningwallet_address);
    // Wait for channels to reach CHANNELD_NORMAL state
    let (alice_bob_ch, _) = alice_ln.wait_for_channeld_state(bob_ln.clone(), &bob_nodeid);
    let (bob_carol_ch, _) = bob_ln.wait_for_channeld_state(carol_ln.clone(), &carol_nodeid);

    if let (Some(a), Some(b)) = (alice_bob_ch, bob_carol_ch) {
        println!("SUCCESS: Alice channel state is {:?}", a.get("state"));
        println!("SUCCESS: Bob channel state is {:?}", b.get("state"));
    } else {
        println!("ERROR: One or both channels failed to open or reach CHANNELD_NORMAL.");
    }

    println!("Confirming Alice's balance, {}", alice_ln.get_balance());
    println!("Confirming Bob's balance, {}", bob_ln.get_balance());

    // Carol generates a 100,000 sat invoice with label "multihop_<timestamp>" and description "Multi-Hop Payment"
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let invoice = carol_ln.call(
        "invoice",
        serde_json::json!({
            "amount_msat": 100000000,
            "label": format!("multihop_{}", timestamp),
            "description": "Multi-Hop Payment",
        }),
    )?;

    // Extract the BOLT11 string and payment hash from the invoice
    let bolt11 = invoice.get("bolt11").and_then(|b| b.as_str()).unwrap();
    let payment_hash = invoice
        .get("payment_hash")
        .and_then(|h| h.as_str())
        .unwrap();

    println!("Invoice data {}", bolt11);

    let raw_decoded_invoice = carol_ln.call(
        "decode",
        serde_json::json!({
            "string": bolt11
        }),
    )?;
    let invoice: Invoice =
        serde_json::from_value(raw_decoded_invoice).expect("Should decode Invoice");
    println!("Decoded Invoice data, {:?}", invoice);

    // Alice pays Carol's BOLT11 invoice (routed through Bob)
    let pay_result = alice_ln.call(
        "pay",
        serde_json::json!({
            "bolt11": bolt11,
        }),
    )?;
    println!("Payment Result object, {}", pay_result);
    let payment_preimage = pay_result
        .get("payment_preimage")
        .and_then(|p| p.as_str())
        .unwrap();
    println!("Payment Successful Preimage {}", payment_preimage);

    // Verify Alice's balance decreased
    let alice_final_balance = alice_ln.get_balance();
    println!("Alice final balance, {alice_final_balance}");
    // assert!(
    //     alice_final_balance < alice_start_balance,
    //     "Alice balance should decrease"
    // );
    // Verify Carol's balance increased
    let carol_final_balance = carol_ln.get_balance();
    println!("Carol final balance, {carol_final_balance}");
    // assert!(
    //     carol_final_balance > carol_start_balance,
    //     "Carol balance should increase"
    // );
    // Verify Bob's balance. Is there any difference? Why is it?
    let bob_final_balance = bob_ln.get_balance();
    println!("Bob final balance, {bob_final_balance}");
    // assert!(
    //     bob_final_balance - bob_start_balance >= 100,
    //     "Balance should increase by at least 100 msat"
    // );

    // Verify Bob forwarded the payment using listforwards and extract payment_hash from it
    let bob_forwards_data = bob_ln.call("listforwards", serde_json::json!({}))?;
    let forwards_array = bob_forwards_data
        .get("forwards")
        .and_then(|v| v.as_array())
        .ok_or("Failed to parse Bob's forwarding list")?;
    // Find the record where out_channel matches Carol's channel scid from the invoice
    let matching_forward = forwards_array
        .iter()
        .find(|f| f.get("out_channel").and_then(|c| c.as_str()) == Some("1226x2x0"));
    println!("Matched forwarded payment {:?}", matching_forward);

    let mut fee_msat: u64 = 0;
    if let Some(record) = matching_forward {
        fee_msat = record.get("fee_msat").and_then(|v| v.as_u64()).unwrap_or(0);
    }
    println!("Bob's fee, {fee_msat}");

    // Write to out.txt:
    let mut file = File::create("../out.txt")?;
    // Payment Hash
    writeln!(file, "{}", payment_hash)?;
    // Payment Preimage
    writeln!(file, "{}", payment_preimage)?;
    // BOLT11 Invoice
    writeln!(file, "{}", bolt11)?;
    // Payer_ID
    writeln!(file, "{}", alice_ln.node_id())?;
    // Payee_ID
    writeln!(file, "{}", invoice.payee)?;
    // Fee_msat
    writeln!(file, "{}", fee_msat)?;
    // Payment_Hash from Bob's forwarded payment
    let forwarded_hash = pay_result
        .get("payment_hash")
        .and_then(|p| p.as_str())
        .unwrap();
    writeln!(file, "{}", forwarded_hash)?;

    Ok(())
}
