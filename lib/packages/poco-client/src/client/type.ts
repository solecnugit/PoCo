import BN from "bn.js"
import { PocoServiceRole, Address } from "../eth"

export type PocoServiceOptions = {
    role: PocoServiceRole,
    endpoint: string
}

export type PocoPostJobOptions = {
    messenger: Address
}

export type PocoSubmitJobOptions = {
    jobId: BN,
    key: string
}

export type PocoServiceStatus = "online" | "offline" | "busy" | "unknown"
export type PocoJobStatus = "pending" | "done"

export type PocoServiceEntry = {
    role: PocoServiceRole;
    user: string;
    endpoint: string;
    status: PocoServiceStatus
}

export type PocoJob = {
    jobId: BN;
    owner: Address;
    messenger: Address;
    status: PocoJobStatus
}