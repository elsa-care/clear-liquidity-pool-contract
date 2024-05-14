#!/bin/bash

set -e

NETWORK=$1

if [[ $NETWORK == "testnet" ]]; then
  echo "Using Testnet network"
  SOROBAN_RPC_URL="https://rpc-futurenet.stellar.org:443"
  SOROBAN_NETWORK_PASSPHRASE="Test SDF Future Network ; October 2022"
  FRIENDBOT_URL="https://friendbot-futurenet.stellar.org"
elif [[ $NETWORK == "futurenet" ]]; then
  echo "Using Futurenet network"
  SOROBAN_RPC_URL="https://soroban-testnet.stellar.org:443"
  SOROBAN_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
  FRIENDBOT_URL="https://friendbot.stellar.org"
fi

echo Add the $NETWORK network to cli client 
  soroban network add \
  --global $NETWORK \
  --rpc-url "$SOROBAN_RPC_URL" \
  --network-passphrase "$SOROBAN_NETWORK_PASSPHRASE"

if !(soroban soroban keys address token-admin | grep admin 2>&1 >/dev/null); then
  echo Create the admin identity
  soroban keys generate --global token-admin --network $NETWORK
fi

ARGS="--network $NETWORK --source-account token-admin"
WASM_PATH="target/wasm32-unknown-unknown/release/"
TOKEN_PATH=$WASM_PATH"token.wasm"
LIQUIDITY_POOL_PATH=$WASM_PATH"liquidity_pool.wasm"

echo Build token-contract
soroban contract build --package token

echo Build liquidity-pool-contract
soroban contract build --package liquidity-pool

echo "Deploy the abundance token A contract"
TOKEN_A_ID="$(
  soroban contract deploy $ARGS \
    --wasm $TOKEN_PATH
)"
echo "Contract deployed succesfully with ID: $TOKEN_A_ID"

echo "Deploy the abundance token B contract"
TOKEN_B_ID="$(
  soroban contract deploy $ARGS \
    --wasm $TOKEN_PATH
)"
echo "Contract deployed succesfully with ID: $TOKEN_B_ID"

echo Deploy the contract deployer
LIQUIDITY_POOL_ID="$(
  soroban contract deploy $ARGS \
    --wasm $LIQUIDITY_POOL_PATH
)"
echo "Liquidity Pool contract deployed succesfully with ID: $LIQUIDITY_POOL_ID"

if [[ $TOKEN_B_ID < $TOKEN_A_ID ]]; then
  echo "Changing tokens order"
  OLD_TOKEN_A_ID=$TOKEN_A_ID
  TOKEN_A_ID=$TOKEN_B_ID
  TOKEN_B_ID=$OLD_TOKEN_A_ID
fi

echo "Initialize the token A contract"
soroban contract invoke \
  $ARGS \
  --id $TOKEN_A_ID \
  -- \
  initialize \
  --symbol USDC \
  --decimal 7 \
  --name USDClear \
  --admin token-admin


echo "Initialize the token B contract"
soroban contract invoke \
  $ARGS \
  --id $TOKEN_B_ID \
  -- \
  initialize \
  --symbol BTC \
  --decimal 7 \
  --name BTClear \
  --admin token-admin

echo "Installing token wasm contract"
TOKEN_WASM_HASH="$(
  soroban contract install $ARGS \
    --wasm $TOKEN_PATH
)"

echo "Initialize the liquidity pool contract"
soroban contract invoke \
  $ARGS \
  --id $LIQUIDITY_POOL_ID \
  -- \
  initialize \
  --token_wasm_hash $TOKEN_WASM_HASH \
  --token_a $TOKEN_A_ID \
  --token_b $TOKEN_B_ID

echo "Getting the share id"
SHARE_ID="$(soroban contract invoke \
  $ARGS \
  --id $LIQUIDITY_POOL_ID \
  -- \
  share_id
)"
echo "Share ID: ${SHARE_ID//\"/}"