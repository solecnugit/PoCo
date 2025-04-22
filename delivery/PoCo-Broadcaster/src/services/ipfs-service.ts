// src/services/ipfs-service.ts
import { create, IPFSHTTPClient } from 'ipfs-http-client';
import { BroadcasterConfig } from '../../../src/types';

export class IPFSService {
  private config: BroadcasterConfig;
  private ipfs: IPFSHTTPClient;
  
  constructor(config: BroadcasterConfig) {
    this.config = config;
    this.ipfs = create({
      host: config.ipfsConfig.host,
      port: config.ipfsConfig.port,
      protocol: config.ipfsConfig.protocol
    });
  }

  /**
   * 测试IPFS连接
   */
  async testConnection(): Promise<boolean> {
    try {
      const version = await this.ipfs.version();
      console.log(`已连接到IPFS节点，版本: ${version.version}`);
      return true;
    } catch (error) {
      console.error('连接IPFS节点失败:', error);
      return false;
    }
  }

  /**
   * 上传文件到IPFS
   * @param file 浏览器File对象
   */
  async uploadFile(file: File): Promise<string> {
    try {
      console.log(`开始上传文件到IPFS: ${file.name}, 大小: ${file.size} 字节`);
      
      // 使用文件上传到IPFS
      const added = await this.ipfs.add(file, {
        progress: (prog) => console.log(`上传进度: ${prog} / ${file.size}`)
      });
      
      console.log(`文件上传成功, CID: ${added.cid.toString()}`);
      return added.cid.toString();
    } catch (error) {
      console.error('上传文件到IPFS失败:', error);
      throw error;
    }
  }

  /**
   * 获取IPFS文件网关URL
   */
  getFileUrl(cid: string): string {
    return `${this.config.ipfsConfig.gateway}/ipfs/${cid}`;
  }
}

export default IPFSService;