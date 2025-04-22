// test-ipfs-service.ts
import { IpfsService } from '../ipfs-service';
import config from '../config';
import * as fs from 'fs';
import * as path from 'path';

async function testIpfsService() {
  console.log('开始测试IPFS服务...');
  
  // 初始化IPFS服务
  const ipfsService = new IpfsService(config);
  const initialized = await ipfsService.initialize();
  
  if (!initialized) {
    console.error('IPFS服务初始化失败，请检查IPFS节点是否正常运行');
    return;
  }
  
  console.log('IPFS服务初始化成功');
  
  // 测试CID - 使用一个已知存在于IPFS网络的文件
  // 这是一个小的测试文件
  const testCid = 'QmWATWQ7fVPP2EFGu71UkfnqhYXDYH566qy47CnJDgvs8u';
  
  try {
    console.log(`尝试从IPFS下载文件: ${testCid}`);
    const filePath = await ipfsService.downloadFile(testCid, 'ipfs-test-file.txt');
    
    console.log(`文件下载成功: ${filePath}`);
    
    // 读取并显示文件内容
    if (fs.existsSync(filePath)) {
      const content = fs.readFileSync(filePath, 'utf8');
      console.log('文件内容:', content);
    } else {
      console.log('文件不存在');
    }
  } catch (error) {
    console.error('下载文件失败:', error);
  }
  
  // 测试下载媒体文件功能
  const sampleTaskId = 'test-task-123';
  const sampleSourceIpfs = 'QmWATWQ7fVPP2EFGu71UkfnqhYXDYH566qy47CnJDgvs8u'; // 替换为实际可用的CID
  const sampleResultIpfs = 'QmWATWQ7fVPP2EFGu71UkfnqhYXDYH566qy47CnJDgvs8u'; // 替换为实际可用的CID
  
  try {
    console.log(`尝试下载媒体文件，任务ID: ${sampleTaskId}`);
    console.log(`源文件CID: ${sampleSourceIpfs}`);
    console.log(`结果文件CID: ${sampleResultIpfs}`);
    
    const { sourcePath, resultPath } = await ipfsService.downloadMediaFiles(
      sampleTaskId,
      sampleSourceIpfs,
      sampleResultIpfs
    );
    
    console.log(`源文件下载成功: ${sourcePath}`);
    console.log(`结果文件下载成功: ${resultPath}`);
  } catch (error) {
    console.error('下载媒体文件失败:', error);
  }
  
  // 测试清理临时文件
  try {
    console.log('清理所有临时文件...');
    await ipfsService.cleanupAllFiles();
    console.log('临时文件清理完成');
  } catch (error) {
    console.error('清理临时文件失败:', error);
  }
}

// 运行测试
testIpfsService().catch(console.error);