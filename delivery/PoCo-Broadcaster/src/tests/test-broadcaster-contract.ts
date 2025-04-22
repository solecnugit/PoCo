// // test-broadcaster-contract.ts
// import { connect, keyStores, KeyPair, Contract, Account } from 'near-api-js';
// import * as fs from 'fs';
// import * as path from 'path';
// import * as os from 'os';

// // 基于Worker的配置创建Broadcaster配置
// const config = {
//   nearConfig: {
//     networkId: 'testnet',
//     nodeUrl: 'https://rpc.testnet.near.org',
//     walletUrl: 'https://wallet.testnet.near.org',
//     helperUrl: 'https://helper.testnet.near.org',
//     explorerUrl: 'https://explorer.testnet.near.org',
//   },
//   contractId: 'pococontract0.testnet', // 你的合约ID
//   broadcasterAccountId: 'pocobroadcaster1.testnet', // 你的广播者账号ID
//   credentialsPath: '~/.near-credentials', // NEAR凭证路径
// };

// /**
//  * 获取密钥存储
//  */
// async function getKeyStore(): Promise<keyStores.KeyStore> {
//   // 创建内存密钥存储
//   const keyStore = new keyStores.InMemoryKeyStore();

//   try {
//     // 从文件读取凭证
//     let credentialsPath = config.credentialsPath || path.join(os.homedir(), '.near-credentials');

//     // 如果路径以~开头，替换为用户主目录
//     if (credentialsPath.startsWith('~')) {
//       credentialsPath = credentialsPath.replace('~', os.homedir());
//     }

//     const networkPath = path.join(credentialsPath, config.nearConfig.networkId);

//     console.log(`尝试加载NEAR凭证目录: ${networkPath}`);

//     if (!fs.existsSync(networkPath)) {
//       throw new Error(`未找到NEAR凭证目录: ${networkPath}`);
//     }

//     const credentialsFilePath = path.join(networkPath, `${config.broadcasterAccountId}.json`);
//     console.log(`尝试加载账户凭证文件: ${credentialsFilePath}`);

//     if (!fs.existsSync(credentialsFilePath)) {
//       throw new Error(`未找到账户凭证文件: ${credentialsFilePath}`);
//     }

//     const credentials = JSON.parse(fs.readFileSync(credentialsFilePath, 'utf-8'));
//     const keyPair = KeyPair.fromString(credentials.private_key);

//     // 将密钥添加到存储
//     await keyStore.setKey(config.nearConfig.networkId, config.broadcasterAccountId, keyPair);
//     console.log(`已加载账户凭证: ${config.broadcasterAccountId}`);

//     return keyStore;
//   } catch (error) {
//     console.error('加载密钥失败:', error);
//     throw error;
//   }
// }

// async function testBroadcasterContract() {
//   console.log('开始测试Broadcaster合约调用...');

//   try {
//     // 配置密钥存储
//     const keyStore = await getKeyStore();

//     // 连接到NEAR
//     const nearConnection = await connect({
//       ...config.nearConfig,
//       keyStore,
//     });

//     // 获取账户对象
//     const account = await nearConnection.account(config.broadcasterAccountId);
//     console.log(`已连接账户: ${config.broadcasterAccountId}`);

//     // 初始化合约接口
//     const contract = new Contract(
//       account,
//       config.contractId,
//       {
//         // 视图方法 - 不需要签名
//         viewMethods: [
//           'get_available_tasks',
//           'get_task',
//           'get_consensus_proof',
//         ],
//         // 修改方法 - 需要签名
//         changeMethods: [
//           'publish_task',
//           'assign_task',
//           'select_verifiers',
//         ],
//         useLocalViewExecution: false
//       }
//     );

//     console.log('合约已初始化');

//     // 测试获取可用任务
//     console.log('获取可用任务...');
//     const tasks = await contract.get_available_tasks({ from_index: 0, limit: 10 });
//     console.log(`找到 ${tasks.length} 个任务`);

//     // 测试发布任务
//     console.log('尝试发布测试任务...');
//     // 发布一个测试任务，使用模拟的IPFS哈希
//     const testIpfsHash = 'QmTest' + Date.now().toString();
//     const requirements = {
//       target_codec: 'h264',
//       target_resolution: '1280x720',
//       target_bitrate: '1000k',
//       target_framerate: '30',
//       additional_params: ''
//     };

//     // 获取合约的完整接口和方法列表
//     console.log('合约对象:', Object.keys(contract));
//     console.log('publish_task方法:', contract.publish_task ? '存在' : '不存在');

//     try {
//       // 尝试直接调用合约方法
//       const taskId = await contract.publish_task({
//         source_ipfs: testIpfsHash,
//         requirements: requirements
//       });

//       console.log(`任务发布成功! 任务ID: ${taskId}`);
//     } catch (pubError) {
//       console.error('发布任务失败 (方法1):', pubError);

//       // 如果直接调用失败，尝试使用functionCall
//       try {
//         console.log('尝试使用account.functionCall...');
//         const result = await account.functionCall({
//           contractId: config.contractId,
//           methodName: 'publish_task',
//           args: {
//             source_ipfs: testIpfsHash,
//             requirements: requirements
//           },
//           gas: '300000000000000' // 300 TGas
//         });

//         // 尝试解析结果
//         const resultValue = result.status.hasOwnProperty('SuccessValue')
//           ? Buffer.from(result.status.SuccessValue, 'base64').toString()
//           : '无返回值';

//         console.log(`任务发布成功 (方法2)! 结果: ${resultValue}`);
//       } catch (funcError) {
//         console.error('发布任务失败 (方法2):', funcError);
//       }
//     }

//   } catch (error) {
//     console.error('测试失败:', error);
//   }
// }

// // 运行测试
// testBroadcasterContract().catch(console.error);
