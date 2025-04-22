import { PBFTEngine } from './PBFTEngine';
import { MessageHandler } from '../network/MessageHandler';
import { QoSValidator } from '../validators/QoSValidator';
import {
  PBFTMessage,
  QoSProof,
  MessageType,
  TaskProcessingState,
  TaskStatus,
  ConsensusType,
  SupplementaryMessage,
  SupplementaryReadyMessage,
  SupplementaryAckMessage,
  EventType,
  GopScore,
  TaskData,
  QosProofStatus,
  ConsensusQosProof,
  GopVerificationResult,
  VerifierQosProof,
} from '../models/types';
import { logger } from '../utils/logger';
import { calculateHash, sign } from '../utils/crypto';
import { GlobalMetricsCollector } from '../service/GlobalMetricsCollector';
import { NearConnectionLeader } from '../near-connection-leader';

export class CommitteeNode {
  private nodeId: string;
  private pbftEngine: PBFTEngine;
  private messageHandler: MessageHandler;
  private qosValidator: QoSValidator;
  private isLeader: boolean;
  private taskStatuses: Map<string, TaskStatus> = new Map();
  private taskProofs: Map<string, VerifierQosProof[]> = new Map();
  private consensusQueue: string[] = []; // 存储等待共识的任务ID
  private processingConsensus: boolean = false; // 标记是否正在处理共识
  private privateKey: string = 'private_key';
  private currentConsensusTaskId: string | null = null;
  private pendingPrePrepareMessages: Map<string, PBFTMessage> = new Map();
  private supplementaryReadyStatus: Map<string, Set<string>> = new Map(); // taskId -> 已就绪的节点集合
  private pendingSupplementaryConsensus: Map<string, any> = new Map(); // 等待启动共识的任务
  private pendingFinalPrePrepareMessages: Map<string, PBFTMessage> = new Map();
  // 在 CommitteeNode 类中添加以下属性
  private nearConnection: NearConnectionLeader | null = null;

  constructor(
    nodeId: string,
    port: number,
    isLeader: boolean,
    peers: string[],
    totalNodes: number
  ) {
    this.nodeId = nodeId;
    this.isLeader = isLeader;

    // 初始化QoS验证器
    this.qosValidator = new QoSValidator();

    // 初始化PBFT引擎，并设置共识回调
    this.pbftEngine = new PBFTEngine(
      nodeId,
      isLeader,
      totalNodes,
      this.onConsensusReached.bind(this)
    );

    // 初始化消息处理器
    this.messageHandler = new MessageHandler(
      nodeId,
      port,
      peers,
      this.onMessageReceived.bind(this),
      this.handleSupplementaryMessage.bind(this)
    );

    // 如果是Leader节点，初始化NEAR连接
    if (this.isLeader) {
      this.initializeNearConnection();
    }
  }

  // 添加 NEAR 连接初始化方法
  private async initializeNearConnection() {
    try {
      const config = require('../committee-config').default;

      console.log(config);

      if (!config) {
        logger.warn('缺少委员会配置，无法初始化NEAR连接');
        return;
      }

      this.nearConnection = new NearConnectionLeader(config);
      const success = await this.nearConnection.initialize();

      if (success) {
        logger.info('Leader节点已成功连接到NEAR网络');
      } else {
        logger.error('Leader节点连接NEAR网络失败');
      }
    } catch (error) {
      logger.error(`初始化NEAR连接失败: ${error}`);
    }
  }

  public getNodeId(): string {
    return this.nodeId;
  }

  // 启动节点
  public start(): void {
    logger.info(`委员会节点 ${this.nodeId} 正在启动...`);
    this.messageHandler.start();
    logger.info(`委员会节点 ${this.nodeId} 成功启动`);
  }

  // 停止节点
  public stop(): void {
    logger.info(`委员会节点 ${this.nodeId} 正在停止...`);
    this.messageHandler.stop();
    logger.info(`委员会节点 ${this.nodeId} 已停止`);
  }

  private storeProof(taskId: string, proof: VerifierQosProof): void {
    let proofs = this.taskProofs.get(taskId) || [];
    proofs.push(proof);
    this.taskProofs.set(taskId, proofs);
  }

  private getProofsForTask(taskId: string): VerifierQosProof[] {
    return this.taskProofs.get(taskId) || [];
  }

  private broadcastRejection(taskId: string, reason: any): void {
    const data = {
      taskId,
      status: TaskProcessingState.Rejected,
      reason,
    };

    const rejectMessage: PBFTMessage = {
      taskId: taskId,
      type: MessageType.StatusUpdate,
      consensusType: ConsensusType.Normal,
      viewNumber: this.pbftEngine.getCurrentViewNumber(),
      sequenceNumber: this.pbftEngine.getNextSequenceNumber(),
      nodeId: this.nodeId,
      data: data,
      digest: calculateHash(data),
      signature: sign(data, this.privateKey),
    };

    const metricsCollector = GlobalMetricsCollector.getInstance();
    // 记录PREPARE_SENT
    metricsCollector.recordEvent({
      taskId: taskId,
      nodeId: this.nodeId,
      eventType: EventType.PROOF_REJECTED,
      timestamp: Date.now(),
      // metadata: { verifierId: proof.verifierId },
    });
    this.messageHandler.broadcast(rejectMessage);
  }

  // 处理补充证明相关消息的入口方法
  private handleSupplementaryMessage(message: SupplementaryMessage): void {
    switch (message.type) {
      case 'SupplementaryReady':
        this.handleSupplementaryReadyMessage(message);
        break;
      case 'SupplementaryAck':
        this.handleSupplementaryAckMessage(message);
        break;
      default:
        logger.warn(`节点 ${this.nodeId} 收到未知类型的补充证明消息`);
    }
  }

  // 处理收到的消息
  private onMessageReceived(message: PBFTMessage): void {
    logger.info(`节点 ${this.nodeId} 收到来自 ${message.nodeId} 的 ${message.type}消息`);

    // 直接从消息中获取taskId
    const taskId = message.taskId;

    if (this.currentConsensusTaskId && taskId !== this.currentConsensusTaskId) {
      logger.info(`节点 ${this.nodeId} 忽略非当前共识任务 ${taskId} 的消息`);
      return;
    }

    let response: PBFTMessage | null = null;

    switch (message.type) {
      case MessageType.PrePrepare:
        // console.log(`${this.nodeId}接收到了preprepare`);
        response = this.processPrePrepareMessage(message);
        // console.warn(`${this.nodeId}的processPrePrepare返回值在这里`);
        // console.log(response);
        if (response) {
          // 当前response是Prepare消息
          logger.warn(`${this.nodeId} 广播了 PPPPPPrepare 信令`);
          this.messageHandler.broadcast(response);

          const metricsCollector = GlobalMetricsCollector.getInstance();
          // 记录PREPARE_SENT
          metricsCollector.recordEvent({
            taskId: taskId,
            nodeId: this.nodeId,
            eventType: EventType.PREPARE_SENT,
            timestamp: Date.now(),
            // metadata: { verifierId: proof.verifierId },
          });

          // 然后处理自己的Prepare消息
          // 这里我们将自己的Prepare消息作为"新收到的消息"来处理
          // console.warn(`${this.nodeId}开始处理自己的PrePare消息`);
          const selfPrepareMsg = { ...response }; // 复制一份避免引用问题
          const commitMsg = this.pbftEngine.handlePrepare(selfPrepareMsg);

          // 如果生成了Commit消息，也广播它
          if (commitMsg) {
            logger.warn(`${this.nodeId} 在 onMessageReceived 广播了 Commit 信令`);
            // const metricsCollector = GlobalMetricsCollector.getInstance();
            // 记录PREPARE_SENT
            metricsCollector.recordEvent({
              taskId: taskId,
              nodeId: this.nodeId,
              eventType: EventType.COMMIT_SENT,
              timestamp: Date.now(),
              // metadata: { verifierId: proof.verifierId },
            });
            this.messageHandler.broadcast(commitMsg);
          }
        }
        break;

      case MessageType.Prepare:
        response = this.pbftEngine.handlePrepare(message);
        break;

      case MessageType.Commit:
        this.pbftEngine.handleCommit(message);
        break;

      default:
        logger.warn(`收到未知类型的消息: ${message.type}`);
        return;
    }

    // 如果有响应消息（且不是PrePrepare情况，因为已经处理过了）
    if (response && message.type !== MessageType.PrePrepare) {
      logger.warn(`${this.nodeId} 在onMessageReceived 最后 广播了 ${response.type} 信令`);
      if (message.type === MessageType.Prepare) {
        const metricsCollector = GlobalMetricsCollector.getInstance();
        // 记录PREPARE_SENT
        metricsCollector.recordEvent({
          taskId: taskId,
          nodeId: this.nodeId,
          eventType: EventType.PREPARE_SENT,
          timestamp: Date.now(),
          // metadata: { verifierId: proof.verifierId },
        });
      } else if (message.type === MessageType.Commit) {
        const metricsCollector = GlobalMetricsCollector.getInstance();
        // 记录PREPARE_SENT
        metricsCollector.recordEvent({
          taskId: taskId,
          nodeId: this.nodeId,
          eventType: EventType.COMMIT_SENT,
          timestamp: Date.now(),
          // metadata: { verifierId: proof.verifierId },
        });
      }

      this.messageHandler.broadcast(response);
    }
  }

