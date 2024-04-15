## 此模块是什么？在PoCo中的作用?

此模块为**链下运行**的PBFT共识模块，作为保证链上输入数据正确性的Oracle。模块会对工作节点的任务完成结果(媒体转码)的任务执行时间，和任务完成质量（有参考的视频质量评估和音频质量评估）进行共识。

## 此模块的实现？输入？输出？

此模块的基本实现依赖于 `ws` 库，通过 websocket 建立 p2p网络。

模块的输入来自于`工作节点请求`和`链上的智能合约`。

## PBFT流程

`NEW ROUND`: Proposer to send new block proposal. Validators wait for PRE-PREPARE message.

`PRE-PREPARED`: A validator has received PRE-PREPARE message and broadcasts PREPARE message. Then it waits for 2F + 1 of PREPARE or COMMIT messages.

`PREPARED`: A validator has received 2F + 1 of PREPARE messages and broadcasts COMMITmessages. Then it waits for 2F + 1 of COMMIT messages.

`COMMITTED`: A validator has received 2F + 1 of COMMIT messages and is able to insert the proposed block into the blockchain.

`FINAL COMMITTED`: A new block is successfully inserted into the blockchain and the validator is ready for the next round.

`ROUND CHANGE`: A validator is waiting for 2F + 1 of ROUND CHANGE messages on the same proposed round number.

## 此模块和智能合约的交互？

to be continued..
