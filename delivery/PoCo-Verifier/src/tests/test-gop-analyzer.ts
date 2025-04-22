// test-gop-analyzer.ts
import { GopAnalyzer } from "../GopAnalyzer";
import * as path from "path";
import * as fs from "fs";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

// 创建简易的logger（如果没有utils/logger）
const logger = {
  info: (msg: string) => console.log(`[INFO] ${msg}`),
  error: (msg: string) => console.error(`[ERROR] ${msg}`),
  debug: (msg: string) => console.log(`[DEBUG] ${msg}`),
  warn: (msg: string) => console.warn(`[WARN] ${msg}`),
};

// 配置
const ffmpegPath = ""; // 空字符串表示使用系统PATH中的ffmpeg
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

// 所有的关键帧时间戳 - 使用固定提供的时间戳而不是动态查询
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
async function testGopAnalyzer() {
  logger.info("开始测试GopAnalyzer...");

  try {
    // 创建GopAnalyzer实例
    const gopAnalyzer = new GopAnalyzer(ffmpegPath);

    // 步骤1: 展示关键帧时间戳
    logger.info("\n--- 使用的关键帧时间戳 ---");
    logger.info(`所有关键帧时间戳: ${allTimestamps.join(", ")}`);
    logger.info(`要评估的关键帧时间戳: ${selectedGops.join(", ")}`);

    // 步骤2: 测试视频切分
    logger.info("\n--- 测试视频切分 ---");

    logger.info("将源视频切分成GOP...");
    const sourceGopMap = await gopAnalyzer.splitVideoIntoGops(
      sourceVideoPath,
      allTimestamps
    );
    logger.info(`源视频已切分为 ${sourceGopMap.size} 个GOP`);
    logger.info(
      `提取的GOP时间戳: ${Array.from(sourceGopMap.keys()).join(", ")}`
    );

    logger.info("将结果视频切分成GOP...");
    const resultGopMap = await gopAnalyzer.splitVideoIntoGops(
      resultVideoPath,
      allTimestamps
    );
    logger.info(`结果视频已切分为 ${resultGopMap.size} 个GOP`);
    logger.info(
      `提取的GOP时间戳: ${Array.from(resultGopMap.keys()).join(", ")}`
    );

    // 步骤3: 测试GOP评估
    logger.info("\n--- 测试GOP评估 ---");

    // 评估选定的GOP
    for (const timestamp of selectedGops) {
      logger.info(`\n评估时间戳为 ${timestamp} 的GOP:`);

      // 获取源视频和结果视频的GOP路径
      const sourceGopPath = sourceGopMap.get(timestamp);
      const resultGopPath = resultGopMap.get(timestamp);

      if (!sourceGopPath || !resultGopPath) {
        logger.error(`未找到时间戳为 ${timestamp} 的GOP`);
        continue;
      }

      // 计算GOP哈希
      const sourceHash = gopAnalyzer.calculateGopHash(sourceGopPath);
      const resultHash = gopAnalyzer.calculateGopHash(resultGopPath);

      logger.info(`源视频GOP路径: ${sourceGopPath}`);
      logger.info(`源视频GOP哈希: ${sourceHash}`);
      logger.info(`结果视频GOP路径: ${resultGopPath}`);
      logger.info(`结果视频GOP哈希: ${resultHash}`);
      logger.info(`哈希值是否匹配: ${sourceHash === resultHash ? "是" : "否"}`);

      // 获取GOP文件大小
      const sourceStats = fs.statSync(sourceGopPath);
      const resultStats = fs.statSync(resultGopPath);
      logger.info(`源视频GOP大小: ${sourceStats.size} 字节`);
      logger.info(`结果视频GOP大小: ${resultStats.size} 字节`);
    }

    // 步骤4: 测试获取视频规格
    logger.info("\n--- 测试获取视频规格 ---");

    const sourceSpecs = await gopAnalyzer.getVideoSpecifications(
      sourceVideoPath
    );
    logger.info("源视频规格:");
    logger.info(JSON.stringify(sourceSpecs, null, 2));

    const resultSpecs = await gopAnalyzer.getVideoSpecifications(
      resultVideoPath
    );
    logger.info("结果视频规格:");
    logger.info(JSON.stringify(resultSpecs, null, 2));

    // 清理临时文件
    logger.info("\n--- 清理测试文件 ---");
    gopAnalyzer.cleanupAllGops();

    logger.info("\nGopAnalyzer测试完成!");
  } catch (error: any) {
    logger.error("测试过程中出错:");
    logger.error(error);
  }
}

// 运行测试
testGopAnalyzer().catch((error) => {
  logger.error("运行测试失败:");
  logger.error(error);
});
