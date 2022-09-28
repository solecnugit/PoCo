export interface EventsMap {
    [event: string]: any;
}

export interface DefaultEventsMap {
    [event: string]: (...args: any[]) => Promise<void> | void;
}

export type EventNames<T extends EventsMap>
    = keyof T & (string | symbol);

export type EventHandler<
    T extends EventsMap,
    E extends EventNames<T>>
    = T[E];

export type EventHandlers<T extends EventsMap>
    = T[keyof T]

export type EventParameter<T extends EventsMap, E extends EventNames<T>> = Parameters<EventHandler<T, E>>[0]

export type EventParameters<T extends EventsMap> = EventParameter<T, EventNames<T>>