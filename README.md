## Project Name: QoS-aware Video Transcoding System based on Web3

## Project Overview

The Decentralized Media Transcoding Service aims to capitalize on the wealth of unused computational resources around the world, crafting an efficient, cost-effective video transcoding network that places a premium on service quality. By offering fully traceable and transparent service records and assessment mechanisms, our project solidifies the network's autonomy. We are dedicated to generating income for providers of idle computing resources while delivering high-quality, more economical transcoding services to our customers.

### Key Features

- Traceability: Our system diligently records the outcomes and quality of each transcoding service, enhancing trust and enabling effective quality monitoring of service providers. This also bolsters security measures against malicious activities or attacks.
- Transparency: We guarantee an open and transparent assessment of service quality and scheduling processes. This ensures subpar nodes are filtered out while fostering fair competition, coaxing providers to elevate their service standards.
- Autonomy: Nodes within our service network autonomously render services adhering to established agreements. They undergo evaluations based on traceable task records and collectively reach consensus on quality assessments. This approach mitigates the risks of a single point of failure and unfair task distribution due to any autonomous entity acting in self-interest.

## Version Updates

This project adheres to Semantic Versioning rules. Each update reflects our commitment to continuous improvement and feature enhancement. Here is a history of the version iterations since the project's inception.

### v0.2.0 - May 2024

Key highlights of this update include:

#### Performance Optimization:

We upgraded the `poco-contract-v2`, transitioning the technical platform from Ethereum to NEAR. NEAR's sharding technology and high throughput enhance our system's ability to handle complex media transcoding tasks. This not only improves transaction efficiency but significantly reduces the cost of processing media transcoding tasks on a large scale, ensuring more efficient and economical operations.

#### Introduction of QoS Assessment:

In this version, we formally introduced the `poco-service` module, which defines specific Quality of Service standards for media transcoding, and includes off-chain media transcoding logic and domain-specific algorithmic logic executed by service providers. This detailed assessment of service quality ensures every media transcoding service is objectively and accurately evaluated.

#### Introduction of Committee for QoS consensus:

To further enhance the credibility and transparency of service quality assessments, this update introduced the `poco-committee` module, implementing an innovative Committee mechanism to facilitate collective consensus decision-making on Quality of Service (QoS). This multi-verifier consensus strategy enhances the objectivity and fairness of assessment results, effectively curbing potential dishonesty and bolstering the platform's overall reputation.

### v0.1.0 - June 2023

Update highlights:

#### Ethereum-based Web Video Transcoding Prototype:

We developed a prototype for web video transcoding based on Ethereum, integrating specially designed underlying utilities, including efficient network transport tools and advanced video codec libraries. By leveraging blockchain technology, we ensured transparency and traceability in the processing, providing a secure and reliable video processing platform for users.

#### Optimization of Video Quality Assessment Algorithm based on Video Group Extraction:

Video quality assessment, a key indicator of domain-specific service quality, is crucial throughout the service chain. Addressing issues of high computational requirements and long duration brought by existing video quality assessment algorithms, we implemented a method based on Video Group (GOP, Group of Pictures) extraction. Without sacrificing assessment accuracy, this approach significantly reduces reliance on computational power and shortens the necessary assessment time. The optimized algorithm maintains accuracy while reducing both assessment time and the number of execution instructions to less than half of the original.

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
    - 2023.12 Addressed issues encountered during merging with `poco-service`.
    - 2024.3 Adjusted for NEAR-SDK V5.0.0 changes and corresponding library updates.

### `poco-service`

- Purpose: This module is the core component where service providers execute tasks, encapsulating the logic for media transcoding off-chain. It also includes domain-specific algorithms needed for assessing service quality.

- Main Updates:

  - Integrated and migrated code related to QoS assessment from the original `poco-codec` module for more unified and efficient service operation.

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
    - 【In Progress】2024.04 Addressing issues encountered during interaction with `poco-service`.
    - 2024.04 Addressing issues when interacting with `poco-agent`.

  ### `poco-agent`

- Purpose: This module acts as an interface between users and between users and smart contracts, providing middleware that seamlessly connects with the frontend. It encapsulates various (previously mentioned) module operations both on-chain and off-chain and unifies interfaces for triggering different interaction logics, greatly improving user experience and facilitating easy operations.
- Main Updates:

  - Enhanced module usability and performance through a comprehensive overhaul of the old `poco-client`.

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

## Complete Task Process

The complete task process commences with the formation of a committee and encompasses task publication, scheduling, execution, and self-assessment. It concludes with the evaluation of service quality through quality assessment and consensus, followed by reward distribution and record synchronization.

### Committee Formation

1. Selecting High-quality Service Providers: At the start of each cycle, the smart contract ( `poco-contract-v2` ) selects several service providers with a history of high-quality service based on service quality records on the blockchain.

2. Internal Collaboration: Committee members contact each other according to predefined off-chain interactions, form a committee, and achieve consensus on the service quality of providers internally.

### Task Publication

3. Service Request Submission: Service requesters submit media transcoding tasks to the smart contract ( `poco-contract-v2` ) via `poco-agent` , specifying transcoding requirements and off-chain interaction methods.

### Task Scheduling

4. Task Monitoring: Service providers monitor task announcements from the smart contract through `poco-agent` , evaluating whether to undertake a new task based on their current workload and capabilities.

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