  // 添加队列管理方法
  private enqueueConsensusTask(
    taskId: string,
    proof: VerifierQosProof,
    consensusType: ConsensusType
  ): void {
    // 将任务ID添加到队列
    this.consensusQueue.push(taskId);
    logger.info(`任务 ${taskId} 加入共识队列，当前队列长度: ${this.consensusQueue.length}`);
    console.warn(`inside enqueueConsensusTask 任务进入共识队列，当前时间${Date.now()}`);

    // 存储任务相关信息以便后续处理
    // this.taskProofs.set(taskId, [proof, ...this.getProofsForTask(taskId)]);

    // 如果不在处理中，则开始处理队列
    if (!this.processingConsensus) {
      this.processConsensusQueue();
    }
  }

  // 处理共识队列
  private async processConsensusQueue(): Promise<void> {
    console.log('inside processConsensusQueue');
    console.log('开始执行第一个任务');
    if (this.processingConsensus || this.consensusQueue.length === 0) {
      return;
    }

    // 标记为处理中
    this.processingConsensus = true;

    // 获取队列中的第一个任务
    const taskId = this.consensusQueue[0];
    const status = this.taskStatuses.get(taskId);

    // 如果任务正在等待补充验证，将其移出队列并处理下一个
    if (status && status.state === TaskProcessingState.AwaitingSupplementary) {
      this.consensusQueue.shift();
      this.processingConsensus = false;
      return this.processConsensusQueue();
    }

    if (!status || status.state !== TaskProcessingState.Consensus) {
      // 任务状态已变更，移除并处理下一个
      this.consensusQueue.shift();
      this.processingConsensus = false;
      this.processConsensusQueue();
      return;
    }

    // 获取任务证明
    const proofs = this.getProofsForTask(taskId);
    if (proofs.length === 0) {
      logger.warn(`队列中的任务 ${taskId} 没有有效的证明`);
      this.consensusQueue.shift();
      this.processingConsensus = false;
      this.processConsensusQueue();
      return;
    }

    // 确定共识类型
    let consensusType = ConsensusType.Normal;
    if (status.validationInfo?.conflictType) {
      consensusType = ConsensusType.Conflict;
    }

    // 这时leader确认currentConsensusTaskId
    this.currentConsensusTaskId = taskId;

    logger.info(`开始处理队列中的任务 ${taskId}，共识类型: ${consensusType}`);

    // 启动共识流程
    if (this.isLeader) {
      // 获取全局收集器实例
      const metricsCollector = GlobalMetricsCollector.getInstance();

      // 记录接收到QoSProof证明事件
      metricsCollector.recordConsensusEvent({
        taskId: taskId,
        nodeId: this.nodeId,
        eventType: EventType.CONSENSUS_STARTED,
        timestamp: Date.now(),
        metadata: { consensusType },
        // 此项指标等待填补
        ConsensusResult: {},
      });

      console.warn(`inside processConsensusQueue 任务启动共识流程，当前时间${Date.now()}`);
      const prePrepareMsg = this.pbftEngine.startConsensus(proofs[0], consensusType);

      if (prePrepareMsg) {
        const metricsCollector = GlobalMetricsCollector.getInstance();
        // 记录PREPARE_SENT
        metricsCollector.recordEvent({
          taskId: taskId,
          nodeId: this.nodeId,
          eventType: EventType.PREPREPARE_SENT,
          timestamp: Date.now(),
          // metadata: { verifierId: proof.verifierId },
        });
        this.messageHandler.broadcast(prePrepareMsg);
        logger.info(`Leader 已启动任务 ${taskId} 的${consensusType}共识流程`);

        // Leader也需要响应PrePrepare消息
        const prepareMsg = this.pbftEngine.handlePrePrepare(prePrepareMsg);
        if (prepareMsg) {
          const metricsCollector = GlobalMetricsCollector.getInstance();
          // 记录PREPARE_SENT
          metricsCollector.recordEvent({
            taskId: taskId,
            nodeId: this.nodeId,
            eventType: EventType.PREPARE_SENT,
            timestamp: Date.now(),
            // metadata: { verifierId: proof.verifierId },
          });
          // 这里是为了把leader的prepareMsg发出来，要不然别的节点拿不到了
          this.messageHandler.broadcast(prepareMsg);

          logger.info(`Leader ${this.nodeId} 响应自己的PrePrepare消息，广播Prepare信令`);
        }
      } else {
        logger.warn(`无法为任务 ${taskId} 启动共识流程`);
        status.state = TaskProcessingState.Validating;
        this.taskStatuses.set(taskId, status);

        // 移除当前任务并处理下一个
        this.consensusQueue.shift();
        this.processingConsensus = false;
        this.processConsensusQueue();
      }
    }
  }

