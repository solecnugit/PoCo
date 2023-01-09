import { Address, getContract, Networks, PocoServiceRole } from "../eth";
import { PocoClientNotReadyError } from "./error";
import {
  PocoClientJob,
  PocoClientLogCategory,
  PocoClientLogLevel,
  PocoClientPostJobOptions,
  PocoClientServiceInfo,
  PocoClientRegisterServiceOptions,
  PocoSubmitJobOptions,
  Optional,
  PocoTakeJobOptions,
  PocoClientSocketIOConnectionEvents,
  PocoClientPeerSocketIOEvents,
  PocoJobStatus,
} from "./type";
import { PocoLocalStorage, PocoStorage } from "../storage";
import {
  PocoSocketIOConnection,
  PocoPeerWebRTCConnection,
  createPocoSocketIOConnection,
  createPocoPeerSocketIOConnection,
  PocoPeerSocketIOConnection,
  createPocoPeerWebRTCConnection,
} from "@poco/net";
import { EventDispatcher } from "@poco/util";
import { BigNumber, ethers, providers } from "ethers";
import { JobCenter, ServiceRegistry } from "@poco/contract";
import { NewJobEvent, SubmitJobEvent } from "@poco/contract/dist/JobCenter";
import {
  NewServiceEvent,
  ServiceUpdateEvent,
} from "@poco/contract/dist/ServiceRegistry";
import { sha256Digest } from "../utils/crypto";
import { transcode } from "@poco/codec"

export interface PocoClientEvents {
  Log: (
    this: ThisType<PocoClient>,
    level: PocoClientLogLevel,
    category: PocoClientLogCategory,
    time: Date,
    message: string
  ) => void;
  JobProcessUpdate: (
    this: ThisType<PocoClient>,
    jobId: BigNumber,
    progressInfo: string
  ) => void;
  JobStatusUpdate: (
    this: ThisType<PocoClient>,
    jobId: BigNumber,
    status: PocoJobStatus
  ) => void;
  JobResultAvailable: (
    this: ThisType<PocoClient>,
    jobId: BigNumber,
    buffer: Uint8Array
  ) => void;
  NewJob: (
    this: ThisType<PocoClient>,
    jobId: BigNumber,
    owner: Address,
    messenger: Address
  ) => void;
  ChainIdChanged: (
    this: ThisType<PocoClient>,
    chainId: string
  ) => void,
  AccountChanged: (
    this: ThisType<PocoClient>,
    account: Address,
  ) => void
  AccountDisconnected: (
    this: ThisType<PocoClient>,
  ) => void
}

export interface PocoClientWebRTCEvents {
  TakeJob: (
    this: ThisType<PocoPeerWebRTCConnection<PocoClientWebRTCEvents>>,
    jobIdInString: string
  ) => void;
  JobError: (
    this: ThisType<PocoPeerWebRTCConnection<PocoClientWebRTCEvents>>,
    jobIdInString: string,
    error: string,
    shouldClose: boolean
  ) => void;
  SendJob: (
    this: ThisType<PocoPeerWebRTCConnection<PocoClientWebRTCEvents>>,
    jobIdInString: string,
    buffer: Uint8Array
  ) => void;
  SubmitJob: (
    this: ThisType<PocoPeerWebRTCConnection<PocoClientWebRTCEvents>>,
    jobIdInString: string,
    buffer: Uint8Array
  ) => void;
  SendJobKey: (
    this: ThisType<PocoPeerWebRTCConnection<PocoClientWebRTCEvents>>,
    jobIdInString: string,
    key: string,
    secret: string
  ) => void;
}

export class PocoClient extends EventDispatcher<PocoClientEvents> {
  public localAddress: Address;
  public chainId: string;

  private provider: providers.Web3Provider;
  private network: Networks;
  private storage: PocoStorage;

  /* Contracts */
  private jobCenter: JobCenter | undefined;
  private serviceRegistry: ServiceRegistry | undefined;

  /* Storage */
  private services: Map<Address, PocoClientServiceInfo>[];
  private jobs: Map<string, PocoClientJob>;

  /* Connections */
  private connections: Map<
    Address,
    PocoSocketIOConnection<PocoClientSocketIOConnectionEvents>
  >;
  private rtcConnections: Map<
    Address,
    PocoPeerWebRTCConnection<PocoClientWebRTCEvents>
  >;

  /* Files */
  private jobToFileMapping: Map<string, File>;

