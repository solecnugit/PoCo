// 任务状态枚举
export enum TaskStatus {
  Published = "Published",
  Assigned = "Assigned",
  Completed = "Completed",
  Verified = "Verified",
  Queued = "Queued",
  OfferCollecting = "OfferCollecting",
}

// 转码要求结构
export interface TranscodingRequirement {
  target_codec: string;
  target_resolution: string;
  target_bitrate: string;
  target_framerate: string;
  additional_params: string;
}

// 任务数据结构
export interface TaskData {
  task_id: string;
  broadcaster_id: string;
  source_ipfs: string;
  requirements: TranscodingRequirement;
  status: TaskStatus;
  assigned_worker?: string;
  assignment_time?: number;
  result_ipfs?: string;
  completion_time?: number;
  assigned_verifiers: string[];
  qos_proof_id?: string;
  publish_time: number;
  hw_acceleration_preferred: boolean;

  keyframe_timestamps?: string[];
  selected_gops?: string[];
}

// 视频规格
export interface VideoSpecification {
  codec: string;
  resolution: string;
  bitrate: number;
  framerate: number;
}

// GOP分数
export interface GopScore {
  timestamp: string;
  vmaf_score: number;
  hash: string;
}

// 验证者质量证明
export interface VerifierQosProof {
  id: string;
  task_id: string;
  verifier_id: string;
  timestamp: number;
  video_specs: VideoSpecification;
  video_score: number;
  gop_scores: GopScore[];
  audio_score?: number;
  sync_score?: number;
  signature: string;
}

// 任务验证状态
export interface TaskVerificationStatus {
  task_id: string;
  verified_by: string[];
  verification_timestamps: number[];
}

// 验证者配置
// export interface VerifierConfig {
//   near_config: {
//     // contract_id: string;
//     // account_id: string;
//     // credentials_path: string;
//     : string;
//     wallet_url: string;
//     node_url: string;
//     helper_url: string;
//     explorer_url: string;
//   };
//   contract_id: string;
//   verifier_account_id: string;
//   credentials_path: string;
//   ipfs_config: {
//     host: string;
//     port: number;
//     protocol: string;
//   };
//   committee_leader_url: string;
//   log_level: string;
//   polling_interval: number;
//   ffmpeg_path: string;
//   vmaf_model_path: string;
// }

// 验证者配置
export interface VerifierConfig {
  nearConfig: {
    // contractId: string;
    // accountId: string;
    // credentialsPath: string;
    networkId: string;
    walletUrl: string;
    nodeUrl: string;
    helperUrl: string;
    explorerUrl: string;
  };
  contractId: string;
  verifierAccountId: string;
  credentialsPath: string;
  ipfsConfig: { host: string; port: number; protocol: string };
  committeeLeaderUrl: string;
  logLevel: string;
  pollingInterval: number;
  ffmpegPath: string;
  vmafModelPath: string;
}
