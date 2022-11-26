#!/bin/sh

echo ">> Building contract"

cd $WORKSPACE_DIR/contract

rustup target add wasm32-unknown-unknown
cargo build --all --target wasm32-unknown-unknown --release
