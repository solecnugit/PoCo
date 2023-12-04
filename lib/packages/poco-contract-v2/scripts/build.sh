#!/bin/sh

echo ">> Building contract"

cd $WORKSPACE_DIR/contract

rustup target add wasm32-unknown-unknown

cargo install cargo-near --vers 0.4.1

# cargo build --all --target wasm32-unknown-unknown --release

# cargo near build --release
cargo near build

cp $WORKSPACE_DIR/contract/target/near/poco_abi.json $WORKSPACE_DIR/../poco-agent/abi.json