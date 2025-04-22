// committee-client.ts
import axios, { AxiosInstance } from "axios";
import { VerifierConfig, VerifierQosProof } from "./types";
import { logger } from "./utils";
import { generatePlaceholderSignature } from "./utils";

/**
 * 委员会成员信息
 */
interface CommitteeMember {
  account_id: string;
  ip_address: string;
  port: number;
  is_leader: boolean;
}

/**
 * 委员会通信模块
 * 负责与委员会节点通信，提交验证结果
 */
export class CommitteeClient {
  private config: VerifierConfig;
  private httpClient: AxiosInstance;
  private committeeMembers: CommitteeMember[] = [];
  private lastMembersUpdateTime: number = 0;
  private readonly MEMBERS_UPDATE_INTERVAL: number = 5 * 60 * 1000; // 5分钟更新一次委员会成员列表

  constructor(config: VerifierConfig) {
    this.config = config;

    // 创建HTTP客户端
    this.httpClient = axios.create({
      timeout: 30000, // 30秒超时
      headers: {
        "Content-Type": "application/json",
      },
    });
  }

  /**
   * 初始化委员会客户端
   */
  async initialize(): Promise<boolean> {
    try {
      // 更新委员会成员列表
      await this.updateCommitteeMembers();
      return true;
    } catch (error) {
      logger.error(`初始化委员会客户端失败: ${error}`);
      return false;
    }
  }

  // 添加一个通用的重试函数到CommitteeClient类中
  private async withRetry<T>(
    operation: () => Promise<T>,
    maxRetries: number = 3,
    initialDelay: number = 2000,
    description: string = "操作"
  ): Promise<T> {
    let retries = 0;
    let delay = initialDelay;

    while (true) {
      try {
        return await operation();
      } catch (error) {
        retries++;
        const errorMessage =
          error instanceof Error ? error.message : String(error);

        if (retries >= maxRetries) {
          logger.error(
            `${description}失败，已达到最大重试次数(${maxRetries}): ${errorMessage}`
          );
          throw error; // 重试次数用尽，抛出错误
        }

        logger.warn(
          `${description}失败，第${retries}/${maxRetries}次重试，等待${delay}ms: ${errorMessage}`
        );

        // 等待后重试
        await new Promise((resolve) => setTimeout(resolve, delay));

        // 指数退避策略，每次重试增加延迟时间
        delay = Math.min(delay * 1.5, 30000); // 最大延迟30秒
      }
    }
  }

  /**
   * 更新委员会成员列表
   * 从区块链获取最新的委员会成员信息
   */
  private async updateCommitteeMembers(): Promise<void> {
    // 检查是否需要更新
    const now = Date.now();
    if (
      now - this.lastMembersUpdateTime < this.MEMBERS_UPDATE_INTERVAL &&
      this.committeeMembers.length > 0
    ) {
      return;
    }

    try {
      // 这里应该通过near-connection获取委员会成员列表
      // 为简化实现，先使用固定的成员列表
      // TODO: 实际项目中，应该从near-connection获取
      this.committeeMembers = [
        {
          account_id: "leader",
          ip_address: "10.24.136.124",
          port: 9000,
          is_leader: true,
        },
        {
          account_id: "follower1",
          ip_address: "10.24.216.33",
          port: 9000,
          is_leader: false,
        },
        {
          account_id: "follower2",
          ip_address: "10.24.198.225",
          port: 9000,
          is_leader: false,
        },
        {
          account_id: "follower3",
          ip_address: "10.24.161.186",
          port: 9000,
          is_leader: false,
        },
      ];

      this.lastMembersUpdateTime = now;
      logger.info(
        `已更新委员会成员列表，共 ${this.committeeMembers.length} 个成员`
      );
    } catch (error) {
      logger.error(`更新委员会成员列表失败: ${error}`);
      throw error;
    }
  }

