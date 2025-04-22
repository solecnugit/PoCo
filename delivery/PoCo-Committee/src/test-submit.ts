import { NearConnectionLeader } from './near-connection-leader';
import { ConsensusQosProof, GopVerificationResult, QosProofStatus } from './models/types';
const config = require('./committee-config').default;

// 配置对象
// const config = {
//   leaderAccountId: 'pocoleader.testnet',
//   contractId: 'pococontract11.testnet',
//   proxyUrl: 'http://127.0.0.1:7890', // 根据您的实际代理地址修改
//   nearConfig: {
//     networkId: 'testnet',
//     nodeUrl: 'https://rpc.testnet.near.org',
//     walletUrl: 'https://wallet.testnet.near.org',
//     explorerUrl: 'https://explorer.testnet.near.org',
//   },
//   credentialsPath: '~/.near-credentials',
// };

async function testSubmitConsensusProof() {
  // 初始化NEAR连接
  console.log('正在初始化NEAR连接...');
  const nearConnection = new NearConnectionLeader(config);
  await nearConnection.initialize();

  // 测试用例：您提供的命令行调用示例（能成功运行的）
  const successProof: ConsensusQosProof = {
    task_id: 'task-pocobroadcaster1.testnet-193543445',
    worker_id: 'pocoworker1.testnet',
    committee_leader: 'committee_leader.testnet',
    timestamp: 1745160042837,
    video_score: 85.591024,
    audio_score: 0.0,
    sync_score: 0.0,
    encoding_start_time: 1745159988448,
    encoding_end_time: 1745160014157,
    video_specs: {
      codec: 'h264',
      resolution: '640x360',
      bitrate: 912360,
      framerate: 30.0,
    },
    frame_count: 0,
    specified_gop_scores: [
      {
        timestamp: '0',
        vmaf_score: 0.0,
        hash: '',
      },
    ],
    gop_verification: GopVerificationResult.Verified,
    status: QosProofStatus.Normal,
  };

  console.log('提交共识证明...');
  console.log('提交数据:', JSON.stringify(successProof, null, 2));

  try {
    const result = await nearConnection.submitConsensusProof(successProof);
    console.log('提交结果:', result);
  } catch (error) {
    console.error('提交失败:', error);
  }
}

// 运行测试
testSubmitConsensusProof()
  .then(() => console.log('测试完成'))
  .catch(err => console.error('测试发生错误:', err));
