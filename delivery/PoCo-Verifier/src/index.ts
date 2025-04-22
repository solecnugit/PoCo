#!/usr/bin/env node
import * as fs from "fs";
import * as path from "path";
import { VerifierService } from "./verifier-service";
import { VerifierConfig } from "./types";
import { logger } from "./utils";

// 命令行参数处理
const args = process.argv.slice(2);
const configPath = args.find((arg) => !arg.startsWith("--")) || "config.json";
const verbose = args.includes("--verbose");

// 设置日志级别
if (verbose) {
  logger.level = "debug";
} else {
  logger.level = "info";
}

// 显示启动信息
console.log(`
==================================
     验证者服务启动
==================================
配置文件: ${configPath}
日志级别: ${logger.level}
`);

// 加载配置
function loadConfig(configPath: string): VerifierConfig {
  try {
    const configFile = path.resolve(process.cwd(), configPath);
    logger.info(`加载配置文件: ${configFile}`);

    if (!fs.existsSync(configFile)) {
      logger.error(`配置文件不存在: ${configFile}`);
      process.exit(1);
    }

    const configData = fs.readFileSync(configFile, "utf-8");
    const config: VerifierConfig = JSON.parse(configData);

    // 验证必要的配置项
    const requiredFields = [
      "verifierAccountId",
      "contractId",
      "nearConfig",
      "ipfsConfig",
      "pollingInterval",
    ];

    for (const field of requiredFields) {
      if (!(config as any)[field]) {
        logger.error(`配置缺少必要字段: ${field}`);
        process.exit(1);
      }
    }

    return config;
  } catch (error) {
    logger.error(`加载配置失败: ${error}`);
    process.exit(1);
  }
}

// 主函数
async function main() {
  try {
    // 加载配置
    const config = loadConfig(configPath);
    logger.info("配置加载成功");

    // 创建验证者服务实例
    const verifierService = new VerifierService(config);
    logger.info("验证者服务实例已创建");

    // 初始化服务
    logger.info("正在初始化验证者服务...");
    const initResult = await verifierService.initialize();

    if (!initResult) {
      logger.error("验证者服务初始化失败");
      process.exit(1);
    }

    logger.info("验证者服务初始化成功");

    // 启动服务
    await verifierService.start();
    logger.info("验证者服务已启动");

    // 设置信号处理器以优雅地关闭
    process.on("SIGINT", async () => {
      logger.info("接收到SIGINT信号，准备关闭...");
      await verifierService.stop();
      process.exit(0);
    });

    process.on("SIGTERM", async () => {
      logger.info("接收到SIGTERM信号，准备关闭...");
      await verifierService.stop();
      process.exit(0);
    });

    // 保持进程运行
    logger.info("验证者服务正在运行，按Ctrl+C关闭");

    // 定期输出状态信息
    setInterval(() => {
      const queueLength = verifierService.getQueueLength();
      const isProcessing = verifierService.isProcessing();

      logger.info(
        `状态报告 - 队列长度: ${queueLength}, 正在处理: ${
          isProcessing ? "是" : "否"
        }`
      );
    }, 60000); // 每分钟报告一次状态
  } catch (error) {
    logger.error(`运行验证者服务时出错: ${error}`);
    process.exit(1);
  }
}

// 处理未捕获的异常
process.on("uncaughtException", (error) => {
  logger.error(`未捕获的异常: ${error}`);
  process.exit(1);
});

process.on("unhandledRejection", (reason, promise) => {
  logger.error(`未处理的Promise拒绝: ${reason}`);
  process.exit(1);
});

// 运行主函数
main().catch((error) => {
  logger.error(`主函数执行失败: ${error}`);
  process.exit(1);
});
