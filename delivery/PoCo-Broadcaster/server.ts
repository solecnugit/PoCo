// server.ts (后端)
import express from "express";
import { Request, Response } from "express";
import cors from "cors";
import path from "path";
import { connect, keyStores, KeyPair, Contract, Account } from "near-api-js";
import * as fs from "fs";
import * as os from "os";
import axios from "axios";
import multer from "multer";
import FormData from "form-data";
import fetch from "node-fetch";
import { HttpsProxyAgent } from "https-proxy-agent";

const app = express();
const PORT = process.env.PORT || 3000;

// 启用CORS和JSON解析
app.use(cors());
app.use(express.json());
app.use(express.static(path.join(__dirname, "public")));

// NEAR配置
const NEAR_CONFIG = {
  networkId: "testnet",
  nodeUrl: "https://rpc.testnet.near.org",
  walletUrl: "https://wallet.testnet.near.org",
  helperUrl: "https://helper.testnet.near.org",
  contractId: "pococontract11.testnet",
  broadcasterAccountId: "pocobroadcaster1.testnet", // 替换为你的账户ID
};

// IPFS配置
const IPFS_CONFIG = {
  host: "localhost",
  port: 5001,
  gateway: "http://localhost:8080",
};

// 配置multer处理文件上传
const upload = multer({ dest: "uploads/" });

// 初始化合约
let nearAccount: Account;
let contract: any;
let nearConnection: any;

// 从本地凭证加载NEAR账户
async function initNear() {
  try {
    console.log("开始初始化NEAR连接...");

    // 创建代理 (如果需要)
    const proxyAgent = new HttpsProxyAgent("http://127.0.0.1:10809");

    // 设置自定义的全局 fetch
    (global as any).fetch = (url: string, options: any = {}) => {
      return fetch(url, {
        ...options,
        agent: proxyAgent,
        timeout: 30000, // 增加超时时间
      });
    };

    // 创建密钥存储
    const keyStore = new keyStores.InMemoryKeyStore();

    // 从文件加载凭证
    const credentialsPath = path.join(
      os.homedir(),
      ".near-credentials",
      NEAR_CONFIG.networkId
    );
    const credentialsFile = path.join(
      credentialsPath,
      `${NEAR_CONFIG.broadcasterAccountId}.json`
    );

    if (!fs.existsSync(credentialsFile)) {
      throw new Error(`找不到凭证文件: ${credentialsFile}`);
    }

    console.log(`加载凭证文件: ${credentialsFile}`);
    const credentials = JSON.parse(fs.readFileSync(credentialsFile, "utf-8"));
    const keyPair = KeyPair.fromString(credentials.private_key);

    // 添加到密钥存储
    await keyStore.setKey(
      NEAR_CONFIG.networkId,
      NEAR_CONFIG.broadcasterAccountId,
      keyPair
    );

    // 连接到NEAR
    console.log("连接到NEAR网络...");
    nearConnection = await connect({
      ...NEAR_CONFIG,
      keyStore,
      headers: {},
    });

    // 获取账户对象
    console.log("获取账户对象...");
    nearAccount = await nearConnection.account(
      NEAR_CONFIG.broadcasterAccountId
    );

    // 初始化合约 - 更新方法列表以匹配新合约
    console.log("初始化合约...");
    contract = new Contract(nearAccount, NEAR_CONFIG.contractId, {
      viewMethods: [
        "get_available_tasks",
        "get_task",
        "get_consensus_proof",
        "get_task_consensus_details",
        "get_worker_info",
        "get_worker_qos_score",
        "get_worker_performance_summary",
        "get_task_offers",
        "get_worker_daily_stats",
        "get_task_queue",
        "get_broadcaster_tasks",
        "get_worker_qos_details",
        "get_verifier_proof",
      ],
      changeMethods: [
        "publish_task",
        "select_and_assign_verifiers",
        "request_supplemental_verifier",
        "check_offer_timeout",
        "check_broadcaster_offer_timeout",
        "complete_task",
      ],
      useLocalViewExecution: false,
    });

    // 测试合约连接
    console.log("测试合约连接...");
    const tasks = await contract.get_available_tasks({
      from_index: 0,
      limit: 5,
    });
    console.log(`成功获取任务, 数量: ${tasks.length}`);

    console.log("NEAR初始化成功!");
    return true;
  } catch (error) {
    console.error("NEAR初始化失败:", error);
    return false;
  }
}

