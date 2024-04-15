const fs = require("fs");
const path = require("path");
const { spawn } = require("child_process");

// 读取配置文件
const configPath = path.resolve(__dirname, "./mock/p2p-config.txt");
const config = fs.readFileSync(configPath, "utf-8");

// 按行分割配置文件，得到一个数组
const lines = config.split("\n");

// 对于每一行（每个节点），启动一个新的进程
lines.forEach((line, index) => {
  const [ip, port] = line.split(" ");

  // 设置环境变量
  const env = {
    SECRET: `NODE${index}`,
    P2P_PORT: port,
    HTTP_PORT: 3000 + index, // 假设 HTTP 端口号是 3000 + 节点索引
    PEERS: lines
      .filter((_, i) => i !== index) // 排除当前节点
      .map((line) => `ws://${line.replace(" ", ":")}`) // 将 IP 和端口号转换为 WebSocket 地址
      .join(","),
  };

  // 启动新的节点
  const child = spawn("node", ["app"], { env: { ...process.env, ...env } });

  child.stdout.on("data", (data) => {
    console.log(`${env.SECRET}: ${data}`);
  });

  child.stderr.on("data", (data) => {
    console.error(`stderr: ${data}`);
  });

  child.on("error", (error) => {
    console.error(`spawn error: ${error}`);
  });
});