  /**
   * 将验证结果发送给所有委员会成员
   * @param proof 验证证明
   * @returns 是否成功发送给至少一个委员会成员
   */
  async sendProofToCommittee(proof: VerifierQosProof): Promise<boolean> {
    try {
      // 更新委员会成员列表
      await this.updateCommitteeMembers();

      if (this.committeeMembers.length === 0) {
        logger.error("委员会成员列表为空，无法发送验证结果");
        return false;
      }

      // 转换为委员会API格式的证明
      // const apiProof = this.convertToApiProofFormat(proof);

      // 找出leader节点
      const leader = this.committeeMembers.find((m) => m.is_leader);
      const followers = this.committeeMembers.filter((m) => !m.is_leader);

      if (!leader) {
        logger.warn("找不到leader节点，将同时向所有节点发送");
        // 如果找不到leader，按原来的方式发送
        const sendPromises = this.committeeMembers.map((member) =>
          this.sendProofToMember(member, proof)
        );
        const results = await Promise.allSettled(sendPromises);
        const successCount = results.filter(
          (r) => r.status === "fulfilled" && r.value
        ).length;
        return successCount > 0;
      }

      // 首先向leader发送
      logger.info(`首先向leader节点 ${leader.account_id} 发送验证结果`);
      const leaderResult = await this.sendProofToMember(leader, proof);

      if (!leaderResult) {
        logger.error(`向leader发送失败，转为向所有节点同时发送`);
        // 如果向leader发送失败，向所有节点发送
        const sendPromises = this.committeeMembers.map((member) =>
          this.sendProofToMember(member, proof)
        );
        const results = await Promise.allSettled(sendPromises);
        const successCount = results.filter(
          (r) => r.status === "fulfilled" && r.value
        ).length;
        return successCount > 0;
      }

      // leader发送成功后，等待一段时间再向其他节点发送
      const LEADER_PROCESSING_DELAY = 2000; // 2秒延迟
      logger.info(
        `Leader已接收消息，等待 ${LEADER_PROCESSING_DELAY}ms 让leader处理...`
      );
      await new Promise((resolve) =>
        setTimeout(resolve, LEADER_PROCESSING_DELAY)
      );

      // 向其他节点发送
      logger.info(`开始向 ${followers.length} 个follower节点发送验证结果`);
      const sendPromises = followers.map((member) =>
        this.sendProofToMember(member, proof)
      );

      // 等待所有follower发送完成
      const results = await Promise.allSettled(sendPromises);

      // 统计成功发送的数量（包括leader）
      const followerSuccessCount = results.filter(
        (r) => r.status === "fulfilled" && r.value
      ).length;
      const totalSuccessCount = leaderResult
        ? followerSuccessCount + 1
        : followerSuccessCount;
      const totalCount = this.committeeMembers.length;

      logger.info(
        `验证结果已发送给 ${totalSuccessCount}/${totalCount} 个委员会成员 (包括leader)`
      );

      // 只要有一个成功发送，就认为成功
      return totalSuccessCount > 0;
    } catch (error) {
      logger.error(`发送验证结果失败: ${error}`);
      return false;
    }
  }

  /**
   * 将验证结果发送给特定委员会成员
   * @param member 委员会成员信息
   * @param proof 验证证明
   * @returns 是否发送成功
   */
  private async sendProofToMember(
    member: CommitteeMember,
    proof: any
  ): Promise<boolean> {
    return this.withRetry(
      async () => {
        const url = `http://${member.ip_address}:${member.port}/proof`;

        logger.debug(
          `正在向委员会成员发送验证结果: ${member.account_id} (${url})`
        );

        const response = await this.httpClient.post(url, proof, {
          timeout: 10000, // 单个请求10秒超时
        });

        if (response.status === 202) {
          logger.info(`验证结果已成功发送给委员会成员: ${member.account_id}`);
          return true;
        } else {
          logger.warn(
            `向委员会成员发送验证结果失败: ${member.account_id}, 状态码: ${response.status}`
          );
          throw new Error(`发送失败，状态码: ${response.status}`);
        }
      },
      2, // 最大重试2次
      1000, // 初始延迟1秒
      `向成员 ${member.account_id} 发送验证结果`
    ).catch((error) => {
      logger.warn(
        `向委员会成员发送验证结果最终失败: ${member.account_id}, 错误: ${error}`
      );
      return false; // 重试失败后返回false
    });
  }

