import _ from "lodash";
import { Socket, ManagerOptions, SocketOptions, io } from "socket.io-client";
import { PocoConnection, PocoPeerConnection } from "./connection";
import { Address, PocoConnectionStatus } from "./types";

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