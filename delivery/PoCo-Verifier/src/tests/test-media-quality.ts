// test-media-quality.ts
import { GopAnalyzer } from "../GopAnalyzer";
import { MediaQualityEvaluator } from "../MediaQualityEvaluator";
import * as path from "path";
import * as fs from "fs";

// 创建简易的logger（如果没有utils/logger）
const logger = {
  info: (msg: string) => console.log(`[INFO] ${msg}`),
  error: (msg: string) => console.error(`[ERROR] ${msg}`),
  debug: (msg: string) => console.log(`[DEBUG] ${msg}`),
  warn: (msg: string) => console.warn(`[WARN] ${msg}`),
};

// 配置
const ffmpegPath = ""; // 空字符串表示使用系统PATH中的ffmpeg
const vmafModelPath = "/usr/share/model/vmaf_v0.6.1.json"; // 根据实际情况修改
const sourceVideoPath = path.join(
  __dirname,
  "test-videos",
  "blue_hair_1920x1080_30.mp4"
);
const resultVideoPath = path.join(
  __dirname,
  "test-videos",
  "blue_hair_1920x1080_30_output.mp4"
);

// 确保测试视频目录存在
const testVideosDir = path.join(__dirname, "test-videos");
if (!fs.existsSync(testVideosDir)) {
  fs.mkdirSync(testVideosDir, { recursive: true });
  logger.info(`创建测试视频目录: ${testVideosDir}`);
  logger.info(`请将测试视频放置在此目录中再运行测试`);
  process.exit(1);
}

// 检查测试视频文件是否存在
if (!fs.existsSync(sourceVideoPath) || !fs.existsSync(resultVideoPath)) {
  logger.error(`测试视频文件不存在，请确保以下文件存在:`);
  logger.error(`- ${sourceVideoPath}`);
  logger.error(`- ${resultVideoPath}`);
  process.exit(1);
}

// 所有的关键帧时间戳
const allTimestamps = [
  "0.000000",
  "1.200000",
  "2.266667",
  "3.466667",
  "5.133333",
  "9.133333",
];

// 要评估的关键帧时间戳
const selectedGops = ["0.000000", "2.266667", "5.133333"];

// 测试函数
async function testMediaQuality() {
  logger.info("开始测试媒体质量评估...");

  try {
    // 创建GopAnalyzer和MediaQualityEvaluator实例
    // 创建GopAnalyzer和MediaQualityEvaluator实例
    const gopAnalyzer = new GopAnalyzer(ffmpegPath);
    const mediaEvaluator = new MediaQualityEvaluator(ffmpegPath);

    // 步骤1: 展示关键帧时间戳
    logger.info("\n--- 使用的关键帧时间戳 ---");
    logger.info(`所有关键帧时间戳: ${allTimestamps.join(", ")}`);
    logger.info(`要评估的关键帧时间戳: ${selectedGops.join(", ")}`);

    // 步骤2: 切分源视频和结果视频
    logger.info("\n--- 切分视频为GOP ---");

    logger.info("将源视频切分成GOP...");
    const sourceGopMap = await gopAnalyzer.splitVideoIntoGops(
      sourceVideoPath,
      allTimestamps
    );
    logger.info(`源视频已切分为 ${sourceGopMap.size} 个GOP`);

    logger.info("将结果视频切分成GOP...");
    const resultGopMap = await gopAnalyzer.splitVideoIntoGops(
      resultVideoPath,
      allTimestamps
    );
    logger.info(`结果视频已切分为 ${resultGopMap.size} 个GOP`);

    // 步骤3: 评估选定GOP的媒体质量
    logger.info("\n--- 评估媒体质量 ---");

    for (const timestamp of selectedGops) {
      logger.info(`\n评估时间戳为 ${timestamp} 的GOP:`);

      // 获取源视频和结果视频的GOP路径
      const sourceGopPath = sourceGopMap.get(timestamp);
      const resultGopPath = resultGopMap.get(timestamp);

      if (!sourceGopPath || !resultGopPath) {
        logger.error(`未找到时间戳为 ${timestamp} 的GOP`);
        continue;
      }

      // 评估媒体质量
      const qualityResult = await mediaEvaluator.evaluateGopQuality(
        sourceGopPath,
        resultGopPath
      );

      // 显示评估结果
      logger.info(`VMAF评分: ${qualityResult.vmafScore.toFixed(2)}`);
      logger.info(`音频评分 (固定值): ${qualityResult.audioScore.toFixed(2)}`);
      logger.info(
        `同步性评分 (固定值): ${qualityResult.syncScore.toFixed(2)} ms`
      );

      // 输出评价
      if (qualityResult.vmafScore >= 95) {
        logger.info("视频质量评价: 非常优秀");
      } else if (qualityResult.vmafScore >= 80) {
        logger.info("视频质量评价: 优秀");
      } else if (qualityResult.vmafScore >= 70) {
        logger.info("视频质量评价: 良好");
      } else if (qualityResult.vmafScore >= 60) {
        logger.info("视频质量评价: 一般");
      } else {
        logger.info("视频质量评价: 较差");
      }
    }

    // 清理临时文件
    logger.info("\n--- 清理测试文件 ---");
    gopAnalyzer.cleanupAllGops();
    mediaEvaluator.cleanupTempFiles();

    logger.info("\n媒体质量评估测试完成!");
  } catch (error: any) {
    logger.error("测试过程中出错:");
    logger.error(error);
  }
}

// 运行测试
testMediaQuality().catch((error) => {
  logger.error("运行测试失败:");
  logger.error(error);
});
