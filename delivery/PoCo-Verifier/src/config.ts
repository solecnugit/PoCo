// config.ts
import { VerifierConfig } from "./types";

const config: VerifierConfig = {
  nearConfig: {
    networkId: "testnet", // 或者 'mainnet', 取决于您部署的网络
    nodeUrl: "https://rpc.testnet.near.org",
    walletUrl: "https://wallet.testnet.near.org",
    helperUrl: "https://helper.testnet.near.org",
    explorerUrl: "https://explorer.testnet.near.org",
  },
  contractId: "pococontract11.testnet", // 转码服务合约ID
  verifierAccountId: "verifier1.testnet", // 验证者账户ID
  credentialsPath: "~/.near-credentials", // NEAR凭证路径
  ipfsConfig: {
    host: "10.24.197.106",
    port: 5001,
    protocol: "http",
  },
  committeeLeaderUrl: "http://localhost:3000", // 委员会Leader API地址
  pollingInterval: 30000, // 轮询间隔30秒
  ffmpegPath: "ffmpeg", // FFmpeg可执行文件路径
  vmafModelPath: "vmaf_v0.6.1.json", // VMAF模型路径
  logLevel: "info", // 日志级别
};

export default config;
