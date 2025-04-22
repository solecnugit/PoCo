// GopAnalyzer.ts
import * as fs from "fs";
import * as path from "path";
import { VideoSpecification } from "./types";
import { logger } from "./utils";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

/**
 * GOP分析器 - 负责视频GOP提取和分析
 */
export class GopAnalyzer {
  private ffmpegPath: string;
  private tempDir: string;

  constructor(ffmpegPath: string = "") {
    this.ffmpegPath = ffmpegPath;

    // 创建临时目录用于存储提取的GOP
    this.tempDir = path.join(process.cwd(), "temp", "gops");
    if (!fs.existsSync(this.tempDir)) {
      fs.mkdirSync(this.tempDir, { recursive: true });
    }

    logger.info(`GOP临时文件目录: ${this.tempDir}`);
  }

  /**
   * 将视频切分成多个GOP
   * @param videoPath 视频文件路径
   * @param timestamps 所有关键帧时间戳
   * @returns 切分后的GOP文件路径映射 (时间戳 -> 文件路径)
   */
  /**
   * 将视频切分成多个GOP
   * @param videoPath 视频文件路径
   * @param timestamps 所有关键帧时间戳
   * @returns 切分后的GOP文件路径映射 (时间戳 -> 文件路径)
   */
  async splitVideoIntoGops(
    videoPath: string,
    timestamps: string[]
  ): Promise<Map<string, string>> {
    logger.info(`将视频切分成GOP: ${videoPath}`);

    // 创建用于存储当前视频GOP的目录
    // 确保视频路径有扩展名，如果没有则添加.mp4
    const videoExt = path.extname(videoPath) || ".mp4";
    const videoPathWithExt = videoExt ? videoPath : `${videoPath}.mp4`;

    // 获取不带扩展名的文件名作为目录名
    const videoBaseName = path.basename(videoPath, videoExt);
    const gopDir = path.join(this.tempDir, videoBaseName);
    // const videoBaseName = path.basename(videoPath, path.extname(videoPath));
    // const gopDir = path.join(this.tempDir, videoBaseName);

    if (!fs.existsSync(gopDir)) {
      fs.mkdirSync(gopDir, { recursive: true });
    }

    try {
      // 将时间戳格式化为ffmpeg segment_times参数需要的格式
      const segmentTimes = timestamps.join(",");

      // 使用ffmpeg切分视频为多个GOP，确保添加引号包围参数
      const cmd = `${
        this.ffmpegPath ? this.ffmpegPath + "/" : ""
      }ffmpeg -i "${videoPathWithExt}" -f segment -segment_times "${segmentTimes}" -c copy -map 0 -reset_timestamps 1 "${gopDir}/segment_%d${videoExt}"`;

      logger.debug(`执行ffmpeg命令: ${cmd}`);
      await execAsync(cmd);

      // 创建时间戳到GOP文件路径的映射
      const gopMap = new Map<string, string>();

      // 处理第一个GOP (0到第一个时间戳)
      const firstGopPath = path.join(gopDir, `segment_0${videoExt}`);
      if (fs.existsSync(firstGopPath)) {
        // 第一个GOP实际上是从0开始，对应timestamps[0]
        gopMap.set(timestamps[0], firstGopPath);
      }

      // 处理中间的GOPs
      for (let i = 1; i < timestamps.length; i++) {
        const segmentPath = path.join(gopDir, `segment_${i}${videoExt}`);
        if (fs.existsSync(segmentPath)) {
          // segment_i对应的是从timestamps[i-1]到timestamps[i]的内容
          // 但我们用起始时间戳标识这个GOP
          gopMap.set(timestamps[i], segmentPath);
        }
      }

      // 处理最后一个GOP (从最后一个时间戳到视频结尾)
      const lastSegmentIdx = timestamps.length;
      const lastSegmentPath = path.join(
        gopDir,
        `segment_${lastSegmentIdx}${videoExt}`
      );
      if (fs.existsSync(lastSegmentPath)) {
        // 最后一个GOP也是用其起始时间戳标识
        gopMap.set(timestamps[timestamps.length - 1], lastSegmentPath);
      }

      logger.info(`视频成功切分为 ${gopMap.size} 个GOP`);
      return gopMap;
    } catch (error) {
      logger.error(`切分视频失败: ${error}`);
      throw error;
    }
  }

