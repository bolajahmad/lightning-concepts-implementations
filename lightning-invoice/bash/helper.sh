bitcoin_cli () {
  docker exec bitcoind bitcoin-cli -regtest -rpcuser=alice -rpcpassword=password "$@"
}

ln_cli () {
  docker exec ln-node lightning-cli --network=regtest "$@"
}
