import { PocoConnection, PocoPeerConnection } from "./connection";
import { PocoSocketIOConnection, PocoPeerSocketIOConnection } from "./socketIO";
import { PocoConnectionOptions, PocoPeerConnectionOptions, UnknownPocoConnectionTypeError } from "./types";

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