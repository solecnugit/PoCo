import { JobCenterInstance, ServiceRegistryInstance } from "poco-contract";
import type * as ServiceRegistryContract from "poco-contract/types/ServiceRegistry";
import type * as JobCenterContract from "poco-contract/types/JobCenter";
import { Address, getContractInstance, Networks, PocoContractInstance, PocoServiceRole } from "../eth";
import JobCenterABI from "poco-contract/abi/JobCenter.json";
import ServiceRegistryABI from "poco-contract/abi/ServiceRegistry.json";
import { PocoClientNotReadyError } from "./error";
import { PocoJob, PocoPostJobOptions, PocoServiceEntry, PocoServiceOptions, PocoSubmitJobOptions } from "./type";
import web3 from "web3";
import { AbstractProvider } from "web3-core";
import detectEthereumProvider from '@metamask/detect-provider'
import { PocoLocalStorage, PocoStorage } from "../storage";
import BN from "bn.js";
import { PocoSocketIOConnection, PocoPeerWebRTCConnection, createPocoSocketIOConnection } from "poco-net";
import { EventDispatcher } from "poco-util";

export class PocoClient extends EventDispatcher {
    public localAddress: Address;
    private provider: AbstractProvider;
    private network: Networks;
    private storage: PocoStorage;

    /* Contracts */
    private jobCenter: PocoContractInstance<JobCenterInstance> | undefined;
    private serviceRegistry: PocoContractInstance<ServiceRegistryInstance> | undefined;

    /* Storage */
    private services: Map<Address, PocoServiceEntry>[];
    private jobs: Map<string, PocoJob>;

    /* Connections */
    private connections: Map<Address, PocoSocketIOConnection>;
    // @ts-ignore
    private rtcConnections: Map<Address, PocoPeerWebRTCConnection>;

    constructor(provider: AbstractProvider, localAddress: Address, network?: Networks, storage?: PocoStorage) {
        super();

        this.provider = provider;
        this.localAddress = localAddress;
        this.network = network || "development";
        this.storage = storage || new PocoLocalStorage();

        this.services = [];
        this.jobs = new Map();

        this.connections = new Map();
        this.rtcConnections = new Map();
    }

    private async recover() {
        // ServiceRegistry

        let events = (await this.serviceRegistry!.getPastEvents("allEvents", {
            fromBlock: 0
        })).reverse();

        const roles = Object.keys(PocoServiceRole).length / 2;

        for (let i = 0; i < roles; i++) {
            this.services.push(new Map())
        }

        for (const eventData of events) {
            const eventName = eventData.event;

            if (eventName === "NewService" || eventName == "ServiceUpdate") {
                const { user, endpoint, role } = (eventData as any).args as
                    (ServiceRegistryContract.NewService["args"]
                        | ServiceRegistryContract.ServiceUpdate["args"]);

                const services = this.services[role.toNumber()]!;

                if (services.has(user))
                    continue;

                services.set(user, {
                    user,
                    endpoint,
                    role: role.toNumber(),
                    status: "unknown"
                })
            }
        }

        // JobCenter
        events = await this.jobCenter!.getPastEvents("allEvents", {
            fromBlock: 0
        })

        for (const eventData of events) {
            const eventName = eventData.event;

            if (eventName === "NewJob") {
                const { jobId, owner, messenger } = (eventData as any).args as JobCenterContract.NewJob["args"];

                this.jobs.set(jobId.toString(), {
                    jobId,
                    messenger,
                    owner,
                    status: "pending"
                })
            } else if (eventName === "SubmitJob") {
                const { jobId } = (eventData as any).args as JobCenterContract.SubmitJob["args"];

                this.jobs.delete(jobId.toString())
            }
        }
    }

    async connectToMessenger() {
        const connections = Array.from(this.getServices(PocoServiceRole.MESSENGER)).map(e => {
            return {
                id: e.user,
                connection: createPocoSocketIOConnection({
                    type: "socketIO",
                    uri: e.endpoint,
                    localAddress: this.localAddress
                })
            }
        }
        );

        const result = await Promise.allSettled(
            connections.map(e => new Promise((resolve, reject) => {
                e.connection
                    .connect()
                    .then(resolve)
                    .catch(reject)
            }))
        );

        for (let i = 0; i < result.length; i++) {
            const { status } = result[i];
            const { id, connection } = connections[i];

            if (status === "fulfilled" && connection.status() === "connected") {
                this.connections.set(id, connection);
                this.services[PocoServiceRole.MESSENGER].get(id)!.status = "online"
            } else {
                this.services[PocoServiceRole.MESSENGER].get(id)!.status = "offline"
            }
        }
    }

    async setup(force?: boolean) {
        this.jobCenter = await getContractInstance<JobCenterInstance>
            (this.provider, JobCenterABI, "JobCenter", { network: this.network });
        this.serviceRegistry = await getContractInstance<ServiceRegistryInstance>
            (this.provider, ServiceRegistryABI, "ServiceRegistry", { network: this.network });

        if (force) {
            this.storage.clear();
        }

        await this.recover();
        await this.connectToMessenger();
    }

    async registerService({ role, endpoint }: PocoServiceOptions) {
        if (!this.serviceRegistry) {
            throw new PocoClientNotReadyError(this);
        }

        await this.serviceRegistry.setRecord({
            role,
            endpoint
        }, {
            from: this.localAddress
        })
    }

    async postJob(opts?: Partial<PocoPostJobOptions>) {
        if (!this.jobCenter) {
            throw new PocoClientNotReadyError(this);
        }

        const messengerToUse = opts?.messenger || Array.from(this.getServices(PocoServiceRole.MESSENGER))[0].user;

        const key = ((await this.getBlockNumber()).toNumber() * Math.random() * Date.now()).toString(16);
        const secret = web3.utils.keccak256(key);

        const response = await this.jobCenter.postJob(messengerToUse, secret, {
            from: this.localAddress
        })

        // @ts-ignore
        const jobId = response.logs[0].args["jobId"];

        this.storage.setItem(`job-${jobId.toString()}`, {
            key,
            secret
        })
    }

    async submitJob({ jobId, key }: PocoSubmitJobOptions) {
        if (!this.jobCenter) {
            throw new PocoClientNotReadyError(this);
        }

        await this.jobCenter.submitJob(jobId, key, {
            from: this.localAddress
        })
    }

    getServices(role: PocoServiceRole): Iterable<PocoServiceEntry> {
        return this.services[role].values()
    }

    getJobs(): Iterable<PocoJob> {
        return this.jobs.values();
    }

    async getBalance(address?: Address): Promise<BN> {
        if (!this.provider) {
            throw new PocoClientNotReadyError(this);
        }

        const addressToQuery = address || this.localAddress;

        const result: string = await this.provider.request!({
            method: 'eth_getBalance',
            params: [
                addressToQuery,
                "latest"
            ]
        })

        return web3.utils.toBN(result);
    }

    async getBlockNumber(): Promise<BN> {
        if (!this.provider) {
            throw new PocoClientNotReadyError(this);
        }

        const result: string = await this.provider.request!({
            method: 'eth_blockNumber'
        })

        return web3.utils.toBN(result);
    }
}

export async function createPocoClient(network?: Networks): Promise<PocoClient> {
    const provider = await detectEthereumProvider({
        mustBeMetaMask: true
    }) as any;

    const [account] = await provider.request({ method: 'eth_requestAccounts' });

    const client = new PocoClient(provider, account, network);

    await client.setup();

    return client;
}