#!/bin/bash
source bash/helper.sh

# Get blockchain info using bitcoin-cli
blockchain_info=$(bitcoin_cli getblockchaininfo)
echo "Blockchain Info: $blockchain_info"

# Get Alice's node info
alice_info=$(alice_ln_cli getinfo)
echo "Alice Node Info: $alice_info"

# Get Bob's node info
bob_info=$(bob_ln_cli getinfo)
echo "Bob Node Info: $bob_info"

# Get Alice's node ID

# Get Bob's node ID

# Connect Alice to Bob as a peer

# Verify peer connection from both Alice's and Bob's perspectives

# Create or load a mining wallet

# Generate a new mining address from the mining wallet

# How many blocks need to be mined to the mining address? Why?

# Verify wallet balance

# Create an on-chain address for Alice and send 1 BTC from mining wallet to this address

# Mine blocks to confirm the funding transaction. How many blocks and why?

# Open a payment channel from Alice to Bob with 500,000 satoshis capacity

# Mine some blocks to confirm the channel opening transaction.

# Wait a few seconds for nodes to recognize the confirmed channel

# Verify channel is active on both Alice's side and Bob's side

# Get channel details from both Alice's and Bob's perspectives

# Check if Alice and Bob are peers

# Write to out.txt
