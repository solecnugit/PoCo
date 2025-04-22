// ipfs-service.ts
import fetch from "node-fetch";
import * as fs from "fs";
import * as path from "path";
import FormData from "form-data";
import { VerifierConfig } from "./types";
import { logger } from "./utils";

export class IpfsService {
  private apiUrl: string;
  private tempDir: string;

  constructor(config: VerifierConfig) {
    this.apiUrl = `${config.ipfsConfig.protocol}://${config.ipfsConfig.host}:${config.ipfsConfig.port}/api/v0`;

    // 创建临时目录用于存储下载的文件
    this.tempDir = path.join(process.cwd(), "temp");
    if (!fs.existsSync(this.tempDir)) {
      fs.mkdirSync(this.tempDir, { recursive: true });
    }

    logger.info(`IPFS临时文件目录: ${this.tempDir}`);
  }

  /**
   * 初始化IPFS服务并测试连接
   */
  async initialize(): Promise<boolean> {
    try {
      const response = await fetch(`${this.apiUrl}/version`, {
        method: "POST",
      });
      const data = await response.json();
      logger.info(`已连接到IPFS节点，版本: ${data.Version}`);
      return true;
    } catch (error) {
      logger.error(`连接IPFS节点失败: ${error}`);
      return false;
    }
  }

  /**
   * 从IPFS下载文件
   * @param cid IPFS内容标识符
   * @param fileName 可选的文件名，如果未提供，将使用CID作为文件名
   * @returns 下载文件的本地路径
   */
  async downloadFile(cid: string, fileName?: string): Promise<string> {
    try {
      logger.info(`开始从IPFS下载文件: ${cid}`);

      const localFileName = fileName || cid;
      const outputPath = path.join(this.tempDir, localFileName);

      // 检查文件是否已经存在
      if (fs.existsSync(outputPath)) {
        logger.info(`文件已存在于本地缓存: ${outputPath}`);
        return outputPath;
      }

      // 确保输出目录存在
      const outputDir = path.dirname(outputPath);
      if (!fs.existsSync(outputDir)) {
        fs.mkdirSync(outputDir, { recursive: true });
      }

      // 使用POST请求
      const response = await fetch(`${this.apiUrl}/cat?arg=${cid}`, {
        method: "POST",
      });

      if (!response.ok) {
        throw new Error(
          `IPFS API响应错误: ${response.status} ${response.statusText}`
        );
      }

      const buffer = await response.buffer();
      fs.writeFileSync(outputPath, buffer);

      logger.info(`文件下载成功: ${outputPath} (${buffer.length} 字节)`);
      return outputPath;
    } catch (error) {
      logger.error(`下载IPFS文件失败: ${error}`);
      throw error;
    }
  }

  /**
   * 清理临时文件
   * @param filePath 要清理的文件路径
   */
  async cleanupFile(filePath: string): Promise<void> {
    if (fs.existsSync(filePath)) {
      fs.unlinkSync(filePath);
      logger.debug(`已删除临时文件: ${filePath}`);
    }
  }

  /**
   * 清理所有临时文件
   */
  async cleanupAllFiles(): Promise<void> {
    const files = fs.readdirSync(this.tempDir);
    for (const file of files) {
      const filePath = path.join(this.tempDir, file);
      fs.unlinkSync(filePath);
    }
    logger.info(`已清理所有临时文件，共 ${files.length} 个`);
  }

  /**
   * 从IPFS下载媒体文件并返回本地路径
   * @param taskId 任务ID (用于日志和文件命名)
   * @param sourceIpfs 源文件IPFS哈希
   * @param resultIpfs 结果文件IPFS哈希
   * @returns 包含源文件和结果文件本地路径的对象
   */
  async downloadMediaFiles(
    taskId: string,
    sourceIpfs: string,
    resultIpfs: string
  ): Promise<{
    sourcePath: string;
    resultPath: string;
  }> {
    logger.info(`开始下载任务 ${taskId} 的媒体文件`);

    // 下载源文件
    const sourceFileName = `${taskId}-source.mp4`;
    const sourcePath = await this.downloadFile(sourceIpfs, sourceFileName);

    // 下载结果文件
    const resultFileName = `${taskId}-result.mp4`;
    const resultPath = await this.downloadFile(resultIpfs, resultFileName);

    logger.info(`任务 ${taskId} 的媒体文件下载完成`);

    return {
      sourcePath,
      resultPath,
    };
  }

  /**
   * 获取临时目录路径（用于测试）
   */
  getTempDir(): string {
    return this.tempDir;
  }
}

export default IpfsService;
