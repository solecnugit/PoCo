import { io, ManagerOptions, Socket, SocketOptions } from "socket.io-client";
import _ from "lodash";

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

export type PocoPeerConnectionOptions = {
    timeout: number;
}

export type PocoPeerConnectionRequestResponse = {
    ok: boolean;
    reason: string;
}

export abstract class PocoPeerConnection extends PocoConnection {
    protected remoteAddress: Address;
    protected options: PocoPeerConnectionOptions;

    constructor(connectionType: PocoConnectionType, localAddress: Address, remoteAddress: Address, opts?: Partial<PocoPeerConnectionOptions>) {
        super(connectionType, localAddress);

        this.remoteAddress = remoteAddress;
        this.options = _.defaults(opts, { timeout: 5000 })
    }
}

export class PocoSocketIOConnection extends PocoConnection {
    private socket: Socket;

    constructor(localAddress: Address, opts?: Partial<ManagerOptions & SocketOptions & { uri?: string }> | undefined) {
        super("socketIO", localAddress)

        const defaultOpts = { autoConnect: false, transports: ["websocket"], protocols: [__PROTOCOL_VERSION__], auth: { address: localAddress } };

        if (opts === undefined) {
            this.socket = io(defaultOpts);
        } else if (opts.uri === undefined) {
            this.socket = io(_.defaults(opts, defaultOpts));
        } else {
            this.socket = io(opts.uri, _.defaults(opts, defaultOpts))
        }
    }

    async connect(): Promise<void> {
        this.socket.connect()
    }

    async disconnect(): Promise<void> {
        this.socket.disconnect();
    }

    async send<T>(payload: T): Promise<void> {
        this.socket.send(payload);
    }

    async emit<T>(event: string, payload: T): Promise<void> {
        this.socket.emit(event, payload)
    }

    onMessage<T>(callback: (payload: T) => Promise<void>): void {
        this.socket.on("message", callback);
    }

    onEvent<T>(event: string, callback: (payload: T) => Promise<void>, once?: boolean): void {
        if (once !== undefined && once) {
            this.socket.once(event, callback);
        } else {
            this.socket.on(event, callback);
        }
    }

    status(): PocoConnectionStatus {
        if (this.socket.connected) {
            return "connected";
        } else if (this.socket.disconnected) {
            return "disconnected";
        } else {
            return "pending";
        }
    }
}

export class PocoPeerSocketIOConnection extends PocoPeerConnection {
    private connection: PocoConnection;
    private connectionStatus: PocoConnectionStatus;

    constructor(localAddress: Address, remoteAddress: Address, connection: PocoConnection) {
        super("socketIO", localAddress, remoteAddress);

        this.connection = connection;
        this.connectionStatus = "pending";
    }

    async connect(): Promise<void> {
        if (this.connectionStatus === "connecting") {
            throw new Error("connecting...")
        }

        if (this.connection.status() !== "connected") {
            await this.connection.connect();
        }

        const status = await Promise.race([
            new Promise<string>(resolve => setTimeout(() => {
                resolve("peer connection timeout")
            }, this.options.timeout)),
            new Promise<string>(resolve => {
                this.connection.onEvent("peer connection established", async () => {
                    resolve("connected")
                }, true)

                this.connectionStatus = "connecting";
                this.connection.emit("peer connection setup", { fromAddress: this.localAddress, toAddress: this.remoteAddress })
            })
        ])

        if (status === "connected") {
            this.connectionStatus = "connected";
        } else {
            this.connectionStatus = "disconnected";

            throw new Error("Connection timeout")
        }
    }

    async disconnect(): Promise<void> {
        if (this.connectionStatus == "disconnected") {
            return;
        }

        this.connection.emit("destroy peer connection", { fromAddress: this.localAddress, toAddress: this.remoteAddress })
        this.connectionStatus = "disconnected";
    }

    status(): PocoConnectionStatus {
        return this.connectionStatus;
    }

    async send<T>(payload: T): Promise<void> {
        this.connection.emit("peer message", { fromAddress: this.localAddress, toAddress: this.remoteAddress, payload })
    }

    async emit<T>(event: string, payload: T): Promise<void> {
        this.connection.emit("peer event", { fromAddress: this.localAddress, toAddress: this.remoteAddress, event, payload })
    }

    onMessage<T>(callback: (payload: T) => Promise<void>): void {
        this.connection.onEvent("peer message", async ({ payload, fromAddress, toAddress }: { payload: T, fromAddress: Address, toAddress: Address }) => {
            if (fromAddress !== this.remoteAddress || toAddress != this.localAddress) {
                return;
            }

            await callback(payload)
        })
    }

    onEvent<T>(event: string, callback: (payload: T) => Promise<void>, once?: boolean | undefined): void {
        this.connection.onEvent("peer event", async ({ event: payloadEvent, payload, fromAddress, toAddress }: { event: string, payload: T, fromAddress: Address, toAddress: Address }) => {
            if (event != payloadEvent || fromAddress !== this.remoteAddress || toAddress != this.localAddress) {
                return;
            }

            await callback(payload)
        }, once)
    }
}