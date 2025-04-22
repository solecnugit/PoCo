import winston from "winston";
import config from "./config";
import crypto from "crypto";

// 配置日志记录器
export const logger = winston.createLogger({
  level: config.logLevel,
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.printf(({ level, message, timestamp }) => {
      return `${timestamp} [${level.toUpperCase()}]: ${message}`;
    })
  ),
  transports: [
    new winston.transports.Console(),
    new winston.transports.File({ filename: "verifier.log" }),
  ],
});

// 生成随机ID
export function generateId(prefix: string): string {
  const randomBytes = crypto.randomBytes(8).toString("hex");
  return `${prefix}-${randomBytes}`;
}

// 计算字符串的哈希值
export function calculateHash(data: string): string {
  return crypto.createHash("sha256").update(data).digest("hex");
}

// 当前时间戳（毫秒）
export function getCurrentTimestamp(): number {
  return Date.now();
}

/**
 * 生成签名占位符
 * 注意：这不是真正的加密签名，仅用于演示目的
 * @returns 签名字符串
 */
export function generatePlaceholderSignature(): string {
  // 生成当前时间戳
  const timestamp = Date.now();

  // 生成一个随机数
  const random = Math.floor(Math.random() * 10000000);

  // 转换为16进制字符串并组合
  const timestampHex = timestamp.toString(16);
  const randomHex = random.toString(16).padStart(8, "0");

  // 创建一个看起来像签名的字符串
  const signature = `${timestampHex}${randomHex}verifier_placeholder_sig`;

  return signature;
}
