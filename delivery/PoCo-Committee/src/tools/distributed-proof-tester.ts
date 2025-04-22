// src/tools/distributed-proof-tester.ts

import axios from 'axios';
import { generateTestProof, generateConflictingProof } from '../utils/proof-utils';
import { QoSProof } from '../models/types';
import { program } from 'commander';
import * as fs from 'fs';
import * as path from 'path';

// 节点配置类型
interface NodeConfig {
  nodeId: string;
  host: string;
  apiPort: number;
}

// 测试场景类型
type TestScenario = 'normal' | 'conflict' | 'supplementary';

// 默认配置
const DEFAULT_CONFIG_PATH = path.join(__dirname, '../../network-config.json');

// 读取节点配置
function loadNodeConfig(configPath: string): NodeConfig[] {
  try {
    const configData = fs.readFileSync(configPath, 'utf8');
    const config = JSON.parse(configData);
    
    if (!Array.isArray(config.nodes)) {
      throw new Error('无效的配置文件格式：缺少nodes数组');
    }
    
    return config.nodes;
  } catch (error: unknown) {
    const errorMessage = error instanceof Error ? error.message: String(error);
    console.error(`读取配置文件失败: ${errorMessage}`);
    process.exit(1);
  }
}

// 生成任务ID（如果未提供）
function generateTaskId(): string {
  return `task-${Date.now()}`;
}

// 提交QoS证明到单个节点
async function submitProofToNode(
  node: NodeConfig, 
  proof: QoSProof, 
  isSupplementary: boolean = false
): Promise<any> {
  try {
    const url = isSupplementary
      ? `http://${node.host}:${node.apiPort}/proof/${proof.taskId}/supplementary`
      : `http://${node.host}:${node.apiPort}/proof`;
    
    console.log(`提交${isSupplementary ? '补充' : ''}证明到节点 ${node.nodeId} (${node.host}:${node.apiPort})...`);
    
    const response = await axios.post(url, proof, {
      headers: { 'Content-Type': 'application/json' },
      timeout: 20000  // 10秒超时
    });
    
    console.log(`  - 状态码: ${response.status}`);
    console.log(`  - 响应: ${JSON.stringify(response.data)}`);
    return response.data;
  } catch (error: unknown) {
    // 处理Axios错误
    if (axios.isAxiosError(error) && error.message) {
      console.error(`  - 错误: ${error.message}`);
      
      // 检查是否有响应数据
      if (error.response) {
        console.error(`  - 状态码: ${error.response.status}`);
        console.error(`  - 响应数据: ${JSON.stringify(error.response.data)}`);
      }
    } else {
      // 处理非Axios错误
      console.error(`  - 错误: ${error instanceof Error ? error.message : String(error)}`);
    }
    return null;
  }
}

// 向所有节点提交证明
async function submitProofToAllNodes(
  nodes: NodeConfig[], 
  proof: QoSProof, 
  isSupplementary: boolean = false
): Promise<void> {
  console.log(`\n开始向所有节点提交${isSupplementary ? '补充' : ''}证明...`);
  console.log(`证明详情: 任务ID=${proof.taskId}, 验证者=${proof.verifierId}`);
  
  const results = await Promise.all(nodes.map(node => submitProofToNode(node, proof, isSupplementary)));
  
  const successCount = results.filter(r => r !== null).length;
  console.log(`\n提交完成: ${successCount}/${nodes.length} 个节点成功接收证明`);
}

// 检查任务状态
async function checkTaskStatus(nodes: NodeConfig[], taskId: string): Promise<void> {
  console.log(`\n检查任务 ${taskId} 在各节点上的状态...`);
  
  for (const node of nodes) {
    try {
      const url = `http://${node.host}:${node.apiPort}/proof/${taskId}/status`;
      const response = await axios.get(url, { timeout: 5000 });
      
      console.log(`\n节点 ${node.nodeId} (${node.host}:${node.apiPort}) 的任务状态:`);
      console.log(`  - 状态: ${response.data.state}`);
      console.log(`  - 证明数量: ${response.data.proofCount}`);
      console.log(`  - 验证者IDs: ${response.data.verifierIds?.join(', ') || 'N/A'}`);
      
      if (response.data.conflictInfo) {
        console.log(`  - 冲突信息: ${JSON.stringify(response.data.conflictInfo)}`);
      }
      
      if (response.data.result) {
        console.log(`  - 结果: ${JSON.stringify(response.data.result)}`);
      }
    } catch (error: unknown) {
        const errorMessage = error instanceof Error ? error.message: String(error);
      console.error(`  - 节点 ${node.nodeId} 状态检查失败: ${errorMessage}`);
    }
  }
}

