import { ManagerOptions, SocketOptions } from "socket.io-client";
import { PocoConnection } from "./connection";

export type PocoConnectionType = "websocket" | "webrtc" | "socketIO"

export class UnknownPocoConnectionTypeError extends Error {
    public type: string;

    constructor(_type: string) {
        super(`unknown connection type ${_type}`);
        this.type = _type;
    }
}

export type Address = string;
export type PocoConnectionStatus = "connected" | "disconnected" | "pending" | "connecting";

export type PocoConnectionOptions = ({
    type: "socketIO"
    uri?: string,
    localAddress: Address;
} & (Partial<ManagerOptions & SocketOptions> | undefined))

export type PocoPeerConnectionOptions = ({
    type: "socketIO",
    connection: PocoConnection,
    remoteAddress: Address,
    timeout: number;
})