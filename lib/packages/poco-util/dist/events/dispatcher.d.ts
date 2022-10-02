/// <reference types="node" />
import { DefaultEventsMap, EventHandler, EventHandlers, EventNames, EventParameter, EventParameters, EventsMap } from "./types";
export declare type ListenerOptions = {
    async: boolean;
};
export declare type OnceListenerOptions = ListenerOptions & {
    signal: AbortSignal | undefined;
};
export declare class EventDispatcher<Events extends EventsMap = DefaultEventsMap> {
    protected _listenerCount: number;
    protected _listeners: Map<EventNames<Events>, {
        callback: EventHandlers<Events>;
        option: ListenerOptions;
    }[]>;
    protected _onceListeners: Map<EventNames<Events>, {
        resolveCallback: (...args: any) => void;
        rejectCallback: (reason?: any) => void;
        option: OnceListenerOptions;
    }[]>;
    constructor();
    protected addListener<Event extends EventNames<Events>, Callback extends EventHandlers<Events> = EventHandler<Events, Event>>(event: Event, callback: Callback, opt?: Partial<ListenerOptions>): boolean;
    protected addOnceListener<Event extends EventNames<Events>>(event: Event, resolveCallback: (...args: EventParameter<Events, Event>) => void, rejectCallback: (reason?: any) => void, option: OnceListenerOptions): void;
    protected removeListener<Event extends EventNames<Events>, Callback extends EventHandlers<Events> = EventHandler<Events, Event>>(event: Event, callback: Callback, opt?: Partial<ListenerOptions>): boolean;
    protected triggerEvent<Event extends EventNames<Events>, Parameters extends EventParameters<Events> = EventParameter<Events, Event>>(event: Event, args: Parameters): void;
    protected removeAllListeners<Event extends EventNames<Events>>(event?: Event): void;
    on<Event extends EventNames<Events>, Callback extends EventHandlers<Events> = EventHandler<Events, Event>>(event: Event, callback: Callback, opt?: Partial<ListenerOptions>): boolean;
    once<Event extends EventNames<Events>>(event: Event, opt?: Partial<Pick<OnceListenerOptions, "async" | "signal">>): Promise<EventParameter<Events, Event>>;
    off<Event extends EventNames<Events>, Callback extends EventHandlers<Events> = EventHandler<Events, Event>>(event: Event, callback: Callback, opt?: Partial<ListenerOptions>): boolean;
    emit<Event extends EventNames<Events>, Parameters extends EventParameters<Events> = EventParameter<Events, Event>>(event: Event, args: Parameters): void;
    clear<Event extends EventNames<Events>>(event?: Event): void;
    listenerCount<Event extends EventNames<Events>>(event?: Event): number;
    listeners<Event extends EventNames<Events>>(event?: Event): {
        callback: EventHandlers<Events>;
        option: ListenerOptions;
    }[];
    eventNames(): EventNames<Events>[];
}
//# sourceMappingURL=dispatcher.d.ts.map