  // 处理从Verifier收到的QoS证明
  // 处理从Verifier收到的QoS证明 - 异步版本
  public async handleQoSProof(proof: VerifierQosProof): Promise<void> {
    // 返回一个Promise，包装原始处理逻辑
    return new Promise<void>(resolve => {
      // 使用setImmediate将处理逻辑放入下一个事件循环
      setImmediate(async () => {
        try {
          const { task_id, verifier_id, timestamp } = proof;
          console.log('inside handleQoSProof');
          console.log('收集到proof');
          console.log(proof);
          const now = new Date().toISOString();
          const metricsCollector = GlobalMetricsCollector.getInstance();

          // 记录接收到QoSProof证明事件
          metricsCollector.recordEvent({
            taskId: proof.task_id,
            nodeId: this.nodeId,
            eventType: EventType.PROOF_RECEIVED,
            timestamp: Date.now(),
            metadata: { verifierId: proof.verifier_id },
          });

          // 首先对每个证明进行快速验证
          logger.info(`${this.nodeId}对任务${proof.task_id}执行快速验证`);
          const validationResult = this.qosValidator.quickValidate(proof);
          if (!validationResult.isValid) {
            logger.warn(
              `${this.nodeId}: 任务 ${task_id} 的QoS证明快速验证失败: ${JSON.stringify(validationResult.details)}`
            );
            // 验证失败仅忽略该证明，不影响任务状态
            resolve(); // 即使验证失败也解析Promise
            return;
          }

          // 获取或创建任务状态
          let status = this.taskStatuses.get(task_id);
          if (!status) {
            status = {
              taskId: task_id,
              state: TaskProcessingState.Pending,
              proofCount: 0,
              verifierIds: [],
              createdAt: now,
              updatedAt: now,
            };
            logger.info(`${this.nodeId}创建新任务状态: ${task_id}`);
          }

          // 检查当前verifier是否已经提交过这个证明
          if (!status.verifierIds.includes(verifier_id)) {
            // 存储验证通过的QoS证明
            this.storeProof(task_id, proof);

            // 更新任务状态
            status.proofCount++;
            status.verifierIds.push(verifier_id);
            status.updatedAt = now;

            logger.info(
              `${this.nodeId}: 任务 ${task_id} 收到来自 ${verifier_id} 的QoS证明,当前共 ${status.proofCount} 个证明`
            );

            console.log('当前qosProof');
            console.log(proof);

            // 处理Leader节点的验证和共识逻辑
            if (this.isLeader) {
              // 第一个证明到达时，更新状态为Validating
              if (status.state === TaskProcessingState.Pending) {
                status.state = TaskProcessingState.Validating;
              }

              // 收集到足够的证明(>=2)时启动共识
              if (status.proofCount >= 2 && status.state !== TaskProcessingState.Consensus) {
                // 获取所有证明
                const proofs = this.getProofsForTask(task_id);

                // 验证是否有足够的有效证明启动共识
                if (proofs.length >= 2) {
                  logger.info(
                    `Leader: 任务 ${task_id} 已收集足够证明(${proofs.length}个)，准备启动共识流程`
                  );

                  // 使用深度验证确定共识类型
                  let consensusType = ConsensusType.Normal;
                  const deepValidationResult = this.qosValidator.deepValidate(proofs);
                  if (
                    deepValidationResult &&
                    !deepValidationResult.isValid &&
                    deepValidationResult.hasConflict
                  ) {
                    // 分析冲突类型
                    deepValidationResult.conflictType =
                      this.qosValidator.analyzeConflictType(deepValidationResult);
                    logger.info(
                      `Leader: 任务 ${task_id} 的证明深度验证不通过，冲突类型: ${deepValidationResult.conflictType}, 详情: ${JSON.stringify(deepValidationResult.details)}`
                    );

                    metricsCollector.recordValidationEvent({
                      taskId: proof.task_id,
                      nodeId: this.nodeId,
                      eventType: EventType.PROOF_CONFLICT,
                      timestamp: Date.now(),
                      metadata: { verifierId: proof.verifier_id },
                      validationResult: deepValidationResult,
                    });

                    consensusType = ConsensusType.Conflict;
                    status.state = TaskProcessingState.Conflict;
                    if (!status.validationInfo) {
                      status.validationInfo = {};
                    }

                    // 将冲突类型存储在任务状态中，用于后续处理
                    status.validationInfo.conflictType = deepValidationResult.conflictType;
                    status.validationInfo.conflictDetails = deepValidationResult.details;
                  } else {
                    metricsCollector.recordValidationEvent({
                      taskId: proof.task_id,
                      nodeId: this.nodeId,
                      eventType: EventType.PROOF_VALIDATED,
                      timestamp: Date.now(),
                      metadata: { verifierId: proof.verifier_id },
                      validationResult: deepValidationResult,
                    });
                  }

                  // 更新任务状态为Consensus
                  status.state = TaskProcessingState.Consensus;
                  this.taskStatuses.set(task_id, status);

                  logger.info(`任务 ${task_id} 已加入共识队列`);

                  // 将任务加入共识队列 - 注意这里可能需要改造为异步
                  await this.enqueueConsensusTask(task_id, proofs[0], consensusType);
                }
              }
            } else {
              // 非Leader节点只负责收集证明和更新状态
              if (status.state === TaskProcessingState.Pending) {
                status.state = TaskProcessingState.Validating;
              }

              // 检查是否有待处理的PrePrepare消息需要处理
              await this.checkPendingPrePrepareMessages(task_id);
            }
          } else {
            logger.info(`任务 ${task_id} 已经收到来自 ${verifier_id} 的重复QoS证明，已忽略`);
          }

          // 更新任务状态
          this.taskStatuses.set(task_id, status);

          // 成功完成处理
          resolve();
        } catch (error) {
          // 记录错误但不中断处理流程
          logger.error(`处理QoS证明时出错: ${error}`);
          resolve(); // 即使出错也解析Promise，避免挂起
        }
      });
    });
  }

  private checkPendingSupplementalPrepareMessages(taskId: string): void {
    const pendingMessage = this.pendingPrePrepareMessages.get(taskId);
    if (!pendingMessage) {
      return;
    }
  }

  // 添加一个新方法来处理待处理的PrePrepare消息
  private checkPendingPrePrepareMessages(taskId: string): void {
    const pendingMessage = this.pendingPrePrepareMessages.get(taskId);
    if (!pendingMessage) {
      return;
    }

    const localProofs = this.getProofsForTask(taskId);

    // 如果现在有足够的证明，处理这个待处理的PrePrepare消息
    if (localProofs.length >= 2) {
      logger.info(`节点 ${this.nodeId} 现在有足够的证明处理之前的PrePrepare消息: 任务ID ${taskId}`);

      // 删除待处理消息
      this.pendingPrePrepareMessages.delete(taskId);

      // 重新处理这个消息
      const response = this.processPrePrepareMessage(pendingMessage);

      const metricsCollector = GlobalMetricsCollector.getInstance();
      // 记录PREPARE_SENT
      metricsCollector.recordEvent({
        taskId: taskId,
        nodeId: this.nodeId,
        eventType: EventType.PREPARE_SENT,
        timestamp: Date.now(),
        // metadata: { verifierId: proof.verifierId },
      });

      // logger.warn(`${this.nodeId} 广播了 PPPPPPrepare 信令`);
      this.messageHandler.broadcast(response!);

      // 这里我们将自己的Prepare消息作为"新收到的消息"来处理
      // console.warn(`${this.nodeId}开始处理自己的PrePare消息`);

      const selfPrepareMsg = { ...response }; // 复制一份避免引用问题
      const commitMsg = this.pbftEngine.handlePrepare(selfPrepareMsg as PBFTMessage);

      if (commitMsg) {
        metricsCollector.recordEvent({
          taskId: taskId,
          nodeId: this.nodeId,
          eventType: EventType.COMMIT_SENT,
          timestamp: Date.now(),
          // metadata: { verifierId: proof.verifierId },
        });
        logger.warn(`${this.nodeId} 在 checkPendingPrePrepareMessages 广播了 Commit 信令`);
        this.messageHandler.broadcast(commitMsg);

        // 这里我们将自己的Commit消息作为"新收到的消息"来处理
        // console.warn(`${this.nodeId}开始处理自己的Commit消息`);

        const selfCommitMsg = { ...commitMsg };
        this.pbftEngine.handleCommit(selfCommitMsg);
      }
    } else {
      logger.info(`节点 ${this.nodeId} 收到新的PrePrepare消息证明: 任务ID ${taskId}`);
      console.log(`当前数量 ${localProofs.length}`);
    }
  }

