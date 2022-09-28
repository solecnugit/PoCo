import _ from "lodash";
import { Socket, ManagerOptions, SocketOptions, io } from "socket.io-client";
import { deserializePocoMessage, deserializePocoObject, PocoMessage, PocoObject, serializePocoObject } from "../protocol";
import { DefaultEventsMap, EventNames, EventParameter, EventsMap } from "../util";
import { PocoConnection, PocoConnectionEvents, PocoPeerConnection } from "./connection";
import { PocoConnectionInvalidProtoclError, PocoConnectionTimeoutError } from "./error";
import { Address, PocoPeerSocketIOConnectionOptions } from "./types";

export interface PocoSocketIOConnectionEvents
    <T extends PocoMessage = PocoObject>
    extends PocoConnectionEvents {
    message: (this: PocoSocketIOConnection, args: { message: T }) => void;
    error: (this: PocoSocketIOConnection, args: { error: string }) => never;
}

export class PocoSocketIOConnection<
    MessagePayload extends PocoMessage = PocoMessage,
    ListenEvents extends EventsMap = DefaultEventsMap,
    EmitEvents extends EventsMap = ListenEvents,
> extends PocoConnection<
    MessagePayload,
    PocoSocketIOConnectionEvents<MessagePayload>> {

    private socket: Socket;

    constructor(localAddress: Address, opts?: Partial<ManagerOptions & SocketOptions & { uri?: string }> | undefined) {
        super("socketIO", localAddress)

        const defaultOpts = {
            autoConnect: false,
            transports: ["websocket"],
            protocols: [__POCO_PROTOCOL_VERSION__],
            auth: { address: localAddress }
        };

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

        this.socket.on("connect_error", (error: Error) => {
            this.setStatus("disconnected")

            throw error;
        })

        this.socket.on("message", (buffer: ArrayBuffer) => {
            const payload = deserializePocoObject(buffer) as MessagePayload;

            this.onMessage(payload)
        })

        this.socket.onAny((event, ...args) => {
            this.triggerEvent(event, deserializePocoMessage(args[0] as any))
        })

        this.on("error", ({ error }) => {
            this.setStatus("failed");

            throw new PocoConnectionInvalidProtoclError(this, error);
        })
    }

    async connect(): Promise<void> {
        this.setStatus("connecting")

        this.socket.connect()
    }

    async disconnect(): Promise<void> {
        this.socket.disconnect();
    }

    send(payload: PocoObject): void {
        const buffer = serializePocoObject(payload);

        this.socket.send(buffer);
    }

    emit<Event extends EventNames<EmitEvents>, Payload extends EventParameter<EmitEvents, Event> = EventParameter<EmitEvents, Event>>(event: Event, payload: Payload): void | Promise<void> {
        const buffer = serializePocoObject(payload as PocoObject);

        this.socket.emit(event as string, buffer)
    }

    onMessage(message: MessagePayload): void | Promise<void> {
        this.triggerEvent("message", { message })
    }
}

export type PocoPeerAddressPayload = { from: Address, to: Address };

export interface PocoPeerSocketIOConnectionEvents
    <T extends PocoMessage = PocoObject,
        Events extends EventsMap = DefaultEventsMap>
    extends PocoConnectionEvents {
    "peer message": (this: PocoPeerSocketIOConnection, args: PocoPeerAddressPayload & { message: T }) => void;
    "peer event": <Event extends EventNames<Events>>(this: PocoPeerSocketIOConnection, args: PocoPeerAddressPayload & { event: Event, payload: EventParameter<Events, Event> }) => void;
    "peer disconnected": (this: PocoPeerSocketIOConnection, args: PocoPeerAddressPayload) => void;
    "peer connected": (this: PocoPeerSocketIOConnection, args: PocoPeerAddressPayload) => void;
    "peer setup": (this: PocoPeerSocketIOConnection, args: PocoPeerAddressPayload) => void;
    "peer destroy": (this: PocoPeerSocketIOConnection, args: PocoPeerAddressPayload) => void;
};

export class PocoPeerSocketIOConnection<
    MessagePayload extends PocoMessage = PocoMessage,
    Events extends EventsMap = DefaultEventsMap
> extends PocoPeerConnection<MessagePayload, Events, Events> {

    private connection: PocoConnection<MessagePayload,
        PocoPeerSocketIOConnectionEvents<MessagePayload, Events>>;
    private options: PocoPeerSocketIOConnectionOptions | undefined;

    constructor(localAddress: Address, remoteAddress: Address, connection: PocoConnection<MessagePayload,
        PocoPeerSocketIOConnectionEvents<MessagePayload, Events>>, opts?: PocoPeerSocketIOConnectionOptions) {

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

            throw new PocoConnectionTimeoutError(this);
        }
    }

    async disconnect(): Promise<void> {
        if (this.connectionStatus == "disconnected") {
            return;
        }

        this.connection.emit("peer destroy", { from: this.localAddress, to: this.remoteAddress })
        this.setStatus("closed")
    }

    send(payload: MessagePayload): void | Promise<void> {
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
            event: event,
            payload: payload
        })
    }

    onMessage(message: MessagePayload): void | Promise<void> {
        this.triggerEvent("message", message)
    }
}