import requests
import os
from bitcoinrpc.authproxy import AuthServiceProxy, JSONRPCException

def call_alice_ln(method, params=None):
    """Call Alice's Lightning node via CLN REST API on port 3010"""
    rune = os.environ.get('ALICE_RUNE')
    url = f'http://localhost:3010/v1/{method}'
    response = requests.post(
        url,
        json=params or {},
        headers={'Rune': rune}
    )
    return response.json()


def call_bob_ln(method, params=None):
    """Call Bob's Lightning node via CLN REST API on port 3011"""
    rune = os.environ.get('BOB_RUNE')
    url = f'http://localhost:3011/v1/{method}'
    response = requests.post(
        url,
        json=params or {},
        headers={'Rune': rune}
    )
    return response.json()


def main():
    try:
        # Bitcoin RPC client
        bitcoin_rpc = AuthServiceProxy("http://alice:password@localhost:18443")
        print("Blockchain Info:", bitcoin_rpc.getblockchaininfo())

        # Get Alice's node info
        alice_info = call_alice_ln("getinfo")
        print("Alice Node Info:", alice_info)

        # Get Bob's node info
        bob_info = call_bob_ln("getinfo")
        print("Bob Node Info:", bob_info)

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

    except JSONRPCException as e:
        print("An error occurred", e)

if __name__ == "__main__":
    main()
