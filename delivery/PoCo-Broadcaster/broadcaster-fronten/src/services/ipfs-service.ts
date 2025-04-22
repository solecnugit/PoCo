import type { BroadcasterConfig } from '@/types';
import ApiService from './api-service';

export class IpfsService {
  private gateway: string;

  constructor(config: BroadcasterConfig) {
    this.gateway = config.ipfsConfig.gateway;
  }

  // 获取IPFS网关URL
  getFileUrl(cid: string): string {
    return `${this.gateway}/ipfs/${cid}`;
  }

  // 上传文件到IPFS
  async uploadFile(file: File, onProgress?: (progress: number) => void): Promise<string> {
    try {
      console.log(`开始上传文件到IPFS: ${file.name}, 大小: ${file.size} 字节`);
      
      // 通过API服务上传文件
      const cid = await ApiService.uploadFileToIpfs(file);
      
      console.log(`文件上传成功, CID: ${cid}`);
      return cid;
    } catch (error) {
      console.error('上传文件到IPFS失败:', error);
      throw error;
    }
  }
}

export default IpfsService;