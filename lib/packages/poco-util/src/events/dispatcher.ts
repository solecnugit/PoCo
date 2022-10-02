import { DefaultEventsMap, EventHandler, EventHandlers, EventNames, EventParameter, EventParameters, EventsMap } from "./types";
import deepEquals from "fast-deep-equal/es6"

export type ListenerOptions = {
    async: boolean
}

export type OnceListenerOptions = ListenerOptions & {
    signal: AbortSignal | undefined
}

export class EventDispatcher<Events extends EventsMap = DefaultEventsMap> {
    protected _listenerCount: number;

    protected _listeners: Map<EventNames<Events>, {
        callback: EventHandlers<Events>,
        option: ListenerOptions
    }[]>;

    protected _onceListeners: Map<EventNames<Events>, {
        resolveCallback: (...args: any) => void,
        rejectCallback: (reason?: any) => void,
        option: OnceListenerOptions
    }[]>;

    constructor() {
        this._listenerCount = 0;
        this._listeners = new Map();
        this._onceListeners = new Map();
    }

    protected addListener
        <Event extends EventNames<Events>,
            Callback extends EventHandlers<Events> = EventHandler<Events, Event>>
        (event: Event, callback: Callback, opt?: Partial<ListenerOptions>): boolean {

        const option = { async: opt?.async || false };
        const listeners = this._listeners.get(event);

        if (!listeners) {
            this._listeners.set(event, [{
                callback,
                option
            }]);

            return true;
        }

        if (listeners.find(e => e.callback == callback && deepEquals(e.option, option)))
            return false;

        listeners.push({
            callback,
            option
        })

        this._listenerCount += 1;

        return true;
    }

    protected addOnceListener<Event extends EventNames<Events>>
        (
            event: Event,
            resolveCallback: (...args: EventParameter<Events, Event>) => void,
            rejectCallback: (reason?: any) => void,
            option: OnceListenerOptions
        ) {

        const listeners = this._onceListeners.get(event);

        if (!listeners) {
            this._onceListeners.set(event, [{
                resolveCallback,
                rejectCallback,
                option
            }]);

            return;
        }

        listeners.push({
            resolveCallback,
            rejectCallback,
            option
        })

        this._listenerCount += 1;
    }

    protected removeListener
        <Event extends EventNames<Events>,
            Callback extends EventHandlers<Events> = EventHandler<Events, Event>>
        (event: Event, callback: Callback, opt?: Partial<ListenerOptions>): boolean {

        // @ts-ignore
        const option = { async: opt?.async || false };


        const listeners = this._listeners.get(event);

        if (!listeners)
            return false;

        const index = listeners.findIndex(e => e.callback == callback && deepEquals(e.option, option));

        if (index < 0)
            return false;

        listeners.splice(index, 1)

        if (listeners.length === 0)
            this._listeners.delete(event)

        this._listenerCount -= 1;

        return true;
    }

    protected triggerEvent
        <Event extends EventNames<Events>,
            Parameters extends EventParameters<Events> = EventParameter<Events, Event>>
        (event: Event, args: Parameters) {

        let listeners = this._listeners.get(event)?.slice();

        if (listeners) {
            for (const { callback, option: { async } } of listeners) {
                if (async) {
                    setImmediate(() => {
                        callback.apply(this, args)
                    })
                } else {
                    callback.apply(this, args)
                }
            }
        }

        let onceListeners = this._onceListeners.get(event)?.slice();

        if (onceListeners) {
            for (const { resolveCallback, rejectCallback, option: { async, signal } } of onceListeners) {
                if (async) {
                    if (signal && signal.aborted) {
                        setImmediate(() => {
                            rejectCallback.apply(this, ["abort"])
                        })
                    } else {
                        setImmediate(() => {
                            resolveCallback.apply(this, [args])
                        })
                    }
                } else {
                    if (signal && signal.aborted) {
                        rejectCallback.apply(this, ["abort"])
                    } else {
                        resolveCallback.apply(this, [args])
                    }
                }
            }

            this._onceListeners.delete(event)
        }
    }

    protected removeAllListeners<Event extends EventNames<Events>>(event?: Event) {
        if (!event) {
            this._listeners.clear();
            return;
        }

        this._listeners.delete(event);
    }

    on
        <Event extends EventNames<Events>,
            Callback extends EventHandlers<Events> = EventHandler<Events, Event>>
        (event: Event, callback: Callback, opt?: Partial<ListenerOptions>): boolean {
        return this.addListener(event, callback, opt)
    }

    once
        <Event extends EventNames<Events>>
        (event: Event, opt?: Partial<Pick<OnceListenerOptions, "async" | "signal">>): Promise<EventParameter<Events, Event>> {

        return new Promise((resolve, reject) => {
            this.addOnceListener(event, resolve as any, reject, {
                async: opt?.async || false,
                signal: opt?.signal,
            })
        })
    }

    off
        <Event extends EventNames<Events>,
            Callback extends EventHandlers<Events> = EventHandler<Events, Event>>
        (event: Event, callback: Callback, opt?: Partial<ListenerOptions>): boolean {
        return this.removeListener(event, callback, opt)
    }

    emit
        <Event extends EventNames<Events>,
            Parameters extends EventParameters<Events> = EventParameter<Events, Event>>
        (event: Event, args: Parameters) {
        this.triggerEvent(event, args);
    }

    clear
        <Event extends EventNames<Events>>(event?: Event) {
        this.removeAllListeners(event);
    }

    listenerCount<Event extends EventNames<Events>>(event?: Event): number {
        if (!event)
            return this._listenerCount;

        return this._listeners.get(event)?.length || 0;
    }

    listeners<Event extends EventNames<Events>>(event?: Event): {
        callback: EventHandlers<Events>,
        option: ListenerOptions
    }[] {
        if (!event) {
            return Array.from(this._listeners.values()).flatMap(e => e);
        }

        return this._listeners.get(event) || []
    }

    eventNames(): EventNames<Events>[] {
        return Array.from(this._listeners.keys())
    }
}

