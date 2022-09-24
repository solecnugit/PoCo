import { ManagerOptions, SocketOptions } from "socket.io-client";
import { Address, PocoConnection, PocoPeerConnection, PocoPeerSocketIOConnection, PocoSocketIOConnection, UnknownPocoConnectionTypeError } from "./connection";

export type PocoConnectionOptions = ({
    type: "socketIO"
    uri?: string
} & (Partial<ManagerOptions & SocketOptions> | undefined)) & { localAddress: Address }

export type PocoPeerConnectionOptions = ({
    type: "socketIO",
    connection: PocoConnection,
    remoteAddress: Address
})

export async function createPocoConnection(opts: PocoConnectionOptions): Promise<PocoConnection> {
    if (opts.type === "socketIO") {
        return new PocoSocketIOConnection(opts.localAddress, opts);
    }

    throw new UnknownPocoConnectionTypeError(opts.type)
}

export async function createPocoPeerConnection(opts: PocoPeerConnectionOptions): Promise<PocoPeerConnection> {
    if (opts.type === "socketIO") {
        return new PocoPeerSocketIOConnection(opts.connection.getLocalAddress(), opts.remoteAddress, opts.connection);
    }

    throw new UnknownPocoConnectionTypeError(opts.type)
}