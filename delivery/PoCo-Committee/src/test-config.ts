// src/test-config.ts
import { config } from './config';

console.log('===== 配置测试开始 =====');
console.log('节点ID:', config.nodeId);
console.log('是否是Leader:', config.isLeader ? '是' : '否');
console.log('端口:', config.port);
console.log('总节点数:', config.totalNodes);
console.log('对等节点列表:');
if (config.peers.length === 0) {
  console.log('- 无对等节点');
} else {
  for (const peer of config.peers) {
    console.log(`- ${peer}`);
  }
}
console.log('===== 配置测试结束 =====');