// 获取账户信息的API
app.get("/api/account", async (req, res) => {
  try {
    if (!nearAccount) {
      await initNear();
    }

    res.json({
      accountId: NEAR_CONFIG.broadcasterAccountId,
      isSignedIn: true,
    });
  } catch (error) {
    res.status(500).json({
      error: String(error),
      message: "获取账户信息失败",
    });
  }
});

// IPFS文件上传端点
app.post(
  "/api/ipfs/upload",
  upload.single("file"),
  async (req, res): Promise<void> => {
    try {
      if (!req.file) {
        res.status(400).json({ error: "没有上传文件" });
        return;
      }

      console.log(
        `接收到文件上传请求: ${req.file.originalname}, 大小: ${req.file.size} 字节`
      );

      // 创建新的FormData
      const formData = new FormData();
      formData.append("file", fs.createReadStream(req.file.path));

      // 将请求转发到IPFS节点
      const ipfsUrl = `http://${IPFS_CONFIG.host}:${IPFS_CONFIG.port}/api/v0/add?pin=true`;
      console.log(`转发请求到IPFS: ${ipfsUrl}`);

      const ipfsResponse = await axios.post(ipfsUrl, formData, {
        headers: { ...formData.getHeaders() },
      });

      console.log(`IPFS上传成功, 响应:`, ipfsResponse.data);

      // 清理临时文件
      fs.unlinkSync(req.file.path);

      // 返回IPFS结果
      res.json(ipfsResponse.data);
    } catch (error) {
      console.error("IPFS上传失败:", error);
      res.status(500).json({
        error: String(error),
        message: "IPFS上传失败",
      });
    }
  }
);

// IPFS状态检查
app.get("/api/ipfs/status", async (req, res) => {
  try {
    const ipfsUrl = `http://${IPFS_CONFIG.host}:${IPFS_CONFIG.port}/api/v0/version`;
    console.log(`检查IPFS状态: ${ipfsUrl}`);

    const ipfsResponse = await axios.post(ipfsUrl);

    res.json({
      status: "connected",
      version: ipfsResponse.data.Version,
    });
  } catch (error) {
    console.error("IPFS状态检查失败:", error);
    res.json({
      status: "disconnected",
      error: String(error),
      message: "IPFS连接失败，请检查IPFS节点是否运行",
    });
  }
});

// 发布任务的API - 更新以支持新参数
app.post("/api/tasks", async (req, res) => {
  console.log("收到发布任务请求");
  try {
    const {
      source_ipfs,
      requirements,
      hw_acceleration_preferred = false,
    } = req.body;

    if (!contract) {
      console.log("合约未初始化，正在尝试初始化...");
      await initNear();

      if (!contract) {
        throw new Error("合约初始化失败");
      }
    }

    console.log(
      `发布任务请求: 源IPFS=${source_ipfs}, 要求=`,
      requirements,
      `硬件加速=${hw_acceleration_preferred}`
    );

    // 使用更新的合约方法，包括新参数
    console.log("调用合约方法 publish_task...");
    const result = await contract.publish_task({
      source_ipfs,
      requirements,
      hw_acceleration_preferred,
    });

    console.log(`任务发布成功, ID: ${result}`);

    // 设置10秒后自动检查该broadcaster的所有任务offer超时
    setTimeout(async () => {
      try {
        console.log(`执行broadcaster的所有任务offer超时检查`);
        await contract.check_broadcaster_offer_timeout({
          broadcaster_id: NEAR_CONFIG.broadcasterAccountId,
        });
        console.log(`broadcaster的任务offer超时检查完成`);
      } catch (error) {
        console.error(`broadcaster的任务offer超时检查失败:`, error);
      }
    }, 10000);

    res.json({ taskId: result });
  } catch (error) {
    console.error("发布任务失败:", error);
    console.error(
      "错误详情:",
      error instanceof Error ? error.stack : String(error)
    );
    res.status(500).json({
      error: String(error),
      message: "发布任务失败，请检查参数和合约状态",
    });
  }
});

