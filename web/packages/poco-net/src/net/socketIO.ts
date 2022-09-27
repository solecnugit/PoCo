import _ from "lodash";
import { Socket, ManagerOptions, SocketOptions, io } from "socket.io-client";
import { PocoConnection, PocoPeerConnection } from "./connection";
import { Address, PocoConnectionStatus, PocoConnectionTimeoutError, PocoPeerSocketIOConnectionOptions } from "./types";

export class PocoSocketIOConnection extends PocoConnection {
    private socket: Socket;

    constructor(localAddress: Address, opts?: Partial<ManagerOptions & SocketOptions & { uri?: string }> | undefined) {
        super("socketIO", localAddress)

        const defaultOpts = { autoConnect: false, transports: ["websocket"], protocols: ["poco-alpha"], auth: { address: localAddress } };

        if (opts === undefined) {
            this.socket = io(defaultOpts);
        } else if (opts.uri === undefined) {
            this.socket = io(_.defaults(opts, defaultOpts));
        } else {
            this.socket = io(opts.uri, _.defaults(opts, defaultOpts))
        }

        this.socket.on("connect", () => {
            this.setStatus("connected")
        })

        this.socket.on("disconnect", (reason: string) => {
            if (reason === "io server disconnect" || reason === "io client disconnect") {
                this.setStatus("closed")
                return;
            }

            this.setStatus("disconnected")
        })

        this.socket.on("connect_eror", (error: Error) => {
            this.setStatus("disconnected")

            throw error;
        })
    }

    async connect(): Promise<void> {
        this.setStatus("connecting")

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
        return this.connectionStatus;
    }
}

export class PocoPeerSocketIOConnection extends PocoPeerConnection {
    private connection: PocoConnection;
    private options: PocoPeerSocketIOConnectionOptions | undefined;
    private messageCallback: ((payload: any) => Promise<void>)[];
    private listeners: Map<string, {
        callback: (payload: any) => Promise<void>,
        once: boolean
    }[]>;

    constructor(localAddress: Address, remoteAddress: Address, connection: PocoConnection, opts?: PocoPeerSocketIOConnectionOptions) {
        super("socketIO", localAddress, remoteAddress);

        this.connection = connection;
        this.options = opts;
        this.listeners = new Map();
        this.messageCallback = [];

        this.connection.onEvent("peer message", async ({ payload, fromAddress, toAddress }: { payload: any, fromAddress: Address, toAddress: Address }) => {
            if (fromAddress !== this.remoteAddress || toAddress != this.localAddress) {
                return;
            }

            for (const callback of this.messageCallback) {
                callback(payload)
            }
        })

        this.connection.onEvent("peer event", async ({ event, payload, fromAddress, toAddress }: { event: string, payload: any, fromAddress: Address, toAddress: Address }) => {
            debugger

            if (fromAddress !== this.remoteAddress || toAddress != this.localAddress) {
                return;
            }

            if (!this.listeners.has(event)) {
                return;
            }

            const listeners = this.listeners.get(event);

            if (!listeners || listeners?.length === 0) {
                return;
            }

            for (const { callback } of listeners) {
                await callback(payload)
            }

            this.listeners.set(event, listeners.filter(it => !it.once))
        })

        this.connection.onEvent("peer connection destroy", async () => {
            this.setStatus("closed")
        })

        this.connection.onStatusChange(async (status) => {
            switch (status) {
                case "closed": {
                    this.setStatus("closed");
                    break;
                }
            }
        })
    }

    async connect(): Promise<void> {
        if (this.connectionStatus === "connecting") {
            return;
        }

        if (this.connection.status() !== "connected" && this.connection.status() !== "closed") {
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

                this.setStatus("connecting")
                this.connection.emit("peer connection setup", { fromAddress: this.localAddress, toAddress: this.remoteAddress })
            })
        ])

        if (status === "connected") {
            this.setStatus("connected");
        } else {
            this.setStatus("failed");

            throw new PocoConnectionTimeoutError(this);
        }
    }

    async disconnect(): Promise<void> {
        if (this.connectionStatus == "disconnected") {
            return;
        }

        this.connection.emit("peer connection destroy", { fromAddress: this.localAddress, toAddress: this.remoteAddress })
        this.setStatus("closed")
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
        if (this.messageCallback.find(e => e == callback))
            return;

        this.messageCallback.push(callback);
    }

    onEvent<T>(event: string, callback: (payload: T) => Promise<void>, once?: boolean | undefined): void {
        const listeners = this.listeners.get(event) || [];
        const flag = once || false;

        if (!flag && listeners.find((e) => e.callback == callback))
            return;

        listeners.push({
            callback,
            once: flag
        })

        this.listeners.set(event, listeners)
    }
}