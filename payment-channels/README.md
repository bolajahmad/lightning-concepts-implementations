# Mastering Lightning Network - Exercise 2: Opening Payment Channels

## Overview

For the second exercise you will:

1. **Set up** Bitcoin Core and two Core Lightning (CLN) nodes (Alice and Bob) using Docker.
2. **Interact** with Bitcoin Core and both Lightning nodes on `regtest`.
3. **Connect** Alice and Bob as peers on the Lightning Network.
4. **Fund** Alice's Lightning node and open a 500,000 satoshi payment channel to Bob.
5. **Output** a small report file named (`output.txt`) in the **current directory** demonstrating successful peer connection and channel opening.
6. **Target Locations** for the solution code for each language are given below:
   - Bash: [solution.sh](./bash/solution.sh)
   - JavaScript: [index.js](./javascript/index.js)
   - Python: [main.py](./python/main.py)
   - Rust: [main.rs](./rust/src/main.rs)

## Problem Statement

Before Lightning nodes can send or receive payments between each other, they need to establish peer connections and open payment channels. The following exercise introduces us to the basics of setting up multiple Lightning nodes and connecting them through payment channels.

We will be using Docker to set up Bitcoin Core and two Core Lightning nodes (Alice and Bob), after which we will connect them as peers, fund Alice's node with `regtest` Bitcoin, and open a 500,000 satoshi payment channel from Alice to Bob

## Solution Requirements

You need to write code in any one of `bash`, `javascript`, `python` or `rust` that will do the following:

### Setup - Docker Compose

The assignment uses Docker Compose to run Bitcoin Core and two Core Lightning nodes. The configuration is provided in [docker-compose.yml](./docker-compose.yml).

Services:
- **bitcoind**: Bitcoin Core node running on regtest
  - RPC port: 18443
  - RPC credentials: alice/password
- **alice**: Core Lightning node (Alice)
  - Connected to bitcoind
  - Network: regtest
  - P2P port: 9735
  - REST port: 3010
- **bob**: Core Lightning node (Bob)
  - Connected to bitcoind
  - Network: regtest
  - P2P port: 9736 (external) → 9735 (internal)
  - REST port: 3011 (external) → 3010 (internal)

To start the services:
```bash
docker compose up -d
```
To stop the services:
```bash
docker compose down -v
```

### Node Interaction - Choose ONE Language

Implement the tasks in exactly one of the language-specific directories: `bash`, `javascript`, `python`, or `rust`.

Each implementation uses helper functions located in the directories.

Your program must:

  - Connect Alice and Bob as Peers
  - Get Alice's and Bob's node IDs
  - Connect Alice to Bob using Bob's node ID and container address
  - Create/load a mining wallet and generate a new address from it
  - Mine new blocks to this address
  - Create Alice's on-chain address
  - Send Bitcoin from the mining wallet to Alice
  - Mine blocks to confirm the transaction
  - Verify Alice received the funds
  - Open a 500,000 satoshi payment channel from Alice to Bob
  - Mine blocks to confirm the channel funding transaction
  - Wait for the channel to reach `CHANNELD_NORMAL` state
  - Verify channel state and balances on both Alice and Bob sides

### Output

Output the following channel setup details to `out.txt` in the root directory. Each field should be in `key=value` format on a new line:
- Alice's Lightning node ID (66-character hex string)
- Bob's Lightning node ID (66-character hex string)
- Number of peers connected to Alice
- Number of peers connected to Bob
- The channel identifier
- The funding transaction ID (64-character hex string)
- Alice's view of the channel state (should be `CHANNELD_NORMAL`)
- Bob's view of the channel state (should be `CHANNELD_NORMAL`)
- Total channel capacity in millisatoshis
- Alice's balance in the channel in millisatoshis
- Bob's balance in the channel in millisatoshis

Sample output file:
```
0364a1b3c5d8f2e4a1b3c5d8f2e4a1b3c5d8f2e4a1b3c5d8f2e4a1b3c5d8f2e4a1
02a1b3c5d8f2e4a1b3c5d8f2e4a1b3c5d8f2e4a1b3c5d8f2e4a1b3c5d8f2e4a1b3
1
1
abcdef123456789...
a1b2c3d4e5f6789abcdef0123456789abcdef0123456789abcdef0123456789a
CHANNELD_NORMAL
CHANNELD_NORMAL
500000000
500000000
0
```

## Code Structure

Each language implementation follows a consistent pattern:

```
<language>/
  ├── helper.<ext>      # Helper functions for Bitcoin/Lightning CLI
  ├── main.<ext>        # Main implementation
  └── run-<language>.sh # Script to run the implementation
```

Helper functions abstract Docker CLI interactions:
- `bitcoin_cli(command)`: Execute bitcoin-cli commands via Docker
- `alice_ln_cli(command)`: Execute lightning-cli commands for Alice via Docker
- `bob_ln_cli(command)`: Execute lightning-cli commands for Bob via Docker

