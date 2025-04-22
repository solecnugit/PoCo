// start-node.ts
import { config } from './config';
import { CommitteeNode } from './core/CommitteeNode';
import { ApiServer } from './network/ApiServer';
import { logger } from './utils/logger';

async function startNode() {
  try {
    logger.info('====================================');
    logger.info(`正在启动节点 ${config.nodeId}`);
    logger.info(`角色: ${config.isLeader ? 'Leader' : 'Follower'}`);
    logger.info(`端口: ${config.port}`);
    logger.info(`API端口: ${config.port + 1000}`); // 假设API端口是节点端口+1000
    logger.info(`Peers: ${config.peers.join(', ')}`);
    logger.info(`总节点数: ${config.totalNodes}`);
    logger.info('====================================');
    
    // 创建并启动Committee节点
    const committeeNode = new CommitteeNode(
      config.nodeId,
      config.port,
      config.isLeader,
      config.peers,
      config.totalNodes
    );
    
    // 创建并启动API服务器
    const apiServer = new ApiServer(config.port + 1000, committeeNode);
    
    // 启动服务
    committeeNode.start();
    await apiServer.start();
    
    logger.info(`节点 ${config.nodeId} 启动完成!`);
    
    // 处理进程退出
    process.on('SIGINT', async () => {
      logger.info('收到终止信号，正在关闭节点...');
      await apiServer.stop();
      committeeNode.stop();
      process.exit(0);
    });
    
    process.on('SIGTERM', async () => {
      logger.info('收到终止信号，正在关闭节点...');
      await apiServer.stop();
      committeeNode.stop();
      process.exit(0);
    });
    
    // 捕获未处理的异常
    process.on('uncaughtException', (error) => {
      logger.error(`未捕获的异常: ${error.message}`);
      logger.error(error.stack);
    });
    
    process.on('unhandledRejection', (reason, promise) => {
      logger.error(`未处理的Promise拒绝: ${reason}`);
    });
  } catch (error) {
    logger.error(`启动节点失败: ${error}`);
    process.exit(1);
  }
}

startNode();