// 检查broadcaster所有任务的offer超时
app.post("/api/broadcaster/check-offers-timeout", async (req, res) => {
  try {
    if (!contract) {
      console.log("合约未初始化，正在尝试初始化...");
      await initNear();
    }

    console.log(`检查broadcaster所有任务的offer超时`);

    // 调用合约检查broadcaster的所有任务
    const result = await contract.check_broadcaster_offer_timeout({
      broadcaster_id: NEAR_CONFIG.broadcasterAccountId,
    });

    console.log(`broadcaster任务offer超时检查结果: ${result}`);
    res.json({
      success: true,
      processed: result,
      message: result ? "已处理超时任务" : "没有需要处理的超时任务",
    });
  } catch (error) {
    console.error("检查broadcaster任务offer超时失败:", error);
    res.status(500).json({
      error: String(error),
      message: "检查broadcaster任务offer超时失败",
    });
  }
});

// 检查任务offer收集超时
app.post("/api/tasks/:taskId/check-offer-timeout", async (req, res) => {
  try {
    const { taskId } = req.params;

    if (!contract) {
      console.log("合约未初始化，正在尝试初始化...");
      await initNear();
    }

    console.log(`检查任务offer收集超时: 任务ID=${taskId}`);

    // 调用合约检查超时
    const result = await contract.check_offer_timeout({
      task_id: taskId,
    });

    console.log(`任务 ${taskId} offer收集检查结果: ${result}`);
    res.json({
      success: result,
      message: result ? "任务已进入分配流程" : "任务offer收集尚未超时",
    });
  } catch (error) {
    console.error("检查任务offer收集超时失败:", error);
    res.status(500).json({
      error: String(error),
      message: "检查任务offer收集超时失败",
    });
  }
});

// // 选择并分配验证者的API - 更新方法名
// app.post("/api/tasks/:taskId/verifiers", async (req, res) => {
//   try {
//     const { taskId } = req.params;

//     if (!contract) {
//       console.log("合约未初始化，正在尝试初始化...");
//       await initNear();
//     }

//     console.log(`选择验证者请求: 任务ID=${taskId}`);

//     // 调用更新的合约选择验证者方法
//     const verifiers = await contract.select_and_assign_verifiers({
//       task_id: taskId,
//     });

//     console.log(`已为任务 ${taskId} 选择验证者:`, verifiers);
//     res.json({
//       verifiers,
//       message: "验证者选择并分配成功",
//     });
//   } catch (error) {
//     console.error("选择验证者失败:", error);
//     res.status(500).json({
//       error: String(error),
//       message: '选择并分配验证者失败，请检查任务状态是否为"已完成"',
//     });
//   }
// });

// 请求补充验证者
// app.post("/api/tasks/:taskId/supplemental-verifier", async (req, res) => {
//   try {
//     const { taskId } = req.params;

//     if (!contract) {
//       console.log("合约未初始化，正在尝试初始化...");
//       await initNear();
//     }

//     console.log(`请求补充验证者: 任务ID=${taskId}`);

//     // 调用合约请求补充验证者
//     const verifier = await contract.request_supplemental_verifier({
//       task_id: taskId,
//     });

//     if (verifier) {
//       console.log(`已为任务 ${taskId} 添加补充验证者: ${verifier}`);
//       res.json({
//         verifier,
//         message: "补充验证者添加成功",
//       });
//     } else {
//       console.log(`任务 ${taskId} 无法添加补充验证者`);
//       res.json({
//         verifier: null,
//         message: "无法添加补充验证者，可能没有可用验证者",
//       });
//     }
//   } catch (error) {
//     console.error("请求补充验证者失败:", error);
//     res.status(500).json({
//       error: String(error),
//       message: "请求补充验证者失败",
//     });
//   }
// });

// 获取任务的API (只读操作)
// app.get("/api/tasks", async (req, res) => {
//   try {
//     const { from_index, limit } = req.query;
//     console.log("收到/api/tasks 请求");

//     if (!contract) {
//       console.log("合约未初始化，正在尝试初始化...");
//       await initNear();
//     }

//     // 调用合约获取任务
//     const tasks = await contract.get_available_tasks({
//       from_index: Number(from_index) || 0,
//       limit: Number(limit) || 50,
//     });