// 运行测试场景
async function runTestScenario(
  nodes: NodeConfig[], 
  scenario: TestScenario, 
  taskId: string,
  checkInterval: number = 5000,
  maxChecks: number = 12
): Promise<void> {
  console.log(`\n运行测试场景: ${scenario}`);
  console.log(`任务ID: ${taskId}`);
  
  // 生成证明
  const proof1 = generateTestProof(taskId, 'verifier1');
  
  // 根据场景生成第二个证明
  let proof2: QoSProof;
  if (scenario === 'conflict') {
    proof2 = generateConflictingProof(taskId, 'verifier2', 'codec');
  } else {
    proof2 = generateTestProof(taskId, 'verifier2');
  }
  
  // 提交第一个证明到所有节点
  await submitProofToAllNodes(nodes, proof1);
  
  // 等待片刻
  await new Promise(resolve => setTimeout(resolve, 1000));
  
  // 提交第二个证明到所有节点
  await submitProofToAllNodes(nodes, proof2);
  
  // 如果是补充验证场景，等待冲突检测并提交补充证明
  if (scenario === 'supplementary') {
    // 先运行冲突场景
    const conflictProof = generateConflictingProof(taskId, 'verifier2', 'codec');
    await submitProofToAllNodes(nodes, conflictProof);
    
    // 等待一段时间让冲突被检测到
    console.log('\n等待冲突被检测...');
    let conflictDetected = false;
    
    for (let i = 0; i < 5; i++) {
      await new Promise(resolve => setTimeout(resolve, 3000));
      
      // 检查状态
      for (const node of nodes) {
        try {
          const url = `http://${node.host}:${node.apiPort}/proof/${taskId}/status`;
          const response = await axios.get(url, { timeout: 5000 });
          
          if (response.data.state === 'conflict_detected' || 
              response.data.state === 'awaiting_supplementary_verification') {
            conflictDetected = true;
            console.log(`\n冲突已在节点 ${node.nodeId} 上检测到，状态为 ${response.data.state}`);
            break;
          }
        } catch (error) {
          // 忽略错误，继续检查其他节点
        }
      }
      
      if (conflictDetected) break;
    }
    
    if (!conflictDetected) {
      console.log('\n警告: 未检测到冲突，但仍将提交补充证明');
    }
    
    // 提交补充证明
    const supplementaryProof = generateTestProof(taskId, 'verifier3');
    supplementaryProof.mediaSpecs.codec = proof1.mediaSpecs.codec; // 使用与第一个证明相同的编码
    
    await submitProofToAllNodes(nodes, supplementaryProof, true);
  }
  
  // 定期检查任务状态
  console.log(`\n将每 ${checkInterval/1000} 秒检查一次任务状态，共检查 ${maxChecks} 次`);
  
  for (let i = 0; i < maxChecks; i++) {
    console.log(`\n状态检查 #${i+1}/${maxChecks}`);
    await checkTaskStatus(nodes, taskId);
    
    // 检查是否所有节点都已完成共识
    try {
      let allFinalized = true;
      
      for (const node of nodes) {
        const url = `http://${node.host}:${node.apiPort}/proof/${taskId}/status`;
        const response = await axios.get(url, { timeout: 5000 });
        
        if (response.data.state !== 'finalized') {
          allFinalized = false;
          break;
        }
      }
      
      if (allFinalized) {
        console.log('\n所有节点已完成共识，测试成功！');
        break;
      }
    } catch (error) {
      // 忽略错误，继续检查
    }
    
    // 最后一次检查不需要等待
    if (i < maxChecks - 1) {
      console.log(`\n等待 ${checkInterval/1000} 秒后再次检查...`);
      await new Promise(resolve => setTimeout(resolve, checkInterval));
    }
  }
  
  console.log('\n测试场景运行完毕');
}

// 主函数
async function main() {
  program
    .description('分布式PBFT网络的QoS证明测试工具')
    .option('-c, --config <path>', '节点配置文件路径', DEFAULT_CONFIG_PATH)
    .option('-t, --task-id <id>', '自定义任务ID（如果未提供将自动生成）')
    .option('-s, --scenario <type>', '测试场景类型: normal, conflict, supplementary', 'normal')
    .option('-i, --interval <ms>', '状态检查间隔（毫秒）', '5000')
    .option('-m, --max-checks <number>', '最大状态检查次数', '12')
    .option('--check-only <taskId>', '仅检查指定任务ID的状态，不生成新证明')
    .parse(process.argv);

  const options = program.opts();
  
  // 加载节点配置
  const nodes = loadNodeConfig(options.config);
  console.log(`已加载 ${nodes.length} 个节点的配置`);
  
  // 如果只是检查任务状态
  if (options.checkOnly) {
    await checkTaskStatus(nodes, options.checkOnly);
    return;
  }
  
  // 确定任务ID
  const taskId = options.taskId || generateTaskId();
  
  // 检查场景类型
  const scenario = options.scenario;
  if (!['normal', 'conflict', 'supplementary'].includes(scenario)) {
    console.error(`错误: 无效的场景类型 "${scenario}"`);
    console.error('有效的场景类型: normal, conflict, supplementary');
    process.exit(1);
  }
  
  // 运行测试场景
  await runTestScenario(
    nodes, 
    scenario as TestScenario, 
    taskId,
    parseInt(options.interval),
    parseInt(options.maxChecks)
  );
}

// 处理未捕获的异常
process.on('uncaughtException', (error) => {
  console.error('未捕获的异常:', error);
  process.exit(1);
});

process.on('unhandledRejection', (error) => {
  console.error('未处理的Promise拒绝:', error);
  process.exit(1);
});

// 启动程序
main().catch(error => {
  console.error('程序执行失败:', error);
  process.exit(1);
});