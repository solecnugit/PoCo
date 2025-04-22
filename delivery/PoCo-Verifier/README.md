# POCO Verifier Module

## Architecture

The POCO Verifier Module implements a distributed verification node that validates media transcoding quality in a blockchain-based transcoding system. It evaluates the quality of service (QoS) for transcoded media and provides trusted verification results to the network.

                ┌───────────────────┐
                │  Verifier Module  │
                └─────────┬─────────┘
                          │
              ┌───────────┴────────────┐
              │                        │
    ┌─────────▼──────────┐   ┌─────────▼──────────┐
    │   Task Management  │   │  Quality Evaluation │
    │                    │   │                     │
    │ - Task Discovery   │   │ - GOP Analysis      │
    │ - Queue Management │   │ - VMAF Calculation  │
    │ - Status Tracking  │   │ - Proof Generation  │
    └─────────┬──────────┘   └─────────┬──────────┘
              │                        │
    ┌─────────▼──────────┐   ┌─────────▼──────────┐
    │  NEAR Connection   │   │    IPFS Service    │
    │                    │   │                    │
    │ - Contract Calls   │   │ - File Retrieval   │
    │ - Proof Submission │   │ - Temp Management  │
    └─────────┬──────────┘   └─────────┬──────────┘
              │                        │
              └──────────┐ ┌───────────┘
                         │ │
               ┌─────────▼─▼────────┐
               │  Committee Client  │
               │                    │
               │ - Consensus        │
               │ - Proof Submission │
               │ - Verification     │
               └────────────────────┘

The verification system consists of the following core components:

1. **VerifierService**: Central orchestration component that coordinates the verification workflow
2. **NearConnection**: Handles all blockchain interactions with the verification contract
3. **IPFS Service**: Manages media file downloads from IPFS for verification
4. **GopAnalyzer**: Analyzes and extracts GOP (Group of Pictures) from media files
5. **MediaQualityEvaluator**: Evaluates transcoding quality metrics (VMAF, audio, sync)
6. **CommitteeClient**: Communicates with Committee nodes for consensus on verification results

The verification process follows a GOP-based sampling approach to efficiently evaluate media quality, significantly reducing computational overhead while maintaining accuracy.

## Workflow

The POCO Verifier follows a specific workflow to participate in the decentralized transcoding verification network:

1. **Initialization Phase**
   - Connect to NEAR blockchain network
   - Authenticate verifier account
   - Connect to IPFS for file retrieval
   - Initialize GOP analysis and quality evaluation components
   - Establish communication with Committee nodes

2. **Task Discovery Phase**
   - Poll blockchain for assigned verification tasks
   - Queue tasks for sequential processing
   - Track task processing status

3. **Verification Phase**
   - Download source and transcoded media from IPFS
   - Split videos into GOP segments for sampling-based analysis
   - Evaluate video quality using VMAF metrics on sampled GOPs
   - Calculate video quality, audio quality, and sync scores
   - Generate cryptographic proof of evaluation results

4. **Result Submission Phase**
   - Submit verification results to blockchain
   - Send quality proof to Committee nodes for consensus
   - Respond to Committee requests for supplementary verification if needed
   - Clean up temporary files and resources

5. **Status Management Phase**
   - Track verification task progress
   - Manage task queue and processing state
   - Handle graceful shutdown for in-progress tasks

## Implementation Details

### VerifierService Component

The VerifierService acts as the central coordinator for the verification process:

```typescript
class VerifierService {
  private config: VerifierConfig;
  private nearConnection: NearConnection;
  private ipfsService: IpfsService;
  private committeeClient: CommitteeClient;
  private gopAnalyzer: GopAnalyzer;
  private mediaEvaluator: MediaQualityEvaluator;

  // Task processing state
  private isProcessingTask: boolean = false;
  private taskQueue: TaskData[] = [];

  // Main verification method
  async verifyTask(task: TaskData): Promise<boolean> {
    // Download media files from IPFS
    // Split videos into GOPs
    // Evaluate quality metrics for selected GOPs
    // Generate verification proof
    // Submit results to blockchain and committee
    // Clean up resources
  }
}
```


### GOP-based Quality Assessment

The system implements an efficient GOP-based sampling approach for quality assessment:

```typescript
// GOP extraction and analysis
class GopAnalyzer {
  async splitVideoIntoGops(
    videoPath: string,
    timestamps: string[]
  ): Promise<Map<string, string>> {
    // Use FFmpeg to split video into GOP segments
    // Create mapping between timestamps and GOP files
    // Return GOP map for quality evaluation
  }
}

// Quality evaluation
class MediaQualityEvaluator {
  async calculateVmafScore(
    referenceGopPath: string,
    distortedGopPath: string
  ): Promise<number> {
    // Use FFmpeg with VMAF filter to calculate quality score
    // Return normalized score for blockchain submission
  }
}
```


### Distributed Consensus Mechanism
The system integrates with a Committee-based consensus mechanism:
```typescript
class CommitteeClient {
  // Submit verification results to committee
  async sendProofToCommittee(proof: VerifierQosProof): Promise<boolean> {
    // Send proof to committee leader with priority
    // Fall back to all members if leader unavailable
    // Implement retry mechanism for reliability
  }

  // Handle supplementary verification requests
  async sendSupplementaryProof(
    taskId: string,
    proof: VerifierQosProof
  ): Promise<boolean> {
    // Send additional verification data when requested
    // Support verification conflict resolution
  }
}
```


### Blockchain Integration
The NearConnection component handles all blockchain interactions:
```typescript
class NearConnection {
  // Query assigned verification tasks
  async queryAssignedTasks(
    onlyUnverified: boolean = true
  ): Promise<TaskData[]> {
    // Get tasks assigned to this verifier
    // Filter by verification status if needed
  }

  // Submit verification results
  async submitVerifierProof(proof: VerifierQosProof): Promise<boolean> {
    // Submit quality proof to blockchain
    // Include GOP scores and overall metrics
  }
}
```

### Usage Configuration

Create a config.json file with the following structure:
```json
{
  "verifierAccountId": "verifier.testnet",
  "contractId": "transcoding-contract.testnet",
  "nearConfig": {
    "networkId": "testnet",
    "nodeUrl": "https://rpc.testnet.near.org"
  },
  "ipfsConfig": {
    "protocol": "http",
    "host": "ipfshost",
    "port": 5001
  },
  "ffmpegPath": "/usr/bin",
  "pollingInterval": 30000,
  "credentialsPath": "~/.near-credentials"
}
```

```bash
# Running the Verifier
Start the verifier with:
npm run start
For verbose logging:
npm run start -- --verbose
```
The verifier will:

1. Connect to blockchain and initialize all components
2. Poll for assigned verification tasks
3. Process tasks sequentially
4. Report results to blockchain and committee
5. Output periodic status reports