  constructor(
    provider: providers.Web3Provider,
    localAddress: Address,
    chainId: string,
    network?: Networks,
    storage?: PocoStorage
  ) {
    super();

    this.provider = provider;
    this.localAddress = localAddress;
    this.chainId = chainId;
    this.network = network || "development";
    this.storage = storage || new PocoLocalStorage();

    this.services = [];
    this.jobs = new Map();
    this.jobToFileMapping = new Map();

    this.connections = new Map();
    this.rtcConnections = new Map();
  }

  protected log(
    level: PocoClientLogLevel,
    category: PocoClientLogCategory,
    message: string
  ) {
    this.emit("Log", level, category, new Date(), message);
  }

  protected updateJobStatus(jobId: BigNumber, status: PocoJobStatus) {
    this.jobs.get(jobId.toString())!.status = status;

    this.emit("JobStatusUpdate", jobId, status);
  }

  protected updateJobProgress(jobId: BigNumber, info: string) {
    this.jobs.get(jobId.toString())!.progressInfo = info;

    this.emit("JobProcessUpdate", jobId, info);
  }

  private async recover() {
    this.log("info", "client", "Recover services from blockchain.");

    // ServiceRegistry
    let events = (
      await this.serviceRegistry!.queryFilter("*" as any, 0)
    ).reverse();

    const roles = Object.keys(PocoServiceRole).length / 2;

    for (let i = 0; i < roles; i++) {
      this.services.push(new Map());
    }

    for (const eventData of events) {
      const eventName = eventData.event;

      if (eventName === "NewService" || eventName == "ServiceUpdate") {
        const { provider, endpoint, role } = (
          eventData as NewServiceEvent | ServiceUpdateEvent
        ).args;

        const services = this.services[role]!;

        if (services.has(provider)) continue;

        services.set(provider, {
          provider,
          endpoint,
          role: role,
          status: "unknown",
        });
      }
    }

    this.log("info", "client", "Recover jobs from blockchain.");

    // JobCenter
    events = await this.jobCenter!.queryFilter("*" as any, 0);

    for (const eventData of events) {
      const eventName = eventData.event;

      if (eventName === "NewJob") {
        const { jobId, owner, messenger } = (eventData as NewJobEvent).args;
        const jobIdInString = jobId.toString();

        this.jobs.set(jobIdInString, {
          jobId,
          messenger,
          owner,
          status: "pending",
          isOwn: owner === this.localAddress,
          progressInfo: "",
          claimer: "",
        });
      } else if (eventName === "SubmitJob") {
        const { jobId, claimer } = (eventData as SubmitJobEvent).args;
        const jobIdInString = jobId.toString();

        const job = this.jobs.get(jobIdInString)!;

        job.status = "submitted";
        job.claimer = claimer;
        job.progressInfo = `Claimed by ${claimer}`;
      }
    }
  }

  async connectToMessenger() {
    const connections = Array.from(
      this.getServices(PocoServiceRole.MESSENGER)
    ).map((e) => {
      return {
        id: e.provider,
        connection:
          createPocoSocketIOConnection<PocoClientSocketIOConnectionEvents>({
            type: "socketIO",
            uri: e.endpoint,
            localAddress: this.localAddress,
          }),
      };
    });

    const result = await Promise.allSettled(
      connections.map(
        (e) =>
          new Promise((resolve, reject) => {
            e.connection.connect().then(resolve).catch(reject);
          })
      )
    );

    for (let i = 0; i < result.length; i++) {
      const { status } = result[i];
      const { id, connection } = connections[i];

      if (status === "fulfilled" && connection.status() === "connected") {
        this.setupConnection(connection);
        this.connections.set(id, connection);
        this.services[PocoServiceRole.MESSENGER].get(id)!.status = "online";

        this.log("info", "client", `Connect to Messenger ${id} successfully.`);
      } else {
        this.services[PocoServiceRole.MESSENGER].get(id)!.status = "offline";

        this.log("warn", "client", `Failed to connect to Messenger ${id}.`);
      }
    }
  }