  // 提取处理PrePrepare消息的逻辑到一个单独的方法
  private processPrePrepareMessage(message: PBFTMessage): PBFTMessage | null {
    if (!message.data) {
      logger.warn(`节点 ${this.nodeId} 收到无效的PrePrepare消息：缺少数据`);
      return null;
    }

    const taskId = message.taskId;
    const localProofs = this.getProofsForTask(taskId);
    this.currentConsensusTaskId = taskId;

    // 记录PREPREPARE_RECEIVED
    const metricsCollector = GlobalMetricsCollector.getInstance();
    metricsCollector.recordEvent({
      taskId: message.taskId,
      nodeId: this.nodeId,
      eventType: EventType.PREPREPARE_RECEIVED,
      timestamp: Date.now(),
    });

    if (localProofs.length < 2) {
      // 考量：就算因为没有足够的证明，现在先暂存共识请求，也应该提前立住currentConsensusTaskId
      logger.warn(`节点 ${this.nodeId} 没有足够的任务 ${taskId} 的证明，暂时存储共识请求`);
      // 存储这个消息，以便稍后处理
      this.pendingPrePrepareMessages.set(taskId, message);
      return null;
    }

    logger.info(`节点 ${this.nodeId} 开始处理任务 ${taskId} 的共识`);

    // 首先检查任务状态
    let status = this.taskStatuses.get(taskId);

    if (status) console.log(`${this.nodeId} 的状态是${status.state}`);

    const isFinalConsensus =
      message.consensusType === ConsensusType.Normal &&
      status &&
      (status.state === TaskProcessingState.Validated ||
        status.state === TaskProcessingState.AwaitingSupplementary);

    if (isFinalConsensus) {
      // 这是最终共识阶段的PrePrepare消息

      if (status!.state === TaskProcessingState.Validated) {
        // 如果节点已经处理过补充证明，直接接受PrePrepare
        logger.info(`节点 ${this.nodeId}: 任务 ${taskId} 已通过补充验证，跳过所有验证步骤`);

        // 更新任务状态为Consensus
        status!.state = TaskProcessingState.Consensus;
        this.taskStatuses.set(taskId, status!);

        // 直接处理PrePrepare消息
        return this.pbftEngine.handlePrePrepare(message);
      } else if (status!.state === TaskProcessingState.AwaitingSupplementary) {
        logger.warn(`节点 ${this.nodeId}: 收到任务 ${taskId} 的最终共识消息，但尚未处理补充证明`);

        // 存储这个最终共识的PrePrepare消息，以待补充证明处理完成后使用
        this.pendingFinalPrePrepareMessages.set(taskId, message);
        return null;

        // // 可以选择请求补充证明
        // const supplementaryProofId = message.data.supplementaryProofId;
        // if (supplementaryProofId) {
        //   logger.info(
        //     `节点 ${this.nodeId}: 请求获取任务 ${taskId} 的补充证明 ${supplementaryProofId}`
        //   );
        // 这里添加请求补充证明的逻辑
        // this.requestSupplementaryProof(taskId, supplementaryProofId);

        // 选择1: 暂停处理PrePrepare，直到收到补充证明
        // 存储这个消息，以便稍后处理
        // this.pendingPrePrepareMessages.set(taskId, message);
        // return null;

        // 选择2: 基于共识信任，接受PrePrepare（尽管没有亲自验证补充证明）
        // 这种情况下，节点信任其他节点的补充验证结果
        /* 
            logger.info(`节点 ${this.nodeId}: 信任其他节点的补充验证结果，接受任务 ${taskId} 的最终共识`);
            status.state = TaskProcessingState.Consensus;
            this.taskStatuses.set(taskId, status);
            return this.pbftEngine.handlePrePrepare(message);
            */
      }
    }

    const validationResult = this.qosValidator.quickValidate(message.data as VerifierQosProof);
    if (!validationResult.isValid) {
      logger.warn(`QoS证明验证失败：${JSON.stringify(validationResult.details)}`);
      return null;
    }

    // 无论什么情况都进行深度验证
    console.info(`${this.nodeId}执行深度验证`);
    const deepValidationResult = this.qosValidator.deepValidate(localProofs);

    // 获取或创建任务状态 - 不管是否冲突都需要
    // let status = this.taskStatuses.get(taskId);
    if (!status) {
      status = {
        taskId: taskId,
        state: TaskProcessingState.Verified, // 初始状态为Verified
        proofCount: localProofs.length,
        verifierIds: localProofs.map((p: VerifierQosProof) => p.verifier_id),
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      };
    }

    // 确保validationInfo存在
    if (!status.validationInfo) {
      status.validationInfo = {};
    }

    const localConsensusType = deepValidationResult.isValid
      ? ConsensusType.Normal
      : ConsensusType.Conflict;

    if (localConsensusType === ConsensusType.Conflict) {
      status.state = TaskProcessingState.Conflict;

      deepValidationResult.conflictType =
        this.qosValidator.analyzeConflictType(deepValidationResult);

      status.validationInfo.conflictType = deepValidationResult.conflictType;
      status.validationInfo.conflictDetails = deepValidationResult.details;

      logger.info(
        `节点 ${this.nodeId}: 任务 ${taskId} 的证明深度验证不通过，冲突类型: ${deepValidationResult.conflictType}, 详情: ${JSON.stringify(deepValidationResult.details)}`
      );
    }

    // console.warn(
    //   `inside processPrePrepareMessage ${this.nodeId} 完成深度验证，准备handlePrePrePare`
    // );

    // 更新任务状态为Consensus
    status.state = TaskProcessingState.Consensus;
    this.taskStatuses.set(taskId, status);

    // console.warn(`inside  processPrePrepareMessage 返回值是`);
    // console.log(response);

    return this.pbftEngine.handlePrePrepare(message);
  }

  // 处理冲突共识达成后的操作
  // private handleConflictConsensusReached(taskId: string): void {
  //   const status = this.taskStatuses.get(taskId);
  //   if (!status) {
  //     logger.warn(`无法处理冲突共识：任务 ${taskId} 无状态`);
  //     return;
  //   }

  //   const metricsCollector = GlobalMetricsCollector.getInstance();
  //   metricsCollector.recordConsensusEvent({
  //     taskId: taskId,
  //     nodeId: this.nodeId,
  //     eventType: EventType.CONSENSUS_REACH_CONFLICT,
  //     timestamp: Date.now(),
  //     // ConsensusResult 到底应该是啥等待填补
  //     ConsensusResult: {},
  //     // metadata: { verifierId: proof.verifierId },
  //   });

  //   // 更新任务状态为等待补充验证
  //   status.state = TaskProcessingState.AwaitingSupplementary;
  //   status.updatedAt = new Date().toISOString();
  //   this.taskStatuses.set(taskId, status);

  //   logger.info(`${this.nodeId}视角下，任务 ${taskId} 进入等待补充验证状态`);

  //   // 请求补充验证
  //   this.requestSupplementaryValidation(taskId);
  // }

  // for test to design
  public forceCheckTimeout(taskId: string): void {
    this.checkSupplementaryTimeout(taskId);
  }

  // 处理补充证明就绪消息
  private handleSupplementaryReadyMessage(message: SupplementaryReadyMessage): void {
    const { taskId, supplementaryProofId, nodeId } = message;

    logger.info(`节点 ${this.nodeId} 收到来自 ${nodeId} 的补充证明就绪消息，任务ID: ${taskId}`);

    // 检查自己是否有该任务的状态和补充证明(这里是完全没有这个任务的信息)
    const status = this.taskStatuses.get(taskId);
    if (!status) {
      logger.warn(`节点 ${this.nodeId} 没有任务 ${taskId} 的状态信息，无法处理补充证明就绪消息`);
      return;
    }

    // 如果是follower，需要检查是否已接收并处理了该补充证明
    if (!this.isLeader) {
      if (
        status.state === TaskProcessingState.Validated ||
        status.state === TaskProcessingState.Consensus ||
        status.state === TaskProcessingState.Finalized
      ) {
        const ackMessage: SupplementaryAckMessage = {
          type: 'SupplementaryAck',
          taskId,
          supplementaryProofId,
          nodeId: this.nodeId,
          timestamp: Date.now(),
          signature: sign(
            {
              type: 'SupplementaryAck',
              taskId,
              supplementaryProofId,
              timestamp: Date.now(),
            },
            'private_key'
          ), // 使用私钥签名
        };

        // 向leader发送确认消息
        this.messageHandler.sendSupplementaryAck(nodeId, ackMessage);
        logger.info(`节点 ${this.nodeId} 已向 ${nodeId} 确认补充证明就绪，任务ID: ${taskId}`);
        return;
      }
      const proofs = this.getProofsForTask(taskId);
      const hasSupplementaryProof = proofs.some(p => p.id === supplementaryProofId);

      if (!hasSupplementaryProof) {
        logger.warn(
          `节点 ${this.nodeId} 没有任务 ${taskId} 的补充证明 ${supplementaryProofId}，请求获取`
        );
        // TODO: 向leader请求获取补充证明的逻辑
        return;
      }
    }
    // Leader不需要在这里做额外处理
  }

