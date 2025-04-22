// MediaQualityEvaluator.ts
import * as fs from "fs";
import * as path from "path";
import { exec } from "child_process";
import { promisify } from "util";
import { logger } from "./utils";

const execAsync = promisify(exec);

/**
 * 媒体质量评估结果
 */
interface QualityEvaluationResult {
  vmafScore: number; // VMAF评分
  audioScore: number; // 音频评分 (固定值)
  syncScore: number; // 同步性评分 (固定值)
}

/**
 * 媒体质量评估器 - 负责评估媒体质量
 */
export class MediaQualityEvaluator {
  private ffmpegPath: string;
  //   private vmafModelPath: string;
  private tempDir: string;

  constructor(ffmpegPath: string = "") {
    this.ffmpegPath = ffmpegPath;

    // 创建临时目录用于存储评估结果
    this.tempDir = path.join(process.cwd(), "temp", "vmaf");
    if (!fs.existsSync(this.tempDir)) {
      fs.mkdirSync(this.tempDir, { recursive: true });
    }

    logger.info(`媒体质量评估临时目录: ${this.tempDir}`);
  }

  /**
   * 计算GOP的VMAF评分
   * @param referenceGopPath 参考GOP文件路径 (原始视频)
   * @param distortedGopPath 待评估GOP文件路径 (转码结果)
   * @returns VMAF评分
   */
  /**
   * 计算GOP的VMAF评分
   * @param referenceGopPath 参考GOP文件路径 (原始视频)
   * @param distortedGopPath 待评估GOP文件路径 (转码结果)
   * @returns VMAF评分
   */
  async calculateVmafScore(
    referenceGopPath: string,
    distortedGopPath: string
  ): Promise<number> {
    logger.info(
      `计算VMAF评分 - 参考: ${referenceGopPath}, 待评估: ${distortedGopPath}`
    );

    try {
      // 创建输出文件名
      const outputFile = path.join(this.tempDir, `vmaf_${Date.now()}.json`);

      // 构建ffmpeg命令
      // 使用VMAF filter评估视频质量，不指定model_path让FFmpeg使用默认模型
      const cmd = `${
        this.ffmpegPath ? this.ffmpegPath + "/" : ""
      }ffmpeg -i "${distortedGopPath}" -i "${referenceGopPath}" -filter_complex "[0:v]scale=1920:1080:flags=bicubic[main];[1:v]scale=1920:1080:flags=bicubic[ref];[main][ref]libvmaf=log_fmt=json:log_path=${outputFile}" -f null -`;

      logger.debug(`执行ffmpeg命令: ${cmd}`);

      // 执行命令
      await execAsync(cmd);

      // 检查输出文件是否存在
      if (!fs.existsSync(outputFile)) {
        throw new Error(`VMAF评估失败，未找到输出文件: ${outputFile}`);
      }

      // 解析VMAF结果
      const vmafResult = JSON.parse(fs.readFileSync(outputFile, "utf-8"));
      const vmafScore = vmafResult.pooled_metrics?.vmaf?.mean || 0;

      logger.info(`VMAF评分: ${vmafScore}`);

      // 清理临时文件
      fs.unlinkSync(outputFile);

      return vmafScore;
    } catch (error) {
      logger.error(`计算VMAF评分失败: ${error}`);

      // 如果评估失败，返回一个很低的分数
      return 0;
    }
  }

  /**
   * 评估GOP的媒体质量
   * @param referenceGopPath 参考GOP文件路径 (原始视频)
   * @param distortedGopPath 待评估GOP文件路径 (转码结果)
   * @returns 质量评估结果
   */
  async evaluateGopQuality(
    referenceGopPath: string,
    distortedGopPath: string
  ): Promise<QualityEvaluationResult> {
    logger.info(
      `评估GOP质量 - 参考: ${referenceGopPath}, 待评估: ${distortedGopPath}`
    );

    // 计算VMAF评分
    const vmafScore = await this.calculateVmafScore(
      referenceGopPath,
      distortedGopPath
    );

    // 返回评估结果，包含固定的音频和同步性评分
    return {
      vmafScore,
      audioScore: this.getFixedAudioScore(),
      syncScore: this.getFixedSyncScore(),
    };
  }

  /**
   * 获取固定的音频评分
   * @returns 固定的音频评分 (PESQ)
   */
  getFixedAudioScore(): number {
    // 返回一个固定的高音频分数 (PESQ评分范围为-0.5到4.5)
    return 4.3;
  }

  /**
   * 获取固定的同步性评分
   * @returns 固定的同步性评分 (毫秒)
   */
  getFixedSyncScore(): number {
    // 返回一个固定的同步性评分 (0表示完美同步)
    return 0;
  }

  /**
   * 清理所有临时文件
   */
  cleanupTempFiles(): void {
    const files = fs.readdirSync(this.tempDir);
    for (const file of files) {
      fs.unlinkSync(path.join(this.tempDir, file));
    }
    logger.info(`已清理所有临时文件，共 ${files.length} 个`);
  }
}

export default MediaQualityEvaluator;
