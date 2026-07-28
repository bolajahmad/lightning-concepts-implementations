const fs = require("fs").promises;
const axios = require("axios");
const bitcoin = require("bitcoin-core");

// Call Alice's Lightning node via CLN REST API on port 3010
async function callAliceLn(method, params = {}) {
  const response = await axios.post(
    `http://localhost:3010/v1/${method}`,
    params,
    { headers: { Rune: process.env.ALICE_RUNE } },
  );
  return response.data;
}

// Call Bob's Lightning node via CLN REST API on port 3011
async function callBobLn(method, params = {}) {
  const response = await axios.post(
    `http://localhost:3011/v1/${method}`,
    params,
    { headers: { Rune: process.env.BOB_RUNE } },
  );
  return response.data;
}

async function main() {
  // Bitcoin RPC client
  const bitcoinClient = new bitcoin({
    network: "regtest",
    username: "alice",
    password: "password",
    port: 18443,
  });

  console.log("Bitcoin Node Info:", await bitcoinClient.getBlockchainInfo());
  // Get Alice's node info
  const alice_info = await callAliceLn("getinfo");
  console.log("Alice Node Info:", alice_info);

  // Get Bob's node info
  const bob_info = await callBobLn("getinfo");
  console.log("Bob Node Info:", bob_info);

  // Get Alice's node ID

  // Get Bob's node ID

  // Connect Alice to Bob as a peer

  // Verify peer connection from both Alice's and Bob's perspectives

  // Create or load a mining wallet

  // Generate a new mining address from the mining wallet

  // How many blocks need to be mined to the mining address? Why?

  // Verify wallet balance

  // Create an on-chain address for Alice and send 1 BTC from mining wallet to this address

  // Mine blocks to confirm the funding transaction. How many blocks and why?

  // Open a payment channel from Alice to Bob with 500,000 satoshis capacity

  // Mine some blocks to confirm the channel opening transaction.

  // Wait a few seconds for nodes to recognize the confirmed channel

  // Verify channel is active on both Alice's side and Bob's side

  // Get channel details from both Alice's and Bob's perspectives

  // Check if Alice and Bob are peers

  // Write to out.txt
}

main().catch(err => {
    console.error('Error:', err);
    process.exit(1);
});
