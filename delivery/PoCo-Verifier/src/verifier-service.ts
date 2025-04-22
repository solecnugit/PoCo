// VerifierService.ts
import { NearConnection } from "./near-connection";
import { IpfsService } from "./ipfs-service";
import { CommitteeClient } from "./committee-client";
import { GopAnalyzer } from "./GopAnalyzer";
import { MediaQualityEvaluator } from "./MediaQualityEvaluator";
import {
  TaskData,
  VerifierConfig,
  GopScore,
  VerifierQosProof,
  VideoSpecification,
} from "./types";
import { logger } from "./utils";
import { generatePlaceholderSignature } from "./utils";

/**
 * 验证者服务 - 主服务模块
 * 负责协调区块链交互、IPFS文件获取、GOP分析和媒体质量评估等组件
 */
export class VerifierService {
  private config: VerifierConfig;
  private nearConnection: NearConnection;
  private ipfsService: IpfsService;
  private committeeClient: CommitteeClient;
  private gopAnalyzer: GopAnalyzer;
  private mediaEvaluator: MediaQualityEvaluator;

  // 任务处理状态标志
  private isProcessingTask: boolean = false;
  private taskQueue: TaskData[] = [];
  private pollingTimer: NodeJS.Timeout | null = null;
  private running: boolean = false;

  constructor(config: VerifierConfig) {
    this.config = config;
    this.nearConnection = new NearConnection(config);
    this.ipfsService = new IpfsService(config);
    this.committeeClient = new CommitteeClient(config);
    this.gopAnalyzer = new GopAnalyzer(config.ffmpegPath);
    this.mediaEvaluator = new MediaQualityEvaluator(config.ffmpegPath);
  }

  /**
   * 初始化所有服务
   */
  async initialize(): Promise<boolean> {
    logger.info("正在初始化验证者服务...");

    try {
      // 初始化子模块
      const nearInitResult = await this.nearConnection.initialize();
      if (!nearInitResult) {
        logger.error("区块链连接初始化失败");
        return false;
      }

      const ipfsInitResult = await this.ipfsService.initialize();
      if (!ipfsInitResult) {
        logger.error("IPFS服务初始化失败");
        return false;
      }

      const committeeInitResult = await this.committeeClient.initialize();
      if (!committeeInitResult) {
        logger.error("委员会客户端初始化失败");
        return false;
      }

      logger.info("验证者服务初始化成功");
      return true;
    } catch (error) {
      logger.error(`验证者服务初始化失败: ${error}`);
      return false;
    }
  }

  /**
   * 开始验证者服务
   */
  async start(): Promise<void> {
    if (this.running) {
      logger.warn("验证者服务已经在运行");
      return;
    }

    logger.info("启动验证者服务");
    this.running = true;

    // 立即执行一次轮询
    await this.pollTasks();

    // 设置定时轮询
    this.pollingTimer = setInterval(async () => {
      if (!this.isProcessingTask) {
        await this.pollTasks();
      }
    }, this.config.pollingInterval);

    logger.info(`已设置轮询间隔: ${this.config.pollingInterval}ms`);
  }

  /**
   * 停止验证者服务
   */
  async stop(): Promise<void> {
    logger.info("停止验证者服务");

    if (this.pollingTimer) {
      clearInterval(this.pollingTimer);
      this.pollingTimer = null;
    }

    this.running = false;
    logger.info("验证者服务已停止");
  }

  /**
   * 轮询分配的任务
   */
  private async pollTasks(): Promise<void> {
    console.log("11111111111111111111111111111111111111");
    if (this.isProcessingTask) {
      logger.debug("当前正在处理任务，跳过轮询");
      return;
    }

    try {
      logger.debug("轮询分配的任务");

      // 获取分配给验证者的未验证任务
      const tasks = await this.nearConnection.queryAssignedTasks(true);
      console.log(tasks);
      logger.info(`查询到 ${tasks.length} 个待验证任务`);

      // 将新任务添加到队列中（避免重复）
      let newTaskCount = 0;
      for (const task of tasks) {
        if (!this.taskQueue.some((t) => t.task_id === task.task_id)) {
          this.taskQueue.push(task);
          newTaskCount++;
        }
      }

      if (newTaskCount > 0) {
        logger.info(`添加 ${newTaskCount} 个新任务到队列`);
      }

      // 如果有任务等待处理且当前没有在处理任务，则开始处理
      if (this.taskQueue.length > 0 && !this.isProcessingTask) {
        logger.info(`开始处理队列中的任务，队列长度: ${this.taskQueue.length}`);
        await this.processNextTask();
      }
    } catch (error) {
      logger.error(`轮询任务时出错: ${error}`);
    }
  }

