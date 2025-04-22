mod models;

// 导出所有模型，方便使用
pub use models::*;

use near_sdk::store::{IterableMap, Vector};
use near_sdk::{env, log, near, AccountId};

const ONE_DAY_MS: u64 = 24 * 60 * 60 * 1000; // 一天的毫秒数
const SEVEN_DAYS_MS: u64 = 7 * ONE_DAY_MS; // 七天的毫秒数

// 合约状态定义
#[near(contract_state)]
pub struct MediaTranscodingContract {
    owner_id: AccountId,
    tasks: IterableMap<String, TaskData>,
    task_ids: Vector<String>,
    pending_tasks: Vec<String>,
    consensus_proofs: IterableMap<String, ConsensusQosProof>,
    verifier_members: Vec<VerifierMemberInfo>,
    committee_members: Vec<CommitteeMemberInfo>,
    worker_performances: IterableMap<String, WorkerPerformance>,
    worker_performance_history: IterableMap<AccountId, WorkerPerformanceHistory>,
    task_offers: IterableMap<String, Vec<WorkerOffer>>,
    verifier_performances: IterableMap<String, VerifierPerformance>,
    active_workers: IterableMap<AccountId, WorkerInfo>,
    task_verification_status: IterableMap<String, TaskVerificationStatus>, // 任务验证状态 (task_id -> status)
    verifier_proofs: IterableMap<String, VerifierQosProof>,                // id -> proof
}

// 默认实现
impl Default for MediaTranscodingContract {
    fn default() -> Self {
        Self {
            owner_id: "syspoco.testnet"
                .parse()
                .expect("Invalid default account ID"),
            tasks: IterableMap::new(b"t"),
            task_ids: Vector::new(b"i"),
            consensus_proofs: IterableMap::new(b"p"),
            committee_members: Vec::new(),
            verifier_members: Vec::new(),
            worker_performance_history: IterableMap::new(b"h"),
            worker_performances: IterableMap::new(b"w"),
            verifier_performances: IterableMap::new(b"v"),
            active_workers: IterableMap::new(b"a"),
            task_offers: IterableMap::new(b"o"),
            pending_tasks: Vec::new(),
            task_verification_status: IterableMap::new(b"z"),
            verifier_proofs: IterableMap::new(b"a"),
        }
    }
}

// 所有合约方法实现
#[near]
impl MediaTranscodingContract {
    // 初始化方法
    #[init]
    pub fn new(
        owner_id: Option<AccountId>,
        committee_members: Option<Vec<CommitteeMemberInfo>>,
        verifier_members: Option<Vec<VerifierMemberInfo>>,
    ) -> Self {
        let owner = owner_id.unwrap_or_else(|| env::predecessor_account_id());

        Self {
            owner_id: owner,
            tasks: IterableMap::new(b"t"),
            task_ids: Vector::new(b"i"),
            consensus_proofs: IterableMap::new(b"p"),
            worker_performance_history: IterableMap::new(b"h"),
            committee_members: committee_members.unwrap_or_default(),
            verifier_members: verifier_members.unwrap_or_default(),
            pending_tasks: Vec::new(),
            worker_performances: IterableMap::new(b"w"),
            verifier_performances: IterableMap::new(b"v"),
            active_workers: IterableMap::new(b"a"),
            task_offers: IterableMap::new(b"o"),
            task_verification_status: IterableMap::new(b"z"),
            verifier_proofs: IterableMap::new(b"a"),
        }
    }

    //
    // worker相关方法
    //

    pub fn get_tasks_for_worker(&mut self, worker_id: Option<AccountId>) -> WorkerTaskCollection {
        let account_id = worker_id.unwrap_or_else(|| env::predecessor_account_id());

        // 初始化返回结构
        let mut result = WorkerTaskCollection {
            assigned_tasks: Vec::new(),
            available_tasks: Vec::new(),
            queued_tasks: Vec::new(),
        };

        // 1. 首先检查Worker状态
        let worker_info = self.active_workers.get(&account_id);
        let is_worker_available = worker_info
            .map(|info| info.available && info.current_task.is_none())
            .unwrap_or(false);

        if !is_worker_available {
            // Worker不可用或已有任务，只返回已分配任务
            result.assigned_tasks = self
                .tasks
                .values()
                .filter(|task| {
                    task.status == TaskStatus::Assigned
                        && task.assigned_worker.as_ref() == Some(&account_id)
                })
                .map(|task| task.clone())
                .collect();

            return result;
        }

        // 2. 检查Worker提交的offer是否已被接受
        let assigned_tasks: Vec<TaskData> = self
            .tasks
            .values()
            .filter(|task| {
                task.status == TaskStatus::Assigned
                    && task.assigned_worker.as_ref() == Some(&account_id)
            })
            .map(|task| task.clone())
            .collect();

        if !assigned_tasks.is_empty() {
            result.assigned_tasks = assigned_tasks;
            return result;
        }

        // 3. 如果没有已分配任务，尝试从队列自动分配一个任务
        if !self.pending_tasks.is_empty() && is_worker_available {
            let task_id = self.pending_tasks[0].clone();
            // 自动将队列中的任务分配给Worker
            let assignment_success = self.assign_task_to_worker(&task_id, &account_id);

            if assignment_success {
                if let Some(task) = self.tasks.get(&task_id) {
                    result.assigned_tasks.push(task.clone());
                    return result;
                }
            }
        }

        // 4. 如果没有队列任务，返回可提交offer的任务
        result.available_tasks = self
            .tasks
            .values()
            .filter(|task| task.status == TaskStatus::OfferCollecting)
            .map(|task| task.clone())
            .collect();

        // 5. 同时返回队列中剩余的任务ID
        result.queued_tasks = self.pending_tasks.clone();

        result
    }

    // worker 心跳

    /// 优化的Worker心跳方法
    pub fn worker_heartbeat(&mut self) -> WorkerStatus {
        let worker_id = env::predecessor_account_id();
        let current_timestamp = env::block_timestamp() / 1_000_000; // 纳秒转毫秒

        // 首先检查worker是否存在
        if !self.active_workers.contains_key(&worker_id) {
            return WorkerStatus {
                is_registered: false,
                current_task: None,
                qos_score: 0.0,
            };
        }

        // 更新worker心跳
        let mut worker_available = false;
        let mut current_task = None;

        if let Some(mut worker_info) = self.active_workers.get_mut(&worker_id) {
            worker_info.last_heartbeat = current_timestamp;
            worker_available = worker_info.available;
            current_task = worker_info.current_task.clone();
        }

        // 更新增强型历史记录
        if let Some(mut enhanced_history) = self.worker_performance_history.get(&worker_id).cloned()
        {
            // 获取当前日期
            let current_day_timestamp = current_timestamp / ONE_DAY_MS * ONE_DAY_MS;

            // 查找或创建今天的统计数据
            if let Some(index) = enhanced_history
                .daily_stats
                .iter()
                .position(|stat| stat.date == current_day_timestamp)
            {
                // 更新现有统计
                let mut stats = enhanced_history.daily_stats[index].clone();
                stats.provided_service = true;
                enhanced_history.daily_stats[index] = stats;
            } else {
                // 创建新的统计
                let stats = WorkerDailyStats {
                    date: current_day_timestamp,
                    tasks_completed: 0,
                    tasks_accepted: 0,
                    provided_service: true,
                    encoding_fps: Vec::new(),
                    quality_scores: Vec::new(),
                };
                enhanced_history.daily_stats.push(stats);
            }

            // 计算时间稳定性评分
            enhanced_history.time_stability_score =
                Self::calculate_time_stability_locally(&enhanced_history);

            // 保存更新后的历史记录
            self.worker_performance_history
                .insert(worker_id.clone(), enhanced_history);
        }

        // 返回Worker当前状态，这样Worker可以根据返回值决定下一步行为
        WorkerStatus {
            is_registered: true,
            current_task,
            qos_score: self
                .active_workers
                .get(&worker_id)
                .map_or(0.0, |w| w.qos_score),
        }
    }

    // 局部辅助函数，避免借用self
    fn calculate_time_stability_locally(history: &WorkerPerformanceHistory) -> f64 {
        // 实现与原calculate_time_stability相同的逻辑，但不需要借用self

        // 获取过去7天的每日统计
        let current_time = env::block_timestamp() / 1_000_000; // 纳秒转毫秒
        let week_ago = if current_time > SEVEN_DAYS_MS {
            // println!(
            //     // "inside calculate_time_stability_locally,${:?}",
            //     current_time - SEVEN_DAYS_MS,
            // );
            current_time - SEVEN_DAYS_MS
        } else {
            0 // 如果当前时间小于7天的纳秒数，则使用0作为起始时间
        };

        let recent_stats: Vec<&WorkerDailyStats> = history
            .daily_stats
            .iter()
            .filter(|stat| stat.date > week_ago)
            .collect();

        if recent_stats.is_empty() {
            return 0.3; // 默认中等评分
        }

        // 1. 计算日均任务量(ADT)
        let total_tasks: u32 = recent_stats.iter().map(|stat| stat.tasks_completed).sum();

        let adt = total_tasks as f64 / 7.0; // 每日平均任务数

        // 归一化ADT（假设每日10个任务为满分）
        let normalized_adt = (adt / 10.0).min(1.0);

        // 2. 计算服务覆盖率(SC)
        let active_days = recent_stats
            .iter()
            .filter(|stat| stat.provided_service)
            .count();

        let sc = active_days as f64 / 7.0; // 7天窗口

        // 加权计算时间稳定性
        let delta1 = 0.5; // 日均任务量权重
        let delta2 = 0.5; // 服务覆盖率权重

        delta1 * normalized_adt + delta2 * sc
    }

    // worker 注册

    pub fn register_worker(&mut self, has_hw_acceleration: bool) -> bool {
        let worker_id = env::predecessor_account_id();

        let worker_info = WorkerInfo {
            account_id: worker_id.clone(),
            has_hw_acceleration,
            last_heartbeat: env::block_timestamp() / 1_000_000,
            available: true,
            current_task: None,
            qos_score: 0.3, // 初始中等评分
            service_reliability_score: 0.3,
            time_stability_score: 0.3,
            performance_score: 0.3,
            total_tasks_completed: 0,
            active_days_last_week: 0,
        };

        self.active_workers.insert(worker_id.clone(), worker_info);

        // 初始化历史记录
        let enhanced_history = WorkerPerformanceHistory {
            worker_id: worker_id.clone(),
            performance_records: Vec::new(),
            daily_stats: Vec::new(),
            total_tasks_completed: 0,
            last_update_time: env::block_timestamp(),
            service_reliability_score: 0.3,
            time_stability_score: 0.3,
            performance_score: 0.3,
            avg_video_score: 0.0,
            avg_audio_score: 0.0,
            avg_sync_score: 0.0,
            avg_encoding_duration: 0,
            completion_rate: 0.0,
            compliance_rate: 0.0,
        };

        self.worker_performance_history
            .insert(worker_id.clone(), enhanced_history);

        // // 初始化当天的统计数据
        // let current_timestamp = env::block_timestamp() / 1_000_000;

        // println!("current_timestamp is  {:?}", current_timestamp);
        // let current_day_timestamp =
        //     current_timestamp / (24 * 60 * 60 * 1_000) * (24 * 60 * 60 * 1_000);

        // println!("current_day_timestamp is {:?}", current_day_timestamp);

        // let today_stats = WorkerDailyStats {
        //     date: current_day_timestamp,
        //     tasks_completed: 0,
        //     tasks_accepted: 0,
        //     provided_service: true, // 注册即视为当天提供服务
        //     encoding_durations: Vec::new(),
        //     quality_scores: Vec::new(),
        // };

        // // 将今天的统计数据添加到历史记录
        // let mut updated_history = self
        //     .worker_performance_history
        //     .get(&worker_id)
        //     .unwrap()
        //     .clone();
        // updated_history.daily_stats.push(today_stats);
        // self.worker_performance_history
        //     .insert(worker_id.clone(), updated_history);

        // 检查队列中是否有待处理任务，并为队列任务提交offer
        self.auto_submit_offer_for_queued_task(&worker_id);

        true
    }

    //
    // verifier相关方法
    //