  private setupConnection(
    connection: PocoSocketIOConnection<PocoClientSocketIOConnectionEvents>
  ) {
    const setupPeerConnection = (
      //peer的属性是PocoPeerSocketIOConnection
      peer: PocoPeerSocketIOConnection<PocoClientPeerSocketIOEvents>
    ) => {
      //监听webrtc offer
      peer.on("webrtc offer", async (offer) => {
        //这里正式创建PocoPeerWebRTCConnection
        const rtcConnection =
          new PocoPeerWebRTCConnection<PocoClientWebRTCEvents>(
            peer.remoteAddress,
            peer.localAddress,
            peer as any,
            {
              offer: offer,
            }
          );

          //创建后对webrocconnection进行监听
          //webrtc connection监听是否connected
        rtcConnection.on("connected", () => {
          this.log(
            "info",
            "client",
            `Create WebRTC connection to ${peer.remoteAddress} successfully`
          );

          this.rtcConnections.set(peer.remoteAddress, rtcConnection);
        });

        //webrtc connection监听message
        rtcConnection.on("message", (message) => {
          this.log(
            "info",
            "network",
            `Message from ${rtcConnection.remoteAddress}: ${message}`
          );
        });

        rtcConnection.on("channel open", (channel) => {
          this.log(
            "info",
            "client",
            `WebRTCChannel ${channel.label} to ${rtcConnection.remoteAddress} opened.`
          );
        });

        rtcConnection.on("channel close", (channel) => {
          this.log(
            "info",
            "client",
            `WebRTCChannel ${channel.label} to ${rtcConnection.remoteAddress} closed.`
          );
        });

        rtcConnection.on("channel error", (channel, error) => {
          this.log(
            "info",
            "client",
            `WebRTCChannel ${channel.label} to ${rtcConnection.remoteAddress} got error.`
          );

          console.error(error);
        });

        //connection监听到TakeJob触发后做的事情
        rtcConnection.on("TakeJob", async (jobIdInString) => {
          const file = this.jobToFileMapping.get(jobIdInString);

          if (!file) {
            this.log("error", "client", `Missing file of job ${jobIdInString}`);
            await rtcConnection.send(
              "JobError",
              jobIdInString,
              "Job file missing",
              true
            );

            this.rtcConnections.delete(rtcConnection.remoteAddress);

            return;
          }

          //更新job的progress
          this.updateJobProgress(
            BigNumber.from(jobIdInString),
            `${rtcConnection.remoteAddress} took the job.`
          );

          //更新job的状态
          this.updateJobStatus(
            BigNumber.from(jobIdInString),
            "running",
          )

          //在takejob的回调中最后触发调用sendJob
          await rtcConnection.send(
            "SendJob",
            jobIdInString,
            new Uint8Array(await file.arrayBuffer())
          );
        });

        //rtc connection触发了submitjob之后做的事情
        rtcConnection.on("SubmitJob", async (jobIdInString, buffer) => {
          const jobId = BigNumber.from(jobIdInString);

          this.updateJobProgress(
            jobId,
            `${rtcConnection.remoteAddress} returned the job result.`
          );

          //将buffer传递至JobResultAvailable
          this.emit("JobResultAvailable", jobId, buffer);
          const { secret, key } = this.storage.getItem(
            `job-${jobIdInString}`
          ) as any;

          await rtcConnection.send("SendJobKey", jobIdInString, key, secret);
        });

        await rtcConnection.connect();
      });
    };

    //直到这里，setupConnection方法才结束，
    //PocoSocketIOConnection 监听peer setu
    connection.on("peer setup", async (from, to) => {
      if (to !== this.localAddress) {
        return;
      }

      this.log(
        "info",
        "client",
        `Receive peer connection request from ${from}.`
      );

      //在这里创建PocoPeerSocketIOConnection
      const peerConnection =
        createPocoPeerSocketIOConnection<PocoClientPeerSocketIOEvents>({
          type: "socketIO",
          localAddress: this.localAddress,
          remoteAddress: from,
          connection: connection as any,
          timeout: 5000,
        });

      setupPeerConnection(peerConnection);

      await peerConnection.connect();
    });

    //PocoSocketIOConnection 监听peer connected
    connection.on("peer connected", async () => {
      this.log("info", "client", `Peer connection established successfully.`);
    });
  }

  protected setupMetaMaskListeners() {
    const ethereum = (window as any).ethereum;

    if (typeof ethereum == "undefined") {
      throw new Error("ethereum is undefined, havn't installed MetaMask? uh.")
    }

    ethereum.on("chainChanged", (_chainId: string) => {
      this.emit("ChainIdChanged", _chainId)
    })

    ethereum.on("accountsChanged", (accounts: Address[]) => {
      if (accounts.length === 0) {
        this.localAddress = "THIS_IS_AN_ILLEGAL_ADDRESS_ONLY_WHEN_ACCOUNT_DISCONNECTED";
        this.emit("AccountDisconnected")
      } else if (accounts[0] != this.localAddress) {
        this.localAddress = accounts[0];
        this.emit("AccountChanged", this.localAddress)
      }
    })
  }

  async setup(force?: boolean) {
    this.setupMetaMaskListeners();

    this.jobCenter = await getContract<JobCenter>(this.provider, "JobCenter", {
      network: this.network,
    });
    this.serviceRegistry = await getContract<ServiceRegistry>(
      this.provider,
      "ServiceRegistry",
      { network: this.network }
    );

    if (force) {
      this.storage.clear();
    }

    await this.recover();

    this.setupContractEventListeners();

    await this.connectToMessenger();
  }

