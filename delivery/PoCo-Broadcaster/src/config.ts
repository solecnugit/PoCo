// src/config.ts
import { BroadcasterConfig } from "./types";

const config: BroadcasterConfig = {
  networkId: "testnet",
  nodeUrl: "https://rpc.testnet.near.org",
  walletUrl: "https://wallet.testnet.near.org",
  helperUrl: "https://helper.testnet.near.org",
  explorerUrl: "https://explorer.testnet.near.org",
  contractId: "pococontract11.testnet",
  ipfsConfig: {
    host: "localhost",
    port: 5001,
    protocol: "http",
    gateway: "http://106.75.224.49:8080",
  },
};

export default config;
