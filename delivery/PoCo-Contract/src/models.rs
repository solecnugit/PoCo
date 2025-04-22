use near_sdk::{near, AccountId};

/// 任务状态枚举
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug, PartialEq)]
pub enum TaskStatus {
    Published,       // 已发布
    Assigned,        // 已分配
    Completed,       // 已完成
    Verified,        // 已验证
    Queued,          // 已进入队列等待分配
    OfferCollecting, // 正在收集Offer
}

#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
pub struct WorkerTaskCollection {
    pub assigned_tasks: Vec<TaskData>,  // 已分配给Worker的任务
    pub available_tasks: Vec<TaskData>, // 可以提交offer的任务
    pub queued_tasks: Vec<String>,      // 队列中的任务ID列表
}

/// Worker状态返回值
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
pub struct WorkerStatus {
    pub is_registered: bool,          // Worker是否已注册
    pub current_task: Option<String>, // 当前正在处理的任务ID
    pub qos_score: f64,               // 当前QoS评分
}

/// 转码要求结构
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
pub struct TranscodingRequirement {
    pub target_codec: String,
    pub target_resolution: String,
    pub target_bitrate: String,
    pub target_framerate: String,
    pub additional_params: String,
}

/// 任务数据结构
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
pub struct TaskData {
    pub task_id: String,
    pub broadcaster_id: AccountId,
    pub source_ipfs: String,
    pub requirements: TranscodingRequirement,
    pub status: TaskStatus,
    pub assigned_worker: Option<AccountId>,
    pub assignment_time: Option<u64>,
    pub result_ipfs: Option<String>,
    pub completion_time: Option<u64>,
    pub assigned_verifiers: Vec<AccountId>,
    pub qos_proof_id: Option<String>,
    pub publish_time: u64,               // 新增：任务发布时间
    pub hw_acceleration_preferred: bool, // 新增：是否偏好硬件加速

    pub keyframe_timestamps: Option<Vec<String>>, // 关键帧时间戳列表
    pub selected_gops: Option<Vec<String>>,       // 被选中用于验证的GOP时间戳
    pub video_duration: Option<f64>,
    pub frame_count: Option<u32>,
}

/// QoS质量证明的状态
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug, PartialEq)]
pub enum QosProofStatus {
    Pending,  // 等待通过
    Normal,   // 普通验证通过
    Conflict, // 经过补充验证通过
    Manual,   // 经过人工验证通过
}

/// GOP验证的结果
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug, PartialEq)]
pub enum GopVerificationResult {
    Verified,          // GOP验证通过
    ScoreMismatch,     // 分数不匹配
    GopMismatch,       // GOP结构不匹配
    UndeterminedError, // 未确定错误
}

/// GOP分数
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
pub struct GopScore {
    pub timestamp: String, // GOP ID
    pub vmaf_score: f64,   // VMAF分数
    pub hash: String,      // GOP哈希
}

/// 视频规格
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
pub struct VideoSpecification {
    pub codec: String,      // 编解码器
    pub resolution: String, // 分辨率
    pub bitrate: u32,       // 比特率
    pub framerate: f32,     // 帧率
}

/// 共识QoS质量证明结构
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
pub struct ConsensusQosProof {
    // 基本信息
    pub task_id: String,      // 任务ID
    pub worker_id: AccountId, // 工作节点ID
    pub timestamp: u64,       // 时间戳

    // 委员会信息
    pub committee_leader: AccountId, // 提交共识结果的委员会leader

    // 质量分数
    pub video_score: f64, // VMAF分数
    pub audio_score: f64, // PESQ分数
    pub sync_score: f64,  // 同步性分数

    // 视频信息
    pub encoding_start_time: u64,        // 编码开始时间
    pub encoding_end_time: u64,          // 编码结束时间
    pub video_specs: VideoSpecification, // 视频规格
    pub frame_count: u32,

    // GOP验证
    pub specified_gop_scores: Vec<GopScore>, // 特定GOP分数
    pub gop_verification: GopVerificationResult, // GOP验证结果

    // 状态
    pub status: QosProofStatus, // 状态（可以保留，虽然默认是已验证）
}

/// Verifier质量证明结构
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
pub struct VerifierQosProof {
    pub id: String,                      // 证明唯一标识符
    pub task_id: String,                 // 任务ID
    pub verifier_id: AccountId,          // 验证者ID
    pub timestamp: u64,                  // 证明生成时间戳
    pub video_specs: VideoSpecification, // 视频规格

    // 视频质量数据
    pub video_score: f64,          // 整体VMAF评分
    pub gop_scores: Vec<GopScore>, // 使用修改后的GopScore结构

    // 可选字段
    pub audio_score: Option<f64>, // 音频PESQ评分
    pub sync_score: Option<f64>,  // 同步性评分

    pub signature: String, // 验证者签名
}

/// 任务验证状态结构 - 记录哪些验证者已完成验证
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
pub struct TaskVerificationStatus {
    pub task_id: String,                   // 任务ID
    pub verified_by: Vec<AccountId>,       // 已验证的验证者列表
    pub verification_timestamps: Vec<u64>, // 对应的验证时间戳
}

/// 委员会成员信息
#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct CommitteeMemberInfo {
    pub account_id: AccountId, // 账户ID
    pub ip_address: String,    // IP地址
    pub port: u16,             // 端口
    pub is_leader: bool,       // 是否为leader
}

/// 验证者成员信息
#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct VerifierMemberInfo {
    pub account_id: AccountId, // 账户ID
    pub ip_address: String,    // IP地址
    pub port: u16,             // 端口
}

