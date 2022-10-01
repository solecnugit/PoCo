import { PocoConnection } from "./connection";
import { EventsMap, DefaultEventsMap } from "./event";

export class PocoConnectionError
    <
        T extends EventsMap = DefaultEventsMap,
        R extends EventsMap = T
    >
    extends Error {

    public connection: PocoConnection<T, R>;

    constructor(connection: PocoConnection<T, R>, error: string) {
        super(error)

        this.connection = connection;
    }
}

export class PocoConnectionUnknownTypeError extends PocoConnectionError {
    public type: string;

    constructor(type: string) {
        super(null as any, `unknown connection type ${type}`);

        this.type = type;
    }
}

export class PocoConnectionClosedError
    <
        T extends EventsMap = DefaultEventsMap,
        R extends EventsMap = T
    >
    extends PocoConnectionError<T, R> {

    constructor(connection: PocoConnection<T, R>) {
        super(connection, `connection is closed`)
    }
}

export class PocoConnectionTimeoutError
    <
        T extends EventsMap = DefaultEventsMap,
        R extends EventsMap = T
    >
    extends PocoConnectionError<T, R> {

    constructor(connection: PocoConnection<T, R>) {
        super(connection, `connection timeout`)
    }
}