//     console.log(`成功获取${tasks.length}个可用任务`);
//     res.json(tasks);
//   } catch (error) {
//     console.error("获取任务失败:", error);
//     res.status(500).json({
//       error: String(error),
//       message: "获取可用任务失败",
//     });
//   }
// });

// 获取任务验证状态
app.get("/api/tasks/:taskId/verification-status", async (req, res) => {
  try {
    const { taskId } = req.params;

    if (!contract) {
      await initNear();
    }

    console.log(`获取任务验证状态请求: 任务ID=${taskId}`);

    const status = await contract.get_task_verification_status({
      task_id: taskId,
    });

    if (status) {
      console.log(`成功获取任务验证状态: ${taskId}`);
      res.json(status);
    } else {
      console.log(`任务验证状态不存在: ${taskId}`);
      res.json(null);
    }
  } catch (error) {
    console.error("获取任务验证状态失败:", error);
    res.status(500).json({
      error: String(error),
      message: "获取任务验证状态失败",
    });
  }
});

// 获取特定任务的特定验证者的验证结果
app.get("/api/tasks/:taskId/verifier-proof/:verifierId", async (req, res) => {
  try {
    const { taskId, verifierId } = req.params;

    if (!contract) {
      console.log("合约未初始化，正在尝试初始化...");
      await initNear();
    }

    console.log(
      `获取验证者质量证明请求: 任务ID=${taskId}, 验证者ID=${verifierId}`
    );

    // 调用合约获取验证者质量证明，注意方法名已更新为 get_task_verifier_proof
    const proof = await contract.get_verifier_proof({
      task_id: taskId,
      verifier_id: verifierId,
    });

    if (proof) {
      console.log(`成功获取验证者质量证明: ${taskId}, ${verifierId}`);
      res.json(proof);
    } else {
      console.log(`验证者质量证明不存在: ${taskId}, ${verifierId}`);
      res.status(404).json({
        error: "Verifier proof not found",
        message: "验证者质量证明不存在",
      });
    }
  } catch (error) {
    console.error("获取验证者质量证明失败:", error);
    res.status(500).json({
      error: String(error),
      message: "获取验证者质量证明失败",
    });
  }
});

// 获取任务的所有验证结果
app.get("/api/tasks/:taskId/proofs", async (req, res) => {
  try {
    const { taskId } = req.params;

    if (!contract) {
      await initNear();
    }

    console.log(`获取任务验证结果请求: 任务ID=${taskId}`);

    const proofs = await contract.get_task_proofs({
      task_id: taskId,
    });

    console.log(`成功获取任务验证结果, 数量: ${proofs.length}`);
    res.json(proofs);
  } catch (error) {
    console.error("获取任务验证结果失败:", error);
    res.status(500).json({
      error: String(error),
      message: "获取任务验证结果失败",
    });
  }
});

// 获取broadcaster的所有任务
app.get("/api/broadcaster/tasks", async (req, res) => {
  try {
    if (!contract) {
      console.log("合约未初始化，正在尝试初始化...");
      await initNear();
    }

    console.log(`获取broadcaster的所有任务`);

    // 调用合约获取broadcaster任务
    const tasks = await contract.get_broadcaster_tasks({
      broadcaster_id: NEAR_CONFIG.broadcasterAccountId,
    });

    console.log(`成功获取${tasks.length}个broadcaster任务`);
    res.json(tasks);
  } catch (error) {
    console.error("获取broadcaster任务失败:", error);
    res.status(500).json({
      error: String(error),
      message: "获取broadcaster任务失败",
    });
  }
});

// 获取单个任务的API
app.get("/api/tasks/:taskId", async (req, res) => {
  try {
    const { taskId } = req.params;

    if (!contract) {
      console.log("合约未初始化，正在尝试初始化...");
      await initNear();
    }

    console.log(`获取任务请求: 任务ID=${taskId}`);

    // 调用合约获取任务
    const task = await contract.get_task({
      task_id: taskId,
    });

    if (task) {
      console.log(`成功获取任务: ${taskId}`);
      res.json(task);
    } else {
      console.log(`任务不存在: ${taskId}`);
      res.status(404).json({
        error: "Task not found",
        message: "任务不存在",
      });
    }
  } catch (error) {
    console.error("获取任务失败:", error);
    res.status(500).json({
      error: String(error),
      message: "获取任务详情失败",
    });
  }
});

