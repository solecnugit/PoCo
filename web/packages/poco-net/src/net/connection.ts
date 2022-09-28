import _ from "lodash";
import { PocoMessage } from "../protocol";
import { EventHandler, EventHandlers, EventNames, EventParameter, EventsMap } from "../util";
import { PocoConnectionType, Address, PocoConnectionStatus } from "./types";

export type PocoConnectionDisconnectedReason = "user closed" | "invalid protocol" | "missing address" | "duplicate address";

export interface PocoConnectionEvents {
    status: (this: PocoConnection, args: { status: PocoConnectionStatus }) => void;
    connected: (this: PocoConnection, args: {}) => void;
    disconnected: (this: PocoConnection, args: { reason: PocoConnectionDisconnectedReason }) => void;
}

export abstract class PocoConnection<
    MessagePayload extends PocoMessage = PocoMessage,
    ListenEvents extends EventsMap = PocoConnectionEvents,
    EmitEvents extends EventsMap = ListenEvents
> {

    public connectionType: PocoConnectionType;
    public localAddress: Address;

    protected connectionStatus: PocoConnectionStatus;
    protected listeners: Map<EventNames<ListenEvents>, {
        callback: EventHandlers<ListenEvents>,
        once: boolean
    }[]>;

    constructor(connectionType: PocoConnectionType, localAddress: Address) {
        this.connectionType = connectionType;
        this.localAddress = localAddress;
        this.connectionStatus = "new";
        this.listeners = new Map();
    }

    protected addEventListener<Event extends EventNames<ListenEvents>,
        Callback extends EventHandlers<ListenEvents> = EventHandler<ListenEvents, Event>>
        (event: Event, callback: Callback, once: boolean = false) {
        const listeners = this.listeners.get(event);

        if (!listeners) {
            this.listeners.set(event, [{
                callback,
                once
            }]);

            return;
        }

        if (listeners.find(e => !e.once && e.callback == callback))
            return;

        listeners.push({
            callback,
            once
        })
    }

    protected removeEventListener<Event extends EventNames<ListenEvents>,
        Callback extends EventHandlers<ListenEvents> = EventHandler<ListenEvents, Event>>
        (event: Event, callback: Callback, includeOnce: boolean = false) {
        const listeners = this.listeners.get(event);

        if (!listeners)
            return;

        const index = listeners.findIndex(e => e.callback == callback && (!e.once || includeOnce));

        if (index < 0)
            return;

        listeners.splice(index, 1)
    }

    protected triggerEvent
        <Event extends EventNames<ListenEvents>,
            Parameters extends EventParameter<ListenEvents, Event>>
        (event: Event, args: Parameters) {

        console.log(event)

        const listeners = this.listeners.get(event)?.slice();

        if (!listeners)
            return;

        const newListeners = [];

        for (const handler of listeners) {
            handler.callback.apply(this, [args]);

            if (!handler.once)
                newListeners.push(handler);
        }

        this.listeners.set(event, newListeners);
    }

    abstract connect(): Promise<void>
    abstract disconnect(): Promise<void>

    public status(): PocoConnectionStatus {
        return this.connectionStatus;
    }

    protected setStatus(status: PocoConnectionStatus): void {
        if (status === this.connectionStatus) return;

        this.connectionStatus = status;

        this.triggerEvent("status", { status });

        if (this.connectionStatus === "connected") {
            this.triggerEvent("connected", {})
        } else if (this.connectionStatus === "disconnected") {
            this.triggerEvent("disconnected", {})
        }
    }

    once<Event extends EventNames<ListenEvents>,
        Callback extends EventHandlers<ListenEvents> = EventHandler<ListenEvents, Event>>
        (event: Event, callback: Callback) {
        this.addEventListener(event, callback, true);
    }

    on<Event extends EventNames<ListenEvents>,
        Callback extends EventHandlers<ListenEvents> = EventHandler<ListenEvents, Event>>
        (event: Event, callback: Callback, once: boolean = false) {
        this.addEventListener(event, callback, once);
    }

    off<Event extends EventNames<ListenEvents>,
        Callback extends EventHandlers<ListenEvents> = EventHandler<ListenEvents, Event>>
        (event: Event, callback: Callback, includeOnce: boolean = false) {
        this.removeEventListener(event, callback, includeOnce);
    }

    abstract send(payload: MessagePayload): Promise<void> | void;

    abstract emit<Event extends EventNames<EmitEvents>,
        Payload extends EventParameter<EmitEvents, Event> = EventParameter<EmitEvents, Event>>
        (event: Event, payload: Payload): Promise<void> | void;

    abstract onMessage(message: MessagePayload): Promise<void> | void;
}


export abstract class PocoPeerConnection<
    MessagePayload extends PocoMessage = PocoMessage,
    ListenEvents extends EventsMap = PocoConnectionEvents,
    EmitEvents extends EventsMap = ListenEvents,
> extends PocoConnection<MessagePayload, ListenEvents, EmitEvents> {

    public remoteAddress: Address;

    constructor(connectionType: PocoConnectionType, localAddress: Address, remoteAddress: Address) {
        super(connectionType, localAddress);

        this.remoteAddress = remoteAddress;
    }
}