  /**
   * 提取指定的GOP
   * @param videoPath 视频文件路径
   * @param allTimestamps 所有关键帧时间戳
   * @param targetTimestamp 要提取的GOP的时间戳
   * @returns 提取的GOP文件路径
   */
  async extractGop(
    videoPath: string,
    allTimestamps: string[],
    targetTimestamp: string
  ): Promise<string> {
    logger.info(`提取GOP: ${targetTimestamp} 从文件: ${videoPath}`);

    try {
      // 确保目标时间戳在时间戳列表中
      if (!allTimestamps.includes(targetTimestamp)) {
        throw new Error(`目标时间戳 ${targetTimestamp} 不在关键帧列表中`);
      }

      // 先将视频切分成所有GOP
      const gopMap = await this.splitVideoIntoGops(videoPath, allTimestamps);

      // 获取目标GOP的路径
      const gopPath = gopMap.get(targetTimestamp);
      if (!gopPath) {
        throw new Error(`未找到时间戳为 ${targetTimestamp} 的GOP`);
      }

      logger.info(`GOP提取成功: ${gopPath}`);
      return gopPath;
    } catch (error) {
      logger.error(`提取GOP失败: ${error}`);
      throw error;
    }
  }

  /**
   * 计算GOP哈希值
   * @param gopPath GOP文件路径
   * @returns 哈希值
   */
  calculateGopHash(gopPath: string): string {
    try {
      // 读取文件内容
      const buffer = fs.readFileSync(gopPath);

      // 计算SHA-256哈希
      const crypto = require("crypto");
      const hash = crypto.createHash("sha256");
      hash.update(buffer);
      const hashValue = hash.digest("hex");

      logger.debug(`计算GOP哈希值: ${hashValue}`);
      return hashValue;
    } catch (error) {
      logger.error(`计算GOP哈希值失败: ${error}`);
      throw error;
    }
  }

  /**
   * 获取视频规格信息
   * @param videoPath 视频文件路径
   * @returns 视频规格
   */
  async getVideoSpecifications(videoPath: string): Promise<VideoSpecification> {
    logger.info(`获取视频规格信息: ${videoPath}`);

    try {
      // 使用ffprobe获取视频信息
      const cmd = `${
        this.ffmpegPath ? this.ffmpegPath + "/" : ""
      }ffprobe -v error -select_streams v -show_entries stream=codec_name,width,height,r_frame_rate,bit_rate -of json "${videoPath}"`;
      const { stdout } = await execAsync(cmd);

      const videoInfo = JSON.parse(stdout);
      const stream = videoInfo.streams[0];

      // 解析帧率（可能是分数形式）
      let framerate = 0;
      if (stream.r_frame_rate) {
        const [num, den] = stream.r_frame_rate.split("/");
        framerate = parseFloat(num) / parseFloat(den);
      }

      // 构建视频规格对象
      const specs: VideoSpecification = {
        codec: stream.codec_name,
        resolution: `${stream.width}x${stream.height}`,
        bitrate: parseInt(stream.bit_rate) || 0,
        framerate: parseFloat(framerate.toFixed(2)),
      };

      logger.info(`视频规格信息: ${JSON.stringify(specs)}`);
      return specs;
    } catch (error) {
      logger.error(`获取视频规格信息失败: ${error}`);
      throw error;
    }
  }

  /**
   * 清理特定目录下的所有GOP文件
   * @param dirPath 目录路径
   */
  cleanupGopDir(dirPath: string): void {
    if (fs.existsSync(dirPath)) {
      const files = fs.readdirSync(dirPath);
      for (const file of files) {
        fs.unlinkSync(path.join(dirPath, file));
      }
      fs.rmdirSync(dirPath);
      logger.debug(`已清理GOP目录: ${dirPath}`);
    }
  }

  /**
   * 清理所有GOP临时文件
   */
  cleanupAllGops(): void {
    const dirs = fs.readdirSync(this.tempDir);
    for (const dir of dirs) {
      const dirPath = path.join(this.tempDir, dir);
      if (fs.statSync(dirPath).isDirectory()) {
        this.cleanupGopDir(dirPath);
      } else {
        fs.unlinkSync(dirPath);
      }
    }
    logger.info(`已清理所有GOP临时文件`);
  }
}

export default GopAnalyzer;
