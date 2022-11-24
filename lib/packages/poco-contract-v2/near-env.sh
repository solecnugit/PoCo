if [ -z $NEAR_NETWORK_IP ]; then
    NEAR_NETWORK_IP="49.52.27.50"
fi

echo "NEAR NETWORK IP: ${NEAR_NETWORK_IP}"
echo "WORKSPACE DIRECTORY: ${WORKSPACE_DIR}"

if [ -z $NEAR_MODE ]; then
    NEAR_MODE="unset"
fi

if [ $NEAR_MODE = "env" ]; then
    echo "Setup enviroment variables"
    export NEAR_ENV="local"
    export NEAR_CLI_LOCALNET_NETWORK_ID="localnet"
    export NEAR_NODE_URL="http://${NEAR_NETWORK_IP}:8332"
    export NEAR_CLI_LOCALNET_KEY_PATH="${WORKSPACE_DIR}/validator-key.json"
    export NEAR_WALLET_URL="http://${NEAR_NETWORK_IP}:8334"
    export NEAR_HELPER_URL="http://${NEAR_NETWORK_IP}:8330"
    export NEAR_HELPER_ACCOUNT="test.near"
    export NEAR_EXPLORER_URL="http://${NEAR_NETWORK_IP}:8331"
elif [ $NEAR_MODE = "command" ]; then
    echo "Setup alias command"
    export NEAR_DEVELOPMENT_COMMAND="NEAR_ENV=\"local\" NEAR_CLI_LOCALNET_NETWORK_ID=\"localnet\" NEAR_NODE_URL=\"http://${NEAR_NETWORK_IP}:8332\" NEAR_CLI_LOCALNET_KEY_PATH=\"${WORKSPACE_DIR}/validator-key.json\" NEAR_WALLET_URL=\"http://${NEAR_NETWORK_IP}:8334\" NEAR_HELPER_URL=\"http:/${NEAR_NETWORK_IP}:8330\" NEAR_HELPER_ACCOUNT=\"test.near\" NEAR_EXPLORER_URL=\"http://${NEAR_NETWORK_IP}:8331\" pnpm exec near"
    
    echo "NEAR DEVELOPMENT COMMAND: ${NEAR_DEVELOPMENT_COMMAND}"

    alias neardev=$NEAR_DEVELOPMENT_COMMAND
elif [ $NEAR_MODE = "unset" ]; then
    echo "Unset enviroment variables"
    unset NEAR_ENV
    unset NEAR_CLI_LOCALNET_NETWORK_ID
    unset NEAR_NODE_URL
    unset NEAR_CLI_LOCALNET_KEY_PATH
    unset NEAR_WALLET_URL
    unset NEAR_HELPER_URL
    unset NEAR_HELPER_ACCOUNT
    unset NEAR_EXPLORER_URL
    unset NEAR_DEVELOPMENT_COMMAND
    unalias neardev
fi