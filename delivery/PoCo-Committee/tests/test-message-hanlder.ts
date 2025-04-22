// test-message-handler.ts
import { CommitteeNode } from '../src/core/CommitteeNode';
import { ApiServer } from '../src/network/ApiServer';
import { logger } from '../src/utils/logger';
import { setTimeout as sleep } from 'timers/promises';

// 测试配置
const TEST_CONFIG = {
  testDuration: 10 * 60 * 1000, // 10分钟测试
  networkDisruptionInterval: 2 * 60 * 1000, // 每2分钟模拟一次网络中断
  disruptionDuration: 30 * 1000, // 中断持续30秒
};

// 模拟节点配置
const nodes = [
  { nodeId: 'leader', port: 8000, isLeader: true, 
    peers: ['follower1:localhost:8001', 'follower2:localhost:8002', 'follower3:localhost:8003'] },
  { nodeId: 'follower1', port: 8001, isLeader: false, 
    peers: ['leader:localhost:8000', 'follower2:localhost:8002', 'follower3:localhost:8003'] },
  { nodeId: 'follower2', port: 8002, isLeader: false, 
    peers: ['leader:localhost:8000', 'follower1:localhost:8001', 'follower3:localhost:8003'] },
  { nodeId: 'follower3', port: 8003, isLeader: false, 
    peers: ['leader:localhost:8000', 'follower1:localhost:8001', 'follower2:localhost:8002'] },
];

// 保存创建的委员会节点和API服务器
const committeeNodes: CommitteeNode[] = [];
const apiServers: ApiServer[] = [];

// 启动所有节点
async function startAllNodes() {
  for (const node of nodes) {
    const committeeNode = new CommitteeNode(
      node.nodeId,
      node.port,
      node.isLeader,
      node.peers,
      nodes.length
    );
    
    const apiServer = new ApiServer(node.port + 1000, committeeNode);
    
    committeeNode.start();
    await apiServer.start();
    
    committeeNodes.push(committeeNode);
    apiServers.push(apiServer);
    
    logger.info(`启动节点 ${node.nodeId} 在端口 ${node.port} (API端口: ${node.port + 1000})`);
  }
  
  logger.info('所有节点已启动');
}

// 停止所有节点
async function stopAllNodes() {
  for (let i = 0; i < committeeNodes.length; i++) {
    const nodeId = nodes[i].nodeId;
    logger.info(`停止节点 ${nodeId}...`);
    
    await apiServers[i].stop();
    committeeNodes[i].stop();
    
    logger.info(`节点 ${nodeId} 已停止`);
  }
}

// 模拟特定节点的网络中断
async function disruptNode(index: number, duration: number) {
  const nodeId = nodes[index].nodeId;
  logger.warn(`⚠️ 模拟节点 ${nodeId} 的网络中断 ${duration/1000} 秒`);
  
  // 停止节点
  await apiServers[index].stop();
  committeeNodes[index].stop();
  
  // 等待指定的中断持续时间
  await sleep(duration);
  
  // 重新启动节点
  committeeNodes[index] = new CommitteeNode(
    nodes[index].nodeId,
    nodes[index].port,
    nodes[index].isLeader,
    nodes[index].peers,
    nodes.length
  );
  
  apiServers[index] = new ApiServer(nodes[index].port + 1000, committeeNodes[index]);
  
  committeeNodes[index].start();
  await apiServers[index].start();
  
  logger.info(`✅ 节点 ${nodeId} 已恢复连接`);
}

// 运行测试
async function runTest() {
  logger.info('=== 开始MessageHandler集成测试 ===');
  
  try {
    // 启动所有节点
    await startAllNodes();
    
    // 等待节点之间建立连接
    logger.info('等待30秒让节点连接稳定...');
    await sleep(30 * 1000);
    
    // 定期模拟网络中断
    const startTime = Date.now();
    let testIteration = 1;
    
    while (Date.now() - startTime < TEST_CONFIG.testDuration) {
      logger.info(`\n--- 测试迭代 #${testIteration} ---`);
      
      // 随机选择一个节点进行中断
      const nodeIndex = Math.floor(Math.random() * nodes.length);
      
      // 模拟网络中断
      await disruptNode(nodeIndex, TEST_CONFIG.disruptionDuration);
      
      // 等待恢复和稳定
      logger.info('等待系统稳定...');
      await sleep(60 * 1000);
      
      // 输出节点状态
      logger.info('当前节点连接状态:');
      for (let i = 0; i < committeeNodes.length; i++) {
        const status = committeeNodes[i].getStatus();
        logger.info(`节点 ${nodes[i].nodeId}: ${JSON.stringify(status.connections)}`);
      }
      
      testIteration++;
      
      // 如果已经到达测试结束时间，就跳出循环
      if (Date.now() - startTime >= TEST_CONFIG.testDuration) {
        break;
      }
      
      // 等待到下一次网络中断
      const waitTime = Math.min(
        TEST_CONFIG.networkDisruptionInterval,
        (startTime + TEST_CONFIG.testDuration) - Date.now()
      );
      
      if (waitTime > 0) {
        logger.info(`等待 ${waitTime/1000} 秒到下一次网络中断测试...`);
        await sleep(waitTime);
      }
    }
    
    logger.info('=== 测试完成 ===');
    logger.info('总测试时间: ' + (Date.now() - startTime)/1000 + ' 秒');
    
  } catch (error) {
    logger.error('测试过程中发生错误:', error);
  } finally {
    // 清理资源
    await stopAllNodes();
  }
}

// 运行测试并处理Ctrl+C
runTest().catch(error => {
  logger.error('未捕获的测试错误:', error);
}).finally(() => {
  logger.info('测试结束，清理资源...');
  stopAllNodes().then(() => {
    process.exit(0);
  });
});

// 处理进程信号
process.on('SIGINT', async () => {
  logger.info('收到中断信号，停止测试...');
  await stopAllNodes();
  process.exit(0);
});