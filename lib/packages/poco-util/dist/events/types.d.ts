export interface EventsMap {
    [event: string]: any;
}
export interface DefaultEventsMap {
    [event: string]: (this: ThisType<any>, ...args: any[]) => void;
}
export declare type EventNames<T extends EventsMap> = keyof T & (string | symbol);
export declare type EventHandler<T extends EventsMap, R extends EventNames<T>> = OmitThisParameter<T[R]>;
export declare type EventHandlers<T extends EventsMap> = OmitThisParameter<T[EventNames<T>]>;
export declare type EventParameters<T extends EventsMap> = Parameters<EventHandlers<T>>;
export declare type EventParameter<T extends EventsMap, R extends EventNames<T>> = Parameters<EventHandler<T, R>>;
//# sourceMappingURL=types.d.ts.map