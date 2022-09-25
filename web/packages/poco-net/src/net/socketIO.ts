import _ from "lodash";
import { Socket, ManagerOptions, SocketOptions, io } from "socket.io-client";
import { PocoConnection, PocoPeerConnection } from "./connection";
import { Address, PocoConnectionStatus, PocoConnectionTimeoutError, PocoPeerConnectionOptions, PocoPeerSocketIOConnectionOptions, PocoSocketIOConnectionOptions } from "./types";

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
    private options: PocoPeerSocketIOConnectionOptions | undefined;
    private messageCallback: (payload: any) => Promise<void>;
    private listeners: Map<string, {
        callback: (payload: any) => Promise<void>,
        once: boolean
    }>;

    constructor(localAddress: Address, remoteAddress: Address, connection: PocoConnection, opts?: PocoPeerSocketIOConnectionOptions) {
        super("socketIO", localAddress, remoteAddress);

        this.connection = connection;
        this.connectionStatus = "pending";
        this.options = opts;
        this.listeners = new Map();
        this.messageCallback = async () => { };

        this.connection.onEvent("peer message", async ({ payload, fromAddress, toAddress }: { payload: any, fromAddress: Address, toAddress: Address }) => {
            if (fromAddress !== this.remoteAddress || toAddress != this.localAddress) {
                return;
            }

            this.messageCallback(payload);
        })

        this.connection.onEvent("peer event", async ({ event, payload, fromAddress, toAddress }: { event: string, payload: any, fromAddress: Address, toAddress: Address }) => {
            console.log(event, payload, fromAddress, toAddress)

            if (fromAddress !== this.remoteAddress || toAddress != this.localAddress) {
                return;
            }

            if (!this.listeners.has(event)) {
                return;
            }

            const { once, callback } = this.listeners.get(event)!;

            await callback(payload)

            if (once) {
                this.listeners.delete(event);
            }
        })

        this.connection.onEvent("peer connection destroy", async () => {
            this.connectionStatus = "disconnected";
        })
    }

    async connect(): Promise<void> {
        if (this.connectionStatus === "connecting") {
            return;
        }

        if (this.connection.status() !== "connected") {
            await this.connection.connect();
        }

        const status = await Promise.race([
            new Promise<string>(resolve => setTimeout(() => {
                resolve("peer connection timeout")
            }, this.options?.timeout || 5000)),
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

            throw new PocoConnectionTimeoutError(this);
        }
    }

    async disconnect(): Promise<void> {
        if (this.connectionStatus == "disconnected") {
            return;
        }

        this.connection.emit("peer connection destroy", { fromAddress: this.localAddress, toAddress: this.remoteAddress })
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
        this.messageCallback = callback;
    }

    onEvent<T>(event: string, callback: (payload: T) => Promise<void>, once?: boolean | undefined): void {
        this.listeners.set(event, {
            callback,
            once: once || false
        })
    }
}