    /// 初始化验证者成员
    pub fn init_verifiers(&mut self, verifier_members: Vec<VerifierMemberInfo>) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Only the owner can initialize verifiers"
        );
        self.verifier_members = verifier_members;
    }

    /// 使用硬编码方式初始化验证者成员
    pub fn initialize_verifiers_with_hardcoded_members(&mut self) {
        let verifier_members = vec![
            VerifierMemberInfo {
                account_id: "verifier1.testnet".parse().expect("Invalid account ID"),
                ip_address: "10.24.10.203".to_string(),
                port: 9000,
            },
            VerifierMemberInfo {
                account_id: "verifier2.testnet".parse().expect("Invalid account ID"),
                ip_address: "10.24.226.115".to_string(),
                port: 9000,
            },
            VerifierMemberInfo {
                account_id: "verifier3.testnet".parse().expect("Invalid account ID"),
                ip_address: "10.24.137.36".to_string(),
                port: 9000,
            },
        ];

        // 调用合约的init_verifiers方法
        self.init_verifiers(verifier_members);
    }

    /// 获取验证者成员
    pub fn get_verifier_members(&self) -> Vec<VerifierMemberInfo> {
        self.verifier_members.clone()
    }

    // /// 为完成的任务分配验证者
    // fn assign_verifiers(&mut self, task_id: &String) -> bool {
    //     if let Some(mut task) = self.tasks.get(task_id) {
    //         if task.status != TaskStatus::Completed {
    //             return false;
    //         }

    //         // 根据系统配置选择验证者
    //         // 这里可以实现多种验证者选择策略
    //         let verifiers = self.select_verifiers(task_id.clone()); // 选择3个验证者

    //         task.assigned_verifiers = verifiers;
    //         self.tasks.insert(task_id.clone(), &task);

    //         true
    //     } else {
    //         false
    //     }
    // }

    // /// 选择验证者
    // pub fn select_verifiers(&mut self, task_id: String) -> Vec<AccountId> {
    //     let mut task = self.tasks.get(&task_id).expect("Task not found").clone();

    //     assert_eq!(
    //         task.status,
    //         TaskStatus::Completed,
    //         "Task is not in completed state"
    //     );
    //     assert!(
    //         task.broadcaster_id == env::predecessor_account_id()
    //             || self.owner_id == env::predecessor_account_id(),
    //         "Only broadcaster or owner can select verifiers"
    //     );

    //     // 获取所有验证者
    //     let all_verifiers: Vec<AccountId> = self
    //         .verifier_members
    //         .iter()
    //         .map(|member| member.account_id.clone())
    //         .collect();

    //     // 确保有足够的验证者
    //     assert!(all_verifiers.len() >= 2, "Not enough verifiers available");

    //     // 使用区块高度作为随机种子
    //     let seed = env::block_height();

    //     // 简单的选择算法：从可用验证者中选择两个
    //     // 注意：这不是真正的随机，但对MVP足够了
    //     let mut selected_verifiers = Vec::with_capacity(2);

    //     for i in 0..2 {
    //         // 使用区块高度和索引计算一个伪随机索引
    //         let index = ((seed + i as u64) % all_verifiers.len() as u64) as usize;
    //         selected_verifiers.push(all_verifiers[index].clone());
    //     }

    //     // 确保没有重复的验证者（如果发生这种情况）
    //     selected_verifiers.sort();
    //     selected_verifiers.dedup();

    //     // 如果去重后不足两个，补充一个
    //     if selected_verifiers.len() < 2 && all_verifiers.len() > 2 {
    //         for verifier in &all_verifiers {
    //             if !selected_verifiers.contains(verifier) {
    //                 selected_verifiers.push(verifier.clone());
    //                 break;
    //             }
    //         }
    //     }

    //     task.assigned_verifiers = selected_verifiers.clone();
    //     self.tasks.insert(task_id.clone(), task);

    //     log!(
    //         "Selected verifiers for task {}: {:?}",
    //         task_id,
    //         selected_verifiers
    //     );
    //     selected_verifiers
    // }

    /// 为完成的任务选择并分配验证者
    pub fn select_and_assign_verifiers(&mut self, task_id: String) -> Vec<AccountId> {
        let mut task = self.tasks.get(&task_id).expect("Task not found").clone();

        assert_eq!(
            task.status,
            TaskStatus::Completed,
            "Task is not in completed state"
        );

        // #[cfg(test)]
        // println!(
        //     "当前调用者: {:?}, 任务broadcaster: {:?}",
        //     env::predecessor_account_id(),
        //     task.broadcaster_id
        // );
        // assert!(
        //     task.broadcaster_id == env::predecessor_account_id()
        //         || self.owner_id == env::predecessor_account_id(),
        //     "Only broadcaster or owner can select verifiers, "
        // );

        // 获取所有验证者
        let all_verifiers: Vec<AccountId> = self
            .verifier_members
            .iter()
            .map(|member| member.account_id.clone())
            .collect();

        // 确保有足够的验证者
        assert!(all_verifiers.len() >= 2, "Not enough verifiers available");

        // 使用区块高度作为随机种子
        let seed = env::block_height();

        // 简单的选择算法
        let mut selected_verifiers = Vec::with_capacity(2);

        for i in 0..2 {
            let index = ((seed + i as u64) % all_verifiers.len() as u64) as usize;
            selected_verifiers.push(all_verifiers[index].clone());
        }

        // 确保没有重复
        selected_verifiers.sort();
        selected_verifiers.dedup();

        // 补充验证者
        if selected_verifiers.len() < 2 && all_verifiers.len() > 2 {
            for verifier in &all_verifiers {
                if !selected_verifiers.contains(verifier) {
                    selected_verifiers.push(verifier.clone());
                    break;
                }
            }
        }

        // 更新任务
        task.assigned_verifiers = selected_verifiers.clone();
        self.tasks.insert(task_id.clone(), task);

        log!(
            "Selected verifiers for task {}: {:?}",
            task_id,
            selected_verifiers
        );
        selected_verifiers
    }

    /// 请求一个补充验证者
    pub fn request_supplemental_verifier(&mut self, task_id: String) -> Option<AccountId> {
        let task = self.tasks.get(&task_id).expect("Task not found").clone();

        // 确保任务状态是Completed
        assert_eq!(
            task.status,
            TaskStatus::Completed,
            "Task is not in completed state"
        );

        let caller = env::predecessor_account_id();
        let is_leader = self
            .committee_members
            .iter()
            .any(|member| member.account_id == caller && member.is_leader);

        assert!(
            is_leader,
            "Only committee leader can request supplemental verifier"
        );

        // 获取已分配的验证者列表
        let assigned_verifiers = &task.assigned_verifiers;

        // 获取所有验证者
        let all_verifiers: Vec<AccountId> = self
            .verifier_members
            .iter()
            .map(|member| member.account_id.clone())
            .collect();

        // 过滤掉已分配的验证者
        let available_verifiers: Vec<AccountId> = all_verifiers
            .into_iter()
            .filter(|v| !assigned_verifiers.contains(v))
            .collect();

        // 如果没有可用的验证者了，返回None
        if available_verifiers.is_empty() {
            return None;
        }

        // 使用区块高度作为随机种子选择一个
        let seed = env::block_height();
        let index = (seed % available_verifiers.len() as u64) as usize;
        let selected_verifier = available_verifiers[index].clone();

        // 将选中的验证者添加到任务的验证者列表
        let mut updated_task = task;
        updated_task
            .assigned_verifiers
            .push(selected_verifier.clone());
        self.tasks.insert(task_id.clone(), updated_task);

        log!(
            "Added supplemental verifier for task {}: {:?}",
            task_id,
            selected_verifier
        );
        Some(selected_verifier)
    }

    /// 获取分配给特定验证者的任务
    /// @param verifier_id: 验证者ID
    /// @param only_unverified: 是否只返回未验证的任务(默认true)
    /// @return 任务列表
    pub fn query_assigned_tasks(
        &self,
        verifier_id: AccountId,
        only_unverified: Option<bool>,
    ) -> Vec<TaskData> {
        let filter_unverified = only_unverified.unwrap_or(true);

        self.tasks
            .values()
            .filter(|task| {
                // 基本条件：任务分配给该验证者且状态为Completed
                let is_assigned = task.assigned_verifiers.contains(&verifier_id)
                    && task.status == TaskStatus::Completed;

                if !is_assigned {
                    return false;
                }

                // 如果需要过滤已验证任务
                if filter_unverified {
                    // 检查验证状态
                    if let Some(status) = self.task_verification_status.get(&task.task_id) {
                        return !status.verified_by.contains(&verifier_id);
                    }
                }

                true
            })
            .map(|task| task.clone())
            .collect()
    }

    /// 提交验证者质量证明
    /// @param proof: 验证者质量证明
    /// @return 是否成功
    pub fn submit_verifier_proof(&mut self, proof: VerifierQosProof) -> bool {
        let verifier_id = env::predecessor_account_id();

        // 验证调用者是否是证明中的验证者
        assert_eq!(
            verifier_id, proof.verifier_id,
            "只有证明中指定的验证者可以提交该证明"
        );

        // 验证调用者是否是该任务的指定验证者
        let task = self.tasks.get(&proof.task_id).expect("任务不存在");
        assert!(
            task.assigned_verifiers.contains(&verifier_id),
            "只有被分配的验证者可以提交证明"
        );

        // 检查是否已经提交过
        if let Some(status) = self.task_verification_status.get(&proof.task_id) {
            assert!(
                !status.verified_by.contains(&verifier_id),
                "该验证者已提交过验证结果"
            );
        }
        // 验证提交的GOP结果是否与指定的GOP匹配
        let task = self.tasks.get(&proof.task_id).expect("任务不存在");
        let required_gops = task.selected_gops.as_ref().expect("未选择验证GOP");

        // 提取提交的GOP时间戳
        let submitted_timestamps: Vec<String> = proof
            .gop_scores
            .iter()
            .map(|score| score.timestamp.clone())
            .collect();

        // 验证是否包含所有必需的GOP
        for gop in required_gops {
            assert!(
                submitted_timestamps.contains(gop),
                "缺少必需的GOP验证结果: {}",
                gop
            );
        }

        // 生成证明ID(如果未提供)
        let proof_id = if proof.id.is_empty() {
            format!("proof-{}-{}", proof.task_id, verifier_id)
        } else {
            proof.id.clone()
        };

        let proof_cpy = proof.clone();

        // 创建包含ID的证明副本
        let proof_with_id = VerifierQosProof {
            id: proof_id.clone(),
            ..proof_cpy
        };

        // 存储验证证明
        self.verifier_proofs.insert(proof_id, proof_with_id);

        // 更新任务验证状态
        let mut status = self
            .task_verification_status
            .get(&proof.task_id)
            .cloned()
            .unwrap_or(TaskVerificationStatus {
                task_id: proof.task_id.clone(),
                verified_by: Vec::new(),
                verification_timestamps: Vec::new(),
            });

        status.verified_by.push(verifier_id.clone());
        status.verification_timestamps.push(env::block_timestamp());
        self.task_verification_status
            .insert(proof.task_id.clone(), status);

        log!(
            "验证者 {} 提交了任务 {} 的质量证明",
            verifier_id,
            proof.task_id
        );

        true
    }

    /// 获取特定验证者对特定任务的验证结果
    /// @param task_id: 任务ID
    /// @param verifier_id: 验证者ID
    /// @return 验证结果(如果存在)
    pub fn get_verifier_proof(
        &self,
        task_id: String,
        verifier_id: AccountId,
    ) -> Option<VerifierQosProof> {
        self.verifier_proofs
            .values()
            .find(|proof| proof.task_id == task_id && proof.verifier_id == verifier_id)
            .cloned()
    }

    /// 获取特定任务的所有验证结果
    /// @param task_id: 任务ID
    /// @return 验证结果列表
    pub fn get_task_proofs(&self, task_id: String) -> Vec<VerifierQosProof> {
        self.verifier_proofs
            .values()
            .filter(|proof| proof.task_id == task_id)
            .map(|proof| proof.clone())
            .collect()
    }

    /// 获取任务验证状态
    /// @param task_id: 任务ID
    /// @return 验证状态(如果存在)
    pub fn get_task_verification_status(&self, task_id: String) -> Option<TaskVerificationStatus> {
        self.task_verification_status.get(&task_id).cloned()
    }

    //
    // 委员会相关方法
    //

    pub fn initialize_committee_with_hardcoded_members(&mut self) {
        let committee_members = vec![
            CommitteeMemberInfo {
                account_id: "pocoleader.testnet".parse().expect("Invalid account ID"),
                ip_address: "10.24.136.124".to_string(),
                port: 8000,
                is_leader: true,
            },
            CommitteeMemberInfo {
                account_id: "follower1.testnet".parse().expect("Invalid account ID"),
                ip_address: "10.24.216.33".to_string(),
                port: 8000,
                is_leader: false,
            },
            CommitteeMemberInfo {
                account_id: "follower2.testnet".parse().expect("Invalid account ID"),
                ip_address: "10.24.198.225".to_string(),
                port: 8000,
                is_leader: false,
            },
            CommitteeMemberInfo {
                account_id: "follower3.testnet".parse().expect("Invalid account ID"),
                ip_address: "10.24.161.186".to_string(),
                port: 8000,
                is_leader: false,
            },
        ];

        // 调用合约的init_committee方法
        self.init_committee(committee_members);
    }

    /// 初始化委员会成员
    pub fn init_committee(&mut self, committee_members: Vec<CommitteeMemberInfo>) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Only the owner can initialize committee"
        );
        self.committee_members = committee_members;
    }

    /// 获取委员会成员
    pub fn get_committee_members(&self) -> Vec<CommitteeMemberInfo> {
        self.committee_members.clone()
    }

    /// 获取委员会leader
    pub fn get_committee_leader(&self) -> Option<CommitteeMemberInfo> {
        self.committee_members
            .iter()
            .find(|member| member.is_leader)
            .cloned()
    }

    //
    // 任务管理相关方法
    //

    /// 发布任务
    pub fn publish_task(
        &mut self,
        source_ipfs: String,
        requirements: TranscodingRequirement,
        hw_acceleration_preferred: bool,
    ) -> String {
        let broadcaster_id = env::predecessor_account_id();
        let task_id = format!("task-{}-{}", broadcaster_id, env::block_height());

        let task = TaskData {
            task_id: task_id.clone(),
            broadcaster_id,
            source_ipfs,
            requirements: requirements.clone(),
            status: TaskStatus::OfferCollecting,
            hw_acceleration_preferred: hw_acceleration_preferred,
            assigned_worker: None,
            assignment_time: None,
            result_ipfs: None,
            completion_time: None,
            assigned_verifiers: Vec::new(),
            qos_proof_id: None,
            publish_time: env::block_timestamp(),
            keyframe_timestamps: None,
            selected_gops: None,
            video_duration: None,
            frame_count: None,
        };

        self.tasks.insert(task_id.clone(), task);
        self.task_offers.insert(task_id.clone(), Vec::new());

        task_id
    }

    /// Worker提交offer
    pub fn submit_offer(&mut self, task_id: String) -> bool {
        let worker_id = env::predecessor_account_id();

        // 检查Worker是否可用
        if let Some(worker_info) = self.active_workers.get(&worker_id) {
            if !worker_info.available || worker_info.current_task.is_some() {
                // Worker不可用或已有任务
                return false;
            }
        } else {
            // Worker未注册
            return false;
        }

        // 检查任务是否存在
        if let Some(task) = self.tasks.get(&task_id) {
            // 更新每日统计中的任务接受数
            if let Some(mut enhanced_history) =
                self.worker_performance_history.get(&worker_id).cloned()
            {
                // 获取当前日期
                let current_timestamp = env::block_timestamp() / 1_000_000;
                let current_day_timestamp =
                    current_timestamp / (24 * 60 * 60 * 1_000) * (24 * 60 * 60 * 1_000);

                // 更新任务接受统计
                if let Some(index) = enhanced_history
                    .daily_stats
                    .iter()
                    .position(|stat| stat.date == current_day_timestamp)
                {
                    let mut stats = enhanced_history.daily_stats[index].clone();
                    stats.tasks_accepted += 1;
                    enhanced_history.daily_stats[index] = stats;
                } else {
                    let stats = WorkerDailyStats {
                        date: current_day_timestamp,
                        tasks_completed: 0,
                        tasks_accepted: 1,
                        provided_service: true,
                        encoding_fps: Vec::new(),
                        quality_scores: Vec::new(),
                    };
                    enhanced_history.daily_stats.push(stats);
                }

                // 更新服务可靠性评分
                enhanced_history.service_reliability_score =
                    self.calculate_service_reliability(&enhanced_history);

                // 保存更新
                self.worker_performance_history
                    .insert(worker_id.clone(), enhanced_history);
            }

            // 根据任务状态处理
            match task.status {
                TaskStatus::OfferCollecting => {
                    // 创建offer
                    let offer = WorkerOffer {
                        worker_id: worker_id.clone(),
                        task_id: task_id.clone(),
                        timestamp: env::block_timestamp(),
                    };

                    let mut offers = self.task_offers.get(&task_id).unwrap().clone();
                    offers.push(offer);
                    self.task_offers.insert(task_id.clone(), offers);

                    true
                }
                TaskStatus::Queued => {
                    // 队列任务处理
                    let offer = WorkerOffer {
                        worker_id: worker_id.clone(),
                        task_id: task_id.clone(),
                        timestamp: env::block_timestamp(),
                    };

                    let mut offers = Vec::new();
                    offers.push(offer);
                    self.task_offers.insert(task_id.clone(), offers);

                    self.assign_task_to_worker(&task_id, &worker_id)
                }
                _ => false,
            }
        } else {
            false
        }
    }

    /// 检查广播者所有处于收集offer状态的任务是否超时
    pub fn check_broadcaster_offer_timeout(&mut self, broadcaster_id: AccountId) -> bool {
        let current_time = env::block_timestamp();
        let offer_collection_time = 8 * 1_000_000_000; // 8秒，纳秒为单位

        // 为避免同时修改集合的问题，先收集需要处理的任务ID
        let tasks_to_check: Vec<String> = self
            .tasks
            .values()
            .filter(|task| {
                task.broadcaster_id == broadcaster_id
                    && task.status == TaskStatus::OfferCollecting
                    && current_time - task.publish_time > offer_collection_time
            })
            .map(|task| task.task_id.clone())
            .collect();

        // 如果没有需要处理的任务，直接返回
        if tasks_to_check.is_empty() {
            return false;
        }

        // 处理收集到的任务
        for task_id in tasks_to_check {
            // 修正这里的类型问题
            if let Some(offers) = self.task_offers.get(&task_id) {
                if !offers.is_empty() {
                    // 有offer，选择最佳的
                    self.select_best_offer(&task_id);
                } else {
                    // 没有offer或offers为空，将任务加入队列
                    self.move_task_to_queue(&task_id);
                }
            } else {
                // 如果没有offers记录，也将任务加入队列
                self.move_task_to_queue(&task_id);
            }

            log!("任务 {} 的offer超时检查已处理", task_id);
        }
        true
        // true // 只要处理了任务就返回true
    }

    pub fn check_offer_timeout(&mut self, task_id: String) -> bool {
        if let Some(task) = self.tasks.get(&task_id) {
            if task.status != TaskStatus::OfferCollecting {
                return false;
            }

            let current_time = env::block_timestamp();
            let offer_collection_time = 8 * 1_000_000_000; // 8秒，纳秒为单位

            if current_time - task.publish_time > offer_collection_time {
                // 收集时间结束，选择最佳worker
                let mut offers = self.task_offers.get(&task_id).unwrap();

                if !offers.is_empty() {
                    // 有offer，选择最佳的
                    self.select_best_offer(&task_id);
                } else {
                    // 没有offer，将任务加入队列
                    self.move_task_to_queue(&task_id);
                }

                return true;
            }
        }

        false
    }

    /// 将任务移到队列中
    fn move_task_to_queue(&mut self, task_id: &String) -> bool {
        if let Some(mut task) = self.tasks.get(task_id).cloned() {
            // 只更新状态
            task.status = TaskStatus::Queued;
            self.tasks.insert(task_id.clone(), task.clone());

            // 直接添加到待处理任务列表
            self.pending_tasks.push(task_id.clone());
            true
        } else {
            false
        }
    }

    /// 选择最佳offer并分配任务
    fn select_best_offer(&mut self, task_id: &String) -> bool {
        let offers = self.task_offers.get(task_id).unwrap();
        if offers.is_empty() {
            return false;
        }

        // 获取任务信息
        let hw_acceleration_preferred = if let Some(task) = self.tasks.get(task_id) {
            task.hw_acceleration_preferred
        } else {
            false
        };

        // 选择最佳worker
        let mut best_worker: Option<AccountId> = None;
        let mut best_score = 0.0;

        for offer in offers {
            let worker_id = &offer.worker_id;

            // 获取QoS评分
            let worker_score =
                if let Some(enhanced_history) = self.worker_performance_history.get(worker_id) {
                    // 使用三维度综合评分
                    self.calculate_worker_qos_score(enhanced_history)
                } else {
                    0.3 // 默认中等评分
                };

            // 考虑硬件加速
            let final_score = if hw_acceleration_preferred {
                if let Some(worker_info) = self.active_workers.get(worker_id) {
                    if worker_info.has_hw_acceleration {
                        worker_score + 0.1
                    } else {
                        worker_score
                    }
                } else {
                    worker_score
                }
            } else {
                worker_score
            };

            if best_worker.is_none() || final_score > best_score {
                best_worker = Some(worker_id.clone());
                best_score = final_score;
            }
        }

        if let Some(worker_id) = best_worker {
            self.assign_task_to_worker(task_id, &worker_id)
        } else {
            false
        }
    }

    //
    // 队列管理与心跳
    //

    /// 自动为队列中的任务提交offer并立即处理
    fn auto_submit_offer_for_queued_task(&mut self, worker_id: &AccountId) -> bool {
        // 获取队列
        if self.pending_tasks.is_empty() {
            return false;
        }

        // 取队头任务
        let task_id = self.pending_tasks.remove(0);

        // 检查任务是否存在且状态为Queued
        if let Some(task) = self.tasks.get(&task_id) {
            if task.status != TaskStatus::Queued {
                return false;
            }

            // 创建offer
            let offer = WorkerOffer {
                worker_id: worker_id.clone(),
                task_id: task_id.clone(),
                timestamp: env::block_timestamp(),
            };

            // 将offer添加到任务
            let mut offers = Vec::new();
            offers.push(offer);
            self.task_offers.insert(task_id.clone(), offers);

            // 直接分配任务给worker
            self.assign_task_to_worker(&task_id, worker_id)
        } else {
            false
        }
    }

    /// 将任务分配给Worker
    fn assign_task_to_worker(&mut self, task_id: &String, worker_id: &AccountId) -> bool {
        if let Some(mut task) = self.tasks.get(task_id).cloned() {
            if task.status != TaskStatus::OfferCollecting && task.status != TaskStatus::Queued {
                return false;
            }

            if let Some(worker_info) = self.active_workers.get(worker_id) {
                // 先检查条件
                if !worker_info.available || worker_info.current_task.is_some() {
                    return false;
                }

                // 创建一个修改后的 worker_info 副本
                let mut updated_worker_info = worker_info.clone();

                // 更新任务状态
                task.status = TaskStatus::Assigned;
                task.assigned_worker = Some(worker_id.clone());
                task.assignment_time = Some(env::block_timestamp());
                self.tasks.insert(task_id.clone(), task);

                // 更新Worker状态
                updated_worker_info.current_task = Some(task_id.clone());
                self.active_workers
                    .insert(worker_id.clone(), updated_worker_info);

                true
            } else {
                false
            }
        } else {
            false
        }
    }

    // /// 分配任务给工作节点
    // pub fn assign_task(&mut self, task_id: String, worker_id: AccountId) -> bool {
    //     let mut task = self.tasks.get(&task_id).expect("Task not found").clone();

    //     assert_eq!(
    //         task.status,
    //         TaskStatus::Published,
    //         "Task is not in published state"
    //     );
    //     assert_eq!(
    //         task.broadcaster_id,
    //         env::predecessor_account_id(),
    //         "Only broadcaster can assign task"
    //     );

    //     task.status = TaskStatus::Assigned;
    //     task.assigned_worker = Some(worker_id.clone());
    //     task.assignment_time = Some(env::block_timestamp());

    //     self.tasks.insert(task_id.clone(), task);
    //     log!("Task {} assigned to worker {}", task_id, worker_id);

    //     true
    // }

    /// 工作节点标记任务完成
    // pub fn complete_task(&mut self, task_id: String, result_ipfs: String) -> bool {
    //     let mut task = self.tasks.get(&task_id).expect("Task not found").clone();

    //     assert_eq!(
    //         task.status,
    //         TaskStatus::Assigned,
    //         "Task is not in assigned state"
    //     );
    //     assert_eq!(
    //         task.assigned_worker.as_ref().unwrap(),
    //         &env::predecessor_account_id(),
    //         "Only assigned worker can complete task"
    //     );

    //     task.status = TaskStatus::Completed;
    //     task.result_ipfs = Some(result_ipfs);
    //     task.completion_time = Some(env::block_timestamp());

    //     self.tasks.insert(task_id.clone(), task);
    //     log!(
    //         "Task {} completed by worker {}",
    //         task_id,
    //         env::predecessor_account_id()
    //     );

    //     true
    // }
    pub fn complete_task(
        &mut self,
        task_id: String,
        result_ipfs: String,
        keyframe_timestamps: Vec<String>,
        video_duration: f64,
    ) -> bool {
        let worker_id = env::predecessor_account_id();

        if let Some(mut task) = self.tasks.get(&task_id).cloned() {
            // 确认调用者是被分配的Worker
            if task.assigned_worker != Some(worker_id.clone())
                || task.status != TaskStatus::Assigned
            {
                return false;
            }

            // 更新任务状态
            task.status = TaskStatus::Completed;
            task.result_ipfs = Some(result_ipfs);
            task.completion_time = Some(env::block_timestamp());
            task.keyframe_timestamps = Some(keyframe_timestamps.clone()); // 保存关键帧时间戳
            task.video_duration = Some(video_duration);

            // 为验证选择GOP
            task.selected_gops = Some(self.select_gops_for_verification(keyframe_timestamps));

            self.tasks.insert(task_id.clone(), task.clone());

            // 释放Worker状态
            if let Some(mut worker_info) = self.active_workers.get(&worker_id).cloned() {
                worker_info.current_task = None;
                self.active_workers
                    .insert(worker_id.clone(), worker_info.clone());

                // 检查队列中是否有其他任务，使用新函数
                if worker_info.available {
                    self.auto_submit_offer_for_queued_task(&worker_id);
                }
            }

            // 安排验证者
            self.select_and_assign_verifiers(task_id);

            true
        } else {
            false
        }
    }

    /// 从关键帧时间戳列表中选择部分GOP用于验证
    /// @param keyframe_timestamps: 所有关键帧时间戳
    /// @return 被选中的GOP时间戳
    pub fn select_gops_for_verification(&self, keyframe_timestamps: Vec<String>) -> Vec<String> {
        // 如果关键帧数量太少，直接返回所有关键帧
        if keyframe_timestamps.len() <= 2 {
            return keyframe_timestamps;
        }

        // 采样固定数量的GOP (比如3个)
        let sample_count = std::cmp::min(3, keyframe_timestamps.len());

        // 使用区块高度和时间戳作为随机数种子
        let seed = env::block_height() + (env::block_timestamp() / 1_000_000) as u64;

        let mut selected_gops = Vec::with_capacity(sample_count);
        let mut used_indices = std::collections::HashSet::new();

        // 确保至少包含第一个GOP
        selected_gops.push(keyframe_timestamps[0].clone());
        used_indices.insert(0);

        // 随机选择剩余的样本
        for i in 1..sample_count {
            // 简单的伪随机算法
            let mut index = ((seed + i as u64) % (keyframe_timestamps.len() as u64)) as usize;

            // 避免重复
            while used_indices.contains(&index) {
                index = (index + 1) % keyframe_timestamps.len();
            }

            selected_gops.push(keyframe_timestamps[index].clone());
            used_indices.insert(index);
        }

        selected_gops
    }

    /// 获取任务信息
    pub fn get_task(&self, task_id: String) -> Option<TaskData> {
        self.tasks.get(&task_id).cloned()
    }

    /// 获取可用任务列表
    pub fn get_available_tasks(
        &self,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<TaskData> {
        let from = from_index.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(50) as usize;

        let available_tasks: Vec<TaskData> = self
            .tasks
            .values()
            .filter(|task| task.status == TaskStatus::OfferCollecting)
            .map(|task| task.clone())
            .collect();

        // 对筛选后的结果分页
        available_tasks.into_iter().skip(from).take(limit).collect()
    }

    /// 获取工作节点的任务列表
    pub fn get_worker_tasks(&self, worker_id: AccountId) -> Vec<TaskData> {
        self.tasks
            .values()
            .filter(|task| {
                task.assigned_worker
                    .as_ref()
                    .map_or(false, |id| *id == worker_id)
            })
            .map(|task| task.clone())
            .collect()
    }

    //
    // 共识相关方法
    //

    /// 提交共识结果
    pub fn submit_consensus_proof(&mut self, proof: ConsensusQosProof) -> bool {
        // 检查调用者是否是委员会leader
        let caller = env::predecessor_account_id();
        let is_leader = self
            .committee_members
            .iter()
            .any(|member| member.account_id == caller && member.is_leader);

        assert!(is_leader, "只有委员会leader可以提交共识结果");

        // 确保调用者与证明中的committee_leader一致
        // assert_eq!(
        //     caller, proof.committee_leader,
        //     "提交者必须是证明中指定的委员会leader"
        // );

        // 检查任务是否存在
        let task_id = proof.task_id.clone();
        let mut task = self.tasks.get(&task_id).expect("Task not found").clone();

        // 更新证明状态和时间戳
        let mut updated_proof = proof;
        updated_proof.timestamp = env::block_timestamp();
        // updated_proof.status = proof_status;

        // 存储共识证明
        self.consensus_proofs
            .insert(task_id.clone(), updated_proof.clone());

        // 更新任务状态
        task.status = TaskStatus::Verified;
        task.qos_proof_id = Some(task_id.clone());
        self.tasks.insert(task_id.clone(), task);

        // 记录工作节点的服务质量
        self.record_worker_performance(&task_id, &updated_proof);

        log!("Consensus proof submitted for task {}", task_id);
        true
    }

    /// 记录工作节点的服务质量
    /// 记录工作节点的服务质量并自动更新QoS评分
    fn record_worker_performance(&mut self, task_id: &String, proof: &ConsensusQosProof) {
        let task = self.tasks.get(task_id).expect("Task not found");
        let worker_id = task
            .assigned_worker
            .as_ref()
            .expect("No worker assigned")
            .clone();

        // 判断视频质量是否达标
        let video_quality_compliant = proof.video_score >= 80.0;

        // 判断音频质量是否达标（如果有音频的话）
        // 使用PESQ 4.3作为合格标准，与calculate_task_quality中使用的阈值一致
        let (audio_score, audio_quality_compliant) = if proof.audio_score > 0.0 {
            (Some(proof.audio_score), Some(proof.audio_score >= 4.0))
        } else {
            (None, None) // 表示NA
        };

        // 判断同步质量是否达标（如果适用的话）
        // 使用-125ms到45ms的范围作为合格标准，与calculate_task_quality中的阈值一致
        let (sync_score, sync_quality_compliant) = if proof.audio_score > 0.0 {
            (
                Some(proof.sync_score),
                Some(proof.sync_score >= -125.0 && proof.sync_score <= 45.0),
            )
        } else {
            (None, None) // 表示NA
        };

        // 判断视频规格是否符合要求
        let specs_compliant =
            MediaTranscodingContract::is_specs_compliant(&proof.video_specs, &task.requirements);

        // 判断总体是否达标
        let overall_compliant = video_quality_compliant
            && (audio_quality_compliant.unwrap_or(true))
            && (sync_quality_compliant.unwrap_or(true))
            && specs_compliant;

        let worker_performance = WorkerPerformance {
            task_id: task_id.clone(),
            worker_id: worker_id.clone(),
            timestamp: env::block_timestamp(),
            frame_count: proof.frame_count,
            video_score: proof.video_score,
            video_quality_compliant,
            audio_score,
            audio_quality_compliant,
            sync_score,
            sync_quality_compliant,
            encoding_duration: proof.encoding_end_time - proof.encoding_start_time,
            video_specs: proof.video_specs.clone(),
            specs_compliant,
            overall_compliant,
        };

        // 存储工作节点性能记录
        self.worker_performances.insert(
            format!("{}:{}", task_id, worker_id),
            worker_performance.clone(),
        );

        // 使用统一的方法更新工作节点性能
        self.update_worker_performance(&worker_id, &worker_performance);

        log!(
            "记录了Worker {} 的服务质量: VMAF={}, PESQ={}, SYNC={}, 总体达标={}",
            worker_id,
            proof.video_score,
            audio_score.map_or("NA".to_string(), |s| s.to_string()),
            sync_score.map_or("NA".to_string(), |s| s.to_string()),
            overall_compliant
        );
    }

    /// 统一更新工作节点性能数据的方法
    fn update_worker_performance(
        &mut self,
        worker_id: &AccountId,
        performance: &WorkerPerformance,
    ) {
        // 获取工作节点的性能历史
        let mut history = if let Some(h) = self.worker_performance_history.get(worker_id).cloned() {
            h
        } else {
            // 如果没有，创建一个新的
            WorkerPerformanceHistory {
                worker_id: worker_id.clone(),
                performance_records: Vec::new(),
                daily_stats: Vec::new(),
                total_tasks_completed: 0,
                last_update_time: 0,
                service_reliability_score: 0.5,
                time_stability_score: 0.5,
                performance_score: 0.5,
                avg_video_score: 0.0,
                avg_audio_score: 0.0,
                avg_sync_score: 0.0,
                avg_encoding_duration: 0,
                completion_rate: 0.0,
                compliance_rate: 0.0,
            }
        };

        // 添加性能记录
        history.performance_records.push(performance.clone());

        // 限制性能记录数量（保留最近50条）
        if history.performance_records.len() > 50 {
            history.performance_records.remove(0);
        }

        // 更新总体统计数据 - 只在这一个地方递增任务完成数量
        history.total_tasks_completed += 1;

        // 获取当前时间戳
        let current_timestamp = env::block_timestamp() / 1_000_000; // 转换为毫秒
        history.last_update_time = current_timestamp;

        // 获取当前日期时间戳（毫秒级别的天）
        let current_day_timestamp =
            current_timestamp / (24 * 60 * 60 * 1_000) * (24 * 60 * 60 * 1_000);

        // 计算任务质量评分
        let task_quality = self.calculate_task_quality(
            performance.video_score,
            performance.audio_score,
            performance.sync_score.map(|s| s as f64),
        );

        // 计算每秒帧数(FPS)
        let frame_count = performance.frame_count as f64;
        let encoding_duration_seconds = performance.encoding_duration as f64 / 1000.0; // 毫秒转秒
        let fps = if encoding_duration_seconds > 0.0 {
            frame_count / encoding_duration_seconds
        } else {
            30.0 // 默认值
        };

        // 更新每日统计数据
        if let Some(index) = history
            .daily_stats
            .iter()
            .position(|stat| stat.date == current_day_timestamp)
        {
            // 已存在今天的统计，获取并更新
            let mut stats = history.daily_stats[index].clone();
            stats.tasks_completed += 1;
            stats.provided_service = true;
            stats.encoding_fps.push(fps);
            stats.quality_scores.push(task_quality);

            // 更新到历史记录
            history.daily_stats[index] = stats;
        } else {
            // 不存在今天的统计，创建新的
            let stats = WorkerDailyStats {
                date: current_day_timestamp,
                tasks_completed: 1,
                tasks_accepted: 1, // 假设完成任务表示之前接受了任务
                provided_service: true,
                encoding_fps: vec![fps],
                quality_scores: vec![task_quality],
            };

            // 添加到历史记录
            history.daily_stats.push(stats);
        }

        // 限制每日统计数据（保留最近30天）
        if history.daily_stats.len() > 30 {
            history.daily_stats.sort_by_key(|stat| stat.date);
            history.daily_stats.remove(0); // 移除最旧的
        }

        // 计算平均值和统计数据
        let mut video_sum = 0.0;
        let mut audio_sum = 0.0;
        let mut sync_sum = 0.0;
        let mut duration_sum = 0;
        let mut compliant_count = 0;

        for perf in &history.performance_records {
            video_sum += perf.video_score;

            if let Some(audio) = perf.audio_score {
                audio_sum += audio;
            }

            if let Some(sync) = perf.sync_score {
                sync_sum += sync as f64;
            }

            duration_sum += perf.encoding_duration;

            if perf.overall_compliant {
                compliant_count += 1;
            }
        }

        let record_count = history.performance_records.len() as f64;
        if record_count > 0.0 {
            history.avg_video_score = video_sum / record_count;
            history.avg_audio_score = audio_sum / record_count;
            history.avg_sync_score = sync_sum / record_count;
            history.avg_encoding_duration = duration_sum / history.performance_records.len() as u64;
            history.compliance_rate = compliant_count as f64 / record_count;
        }

        // 计算完成率
        let total_accepted: u32 = history
            .daily_stats
            .iter()
            .map(|stat| stat.tasks_accepted)
            .sum();

        if total_accepted > 0 {
            history.completion_rate = history.total_tasks_completed as f64 / total_accepted as f64;
        }

        // 计算三个维度的评分
        history.service_reliability_score = self.calculate_service_reliability(&history);
        history.time_stability_score = self.calculate_time_stability(&history);
        history.performance_score = self.calculate_performance_score(&history);

        // 保存更新后的历史记录
        self.worker_performance_history
            .insert(worker_id.clone(), history.clone());

        // 计算并更新Worker的QoS评分
        let qos_score = self.calculate_worker_qos_score(&history);
        if let Some(mut worker_info) = self.active_workers.get(worker_id).cloned() {
            worker_info.qos_score = qos_score;

            // 同时更新Worker信息中的各项评分
            worker_info.service_reliability_score = history.service_reliability_score;
            worker_info.time_stability_score = history.time_stability_score;
            worker_info.performance_score = history.performance_score;
            worker_info.total_tasks_completed = history.total_tasks_completed;

            // 计算最近7天的活跃天数
            let week_ago = if current_timestamp > SEVEN_DAYS_MS {
                current_timestamp - SEVEN_DAYS_MS
            } else {
                0
            };

            let active_days = history
                .daily_stats
                .iter()
                .filter(|stat| stat.date >= week_ago && stat.provided_service)
                .count() as u32;

            worker_info.active_days_last_week = active_days;

            self.active_workers.insert(worker_id.clone(), worker_info);

            log!("Worker {} 的QoS评分已更新: {}", worker_id, qos_score);
        }
    }

    // 判断视频规格是否符合要求
    fn is_specs_compliant(
        video_specs: &VideoSpecification,
        requirements: &TranscodingRequirement,
    ) -> bool {
        // 检查编解码器
        let codec_match = video_specs.codec == requirements.target_codec;

        // 检查分辨率
        let resolution_match = video_specs.resolution == requirements.target_resolution;

        // 检查比特率（允许一定的偏差）
        let target_bitrate = requirements.target_bitrate.parse::<u32>().unwrap_or(0);
        let bitrate_match = if target_bitrate > 0 {
            // 允许实际比特率在目标的±10%范围内
            let min_bitrate = (target_bitrate as f64 * 0.9) as u32;
            let max_bitrate = (target_bitrate as f64 * 1.1) as u32;
            video_specs.bitrate >= min_bitrate && video_specs.bitrate <= max_bitrate
        } else {
            true
        };

        // 检查帧率（允许一定的偏差）
        let target_framerate = requirements.target_framerate.parse::<f32>().unwrap_or(0.0);
        let framerate_match = if target_framerate > 0.0 {
            // 允许实际帧率在目标的±5%范围内
            let min_framerate = target_framerate * 0.95;
            let max_framerate = target_framerate * 1.05;
            video_specs.framerate >= min_framerate && video_specs.framerate <= max_framerate
        } else {
            true
        };

        // 所有条件都满足才返回true
        codec_match && resolution_match && bitrate_match && framerate_match
    }

    /// 获取QoS证明
    pub fn get_consensus_proof(&self, task_id: String) -> Option<ConsensusQosProof> {
        // let cureent_proof = self
        //     .consensus_proofs
        //     .get(&task_id)
        //     .expect("Consensus proof not found")
        //     .clone();
        // println!("当前证明: {:?}", cureent_proof);
        self.consensus_proofs.get(&task_id).cloned()
    }

    /// 获取所有共识证明
    pub fn get_all_consensus_proofs(
        &self,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<ConsensusQosProof> {
        let from = from_index.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(50) as usize;

        self.consensus_proofs
            .values()
            .skip(from)
            .take(limit)
            .map(|proof| proof.clone())
            .collect()
    }

    //
    // qos 信息
    //

    /// 获取VMAF质量系数
    fn get_vmaf_quality_coefficient(&self, vmaf_score: f64) -> f64 {
        if vmaf_score >= 98.0 {
            return 0.95;
        } else if vmaf_score >= 92.0 {
            return 0.90;
        } else if vmaf_score >= 86.0 {
            return 0.85;
        } else if vmaf_score >= 80.0 {
            return 0.80;
        } else {
            return 0.0;
        }
    }

    /// 计算任务质量评分
    fn calculate_task_quality(
        &self,
        vmaf_score: f64,
        audio_score: Option<f64>,
        sync_score: Option<f64>,
    ) -> f64 {
        // 检查音频质量是否达标
        let audio_compliant = audio_score.map_or(true, |score| score >= 4.0);

        // 检查音视频同步是否达标 (假设sync_score是毫秒差值的绝对值)
        let sync_compliant = sync_score.map_or(true, |score| score >= -125.0 && score <= 45.0);

        // 只有当音频和同步均达标时，才应用VMAF分层
        if audio_compliant && sync_compliant {
            return self.get_vmaf_quality_coefficient(vmaf_score);
        } else {
            return 0.0;
        }
    }

    /// 计算服务可靠性评分(SR)
    fn calculate_service_reliability(&self, history: &WorkerPerformanceHistory) -> f64 {
        // 获取过去7天的每日统计
        let current_time = env::block_timestamp() / 1_000_000; // 纳秒转毫秒
                                                               // 安全计算7天前的时间
        let week_ago = if current_time > SEVEN_DAYS_MS {
            // println!(
            //     "calculate_service_reliability 进入 current_time > SEVEN_DAYS_MS分支，结果是{:?}",
            //     current_time - SEVEN_DAYS_MS
            // );
            current_time - SEVEN_DAYS_MS
        } else {
            0 // 如果当前时间小于7天的毫秒数，则使用0作为起始时间
        };
        // println!("初始统计数据: {:?}", history.daily_stats);

        let recent_stats: Vec<&WorkerDailyStats> = history
            .daily_stats
            .iter()
            .filter(|stat| stat.date > week_ago)
            .collect();

        if recent_stats.is_empty() {
            return 0.3; // 默认中等评分
        }

        // 1. 计算成功完成率(SCR)
        let mut total_completed = 0;
        let mut total_accepted = 0;
        for stat in &recent_stats {
            total_completed += stat.tasks_completed;
            total_accepted += stat.tasks_accepted;
        }

        let scr = if total_accepted > 0 {
            total_completed as f64 / total_accepted as f64
        } else {
            0.0
        };

        // 2. 计算任务参与度(TP)
        let active_days = recent_stats
            .iter()
            .filter(|stat| stat.provided_service)
            .count();

        let tp = active_days as f64 / 7.0; // 7天窗口

        // 3. 计算服务质量评分(SQS)
        let mut total_quality = 0.0;
        let mut quality_count = 0;

        for stat in &recent_stats {
            if !stat.quality_scores.is_empty() {
                total_quality += stat.quality_scores.iter().sum::<f64>();
                quality_count += stat.quality_scores.len();
            }
        }

        let sqs = if quality_count > 0 {
            total_quality / quality_count as f64
        } else {
            0.0
        };

        // 加权计算服务可靠性
        let beta1 = 0.4; // 成功完成率权重
        let beta2 = 0.2; // 任务参与度权重
        let beta3 = 0.4; // 服务质量评分权重
                         // let sr = beta1 * scr + beta2 * tp + beta3 * sqs;

        // println!("SR测量结果为{}", sr);

        beta1 * scr + beta2 * tp + beta3 * sqs
    }

    /// 计算时间稳定性评分(TS)
    fn calculate_time_stability(&self, history: &WorkerPerformanceHistory) -> f64 {
        // 获取过去7天的每日统计
        let current_time = env::block_timestamp() / 1_000_000;
        // 安全计算7天前的时间
        let week_ago = if current_time > SEVEN_DAYS_MS {
            current_time - SEVEN_DAYS_MS
        } else {
            0 // 如果当前时间小于7天的毫秒数，则使用0作为起始时间
        };

        let recent_stats: Vec<&WorkerDailyStats> = history
            .daily_stats
            .iter()
            .filter(|stat| stat.date > week_ago)
            .collect();

        if recent_stats.is_empty() {
            return 0.3; // 默认中等评分
        }

        // 1. 计算日均任务量(ADT)
        let total_tasks: u32 = recent_stats.iter().map(|stat| stat.tasks_completed).sum();

        let adt = total_tasks as f64 / 7.0; // 每日平均任务数

        // 归一化ADT（假设每日10个任务为满分）
        let normalized_adt = (adt / 10.0).min(1.0);

        // 2. 计算服务覆盖率(SC)
        let active_days = recent_stats
            .iter()
            .filter(|stat| stat.provided_service)
            .count();

        let sc = active_days as f64 / 7.0; // 7天窗口
                                           // println!("inside calculate_time_stability, adt:{}, sc:{}", adt, sc);

        // 加权计算时间稳定性
        let delta1 = 0.5; // 日均任务量权重
        let delta2 = 0.5; // 服务覆盖率权重

        delta1 * normalized_adt + delta2 * sc
    }

    /// 计算性能表现评分(PP)
    fn calculate_performance_score(&self, history: &WorkerPerformanceHistory) -> f64 {
        // 获取过去7天的每日统计
        let current_time = env::block_timestamp() / 1_000_000;
        // 安全计算7天前的时间
        let week_ago = if current_time > SEVEN_DAYS_MS {
            current_time - SEVEN_DAYS_MS
        } else {
            0 // 如果当前时间小于7天的毫秒数，则使用0作为起始时间
        };

        let recent_stats: Vec<&WorkerDailyStats> = history
            .daily_stats
            .iter()
            .filter(|stat| stat.date > week_ago)
            .collect();

        if recent_stats.is_empty() {
            return 0.3; // 默认中等评分
        }
        // 收集所有FPS数据
        let mut all_fps: Vec<f64> = Vec::new();
        for stat in &recent_stats {
            all_fps.extend(&stat.encoding_fps);
        }

        if all_fps.is_empty() {
            return 0.3; // 默认中等评分
        }

        // 1. 计算平均FPS(APF)
        let avg_fps = all_fps.iter().sum::<f64>() / all_fps.len() as f64;

        // 基准FPS（假设15fps为标准）
        let benchmark_fps = 25.0;

        // FPS越高，分数越高
        let normalized_apf = (avg_fps / benchmark_fps).min(1.0);

        // 2. 计算性能稳定性(PS)
        // 计算标准差
        let variance = all_fps
            .iter()
            .map(|&f| {
                let diff = f - avg_fps;
                diff * diff
            })
            .sum::<f64>()
            / all_fps.len() as f64;

        let std_dev = variance.sqrt();

        // 性能稳定性 = 1 - 变异系数（标准差/平均值）
        let ps = (1.0 - std_dev / avg_fps).max(0.0).min(1.0);

        // println!("normalized_apf:{}, ps:{}", normalized_apf, ps);

        // 加权计算性能表现
        let epsilon1 = 0.7; // 平均处理速度权重
        let epsilon2 = 0.3; // 性能稳定性权重

        epsilon1 * normalized_apf + epsilon2 * ps
    }

    /// 计算Worker的QoS综合评分
    /// 重新设计的计算Worker QoS评分函数
    pub fn calculate_worker_qos_score(&self, history: &WorkerPerformanceHistory) -> f64 {
        // 计算三个维度的评分
        let sr = self.calculate_service_reliability(history);
        let ts = self.calculate_time_stability(history);
        let pp = self.calculate_performance_score(history);

        // println!("sr:{}, ts:{}, pp:{}", sr, ts, pp);

        // 最终评分计算
        let qos_score = 0.4 * sr + 0.2 * ts + 0.4 * pp;

        // 确保最终分数在0-1范围内
        qos_score.max(0.0).min(1.0)
    }

    // 辅助方法 - 获取增强型Worker历史
    fn get_worker_history(&self, worker_id: &AccountId) -> Option<WorkerPerformanceHistory> {
        // 这个函数需要根据你的存储方式实现
        // 假设你使用了一个新的映射来存储增强型历史
        self.worker_performance_history.get(worker_id).cloned()
    }

    // 辅助方法 - 保存增强型Worker历史
    fn save_worker_history(&mut self, worker_id: &AccountId, history: &WorkerPerformanceHistory) {
        // 这个函数需要根据你的存储方式实现
        self.worker_performance_history
            .insert(worker_id.clone(), history.clone());
    }

    // 公开API方法，获取工作节点的QoS评分明细
    pub fn get_worker_qos_details(&self, worker_id: AccountId) -> Option<QosScoreDetails> {
        if let Some(enhanced_history) = self.worker_performance_history.get(&worker_id) {
            Some(QosScoreDetails {
                worker_id: worker_id.clone(),
                service_reliability_score: enhanced_history.service_reliability_score,
                time_stability_score: enhanced_history.time_stability_score,
                performance_score: enhanced_history.performance_score,
                overall_qos_score: self.calculate_worker_qos_score(enhanced_history),
                last_update_time: enhanced_history.last_update_time,
            })
        } else {
            None
        }
    }

    // 公开API方法，获取工作节点的每日统计数据
    pub fn get_worker_daily_stats(
        &self,
        worker_id: AccountId,
        days: Option<u32>,
    ) -> Option<Vec<WorkerDailyStats>> {
        if let Some(enhanced_history) = self.worker_performance_history.get(&worker_id) {
            let days_count = days.unwrap_or(7);

            // 获取过去days_count天的每日统计
            let current_time = env::block_timestamp();
            let cutoff_time = current_time - days_count as u64 * 24 * 60 * 60 * 1_000_000_000;

            let recent_stats: Vec<WorkerDailyStats> = enhanced_history
                .daily_stats
                .iter()
                .filter(|stat| stat.date >= cutoff_time)
                .cloned()
                .collect();

            if recent_stats.is_empty() {
                None
            } else {
                Some(recent_stats)
            }
        } else {
            None
        }
    }

    //
    // get 方法
    //

    pub fn get_task_queue(&self) -> Vec<String> {
        self.pending_tasks.clone()
    }

    /// 获取任务offer列表
    pub fn get_task_offers(&self, task_id: String) -> Option<Vec<WorkerOffer>> {
        self.task_offers.get(&task_id).cloned()
    }

    /// 获取worker信息
    pub fn get_worker_info(&self, worker_id: AccountId) -> Option<WorkerInfo> {
        self.active_workers.get(&worker_id).cloned()
    }

    /// 获取worker QoS评分
    pub fn get_worker_qos_score(&self, worker_id: AccountId) -> Option<f64> {
        if let Some(history) = self.worker_performance_history.get(&worker_id) {
            return Some(self.calculate_worker_qos_score(history));
        }
        return None;
    }

    /// 获取worker性能历史摘要
    pub fn get_worker_performance_summary(
        &self,
        worker_id: AccountId,
    ) -> Option<WorkerPerformanceSummary> {
        if let Some(enhanced_history) = self.worker_performance_history.get(&worker_id) {
            // 计算最近7天的服务天数
            let current_time = env::block_timestamp() / 1_000_000;
            let week_ago = if current_time > SEVEN_DAYS_MS {
                current_time - SEVEN_DAYS_MS
            } else {
                0 // 如果当前时间小于7天的毫秒数，则使用0作为起始时间
            };

            let service_days = enhanced_history
                .daily_stats
                .iter()
                .filter(|stat| stat.date >= week_ago && stat.provided_service)
                .count() as u32;

            // 综合评分
            let qos_score = self.calculate_worker_qos_score(enhanced_history);

            Some(WorkerPerformanceSummary {
                worker_id: worker_id.clone(),
                total_tasks_completed: enhanced_history.total_tasks_completed,
                avg_video_score: enhanced_history.avg_video_score,
                avg_audio_score: enhanced_history.avg_audio_score,
                avg_sync_score: enhanced_history.avg_sync_score,
                avg_encoding_duration: enhanced_history.avg_encoding_duration,
                completion_rate: enhanced_history.completion_rate,
                compliance_rate: enhanced_history.compliance_rate,
                service_days_last_week: service_days,
                qos_score,
            })
        } else {
            None
        }
    }

    pub fn get_broadcaster_tasks(&self, broadcaster_id: AccountId) -> Vec<TaskData> {
        // 从任务列表中筛选出指定broadcaster的任务
        self.tasks
            .values()
            .filter(|task| task.broadcaster_id == broadcaster_id)
            .map(|task| task.clone())
            .collect()
    }

    /// 获取任务的共识详情，包括任务信息、工作节点表现和共识证明
    pub fn get_task_consensus_details(&self, task_id: String) -> Option<TaskConsensusDetails> {
        // 获取任务信息
        let task = match self.tasks.get(&task_id) {
            Some(t) => t.clone(),
            None => return None,
        };

        // 获取共识证明
        let consensus_proof = self.consensus_proofs.get(&task_id).cloned();

        // 获取工作节点表现（如果任务已分配工作节点）
        let worker_performance = if let Some(worker_id) = &task.assigned_worker {
            self.worker_performances
                .get(&format!("{}:{}", task_id, worker_id))
                .cloned()
        } else {
            None
        };

        // 创建并返回详细信息结构体
        Some(TaskConsensusDetails {
            task,
            consensus_proof,
            worker_performance,
        })
    }

    pub fn get_tasks(&self) -> Vec<TaskData> {
        // 返回所有任务
        self.tasks.values().cloned().collect()
    }
}

// 测试模块
#[cfg(test)]
mod tests {
    use std::string;

    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, VMContext};

    fn get_context(predecessor_account_id: AccountId) -> VMContext {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor_account_id);
        // 设置区块时间戳和高度，用于测试
        let timestamp_2023_01_01 = 1672531200_000_000_000;

        builder.block_timestamp(timestamp_2023_01_01);
        // builder.block_timestamp(100_000_000_000);
        builder.block_height(100);
        builder.build()
    }

    fn setup_contract() -> (MediaTranscodingContract, AccountId) {
        let owner = accounts(0);
        let worker_contract = accounts(1);
        let verifier_contract = accounts(2);
        let context = get_context(owner.clone());
        testing_env!(context);

        // // 创建几个委员会成员（带有假数据）
        // let committee_member1 = CommitteeMemberInfo {
        //     account_id: accounts(3),
        //     ip_address: "192.168.1.101".to_string(),
        //     port: 8545,
        //     is_leader: true,
        // };

        // let committee_member2 = CommitteeMemberInfo {
        //     account_id: accounts(4),
        //     ip_address: "192.168.1.102".to_string(),
        //     port: 8546,
        //     is_leader: false,
        // };

        // let committee_member3 = CommitteeMemberInfo {
        //     account_id: accounts(5),
        //     ip_address: "192.168.1.103".to_string(),
        //     port: 8547,
        //     is_leader: false,
        // };

        // let committee_members = vec![committee_member1, committee_member2, committee_member3];

        // 初始化合约
        let contract = MediaTranscodingContract::new(
            Some(owner.clone()),
            // Some(worker_contract),
            // Some(verifier_contract),
            None,
            None,
        );
        (contract, owner)
    }

    // // 测试分配任务
    // #[test]
    // fn test_assign_task() {
    //     let (mut contract, owner) = setup_contract();
    //     let broadcaster = accounts(1);
    //     let worker = accounts(2);

    //     // 模拟广播者上下文
    //     testing_env!(get_context(broadcaster.clone()));

    //     // 创建一个任务
    //     let task_id = "task1".to_string();

    //     // 创建一些模拟的转码需求
    //     let requirements1 = TranscodingRequirement {
    //         target_codec: "H.264".to_string(),
    //         target_resolution: "1920x1080".to_string(),
    //         target_bitrate: "5000kbps".to_string(),
    //         target_framerate: "30fps".to_string(),
    //         additional_params: "--preset medium --profile high".to_string(),
    //     };

    //     let source_ipfs1 = "ipfs://QmT7fYV4HxRnbvS8H9hNNyZyA1wxCDQf3MhQD8FXQz5yzt".to_string();

    //     // let requirements2 = TranscodingRequirement {
    //     //     target_codec: "H.265".to_string(),
    //     //     target_resolution: "3840x2160".to_string(),
    //     //     target_bitrate: "15000kbps".to_string(),
    //     //     target_framerate: "60fps".to_string(),
    //     //     additional_params: "--preset slow --crf 22".to_string(),
    //     // };

    //     // let requirements3 = TranscodingRequirement {
    //     //     target_codec: "VP9".to_string(),
    //     //     target_resolution: "1280x720".to_string(),
    //     //     target_bitrate: "2500kbps".to_string(),
    //     //     target_framerate: "24fps".to_string(),
    //     //     additional_params: "--tile-columns 2 --frame-parallel 1".to_string(),
    //     // };

    //     let task_id = contract.publish_task(source_ipfs1.clone(), requirements1.clone());

    //     // 分配任务给工作者
    //     let result = contract.assign_task(task_id.clone(), worker.clone());
    //     assert!(result);

    //     // 验证任务状态
    //     let task = contract.tasks.get(&task_id).unwrap();
    //     assert_eq!(task.status, TaskStatus::Assigned);
    //     assert_eq!(task.assigned_worker, Some(worker));
    // }

    // #[test]
    // fn test_initialize_committee_with_hardcoded_members() {
    //     let (mut contract, owner) = setup_contract();

    //     // 确保是以拥有者身份调用
    //     testing_env!(get_context(owner.clone()));

    //     // 调用初始化委员会的方法
    //     contract.initialize_committee_with_hardcoded_members();

    //     // 检查委员会成员是否正确初始化
    //     let committee_members = contract.get_committee_members();

    //     // 验证成员数量
    //     assert_eq!(committee_members.len(), 4, "应该有4个委员会成员");

    //     // 验证leader是否正确
    //     let leader = contract.get_committee_leader().expect("应该有一个leader");
    //     assert_eq!(
    //         leader.account_id.to_string(),
    //         "leader.testnet",
    //         "leader账户ID不匹配"
    //     );
    //     assert_eq!(leader.ip_address, "10.24.136.124", "leader IP地址不匹配");
    //     assert_eq!(leader.port, 8000, "leader端口不匹配");
    //     assert!(leader.is_leader, "leader标志应为true");

    //     // 验证其他成员
    //     let follower_accounts = committee_members
    //         .iter()
    //         .filter(|member| !member.is_leader)
    //         .map(|member| member.account_id.to_string())
    //         .collect::<Vec<String>>();

    //     assert!(
    //         follower_accounts.contains(&"follower1.testnet".to_string()),
    //         "缺少follower1"
    //     );
    //     assert!(
    //         follower_accounts.contains(&"follower2.testnet".to_string()),
    //         "缺少follower2"
    //     );
    //     assert!(
    //         follower_accounts.contains(&"follower3.testnet".to_string()),
    //         "缺少follower3"
    //     );
    // }

    // // 单独测试非拥有者无法初始化委员会的情况
    // #[test]
    // #[should_panic(expected = "Only the owner can initialize committee")]
    // fn test_non_owner_cannot_initialize_committee() {
    //     let (mut contract, _) = setup_contract();

    //     // 使用非拥有者账户
    //     testing_env!(get_context(accounts(1)));

    //     // 这应该会panic，使用should_panic属性来捕获
    //     contract.initialize_committee_with_hardcoded_members();
    // }
    // #[test]
    // fn test_get_committee_members() {
    //     let (mut contract, owner) = setup_contract();

    //     // 初始状态下应该没有委员会成员
    //     let initial_members = contract.get_committee_members();
    //     assert_eq!(initial_members.len(), 0, "初始状态下不应该有委员会成员");

    //     // 初始化委员会
    //     testing_env!(get_context(owner.clone()));
    //     contract.initialize_committee_with_hardcoded_members();

    //     // 验证获取的成员列表
    //     let members = contract.get_committee_members();
    //     assert_eq!(members.len(), 4, "应该有4个委员会成员");

    //     // 验证成员详情
    //     let expected_accounts = vec![
    //         "leader.testnet",
    //         "follower1.testnet",
    //         "follower2.testnet",
    //         "follower3.testnet",
    //     ];

    //     for account in expected_accounts {
    //         assert!(
    //             members.iter().any(|m| m.account_id.to_string() == account),
    //             "缺少预期的委员会成员: {}",
    //             account
    //         );
    //     }

    //     // 验证IP地址
    //     let expected_ips = vec![
    //         "10.24.136.124",
    //         "10.24.216.33",
    //         "10.24.198.225",
    //         "10.24.161.186",
    //     ];

    //     for ip in expected_ips {
    //         assert!(
    //             members.iter().any(|m| m.ip_address == ip),
    //             "缺少预期的IP地址: {}",
    //             ip
    //         );
    //     }
    // }

    // #[test]
    // fn test_get_committee_leader() {
    //     let (mut contract, owner) = setup_contract();

    //     // 初始状态下应该没有leader
    //     let initial_leader = contract.get_committee_leader();
    //     assert!(initial_leader.is_none(), "初始状态下不应该有委员会leader");

    //     // 初始化委员会
    //     testing_env!(get_context(owner.clone()));
    //     contract.initialize_committee_with_hardcoded_members();

    //     // 验证leader信息
    //     let leader = contract.get_committee_leader();
    //     assert!(leader.is_some(), "应该有一个委员会leader");

    //     let leader = leader.unwrap();
    //     assert_eq!(
    //         leader.account_id.to_string(),
    //         "leader.testnet",
    //         "leader账户ID不匹配"
    //     );
    //     assert_eq!(leader.ip_address, "10.24.136.124", "leader IP地址不匹配");
    //     assert_eq!(leader.port, 8000, "leader端口不匹配");
    //     assert!(leader.is_leader, "leader标志应为true");

    //     // 确保只有一个leader
    //     let members = contract.get_committee_members();
    //     let leader_count = members.iter().filter(|m| m.is_leader).count();
    //     assert_eq!(leader_count, 1, "应该只有一个委员会leader");
    // }

    // // 测试获取可用任务
    // #[test]
    // fn test_get_available_tasks() {
    //     let (mut contract, _) = setup_contract();
    //     let broadcaster = accounts(1);

    //     // 模拟广播者上下文
    //     testing_env!(get_context(broadcaster.clone()));

    //     // 创建一些模拟的转码需求
    //     let requirements1 = TranscodingRequirement {
    //         target_codec: "H.264".to_string(),
    //         target_resolution: "1920x1080".to_string(),
    //         target_bitrate: "5000kbps".to_string(),
    //         target_framerate: "30fps".to_string(),
    //         additional_params: "--preset medium --profile high".to_string(),
    //     };

    //     let source_ipfs1 = "ipfs://QmT7fYV4HxRnbvS8H9hNNyZyA1wxCDQf3MhQD8FXQz5yzt".to_string();

    //     // 创建多个任务
    //     for i in 0..5 {
    //         let task_id = format!("task{}", i);
    //         contract.publish_task(source_ipfs1.clone(), requirements1.clone());
    //     }

    //     // 获取可用任务
    //     let tasks = contract.get_available_tasks(None, None);
    //     assert_eq!(tasks.len(), 5);

    //     // 测试分页
    //     let tasks = contract.get_available_tasks(Some(2), Some(2));
    //     assert_eq!(tasks.len(), 2);
    // }

    // // 测试获取工作者任务
    // #[test]
    // fn test_get_worker_tasks() {
    //     let (mut contract, _) = setup_contract();
    //     let broadcaster = accounts(1);
    //     let worker = accounts(2);

    //     // 模拟广播者上下文
    //     // let mut context = get_context(broadcaster.clone());
    //     // context.block_timestamp = 1_000_000;
    //     // testing_env!(context);

    //     // 初始化时间戳
    //     let mut current_timestamp = 1_000_000;

    //     let requirements1 = TranscodingRequirement {
    //         target_codec: "H.264".to_string(),
    //         target_resolution: "1920x1080".to_string(),
    //         target_bitrate: "5000kbps".to_string(),
    //         target_framerate: "30fps".to_string(),
    //         additional_params: "--preset medium --profile high".to_string(),
    //     };

    //     let source_ipfs1 = "ipfs://QmT7fYV4HxRnbvS8H9hNNyZyA1wxCDQf3MhQD8FXQz5yzt".to_string();

    //     // 创建并分配多个任务
    //     for i in 0..3 {
    //         let mut context = get_context(broadcaster.clone());
    //         context.block_timestamp = current_timestamp;
    //         testing_env!(context);
    //         // let task_id = format!("task{}", i);
    //         let task_id = contract.publish_task(source_ipfs1.clone(), requirements1.clone());
    //         println!("{task_id}");
    //         contract.assign_task(task_id, worker.clone());

    //         // 增加时间戳，确保下一个任务有不同的 ID
    //         current_timestamp += 1_000_000;
    //     }

    //     // 获取工作者任务
    //     let tasks = contract.get_worker_tasks(worker);
    //     assert_eq!(tasks.len(), 3);
    // }

    // #[test]
    // fn test_complete_task_success_path() {
    //     let (mut contract, owner) = setup_contract();
    //     let broadcaster = accounts(1);
    //     let worker = accounts(2);

    //     // 模拟广播者上下文创建任务
    //     let mut context = get_context(broadcaster.clone());
    //     context.block_timestamp = 1_000_000;
    //     testing_env!(context);

    //     // 创建一个任务
    //     let requirements = TranscodingRequirement {
    //         target_codec: "H.264".to_string(),
    //         target_resolution: "1920x1080".to_string(),
    //         target_bitrate: "5000kbps".to_string(),
    //         target_framerate: "30fps".to_string(),
    //         additional_params: "--preset medium".to_string(),
    //     };

    //     let source_ipfs = "ipfs://QmT7fYV4HxRnbvS8H9hNNyZyA1wxCDQf3MhQD8FXQz5yzt".to_string();
    //     let task_id = contract.publish_task(source_ipfs, requirements);

    //     // 广播者分配任务给工作者
    //     let assign_result = contract.assign_task(task_id.clone(), worker.clone());
    //     assert!(assign_result, "任务分配应该成功");

    //     // 验证任务状态是已分配
    //     let task = contract.tasks.get(&task_id).unwrap();
    //     assert_eq!(task.status, TaskStatus::Assigned, "任务状态应为已分配");
    //     assert_eq!(
    //         task.assigned_worker,
    //         Some(worker.clone()),
    //         "任务应分配给正确的工作者"
    //     );

    //     // 模拟工作者上下文完成任务
    //     testing_env!(get_context(worker.clone()));

    //     // 工作者完成任务
    //     let result_ipfs = "ipfs://QmUNLLsPACCz1vLxQVkXqqLX5R1X345qqfHbsf67hvA3Nn".to_string();
    //     let complete_result = contract.complete_task(task_id.clone(), result_ipfs.clone());
    //     assert!(complete_result, "任务完成应该成功");

    //     // 验证任务状态已更新为已完成
    //     let completed_task = contract.tasks.get(&task_id).unwrap();
    //     assert_eq!(
    //         completed_task.status,
    //         TaskStatus::Completed,
    //         "任务状态应为已完成"
    //     );
    //     assert_eq!(
    //         completed_task.result_ipfs,
    //         Some(result_ipfs),
    //         "任务结果IPFS应正确设置"
    //     );
    //     assert!(completed_task.completion_time.is_some(), "完成时间应设置");
    // }

    // // 使用should_panic分别测试失败场景
    // #[test]
    // #[should_panic(expected = "Only assigned worker can complete task")]
    // fn test_only_assigned_worker_can_complete_task() {
    //     let (mut contract, _) = setup_contract();
    //     let broadcaster = accounts(1);
    //     let worker = accounts(2);
    //     let other_account = accounts(3);

    //     // 创建并分配任务
    //     testing_env!(get_context(broadcaster.clone()));
    //     let requirements = TranscodingRequirement {
    //         target_codec: "H.264".to_string(),
    //         target_resolution: "1920x1080".to_string(),
    //         target_bitrate: "5000kbps".to_string(),
    //         target_framerate: "30fps".to_string(),
    //         additional_params: "--preset medium".to_string(),
    //     };

    //     let source_ipfs = "ipfs://QmSource".to_string();
    //     let task_id = contract.publish_task(source_ipfs, requirements);
    //     contract.assign_task(task_id.clone(), worker.clone());

    //     // 非指定工作者尝试完成任务应该失败
    //     testing_env!(get_context(other_account));
    //     contract.complete_task(task_id, "ipfs://QmResult".to_string());
    //     // 此处应该panic
    // }

    // #[test]
    // #[should_panic(expected = "Task is not in assigned state")]
    // fn test_task_must_be_in_assigned_state() {
    //     let (mut contract, _) = setup_contract();
    //     let broadcaster = accounts(1);
    //     let worker = accounts(2);

    //     // 创建任务但不分配
    //     testing_env!(get_context(broadcaster.clone()));
    //     let requirements = TranscodingRequirement {
    //         target_codec: "H.264".to_string(),
    //         target_resolution: "1920x1080".to_string(),
    //         target_bitrate: "5000kbps".to_string(),
    //         target_framerate: "30fps".to_string(),
    //         additional_params: "--preset medium".to_string(),
    //     };

    //     let source_ipfs = "ipfs://QmSource".to_string();
    //     let task_id = contract.publish_task(source_ipfs, requirements);

    //     // 尝试完成未分配的任务应该失败
    //     testing_env!(get_context(worker.clone()));
    //     contract.complete_task(task_id, "ipfs://QmResult".to_string());
    //     // 此处应该panic
    // }

    // #[test]
    // fn test_init_verifiers() {
    //     let (mut contract, owner) = setup_contract();

    //     // 设置上下文为所有者
    //     let context = get_context(owner.clone());
    //     testing_env!(context);

    //     // 创建测试验证者成员
    //     let verifier_members = vec![
    //         VerifierMemberInfo {
    //             account_id: accounts(3),
    //             ip_address: "10.0.0.1".to_string(),
    //             port: 9000,
    //         },
    //         VerifierMemberInfo {
    //             account_id: accounts(4),
    //             ip_address: "10.0.0.2".to_string(),
    //             port: 9000,
    //         },
    //     ];

    //     // 初始化验证者
    //     contract.init_verifiers(verifier_members.clone());

    //     // 验证初始化是否成功
    //     let verifiers = contract.get_verifier_members();
    //     assert_eq!(verifiers.len(), 2);
    //     assert_eq!(verifiers[0].account_id, accounts(3));
    //     assert_eq!(verifiers[1].account_id, accounts(4));
    //     // assert!(verifiers[0].is_leader);
    //     // assert!(!verifiers[1].is_leader);
    // }

    // #[test]
    // #[should_panic(expected = "Only the owner can initialize verifiers")]
    // fn test_init_verifiers_not_owner() {
    //     let (mut contract, _) = setup_contract();

    //     // 设置上下文为非所有者
    //     let context = get_context(accounts(5));
    //     testing_env!(context);

    //     // 创建测试验证者成员
    //     let verifier_members = vec![VerifierMemberInfo {
    //         account_id: accounts(3),
    //         ip_address: "10.0.0.1".to_string(),
    //         port: 9000,
    //         // is_leader: true,
    //     }];

    //     // 非所有者尝试初始化验证者，应该失败
    //     contract.init_verifiers(verifier_members);
    // }

    // #[test]
    // fn test_initialize_verifiers_with_hardcoded_members() {
    //     let (mut contract, owner) = setup_contract();

    //     // 设置上下文为所有者
    //     let context = get_context(owner.clone());
    //     testing_env!(context);

    //     // 使用硬编码方法初始化验证者
    //     contract.initialize_verifiers_with_hardcoded_members();

    //     // 验证是否成功初始化了预期数量的验证者
    //     let verifiers = contract.get_verifier_members();
    //     assert_eq!(verifiers.len(), 3); //
    // }

    // #[test]
    // fn test_select_verifiers() {
    //     let (mut contract, owner) = setup_contract();

    //     // 设置上下文为所有者，并添加区块索引
    //     let mut context_builder = VMContextBuilder::new();
    //     context_builder
    //         .predecessor_account_id(owner.clone())
    //         .block_height(1234); // 设置区块索引用于伪随机选择
    //     testing_env!(context_builder.build());

    //     contract.initialize_verifiers_with_hardcoded_members();

    //     let worker = accounts(2);

    //     // 创建一个任务
    //     let requirements = TranscodingRequirement {
    //         target_codec: "H.264".to_string(),
    //         target_resolution: "1920x1080".to_string(),
    //         target_bitrate: "5000kbps".to_string(),
    //         target_framerate: "30fps".to_string(),
    //         additional_params: "--preset medium".to_string(),
    //     };

    //     let source_ipfs = "ipfs://QmT7fYV4HxRnbvS8H9hNNyZyA1wxCDQf3MhQD8FXQz5yzt".to_string();
    //     let task_id = contract.publish_task(source_ipfs, requirements);

    //     // 广播者分配任务给工作者
    //     let assign_result = contract.assign_task(task_id.clone(), worker.clone());

    //     // 设置上下文为工作者
    //     let worker_context = get_context(worker.clone());
    //     testing_env!(worker_context);

    //     // 工作者提交完成的任务
    //     let result_url = "ipfs://QmResultVideo".to_string();
    //     contract.complete_task(task_id.clone(), result_url);

    //     // 设置上下文回到所有者
    //     let owner_context = get_context(owner.clone());
    //     testing_env!(owner_context);

    //     // 选择验证者
    //     let selected_verifiers = contract.select_verifiers(task_id.clone());

    //     // 验证是否选择了两个验证者
    //     assert_eq!(selected_verifiers.len(), 2);

    //     // 验证选择的验证者是否都来自我们的验证者列表
    //     let all_verifier_ids: Vec<_> = contract
    //         .get_verifier_members()
    //         .iter()
    //         .map(|v| v.account_id.clone())
    //         .collect();

    //     for verifier in &selected_verifiers {
    //         assert!(all_verifier_ids.contains(verifier));
    //     }

    //     // 验证任务是否已更新选定的验证者
    //     let updated_task = contract.tasks.get(&task_id).unwrap();
    //     assert_eq!(updated_task.assigned_verifiers, selected_verifiers);
    // }

    // #[test]
    // #[should_panic(expected = "Task is not in completed state")]
    // fn test_select_verifiers_task_not_completed() {
    //     let (mut contract, owner) = setup_contract();

    //     // 设置上下文
    //     let context = get_context(owner.clone());
    //     testing_env!(context);

    //     let worker = accounts(2);

    //     contract.initialize_verifiers_with_hardcoded_members();

    //     // 创建一个任务
    //     let requirements = TranscodingRequirement {
    //         target_codec: "H.264".to_string(),
    //         target_resolution: "1920x1080".to_string(),
    //         target_bitrate: "5000kbps".to_string(),
    //         target_framerate: "30fps".to_string(),
    //         additional_params: "--preset medium".to_string(),
    //     };

    //     let source_ipfs = "ipfs://QmT7fYV4HxRnbvS8H9hNNyZyA1wxCDQf3MhQD8FXQz5yzt".to_string();
    //     let task_id = contract.publish_task(source_ipfs, requirements);

    //     // 广播者分配任务给工作者
    //     contract.assign_task(task_id.clone(), worker.clone());

    //     // 此时任务状态为Assigned，尚未完成
    //     let task = contract.tasks.get(&task_id).unwrap();
    //     assert_eq!(task.status, TaskStatus::Assigned, "任务状态应为已分配");

    //     // 尝试选择验证者，应该失败
    //     contract.select_verifiers(task_id);
    // }

    // #[test]
    // fn test_request_supplemental_verifier() {
    //     let (mut contract, owner) = setup_contract();

    //     // 设置上下文为所有者
    //     let mut context_builder = VMContextBuilder::new();
    //     context_builder
    //         .predecessor_account_id(owner.clone())
    //         .block_height(1234); // 设置区块索引用于伪随机选择
    //     testing_env!(context_builder.build());

    //     // 初始化3个验证者
    //     contract.initialize_verifiers_with_hardcoded_members();

    //     // 创建一个任务
    //     let worker = accounts(2);
    //     let requirements = TranscodingRequirement {
    //         target_codec: "H.264".to_string(),
    //         target_resolution: "1920x1080".to_string(),
    //         target_bitrate: "5000kbps".to_string(),
    //         target_framerate: "30fps".to_string(),
    //         additional_params: "--preset medium".to_string(),
    //     };

    //     let source_ipfs = "ipfs://QmSourceVideo".to_string();
    //     let task_id = contract.publish_task(source_ipfs, requirements);

    //     // 将任务分配给工作者
    //     contract.assign_task(task_id.clone(), worker.clone());

    //     // 设置上下文为工作者
    //     let worker_context = get_context(worker.clone());
    //     testing_env!(worker_context);

    //     // 工作者提交完成的任务
    //     let result_url = "ipfs://QmResultVideo".to_string();
    //     contract.complete_task(task_id.clone(), result_url);

    //     // 设置上下文回到所有者
    //     let owner_context = get_context(owner.clone());
    //     testing_env!(owner_context);

    //     // 先选择初始验证者
    //     let initial_verifiers = contract.select_verifiers(task_id.clone());

    //     // 确保初始验证者数量为2
    //     assert_eq!(initial_verifiers.len(), 2);

    //     // 请求补充验证者
    //     let supplemental_verifier = contract.request_supplemental_verifier(task_id.clone());

    //     // 确保返回了一个验证者
    //     assert!(supplemental_verifier.is_some());

    //     // 获取更新后的任务
    //     let updated_task = contract.tasks.get(&task_id).unwrap();

    //     // 确保任务现在有3个验证者
    //     assert_eq!(updated_task.assigned_verifiers.len(), 3);

    //     // 确保补充验证者不在初始验证者列表中
    //     assert!(!initial_verifiers.contains(&supplemental_verifier.unwrap()));

    //     // 确保所有3个验证者都被分配了
    //     let all_verifier_ids: Vec<_> = contract
    //         .get_verifier_members()
    //         .iter()
    //         .map(|v| v.account_id.clone())
    //         .collect();

    //     for verifier in &updated_task.assigned_verifiers {
    //         assert!(all_verifier_ids.contains(verifier));
    //     }

    //     // 确保我们已经用完了所有验证者
    //     let another_supplemental = contract.request_supplemental_verifier(task_id.clone());
    //     assert!(another_supplemental.is_none(), "应该没有更多可用的验证者了");
    // }

    #[test]
    fn test_normal_task_flow_with_verification() {
        // 设置合约和账户
        let (mut contract, owner) = setup_contract();
        let broadcaster = accounts(1);
        let worker = accounts(2);
        let anyone = accounts(3);
        let verifier1 = accounts(4); // 添加验证者账户
        let verifier2 = accounts(5); // 添加第二个验证者账户

        contract.initialize_committee_with_hardcoded_members(); // 初始化委员会
        let committee_leader = contract
            .get_committee_leader()
            .expect("找不到委员会leader")
            .account_id;

        // 设置上下文为所有者
        let context = get_context(owner.clone());
        testing_env!(context);

        // 使用硬编码方法初始化验证者和委员会
        contract.initialize_verifiers_with_hardcoded_members();

        // 重写验证者成员，使用我们的测试账户
        let verifier_members = vec![
            VerifierMemberInfo {
                account_id: verifier1.clone(),
                ip_address: "10.24.100.100".to_string(),
                port: 9000,
            },
            VerifierMemberInfo {
                account_id: verifier2.clone(),
                ip_address: "10.24.100.101".to_string(),
                port: 9000,
            },
        ];
        contract.init_verifiers(verifier_members);

        // 重写委员会成员，使用我们的测试账户
        // let committee_members = vec![CommitteeMemberInfo {
        //     account_id: committee_leader.clone(),
        //     ip_address: "10.24.136.124".to_string(),
        //     port: 8000,
        //     is_leader: true,
        // }];
        // contract.init_committee(committee_members);

        // 注册Worker
        testing_env!(get_context(worker.clone()));
        println!("worker是: {:?}", env::predecessor_account_id());
        assert!(contract.register_worker(true), "Worker注册应该成功"); // 注册支持硬件加速的Worker

        // 1. Broadcaster发布任务
        testing_env!(get_context(broadcaster.clone()));
        let task_id = contract.publish_task(
            "ipfs://source_video".to_string(),
            TranscodingRequirement {
                target_codec: "h264".to_string(),
                target_resolution: "1920x1080".to_string(),
                target_bitrate: "5000".to_string(),
                target_framerate: "30".to_string(),
                additional_params: "".to_string(),
            },
            true, // 偏好硬件加速
        );

        // 验证任务状态为OfferCollecting
        let task = contract.get_task(task_id.clone()).unwrap();
        assert_eq!(
            task.status,
            TaskStatus::OfferCollecting,
            "任务状态应该是OfferCollecting"
        );

        // 记录任务发布时间
        let publish_time = task.publish_time;

        // 2. Worker提交Offer
        testing_env!(get_context(worker.clone()));
        assert!(
            contract.submit_offer(task_id.clone()),
            "Worker提交offer应该成功"
        );

        // 验证offer已提交
        let offers = contract.get_task_offers(task_id.clone()).unwrap();
        assert_eq!(offers.len(), 1, "应该有1个offer");
        assert_eq!(offers[0].worker_id, worker, "offer的worker_id应该匹配");

        // 3. 推进时间8秒（模拟等待超时）
        let mut context = get_context(anyone.clone());
        context.block_timestamp = publish_time + 9_000_000_000;
        testing_env!(context);

        // 4. 任何人调用超时检查
        assert!(
            contract.check_offer_timeout(task_id.clone()),
            "超时检查应该成功处理"
        );

        // 5. 验证任务已分配给Worker
        let updated_task = contract.get_task(task_id.clone()).unwrap();
        assert_eq!(
            updated_task.status,
            TaskStatus::Assigned,
            "任务状态应该变为Assigned"
        );
        assert_eq!(
            updated_task.assigned_worker.unwrap(),
            worker,
            "任务应该分配给指定的Worker"
        );

        // 6. Worker完成任务
        testing_env!(get_context(worker.clone()));
        assert!(
            contract.complete_task(
                task_id.clone(),
                "ipfs://result_video".to_string(),
                vec![
                    "0.00".to_string(),
                    "2.50".to_string(),
                    "5.75".to_string(),
                    "8.20".to_string()
                ]
            ),
            "完成任务应该成功"
        );

        // 验证任务状态已更新为Completed
        let completed_task = contract.get_task(task_id.clone()).unwrap();
        assert_eq!(
            completed_task.status,
            TaskStatus::Completed,
            "任务状态应该变为Completed"
        );

        // 验证GOP选择
        assert!(
            completed_task.keyframe_timestamps.is_some(),
            "任务应有关键帧时间戳"
        );
        assert!(completed_task.selected_gops.is_some(), "任务应有选定的GOP");
        println!("关键帧时间戳: {:?}", completed_task.keyframe_timestamps);
        println!("选定用于验证的GOP: {:?}", completed_task.selected_gops);

        assert!(completed_task.result_ipfs.is_some(), "任务应该有结果IPFS");
        assert!(
            completed_task.completion_time.is_some(),
            "任务应该有完成时间"
        );

        // 验证验证者已分配
        assert!(
            !completed_task.assigned_verifiers.is_empty(),
            "任务应该已分配验证者"
        );
        println!("分配的验证者: {:?}", completed_task.assigned_verifiers);

        // 确认第一个验证者是verifier1
        assert!(
            completed_task.assigned_verifiers.contains(&verifier1),
            "任务应该分配给verifier1"
        );

        // 7. 验证者1查询分配的任务
        testing_env!(get_context(verifier1.clone()));
        let assigned_tasks = contract.query_assigned_tasks(verifier1.clone(), Some(true));
        assert_eq!(assigned_tasks.len(), 1, "verifier1应该有1个分配的任务");
        assert_eq!(assigned_tasks[0].task_id, task_id, "分配的任务ID应该匹配");

        // 8. 验证者1提交验证结果
        let verifier1_proof = VerifierQosProof {
            id: "".to_string(), // 留空，让合约生成ID
            task_id: task_id.clone(),
            verifier_id: verifier1.clone(),
            timestamp: env::block_timestamp(),
            video_specs: VideoSpecification {
                codec: "h264".to_string(),
                resolution: "1920x1080".to_string(),
                bitrate: 5000,
                framerate: 30.0,
            },
            video_score: 92.5, // 较高的VMAF分数
            // 修改gop_scores，使用选定的GOP
            gop_scores: completed_task
                .selected_gops
                .clone()
                .unwrap()
                .iter()
                .map(|timestamp| GopScore {
                    timestamp: timestamp.clone(),
                    vmaf_score: 92.5, // 可以为不同GOP设置不同分数
                    hash: format!("hash-{}", timestamp),
                })
                .collect(),
            audio_score: Some(4.2), // 较好的音频质量
            sync_score: Some(5.0),  // 良好的同步性
            signature: "signature1".to_string(),
        };

        assert!(
            contract.submit_verifier_proof(verifier1_proof),
            "验证者1提交证明应该成功"
        );

        // 9. 检查验证者1提交的证明
        let verifier1_result = contract.get_verifier_proof(task_id.clone(), verifier1.clone());
        assert!(verifier1_result.is_some(), "应该能找到验证者1的证明");
        let proof = verifier1_result.unwrap();
        assert_eq!(proof.verifier_id, verifier1, "证明的验证者ID应该匹配");
        assert_eq!(proof.video_score, 92.5, "视频分数应该匹配");
        assert_eq!(proof.audio_score, Some(4.2), "音频分数应该匹配");
        assert_eq!(proof.sync_score, Some(5.0), "同步分数应该匹配");

        // 10. 验证任务验证状态
        let verification_status = contract.get_task_verification_status(task_id.clone());
        assert!(verification_status.is_some(), "应该有任务验证状态");
        let status = verification_status.unwrap();
        assert_eq!(status.verified_by.len(), 1, "应该有1个验证者已验证");
        assert_eq!(status.verified_by[0], verifier1, "验证者应该是verifier1");

        // 11. 验证者1再次查询任务，确认不会返回已验证的任务
        let assigned_tasks_after = contract.query_assigned_tasks(verifier1.clone(), Some(true));
        assert_eq!(
            assigned_tasks_after.len(),
            0,
            "verifier1不应该有未验证的任务"
        );

        // 12. 验证者2查询分配的任务
        testing_env!(get_context(verifier2.clone()));
        let assigned_tasks_2 = contract.query_assigned_tasks(verifier2.clone(), Some(true));
        assert_eq!(assigned_tasks_2.len(), 1, "verifier2应该有1个分配的任务");

        // 13. 验证者2提交验证结果
        let verifier2_proof = VerifierQosProof {
            id: "".to_string(),
            task_id: task_id.clone(),
            verifier_id: verifier2.clone(),
            timestamp: env::block_timestamp(),
            video_specs: VideoSpecification {
                codec: "h264".to_string(),
                resolution: "1920x1080".to_string(),
                bitrate: 5000,
                framerate: 30.0,
            },
            // 故意设置不同的评分，模拟不一致的验证结果
            video_score: 87.0,
            // 修改gop_scores，使用选定的GOP
            gop_scores: completed_task
                .selected_gops
                .clone()
                .unwrap()
                .iter()
                .map(|timestamp| GopScore {
                    timestamp: timestamp.clone(),
                    vmaf_score: 92.5, // 可以为不同GOP设置不同分数
                    hash: format!("hash-{}", timestamp),
                })
                .collect(),
            audio_score: Some(3.9),
            sync_score: Some(10.0),
            signature: "signature2".to_string(),
        };

        assert!(
            contract.submit_verifier_proof(verifier2_proof),
            "验证者2提交证明应该成功"
        );

        // 14. 获取所有验证结果
        let all_proofs = contract.get_task_proofs(task_id.clone());
        assert_eq!(all_proofs.len(), 2, "应该有2个验证结果");

        // 在验证获取证明的测试部分
        assert_eq!(
            proof.gop_scores.len(),
            completed_task.selected_gops.clone().unwrap().len(),
            "GOP分数数量应匹配选定的GOP数量"
        );

        // 验证是否包含所有选定的GOP
        for timestamp in completed_task.selected_gops.unwrap() {
            assert!(
                proof
                    .gop_scores
                    .iter()
                    .any(|score| score.timestamp == timestamp),
                "应包含选定GOP的分数: {}",
                timestamp
            );
        }

        // 15. 验证任务验证状态已更新
        let verification_status_after = contract.get_task_verification_status(task_id.clone());
        assert!(verification_status_after.is_some(), "应该有任务验证状态");
        let status_after = verification_status_after.unwrap();
        assert_eq!(status_after.verified_by.len(), 2, "应该有2个验证者已验证");
        assert!(
            status_after.verified_by.contains(&verifier1),
            "验证者列表应该包含verifier1"
        );
        assert!(
            status_after.verified_by.contains(&verifier2),
            "验证者列表应该包含verifier2"
        );

        // 16. 测试重复提交
        testing_env!(get_context(verifier1.clone()));
        let verifier1_proof_duplicate = VerifierQosProof {
            id: "".to_string(),
            task_id: task_id.clone(),
            verifier_id: verifier1.clone(),
            timestamp: env::block_timestamp(),
            video_specs: VideoSpecification {
                codec: "h264".to_string(),
                resolution: "1920x1080".to_string(),
                bitrate: 5000,
                framerate: 30.0,
            },
            video_score: 95.0, // 不同的分数
            gop_scores: vec![],
            audio_score: None,
            sync_score: None,
            signature: "signature_duplicate".to_string(),
        };

        // 这里应该会失败，因为verifier1已经提交过
        let result = std::panic::catch_unwind(move || {
            contract.submit_verifier_proof(verifier1_proof_duplicate)
        });
        assert!(result.is_err(), "重复提交应该失败");

        panic!("测试结束，查看输出");

        // 17. 准备委员会提交共识结果（这里可以扩展测试委员会共识部分）
        // ...
    }

    // #[test]
    // fn test_normal_task_flow() {
    //     // 设置合约和账户
    //     let (mut contract, owner) = setup_contract();
    //     let broadcaster = accounts(3);
    //     let worker = accounts(4);
    //     let anyone = accounts(5);

    //     // 设置上下文为所有者
    //     let context = get_context(owner.clone());
    //     testing_env!(context);

    //     // 使用硬编码方法初始化验证者
    //     contract.initialize_verifiers_with_hardcoded_members();

    //     // 在调用select_and_assign_verifiers之前，打印当前调用者
    //     testing_env!(get_context(broadcaster.clone()));
    //     println!("broadcaster是: {:?}", env::predecessor_account_id());

    //     // 注册Worker
    //     testing_env!(get_context(worker.clone()));
    //     println!("worker是: {:?}", env::predecessor_account_id());
    //     assert!(contract.register_worker(true), "Worker注册应该成功"); // 注册支持硬件加速的Worker

    //     // 1. Broadcaster发布任务
    //     testing_env!(get_context(broadcaster.clone()));
    //     let task_id = contract.publish_task(
    //         "ipfs://source_video".to_string(),
    //         TranscodingRequirement {
    //             target_codec: "h264".to_string(),
    //             target_resolution: "1920x1080".to_string(),
    //             target_bitrate: "5000".to_string(),
    //             target_framerate: "30".to_string(),
    //             additional_params: "".to_string(),
    //         },
    //         true, // 偏好硬件加速
    //     );

    //     // 验证任务状态为OfferCollecting
    //     let task = contract.get_task(task_id.clone()).unwrap();
    //     assert_eq!(
    //         task.status,
    //         TaskStatus::OfferCollecting,
    //         "任务状态应该是OfferCollecting"
    //     );

    //     // 记录任务发布时间
    //     let publish_time = task.publish_time;

    //     // 2. Worker提交Offer
    //     testing_env!(get_context(worker.clone()));
    //     assert!(
    //         contract.submit_offer(task_id.clone()),
    //         "Worker提交offer应该成功"
    //     );

    //     // 验证offer已提交
    //     let offers = contract.get_task_offers(task_id.clone()).unwrap();
    //     assert_eq!(offers.len(), 1, "应该有1个offer");
    //     assert_eq!(offers[0].worker_id, worker, "offer的worker_id应该匹配");

    //     // // 3. 推进时间，确保超过超时阈值（使用30秒而不是8秒，以防万一）
    //     // let mut context = get_context(broadcaster.clone()); // 由broadcaster调用超时检查
    //     //                                                     // 新时间戳 = 发布时间 + 30秒
    //     // context.block_timestamp = publish_time + 10_000_000_000;
    //     // testing_env!(context);

    //     // 3. 推进时间8秒（模拟等待超时）
    //     let mut context = get_context(anyone.clone());
    //     // 新的时间戳应该是原始时间戳加上8秒（以纳秒为单位）
    //     context.block_timestamp = publish_time + 9_000_000_000;
    //     testing_env!(context);

    //     // 4. 任何人调用超时检查
    //     assert!(
    //         contract.check_offer_timeout(task_id.clone()),
    //         "超时检查应该成功处理"
    //     );

    //     // 5. 验证任务已分配给Worker（系统内部已调用select_best_offer和assign_task_to_worker）
    //     let updated_task = contract.get_task(task_id.clone()).unwrap();
    //     assert_eq!(
    //         updated_task.status,
    //         TaskStatus::Assigned,
    //         "任务状态应该变为Assigned"
    //     );
    //     assert_eq!(
    //         updated_task.assigned_worker.unwrap(),
    //         worker,
    //         "任务应该分配给指定的Worker"
    //     );

    //     // 6. 检查被选中的Worker的QoS评分
    //     let worker_info = contract.get_worker_info(worker.clone()).unwrap();
    //     println!("选中Worker的QoS评分: {}", worker_info.qos_score);

    //     // 获取更详细的QoS评分信息
    //     if let Some(qos_details) = contract.get_worker_qos_details(worker.clone()) {
    //         println!("服务可靠性评分: {}", qos_details.service_reliability_score);
    //         println!("时间稳定性评分: {}", qos_details.time_stability_score);
    //         println!("性能表现评分: {}", qos_details.performance_score);
    //         println!("总体QoS评分: {}", qos_details.overall_qos_score);
    //         println!("最后更新时间: {}", qos_details.last_update_time);
    //     } else {
    //         println!("未找到Worker的QoS详情");
    //     }

    //     // 如果有性能历史记录，也可以检查
    //     if let Some(summary) = contract.get_worker_performance_summary(worker.clone()) {
    //         println!("总完成任务数: {}", summary.total_tasks_completed);
    //         println!("平均视频分数: {}", summary.avg_video_score);
    //         println!("平均音频分数: {}", summary.avg_audio_score);
    //         println!("平均编码时间: {}ms", summary.avg_encoding_duration);
    //         println!("完成率: {}%", summary.completion_rate * 100.0);
    //         println!("合规率: {}%", summary.compliance_rate * 100.0);
    //         println!("最近7天服务天数: {}", summary.service_days_last_week);
    //     } else {
    //         println!("未找到Worker的性能摘要");
    //     }

    //     // assert!(false, "故意中断测试以查看输出");

    //     // 验证Worker状态已更新
    //     let worker_info = contract.get_worker_info(worker.clone()).unwrap();
    //     assert_eq!(
    //         worker_info.current_task.unwrap(),
    //         task_id,
    //         "Worker的当前任务应该已更新"
    //     );

    //     // 6. Worker完成任务
    //     testing_env!(get_context(worker.clone()));
    //     assert!(
    //         contract.complete_task(task_id.clone(), "ipfs://result_video".to_string()),
    //         "完成任务应该成功"
    //     );

    //     // 验证任务状态已更新为Completed
    //     let completed_task = contract.get_task(task_id.clone()).unwrap();
    //     assert_eq!(
    //         completed_task.status,
    //         TaskStatus::Completed,
    //         "任务状态应该变为Completed"
    //     );
    //     assert!(completed_task.result_ipfs.is_some(), "任务应该有结果IPFS");
    //     assert!(
    //         completed_task.completion_time.is_some(),
    //         "任务应该有完成时间"
    //     );

    //     // 验证Worker状态已释放
    //     let updated_worker_info = contract.get_worker_info(worker.clone()).unwrap();
    //     assert!(
    //         updated_worker_info.current_task.is_none(),
    //         "Worker的当前任务应该被释放"
    //     );

    //     // 验证验证者已分配（虽然我们不测试验证过程）
    //     assert!(
    //         !completed_task.assigned_verifiers.is_empty(),
    //         "任务应该已分配验证者"
    //     );
    // }

    // #[test]
    // fn test_worker_heartbeat() {
    //     // 设置合约和账户
    //     let (mut contract, _owner) = setup_contract();
    //     let worker = accounts(4);

    //     // 1. 测试未注册的Worker调用heartbeat
    //     testing_env!(get_context(worker.clone()));
    //     let heartbeat_result = contract.worker_heartbeat();
    //     assert!(!heartbeat_result, "未注册Worker的心跳应该失败");

    //     // 2. 注册Worker
    //     testing_env!(get_context(worker.clone()));
    //     assert!(contract.register_worker(true), "Worker注册应该成功");

    //     // 创建测试任务并设置为队列状态
    //     let broadcaster = accounts(3);
    //     testing_env!(get_context(broadcaster.clone()));
    //     let task_id = contract.publish_task(
    //         "ipfs://source_video".to_string(),
    //         TranscodingRequirement {
    //             target_codec: "h264".to_string(),
    //             target_resolution: "1920x1080".to_string(),
    //             target_bitrate: "5000".to_string(),
    //             target_framerate: "30".to_string(),
    //             additional_params: "".to_string(),
    //         },
    //         true,
    //     );

    //     // 手动将任务设置为Queued状态
    //     contract.move_task_to_queue(&task_id);

    //     // 验证任务状态为Queued
    //     let task = contract.get_task(task_id.clone()).unwrap();
    //     assert_eq!(task.status, TaskStatus::Queued, "任务应该处于Queued状态");

    //     // 确保Worker可用且没有当前任务
    //     let worker_info = contract.get_worker_info(worker.clone()).unwrap();
    //     assert!(worker_info.available, "Worker应该处于可用状态");
    //     assert!(worker_info.current_task.is_none(), "Worker不应有当前任务");

    //     // 3. 测试已注册的Worker调用heartbeat，应该自动处理队列任务
    //     testing_env!(get_context(worker.clone()));
    //     // 模拟不同的时间戳
    //     let mut context = get_context(worker.clone());
    //     context.block_timestamp = 2_000_000_000; // 不同于注册时的时间戳
    //     testing_env!(context);

    //     let heartbeat_result = contract.worker_heartbeat();
    //     assert!(heartbeat_result, "已注册Worker的心跳应该成功");

    //     // 4. 验证任务是否已分配给Worker
    //     let updated_task = contract.get_task(task_id.clone()).unwrap();
    //     assert_eq!(
    //         updated_task.status,
    //         TaskStatus::Assigned,
    //         "任务状态应该更新为Assigned"
    //     );
    //     assert_eq!(
    //         updated_task.assigned_worker.unwrap(),
    //         worker,
    //         "任务应该分配给正确的Worker"
    //     );

    //     // 5. 验证Worker信息是否正确更新
    //     let updated_worker = contract.get_worker_info(worker.clone()).unwrap();
    //     assert_eq!(
    //         updated_worker.current_task.unwrap(),
    //         task_id,
    //         "Worker的当前任务应该已更新"
    //     );

    //     // 注意：这里检查的是毫秒值，而不是纳秒值
    //     assert_eq!(
    //         updated_worker.last_heartbeat,
    //         2_000_000_000 / 1_000_000, // 转换为毫秒后应为2000
    //         "Worker的最后心跳时间应该已更新为毫秒单位"
    //     );

    //     // 6. 验证每日统计是否正确更新
    //     let worker_history = contract.worker_performance_history.get(&worker).unwrap();

    //     // 计算当前天的时间戳（毫秒单位）
    //     const ONE_DAY_MS: u64 = 24 * 60 * 60 * 1000; // 一天的毫秒数
    //     let current_day_timestamp = (2_000_000_000 / 1_000_000) / ONE_DAY_MS * ONE_DAY_MS;

    //     let found_stats = worker_history
    //         .daily_stats
    //         .iter()
    //         .find(|stat| stat.date == current_day_timestamp);
    //     assert!(found_stats.is_some(), "应该存在当天的统计数据");
    //     assert!(
    //         found_stats.unwrap().provided_service,
    //         "当天的provided_service应该为true"
    //     );
    // }

    // fn test_queue_task_flow() {
    //     // 设置合约和账户
    //     let (mut contract, owner) = setup_contract();
    //     let broadcaster = accounts(3);
    //     let worker = accounts(4);

    //     // 设置上下文为所有者
    //     let context = get_context(owner.clone());
    //     testing_env!(context);

    //     // 使用硬编码方法初始化验证者
    //     contract.initialize_verifiers_with_hardcoded_members();

    //     // 0. 注册Worker（此时任务已在队列中）
    //     testing_env!(get_context(worker.clone()));
    //     assert!(contract.register_worker(true), "Worker注册应该成功");

    //     // 1. Broadcaster发布任务
    //     testing_env!(get_context(broadcaster.clone()));
    //     let task_id = contract.publish_task(
    //         "ipfs://source_video".to_string(),
    //         TranscodingRequirement {
    //             target_codec: "h264".to_string(),
    //             target_resolution: "1920x1080".to_string(),
    //             target_bitrate: "5000".to_string(),
    //             target_framerate: "30".to_string(),
    //             additional_params: "".to_string(),
    //         },
    //         true, // 偏好硬件加速
    //     );

    //     // 验证任务状态为OfferCollecting
    //     let task = contract.get_task(task_id.clone()).unwrap();
    //     assert_eq!(task.status, TaskStatus::OfferCollecting);
    //     let publish_time = task.publish_time;

    //     // 2. 推进时间30秒（模拟等待超时，无Worker提交offer）
    //     let mut context = get_context(broadcaster.clone());
    //     context.block_timestamp = publish_time + 30_000_000_000;
    //     testing_env!(context);

    //     // 3. Broadcaster调用超时检查
    //     let timeout_result = contract.check_offer_timeout(task_id.clone());
    //     println!("超时检查结果: {}", timeout_result);

    //     // 4. 验证任务状态已更新为Queued
    //     let queued_task = contract.get_task(task_id.clone()).unwrap();
    //     assert_eq!(
    //         queued_task.status,
    //         TaskStatus::Queued,
    //         "任务状态应该变为Queued"
    //     );

    //     testing_env!(get_context(worker.clone()));
    //     // 6. Worker调用heartbeat（这应该触发从队列中获取任务）
    //     let heartbeat_result = contract.worker_heartbeat();
    //     assert!(heartbeat_result, "Worker心跳应该成功");

    //     // 7. 验证任务已从队列中分配给Worker
    //     let assigned_task = contract.get_task(task_id.clone()).unwrap();
    //     assert_eq!(
    //         assigned_task.status,
    //         TaskStatus::Assigned,
    //         "任务状态应该变为Assigned"
    //     );
    //     assert_eq!(
    //         assigned_task.assigned_worker.unwrap(),
    //         worker,
    //         "任务应该分配给正确的Worker"
    //     );

    //     // 验证Worker状态已更新
    //     let worker_info = contract.get_worker_info(worker.clone()).unwrap();
    //     assert_eq!(
    //         worker_info.current_task.unwrap(),
    //         task_id,
    //         "Worker的当前任务应该已更新"
    //     );

    //     // 8. Worker完成任务
    //     testing_env!(get_context(worker.clone()));
    //     assert!(
    //         contract.complete_task(task_id.clone(), "ipfs://result_video".to_string()),
    //         "完成任务应该成功"
    //     );

    //     // 9. 验证任务状态已更新为Completed
    //     let completed_task = contract.get_task(task_id.clone()).unwrap();
    //     assert_eq!(
    //         completed_task.status,
    //         TaskStatus::Completed,
    //         "任务状态应该变为Completed"
    //     );
    // }

    // 辅助函数：直接更新Worker的性能历史记录以影响QoS评分
    fn update_worker_performance_directly(
        contract: &mut MediaTranscodingContract,
        worker_id: &AccountId,
        service_reliability: f64,
        time_stability: f64,
        performance_score: f64,
    ) {
        // 切换到合约所有者上下文，确保有权限
        testing_env!(get_context(contract.owner_id.clone()));

        // 获取Worker的性能历史记录
        if let Some(mut history) = contract.worker_performance_history.get(worker_id).cloned() {
            // 清空现有的每日统计数据
            history.daily_stats.clear();

            // 创建过去7天的每日统计数据
            let current_timestamp = env::block_timestamp() / 1_000_000;

            // 根据目标评分设置参数
            let (tasks_completed, active_days) =
                match (service_reliability, time_stability, performance_score) {
                    (s, t, p) if s > 0.8 && t > 0.8 && p > 0.8 => (10, 7), // 高QoS
                    (s, t, p) if s > 0.6 && t > 0.6 && p > 0.6 => (7, 5),  // 中QoS
                    _ => (4, 3),                                           // 低QoS
                };

            // 设置编码持续时间
            let encoding_duration = match performance_score {
                p if p > 0.8 => 30_000, // 高性能：30秒
                p if p > 0.6 => 45_000, // 中性能：45秒
                _ => 60_000,            // 低性能：60秒
            };

            // 假设标准视频是30 FPS，计算相应的帧数
            let video_frames = 1800; // 60秒视频，30 FPS = 1800帧

            // 根据编码时间计算fps
            let fps = match performance_score {
                p if p > 0.8 => 40.0, // 高性能：更高的FPS
                p if p > 0.6 => 30.0, // 中性能：标准FPS
                _ => 20.0,            // 低性能：较低的FPS
            };

            // 添加每日统计数据
            for i in 0..7 {
                let is_active = i < active_days;
                let day_timestamp = current_timestamp - (i * ONE_DAY_MS);
                let day_timestamp_normalized = day_timestamp / ONE_DAY_MS * ONE_DAY_MS;

                let stats = WorkerDailyStats {
                    date: day_timestamp_normalized,
                    tasks_completed: if is_active { tasks_completed } else { 0 },
                    tasks_accepted: if is_active { tasks_completed } else { 0 }, // 简化：假设完成率为100%
                    provided_service: is_active,
                    encoding_fps: if is_active {
                        vec![fps; tasks_completed as usize]
                    } else {
                        Vec::new()
                    },
                    quality_scores: if is_active {
                        vec![0.9; tasks_completed as usize]
                    } else {
                        Vec::new()
                    },
                };

                history.daily_stats.push(stats);
            }

            // 更新总任务完成数
            history.total_tasks_completed =
                history.daily_stats.iter().map(|s| s.tasks_completed).sum();

            // 保存更新后的历史记录
            contract
                .worker_performance_history
                .insert(worker_id.clone(), history.clone());

            // 重新计算QoS评分
            let qos_score = contract.calculate_worker_qos_score(&history);

            // 更新WorkerInfo
            if let Some(mut worker_info) = contract.active_workers.get(worker_id).cloned() {
                worker_info.qos_score = qos_score;
                worker_info.service_reliability_score = service_reliability;
                worker_info.time_stability_score = time_stability;
                worker_info.performance_score = performance_score;
                worker_info.total_tasks_completed = history.total_tasks_completed;
                worker_info.active_days_last_week = active_days as u32;

                contract
                    .active_workers
                    .insert(worker_id.clone(), worker_info);
            }

            println!(
                "Worker {} 的性能历史记录已更新，QoS评分为: {}",
                worker_id, qos_score
            );
        } else {
            println!("未找到Worker {} 的性能历史记录", worker_id);
        }
    }

    // #[test]
    // fn test_qos_based_worker_selection_and_verifier_assignment() {
    //     // 设置合约和账户
    //     let (mut contract, owner) = setup_contract();
    //     let broadcaster = accounts(1);

    //     // 创建多个Worker账户
    //     let worker_high_qos = accounts(2);
    //     let worker_medium_qos = accounts(3);
    //     let worker_low_qos = accounts(4);

    //     // 设置上下文为所有者初始化验证者
    //     testing_env!(get_context(owner.clone()));
    //     contract.initialize_verifiers_with_hardcoded_members();

    //     // 注册所有Worker
    //     testing_env!(get_context(worker_high_qos.clone()));
    //     contract.register_worker(true);

    //     testing_env!(get_context(worker_medium_qos.clone()));
    //     contract.register_worker(true);

    //     testing_env!(get_context(worker_low_qos.clone()));
    //     contract.register_worker(true);

    //     // 输出初始QoS评分
    //     println!(
    //         "初始高QoS Worker评分: {}",
    //         contract
    //             .get_worker_qos_score(worker_high_qos.clone())
    //             .unwrap()
    //     );
    //     println!(
    //         "初始中QoS Worker评分: {}",
    //         contract
    //             .get_worker_qos_score(worker_medium_qos.clone())
    //             .unwrap()
    //     );
    //     println!(
    //         "初始低QoS Worker评分: {}",
    //         contract
    //             .get_worker_qos_score(worker_low_qos.clone())
    //             .unwrap()
    //     );

    //     // 直接更新Worker的性能历史记录以影响QoS评分
    //     update_worker_performance_directly(&mut contract, &worker_high_qos, 0.9, 0.8, 0.9); // 高QoS
    //     update_worker_performance_directly(&mut contract, &worker_medium_qos, 0.7, 0.6, 0.7); // 中QoS
    //     update_worker_performance_directly(&mut contract, &worker_low_qos, 0.5, 0.4, 0.5); // 低QoS

    //     // 验证QoS评分设置成功
    //     let high_qos = contract
    //         .get_worker_qos_score(worker_high_qos.clone())
    //         .unwrap();
    //     let medium_qos = contract
    //         .get_worker_qos_score(worker_medium_qos.clone())
    //         .unwrap();
    //     let low_qos = contract
    //         .get_worker_qos_score(worker_low_qos.clone())
    //         .unwrap();

    //     println!("修改后高QoS Worker评分: {}", high_qos);
    //     println!("修改后中QoS Worker评分: {}", medium_qos);
    //     println!("修改后低QoS Worker评分: {}", low_qos);

    //     assert!(high_qos > medium_qos, "高QoS Worker应大于中QoS Worker");
    //     assert!(medium_qos > low_qos, "中QoS Worker应大于低QoS Worker");

    //     // Broadcaster发布任务
    //     testing_env!(get_context(broadcaster.clone()));
    //     let task_id = contract.publish_task(
    //         "ipfs://source_video".to_string(),
    //         TranscodingRequirement {
    //             target_codec: "h264".to_string(),
    //             target_resolution: "1920x1080".to_string(),
    //             target_bitrate: "5000".to_string(),
    //             target_framerate: "30".to_string(),
    //             additional_params: "".to_string(),
    //         },
    //         true,
    //     );

    //     // 所有Worker提交offer
    //     testing_env!(get_context(worker_high_qos.clone()));
    //     contract.submit_offer(task_id.clone());

    //     testing_env!(get_context(worker_medium_qos.clone()));
    //     contract.submit_offer(task_id.clone());

    //     testing_env!(get_context(worker_low_qos.clone()));
    //     contract.submit_offer(task_id.clone());

    //     // 检查offers已提交
    //     let offers = contract.get_task_offers(task_id.clone()).unwrap();
    //     assert_eq!(offers.len(), 3, "应该有3个offer");

    //     // 推进时间并触发超时检查 - 使用毫秒单位
    //     let mut context = get_context(broadcaster.clone());
    //     context.block_timestamp = context.block_timestamp + 10_000_000_000; // 10秒
    //     testing_env!(context);

    //     let timeout_result = contract.check_offer_timeout(task_id.clone());
    //     assert!(timeout_result, "超时检查应该返回true");

    //     // 验证任务分配给了高QoS Worker
    //     let task = contract.get_task(task_id.clone()).unwrap();
    //     assert_eq!(task.status, TaskStatus::Assigned, "任务应该已分配");
    //     assert_eq!(
    //         task.assigned_worker.unwrap(),
    //         worker_high_qos,
    //         "任务应该分配给QoS最高的Worker"
    //     );

    //     // Worker完成任务
    //     testing_env!(get_context(worker_high_qos.clone()));
    //     let complete_result =
    //         contract.complete_task(task_id.clone(), "ipfs://result_video".to_string());
    //     assert!(complete_result, "任务完成应该成功");

    //     // 验证任务状态已更新为Completed
    //     let task = contract.get_task(task_id.clone()).unwrap();
    //     assert_eq!(
    //         task.status,
    //         TaskStatus::Completed,
    //         "任务状态应该为Completed"
    //     );

    //     // 验证验证者是否已分配
    //     assert!(!task.assigned_verifiers.is_empty(), "应该有验证者被分配");
    //     println!("分配的验证者数量: {}", task.assigned_verifiers.len());
    //     println!("验证者列表: {:?}", task.assigned_verifiers);
    //     assert!(task.assigned_verifiers.len() >= 2, "应该至少分配两个验证者");

    //     // 检查验证者不重复
    //     let mut unique_verifiers = task.assigned_verifiers.clone();
    //     unique_verifiers.sort();
    //     unique_verifiers.dedup();
    //     assert_eq!(
    //         unique_verifiers.len(),
    //         task.assigned_verifiers.len(),
    //         "验证者列表中不应有重复"
    //     );

    //     // 测试请求补充验证者
    //     testing_env!(get_context(broadcaster.clone()));
    //     let supplemental_verifier = contract.request_supplemental_verifier(task_id.clone());
    //     assert!(supplemental_verifier.is_some(), "应该能获取到补充验证者");
    //     println!("补充验证者: {:?}", supplemental_verifier.unwrap());

    //     // 验证补充验证者已添加
    //     let updated_task = contract.get_task(task_id.clone()).unwrap();
    //     assert!(
    //         updated_task.assigned_verifiers.len() > task.assigned_verifiers.len(),
    //         "验证者数量应该增加"
    //     );

    //     // 故意中断以查看输出
    //     // panic!("测试结束，查看输出");
    // }

    // #[test]
    // fn test_submit_consensus_proof_updates_worker_qos() {
    //     // 设置合约和账户
    //     let (mut contract, owner) = setup_contract();
    //     let worker_id = accounts(1);
    //     // let committee_leader = accounts(2);

    //     // 注册worker
    //     testing_env!(get_context(worker_id.clone()));
    //     contract.register_worker(true);

    //     // 获取Worker性能摘要，检查详细指标
    //     // if let Some(summary) = contract.get_worker_performance_summary(worker_id.clone()) {
    //     //     println!("Worker性能摘要:");
    //     //     println!("总完成任务数: {}", summary.total_tasks_completed);
    //     //     println!("平均视频分数: {}", summary.avg_video_score);
    //     //     println!("平均音频分数: {}", summary.avg_audio_score);
    //     //     println!("平均同步分数: {}", summary.avg_sync_score);
    //     //     println!("平均编码时间: {}", summary.avg_encoding_duration);
    //     //     println!("任务完成率: {}", summary.completion_rate);
    //     //     println!("质量达标率: {}", summary.compliance_rate);
    //     //     println!("最近7天服务天数: {}", summary.service_days_last_week);
    //     //     println!("当前QoS评分: {}", summary.qos_score);
    //     // }

    //     // 初始化委员会成员
    //     testing_env!(get_context(owner.clone()));
    //     contract.initialize_committee_with_hardcoded_members();
    //     // contract.init_committee(committee_members);

    //     // 初始化验证者
    //     contract.initialize_verifiers_with_hardcoded_members();

    //     let committee_leader = contract
    //         .get_committee_leader()
    //         .expect("找不到委员会leader")
    //         .account_id;

    //     // 获取初始QoS评分
    //     let initial_qos = contract.get_worker_qos_score(worker_id.clone()).unwrap();
    //     println!("Worker初始QoS评分: {}", initial_qos);

    //     // 发布任务
    //     let broadcaster = accounts(4);
    //     testing_env!(get_context(broadcaster.clone()));
    //     let task_id = contract.publish_task(
    //         "ipfs://source_video".to_string(),
    //         TranscodingRequirement {
    //             target_codec: "h264".to_string(),
    //             target_resolution: "1920x1080".to_string(),
    //             target_bitrate: "5000".to_string(),
    //             target_framerate: "30".to_string(),
    //             additional_params: "".to_string(),
    //         },
    //         false,
    //     );

    //     // 验证任务状态为OfferCollecting
    //     let task = contract.get_task(task_id.clone()).unwrap();
    //     assert_eq!(
    //         task.status,
    //         TaskStatus::OfferCollecting,
    //         "任务状态应该是OfferCollecting"
    //     );

    //     // 记录任务发布时间
    //     let publish_time = task.publish_time;

    //     // 模拟任务分配给worker
    //     // 1. 提交offer
    //     testing_env!(get_context(worker_id.clone()));
    //     contract.submit_offer(task_id.clone());

    //     // 2. 触发offer超时检查，分配任务

    //     // 3. 推进时间，确保超过超时阈值（使用30秒而不是8秒，以防万一）
    //     let mut context = get_context(broadcaster.clone()); // 由broadcaster调用超时检查
    //                                                         // 新时间戳 = 发布时间 + 30秒
    //     context.block_timestamp = publish_time + 10_000_000_000;
    //     testing_env!(context);

    //     // testing_env!(get_context(broadcaster.clone()));
    //     // let context = get_context(broadcaster.clone());
    //     // testing_env!(context.block_timestamp(context.block_timestamp + 10_000_000_000));
    //     contract.check_offer_timeout(task_id.clone());

    //     // 3. 完成任务
    //     testing_env!(get_context(worker_id.clone()));
    //     contract.complete_task(task_id.clone(), "ipfs://result_video".to_string());

    //     // 创建QoS质量证明
    //     let consensus_proof = ConsensusQosProof {
    //         task_id: task_id.clone(),
    //         worker_id: worker_id.clone(),
    //         timestamp: env::block_timestamp(),
    //         committee_members: vec![committee_leader.clone(), accounts(3)],
    //         committee_leader: committee_leader.clone(),
    //         video_score: 95.0, // 优秀的视频分数
    //         frame_count: 600,
    //         audio_score: 4.8, // 优秀的音频分数
    //         sync_score: 0.0,  // 优秀的同步分数
    //         encoding_start_time: env::block_timestamp() - 35_000_000_000, // 35秒前
    //         encoding_end_time: env::block_timestamp(),
    //         video_specs: VideoSpecification {
    //             codec: "h264".to_string(),
    //             resolution: "1920x1080".to_string(),
    //             bitrate: 5000,
    //             framerate: 30.0,
    //         },
    //         specified_gop_scores: vec![
    //             GopScore {
    //                 gop_id: 1,
    //                 vmaf_score: 95.0,
    //                 hash: "hash1".to_string(),
    //             },
    //             GopScore {
    //                 gop_id: 100,
    //                 vmaf_score: 94.5,
    //                 hash: "hash100".to_string(),
    //             },
    //         ],
    //         gop_verification: GopVerificationResult::Verified,
    //         status: QosProofStatus::Normal,
    //     };

    //     // 以委员会leader身份提交QoS证明
    //     testing_env!(get_context(committee_leader.clone()));
    //     let submit_result =
    //         contract.submit_consensus_proof(consensus_proof, QosProofStatus::Normal);

    //     // 验证结果
    //     assert!(submit_result, "提交QoS证明应成功");

    //     // 获取更新后的QoS评分
    //     let updated_qos = contract.get_worker_qos_score(worker_id.clone()).unwrap();
    //     println!("提交QoS证明后，Worker的QoS评分: {}", updated_qos);

    //     // 验证QoS评分已更新且高于初始值
    //     assert!(
    //         updated_qos != initial_qos,
    //         "提交高质量的QoS证明应该提高Worker的QoS评分"
    //     );

    //     // 获取Worker性能摘要，检查详细指标
    //     if let Some(summary) = contract.get_worker_performance_summary(worker_id.clone()) {
    //         println!("Worker性能摘要:");
    //         println!("总完成任务数: {}", summary.total_tasks_completed);
    //         println!("平均视频分数: {}", summary.avg_video_score);
    //         println!("平均音频分数: {}", summary.avg_audio_score);
    //         println!("平均同步分数: {}", summary.avg_sync_score);
    //         println!("平均编码时间: {}", summary.avg_encoding_duration);
    //         println!("任务完成率: {}", summary.completion_rate);
    //         println!("质量达标率: {}", summary.compliance_rate);
    //         println!("最近7天服务天数: {}", summary.service_days_last_week);
    //         println!("当前QoS评分: {}", summary.qos_score);
    //     }

    //     // 验证任务状态已更新为Verified
    //     let task = contract.get_task(task_id.clone()).unwrap();
    //     assert_eq!(
    //         task.status,
    //         TaskStatus::Verified,
    //         "任务状态应更新为Verified"
    //     );

    //     // 验证QoS证明已存储
    //     let proof_result = contract.get_consensus_proof(task_id.clone());
    //     assert!(proof_result.is_some(), "应该能够检索到QoS证明");

    //     let res = contract.get_task_consensus_details(task_id).unwrap();
    //     println!("res:{:?}", res);

    //     assert!(false, "暂停")
    // }

    // 辅助函数：创建Worker性能历史记录
    // fn create_performance_history(
    //     contract: &mut MediaTranscodingContract,
    //     worker_id: &AccountId,
    //     video_score: f64,
    //     audio_score: f64,
    //     sync_score: f64,
    // ) {
    //     // 切换到合约所有者上下文，确保有权限
    //     testing_env!(get_context(contract.owner_id.clone()));

    //     // 创建一个模拟任务ID
    //     let task_id = format!("mock-task-{}", worker_id);

    //     // 创建性能记录
    //     let performance = WorkerPerformance {
    //         task_id: task_id.clone(),
    //         worker_id: worker_id.clone(),
    //         timestamp: env::block_timestamp() / 1_000_000, // 转换为毫秒
    //         video_score,
    //         video_quality_compliant: video_score > 75.0,
    //         audio_score: Some(audio_score),
    //         audio_quality_compliant: Some(audio_score > 3.5),
    //         sync_score: Some(sync_score),
    //         sync_quality_compliant: Some(sync_score > 90.0),
    //         encoding_duration: 50_000, // 50秒，毫秒单位
    //         video_specs: VideoSpecification {
    //             codec: "h264".to_string(),
    //             resolution: "1920x1080".to_string(),
    //             bitrate: 5000,
    //             framerate: 30.0,
    //         },
    //         specs_compliant: true,
    //         overall_compliant: true,
    //     };

    //     // 使用内部方法更新Worker性能历史
    //     contract.update_worker_performance_history(worker_id, &performance);

    //     // 打印当前Worker的QoS评分
    //     if let Some(qos_score) = contract.get_worker_qos_score(worker_id.clone()) {
    //         println!("Worker {} 的QoS评分设置为: {}", worker_id, qos_score);
    //     }
    // }

    // #[test]
    // fn test_qos_based_worker_selection() {
    //     // 设置合约和账户
    //     let (mut contract, owner) = setup_contract();
    //     let broadcaster = accounts(1);

    //     // 创建多个Worker账户
    //     let worker_high_qos = accounts(2);
    //     let worker_medium_qos = accounts(3);
    //     let worker_low_qos = accounts(4);

    //     // 注册所有Worker
    //     testing_env!(get_context(worker_high_qos.clone()));
    //     contract.register_worker(true);

    //     testing_env!(get_context(worker_medium_qos.clone()));
    //     contract.register_worker(true);

    //     testing_env!(get_context(worker_low_qos.clone()));
    //     contract.register_worker(true);

    //     // 模拟历史性能记录 - 高QoS Worker
    //     create_performance_history(&mut contract, &worker_high_qos, 95.0, 4.5, 95.0);

    //     // 模拟历史性能记录 - 中QoS Worker
    //     create_performance_history(&mut contract, &worker_medium_qos, 85.0, 4.0, 90.0);

    //     // 模拟历史性能记录 - 低QoS Worker
    //     create_performance_history(&mut contract, &worker_low_qos, 75.0, 3.5, 85.0);

    //     // 验证QoS评分设置成功
    //     assert!(
    //         contract
    //             .get_worker_qos_score(worker_high_qos.clone())
    //             .unwrap()
    //             > contract
    //                 .get_worker_qos_score(worker_medium_qos.clone())
    //                 .unwrap()
    //     );

    //     assert!(
    //         contract
    //             .get_worker_qos_score(worker_medium_qos.clone())
    //             .unwrap()
    //             > contract
    //                 .get_worker_qos_score(worker_low_qos.clone())
    //                 .unwrap()
    //     );

    //     // Broadcaster发布任务
    //     testing_env!(get_context(broadcaster.clone()));
    //     let task_id = contract.publish_task(
    //         "ipfs://source_video".to_string(),
    //         TranscodingRequirement {
    //             target_codec: "h264".to_string(),
    //             target_resolution: "1920x1080".to_string(),
    //             target_bitrate: "5000".to_string(),
    //             target_framerate: "30".to_string(),
    //             additional_params: "".to_string(),
    //         },
    //         true,
    //     );

    //     // 所有Worker提交offer
    //     testing_env!(get_context(worker_high_qos.clone()));
    //     contract.submit_offer(task_id.clone());

    //     testing_env!(get_context(worker_medium_qos.clone()));
    //     contract.submit_offer(task_id.clone());

    //     testing_env!(get_context(worker_low_qos.clone()));
    //     contract.submit_offer(task_id.clone());

    //     // 检查offers已提交
    //     let offers = contract.get_task_offers(task_id.clone()).unwrap();
    //     assert_eq!(offers.len(), 3, "应该有3个offer");

    //     // 推进时间并触发超时检查
    //     let mut context = get_context(broadcaster.clone());
    //     context.block_timestamp = context.block_timestamp + 30_000_000_000;
    //     testing_env!(context);

    //     contract.check_offer_timeout(task_id.clone());

    //     // 验证任务分配给了高QoS Worker
    //     let task = contract.get_task(task_id.clone()).unwrap();
    //     assert_eq!(task.status, TaskStatus::Assigned, "任务应该已分配");
    //     assert_eq!(
    //         task.assigned_worker.unwrap(),
    //         worker_high_qos,
    //         "任务应该分配给QoS最高的Worker"
    //     );
    // }

    // // 辅助函数：创建Worker性能历史记录
    // fn create_performance_history(
    //     contract: &mut MediaTranscodingContract,
    //     worker_id: &AccountId,
    //     video_score: f64,
    //     audio_score: f64,
    //     sync_score: f64,
    // ) {
    //     // 切换到合约所有者上下文，确保有权限
    //     testing_env!(get_context(contract.owner_id.clone()));

    //     // 创建一个模拟任务ID
    //     let task_id = format!("mock-task-{}", worker_id);

    //     // 创建性能记录
    //     let performance = WorkerPerformance {
    //         task_id: task_id.clone(),
    //         worker_id: worker_id.clone(),
    //         timestamp: env::block_timestamp(),
    //         video_score,
    //         video_quality_compliant: video_score > 75.0,
    //         audio_score: Some(audio_score),
    //         audio_quality_compliant: Some(audio_score > 3.5),
    //         sync_score: Some(sync_score),
    //         sync_quality_compliant: Some(sync_score > 90.0),
    //         encoding_duration: 50_000_000, // 50秒
    //         video_specs: VideoSpecification {
    //             codec: "h264".to_string(),
    //             resolution: "1920x1080".to_string(),
    //             bitrate: 5000,
    //             framerate: 30.0,
    //         },
    //         specs_compliant: true,
    //         overall_compliant: true,
    //     };

    //     // 使用内部方法更新Worker性能历史
    //     contract.update_worker_performance_history(worker_id, &performance);
    // }

    // #[test]
    // fn test_submit_consensus_proof() {
    //     let (mut contract, _) = setup_contract();
    //     let broadcaster = accounts(5);
    //     let worker = accounts(4);
    //     let committee_leader = accounts(3);

    //     contract.initialize_committee_with_hardcoded_members();
    //     contract.initialize_verifiers_with_hardcoded_members();

    //     // 获取委员会leader的账户ID
    //     let committee_leader = contract
    //         .get_committee_leader()
    //         .expect("找不到委员会leader")
    //         .account_id;

    //     // 1. 发布任务
    //     testing_env!(get_context(broadcaster.clone()));
    //     let task_id = contract.publish_task(
    //         "ipfs://source_video".to_string(),
    //         TranscodingRequirement {
    //             target_codec: "h264".to_string(),
    //             target_resolution: "1920x1080".to_string(),
    //             target_bitrate: "5000".to_string(),
    //             target_framerate: "30".to_string(),
    //             additional_params: "".to_string(),
    //         },
    //     );

    //     // 2. 分配任务给工作节点
    //     contract.assign_task(task_id.clone(), worker.clone());

    //     // 3. 工作节点完成任务
    //     testing_env!(get_context(worker.clone()));
    //     contract.complete_task(task_id.clone(), "ipfs://result_video".to_string());

    //     // 4. 由broadcaster选择验证者
    //     testing_env!(get_context(broadcaster.clone()));
    //     let verifiers = contract.select_verifiers(task_id.clone());
    //     assert_eq!(verifiers.len(), 2, "应该选择2个验证者");

    //     // 5. 创建共识证明
    //     let consensus_proof = ConsensusQosProof {
    //         task_id: task_id.clone(),
    //         worker_id: worker.clone(),
    //         timestamp: 0, // 将在提交时更新
    //         committee_members: vec![accounts(3), accounts(4)],
    //         committee_leader: committee_leader.clone(),
    //         video_score: 85.5,
    //         audio_score: 4.2,
    //         sync_score: -0.2,
    //         encoding_start_time: 1_000_000_000,
    //         encoding_end_time: 1_100_000_000,
    //         video_specs: VideoSpecification {
    //             codec: "h264".to_string(),
    //             resolution: "1920x1080".to_string(),
    //             bitrate: 5100,
    //             framerate: 30.0,
    //         },
    //         specified_gop_scores: vec![
    //             GopScore {
    //                 gop_id: 1,
    //                 vmaf_score: 86.0,
    //                 hash: "hash1".to_string(),
    //             },
    //             GopScore {
    //                 gop_id: 2,
    //                 vmaf_score: 85.0,
    //                 hash: "hash2".to_string(),
    //             },
    //         ],
    //         gop_verification: GopVerificationResult::Verified,
    //         status: QosProofStatus::Pending,
    //     };

    //     // 6. 提交共识证明 (由委员会leader提交)
    //     testing_env!(get_context(committee_leader.clone()));
    //     let result = contract.submit_consensus_proof(consensus_proof, QosProofStatus::Normal);
    //     assert!(result, "提交共识证明应该成功");

    //     // 7. 检查任务状态是否已更新为Verified
    //     let task = contract.get_task(task_id.clone()).unwrap();
    //     assert_eq!(
    //         task.status,
    //         TaskStatus::Verified,
    //         "任务状态应该更新为Verified"
    //     );

    //     // 8. 检查共识证明是否已正确存储
    //     let stored_proof = contract.get_consensus_proof(task_id.clone()).unwrap();
    //     assert_eq!(
    //         stored_proof.status,
    //         QosProofStatus::Normal,
    //         "共识证明状态应该是Normal"
    //     );
    //     assert!(stored_proof.timestamp > 0, "时间戳应该已更新");

    //     // 9. 检查工作节点性能记录是否已创建
    //     let worker_perf_key = format!("{}:{}", task_id, worker);
    //     let worker_perf = contract.worker_performances.get(&worker_perf_key);
    //     assert!(worker_perf.is_some(), "应该创建工作节点性能记录");

    //     let perf = worker_perf.unwrap();
    //     assert_eq!(perf.worker_id, worker, "工作节点ID应该匹配");
    //     assert_eq!(perf.video_score, 85.5, "视频分数应该正确");
    //     assert_eq!(perf.encoding_duration, 100_000_000, "编码时间应该正确");
    //     assert!(perf.overall_compliant, "整体性能应该达标");
    // }

    // // 测试提交共识证明
    // #[test]
    // fn test_submit_consensus_proof() {
    //     let (mut contract, _) = setup_contract();
    //     let broadcaster = accounts(1);
    //     let worker = accounts(2);
    //     let committee_leader = accounts(3);

    //     // 添加委员会成员
    //     testing_env!(get_context(accounts(0)));
    //     contract.add_committee_member(committee_leader.clone(), true);

    //     // 模拟广播者上下文
    //     testing_env!(get_context(broadcaster.clone()));

    //     // 创建并分配任务
    //     let task_id = "task1".to_string();
    //     contract.create_task(task_id.clone(), "Test Task".to_string(), "Description".to_string());
    //     contract.assign_task(task_id.clone(), worker.clone());

    //     // 提交任务结果
    //     testing_env!(get_context(worker.clone()));
    //     contract.submit_task_result(task_id.clone(), "Result".to_string());

    //     // 选择验证者
    //     testing_env!(get_context(broadcaster.clone()));
    //     contract.select_verifiers(task_id.clone());

    //     // 提交共识证明
    //     testing_env!(get_context(committee_leader.clone()));
    //     let proof = ConsensusQosProof {
    //         task_id: task_id.clone(),
    //         committee_leader: committee_leader.clone(),
    //         timestamp: 0,
    //         status: QosProofStatus::Approved,
    //         signatures: vec![],
    //     };

    //     let result = contract.submit_consensus_proof(proof, QosProofStatus::Approved);
    //     assert!(result);

    //     // 验证任务状态
    //     let task = contract.tasks.get(&task_id).unwrap();
    //     assert_eq!(task.status, TaskStatus::Verified);

    //     // 验证共识证明
    //     let stored_proof = contract.consensus_proofs.get(&task_id).unwrap();
    //     assert_eq!(stored_proof.status, QosProofStatus::Approved);
    // }

    // // 测试获取所有共识证明
    // #[test]
    // fn test_get_all_consensus_proofs() {
    //     let (mut contract, _) = setup_contract();
    //     let broadcaster = accounts(1);
    //     let worker = accounts(2);
    //     let committee_leader = accounts(3);

    //     // 添加委员会成员
    //     testing_env!(get_context(accounts(0)));
    //     contract.add_committee_member(committee_leader.clone(), true);

    //     // 提交多个共识证明
    //     for i in 0..3 {
    //         // 设置任务
    //         testing_env!(get_context(broadcaster.clone()));
    //         let task_id = format!("task{}", i);
    //         contract.create_task(format!("Task {}", i), "Description".to_string());
    //         contract.assign_task(task_id.clone(), worker.clone());

    //         testing_env!(get_context(worker.clone()));
    //         contract.submit_task_result(task_id.clone(), "Result".to_string());

    //         testing_env!(get_context(broadcaster.clone()));
    //         contract.select_verifiers(task_id.clone());

    //         // 提交共识证明
    //         testing_env!(get_context(committee_leader.clone()));
    //         let proof = ConsensusQosProof {
    //             task_id: task_id.clone(),
    //             committee_leader: committee_leader.clone(),
    //             timestamp: 0,
    //             status: QosProofStatus::Approved,
    //             signatures: vec![],
    //         };

    //         contract.submit_consensus_proof(proof, QosProofStatus::Approved);
    //     }

    //     // 获取所有共识证明
    //     let proofs = contract.get_all_consensus_proofs(None, None);
    //     assert_eq!(proofs.len(), 3);

    //     // 测试分页
    //     let proofs = contract.get_all_consensus_proofs(Some(1), Some(1));
    //     assert_eq!(proofs.len(), 1);
    // }
}
