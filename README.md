## 项目名称：基于 Web3 的 QoS 感知的视频转码系统

## Project Name: QoS-aware Video Transcoding System based on Web3

## 项目概览

去中心化媒体转码服务的目标是利用全球广布的闲置计算资源，为客户构建一个高效、性价比高且以服务质量为核心的视频转码网络。通过提供完全可以追溯和透明的服务记录及评估机制，本项目确立了服务网络的自治性。我们致力于为拥有闲置计算资源的提供者创造收益的同时，为客户带来高质量、更经济的转码服务。

### 主要特点

- **可追溯性**：我们确保每一项转码服务的结果和质量都有详尽的记录。这种做法不仅增强了信任感，还方便了对服务提供商的质量监控，同时提升了系统的安全性，能有效识别并防范恶意行为或攻击。
- **透明性**：我们的服务质量评估方法和任务调度过程都是公开透明的，确保了节点间的公平竞争，并鼓励服务提供商提升其服务水平。通过透明性原则，我们能够有效地筛选出表现不佳的节点，从而最大程度上提升服务质量。
- **自治性**：在我们的服务网络中，每个节点都根据已设定的标准自主提供服务。依据可追溯的记录来评估节点的服务质量，并通过共识机制来确认，此举旨在避免因中心化带来的单点故障，并减少因单一节点出于自利行为可能引发的不公正任务调度。

## Project Overview

The Decentralized Media Transcoding Service aims to capitalize on the wealth of unused computational resources around the world, crafting an efficient, cost-effective video transcoding network that places a premium on service quality. By offering fully traceable and transparent service records and assessment mechanisms, our project solidifies the network's autonomy. We are dedicated to generating income for providers of idle computing resources while delivering high-quality, more economical transcoding services to our customers.

### Key Features

- Traceability: Our system diligently records the outcomes and quality of each transcoding service, enhancing trust and enabling effective quality monitoring of service providers. This also bolsters security measures against malicious activities or attacks.
- Transparency: We guarantee an open and transparent assessment of service quality and scheduling processes. This ensures subpar nodes are filtered out while fostering fair competition, coaxing providers to elevate their service standards.
- Autonomy: Nodes within our service network autonomously render services adhering to established agreements. They undergo evaluations based on traceable task records and collectively reach consensus on quality assessments. This approach mitigates the risks of a single point of failure and unfair task distribution due to any autonomous entity acting in self-interest.

## 关键进展

在过去一年，结合我们预先制定的目标和实际遇到的问题，我们取得了以下关键进展：

- ### 成功将智能合约从以太坊迁移至 NEAR。
  - NEAR 提供了低成本和高效的服务，这与我们利用去中心化方案满足实际转码服务需求方案的使命紧密相符。NEAR 独特的分片技术显著提高了吞吐量，确保我们在扩展服务的同时，不会牺牲速度或可靠性。此外，NEAR 还提供了用户友好的体验，具有简洁的账户模型和低交易费用——这对于最大化吸引节点提供者和服务消费者的参与至关重要。
  - 它的实现在 lib/packages/poco-contract-v2 中
- ### 开发并实现转码领域特定 QoS 的评估框架。
  - 我们为非实时场景下的转码服务定制了领域特定的服务质量标准。具体而言，我们从两个维度评估服务提供商的媒体转码质量：一是对比转码后视频与源视频在视觉上的质量差异，以体现人眼可感知的视频质量变化；二是通过计算服务提供商在单位时间内转码的帧数，以此来衡量其服务的处理速度。这两个指标共同构成了我们评估媒体转码服务质量的标准。
  - 它的实现在 lib/packages/poco-service 中
- ### 开发并实现一个区块链原型作为委员会实现对服务提供商服务质量的共识。
  - 我们开发并实现了一个区块链原型，用以通过委员会机制，在特定服务场景中实现针对服务提供商服务质量的共识。鉴于此场景下闲置计算资源的非固定性，大量不稳定节点所进行的共识可能会耗费宝贵的计算力并浪费时间，这与我们项目的初衷——充分利用分散计算资源——背道而驰。因此，我们挑选了表现优异的节点组成委员会(Committee)，这个委员会负责完成服务质量(QoS)的共识并将其上链。其他工作节点则将链上信息同步到自己的本地副本中，委员会本身将周期性重建并且重新选择节点加入其中。这一策略有效地提升了共识性能，同时兼顾了资源使用的高效性。
  - 它的实现在 lib/packages/poco-committee 中

## Key Achievements

In the past year, aligning with our pre-established objectives and tackling the challenges encountered, we have made the following key progress:

