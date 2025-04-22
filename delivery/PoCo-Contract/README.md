# Blockchain-based QoS-aware Media Transcoding Contract

## Architecture
The contract contains these key components:

1. **Contract State**: Maintains all data structures and storage for the transcoding service.
2. **Task Management**: Handles task lifecycle from creation through verification.
3. **Worker System**: Manages worker registration, performance tracking, and QoS metrics.
4. **Consensus System**: Handles verification and consensus through committee and verifier members.

## Workflow

The contract facilitates a comprehensive workflow for decentralized media transcoding:

1. **Task Publication Phase**
   - Broadcaster publishes transcoding task with requirements
   - Task enters offer collection state
   - Workers submit offers during the collection period

2. **Task Assignment Phase**
   - After offer timeout, the contract selects the best worker based on QoS scores
   - Worker is notified and begins transcoding the media
   - Alternative: Tasks can enter queue for later assignment

3. **Task Completion Phase**
   - Worker completes transcoding and uploads result
   - Worker submits keyframe timestamps and results
   - Contract selects GOP samples for quality verification

4. **Verification Phase**
   - Contract selects multiple verifiers for the task
   - Verifiers perform quality assessment on selected GOP samples
   - Verifiers submit QoS proofs with various quality metrics

5. **Consensus Phase**
   - Committee leader aggregates verification results
   - In case of conflicts, supplementary verification is requested
   - Final consensus proof is submitted to the blockchain

6. **QoS Update Phase**
   - Worker's performance metrics are updated based on consensus results
   - Three-dimensional QoS scoring affects future task assignments
   - Historical performance data is maintained for workers

## Implementation Details

### Task Data Structure

Tasks track the complete lifecycle of a transcoding job:

```rust
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
    pub publish_time: u64,
    pub hw_acceleration_preferred: bool,
    pub keyframe_timestamps: Option<Vec<String>>,
    pub selected_gops: Option<Vec<String>>,
    pub video_duration: Option<f64>,
    pub frame_count: Option<u32>,
}
```
### QoS Scoring System

QoS scoring uses a three-dimensional approach:

1. **Service Reliability (SR)**: 40% weight
   * Success completion rate
   * Task participation
   * Service quality scores

2. **Time Stability (TS)**: 20% weight
   * Average daily task throughput
   * Service coverage over time

3. **Performance Performance (PP)**: 40% weight
   * Encoding speed (FPS)
   * Performance stability

QoS Score = 0.4 × SR + 0.2 × TS + 0.4 × PP

### GOP-based Verification

The contract implements a Group of Pictures (GOP) sampling mechanism:

* When workers complete tasks, they submit all keyframe timestamps
* Contract selects specific GOPs for verification (typically 3 samples)
* Verifiers check only these GOPs to reduce computational load
* Verification focuses on video quality (VMAF), audio quality (PESQ), and synchronization

### Conflict Resolution

When verifiers disagree on quality metrics:

1. Committee leader detects conflicts during consensus
2. Supplementary verifier is requested from the pool
3. Statistical or majority-based conflict resolution is applied
4. Final consensus is reached with resolved data

### Worker Selection Algorithm

Tasks are assigned to workers using a quality-oriented approach:

```rust
fn select_best_offer(&mut self, task_id: &String) -> bool {
    // Get all offers
    let offers = self.task_offers.get(task_id).unwrap();
    
    // Calculate weighted QoS scores considering:
    // - Service reliability (40%)
    // - Time stability (20%)  
    // - Performance metrics (40%)
    // - Hardware acceleration preference (bonus)
    
    // Select worker with highest score
    // ...
}
```

### Usage
#### For Broadcasters

```rust
// Publish a new transcoding task
let task_id = contract.publish_task(
    "ipfs://QmSourceVideoHash",
    TranscodingRequirement {
        target_codec: "h264",
        target_resolution: "1920x1080",
        target_bitrate: "5000",
        target_framerate: "30",
        additional_params: "--preset medium",
    },
    true  // Prefer hardware acceleration
);

// Check task status
let task = contract.get_task(task_id);

// Check offers timeout 
contract.check_offer_timeout(task_id);

// Get broadcaster's tasks
let my_tasks = contract.get_broadcaster_tasks(broadcaster_account_id);

```


#### For Workers
```rust
// Register as a worker
contract.register_worker(true);  // true = supports hardware acceleration

// Send heartbeat to maintain availability
let status = contract.worker_heartbeat();

// Submit offer for a task
contract.submit_offer(task_id);

// Complete a task
contract.complete_task(
    task_id,
    "ipfs://QmResultVideoHash",
    vec!["0.00", "2.50", "5.75", "8.20"],  // Keyframe timestamps
    180.5  // Video duration in seconds
);

// Get assigned tasks
let tasks = contract.get_tasks_for_worker(None);  // None = use caller's account
```

#### For verifiers

```rust
// Query assigned verification tasks
let tasks = contract.query_assigned_tasks(verifier_id, true);  // true = only unverified

// Submit verification proof
contract.submit_verifier_proof(VerifierQosProof {
    id: "",  // Empty = auto-generated
    task_id: task_id,
    verifier_id: verifier_id,
    timestamp: env::block_timestamp(),
    video_specs: video_specs,
    video_score: 92.5,
    gop_scores: gop_scores,
    audio_score: Some(4.2),
    sync_score: Some(5.0),
    signature: signature,
});
```

#### For Committee members

```rust
// Submit consensus proof (leader only)
contract.submit_consensus_proof(ConsensusQosProof {
    task_id: task_id,
    worker_id: worker_id,
    committee_leader: leader_id,
    video_score: 92.5,
    audio_score: 4.2,
    sync_score: 0.0,
    // Additional fields...
});

// Request supplementary verifier for conflict resolution
contract.request_supplemental_verifier(task_id);

// Get task consensus details
let details = contract.get_task_consensus_details(task_id);
```
