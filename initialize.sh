#!/bin/bash

set -e

NETWORK="$1"

WASM_PATH="target/wasm32-unknown-unknown/release/"
CLEAR_WASM=$WASM_PATH"liquidity_pool"
DEPLOYER_WASM=$WASM_PATH"liquidity_pool_deployer"
TOKEN_NATIVE_ID="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"

case "$1" in
standalone)
  echo "Using standalone network"
  SOROBAN_NETWORK_PASSPHRASE="Standalone Network ; February 2017"
  FRIENDBOT_URL="http://localhost:8000/friendbot"
  SOROBAN_RPC_URL="http://localhost:8000/rpc"
  ;;
futurenet)
  echo "Using Futurenet network"
  SOROBAN_NETWORK_PASSPHRASE="Test SDF Future Network ; October 2022"
  FRIENDBOT_URL="https://friendbot-futurenet.stellar.org/"
  SOROBAN_RPC_URL="https://rpc-futurenet.stellar.org:443"
  ;;
testnet)
  echo "Using Testnet network"
  SOROBAN_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
  FRIENDBOT_URL="https://friendbot.stellar.org/"
  SOROBAN_RPC_URL="https://soroban-testnet.stellar.org:443"
  ;;
*)
  echo "Usage: $0 standalone|futurenet|testnet"
  exit 1
  ;;
esac

echo Add the $NETWORK network to cli client 
  soroban network add \
  --global $NETWORK \
  --rpc-url "$SOROBAN_RPC_URL" \
  --network-passphrase "$SOROBAN_NETWORK_PASSPHRASE"

if !(soroban soroban keys address clear-admin | grep admin 2>&1 >/dev/null); then
  echo Create the admin identity
  soroban keys generate clear-admin --network $NETWORK
fi

CLEAR_ADMIN_SECRET="$(soroban keys show clear-admin)"
CLEAR_ADMIN_ADDRESS="$(soroban keys address clear-admin)"

echo "Admin Public key: $CLEAR_ADMIN_ADDRESS"
echo "Admin Secret key: $CLEAR_ADMIN_SECRET"

ARGS="--network $NETWORK --source-account clear-admin"

echo Build and optimize liquidity-pool
cargo build --target wasm32-unknown-unknown --release -p liquidity-pool
soroban contract optimize --wasm $CLEAR_WASM".wasm"

echo Build and optimize liquidity-pool-deployer
cargo build --target wasm32-unknown-unknown --release -p liquidity-pool-deployer
soroban contract optimize --wasm $DEPLOYER_WASM".wasm"

echo Deploy the clear contract deployer
CONTRACT_DEPLOYER_ID="$(
  soroban contract deploy $ARGS \
  --wasm $DEPLOYER_WASM".optimized.wasm"
)"
echo "Contract deployed successfully with ID: $CONTRACT_DEPLOYER_ID"

echo "Done"