- ### Successfully migrated our smart contract from Ethereum to NEAR.
  - NEAR offers low-cost and efficient services, which closely align with our mission to meet real-world transcoding service needs with a decentralized approach. NEAR's distinctive sharding technology significantly increases throughput, ensuring that our service expansion does not compromise speed or reliability. Moreover, NEAR provides a user-friendly experience with a streamlined account model and low transaction costs—critical for maximizing the involvement of node providers and service consumers.
  - The implementation can be found in lib/packages/poco-contract-v2.
- ### Developed and implemented a domain-specific QoS assessment framework for transcoding.
  - We crafted service quality standards tailored to non-real-time transcoding scenarios. Specifically, we evaluate the media transcoding quality from service providers on two fronts: one, by comparing the visual quality difference between the post-transcoded video and the source video, reflecting perceptible changes in video quality to the human eye; and two, by calculating the number of frames transcoded by the service provider per unit of time, thereby gauging the efficiency of their service delivery. These two metrics together form our standards for assessing media transcoding service quality.
  - The implementation can be found in lib/packages/poco-service.
- ### Developed and implemented a blockchain prototype to facilitate consensus on service provider quality via a committee.
  - We have developed and deployed a blockchain prototype that employs a committee mechanism to achieve consensus on service provider quality in specific service scenarios. Given the inherent variability of idle computing resources in this context, consensus amongst a large number of unstable nodes could consume valuable computational resources and time. This runs counter to our project's original intent to fully leverage distributed computing resources. Hence, we selected a group of top-performing nodes to form a Committee, tasked with reaching a consensus on the Quality of Service (QoS) and recording it on the chain. Other working nodes synchronize chain information to their local copies, a strategy that effectively boosts consensus performance while maintaining efficient use of resources.
  - The implementation can be found in lib/packages/poco-committee.

## 模块变动简介

随着我们项目的发展，各个模块都进行了显著更新，以改善功能和性能。下面是关键模块及其在我们服务中角色的概览：

### `poco-contract-v2`

- 目的：作为基于 NEAR 协议构建的智能合约，本模块设定了广播商和服务提供商，委员会之间的所有链上的工作流程，并详细记录了所有媒体转码任务及服务提供商的服务质量表现。该模块确保了体系中所有记录的可追溯性，并利用智能合约实施控制节点在链上的行为。

- 主要更新：
  - 将原本部署在以太坊上的智能合约迁移到 NEAR 平台，目的是提高系统的吞吐能力与成本效率，完成了从 `poco-contract` 到 `poco-contract-v2` 的重大升级。
- Changelog：

  - 首次引入：2023.02
  - 变更记录：
    - 2023.4 适配 NEAR-SDK V4.0.0 版本变化和相应库的更新。
    - 2023.11 适配 NEAR-SDK V4.1.0 版本变化和相应库的更新。
    - 2023.12 处理和 poco-service merge 时遇到的问题。
    - 2024.3 适配 NEAR-SDK V5.0.0 版本变化和相应库的更新。

### `poco-service`

- 目的：本模块是服务提供商执行任务的核心环节，它封装了服务提供商在链下执行的媒体转码逻辑。同时，本模块也包含验证服务质量所需的领域特定算法逻辑。

- 主要更新：

  - 基于原 `poco-codec` 模块的功能，整合并迁移了 QoS 评估模块的相关代码，实现更为统一和高效的服务运作。

- Changelog：
  - 首次引入：2023.05
  - 变更记录：
    - 2023.5 探索视频质量评估算法 SSIM PSNR VMAF。
    - 2023.6 尝试基于 GOP（Group of picture）优化 VMAF。
    - 2023.7 寻找 benchmark 验证优化算法结果。
    - 2023.9 设计多种场景下（延时敏感，直播，空间敏感）三种场景下的 QoS 采集方式。（后搁置）
    - 2023.12 处理和 `poco-contract-v2` merge 时遇到的问题。

### `poco-committee`

- 目的：该模块是实现对服务提供商服务质量共识的核心组件。共识过程在链下完成，其中共识算法基于 PBFT（实用拜占庭容错）机制。该模块负责接收来自多个验证节点的服务质量数据，并通过结合领域特定的验证方法与传统的加密校验算法，达成对服务提供商服务质量的共识。

- 主要更新：

  - 设计并实现一个简单的使用拜占庭容错算法。

- Changelog：
  - 首次引入：2024.03
  - 变更记录：
    - 【进行中】2024.04 处理和 `poco-service` 交互时遇到的问题。
    - 2024.04 处理和 `poco-agent` 交互时遇到的问题。

### `poco-agent`

- 目的：本模块充当用户间及用户与智能合约交互的接口，提供了一个与前端无缝连接的中间件。它封装了实现链上及链下多种操作的（前文提及的多种）模块，并统一了触发不同交互逻辑的接口，极大地优化了用户体验，便于用户轻松地进行各种操作。

- 主要更新：

  - 通过对旧版 `poco-client` 的全面重构，增强了模块的可用性和性能。

