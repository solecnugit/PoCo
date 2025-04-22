# PoCo-Committee

## Overview

PoCo-Committee is a critical component of the blockchain-based QoS-aware media transcoding system, responsible for establishing consensus on service quality assessments. It implements a modified PBFT (Practical Byzantine Fault Tolerance) consensus mechanism with domain-specific verification methods tailored for media quality assessment.

## Key Features

- **Domain-Specific PBFT Implementation**: Combines traditional PBFT with media-specific verification techniques to achieve reliable consensus even with potentially malicious nodes
- **Multi-stage Verification Process**: Implements a two-tier verification strategy with quick validation and deep assessment phases
- **Conflict Resolution**: Robust mechanism to resolve disagreements between quality assessments
- **On-chain Result Recording**: Records consensus outcomes on the blockchain for transparent quality tracking

## Usage

### Starting a Committee Node

**Configure environment variables for the node**
```bash
export NODE_ID=committee-1
export PORT=8000
export IS_LEADER=true
export PEERS="committee-2:host2:port2,committee-3:host3:port3"
export TOTAL_NODES=4

# Start the node
npx ts-node start-node.ts
```


## Architecture

The Committee system consists of the following key components:

- CommitteeNode: Central component that orchestrates the consensus process
- PBFTEngine: Implements the PBFT consensus algorithm
- QoSValidator: Validates QoS proofs from verifiers
- MessageHandler: Manages WebSocket-based peer-to-peer communication
- ApiServer: Provides HTTP API for external interactions
- NearConnectionLeader: Interfaces with the NEAR blockchain (for leader nodes)

## Workflow
The consensus process follows these stages:

### 1. QoS Proof Collection:

Verifier nodes submit QoS proofs to Committee nodes via API
Each committee node stores and performs quick validation


### 2. Deep Validation:

When enough proofs are collected (minimum 2), deep validation occurs
The system checks for consistency across video quality, audio quality, and synchronization


### 3. PBFT Consensus:

Leader initiates consensus with PrePrepare message containing validated proof
Followers respond with Prepare messages after validation
When enough Prepare messages are received, nodes send Commit messages
Consensus is reached when enough Commit messages are collected


### 3.1 Conflict Resolution:

If conflicts are detected, a supplementary verification process begins
Supplementary proofs are used to resolve conflicts through majority voting


### 4. Blockchain Integration:

After consensus, leader commits results to NEAR blockchain
Results include comprehensive QoS metrics (video quality, audio quality, sync)

## Implementation Details

### Committee Node

The Committee Node handles QoS proof collection, validation, and consensus. It can operate in leader or follower mode:

```typescript
// Create a Committee node
const committeeNode = new CommitteeNode(
  nodeId,           // Unique identifier for this node
  port,             // WebSocket port for peer communication
  isLeader,         // Whether this node is the leader
  peers,            // Array of peer connection strings
  totalNodes        // Total number of committee nodes
);

// Start the node
committeeNode.start();
```

### PBFT Consensus
The PBFT consensus process follows these phases:

1. **PrePrepare**: Leader broadcasts proof to all followers
2. **Prepare**: Followers validate and broadcast agreement
3. **Commit**: Nodes commit when sufficient agreements received

The algorithm ensures Byzantine fault tolerance, handling up to f faulty nodes in a system with 3f+1 total nodes.

### QoS Validation

QoS validation occurs in two phases:

1. **Quick Validation**: Basic validation of proof structure and data integrity
2. **Deep Validation**: Cross-validation between multiple proofs to ensure consistency

Validation covers:
- Video quality (VMAF scores)
- Audio quality (PESQ scores)
- Audio-video synchronization
- GOP (Group of Pictures) sample verification
- Encoding performance

### Supplementary Verification

When conflicts are detected:

1. Leader requests supplementary verification
2. Additional verifier provides independent assessment
3. Statistical or majority-based conflict resolution
4. Final consensus with resolved data

### NEAR Blockchain Integration

Results are stored on NEAR blockchain using a smart contract interface:

```typescript
// For leader nodes
await nearConnection.submitConsensusProof({
  task_id: taskId,
  worker_id: workerId,
  committee_leader: nodeId,
  video_score: videoScore,
  audio_score: audioScore,
  sync_score: syncScore,
  // Additional QoS metrics...
});
```

### API Endpoints
The Committee node exposes these main API endpoints:
```
GET /health: Basic health check
GET /status: Detailed node status
POST /proof: Submit QoS proof for consensus
POST /proofs/batch: Submit multiple QoS proofs
POST /proof/:taskId/supplementary: Submit supplementary proof
GET /proof/:taskId/status: Get task processing status
```

### Monitoring Consensus
```bash
# Check status of a specific task
curl http://localhost:9000/proof/task-123/status

# Response example
{
  "taskId": "task-123",
  "state": "finalized",
  "proofCount": 3,
  "verifierIds": ["verifier-1", "verifier-2", "verifier-3"],
  "result": {
    "consensusTimestamp": "2025-04-22T10:15:30.123Z",
    "videoScore": 92.75,
    "audioScore": 4.2,
    "syncScore": 0.98
  }
}
```

### Configuration
```typescript
# Configure the system through the committee-config.ts file:
const config: CommitteeConfig = {
  nearConfig: {
    networkId: 'testnet',
    nodeUrl: 'https://rpc.testnet.near.org',
    // Other NEAR settings...
  },
  contractId: 'pococontract11.testnet',
  leaderAccountId: 'pocoleader.testnet',
  credentialsPath: '~/.near-credentials',
  // Other configuration parameters...
};
```
This system provides robust, Byzantine fault-tolerant consensus for QoS verification in decentralized media transcoding services.