  /**
   * 查询任务验证状态
   * @param taskId 任务ID
   * @returns 任务状态，如果查询失败则返回null
   */
  async getTaskStatus(taskId: string): Promise<any | null> {
    return this.withRetry(
      async () => {
        // 原有代码...
        await this.updateCommitteeMembers();

        if (this.committeeMembers.length === 0) {
          logger.error("委员会成员列表为空，无法查询任务状态");
          return null;
        }

        // 优先从Leader查询
        const leader = this.committeeMembers.find((m) => m.is_leader);
        if (leader) {
          try {
            const status = await this.getTaskStatusFromMember(leader, taskId);
            if (status) {
              return status;
            }
          } catch (error) {
            logger.warn(
              `从Leader查询任务状态失败，尝试其他委员会成员: ${error}`
            );
          }
        }

        // 如果Leader查询失败，尝试其他成员
        for (const member of this.committeeMembers.filter(
          (m) => !m.is_leader
        )) {
          try {
            const status = await this.getTaskStatusFromMember(member, taskId);
            if (status) {
              return status;
            }
          } catch (error) {
            continue;
          }
        }

        // 如果所有查询都失败，抛出错误触发重试
        throw new Error(`无法从任何委员会成员获取任务 ${taskId} 的状态`);
      },
      2, // 最大重试2次
      1000, // 初始延迟1秒
      "查询任务状态"
    ).catch((error) => {
      logger.error(`查询任务状态最终失败: ${error}`);
      return null; // 即使重试失败，我们也返回null而不是抛出错误
    });
  }

  /**
   * 从特定委员会成员查询任务状态
   */
  /**
   * 从特定委员会成员查询任务状态，带有重试机制
   * @param member 委员会成员信息
   * @param taskId 任务ID
   * @returns 任务状态，或null（如果查询失败）
   */
  private async getTaskStatusFromMember(
    member: CommitteeMember,
    taskId: string
  ): Promise<any | null> {
    return this.withRetry(
      async () => {
        const url = `http://${member.ip_address}:${member.port}/proof/${taskId}/status`;

        logger.debug(
          `正在从委员会成员查询任务状态: ${member.account_id} (${url})`
        );

        const response = await this.httpClient.get(url, {
          timeout: 5000, // 单个请求5秒超时
        });

        if (response.status === 200 && response.data) {
          logger.debug(`已从委员会成员获取任务状态: ${member.account_id}`);
          return response.data;
        } else {
          logger.warn(
            `从委员会成员获取任务状态失败: ${member.account_id}, 状态码: ${response.status}`
          );
          throw new Error(`获取状态失败，状态码: ${response.status}`);
        }
      },
      2, // 最大重试2次
      1000, // 初始延迟1秒
      `从成员 ${member.account_id} 查询任务状态`
    ).catch((error) => {
      logger.warn(
        `从委员会成员获取任务状态最终失败: ${member.account_id}, 错误: ${error}`
      );
      return null; // 重试失败后返回null
    });
  }

