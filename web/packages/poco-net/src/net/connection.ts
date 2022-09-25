import _ from "lodash";
import { PocoConnectionType, Address, PocoConnectionStatus } from "./types";

export abstract class PocoConnection {
    public connectionType: PocoConnectionType;
    public localAddress: Address;

    constructor(connectionType: PocoConnectionType, localAddress: Address) {
        this.connectionType = connectionType;
        this.localAddress = localAddress;
    }

    abstract connect(): Promise<void>
    abstract disconnect(): Promise<void>
    abstract status(): PocoConnectionStatus;

    abstract send<T>(payload: T): Promise<void>
    abstract emit<T>(event: string, payload: T): Promise<void>;
    abstract onMessage<T>(callback: (payload: T) => Promise<void>): void;
    abstract onEvent<T>(event: string, callback: (payload: T) => Promise<void>, once?: boolean): void;
}


export abstract class PocoPeerConnection extends PocoConnection {
    public remoteAddress: Address;

    constructor(connectionType: PocoConnectionType, localAddress: Address, remoteAddress: Address) {
        super(connectionType, localAddress);

        this.remoteAddress = remoteAddress;
    }
}