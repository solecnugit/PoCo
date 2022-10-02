class EventDispatcher {
    _listenerCount;
    _listeners;
    _onceListeners;
    constructor() {
        this._listenerCount = 0;
        this._listeners = new Map();
        this._onceListeners = new Map();
    }
    addListener(event, callback, opt) {
        const option = { async: opt?.async || false };
        const listeners = this._listeners.get(event);
        if (!listeners) {
            this._listeners.set(event, [{
                    callback,
                    option
                }]);
            return true;
        }
        if (listeners.find(e => e.callback == callback && e.option === option))
            return false;
        listeners.push({
            callback,
            option
        });
        this._listenerCount += 1;
        return true;
    }
    addOnceListener(event, resolveCallback, rejectCallback, option) {
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
        });
        this._listenerCount += 1;
    }
    removeListener(event, callback, opt) {
        const option = { async: opt?.async || false };
        const listeners = this._listeners.get(event);
        if (!listeners)
            return false;
        const index = listeners.findIndex(e => e.callback == callback && e.option === option);
        if (index < 0)
            return false;
        listeners.splice(index, 1);
        if (listeners.length === 0)
            this._listeners.delete(event);
        this._listenerCount -= 1;
        return true;
    }
    triggerEvent(event, args) {
        let listeners = this._listeners.get(event)?.slice();
        if (listeners) {
            for (const { callback, option: { async } } of listeners) {
                if (async) {
                    setImmediate(() => {
                        callback.apply(this, args);
                    });
                }
                else {
                    callback.apply(this, args);
                }
            }
        }
        let onceListeners = this._onceListeners.get(event)?.slice();
        if (onceListeners) {
            for (const { resolveCallback, rejectCallback, option: { async, signal } } of onceListeners) {
                if (async) {
                    if (signal && signal.aborted) {
                        setImmediate(() => {
                            rejectCallback.apply(this, ["abort"]);
                        });
                    }
                    else {
                        setImmediate(() => {
                            resolveCallback.apply(this, args);
                        });
                    }
                }
                else {
                    if (signal && signal.aborted) {
                        rejectCallback.apply(this, ["abort"]);
                    }
                    else {
                        resolveCallback.apply(this, args);
                    }
                }
            }
            this._onceListeners.delete(event);
        }
    }
    removeAllListeners(event) {
        if (!event) {
            this._listeners.clear();
            return;
        }
        this._listeners.delete(event);
    }
    on(event, callback, opt) {
        return this.addListener(event, callback, opt);
    }
    once(event, opt) {
        return new Promise((resolve, reject) => {
            this.addOnceListener(event, resolve, reject, {
                async: opt?.async || false,
                signal: opt?.signal,
            });
        });
    }
    off(event, callback, opt) {
        return this.removeListener(event, callback, opt);
    }
    emit(event, args) {
        this.triggerEvent(event, args);
    }
    clear(event) {
        this.removeAllListeners(event);
    }
    listenerCount(event) {
        if (!event)
            return this._listenerCount;
        return this._listeners.get(event)?.length || 0;
    }
    listeners(event) {
        if (!event) {
            return Array.from(this._listeners.values()).flatMap(e => e);
        }
        return this._listeners.get(event) || [];
    }
    eventNames() {
        return Array.from(this._listeners.keys());
    }
}

export { EventDispatcher };
//# sourceMappingURL=index.esm.js.map
