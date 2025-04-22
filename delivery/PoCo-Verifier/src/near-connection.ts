// near-connection.ts
import { connect, KeyPair, keyStores, Contract, Account } from "near-api-js";
import {
  TaskData,
  VerifierConfig,
  VerifierQosProof,
  TaskVerificationStatus,
} from "./types";
import * as fs from "fs";
import * as path from "path";
import * as os from "os";
import fetch from "node-fetch";
import { HttpsProxyAgent } from "https-proxy-agent";
import { logger } from "./utils";

export class NearConnection {
  private config: VerifierConfig;
  private nearConnection: any;
  private accountId: string;
  private contractId: string;
  private contract: any;

  constructor(config: VerifierConfig) {
    this.config = config;
    this.accountId = config.verifierAccountId;
    this.contractId = config.contractId;
  }

  /**
   * 初始化NEAR连接
   */
  async initialize(): Promise<boolean> {
    try {
      // 创建代理
      const proxyAgent = new HttpsProxyAgent("http://127.0.0.1:10809");

      // 设置自定义的全局 fetch (不在 connect 配置中传递)
      (global as any).fetch = (url: string, options: any = {}) => {
        return fetch(url, {
          ...options,
          agent: proxyAgent,
          timeout: 30000, // 增加超时时间
        });
      };

      // 配置密钥存储
      const keyStore = this.getKeyStore();

      // 连接到 NEAR (不传递 fetch)
      this.nearConnection = await connect({
        ...this.config.nearConfig,
        keyStore,
        headers: {},
      });

      // 获取账户对象
      const account = await this.nearConnection.account(this.accountId);

      // 初始化合约接口
      this.contract = new Contract(account, this.contractId, {
        // 视图方法 - 不需要签名
        viewMethods: [
          "get_task",
          "get_verifier_members",
          "get_committee_leader",
          "get_verifier_proof",
          "get_task_proofs",
          "get_task_verification_status",
          "get_consensus_proof",
        ],
        // 修改方法 - 需要签名
        changeMethods: [
          "query_assigned_tasks",
          "submit_verifier_proof",
          "request_supplemental_verifier",
        ],
        useLocalViewExecution: false,
      });

      logger.info(`已连接到NEAR网络，合约ID: ${this.contractId}`);
      return true;
    } catch (error) {
      logger.error(`连接NEAR网络失败: ${error}`);
      return false;
    }
  }

  /**
   * 关闭NEAR连接
   */
  async shutdown(): Promise<void> {
    logger.info("关闭NEAR连接");
    // 实际上无需特别关闭，添加此方法是为了保持接口一致性
  }

  /**
   * 获取密钥存储
   */
  private getKeyStore(): keyStores.KeyStore {
    // 创建内存密钥存储
    const keyStore = new keyStores.InMemoryKeyStore();

    try {
      // 从文件读取凭证
      let credentialsPath =
        this.config.credentialsPath ||
        path.join(os.homedir(), ".near-credentials");

      // 如果路径以~开头，替换为用户主目录
      if (credentialsPath.startsWith("~")) {
        credentialsPath = credentialsPath.replace("~", os.homedir());
      }

      const networkPath = path.join(
        credentialsPath,
        this.config.nearConfig.networkId
      );

      logger.info(`尝试加载NEAR凭证目录: ${networkPath}`);

      if (!fs.existsSync(networkPath)) {
        throw new Error(`未找到NEAR凭证目录: ${networkPath}`);
      }

      const credentialsFilePath = path.join(
        networkPath,
        `${this.accountId}.json`
      );
      logger.info(`尝试加载账户凭证文件: ${credentialsFilePath}`);

      if (!fs.existsSync(credentialsFilePath)) {
        throw new Error(`未找到账户凭证文件: ${credentialsFilePath}`);
      }

      const credentials = JSON.parse(
        fs.readFileSync(credentialsFilePath, "utf-8")
      );
      const keyPair = KeyPair.fromString(credentials.private_key);

      // 将密钥添加到存储
      keyStore.setKey(
        this.config.nearConfig.networkId,
        this.accountId,
        keyPair
      );
      logger.info(`已加载账户凭证: ${this.accountId}`);

      return keyStore;
    } catch (error) {
      logger.error(`加载密钥失败: ${error}`);
      throw error;
    }
  }