  /**
   * 将验证结果转换为委员会API格式
   * 这个转换函数适配了验证者的VerifierQosProof格式到委员会API需要的格式
   */
  private convertToApiProofFormat(proof: VerifierQosProof): any {
    return {
      taskId: proof.task_id,
      verifierId: proof.verifier_id,
      timestamp: proof.timestamp,
      // 修改为mediaSpecs
      mediaSpecs: {
        codec: proof.video_specs.codec,
        resolution: proof.video_specs.resolution,
        bitrate: proof.video_specs.bitrate,
        framerate: proof.video_specs.framerate,
      },
      // 修改视频质量数据结构
      videoQualityData: {
        overallScore: proof.video_score,
        gopScores: proof.gop_scores.map((gop) => ({
          timestamp: gop.timestamp,
          vmafScore: gop.vmaf_score,
          hash: gop.hash,
        })),
      },
      // 添加其他必要字段
      signature: proof.signature,
    };
  }

  /**
   * 将补充验证结果发送给特定委员会成员，带有重试机制
   * @param member 委员会成员信息
   * @param taskId 任务ID
   * @param proof 验证证明
   * @returns 是否发送成功
   */
  private async sendSupplementaryProofToMember(
    member: CommitteeMember,
    taskId: string,
    proof: any
  ): Promise<boolean> {
    return this.withRetry(
      async () => {
        const url = `http://${member.ip_address}:${member.port}/proof/${taskId}/supplementary`;

        logger.debug(
          `正在向委员会成员发送补充验证结果: ${member.account_id} (${url})`
        );

        const response = await this.httpClient.post(url, proof, {
          timeout: 10000, // 10秒超时
        });

        if (response.status === 202) {
          logger.info(
            `补充验证结果已成功发送给委员会成员: ${member.account_id}`
          );
          return true;
        } else {
          logger.warn(
            `向委员会成员发送补充验证结果失败: ${member.account_id}, 状态码: ${response.status}`
          );
          throw new Error(`发送失败，状态码: ${response.status}`);
        }
      },
      3, // 最大重试3次
      2000, // 初始延迟2秒
      `向成员 ${member.account_id} 发送补充验证结果`
    ).catch((error) => {
      logger.warn(
        `向委员会成员发送补充验证结果最终失败: ${member.account_id}, 错误: ${error}`
      );
      return false; // 重试失败后返回false
    });
  }

  /**
   * 发送补充验证结果
   * 在出现验证结果冲突时使用
   */
  async sendSupplementaryProof(
    taskId: string,
    proof: VerifierQosProof
  ): Promise<boolean> {
    try {
      // 更新委员会成员列表
      await this.updateCommitteeMembers();

      if (this.committeeMembers.length === 0) {
        logger.error("委员会成员列表为空，无法发送补充验证结果");
        return false;
      }

      // 转换为委员会API格式的证明
      // const apiProof = this.convertToApiProofFormat(proof);

      // 找出leader节点
      const leader = this.committeeMembers.find((m) => m.is_leader);
      const followers = this.committeeMembers.filter((m) => !m.is_leader);

      if (!leader) {
        logger.warn("找不到leader节点，将同时向所有节点发送补充验证结果");
        // 如果找不到leader，按原来的方式发送
        const sendPromises = this.committeeMembers.map((member) =>
          this.sendSupplementaryProofToMember(member, taskId, proof)
        );
        const results = await Promise.allSettled(sendPromises);
        const successCount = results.filter(
          (r) => r.status === "fulfilled" && r.value
        ).length;
        return successCount > 0;
      }

      // 首先向leader发送
      logger.info(`首先向leader节点 ${leader.account_id} 发送补充验证结果`);
      const leaderResult = await this.sendSupplementaryProofToMember(
        leader,
        taskId,
        proof
      );

      if (!leaderResult) {
        logger.error(`向leader发送补充验证结果失败，转为向所有节点同时发送`);
        // 如果向leader发送失败，向所有节点发送
        const sendPromises = this.committeeMembers.map((member) =>
          this.sendSupplementaryProofToMember(member, taskId, proof)
        );
        const results = await Promise.allSettled(sendPromises);
        const successCount = results.filter(
          (r) => r.status === "fulfilled" && r.value
        ).length;
        return successCount > 0;
      }

      // leader发送成功后，等待一段时间再向其他节点发送
      const LEADER_PROCESSING_DELAY = 2000; // 2秒延迟
      logger.info(
        `Leader已接收补充验证结果，等待 ${LEADER_PROCESSING_DELAY}ms 让leader处理...`
      );
      await new Promise((resolve) =>
        setTimeout(resolve, LEADER_PROCESSING_DELAY)
      );

      // 向其他节点发送
      logger.info(`开始向 ${followers.length} 个follower节点发送补充验证结果`);
      const sendPromises = followers.map((member) =>
        this.sendSupplementaryProofToMember(member, taskId, proof)
      );

      // 等待所有follower发送完成
      const results = await Promise.allSettled(sendPromises);

      // 统计成功发送的数量（包括leader）
      const followerSuccessCount = results.filter(
        (r) => r.status === "fulfilled" && r.value
      ).length;
      const totalSuccessCount = leaderResult
        ? followerSuccessCount + 1
        : followerSuccessCount;
      const totalCount = this.committeeMembers.length;

      logger.info(
        `补充验证结果已发送给 ${totalSuccessCount}/${totalCount} 个委员会成员 (包括leader)`
      );

      // 只要有一个成功发送，就认为成功
      return totalSuccessCount > 0;
    } catch (error) {
      logger.error(`发送补充验证结果失败: ${error}`);
      return false;
    }
  }

