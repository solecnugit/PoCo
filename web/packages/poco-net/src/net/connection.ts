import _ from "lodash";
import { PocoConnectionType, Address, PocoConnectionStatus } from "./types";

export abstract class PocoConnection {
    protected connectionType: PocoConnectionType;
    protected localAddress: Address;

    constructor(connectionType: PocoConnectionType, localAddress: Address) {
        this.connectionType = connectionType;
        this.localAddress = localAddress;
    }

    getLocalAddress(): Address {
        return this.localAddress;
    }

    getConnectionType(): PocoConnectionType {
        return this.connectionType;
    }

    abstract connect(): Promise<void>
    abstract disconnect(): Promise<void>
    abstract status(): PocoConnectionStatus;

    abstract send<T>(payload: T): Promise<void>
    abstract emit<T>(event: string, payload: T): Promise<void>;
    abstract onMessage<T>(callback: (payload: T) => Promise<void>): void;
    abstract onEvent<T>(event: string, callback: (payload: T) => Promise<void>, once?: boolean): void;
}


export type PocoPeerConnectionRequestResponse = {
    ok: boolean;
    reason: string;
}

export abstract class PocoPeerConnection extends PocoConnection {
    protected remoteAddress: Address;
    protected options: { timeout: number };

    constructor(connectionType: PocoConnectionType, localAddress: Address, remoteAddress: Address, opts?: Partial<{ timeout: number }>) {
        super(connectionType, localAddress);

        this.remoteAddress = remoteAddress;
        this.options = _.defaults(opts, { timeout: 5000 })
    }
}