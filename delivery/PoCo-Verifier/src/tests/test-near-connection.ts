// test-near-connection.ts
import { NearConnection } from '../near-connection';
import config from '../config';

async function testNearConnection() {
  console.log('开始测试NEAR连接...');
  
  // 初始化连接
  const nearConnection = new NearConnection(config);
  const initialized = await nearConnection.initialize();
  
  if (!initialized) {
    console.error('NEAR连接初始化失败');
    return;
  }
  
  console.log('NEAR连接初始化成功');
  
  // 测试查询分配的任务
  try {
    console.log('查询分配给验证者的任务...');
    const assignedTasks = await nearConnection.queryAssignedTasks();
    console.log(`找到 ${assignedTasks.length} 个分配给验证者的任务:`);
    console.log(JSON.stringify(assignedTasks, null, 2));
  } catch (error) {
    console.error('查询分配的任务失败:', error);
  }
  
  // 如果有任务，测试获取特定任务
  try {
    console.log('查询分配给验证者的任务...');
    const assignedTasks = await nearConnection.queryAssignedTasks();
    
    if (assignedTasks.length > 0) {
      const taskId = assignedTasks[0].taskId;
      console.log(`获取任务详情: ${taskId}`);
      const task = await nearConnection.getTask(taskId);
      console.log('任务详情:');
      console.log(JSON.stringify(task, null, 2));
    } else {
      console.log('没有可用任务，跳过获取任务详情测试');
    }
  } catch (error) {
    console.error('获取任务详情失败:', error);
  }
  
  // 测试获取委员会Leader
  try {
    console.log('获取委员会Leader信息...');
    const leader = await nearConnection.getCommitteeLeader();
    console.log('委员会Leader信息:');
    console.log(JSON.stringify(leader, null, 2));
  } catch (error) {
    console.error('获取委员会Leader信息失败:', error);
  }
  
  // 测试获取任务验证状态（如果有任务）
  try {
    const assignedTasks = await nearConnection.queryAssignedTasks();
    
    if (assignedTasks.length > 0) {
      const taskId = assignedTasks[0].taskId;
      console.log(`获取任务验证状态: ${taskId}`);
      const status = await nearConnection.getTaskVerificationStatus(taskId);
      console.log('任务验证状态:');
      console.log(JSON.stringify(status, null, 2));
    } else {
      console.log('没有可用任务，跳过获取任务验证状态测试');
    }
  } catch (error) {
    console.error('获取任务验证状态失败:', error);
  }

  // 测试获取共识证明（如果有任务）
  try {
    const assignedTasks = await nearConnection.queryAssignedTasks();
    
    if (assignedTasks.length > 0) {
      const taskId = assignedTasks[0].taskId;
      console.log(`获取任务共识证明: ${taskId}`);
      const consensusProof = await nearConnection.getConsensusProof(taskId);
      console.log('任务共识证明:');
      console.log(JSON.stringify(consensusProof, null, 2));
    } else {
      console.log('没有可用任务，跳过获取任务共识证明测试');
    }
  } catch (error) {
    console.error('获取任务共识证明失败:', error);
  }
}

// 运行测试
testNearConnection().catch(console.error);