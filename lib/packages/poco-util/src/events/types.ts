export interface EventsMap {
    [event: string]: any;
}

export interface DefaultEventsMap {
    [event: string]: (this: ThisType<any>, ...args: any[]) => void;
}

export type EventNames<T extends EventsMap>
    = keyof T & (string | symbol)

export type EventHandler<T extends EventsMap, R extends EventNames<T>>
    = OmitThisParameter<T[R]>

export type EventHandlers<T extends EventsMap>
    = OmitThisParameter<T[EventNames<T>]>

export type EventParameters<T extends EventsMap>
    = Parameters<EventHandlers<T>>

export type EventParameter<T extends EventsMap, R extends EventNames<T>>
    = Parameters<EventHandler<T, R>>
