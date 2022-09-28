import { PocoMessage } from "../protocol";
import { DefaultEventsMap, EventsMap } from "../util";
import { PocoSocketIOConnection, PocoPeerSocketIOConnection } from "./socketIO";
import { PocoPeerSocketIOConnectionOptions, PocoPeerWebRTCConnectionOptions, PocoSocketIOConnectionOptions } from "./types";
import { PocoPeerWebRTCConnection } from "./webrtc";

export function createPocoSocketIOConnection
    <MessagePayload extends PocoMessage = PocoMessage,
        Events extends EventsMap = DefaultEventsMap>
    (opts: PocoSocketIOConnectionOptions): PocoSocketIOConnection<MessagePayload, Events> {
    return new PocoSocketIOConnection<MessagePayload, Events>(opts.localAddress, opts);
}

export function createPocoPeerSocketIOConnection
    <MessagePayload extends PocoMessage = PocoMessage,
        Events extends EventsMap = DefaultEventsMap>
    (opts: PocoPeerSocketIOConnectionOptions): PocoPeerSocketIOConnection<MessagePayload, Events> {
    return new PocoPeerSocketIOConnection<MessagePayload, Events>(opts.localAddress, opts.remoteAddress, opts.connection as any, opts);
}

export function createPocoPeerWebRTCConnection
    <MessagePayload extends PocoMessage = PocoMessage,
        Events extends EventsMap = DefaultEventsMap>
    (opts: PocoPeerWebRTCConnectionOptions): PocoPeerWebRTCConnection<MessagePayload, Events> {
    return new PocoPeerWebRTCConnection<MessagePayload, Events>(opts.localAddress, opts.remoteAddress, opts.connection as any, opts);
}