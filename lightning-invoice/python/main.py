import requests
import os
from bitcoinrpc.authproxy import AuthServiceProxy, JSONRPCException


def call_cln(method, params=None):
    rune = os.environ.get('CLN_RUNE')
    url = f'http://localhost:3010/v1/{method}'
    response = requests.post(
        url,
        json=params or {},
        headers={'Rune': rune}
    )
    return response.json()

def main():

    try:
        # Get blockchain info
        rpc = AuthServiceProxy("http://alice:password@localhost:18443")
        print("Blockchain Info:", rpc.getblockchaininfo())

        # Get Lightning node info
        ln_info = call_cln('getinfo')
        print("Lightning Node Info:", ln_info)

        # Create a new address for funding using lightning-cli and store it in CLN_ADDRESS

        # Check if wallet exists, if not Create a bitcoin wallet named 'mining_wallet' using bitcoin-cli for mining

        # Generate a new address and mine blocks to it. How many blocks need to mined? Why?

        # Fund the Lightning node by sending 0.1 BTC from the mining wallet to CLN_ADDRESS

        # Confirm the funding transaction by mining 6 blocks

        # Verify Lightning wallet balance using lightning-cli listfunds

        # Create an invoice with parameters and store the invoice string:
        # - Amount: 50,000 satoshis (50000000 millisatoshis)
        # - Label: Generate unique label using timestamp (e.g., "invoice_$(date +%s)")
        # - Description: "Coffee Payment"
        # - Expiry: 3600 seconds

        # Decode the invoice string using lightning-cli decodepay and verify the parameters

        # Output the invoice details in the specified format to out.txt
        # - Payment hash
        # - BOLT11 invoice string
        # - Amount
        # - Description
        # - Expiry time
    except JSONRPCException as e:
        print("An error occurred", e)

if __name__ == "__main__":
    main()