  /**
   * 处理队列中的下一个任务
   */
  private async processNextTask(): Promise<void> {
    if (this.isProcessingTask || this.taskQueue.length === 0) {
      return;
    }

    this.isProcessingTask = true;
    const task = this.taskQueue.shift();

    console.log("lalallala");
    console.log(task);

    try {
      logger.info(`开始处理任务: ${task!.task_id}`);
      const result = await this.verifyTask(task!);
      logger.info(`任务处理${result ? "成功" : "失败"}: ${task!.task_id}`);
    } catch (error) {
      logger.error(`处理任务 ${task!.task_id} 时出错: ${error}`);
    } finally {
      this.isProcessingTask = false;

      // 处理完成后，检查是否还有下一个任务
      if (this.taskQueue.length > 0) {
        // 使用setTimeout确保在当前执行栈完成后再处理下一个任务
        setTimeout(() => this.processNextTask(), 0);
      }
    }
  }

  /**
   * 验证单个任务
   * @param task 要验证的任务
   * @returns 是否验证成功
   */
  async verifyTask(task: TaskData): Promise<boolean> {
    logger.info(`开始验证任务: ${task.task_id}`);

    try {
      // 1. 确认任务中包含必要的信息
      if (!task.source_ipfs || !task.result_ipfs || !task.selected_gops) {
        throw new Error(`任务信息不完整: ${task.task_id}`);
      }

      if (!task.selected_gops.length) {
        throw new Error(`任务没有指定GOP: ${task.task_id}`);
      }

      logger.info(
        `任务信息: 源文件=${task.source_ipfs}, 结果文件=${task.result_ipfs}`
      );
      logger.info(`选定的GOP时间戳: ${task.selected_gops.join(", ")}`);

      // 2. 获取所有关键帧时间戳
      // 为了兼容性，如果任务中没有提供keyframeTimestamps，则使用selectedGops作为时间戳列表
      const allTimestamps = task.keyframe_timestamps || task.selected_gops;

      // 确保时间戳列表包含所有selectedGops
      for (const gopTimestamp of task.selected_gops) {
        if (!allTimestamps.includes(gopTimestamp)) {
          allTimestamps.push(gopTimestamp);
        }
      }

      logger.info(`所有关键帧时间戳: ${allTimestamps.join(", ")}`);

      // 3. 从IPFS下载媒体文件
      logger.info("从IPFS下载媒体文件...");
      const { sourcePath, resultPath } =
        await this.ipfsService.downloadMediaFiles(
          task.task_id,
          task.source_ipfs,
          task.result_ipfs
        );
      logger.info(
        `媒体文件下载完成: 源文件=${sourcePath}, 结果文件=${resultPath}`
      );

      // 4. 切分视频为GOP
      logger.info("将视频切分为GOP...");
      const sourceGopMap = await this.gopAnalyzer.splitVideoIntoGops(
        sourcePath,
        allTimestamps
      );
      const resultGopMap = await this.gopAnalyzer.splitVideoIntoGops(
        resultPath,
        allTimestamps
      );

      logger.info(`源视频已切分为 ${sourceGopMap.size} 个GOP`);
      logger.info(`结果视频已切分为 ${resultGopMap.size} 个GOP`);

      // 5. 对每个指定的GOP进行评估
      const gopScores: GopScore[] = [];
      logger.info("开始评估指定的GOP...");

      for (const gopTimestamp of task.selected_gops) {
        logger.info(`评估时间戳为 ${gopTimestamp} 的GOP`);

        // 获取指定GOP的文件路径
        const sourceGopPath = sourceGopMap.get(gopTimestamp);
        const resultGopPath = resultGopMap.get(gopTimestamp);

        if (!sourceGopPath || !resultGopPath) {
          logger.error(`未找到时间戳为 ${gopTimestamp} 的GOP`);
          continue;
        }

        // 计算VMAF分数
        const vmafScore = await this.mediaEvaluator.calculateVmafScore(
          sourceGopPath,
          resultGopPath
        );
        logger.info(`GOP ${gopTimestamp} 的VMAF评分: ${vmafScore}`);

        // 计算GOP哈希
        const gopHash = this.gopAnalyzer.calculateGopHash(resultGopPath);
        logger.info(
          `GOP ${gopTimestamp} 的哈希: ${gopHash.substring(0, 16)}...`
        );

        // 添加到评分列表
        gopScores.push({
          timestamp: gopTimestamp,
          vmaf_score: vmafScore,
          hash: gopHash,
        });
      }

      // 验证是否所有GOP都已评估
      if (gopScores.length === 0) {
        throw new Error(`没有GOP被成功评估: ${task.task_id}`);
      }

      if (gopScores.length < task.selected_gops.length) {
        logger.warn(
          `只有部分GOP被成功评估: ${gopScores.length}/${task.selected_gops.length}`
        );
      }

      // 6. 计算总体视频质量分数（所有GOP评分的平均值）
      const overallVideoScore =
        gopScores.reduce((sum, score) => sum + score.vmaf_score, 0) /
        gopScores.length;

      logger.info(`总体VMAF评分: ${overallVideoScore.toFixed(2)}`);

      // 7. 获取视频规格信息
      const videoSpecs = await this.gopAnalyzer.getVideoSpecifications(
        resultPath
      );
      logger.info(`视频规格: ${JSON.stringify(videoSpecs)}`);

      // 8. 创建验证者质量证明
      const proof: VerifierQosProof = {
        id: `proof-${task.task_id}-${this.config.verifierAccountId}`,
        task_id: task.task_id,
        verifier_id: this.config.verifierAccountId,
        timestamp: Date.now(),
        video_specs: videoSpecs,
        video_score: overallVideoScore,
        gop_scores: gopScores,
        audio_score: this.mediaEvaluator.getFixedAudioScore(),
        sync_score: this.mediaEvaluator.getFixedSyncScore(),
        signature: generatePlaceholderSignature(),
      };

      logger.info("已生成验证者质量证明");

      // 9. 将验证结果提交到区块链
      logger.info("提交验证结果到区块链...");
      const submitResult = await this.nearConnection.submitVerifierProof(proof);

      if (!submitResult) {
        throw new Error(`提交验证结果到区块链失败: ${task.task_id}`);
      }

      logger.info("验证结果已成功提交到区块链");

      console.log("验证结果:");
      console.log(proof);

      // 10. 发送验证结果给委员会
      logger.info("发送验证结果给委员会...");
      const sendResult = await this.committeeClient.sendProofToCommittee(proof);

      if (!sendResult) {
        logger.warn(
          `发送验证结果给委员会失败，但已成功提交到链上: ${task.task_id}`
        );
      } else {
        logger.info("验证结果已成功发送给委员会");
      }

      // 11. 清理资源
      logger.info("清理临时文件...");

      // 清理下载的媒体文件
      this.ipfsService.cleanupFile(sourcePath);
      this.ipfsService.cleanupFile(resultPath);

      // 清理GOP目录
      this.gopAnalyzer.cleanupAllGops();

      logger.info(`任务验证成功完成: ${task.task_id}`);
      return true;
    } catch (error) {
      logger.error(`验证任务 ${task.task_id} 失败: ${error}`);
      return false;
    }
  }

  /**
   * 获取当前任务队列长度
   */
  getQueueLength(): number {
    return this.taskQueue.length;
  }

  /**
   * 检查服务是否在运行
   */
  isRunning(): boolean {
    return this.running;
  }

  /**
   * 检查是否正在处理任务
   */
  isProcessing(): boolean {
    return this.isProcessingTask;
  }
}

export default VerifierService;