## Local Testing

### Prerequisites

| Language       | Prerequisite packages                  |
| -------------- |----------------------------------------|
| **Bash**       | Docker, Docker Compose, `jq`           |
| **JavaScript** | Docker, Docker Compose, Node.js ≥ 20   |
| **Python**     | Docker, Docker Compose, Python ≥ 3.9  |
| **Rust**       | Docker, Docker Compose, Rust toolchain |

### Setup Steps

1. **Install Docker and Docker Compose**
   ```bash
   # Follow Docker installation guide for your OS
   # https://docs.docker.com/get-docker/
   ```

2. **Install Language Components**

   #### Bash
   - **Version** 4.0 or higher (usually pre-installed on Linux/macOS)
    ```bash
    # check version
    bash --version
    # to install jq [JSON processor to parse JSON responses]
    sudo apt-get update && sudo apt-get install -y jq       # Ubuntu/Debian
    brew install jq                                         # macOS
    ```

   #### JavaScript
   - **Node.js Version** 20.x or higher

   ```bash
    # check version
    node --version
    npm --version
    # install nvm
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
    nvm install 20
    nvm use 20
    # install project dependencies
    cd javascript
    npm install
    ```

   #### Python
   - **Version** 3.9 or higher
    ```bash
    # check version
    python3 --version
    pip3 --version

    # install python
    sudo apt-get update && sudo apt-get install -y python3 python3-pip python3-venv       # Ubuntu/Debian
    brew install python@3.9                                                               # macOS

    # install required dependencies
    pip3 install requests python-bitcoinrpc
    ```

   #### Rust
   - **Version** 1.70.0 or higher
    ```bash
    # check version
    rustc --version
    cargo --version

    # installation via rustup
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh                        # Linux/macOS
    # Follow the prompts, then reload your shell.
    source $HOME/.cargo/env

    # build essentials
    sudo apt-get install -y build-essential pkg-config libssl-dev                         # Ubuntu/Debian

    # Build the project
    cd rust
    cargo build
    ```

3. **Start the nodes**
   ```bash
   docker compose up -d
   ```

### Local Testing Steps
It's a good idea to run the whole test locally to ensure your code is working properly.

- Uncomment the specific line in [run.sh](./run.sh) corresponding to your language of choice. 
- Grant execution permission to [test.sh](./test.sh), by running `chmod +x ./test.sh`.
- Execute `./test.sh`.
- The test script will run your script and verify the output. If the test script passes, you have successfully completed the challenge and are ready to submit your solution.

### Common Issues

- If docker containers not running ensure `docker compose up -d` completed successfully
- Make sure Docker daemon is running and you have permissions using `docker ps`
- Ensure `out.txt` has exactly 11 lines in the correct order
- The autograder will run the test script on an Ubuntu 22.04 environment. Make sure your script is compatible with this environment.
- If you are unable to run the test script locally, you can submit your solution and check the results on the Github.

## Submission

- Commit all code inside the appropriate language directory and the modified `run.sh`.
  ```
  git add .
  git commit -m "Week 2 solution"
  ```
- Push to the main branch:
  ```
    git push origin main
  ```
- The autograder will run your script against a test script to verify the functionality.
- Check the status of the autograder on the Github Classroom portal to see if it passed successfully or failed. Once you pass the autograder with a score of 100, you have successfully completed the challenge.
- You can submit multiple times before the deadline. The latest submission before the deadline will be considered your final submission.
- You will lose access to the repository after the deadline.

## Evaluation Criteria

| Area                   | Weight      | Description                                                                                                                         |
| ---------------------- | ----------- |-------------------------------------------------------------------------------------------------------------------------------------|
| **Autograder**         | **Primary** | Your code must pass the autograder [test script](./test/test.spec.ts).                                                                              |
| **Explainer comments** | Required    | Include comments explaining each step of your code.                                                                                 |
| **Code quality**       | Required    | Your code should be well-organized, commented, and adhere to best practices like idiomatic style, meaningful names, error handling. |

### Plagiarism Policy
Our plagiarism detection checker thoroughly identifies any instances of copying or cheating. Participants are required to publish their solutions in the designated repository, which is private and accessible only to the individual and the administrator. Solutions should not be shared publicly or with peers. In case of plagiarism, both parties involved will be directly disqualified to maintain fairness and integrity.

### AI Usage Disclaimer
You may use AI tools like ChatGPT to gather information and explore alternative approaches, but avoid relying solely on AI for complete solutions. Verify and validate any insights obtained and maintain a balance between AI assistance and independent problem-solving.

## Why These Restrictions?
These rules are designed to enhance your understanding of the technical aspects of Bitcoin. By completing this assignment, you gain practical experience with the technology that secures and maintains the trustlessness of Bitcoin. This challenge not only tests your ability to develop functional Bitcoin applications but also encourages deep engagement with the core elements of Bitcoin technology.
