## 此模块是什么？在PoCo中的作用?

此模块为**链下运行**的PBFT共识模块，作为保证链上输入数据正确性的Oracle。模块会对工作节点的任务完成结果(媒体转码)的任务执行时间，和任务完成质量（有参考的视频质量评估和音频质量评估）进行共识。

此模块通过构建一个去中心化的oracle网络以降低潜在的欺诈和故障的风险，同时，依赖于PBFT这样的共识机制的网络能够确保收集到的数据在一定程度上经过了多方验证并且被认为是可靠的。

在这个场景中，Committee 作为实现上独立于 NEAR 区块链平台的组件，并不直接依赖于 NEAR 的特定技术或服务，它主要负责链接现实世界和区块链世界。Committee 的主要职责是：

1. 收集和处理现实世界数据：即使 Committee 不是 NEAR 的一部分，正确地执行其数据收集和验证职责。

2. 将验证过的数据提交给区块链：通过智能合约进行，合约将接收并利用 Oracle 提供的数据，如评估工作节点提供的转码服务的质量和时间。

This module is a PBFT consensus module for off-chain execution, serving as an Oracle to ensure the correctness of input data on the blockchain. The module achieves consensus on the task execution time and task completion quality (video and audio quality evaluation with references) of the working nodes' job (media transcoding).

By building a decentralized oracle network, this module reduces the risks of potential fraud and failure. Moreover, relying on consensus mechanisms like PBFT, the network can ensure that the collected data has been verified by multiple parties and is considered reliable to a certain extent.

In this scenario, the Committee functions as a component independent of the NEAR blockchain platform, not relying directly on specific NEAR technologies or services. Its main responsibility is to:

1. Collect and process real-world data: Even though the Committee is not part of NEAR, it correctly fulfills its data collection and verification responsibilities.

2. Submit validated data to the blockchain: This is done through smart contracts, where the contract receives and utilizes the data provided by the Oracle, such as evaluating the quality and time of transcoding services provided by the working nodes.

## 此模块的实现？输入？输出？

此模块的基本实现依赖于 `ws` 库，通过 websocket 建立 p2p网络。

模块的输入来自于`工作节点请求`和`链上的智能合约`。（wait for finished: picture）

每当积累的transaction数量达到 “TRANSACTION_THRESHOLD” 时，将会触发PBFT共识过程，产出BLOCK。 (wait for finished: picture)

## PBFT流程

`NEW ROUND`: Proposer to send new block proposal. Validators wait for PRE-PREPARE message.

`PRE-PREPARED`: A validator has received PRE-PREPARE message and broadcasts PREPARE message. Then it waits for 2F + 1 of PREPARE or COMMIT messages.

`PREPARED`: A validator has received 2F + 1 of PREPARE messages and broadcasts COMMITmessages. Then it waits for 2F + 1 of COMMIT messages.

`COMMITTED`: A validator has received 2F + 1 of COMMIT messages and is able to insert the proposed block into the blockchain.

`FINAL COMMITTED`: A new block is successfully inserted into the blockchain and the validator is ready for the next round.

`ROUND CHANGE`: A validator is waiting for 2F + 1 of ROUND CHANGE messages on the same proposed round number.

## 此模块和智能合约的交互？

wait for finished
