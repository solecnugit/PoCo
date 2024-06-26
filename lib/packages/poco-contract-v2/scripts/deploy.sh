#!/bin/bash

$WORKSPACE_DIR/scripts/build.sh

if [ $? -ne 0 ]; then
  echo ">> Error building contract"
  exit 1
fi

echo ">> Deploying contract"

if [ "$NEAR_ENV" = "local" ]; then
  NEAR_MODE="env" source $WORKSPACE_DIR/scripts/near-env.sh
  echo ">> Use local near blockchain"
  NEAR_DEPLOY_COMMAND="pnpm exec near dev-deploy"
elif [ "$NEAR_ENV" = "testnet" ]; then
  echo ">> Use near testnet"
  # NEAR_DEPLOY_COMMAND="pnpm exec near deploy dev-1699848063836-18477754701708 --helperUrl https://near-contract-helper.onrender.com"
  NEAR_DEPLOY_COMMAND="pnpm exec near deploy poqos.testnet contract/target/wasm32-unknown-unknown/release/poco.wasm"
else
  echo "Unsupport NEAR_ENV : ${NEAR_ENV}"
  exit 1
fi


# https://docs.near.org/tools/near-cli#near-dev-deploy
OUTPUT=$(echo "y" | $NEAR_DEPLOY_COMMAND --wasmFile ${WORKSPACE_DIR}/contract/target/near/poco.wasm --accountId ${NEAR_HELPER_ACCOUNT})
CONTRACT_ID=$(echo $OUTPUT | awk '{ print $NF }')

echo $OUTPUT

echo "Contract Id: ${CONTRACT_ID}"
echo ">> Deployed."