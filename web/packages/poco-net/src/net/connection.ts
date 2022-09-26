import _ from "lodash";
import { PocoConnectionType, Address, PocoConnectionStatus } from "./types";

export abstract class PocoConnection {
    public connectionType: PocoConnectionType;
    public localAddress: Address;
    protected connectionStatus: PocoConnectionStatus;
    protected connectionStatusCallback: ((status: PocoConnectionStatus, connection: PocoConnection) => Promise<void>)[];

    constructor(connectionType: PocoConnectionType, localAddress: Address) {
        this.connectionType = connectionType;
        this.localAddress = localAddress;
        this.connectionStatus = "new";
        this.connectionStatusCallback = [];
    }

    abstract connect(): Promise<void>
    abstract disconnect(): Promise<void>
    abstract status(): PocoConnectionStatus;

    protected setStatus(status: PocoConnectionStatus): void {
        this.connectionStatus = status;

        for (const callback of this.connectionStatusCallback) {
            callback(status, this)
        }
    }

    onStatusChange(callback: (status: PocoConnectionStatus, connection: PocoConnection) => Promise<void>): void {
        if (this.connectionStatusCallback.find(e => e == callback))
            return;

        this.connectionStatusCallback.push(callback);
    }

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