  // 处理补充证明确认消息
  private handleSupplementaryAckMessage(message: SupplementaryAckMessage): void {
    if (!this.isLeader) {
      // 只有leader需要处理确认消息
      return;
    }

    const { taskId, supplementaryProofId, nodeId } = message;

    // 首先检查任务状态，确认是否已经进入最终共识阶段
    const status = this.taskStatuses.get(taskId);
    if (
      status &&
      (status.state === TaskProcessingState.Consensus ||
        status.state === TaskProcessingState.Finalized)
    ) {
      // 任务已经进入最终共识或已完成，忽略晚到的确认消息
      logger.info(
        `Leader ${this.nodeId} 收到来自 ${nodeId} 的晚到确认消息，任务 ${taskId} 已进入 ${status.state} 阶段`
      );
      return;
    }

    logger.info(`Leader ${this.nodeId} 收到来自 ${nodeId} 的补充证明确认消息，任务ID: ${taskId}`);

    // 获取或初始化已就绪节点集合
    if (!this.supplementaryReadyStatus.has(taskId)) {
      this.supplementaryReadyStatus.set(taskId, new Set<string>());
    }

    const readyNodes = this.supplementaryReadyStatus.get(taskId)!;
    readyNodes.add(nodeId);

    // 检查是否有足够多的节点已就绪
    // 假设总节点数为4（1个leader和3个follower），需要2f+1=3个节点就绪
    const totalNodes = 4; // 应该从配置中获取
    const requiredNodes = Math.floor(2 * Math.floor((totalNodes - 1) / 3) + 1);

    // 确保leader自己也计入就绪节点
    readyNodes.add(this.nodeId);

    logger.info(`任务 ${taskId} 的就绪节点数：${readyNodes.size}/${requiredNodes}`);

    if (readyNodes.size >= requiredNodes) {
      logger.info(`任务 ${taskId} 已有足够节点就绪，启动最终共识`);

      // 检查是否有待处理的共识请求
      const pendingConsensusData = this.pendingSupplementaryConsensus.get(taskId);
      if (pendingConsensusData) {
        this.startFinalConsensus(taskId, pendingConsensusData);
        this.pendingSupplementaryConsensus.delete(taskId);
        this.supplementaryReadyStatus.delete(taskId);
      }
    }
  }