  /**
   * 批量发送验证结果
   * 当有多个任务需要同时验证时使用
   */
  // async sendProofsBatch(proofs: VerifierQosProof[]): Promise<boolean> {
  //   try {
  //     // 更新委员会成员列表
  //     await this.updateCommitteeMembers();

  //     if (this.committeeMembers.length === 0) {
  //       logger.error("委员会成员列表为空，无法发送批量验证结果");
  //       return false;
  //     }

  //     // 转换为委员会API格式的证明
  //     const apiProofs = proofs.map((proof) =>
  //       this.convertToApiProofFormat(proof)
  //     );

  //     // 向所有委员会成员发送批量验证结果
  //     const sendPromises = this.committeeMembers.map((member) =>
  //       this.sendProofsBatchToMember(member, apiProofs)
  //     );

  //     // 等待所有发送完成
  //     const results = await Promise.allSettled(sendPromises);

  //     // 统计成功发送的数量
  //     const successCount = results.filter(
  //       (r) => r.status === "fulfilled" && r.value
  //     ).length;
  //     const totalCount = this.committeeMembers.length;

  //     logger.info(
  //       `批量验证结果已发送给 ${successCount}/${totalCount} 个委员会成员`
  //     );

  //     // 只要有一个成功发送，就认为成功
  //     return successCount > 0;
  //   } catch (error) {
  //     logger.error(`发送批量验证结果失败: ${error}`);
  //     return false;
  //   }
  // }

  // /**
  //  * 向特定委员会成员发送批量验证结果
  //  */
  // private async sendProofsBatchToMember(
  //   member: CommitteeMember,
  //   proofs: any[]
  // ): Promise<boolean> {
  //   try {
  //     const url = `http://${member.ip_address}:${member.port}/proofs/batch`;

  //     logger.debug(
  //       `正在向委员会成员发送批量验证结果: ${member.account_id} (${url})`
  //     );

  //     const response = await this.httpClient.post(url, proofs, {
  //       timeout: 20000, // 批量请求20秒超时
  //     });

  //     if (response.status === 202) {
  //       logger.info(`批量验证结果已成功发送给委员会成员: ${member.account_id}`);
  //       return true;
  //     } else {
  //       logger.warn(
  //         `向委员会成员发送批量验证结果失败: ${member.account_id}, 状态码: ${response.status}`
  //       );
  //       return false;
  //     }
  //   } catch (error) {
  //     logger.warn(
  //       `向委员会成员发送批量验证结果时出错: ${member.account_id}, 错误: ${error}`
  //     );
  //     return false;
  //   }
  // }
}

export default CommitteeClient;