#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
pub struct WorkerPerformance {
    // 基本信息
    pub task_id: String,
    pub worker_id: AccountId,
    pub timestamp: u64,
    pub encoding_duration: u64, // 编码耗时
    pub frame_count: u32,       // 视频总帧数

    // 视频质量相关
    pub video_score: f64,              // VMAF分数，可能为None
    pub video_quality_compliant: bool, // 视频质量是否达标

    // 音频质量相关
    pub audio_score: Option<f64>, // PESQ分数，可能为None (NA)
    pub audio_quality_compliant: Option<bool>, // 音频质量是否达标，可能为None

    // 同步相关
    pub sync_score: Option<f64>,              // 同步分数，可能为None (NA)
    pub sync_quality_compliant: Option<bool>, // 同步质量是否达标，可能为None

    // 规格相关
    pub video_specs: VideoSpecification,
    pub specs_compliant: bool,

    // 总体评价
    pub overall_compliant: bool, // 总体是否达标
}

/// 工作节点每日统计数据
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
pub struct WorkerDailyStats {
    pub date: u64,                // 日期时间戳（YYYYMMDD格式）
    pub tasks_completed: u32,     // 当日完成任务数
    pub tasks_accepted: u32,      // 当日接受任务数
    pub provided_service: bool,   // 当日是否提供服务
    pub encoding_fps: Vec<f64>,   // 单位时间编码
    pub quality_scores: Vec<f64>, // 质量评分列表
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct WorkerPerformanceHistory {
    pub worker_id: AccountId,
    pub performance_records: Vec<WorkerPerformance>, // 最近的性能记录
    pub daily_stats: Vec<WorkerDailyStats>,
    pub total_tasks_completed: u32, // 总完成任务数
    pub last_update_time: u64,      // 最后更新时间

    // QoS评分细分
    pub service_reliability_score: f64, // 服务可靠性评分
    pub time_stability_score: f64,      // 时间稳定性评分
    pub performance_score: f64,         // 性能表现评分

    // 聚合统计数据
    pub avg_video_score: f64,       // 平均视频分数
    pub avg_audio_score: f64,       // 平均音频分数
    pub avg_sync_score: f64,        // 平均同步分数
    pub avg_encoding_duration: u64, // 平均编码时间
    pub completion_rate: f64,       // 任务完成率
    pub compliance_rate: f64,       // 质量达标率
}

/// QoS评分详情结构
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
pub struct QosScoreDetails {
    pub worker_id: AccountId,           // 工作节点ID
    pub service_reliability_score: f64, // 服务可靠性评分
    pub time_stability_score: f64,      // 时间稳定性评分
    pub performance_score: f64,         // 性能表现评分
    pub overall_qos_score: f64,         // 综合QoS评分
    pub last_update_time: u64,          // 最后更新时间
}

/// 工作节点性能统计摘要
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
pub struct WorkerPerformanceSummary {
    pub worker_id: AccountId,        // 工作节点ID
    pub total_tasks_completed: u32,  // 总完成任务数
    pub avg_video_score: f64,        // 平均视频分数
    pub avg_audio_score: f64,        // 平均音频分数
    pub avg_sync_score: f64,         // 平均同步分数
    pub avg_encoding_duration: u64,  // 平均编码时间（毫秒）
    pub completion_rate: f64,        // 任务完成率
    pub compliance_rate: f64,        // 质量达标率
    pub service_days_last_week: u32, // 最近7天内服务天数
    pub qos_score: f64,              // 当前QoS评分
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct VerifierPerformance {
    // 基本信息
    pub task_id: String,
    pub verifier_id: AccountId,
    pub timestamp: u64,

    // 响应时间
    pub response_time: u64,            // 响应时间(毫秒)
    pub response_time_compliant: bool, // 响应时间是否符合要求

    // 验证结果
    pub video_score: Option<f64>, // 验证者给出的VMAF分数
    pub audio_score: Option<f64>, // 验证者给出的PESQ分数
    pub sync_score: Option<f64>,  // 验证者给出的同步分数

    // 总体评价
    pub overall_consensus_match: bool, // 总体是否与共识匹配
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct WorkerInfo {
    pub account_id: AccountId,
    pub has_hw_acceleration: bool, // 是否支持硬件加速
    // pub capabilities: Vec<String>,    // 支持的编码格式、分辨率等
    pub last_heartbeat: u64,          // 最后一次心跳时间
    pub available: bool,              // 是否可用
    pub current_task: Option<String>, // 当前正在处理的任务
    pub qos_score: f64,               // 综合QoS评分

    //新增字段
    pub service_reliability_score: f64, // 服务可靠性评分
    pub time_stability_score: f64,      // 时间稳定性评分
    pub performance_score: f64,         // 性能表现评分
    pub total_tasks_completed: u32,     // 总完成任务数
    pub active_days_last_week: u32,     // 最近7天活跃天数
}

/// Worker Offer结构
#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct WorkerOffer {
    pub worker_id: AccountId,
    pub task_id: String,
    pub timestamp: u64, // 提交offer的时间
                        // pub estimated_completion_time: u64, // 预计完成时间(毫秒)
}

/// 任务共识详细信息结构体
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug)]
pub struct TaskConsensusDetails {
    pub task: TaskData,                                // 任务基本信息
    pub consensus_proof: Option<ConsensusQosProof>,    // 共识证明（如果有）
    pub worker_performance: Option<WorkerPerformance>, // 工作节点表现（如果有）
}

// #[near(serializers = [json, borsh])]
// #[derive(Clone)]
// pub struct TaskQueue {
//     pub pending_tasks: Vec<String>, // 待处理任务ID列表
//                                     // pub last_assignment_time: u64,  // 最后一次分配时间
// }