// 获取任务的报价列表
app.get("/api/tasks/:taskId/offers", async (req, res) => {
  try {
    const { taskId } = req.params;

    if (!contract) {
      console.log("合约未初始化，正在尝试初始化...");
      await initNear();
    }

    console.log(`获取任务报价请求: 任务ID=${taskId}`);

    // 调用合约获取任务报价
    const offers = await contract.get_task_offers({
      task_id: taskId,
    });

    console.log(`成功获取任务报价, 数量: ${offers ? offers.length : 0}`);
    res.json(offers || []);
  } catch (error) {
    console.error("获取任务报价失败:", error);
    res.status(500).json({
      error: String(error),
      message: "获取任务报价失败",
    });
  }
});

// 获取工作节点信息
app.get("/api/workers/:workerId", async (req, res) => {
  try {
    const { workerId } = req.params;

    if (!contract) {
      console.log("合约未初始化，正在尝试初始化...");
      await initNear();
    }

    console.log(`获取工作节点信息请求: 节点ID=${workerId}`);

    // 调用合约获取工作节点信息
    const workerInfo = await contract.get_worker_info({
      worker_id: workerId,
    });

    if (workerInfo) {
      console.log(`成功获取工作节点信息: ${workerId}`);
      res.json(workerInfo);
    } else {
      console.log(`工作节点不存在: ${workerId}`);
      res.status(404).json({
        error: "Worker not found",
        message: "工作节点不存在或未注册",
      });
    }
  } catch (error) {
    console.error("获取工作节点信息失败:", error);
    res.status(500).json({
      error: String(error),
      message: "获取工作节点信息失败",
    });
  }
});

// 获取工作节点QoS评分详情
app.get("/api/workers/:workerId/qos", async (req, res) => {
  try {
    const { workerId } = req.params;

    if (!contract) {
      console.log("合约未初始化，正在尝试初始化...");
      await initNear();
    }

    console.log(`获取工作节点QoS评分请求: 节点ID=${workerId}`);

    // 调用合约获取工作节点QoS评分详情
    const qosDetails = await contract.get_worker_qos_details({
      worker_id: workerId,
    });

    if (qosDetails) {
      console.log(`成功获取工作节点QoS评分详情: ${workerId}`);
      res.json(qosDetails);
    } else {
      console.log(`工作节点QoS评分详情不存在: ${workerId}`);
      res.json({
        message: "工作节点QoS评分详情不存在",
        worker_id: workerId,
        qos_score: 0.5, // 默认中等分数
      });
    }
  } catch (error) {
    console.error("获取工作节点QoS评分详情失败:", error);
    res.status(500).json({
      error: String(error),
      message: "获取工作节点QoS评分详情失败",
    });
  }
});

// 获取工作节点性能摘要
app.get("/api/workers/:workerId/performance", async (req, res) => {
  try {
    const { workerId } = req.params;

    if (!contract) {
      console.log("合约未初始化，正在尝试初始化...");
      await initNear();
    }

    console.log(`获取工作节点性能摘要请求: 节点ID=${workerId}`);

    // 调用合约获取工作节点性能摘要
    const performanceSummary = await contract.get_worker_performance_summary({
      worker_id: workerId,
    });

    if (performanceSummary) {
      console.log(`成功获取工作节点性能摘要: ${workerId}`);
      res.json(performanceSummary);
    } else {
      console.log(`工作节点性能摘要不存在: ${workerId}`);
      res.status(404).json({
        error: "Performance summary not found",
        message: "工作节点性能摘要不存在",
      });
    }
  } catch (error) {
    console.error("获取工作节点性能摘要失败:", error);
    res.status(500).json({
      error: String(error),
      message: "获取工作节点性能摘要失败",
    });
  }
});