  /**
   * 查询分配给验证者的任务
   * @param onlyUnverified 是否只返回未验证的任务
   */
  async queryAssignedTasks(
    onlyUnverified: boolean = true
  ): Promise<TaskData[]> {
    try {
      const tasks: TaskData[] = await this.contract.query_assigned_tasks({
        verifier_id: this.accountId,
        only_unverified: onlyUnverified,
      });
      logger.info(`已查询到 ${tasks.length} 个分配给验证者的任务`);
      return tasks;
    } catch (error) {
      logger.error(`查询分配的任务失败: ${error}`);
      return [];
    }
  }

  /**
   * 获取特定任务
   */
  async getTask(taskId: string): Promise<TaskData | null> {
    try {
      const task: TaskData = await this.contract.get_task({ task_id: taskId });
      return task;
    } catch (error) {
      logger.error(`获取任务 ${taskId} 失败: ${error}`);
      return null;
    }
  }

  /**
   * 提交验证者质量证明
   */
  async submitVerifierProof(proof: VerifierQosProof): Promise<boolean> {
    try {
      const result = await this.contract.submit_verifier_proof({ proof });
      logger.info(`提交任务 ${proof.task_id} 的质量证明成功`);
      return result;
    } catch (error) {
      logger.error(`提交任务 ${proof.task_id} 的质量证明失败: ${error}`);
      return false;
    }
  }

  /**
   * 获取特定验证者的验证结果
   */
  async getVerifierProof(
    taskId: string,
    verifierId: string = this.accountId
  ): Promise<VerifierQosProof | null> {
    try {
      const proof = await this.contract.get_verifier_proof({
        task_id: taskId,
        verifier_id: verifierId,
      });
      return proof;
    } catch (error) {
      logger.error(
        `获取验证者 ${verifierId} 对任务 ${taskId} 的验证结果失败: ${error}`
      );
      return null;
    }
  }

  /**
   * 获取任务的所有验证结果
   */
  async getTaskProofs(taskId: string): Promise<VerifierQosProof[]> {
    try {
      const proofs = await this.contract.get_task_proofs({
        task_id: taskId,
      });
      return proofs;
    } catch (error) {
      logger.error(`获取任务 ${taskId} 的所有验证结果失败: ${error}`);
      return [];
    }
  }

  /**
   * 获取任务验证状态
   */
  async getTaskVerificationStatus(
    taskId: string
  ): Promise<TaskVerificationStatus | null> {
    try {
      const status = await this.contract.get_task_verification_status({
        task_id: taskId,
      });
      return status;
    } catch (error) {
      logger.error(`获取任务 ${taskId} 的验证状态失败: ${error}`);
      return null;
    }
  }

  /**
   * 获取共识证明
   */
  async getConsensusProof(taskId: string): Promise<any | null> {
    try {
      const proof = await this.contract.get_consensus_proof({
        task_id: taskId,
      });
      return proof;
    } catch (error) {
      logger.error(`获取任务 ${taskId} 的共识证明失败: ${error}`);
      return null;
    }
  }

  /**
   * 获取委员会Leader信息
   */
  async getCommitteeLeader(): Promise<any | null> {
    try {
      const leader = await this.contract.get_committee_leader({});
      return leader;
    } catch (error) {
      logger.error(`获取委员会Leader信息失败: ${error}`);
      return null;
    }
  }

  /**
   * 请求补充验证者
   */
  async requestSupplementalVerifier(taskId: string): Promise<string | null> {
    try {
      const verifierId = await this.contract.request_supplemental_verifier({
        task_id: taskId,
      });
      return verifierId;
    } catch (error) {
      logger.error(`请求任务 ${taskId} 的补充验证者失败: ${error}`);
      return null;
    }
  }
}

export default NearConnection;