- Changelog:

  - 首次引入：2023.03
  - 变更记录：
    - 2023.3-至今 处理和其他模块的交互以及 NEAR 平台的更新。

### `poco-types`

- 目的：本模块为整个系统中的多个组件提供统一的数据结构、类型定义以及工具函数。它作为基础库，确保了系统内部各模块之间的数据一致性和接口标准化，从而降低了模块间的耦合度，并提高了整体开发效率。

- 主要更新：

  - 更新了工具函数库，以支持更复杂的数据操作和处理，满足系统演进的需要。

- Changelog:
  - 首次引入：2022.10
  - 变更记录：
    - 2022.10-至今 适配实际需要。

## Module Changes Summary

As our project has evolved, significant updates have been made to various modules to improve functionality and performance. Below is an overview of key modules and their roles within our service:

### `poco-contract-v2`

- Purpose: Serving as a smart contract built on the NEAR protocol, this module sets the workflows on-chain for broadcasters, service providers, and the committee, detailing all media transcoding tasks and service providers' performance. It ensures the traceability of all records in the system and leverages smart contracts to control node behaviors on the blockchain.

- Main Updates:

  - Migrated the smart contract from Ethereum to NEAR to enhance system throughput and cost efficiency, marking a significant upgrade from `poco-contract` to `poco-contract-v2` .

- Changelog:

  - Introduced: 2023.02
  - Changes:

    - 2023.4 Adjusted for NEAR-SDK V4.0.0 updates and corresponding library updates.
    - 2023.11 Adapted to NEAR-SDK V4.1.0 alterations and related library updates.
    - 2023.12 Addressed issues encountered during merging with poco-service.
    - 2024.3 Adjusted for NEAR-SDK V5.0.0 changes and corresponding library updates.

### `poco-service`

- Purpose: This module is the core component where service providers execute tasks, encapsulating the logic for media transcoding off-chain. It also includes domain-specific algorithms needed for assessing service quality.

- Main Updates:

  - Integrated and migrated code related to QoS assessment from the original poco-codec module for more unified and efficient service operation.

- Changelog:

  - Introduced: 2023.05
  - Changes:
    - 2023.5 Explored video quality assessment algorithms like SSIM, PSNR, VMAF.
    - 2023.6 Tested optimizations based on GOP (Group of picture) for VMAF.
    - 2023.7 Sought benchmarks to validate optimization algorithm outcomes.
    - 2023.9 Designed QoS collection methods for various scenarios (latency-sensitive, live streaming, space-sensitive) (later shelved).
    - 2023.12 Addressed issues encountered during merging with `poco-contract-v2` .

### `poco-committee`

- Purpose: This module is the core component for achieving consensus on service provider quality. The consensus process occurs off-chain, based on the PBFT (Practical Byzantine Fault Tolerance) mechanism. It receives service quality data from multiple validation nodes and achieves consensus on service quality using domain-specific validation methods and traditional cryptographic verification algorithms.
- Main Updates:
  - Designed and implemented a simple Byzantine fault tolerance algorithm.
- Changelog:

  - Introduced: 2024.03
  - Changes:
    - 【In Progress】2024.04 Addressing issues encountered during interaction with poco-service.
    - 2024.04 Addressing issues when interacting with poco-agent.

  ### `poco-agent`

- Purpose: This module acts as an interface between users and between users and smart contracts, providing middleware that seamlessly connects with the frontend. It encapsulates various (previously mentioned) module operations both on-chain and off-chain and unifies interfaces for triggering different interaction logics, greatly improving user experience and facilitating easy operations.
- Main Updates:

  - Enhanced module usability and performance through a comprehensive overhaul of the old poco-client.

- Changelog:

  - Introduced: 2023.03
  - Changes:
    - 2023.3-Present Addressing interactions with other modules and updates on the NEAR platform.

  ### `poco-types`

- Purpose: This module provides unified data structures, type definitions, and utility functions for multiple components within the system. Serving as a foundational library, it ensures data consistency and interface standardization across modules, thereby reducing coupling between modules and enhancing overall development efficiency.

- Main Updates:

  - Updated the utility library to support more complex data operations and processing to meet the evolving needs of the system.

- Changelog:
  - Introduced: 2022.10
  - Changes:
    - 2022.10-Present Adapted as needed.

## 完整任务流程

完整任务流程从委员会的组建开始，涵盖了任务发布、调度、执行及自我评估，最终通过质量评估与共识形成服务质量的评定，进而执行奖励分配和记录同步。

### 委员会组建

1. 选择优质服务提供商：在每一轮开始之前，智能合约( `poco-contract-v2` )根据链上记录的服务质量信息，选择出若干个长期提供优质服务质量的服务提供商组成委员会。

