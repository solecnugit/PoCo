// src/types.ts

export enum TaskStatus {
  Published = "Published",
  Assigned = "Assigned",
  Completed = "Completed",
  Verified = "Verified",
  Queued = "Queued",
  OfferCollecting = "OfferCollecting",
}

export enum QosProofStatus {
  Pending = "Pending", // 等待通过
  Normal = "Normal", // 普通验证通过
  Conflict = "Conflict", // 经过补充验证通过
  Manual = "Manual", // 经过人工验证通过
}

export enum GopVerificationResult {
  Verified = "Verified", // GOP验证通过
  ScoreMismatch = "ScoreMismatch", // 分数不匹配
  GopMismatch = "GopMismatch", // GOP结构不匹配
  UndeterminedError = "UndeterminedError", // 未确定错误
}

export interface TranscodingRequirement {
  target_codec: string;
  target_resolution: string;
  target_bitrate: string;
  target_framerate: string;
  additional_params: string;
}

export interface TaskData {
  task_id: string;
  broadcaster_id: string;
  source_ipfs: string;
  requirements: TranscodingRequirement;
  status: TaskStatus;
  assigned_worker: string | null;
  assignment_time: number | null;
  result_ipfs: string | null;
  completion_time: number | null;
  assigned_verifiers: string[];
  qos_proof_id: string | null;

  // 新增字段
  publish_time: number;
  hw_acceleration_preferred: boolean;

  // 新增字段 - 从新合约中添加
  keyframe_timestamps: string[] | null; // 关键帧时间戳列表
  selected_gops: string[] | null; // 被选中用于验证的GOP时间戳
}

// 视频规格
export interface VideoSpecification {
  codec: string; // 编解码器
  resolution: string; // 分辨率
  bitrate: number; // 比特率
  framerate: number; // 帧率
}

// GOP分数
export interface GopScore {
  timestamp: string; // GOP ID
  vmaf_score: number; // VMAF分数
  hash: string; // GOP哈希
}

// 新增工作节点报价类型
export interface WorkerOffer {
  worker_id: string;
  task_id: string;
  timestamp: number;
}

// 新增工作节点信息类型
export interface WorkerInfo {
  account_id: string;
  has_hw_acceleration: boolean;
  last_heartbeat: number;
  available: boolean;
  current_task?: string;
  qos_score: number;
  service_reliability_score: number;
  time_stability_score: number;
  performance_score: number;
  total_tasks_completed: number;
  active_days_last_week: number;
}

// 共识QoS质量证明结构
export interface ConsensusQosProof {
  // 基本信息
  task_id: string; // 任务ID
  worker_id: string; // 工作节点ID
  timestamp: number; // 时间戳

  // 委员会信息
  committee_members: string[]; // 参与共识的委员会成员
  committee_leader: string; // 提交共识结果的委员会leader

  // 质量分数
  video_score: number; // VMAF分数
  audio_score: number; // PESQ分数
  sync_score: number; // 同步性分数

  // 视频信息
  encoding_start_time: number; // 编码开始时间
  encoding_end_time: number; // 编码结束时间
  video_specs: VideoSpecification; // 视频规格
  frame_count: number;

  // GOP验证
  specified_gop_scores: GopScore[]; // 特定GOP分数
  gop_verification: GopVerificationResult; // GOP验证结果

  // 状态
  status: QosProofStatus; // 状态（可以保留，虽然默认是已验证）
}

// 工作节点性能记录
export interface WorkerPerformance {
  // 基本信息
  task_id: string;
  worker_id: string;
  timestamp: number;
  encoding_duration: number; // 编码耗时
  frame_count: number; // 视频总帧数

  // 视频质量相关
  video_score: number; // VMAF分数
  video_quality_compliant: boolean; // 视频质量是否达标

  // 音频质量相关
  audio_score?: number; // PESQ分数，可能为null
  audio_quality_compliant?: boolean; // 音频质量是否达标，可能为null

  // 同步相关
  sync_score?: number; // 同步分数，可能为null
  sync_quality_compliant?: boolean; // 同步质量是否达标，可能为null

  // 规格相关
  video_specs: VideoSpecification;
  specs_compliant: boolean;

  // 总体评价
  overall_compliant: boolean; // 总体是否达标
}

export interface BroadcasterConfig {
  networkId: string;
  nodeUrl: string;
  walletUrl: string;
  helperUrl: string;
  explorerUrl: string;
  contractId: string;
  broadcasterAccountId?: string;
  ipfsConfig: {
    host: string;
    port: number;
    protocol: string;
    gateway: string;
  };
}

// 工作节点每日统计数据
export interface WorkerDailyStats {
  date: number; // 日期时间戳（YYYYMMDD格式）
  tasks_completed: number; // 当日完成任务数
  tasks_accepted: number; // 当日接受任务数
  provided_service: boolean; // 当日是否提供服务
  encoding_fps: number[]; // 单位时间编码
  quality_scores: number[]; // 质量评分列表
}

// QoS评分详情结构
export interface QosScoreDetails {
  worker_id: string; // 工作节点ID
  service_reliability_score: number; // 服务可靠性评分
  time_stability_score: number; // 时间稳定性评分
  performance_score: number; // 性能表现评分
  overall_qos_score: number; // 综合QoS评分
  last_update_time: number; // 最后更新时间
}

// 工作节点性能统计摘要
export interface WorkerPerformanceSummary {
  worker_id: string; // 工作节点ID
  total_tasks_completed: number; // 总完成任务数
  avg_video_score: number; // 平均视频分数
  avg_audio_score: number; // 平均音频分数
  avg_sync_score: number; // 平均同步分数
  avg_encoding_duration: number; // 平均编码时间（毫秒）
  completion_rate: number; // 任务完成率
  compliance_rate: number; // 质量达标率
  service_days_last_week: number; // 最近7天内服务天数
  qos_score: number; // 当前QoS评分
}

// 任务共识详细信息结构体
export interface TaskConsensusDetails {
  task: TaskData; // 任务基本信息
  consensus_proof?: ConsensusQosProof; // 共识证明（如果有）
  worker_performance?: WorkerPerformance; // 工作节点表现（如果有）
}
