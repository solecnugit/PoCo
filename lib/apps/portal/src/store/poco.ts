import { defineStore } from "pinia";
import { createPocoClient, Networks, PocoClient, PocoServiceRole, PocoClientLog, PocoClientLogLevel, PocoClientLogCategory, PocoClientJob, PocoClientServiceInfo } from "poco-client";
import { BigNumber, ethers } from "ethers";

interface UserInfo {
    account: string;
    balanceInWei: BigNumber;
}

interface NetworkInfo {
    networkName: Networks;
    blockNum: number;
}

declare global {
    interface Window {
        pocoClientInstance: PocoClient | undefined
    }
}

export const usePoco = defineStore("poco", {
    state: () => {
        return {
            initialized: false,
            userInfo: {
                account: "",
                balanceInWei: BigNumber.from(0),
            } as UserInfo,
            networkInfo: {
                networkName: "development" as Networks,
                blockNum: 0
            } as NetworkInfo,
            services: {
                messenger: [] as PocoClientServiceInfo[]
            },
            logs: [] as (PocoClientLog & { id: number })[],
            jobs: [] as (PocoClientJob & { buffer: ArrayBuffer | undefined })[],
            jobFileMapping: new Map<string, FileSystemHandle>
        }
    },
    actions: {
        async setup(network: Networks) {
            if (this.initialized || window.pocoClientInstance) {
                console.warn("Poco can not be initialized twice!")
                return;
            }

            window.pocoClientInstance = await createPocoClient(network)

            window.pocoClientInstance.on("Log", this.log.bind(this))
            window.pocoClientInstance.on("NewJob", (jobId, owner, messenger) => {
                this.jobs.push({
                    jobId,
                    owner,
                    messenger,
                    claimer: "",
                    status: "pending",
                    isOwn: owner === this.userAccount,
                    progressInfo: "",
                    buffer: undefined
                })
            })

            window.pocoClientInstance.on("JobProcessUpdate", (jobId, info) => {
                const job = this.jobs.find(e => e.jobId.eq(jobId));

                if (!job) {
                    this.log("warn", "client", new Date(), `Unknown job id ${jobId.toString()} emit progress update.`);
                    return;
                }

                job.progressInfo = info;
            })

            window.pocoClientInstance.on("JobStatusUpdate", (jobId, status) => {
                const job = this.jobs.find(e => e.jobId.eq(jobId));

                if (!job) {
                    this.log("warn", "client", new Date(), `Unknown job id ${jobId.toString()} emit status update.`);
                    return;
                }

                job.status = status;
            })

            window.pocoClientInstance.on("JobResultAvailable", (jobId, buffer) => {
                const job = this.jobs.find(e => e.jobId.eq(jobId));

                if (!job) {
                    this.log("warn", "client", new Date(), `Unknown job id ${jobId.toString()} emit result.`);
                    return;
                }

                job.buffer = buffer;
            })

            await window.pocoClientInstance.setup();

            this.userInfo = {
                account: window.pocoClientInstance.localAddress,
                balanceInWei: await window.pocoClientInstance.getBalance()
            };

            this.networkInfo = {
                networkName: network,
                blockNum: await window.pocoClientInstance.getBlockNumber()
            }

            this.services.messenger = Array.from(window.pocoClientInstance.getServices(PocoServiceRole.MESSENGER));

            this.jobs = Array.from(window.pocoClientInstance.getAllJobs()).map(e => {
                return {
                    ...e,
                    buffer: undefined
                }
            });

            this.initialized = true;
        },
        log(level: PocoClientLogLevel, category: PocoClientLogCategory, time: Date, message: string) {
            this.logs.push({
                id: this.logs.length,
                category,
                level,
                time,
                message
            })
        },
        async postJob() {
            if (!window.pocoClientInstance) {
                throw new Error("client not ready")
            }

            const [fileHandle] = await window.showOpenFilePicker({
                multiple: false
            });

            const jobId = await window.pocoClientInstance.postJob({
                file: await fileHandle.getFile(),
                messenger: this.services.messenger[0].provider
            })

            this.jobFileMapping.set(jobId.toString(), fileHandle)
        }
    },
    getters: {
        userBalance(state) {
            const balanceInEther = ethers.utils.formatUnits(state.userInfo.balanceInWei, "ether")

            return balanceInEther.substring(0, balanceInEther.indexOf(".") + 5)
        },

        userAccount: (state) => state.userInfo.account,
        networkName: (state) => state.networkInfo.networkName,
        networkBlockNum: (state) => state.networkInfo.blockNum.toString(),
        clientInstance: (state) => window.pocoClientInstance
    }
})