// 获取工作节点每日统计数据
app.get("/api/workers/:workerId/daily-stats", async (req, res) => {
  try {
    const { workerId } = req.params;
    const days = req.query.days ? Number(req.query.days) : 7;

    if (!contract) {
      console.log("合约未初始化，正在尝试初始化...");
      await initNear();
    }

    console.log(
      `获取工作节点每日统计数据请求: 节点ID=${workerId}, 天数=${days}`
    );

    // 调用合约获取工作节点每日统计数据
    const dailyStats = await contract.get_worker_daily_stats({
      worker_id: workerId,
      days,
    });

    if (dailyStats) {
      console.log(`成功获取工作节点每日统计数据: ${workerId}`);
      res.json(dailyStats);
    } else {
      console.log(`工作节点每日统计数据不存在: ${workerId}`);
      res.json([]);
    }
  } catch (error) {
    console.error("获取工作节点每日统计数据失败:", error);
    res.status(500).json({
      error: String(error),
      message: "获取工作节点每日统计数据失败",
    });
  }
});

// 获取任务队列
app.get("/api/tasks/queue", async (req, res) => {
  try {
    if (!contract) {
      console.log("合约未初始化，正在尝试初始化...");
      await initNear();
    }

    console.log(`获取任务队列请求`);

    // 调用合约获取任务队列
    const taskQueue = await contract.get_task_queue();

    console.log(`成功获取任务队列, 数量: ${taskQueue.length}`);
    res.json(taskQueue);
  } catch (error) {
    console.error("获取任务队列失败:", error);
    res.status(500).json({
      error: String(error),
      message: "获取任务队列失败",
    });
  }
});

// 获取QoS共识证明
app.get("/api/consensus/:taskId", async (req, res) => {
  try {
    const { taskId } = req.params;

    if (!contract) {
      console.log("合约未初始化，正在尝试初始化...");
      await initNear();
    }

    console.log(`获取共识证明请求: 任务ID=${taskId}`);

    // 调用合约获取共识证明
    const proof = await contract.get_consensus_proof({
      task_id: taskId,
    });

    if (proof) {
      console.log(`成功获取任务共识证明: ${taskId}`);
      res.json(proof);
    } else {
      console.log(`任务共识证明不存在: ${taskId}`);
      res.json(null);
    }
  } catch (error) {
    console.error("获取共识证明失败:", error);
    res.status(500).json({
      error: String(error),
      message: "获取共识证明失败",
    });
  }
});

// 获取任务共识详情
app.get("/api/tasks/:taskId/consensus-details", async (req, res) => {
  try {
    const { taskId } = req.params;

    if (!contract) {
      console.log("合约未初始化，正在尝试初始化...");
      await initNear();
    }

    console.log(`获取任务共识详情请求: 任务ID=${taskId}`);

    // 调用合约获取任务共识详情
    const consensusDetails = await contract.get_task_consensus_details({
      task_id: taskId,
    });

    if (consensusDetails) {
      console.log(`成功获取任务共识详情: ${taskId}`);
      res.json(consensusDetails);
    } else {
      console.log(`任务共识详情不存在: ${taskId}`);
      res.status(404).json({
        error: "Consensus details not found",
        message: "任务共识详情不存在",
      });
    }
  } catch (error) {
    console.error("获取任务共识详情失败:", error);
    res.status(500).json({
      error: String(error),
      message: "获取任务共识详情失败",
    });
  }
});

// 确保上传目录存在
if (!fs.existsSync("uploads")) {
  fs.mkdirSync("uploads");
}

// 启动服务器
app.listen(PORT, () => {
  console.log(`Broadcaster服务已启动: http://localhost:${PORT}`);
  console.log(`使用账户: ${NEAR_CONFIG.broadcasterAccountId}`);
  console.log(`IPFS配置: ${IPFS_CONFIG.host}:${IPFS_CONFIG.port}`);

  // 启动时初始化NEAR连接
  // 启动时初始化NEAR连接
  initNear().then((success) => {
    if (success) {
      console.log("NEAR连接已初始化");

      // 设置定时任务，每30秒检查一次broadcaster的所有任务offer超时
      setInterval(async () => {
        try {
          console.log("执行定时broadcaster任务offer超时检查");
          await contract.check_broadcaster_offer_timeout({
            broadcaster_id: NEAR_CONFIG.broadcasterAccountId,
          });
          console.log("定时broadcaster任务offer超时检查完成");
        } catch (error) {
          console.error("定时broadcaster任务offer超时检查失败:", error);
        }
      }, 30000); // 每30秒执行一次
    } else {
      console.error("NEAR连接初始化失败，某些功能可能不可用");
    }
  });
});
