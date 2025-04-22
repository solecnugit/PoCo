export enum TaskStatus {
  Published = 'Published',
  Assigned = 'Assigned',
  Completed = 'Completed',
  Verified = 'Verified',
  OfferCollecting = 'OfferCollecting',
  Queued = 'Queued',
}

export interface TranscodingRequirement {
  target_codec: string
  target_resolution: string
  target_bitrate: string
  target_framerate: string
  additional_params: string
}
// GOP分数
export interface GopScore {
  timestamp: string // GOP ID
  vmaf_score: number // VMAF分数
  hash: string // GOP哈希
}

export interface VerifierQosProof {
  id: string // Proof unique identifier
  taskId: string // Task ID
  verifierId: string // Verifier ID
  timestamp: number // Proof generation timestamp
  videoSpecs: VideoSpecification // Video specifications

  // Video quality data
  videoScore: number // Overall VMAF score
  gopScores: GopScore[] // Using modified GopScore structure

  // Optional fields
  audioScore?: number // Audio PESQ score
  syncScore?: number // Synchronization score

  signature: string // Verifier signature
}

// 视频规格
export interface VideoSpecification {
  codec: string // 编解码器
  resolution: string // 分辨率
  bitrate: number // 比特率
  framerate: number // 帧率
}

// 工作节点性能摘要
export interface WorkerPerformanceSummary {
  worker_id: string // 工作节点ID
  total_tasks_completed: number // 总完成任务数
  avg_video_score: number // 平均视频分数
  avg_audio_score: number // 平均音频分数
  avg_sync_score: number // 平均同步分数
  avg_encoding_duration: number // 平均编码时间（毫秒）
  completion_rate: number // 任务完成率
  compliance_rate: number // 质量达标率
  service_days_last_week: number // 最近7天内服务天数
  qos_score: number // 当前QoS评分
}

// QoS评分详情
export interface QosScoreDetails {
  worker_id: string // 工作节点ID
  service_reliability_score: number // 服务可靠性评分
  time_stability_score: number // 时间稳定性评分
  performance_score: number // 性能表现评分
  overall_qos_score: number // 综合QoS评分
  last_update_time: number // 最后更新时间
}

export interface TaskData {
  task_id: string
  broadcaster_id: string
  source_ipfs: string
  requirements: TranscodingRequirement
  status: TaskStatus
  assigned_worker: string | null
  assignment_time: number | null
  result_ipfs: string | null
  completion_time: number | null
  assigned_verifiers: string[]
  qos_proof_id: string | null

  // 新增字段
  publish_time: number
  hw_acceleration_preferred: boolean
}

export interface BroadcasterConfig {
  networkId: string
  nodeUrl: string
  walletUrl: string
  helperUrl: string
  explorerUrl: string
  contractId: string
  broadcasterAccountId?: string
  ipfsConfig: {
    host: string
    port: number
    protocol: string
    gateway: string
  }
}

export interface QosProof {
  task_id: string
  worker_id: string
  timestamp: number
  committee_members: string[]
  committee_leader: string
  video_score: number
  audio_score: number
  sync_score: number
  encoding_start_time: number
  encoding_end_time: number
  video_specs: {
    codec: string
    resolution: string
    bitrate: number
    framerate: number
  }
  specified_gop_scores: {
    gop_id: number
    vmaf_score: number
    hash: string
  }[]
  gop_verification: string
  status: string
}

// 添加WorkerInfo接口
export interface WorkerInfo {
  account_id: string
  has_hw_acceleration: boolean
  last_heartbeat: number
  available: boolean
  current_task: string | null
  qos_score: number
  service_reliability_score: number
  time_stability_score: number
  performance_score: number
  total_tasks_completed: number
  active_days_last_week: number
}
