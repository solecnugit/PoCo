import { PocoMessage } from "../protocol";
import { EventsMap } from "../util";
import { PocoConnection } from "./connection";

export class PocoConnectionUnknownTypeError extends Error {
    public type: string;

    constructor(type: string) {
        super(`unknown connection type ${type}`);
        this.type = type;
    }
}

export class PocoConnectionClosedError
    <M extends PocoMessage, T extends EventsMap, R extends EventsMap>
    extends Error {
    public connection: PocoConnection<M, T, R>;

    constructor(connection: PocoConnection<M, T, R>) {
        super(`connection is closed`)

        this.connection = connection;
    }
}

export class PocoConnectionTimeoutError
    <M extends PocoMessage, T extends EventsMap, R extends EventsMap>
    extends Error {
    public connection: PocoConnection<M, T, R>;

    constructor(connection: PocoConnection<M, T, R>) {
        super(`connection timeout`)

        this.connection = connection;
    }
}

export class PocoConnectionInvalidProtoclError
    <M extends PocoMessage, T extends EventsMap, R extends EventsMap>
    extends Error {
    public connection: PocoConnection<M, T, R>;

    constructor(connection: PocoConnection<M, T, R>, error: string) {
        super(error)

        this.connection = connection;
    }
}