2. 委员会内部协作：委员会成员根据预先定义的链下交互方式彼此联系，组成委员会，并将在委员会内部对服务提供商的服务质量进行共识。

### 任务发布

1. 服务请求提交：服务请求者通过 `poco-agent` 将媒体转码任务提交到智能合约( `poco-contract-v2` )，明确转码需求及链下交互方式。

### 任务调度

1. 任务监听：服务提供商通过 `poco-agent` 监听智能合约的任务发布事件，根据当前的工作负载和能力评估是否承担新任务。

2. 链下协商：选择接受任务的服务提供商将通过任务发布者预定义的链下交互方式与发布者沟通，确保对任务细节的共识。

3. 服务提供商选择：服务请求者基于服务提供商的服务质量记录，自主选择合适的服务提供商执行任务，优先保证高质量的服务交付。

### 任务执行与自我评估

7. 独立完成任务：选定的服务提供商根据任务需求在链下独立完成媒体转码任务。

8. 质量自我评估：任务完成后，服务提供商将运用视频质量评估算法对转码后的结果进行自我评估，作为服务质量的基准。以上过程充分利用了 `poco-service` 模块中的转码逻辑和工具。

9. 提交结果：自我评估完成后，服务提供商将评估结果反馈给智能合约和服务请求者，供后续评估参考。

### 质量评估与共识

10. 双重服务质量评估：这包含了视频质量评估和编码速度评估。智能合约随机选定服务提供商参与视频质量的第三方评估，委员会汇总评估结果作为服务质量综合判断的依据

11. 质量共识决策：委员会成员对采集到的服务质量 metric 采用 PBFT 算法进行共识，共识过程中委员会成员可自行以领域特定的方式验证服务提供商的服务质量和产出结果。共识结果由委员会领导者记录在区块链上，更新服务提供商的质量评分。

### 奖励与记录同步

12. 自动奖励分配：智能合约根据服务质量评分，向表现优异的服务提供商自动分配奖励。

13. 同步链上记录：服务提供商定期复制并同步链上的任务记录和服务质量记录到本地数据库，保持数据一致性。

此完整任务流程描绘了从任务发布到完成的全过程，并通过共识机制强化了服务质量的客观评估，最终通过奖励机制激励优质服务，确保了整个系统的高效和公正。

## Complete Task Process

The complete task process commences with the formation of a committee and encompasses task publication, scheduling, execution, and self-assessment. It concludes with the evaluation of service quality through quality assessment and consensus, followed by reward distribution and record synchronization.

### Committee Formation

1. Selecting High-quality Service Providers: At the start of each cycle, the smart contract (poco-contract-v2) selects several service providers with a history of high-quality service based on service quality records on the blockchain.

2. Internal Collaboration: Committee members contact each other according to predefined off-chain interactions, form a committee, and achieve consensus on the service quality of providers internally.

### Task Publication

3. Service Request Submission: Service requesters submit media transcoding tasks to the smart contract (poco-contract-v2) via poco-agent, specifying transcoding requirements and off-chain interaction methods.

### Task Scheduling

4. Task Monitoring: Service providers monitor task announcements from the smart contract through poco-agent, evaluating whether to undertake a new task based on their current workload and capabilities.

5. Off-chain Negotiation: Service providers accepting tasks communicate with the requester via predefined off-chain methods to ensure consensus on task details.

6. Service Provider Selection: Requesters independently choose a service provider based on their service quality records, prioritizing the delivery of high-quality services.

### Task Execution and Self-assessment

7. Independent Task Completion: The selected service provider independently completes the media transcoding task off-chain based on task requirements.

8. Quality Self-assessment: After completion, the service provider uses a video quality assessment algorithm to self-evaluate the transcoded output as a benchmark for service quality.

9. Result Submission: The service provider submits the self-assessment results to the smart contract and requester for subsequent evaluations.

### Quality Assessment and Consensus

10. Dual Service Quality Evaluation: This includes video quality and encoding speed assessments. The smart contract randomly selects providers for third-party video quality assessment, with the committee consolidating results as a basis for overall service quality judgment.

11. Consensus on Quality Decision: Committee members use the PBFT algorithm for consensus on collected service quality metrics, with members independently verifying service quality and outputs. The consensus outcome is recorded on the blockchain by the committee leader, updating the service provider’s quality rating.

### Rewards and Record Synchronization

12. Automatic Reward Distribution: Based on the service quality rating, the smart contract automatically distributes rewards to outstanding service providers.

13. Synchronizing Blockchain Records: Service providers regularly duplicate and synchronize blockchain task records and service quality records to their local database, ensuring data consistency.

This entire process outlines the sequential flow from task publication to completion, strengthening objective evaluation of service quality through a consensus mechanism, and ultimately, incentivizing high-quality service through a rewards system to ensure efficiency and fairness of the entire system.