  private setupContractEventListeners() {
    const newJobFilter = this.jobCenter!.filters.NewJob();

    this.jobCenter!.on(newJobFilter, (jobId, owner, messenger) => {
      this.jobs.set(jobId.toString(), {
        jobId,
        owner,
        messenger,
        isOwn: owner === this.localAddress,
        status: "pending",
        progressInfo: "",
        claimer: "",
      });
      this.emit("NewJob", jobId, owner, messenger);
    });
  }

  async registerService({ role, endpoint }: PocoClientRegisterServiceOptions) {
    if (!this.serviceRegistry) {
      throw new PocoClientNotReadyError(this);
    }

    await (
      await this.serviceRegistry.setRecord({
        role,
        endpoint,
      })
    ).wait();
  }

  // deploy task通过postjob发送文件
  // 在这里加入split方法，将文件进行切分
  async postJob(
    opts: Optional<PocoClientPostJobOptions, "messenger">
  ): Promise<BigNumber> {
    if (!this.jobCenter) {
      throw new PocoClientNotReadyError(this);
    }
    console.log("opts file 输出");
    console.log(opts.file);
    // 将file转成arrayBuffer
    const fileBuffer = await opts.file.arrayBuffer();
    // 获取arrayBuffer的hash
    const fileHash = await sha256Digest(fileBuffer);

    console.log("获取当前filehash");
    console.log(fileHash);

    const messengerToUse =
    // address 应该是发送者的
      opts?.messenger ||
      Array.from(this.getServices(PocoServiceRole.MESSENGER))[0].provider;

      // 加密算法
    const key = (
      (await this.getBlockNumber()) *
      Math.random() *
      Date.now()
    ).toString(16);
    const secret = ethers.utils.keccak256(ethers.utils.toUtf8Bytes(key));

    console.log("jobcenter post job start");
    const response = await (
      await this.jobCenter.postJob(messengerToUse, secret)
    ).wait();
    console.log("jobcenter post job over");
    const args = (response.events![0] as NewJobEvent).args;
    const jobId = args["jobId"];
    const jobIdInString = jobId.toString();

    this.storage.setItem(`job-${jobIdInString}`, {
      key,
      secret,
      fileHash,
    });

//  这里不用动了把
    this.jobToFileMapping.set(jobIdInString, opts.file);

    this.log("info", "client", `New job ${jobIdInString} has been posted.`);

    return jobId;
  }

