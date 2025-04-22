// test-committee-client.ts
import { CommitteeClient } from "../committee-client";
import config from "../config";
import { VerifierQosProof, VideoSpecification, GopScore } from "../types";
import { generatePlaceholderSignature, getCurrentTimestamp } from "../utils";

/**
 * 测试委员会通信模块
 */
async function testCommitteeClient() {
  console.log("开始测试委员会通信模块...");

  // 初始化委员会客户端
  const committeeClient = new CommitteeClient(config);
  const initialized = await committeeClient.initialize();

  if (!initialized) {
    console.error("委员会客户端初始化失败");
    return;
  }

  console.log("委员会客户端初始化成功");

  // 创建测试验证证明
  const testTaskId = "test-task-" + Date.now().toString();
  const testVerifierId = config.verifierAccountId;

  // 创建测试视频规格
  const videoSpecs: VideoSpecification = {
    codec: "h264",
    resolution: "1920x1080",
    bitrate: 5000000,
    framerate: 30,
  };

  // 创建测试GOP分数
  const gopScores: GopScore[] = [
    {
      timestamp: "00:00:10.000",
      vmafScore: 95.8,
      hash: "0x" + Math.random().toString(16).substring(2),
    },
    {
      timestamp: "00:00:30.000",
      vmafScore: 94.2,
      hash: "0x" + Math.random().toString(16).substring(2),
    },
    {
      timestamp: "00:01:00.000",
      vmafScore: 93.5,
      hash: "0x" + Math.random().toString(16).substring(2),
    },
  ];

  // 创建测试验证证明
  const testProof: VerifierQosProof = {
    id: `proof-${testTaskId}-${testVerifierId}`,
    taskId: testTaskId,
    verifierId: testVerifierId,
    timestamp: getCurrentTimestamp(),
    videoSpecs: videoSpecs,
    videoScore: 94.5, // 平均分数
    gopScores: gopScores,
    audioScore: 4.3,
    syncScore: 12.0,
    signature: generatePlaceholderSignature(),
  };

  // 测试发送验证结果
  try {
    console.log(`测试发送验证结果到委员会，任务ID: ${testTaskId}`);
    const sendResult = await committeeClient.sendProofToCommittee(testProof);

    if (sendResult) {
      console.log("验证结果发送成功");
    } else {
      console.error("验证结果发送失败");
    }
  } catch (error) {
    console.error("发送验证结果时出错:", error);
  }

  // 等待一段时间让委员会处理
  console.log("等待3秒让委员会处理验证结果...");
  // await new Promise(resolve => setTimeout(resolve, 3000));

  // 测试发送补充验证结果
  try {
    console.log(`测试发送补充验证结果到委员会，任务ID: ${testTaskId}`);

    // 修改一下原始验证证明作为补充验证结果
    const supplementaryProof = { ...testProof };
    supplementaryProof.videoScore = 95.0; // 略微调整分数
    supplementaryProof.timestamp = getCurrentTimestamp();
    supplementaryProof.signature = generatePlaceholderSignature();

    const supplementaryResult = await committeeClient.sendSupplementaryProof(
      testTaskId,
      supplementaryProof
    );

    if (supplementaryResult) {
      console.log("补充验证结果发送成功");
    } else {
      console.error("补充验证结果发送失败");
    }
  } catch (error) {
    console.error("发送补充验证结果时出错:", error);
  }

  // 等待一段时间让委员会处理
  console.log("等待3秒让委员会处理验证结果...");
  // await new Promise(resolve => setTimeout(resolve, 3000));

  // // 测试查询任务状态
  try {
    console.log(`测试查询任务状态，任务ID: ${testTaskId}`);
    const taskStatus = await committeeClient.getTaskStatus(testTaskId);

    if (taskStatus) {
      console.log("任务状态查询成功:");
      console.log(JSON.stringify(taskStatus, null, 2));
    } else {
      console.log("任务状态查询失败或任务不存在");
    }
  } catch (error) {
    console.error("查询任务状态时出错:", error);
  }

  // 测试批量发送验证结果
  // try {
  //   // 创建多个测试验证证明
  //   const batchProofs: VerifierQosProof[] = [];
  //   for (let i = 0; i < 3; i++) {
  //     const batchTaskId = `test-batch-${Date.now()}-${i}`;

  //     batchProofs.push({
  //       id: `proof-${batchTaskId}-${testVerifierId}`,
  //       taskId: batchTaskId,
  //       verifierId: testVerifierId,
  //       timestamp: getCurrentTimestamp(),
  //       videoSpecs: videoSpecs,
  //       videoScore: 90 + Math.random() * 10, // 90-100之间的随机分数
  //       gopScores: gopScores,
  //       audioScore: 4.2,
  //       syncScore: 14.0,
  //       signature: generatePlaceholderSignature({
  //         taskId: batchTaskId,
  //         verifierId: testVerifierId,
  //         timestamp: getCurrentTimestamp()
  //       })
  //     });
  //   }

  //   console.log(`测试批量发送验证结果到委员会，共 ${batchProofs.length} 个任务`);
  //   const batchResult = await committeeClient.sendProofsBatch(batchProofs);

  //   if (batchResult) {
  //     console.log('批量验证结果发送成功');
  //   } else {
  //     console.error('批量验证结果发送失败');
  //   }
  // } catch (error) {
  //   console.error('批量发送验证结果时出错:', error);
  // }

  // // 测试发送补充验证结果
  // try {
  //   console.log(`测试发送补充验证结果到委员会，任务ID: ${testTaskId}`);

  //   // 修改一下原始验证证明作为补充验证结果
  //   const supplementaryProof = { ...testProof };
  //   supplementaryProof.videoScore = 95.0; // 略微调整分数
  //   supplementaryProof.timestamp = getCurrentTimestamp();
  //   supplementaryProof.signature = generatePlaceholderSignature({
  //     taskId: testTaskId,
  //     verifierId: testVerifierId,
  //     timestamp: supplementaryProof.timestamp
  //   });

  //   const supplementaryResult = await committeeClient.sendSupplementaryProof(testTaskId, supplementaryProof);

  //   if (supplementaryResult) {
  //     console.log('补充验证结果发送成功');
  //   } else {
  //     console.error('补充验证结果发送失败');
  //   }
  // } catch (error) {
  //   console.error('发送补充验证结果时出错:', error);
  // }

  console.log("委员会通信模块测试完成");
}

// 运行测试
testCommitteeClient().catch(console.error);