  // 启动最终共识的方法
  private startFinalConsensus(taskId: string, consensusData: any): void {
    const status = this.taskStatuses.get(taskId);
    if (!status || status.state !== TaskProcessingState.Validated) {
      logger.warn(`无法为任务 ${taskId} 启动最终共识：状态不正确`);
      return;
    }

    // 更新任务状态为共识中
    status.state = TaskProcessingState.Consensus;
    this.taskStatuses.set(taskId, status);
    this.currentConsensusTaskId = taskId;

    // 清理相关的临时状态
    this.supplementaryReadyStatus.delete(taskId);
    this.pendingSupplementaryConsensus.delete(taskId);

    try {
      // 启动正常共识流程

      const metricsCollector = GlobalMetricsCollector.getInstance();

      // 记录接收到QoSProof证明事件
      metricsCollector.recordConsensusEvent({
        taskId: taskId,
        nodeId: this.nodeId,
        eventType: EventType.CONSENSUS_STARTED,
        timestamp: Date.now(),
        metadata: { consensusType: ConsensusType.Normal },
        // 此项指标等待填补
        ConsensusResult: {},
      });

      const prePrepareMsg = this.pbftEngine.startConsensus(consensusData, ConsensusType.Normal);

      if (prePrepareMsg) {
        const metricsCollector = GlobalMetricsCollector.getInstance();
        metricsCollector.recordEvent({
          taskId: taskId,
          nodeId: this.nodeId,
          eventType: EventType.PREPREPARE_SENT,
          timestamp: Date.now(),
          // metadata: { verifierId: proof.verifierId },
        });

        this.messageHandler.broadcast(prePrepareMsg);
        logger.info(`Leader 已启动任务 ${taskId} 的最终共识流程`);

        // Leader也需要响应PrePrepare消息
        const prepareMsg = this.pbftEngine.handlePrePrepare(prePrepareMsg);
        if (prepareMsg) {
          metricsCollector.recordEvent({
            taskId: taskId,
            nodeId: this.nodeId,
            eventType: EventType.PREPARE_SENT,
            timestamp: Date.now(),
            // metadata: { verifierId: proof.verifierId },
          });
          // 这里是为了把leader的prepareMsg发出来，要不然别的节点拿不到了
          this.messageHandler.broadcast(prepareMsg);

          logger.info(`Leader ${this.nodeId} 响应自己的PrePrepare消息，广播Prepare信令`);
        }
      } else {
        logger.warn(`无法为已验证任务 ${taskId} 启动共识流程`);
        status.state = TaskProcessingState.Validated;
        this.taskStatuses.set(taskId, status);
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      logger.error(`启动共识过程出错: ${errorMessage}`);
      status.state = TaskProcessingState.Validated; // 退回到已验证状态
      this.taskStatuses.set(taskId, status);
    }
  }

  // 请求补充验证
  private requestSupplementaryValidation(taskId: string): void {
    const status = this.taskStatuses.get(taskId);
    if (!status || status.state !== TaskProcessingState.AwaitingSupplementary) {
      logger.warn(`无法请求补充验证：任务 ${taskId} 不在等待补充验证状态`);
      return;
    }

    // 确保validationInfo存在
    if (!status.validationInfo) {
      status.validationInfo = {};
    }

    // 标记已请求补充验证
    status.validationInfo.supplementaryRequested = true;
    status.validationInfo.supplementaryRequestTime = new Date().toISOString();
    this.taskStatuses.set(taskId, status);

    logger.info(`已为任务 ${taskId} 请求补充验证`);

    // 这里实际项目中应该实现外部通知或智能合约调用
    // 例如：向指定的Verifier节点发送验证请求
    // 或者：调用区块链智能合约选择额外的验证者

    // 设置超时检查 - 两小时后检查
    const TWO_HOURS = 2 * 60 * 60 * 1000; // 2小时
    const FIVE_SECONDS = 5 * 1000;
    setTimeout(() => {
      this.checkSupplementaryTimeout(taskId);
    }, TWO_HOURS);

    // 这里应该是通知系统选定的额外Verifier执行验证任务
    // 在实际实现中，这可能涉及调用智能合约或发送网络消息

    // logger.info(`已为任务 ${taskId} 请求补充验证`);

    // 模拟请求补充验证成功的日志
    // 实际应用中，补充验证的结果会通过handleSupplementaryProof方法接收
  }

  private checkSupplementaryTimeout(taskId: string): void {
    const status = this.taskStatuses.get(taskId);
    if (!status) {
      logger.warn(`无法检查补充验证超时：任务 ${taskId} 不存在`);
      return;
    }

    // 如果任务状态不再是等待补充验证，则不需处理
    if (status.state !== TaskProcessingState.AwaitingSupplementary) {
      logger.info(`任务 ${taskId} 已不在等待补充验证状态，超时检查跳过`);
      return;
    }

    // 检查是否已收到补充验证
    if (status.supplementaryVerifierIds && status.supplementaryVerifierIds.length > 0) {
      logger.info(`任务 ${taskId} 已收到补充验证，超时检查跳过`);
      return; // 已收到补充验证，无需处理
    }

    logger.warn(`任务 ${taskId} 补充验证请求超时（两小时）`);

    // 更新任务状态为需要人工审核
    status.state = TaskProcessingState.NeedsManualReview;
    status.updatedAt = new Date().toISOString();

    if (!status.validationInfo) {
      status.validationInfo = {};
    }
    logger.warn(`任务 ${taskId} 补充验证请求超时（两小时）`);
    status.validationInfo.timeoutReason = '补充验证请求超时未收到响应（两小时）';

    this.taskStatuses.set(taskId, status);

    logger.info(`任务 ${taskId} 已更新为需要人工审核状态`);

    // 如果这个任务还在共识队列中，下次处理队列时会跳过它
    // 由于状态已经不是AwaitingSupplementary，所以不会被处理
  }

  // private findLeaderNodeId(): string {
  //   const peers = this.messageHandler.getConnectedPeers();

  //   for (const peer of peers) {
  //     if (peer.isLeader) {
  //       return peer.nodeId;
  //     }
  //   }

  //   return 'leader'; // 默认值
  // }

  public async handleSupplementaryProof(taskId: string, proof: VerifierQosProof): Promise<void> {
    // 返回一个Promise，包装原始处理逻辑
    return new Promise<void>(resolve => {
      // 使用setImmediate将处理逻辑放入下一个事件循环
      setImmediate(async () => {
        try {
          // 验证任务状态是否为"等待补充验证"
          const status = this.taskStatuses.get(taskId);
          if (!status || status.state !== TaskProcessingState.AwaitingSupplementary) {
            logger.warn(`无法处理补充证明：任务 ${taskId} 不在等待补充验证状态`);
            resolve(); // 即使验证失败也解析Promise
            return;
          }

          // 确认已有足够的原始证明
          const originalProofs = this.getProofsForTask(taskId);
          logger.info(`进入${this.nodeId}视角的handleSupplementaryProof`);

          if (originalProofs.length < 2) {
            logger.warn(`无法处理补充证明：任务 ${taskId} 的原始证明不足`);
            resolve();
            return;
          }

          // 对补充证明进行快速验证
          const validationResult = this.qosValidator.quickValidate(proof);
          if (!validationResult.isValid) {
            logger.warn(`补充证明快速验证失败: ${JSON.stringify(validationResult.details)}`);
            // 更新任务状态为Failed，而不是简单返回
            status.state = TaskProcessingState.Failed;
            status.updatedAt = new Date().toISOString();

            if (!status.validationInfo) {
              status.validationInfo = {};
            }
            status.validationInfo.supplementaryResult = {
              isValid: false,
              details: validationResult.details,
            };

            this.taskStatuses.set(taskId, status);
            resolve();
            return;
          }

          // 确保补充证明有唯一ID
          if (!proof.id) {
            proof.id = `supplementary-${taskId}-${Date.now()}`;
          }

          // 将补充证明存储到系统中
          this.storeProof(taskId, proof);
          status.proofCount++;
          status.verifierIds.push(proof.verifier_id);
          status.updatedAt = new Date().toISOString();

          // 专门记录补充验证者ID
          if (!status.supplementaryVerifierIds) {
            status.supplementaryVerifierIds = [];
          }
          status.supplementaryVerifierIds.push(proof.verifier_id);

          // 获取保存的冲突信息
          const conflictType = status.validationInfo?.conflictType || 'structural';
          const initialResult = {
            isValid: false,
            hasConflict: true,
            conflictType: conflictType,
            details: status.validationInfo?.conflictDetails,
          };

          try {
            // 使用补充证明尝试解决之前发现的冲突
            const resolvedResult = this.qosValidator.resolveWithSupplementaryProof(
              originalProofs,
              proof,
              initialResult
            );

            logger.info(
              `${this.nodeId}视角下:任务 ${taskId} 的补充验证结果: ${JSON.stringify(resolvedResult)}`
            );

            // 确保 validationInfo 对象存在
            if (!status.validationInfo) {
              status.validationInfo = {};
            }
            status.validationInfo.resolvedResult = resolvedResult;

            // 更新任务状态
            if (resolvedResult.isValid) {
              status.state = TaskProcessingState.Validated;
              logger.info(`任务 ${taskId} 通过补充验证已解决冲突`);

              // 启动最终共识流程前，只有Leader能通知所有节点补充证明就绪
              if (this.isLeader) {
                // 获取所有证明（包括补充证明）
                const allProofs = this.getProofsForTask(taskId);

                // 准备最终共识数据
                const consensusData = {
                  ...allProofs[0],
                  supplementaryInfo: resolvedResult,
                };

                // 存储等待共识的数据
                this.pendingSupplementaryConsensus.set(taskId, consensusData);

                // 广播补充证明就绪消息
                const readyMessage: SupplementaryReadyMessage = {
                  type: 'SupplementaryReady',
                  taskId,
                  supplementaryProofId: proof.id,
                  nodeId: this.nodeId,
                  timestamp: Date.now(),
                  signature: sign(
                    {
                      type: 'SupplementaryReady',
                      taskId,
                      supplementaryProofId: proof.id,
                      timestamp: Date.now(),
                    },
                    'private_key'
                  ), // 使用私钥签名
                };

                // 广播就绪消息 - 注意这里可能需要改造为异步
                await this.messageHandler.broadcastSupplementaryReady(readyMessage);
                logger.info(`Leader ${this.nodeId} 已广播任务 ${taskId} 的补充证明就绪消息`);

                // 初始化就绪状态跟踪
                if (!this.supplementaryReadyStatus.has(taskId)) {
                  this.supplementaryReadyStatus.set(taskId, new Set<string>());
                }
                // Leader自己标记为就绪
                this.supplementaryReadyStatus.get(taskId)!.add(this.nodeId);
              }
              // 如果是follower并收到了补充证明，直接更新状态为已验证
              else {
                logger.info(`Follower ${this.nodeId} 已处理任务 ${taskId} 的补充证明`);
                status.state = TaskProcessingState.Validated;
                this.taskStatuses.set(taskId, status);

                const pendingFinalPrePrepare = this.pendingFinalPrePrepareMessages.get(taskId);

                if (pendingFinalPrePrepare) {
                  logger.info(`节点 ${this.nodeId} 开始处理之前延迟的最终共识PrePrepare消息`);

                  // 移除待处理消息
                  this.pendingFinalPrePrepareMessages.delete(taskId);

                  // 现在可以处理这个PrePrepare消息了
                  const response = await this.processPrePrepareMessage(pendingFinalPrePrepare);
                  if (response) {
                    await this.messageHandler.broadcast(response);

                    // 处理自己的Prepare消息
                    const selfPrepareMsg = { ...response };
                    const commitMsg = await this.pbftEngine.handlePrepare(
                      selfPrepareMsg as PBFTMessage
                    );

                    if (commitMsg) {
                      await this.messageHandler.broadcast(commitMsg);

                      // 处理自己的Commit消息
                      const selfCommitMsg = { ...commitMsg };
                      await this.pbftEngine.handleCommit(selfCommitMsg);
                    }
                  }
                } else {
                  // 主动向leader发送确认消息
                  const ackMessage: SupplementaryAckMessage = {
                    type: 'SupplementaryAck',
                    taskId,
                    supplementaryProofId: proof.id,
                    nodeId: this.nodeId,
                    timestamp: Date.now(),
                    signature: sign(
                      {
                        type: 'SupplementaryAck',
                        taskId,
                        supplementaryProofId: proof.id,
                        timestamp: Date.now(),
                      },
                      'private_key'
                    ), // 使用私钥签名
                  };

                  // 查找leader节点
                  const leaderNodeId = 'leader'; // 假设leader节点ID为'leader'，实际环境中可能需要动态获取

                  // 向leader发送确认消息
                  try {
                    await this.messageHandler.sendSupplementaryAck(leaderNodeId, ackMessage);
                    logger.info(
                      `Follower ${this.nodeId} 已主动向leader确认补充证明处理完成，任务ID: ${taskId}`
                    );
                  } catch (error) {
                    const errorMessage = error instanceof Error ? error.message : String(error);
                    logger.error(
                      `Follower ${this.nodeId} 向leader发送确认消息失败: ${errorMessage}`
                    );
                  }
                }
              }
            } else if (resolvedResult.needsManualReview) {
              // 如果需要人工审核：状态更新为"需要人工审核"
              status.state = TaskProcessingState.NeedsManualReview;
              logger.warn(`任务 ${taskId} 需要人工审核: ${resolvedResult.details?.reason}`);
            } else {
              // 如果解决失败：状态更新为"失败"
              status.state = TaskProcessingState.Failed;
              logger.error(`任务 ${taskId} 补充验证失败: ${resolvedResult.details?.reason}`);
            }

            // 无论如何都更新任务状态
            this.taskStatuses.set(taskId, status);
          } catch (error) {
            // 处理解决冲突过程中的异常
            const errorMessage = error instanceof Error ? error.message : String(error);
            logger.error(`解决冲突过程出错: ${errorMessage}`);
            status.state = TaskProcessingState.Failed;

            if (!status.validationInfo) {
              status.validationInfo = {};
            }
            status.validationInfo.errorMessage = errorMessage;

            this.taskStatuses.set(taskId, status);
          }

          // 成功完成处理
          resolve();
        } catch (error) {
          // 记录错误但不中断处理流程
          const errorMessage = error instanceof Error ? error.message : String(error);
          logger.error(`处理补充证明时出错: ${errorMessage}`);
          resolve(); // 即使出错也解析Promise，避免挂起
        }
      });
    });
  }

  public updateTaskState(
    taskId: string,
    state: TaskProcessingState,
    details?: Partial<TaskStatus>
  ): void {
    const status = this.taskStatuses.get(taskId);
    if (status) {
      status.state = state;
      status.updatedAt = new Date().toISOString();

      if (details) {
        Object.assign(status, details);
      }
      logger.info(`任务 ${taskId} 状态更新为 ${state}`);
      this.taskStatuses.set(taskId, status);
    } else {
      logger.warn(`尝试更新不存在的任务状态: ${taskId}`);
    }
  }

  public getTaskStatus(taskId: string): TaskStatus | null {
    return this.taskStatuses.get(taskId) || null;
  }

  // 获取音频评分的辅助方法
  private getAverageAudioScore(taskId: string): number {
    const proofs = this.getProofsForTask(taskId);
    const scores = proofs
      .filter(p => p.audio_score && typeof p.audio_score === 'number')
      .map(p => p.audio_score as number);

    if (scores.length === 0) {
      return 0;
    }

    return scores.reduce((sum, score) => sum + score, 0) / scores.length;
  }

  // 获取GOP验证结果的辅助方法
  // 获取GOP验证结果的辅助方法
  private getVerificationResults(taskId: string): {
    gopScores: GopScore[];
    verification: GopVerificationResult;
  } {
    const proofs = this.getProofsForTask(taskId);

    // console.log('inside getVerificationResults');
    // console.log('proofs');
    // console.log(proofs);
    // console.log('first proof 的 videoQualityData 的 gopScores');
    // console.log(proofs[0].videoQualityData.gopScores);
    // console.log('second proof 的 videoQualityData 的 gopScores');
    // console.log(proofs[1].videoQualityData.gopScores);

    if (proofs.length === 0 || !proofs[0].video_score) {
      return { gopScores: [], verification: GopVerificationResult.UndeterminedError };
    }

    let originalGopScores = proofs[0].gop_scores;

    // 转换gopScores，确保使用vmaf_score字段名
    const gopScores = originalGopScores.map(score => ({
      timestamp: score.timestamp,
      // @ts-ignore
      vmaf_score: score.vmafResult || score.vmafScore || 0, // 将vmafResult转换为vmaf_score
      hash: score.hash || '',
    }));

    // 确定验证结果
    // 默认为已验证
    let verification = GopVerificationResult.Verified;

    // 检查是否所有证明的GOP评分一致
    if (proofs.length > 1) {
      const allGopScoresMatch = this.compareGopScores(proofs);
      if (!allGopScoresMatch) {
        verification = GopVerificationResult.GopMismatch;
      }
    }

    return { gopScores, verification };
  }

  // 比较多个证明的GOP分数是否一致
  private compareGopScores(proofs: VerifierQosProof[]): boolean {
    if (proofs.length <= 1) {
      return true;
    }

    // 获取第一个证明的GOP分数数组
    const firstGopScoresArray = proofs[0].gop_scores || [];

    // 创建一个Map用于快速查找，键为timestamp
    const firstGopScoresMap = new Map<string, number>();
    firstGopScoresArray.forEach(score => {
      firstGopScoresMap.set(score.timestamp, score.vmaf_score);
    });

    const firstTimestamps = Array.from(firstGopScoresMap.keys()).sort();

    // 与其他证明比较
    for (let i = 1; i < proofs.length; i++) {
      const currentGopScoresArray = proofs[i].gop_scores || [];

      // 创建当前证明的Map
      const currentGopScoresMap = new Map<string, number>();
      currentGopScoresArray.forEach(score => {
        currentGopScoresMap.set(score.timestamp, score.vmaf_score);
      });

      const currentTimestamps = Array.from(currentGopScoresMap.keys()).sort();

      // 比较时间戳数量
      if (firstTimestamps.length !== currentTimestamps.length) {
        return false;
      }

      // 比较每个时间戳的评分
      for (const timestamp of firstTimestamps) {
        const currentScore = currentGopScoresMap.get(timestamp);

        // 如果当前时间戳不存在
        if (currentScore === undefined) {
          return false;
        }

        const firstScore = firstGopScoresMap.get(timestamp)!;
        const difference = Math.abs(firstScore - currentScore);

        // 允许一定的分数差异(例如1%)
        if (difference > firstScore * 0.01) {
          return false;
        }
      }
    }

    return true;
  }

  // 获取同步评分的辅助方法
  private getAverageSyncScore(taskId: string): number {
    const proofs = this.getProofsForTask(taskId);
    const scores = proofs
      .filter(p => p.sync_score && typeof p.sync_score === 'number')
      .map(p => p.sync_score as number);

    if (scores.length === 0) {
      return 0;
    }

    return scores.reduce((sum, score) => sum + score, 0) / scores.length;
  }

  // 获取视频评分的辅助方法
  private getVideoScores(taskId: string): { average: number; scores: number[] } {
    const proofs = this.getProofsForTask(taskId);
    const scores = proofs.map(p => p.video_score).filter(s => s > 0);

    if (scores.length === 0) {
      return { average: 0, scores: [] };
    }

    const average = scores.reduce((sum, score) => sum + score, 0) / scores.length;
    return { average, scores };
  }

  /**
   * 将纳秒转换为毫秒
   * @param nanoseconds 纳秒时间戳
   * @returns 毫秒时间戳
   */
  private convertNanoToMilliseconds(nanoseconds: number | undefined | null): number {
    // 如果输入是undefined或null，则返回0
    const nanos = nanoseconds || 0;
    // 纳秒转换为毫秒 (1毫秒 = 1,000,000纳秒)
    return Math.floor(nanos / 1_000_000);
  }

  // 构造共识证明对象
  private constructConsensusProof(
    taskId: string,
    proof: VerifierQosProof,
    task: TaskData
  ): ConsensusQosProof {
    const videoScores = this.getVideoScores(taskId);
    console.log('audio_score');
    console.log(videoScores);
    const gopsResults = this.getVerificationResults(taskId);
    console.log('gopsResults.gopScores');
    console.log(gopsResults.gopScores);
    console.log('gopsResults.verification');
    console.log(gopsResults.verification);

    return {
      task_id: taskId,
      worker_id: task.assigned_worker!,
      // committee_members: committeeMembers,
      committee_leader: this.nodeId,
      timestamp: Date.now(),

      // 以下值需要从验证结果中提取
      video_score: videoScores.average,
      audio_score: this.getAverageAudioScore(taskId),
      sync_score: this.getAverageSyncScore(taskId),

      encoding_start_time: this.convertNanoToMilliseconds(task.assignment_time),
      encoding_end_time: this.convertNanoToMilliseconds(task.completion_time),
      video_specs: proof.video_specs,
      frame_count: task.frame_count || 0,

      specified_gop_scores: gopsResults.gopScores,
      gop_verification: GopVerificationResult.Verified,

      status: QosProofStatus.Normal, // 默认状态，会被提交时的参数覆盖
    };
  }

  // 添加向区块链提交共识结果的方法
  private async submitConsensusToBlockchain(
    taskId: string,
    proof: VerifierQosProof,
    status: QosProofStatus
  ): Promise<boolean> {
    if (!this.nearConnection) {
      logger.error(`无法提交共识结果: NEAR连接未初始化`);
      return false;
    }

    try {
      // 获取任务详情
      const task = await this.nearConnection.getTask(taskId);
      if (!task) {
        logger.error(`无法获取任务 ${taskId} 的详情`);
        return false;
      }

      // 构造共识证明
      const consensusProof = this.constructConsensusProof(taskId, proof, task);

      console.log('已经构建consensusProof');
      console.log(consensusProof);

      // 添加调试输出
      console.log('consensusProof对象类型检查:');
      console.log('gop_verification类型:', typeof consensusProof.gop_verification);
      console.log('gop_verification值:', consensusProof.gop_verification);
      console.log('status类型:', typeof consensusProof.status);
      console.log('status值:', consensusProof.status);

      // 提交共识证明
      const result = await this.nearConnection.submitConsensusProof(consensusProof);

      if (result) {
        logger.info(`成功提交任务 ${taskId} 的共识结果到区块链，状态: ${status}`);
      } else {
        logger.error(`提交任务 ${taskId} 的共识结果到区块链失败`);
      }

      return result;
    } catch (error) {
      logger.error(`提交共识结果到区块链时出错: ${error}`);
      return false;
    }
  }

  // 共识达成后的回调
  private async onConsensusReached(
    proof: VerifierQosProof,
    consensusType: ConsensusType
  ): Promise<void> {
    const taskId = proof.task_id;
    logger.info(`节点 ${this.nodeId} 达成共识: 任务ID ${taskId}`);

    console.warn(`${this.nodeId} 视角下 inside onConsensusReached，当前时间${Date.now()}`);

    console.log('proof');
    console.log(proof);

    // 只有Leader节点执行区块链交互
    if (this.isLeader && this.nearConnection) {
      if (consensusType === ConsensusType.Conflict) {
        // 处理冲突共识
        await this.handleConflictConsensusReached(taskId);
      } else {
        // 处理正常共识，提交结果到区块链
        await this.submitConsensusToBlockchain(taskId, proof, QosProofStatus.Normal);
      }
    }

    // TODO: 在此处添加上链或其他后处理逻辑
    // 例如：调用智能合约将结果存储在区块链上
    console.log(`${taskId}的共识类型是${consensusType}`);

    if (consensusType === ConsensusType.Conflict) {
      // 处理冲突共识
      this.handleConflictConsensusReached(taskId);
    } else {
      // 处理正常共识
      const status = this.taskStatuses.get(taskId);
      if (status) {
        status.state = TaskProcessingState.Finalized;
        status.updatedAt = new Date().toISOString();
        if (!status.result) {
          status.result = {};
        }
        status.result.consensusTimestamp = new Date().toISOString();
        const metricsCollector = GlobalMetricsCollector.getInstance();
        metricsCollector.recordConsensusEvent({
          taskId: taskId,
          nodeId: this.nodeId,
          eventType: EventType.CONSENSUS_REACH_NORMAL,
          timestamp: Date.now(),
          // ConsensusResult 到底应该是啥等待填补
          ConsensusResult: {},
          // metadata: { verifierId: proof.verifierId },
        });
        this.taskStatuses.set(taskId, status);
      }

      // TODO: 调用智能合约将结果上链

      logger.info(`共识结果已处理: ${calculateHash(proof)}`);
    }

    // 从队列中移除当前任务
    if (this.consensusQueue.length > 0 && this.consensusQueue[0] === taskId) {
      this.consensusQueue.shift();
    }

    // 重置处理状态并处理下一个任务
    this.processingConsensus = false;
    this.currentConsensusTaskId = null;
    this.processConsensusQueue();
  }

  // 处理冲突共识后向区块链请求补充验证者
  private async handleConflictConsensusReached(taskId: string): Promise<void> {
    if (!this.nearConnection) {
      logger.error(`无法处理冲突共识: NEAR连接未初始化`);
      return;
    }

    const status = this.taskStatuses.get(taskId);
    if (!status) {
      logger.warn(`无法处理冲突共识：任务 ${taskId} 无状态`);
      return;
    }

    // 记录事件
    const metricsCollector = GlobalMetricsCollector.getInstance();
    metricsCollector.recordConsensusEvent({
      taskId: taskId,
      nodeId: this.nodeId,
      eventType: EventType.CONSENSUS_REACH_CONFLICT,
      timestamp: Date.now(),
      ConsensusResult: {},
    });

    // 更新任务状态为等待补充验证
    status.state = TaskProcessingState.AwaitingSupplementary;
    status.updatedAt = new Date().toISOString();
    this.taskStatuses.set(taskId, status);

    logger.info(`任务 ${taskId} 进入等待补充验证状态`);

    try {
      // 请求补充验证者
      const supplementalVerifierId = await this.nearConnection.requestSupplementalVerifier(taskId);

      if (supplementalVerifierId) {
        logger.info(`成功为任务 ${taskId} 请求补充验证者: ${supplementalVerifierId}`);

        // 更新任务状态，记录补充验证者信息
        if (!status.validationInfo) {
          status.validationInfo = {};
        }

        status.validationInfo.supplementaryRequested = true;
        status.validationInfo.supplementaryRequestTime = new Date().toISOString();
        status.supplementaryVerifierIds?.push(supplementalVerifierId);

        this.taskStatuses.set(taskId, status);
      } else {
        logger.error(`请求补充验证者失败，任务: ${taskId}`);

        // 更新状态为需要人工审核
        status.state = TaskProcessingState.NeedsManualReview;
        this.taskStatuses.set(taskId, status);
      }
    } catch (error) {
      logger.error(`请求补充验证者时出错: ${error}`);

      // 更新状态为需要人工审核
      status.state = TaskProcessingState.NeedsManualReview;
      this.taskStatuses.set(taskId, status);
    }
  }

  // 获取节点状态
  public getStatus(): any {
    return {
      nodeId: this.nodeId,
      isLeader: this.isLeader,
      pbftState: this.pbftEngine.getState(),
      connections: this.messageHandler.getConnectionStatus(),
    };
  }

  private startTakStatusCleanup(): void {
    const CLEANUP_INTERVAL = 1000 * 60 * 60; // 1小时
    const MAX_AGE = 24 * 60 * 60 * 1000; // 24小时

    setInterval(() => {
      const now = Date.now();
      let expiredCount = 0;
      for (const [taskId, status] of this.taskStatuses.entries()) {
        const updatedTime = new Date(status.updatedAt).getTime();

        // 检查pending状态的任务是否过期
        if (status.state === TaskProcessingState.Pending && now - updatedTime > MAX_AGE) {
          this.updateTaskState(taskId, TaskProcessingState.Expired);
          expiredCount++;
        }
      }

      if (expiredCount > 0) {
        logger.info(`清理了 ${expiredCount} 个过期任务状态`);
      }
    }, CLEANUP_INTERVAL);
  }
}
