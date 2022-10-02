import _ from "lodash";
import { Socket, ManagerOptions, SocketOptions, io } from "socket.io-client";
import { deserializePocoObject, PocoObject, serializePocoObject } from "../protocol";
import { PocoConnection, PocoConnectionEvents, PocoPeerConnection } from "./connection";
import { PocoConnectionError } from "./error";
import { EventsMap, DefaultEventsMap, EventNames, EventParameter } from "./event";
import { Address, PocoPeerSocketIOConnectionOptions } from "./types";

export interface PocoSocketIOConnectionEvents extends PocoConnectionEvents { };

export class PocoSocketIOConnection
    <
        ListenEvents extends EventsMap = DefaultEventsMap,
        EmitEvents extends EventsMap = ListenEvents,
    >
    extends PocoConnection
    <
        PocoSocketIOConnectionEvents & ListenEvents,
        PocoSocketIOConnectionEvents & EmitEvents
    > {

    private socket: Socket;
    private connectRejectCallback: ((reason: any) => void) | null;
    private connectResolveCallback: ((value: void | PromiseLike<void>) => void) | null;

    constructor(localAddress: Address, opts?: Partial<ManagerOptions & SocketOptions & { uri?: string }> | undefined) {
        super("socketIO", localAddress)

        const defaultOpts = {
            autoConnect: false,
            transports: ["websocket"],
            protocols: [__POCO_PROTOCOL_VERSION__],
            auth: { address: localAddress }
        };

        this.connectRejectCallback = null;
        this.connectResolveCallback = null;

        if (opts === undefined) {
            this.socket = io(defaultOpts);
        } else if (opts.uri === undefined) {
            this.socket = io(_.defaults(opts, defaultOpts));
        } else {
            this.socket = io(opts.uri, _.defaults(opts, defaultOpts))
        }

        this.socket.on("connect", () => {
            this.setStatus("connected")

            if (this.connectResolveCallback) {
                const callback = this.connectResolveCallback;
                this.connectResolveCallback = null;

                callback();
            }
        })

        this.socket.on("disconnect", (reason: string) => {
            if (reason === "io server disconnect" || reason === "io client disconnect") {
                this.setStatus("closed")
                return;
            }

            this.setStatus("failed")
        })

        this.socket.on("connect_error", (error: Error) => {
            if (this.connectRejectCallback) {
                const callback = this.connectRejectCallback;
                this.connectRejectCallback = null;

                callback(error);

                this.setStatus("failed")
            } else {
                this.setStatus("failed")

                throw error;
            }
        })

        this.socket.on("message", (buffer: ArrayBuffer) => {
            const payload = deserializePocoObject(buffer);

            this.onMessage(payload)
        })

        this.socket.onAny((event, ...args) => {
            this.triggerEvent(event, deserializePocoObject(args[0] as any))
        })

        this.on("error", ({ error }: { error: string }) => {
            this.setStatus("failed");
            this.socket.close();

            console.error("connection error:", error)

            throw new PocoConnectionError(this as any, error);
        })

    }

    async connect(): Promise<void> {
        this.setStatus("connecting")

        return new Promise((resolve, reject) => {
            this.connectResolveCallback = resolve;
            this.connectRejectCallback = reject;

            this.socket.connect();
        })
    }

    async disconnect(): Promise<void> {
        this.socket.disconnect();
    }

    send(payload: PocoObject): void {
        const buffer = serializePocoObject(payload);

        this.socket.send(buffer);
    }

    emit<Event extends EventNames<PocoSocketIOConnectionEvents & EmitEvents & PocoConnectionEvents>,
        Payload extends EventParameter<PocoSocketIOConnectionEvents & EmitEvents & PocoConnectionEvents, Event> = EventParameter<PocoSocketIOConnectionEvents & EmitEvents & PocoConnectionEvents, Event>>
        (event: Event, payload: Payload): void | Promise<void> {
        const buffer = serializePocoObject(payload as PocoObject);

        this.socket.emit(event as string, buffer)
    }
}

export type PocoPeerAddressPayload = { from: Address, to: Address };

export interface PocoPeerSocketIOConnectionEvents
    <Events extends EventsMap = DefaultEventsMap>
    extends PocoConnectionEvents {
    "peer message": (this: PocoPeerSocketIOConnection<any>, args: PocoPeerAddressPayload & { message: PocoObject }) => void;
    "peer event": <Event extends EventNames<Events>>(this: PocoPeerSocketIOConnection<any>, args: PocoPeerAddressPayload & { event: Event, payload: EventParameter<Events, Event> }) => void;
    "peer disconnected": (this: PocoPeerSocketIOConnection<any>, args: PocoPeerAddressPayload) => void;
    "peer connected": (this: PocoPeerSocketIOConnection<any>, args: PocoPeerAddressPayload) => void;
    "peer setup": (this: PocoPeerSocketIOConnection<any>, args: PocoPeerAddressPayload) => void;
    "peer destroy": (this: PocoPeerSocketIOConnection<any>, args: PocoPeerAddressPayload) => void;
};

export class PocoPeerSocketIOConnection<
    Events extends EventsMap = DefaultEventsMap
> extends PocoPeerConnection<Events> {

    private connection: PocoConnection<PocoPeerSocketIOConnectionEvents<Events>>;
    private options: PocoPeerSocketIOConnectionOptions | undefined;

    constructor(
        localAddress: Address,
        remoteAddress: Address,
        connection: PocoConnection<PocoPeerSocketIOConnectionEvents<Events>>,
        opts?: PocoPeerSocketIOConnectionOptions
    ) {

        super("socketIO", localAddress, remoteAddress);

        this.connection = connection;
        this.options = opts;

        this.connection.on("peer message", ({ from, to, message }) => {
            if (from !== this.remoteAddress || to !== this.localAddress) {
                return;
            }

            this.onMessage(message)
        })

        this.connection.on("peer event", ({ from, to, event, payload }) => {
            if (from !== this.remoteAddress || to !== this.localAddress) {
                return;
            }

            this.triggerEvent(event, payload)
        })

        this.connection.once("peer disconnected", () => {
            this.setStatus("closed")
        })

        // this.connection.once("peer connected", () => {
        //     this.setStatus("connected")
        // })
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
                resolve("timeout")
            }, this.options?.timeout || 5000)),
            new Promise<string>(resolve => {
                this.connection.once("peer connected", () => resolve("connected"));

                this.setStatus("connecting")
                this.connection.emit("peer setup", { from: this.localAddress, to: this.remoteAddress })
            })
        ])

        if (status === "connected") {
            this.setStatus("connected");
        } else {
            this.setStatus("failed");

            throw new PocoConnectionError(this, status);
        }
    }

    async disconnect(): Promise<void> {
        if (this.connectionStatus == "disconnected") {
            return;
        }

        this.connection.emit("peer destroy", { from: this.localAddress, to: this.remoteAddress })
        this.setStatus("closed")
    }

    send(payload: PocoObject): void | Promise<void> {
        this.connection.emit("peer message", {
            from: this.localAddress,
            to: this.remoteAddress,
            message: payload
        });
    }

    emit<Event extends EventNames<Events>, Payload extends EventParameter<Events, Event> = EventParameter<Events, Event>>(event: Event, payload: Payload): void | Promise<void> {
        this.connection.emit("peer event", {
            from: this.localAddress,
            to: this.remoteAddress,
            event: event as any,
            payload: payload
        })
    }
}