import _ from "lodash";
import { PocoObject } from "../protocol";
import { EventsMap, Compose, DefaultEventsMap, EventNames, EventHandlers, EventHandler, EventParameter } from "./event";
import { PocoConnectionType, Address, PocoConnectionStatus } from "./types";

export type PocoConnectionClosedReason = "user closed" | "invalid protocol" | "missing address" | "duplicate address";

export interface PocoConnectionEvents extends EventsMap {
    status: (this: PocoConnection, args: { status: PocoConnectionStatus }) => void;
    connected: (this: PocoConnection, args: {}) => void;
    disconnected: (this: PocoConnection, args: { reason: PocoConnectionClosedReason }) => void;
    message: (this: PocoConnection, args: { message: PocoObject }) => void;
    error: (this: PocoConnection, args: { error: string }) => void;
}

type ReservedOrUserEvents<T extends EventsMap> = Compose<T, PocoConnectionEvents>

export abstract class PocoConnection<
    ListenEvents extends EventsMap = DefaultEventsMap,
    EmitEvents extends EventsMap = ListenEvents,
> {
    public connectionType: PocoConnectionType;
    public localAddress: Address;

    protected connectionStatus: PocoConnectionStatus;
    protected listeners: Map<EventNames<ReservedOrUserEvents<ListenEvents>>, {
        callback: EventHandlers<ReservedOrUserEvents<ListenEvents>>,
        once: boolean
    }[]>;


    constructor(connectionType: PocoConnectionType, localAddress: Address) {
        this.connectionType = connectionType;
        this.localAddress = localAddress;
        this.connectionStatus = "new";
        this.listeners = new Map();
    }

    protected addEventListener
        <Event extends EventNames<ReservedOrUserEvents<ListenEvents>>,
            Callback extends EventHandlers<ReservedOrUserEvents<ListenEvents>>>
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

    protected removeEventListener<Event extends EventNames<ReservedOrUserEvents<ListenEvents>>,
        Callback extends EventHandlers<ReservedOrUserEvents<ListenEvents>> = EventHandler<ReservedOrUserEvents<ListenEvents>, Event>>
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
        <Event extends EventNames<ReservedOrUserEvents<ListenEvents>>,
            Parameters extends EventParameter<ReservedOrUserEvents<ListenEvents>, Event>>
        (event: Event, args: Parameters) {
        debugger

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

        if (status === "failed") {
            this.disconnect()
        }
    }

    once<Event extends EventNames<ReservedOrUserEvents<ListenEvents>>,
        Callback extends EventHandlers<ReservedOrUserEvents<ListenEvents>> = EventHandler<ReservedOrUserEvents<ListenEvents>, Event>>
        (event: Event, callback: Callback) {
        this.addEventListener(event, callback, true);
    }

    on<Event extends EventNames<ReservedOrUserEvents<ListenEvents>>,
        Callback extends EventHandlers<ReservedOrUserEvents<ListenEvents>> = EventHandler<ReservedOrUserEvents<ListenEvents>, Event>>
        (event: Event, callback: Callback, once: boolean = false) {
        this.addEventListener(event, callback, once);
    }

    off<Event extends EventNames<ReservedOrUserEvents<ListenEvents>>,
        Callback extends EventHandlers<ReservedOrUserEvents<ListenEvents>> = EventHandler<ReservedOrUserEvents<ListenEvents>, Event>>
        (event: Event, callback: Callback, includeOnce: boolean = false) {
        this.removeEventListener(event, callback, includeOnce);
    }

    abstract send(payload: PocoObject): Promise<void> | void;

    abstract emit<Event extends EventNames<ReservedOrUserEvents<EmitEvents>>,
        Payload extends EventParameter<ReservedOrUserEvents<EmitEvents>, Event> = EventParameter<ReservedOrUserEvents<EmitEvents>, Event>>
        (event: Event, payload: Payload): Promise<void> | void;

    onMessage(message: PocoObject): Promise<void> | void {
        this.triggerEvent("message", { message });
    }
}

export abstract class PocoPeerConnection<
    Events extends EventsMap = PocoConnectionEvents,
> extends PocoConnection<Events, Events> {

    public remoteAddress: Address;

    constructor(connectionType: PocoConnectionType, localAddress: Address, remoteAddress: Address) {
        super(connectionType, localAddress);

        this.remoteAddress = remoteAddress;
    }
}