  //点击takehjob按钮之后，第一步执行这个
  async takeJob({ jobId }: PocoTakeJobOptions) {
    if (this.connections.size === 0) {
      throw new PocoClientNotReadyError(this);
    }

    const jobIdInString = jobId.toString();
    const job = this.jobs.get(jobIdInString)!;

    if (!this.connections.has(job.messenger)) {
      this.log(
        "error",
        "client",
        `we have not connect to ${job.messenger} yet`
      );
      throw new Error(`we have not connect to ${job.messenger} yet`);
    }

    //更新job的状态和progress
    this.updateJobStatus(jobId, "running");
    this.updateJobProgress(jobId, "Ready to establish connection.");

    const connection = this.connections.get(job.messenger)!;
    //通过job的connection创建PocoPeerSocketIOConnection
    const peerConnection = createPocoPeerSocketIOConnection({
      type: "socketIO",
      localAddress: this.localAddress,
      remoteAddress: job.owner,
      connection: connection as any,
      timeout: 5000,
    });

    await peerConnection.connect();

    this.updateJobProgress(
      jobId,
      "SocketIO peer connection established successfully."
    );

    //根据PocoPeerSocketIOConnection创建PocoPeerWebRTCConnection
    const rtcConnection =
      createPocoPeerWebRTCConnection<PocoClientWebRTCEvents>({
        type: "webrtc",
        localAddress: this.localAddress,
        remoteAddress: job.owner,
        connection: peerConnection,
      });

    rtcConnection.on("channel open", (channel) => {
      this.log(
        "info",
        "client",
        `WebRTCChannel ${channel.label} to ${rtcConnection.remoteAddress} opened.`
      );
    });

    rtcConnection.on("channel close", (channel) => {
      this.log(
        "info",
        "client",
        `WebRTCChannel ${channel.label} to ${rtcConnection.remoteAddress} closed.`
      );
    });

    rtcConnection.on("channel error", (channel, error) => {
      this.log(
        "info",
        "client",
        `WebRTCChannel ${channel.label} to ${rtcConnection.remoteAddress} got error.`
      );

      console.error(error);
    });

    rtcConnection.on("JobError", async (jobIdInString, error, shouldClose) => {
      this.log(
        "error",
        "network",
        `Error when handling ${jobIdInString}: ${error}`
      );

      if (shouldClose) {
        this.rtcConnections.delete(rtcConnection.remoteAddress);

        this.updateJobStatus(jobId, "pending");

        await rtcConnection.disconnect();
        await peerConnection.disconnect();
      }
    });

    //监听sendJob方法的调用
    rtcConnection.on("SendJob", async (jobIdInString, buffer) => {
      this.log("info", "network", `Receive buffer of ${jobIdInString}`);

      this.updateJobProgress(
        BigNumber.from(jobIdInString),
        "Job buffer received."
      );


      //之前的写法：将整个buffer全部转换完才会返回，现在需要改成流式传输
      //此时接收到的buffer应该已经是流失传输后的一个个分片
      //应当在此前就进行分片了
      // const finalbuffer = await transcode(buffer);
      const finalbuffer = await transcode(buffer);

      //以下部分是第一次对buffer进行修改，并且尝试改为stream的过程。
      // const finalStream = await transcodeStream(buffer);

      // const streamReader = finalStream.getReader();

      // async function dealStream({done, value}: ReadableStreamReadResult<Uint8Array>) {
      //   if(done) {
      //     // 发送一个特殊指令，表示传完了
      //     return;
      //   }

      //   let currentBuf = value.buffer;

      //   console.log(currentBuf);

      //   //@ts-ignore
      //   await rtcConnection.send("SubmitJob", jobIdInString, currentBuf);

      //   streamReader.read().then(dealStream);
      // }
      // streamReader.read().then(dealStream);



      //在触发SendJob回调之后的最后一步，调用SubnmitJob
      //@ts-ignore
      await rtcConnection.send("SubmitJob", jobIdInString, finalbuffer);
    });

    rtcConnection.on("SendJobKey", async (jobIdInString, key, secret) => {
      const jobId = BigNumber.from(jobIdInString);
      this.updateJobProgress(jobId, "Job key received.");

      const selfSecret = ethers.utils.keccak256(ethers.utils.toUtf8Bytes(key));

      if (selfSecret !== secret) {
        this.log("error", "network", `Mismatched secret get from ${jobId}.`);
        return;
      }

      await this.submitJob({
        jobId: BigNumber.from(jobIdInString),
        key,
      });

      this.log("info", "network", `Job ${jobIdInString} have been submitted.`);
      this.updateJobStatus(jobId, "submitted");
      this.updateJobProgress(jobId, "Job submitted.");
    });

    //所有事件的监听发生在connect之前
    await rtcConnection.connect();

    this.updateJobProgress(
      jobId,
      "WebRTC connection established successfully."
    );

    this.log(
      "info",
      "client",
      `Create WebRTC connection to ${peerConnection.remoteAddress} successfully.`
    );

    //rtcconnection触发TakeJob事件，等待其完成
    await rtcConnection.send("TakeJob", jobIdInString);

    this.updateJobProgress(jobId, "Sending TakeJob request.");

    //在connections中添加当前的connection
    this.rtcConnections.set(peerConnection.remoteAddress, rtcConnection);
  }

  async submitJob({ jobId, key }: PocoSubmitJobOptions) {
    if (!this.jobCenter) {
      throw new PocoClientNotReadyError(this);
    }

    await (
      await this.jobCenter.submitJob(jobId, ethers.utils.toUtf8Bytes(key))
    ).wait();
  }

  getServices(role: PocoServiceRole): Iterable<PocoClientServiceInfo> {
    return this.services[role].values();
  }

  getAllJobs(): Iterable<PocoClientJob> {
    return this.jobs.values();
  }

  async getBalance(address?: Address): Promise<BigNumber> {
    if (!this.provider) {
      throw new PocoClientNotReadyError(this);
    }

    const addressToQuery = address || this.localAddress;

    return this.provider.getBalance(addressToQuery);
  }

  async getBlockNumber(): Promise<number> {
    if (!this.provider) {
      throw new PocoClientNotReadyError(this);
    }

    return this.provider.getBlockNumber();
  }

  setJobFile(jobId: BigNumber, file: File) {
    this.jobToFileMapping.set(jobId.toString(), file);
  }

  getJobFile(jobId: BigNumber): File {
    return this.jobToFileMapping.get(jobId.toString())!;
  }

  hasJobFile(jobId: BigNumber): boolean {
    return this.jobToFileMapping.get(jobId.toString()) !== undefined;
  }
}
