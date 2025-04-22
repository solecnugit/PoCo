// committee-config.ts
import { CommitteeConfig } from './models/types';

const config: CommitteeConfig = {
  nearConfig: {
    networkId: 'testnet', // 或者 'mainnet', 取决于您部署的网络
    nodeUrl: 'https://rpc.testnet.near.org',
    walletUrl: 'https://wallet.testnet.near.org',
    helperUrl: 'https://helper.testnet.near.org',
    explorerUrl: 'https://explorer.testnet.near.org',
  },
  contractId: 'pococontract11.testnet', // 转码服务合约ID
  leaderAccountId: 'pocoleader.testnet', // 委员会Leader账户ID
  credentialsPath: '~/.near-credentials', // NEAR凭证路径
  ipfsConfig: {
    host: '10.24.197.106',
    port: 5001,
    protocol: 'http',
  },
  proxyUrl: 'http://127.0.0.1:10809', // 代理服务器URL，用于访问NEAR网络
  pollingInterval: 30000, // 轮询间隔30秒
  consensusTimeout: 120000, // 共识超时时间(毫秒)
  supplementaryTimeout: 7200000, // 补充验证超时时间(2小时，毫秒)
  logLevel: 'info', // 日志级别
};

export default config;
