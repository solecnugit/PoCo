import dotenv from 'dotenv';
import path from 'path';
// import fs from 'fs';

// 加载.env文件
dotenv.config({ path: path.resolve(__dirname, '../../.env') });

// //定义配置文件类型
// interface ConfigFile {
//   nodeId?: string;
//   isLeader?: boolean;
//   port?: number;
//   peers?: Array<{nodeId: string; host: string; port: number}> | string[];
//   totalNodes?: number;
// }

// // 尝试从配置文件加载（如果指定了CONFIG_FILE环境变量）
// let fileConfig:ConfigFile  = {};
// const configFilePath = process.env.CONFIG_FILE;

// if (configFilePath) {
//   try {
//     const configData = fs.readFileSync(path.resolve(configFilePath), 'utf8');
//     fileConfig = JSON.parse(configData);
//     console.log(`成功从 ${configFilePath} 加载配置`);
//   } catch (error) {
//     console.warn(`无法从 ${configFilePath} 加载配置: ${error}`);
//   }
// }

export const config = {
  nodeId: process.env.NODE_ID || 'node_1',
  isLeader: process.env.IS_LEADER === 'true',
  port: parseInt(process.env.PORT || '3000', 10),
  peers: (process.env.PEERS || '').split(',').filter(Boolean),
  totalNodes: parseInt(process.env.TOTAL_NODES || '4', 10),
};

// 如果配置文件中提供了peers对象数组，转换为正确的字符串格式
// if (Array.isArray(fileConfig.peers) && fileConfig.peers.length > 0 && typeof fileConfig.peers[0] === 'object') {
//   // 使用类型断言确保TypeScript不会报错
//   const peerObjects = fileConfig.peers as Array<{nodeId: string; host: string; port: number}>;
//   config.peers = peerObjects.map(peer => 
//     `${peer.nodeId}:${peer.host}:${peer.port}`
//   );
// }

// 输出最终配置(仅在开发环境)
if (process.env.NODE_ENV !== 'production') {
  console.log('当前配置:', config);
}