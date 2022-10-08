import { serialize, deserialize } from "bson"
import _ from "lodash"
import { PocoMessagePayload, PocoObject } from "./types"
import { Buffer } from "buffer";
import { PACKET_HEADER_LENGTH_IN_BYTES, PocoProtocolPacket } from "./packet";

export function serializePocoObject(object: PocoObject): ArrayBuffer {
    return serialize(object)
}

export function deserializePocoObject(buffer: ArrayBuffer): PocoObject {
    return deserialize(buffer)
}

export function serializePocoMessagePayload(payload: PocoMessagePayload): ArrayBuffer {
    for (let i = 0; i < payload.length; i++) {
        if (_.isArrayBuffer(payload[i])) {
            payload[i] = Buffer.from(payload[i]);
        }
    }

    const buffer = serialize(payload);

    return buffer.buffer.slice(
        buffer.byteOffset, buffer.byteOffset + buffer.byteLength
    )
}

export function isBinary2Type(obj: any): boolean {
    return obj.__proto__.constructor.name === "Binary2"
}

export function deserializeMessagePayload(buffer: ArrayBuffer): PocoMessagePayload {
    const payload = Object.values(deserialize(buffer, {
        allowObjectSmallerThanBufferSize: true
    }));

    for (let i = 0; i < payload.length; i++) {
        if (isBinary2Type(payload[i])) {
            payload[i] = payload[i].buffer;
        }
    }

    return payload;
}

export function toPackets(buffer: ArrayBuffer, size: number): PocoProtocolPacket[] {
    const bodySize = size - PACKET_HEADER_LENGTH_IN_BYTES;

    if (buffer.byteLength <= bodySize) {
        const packet = new PocoProtocolPacket();

        packet.header().setNoMoreSegmentFlag();
        packet.header().setNoSegmentFlag();
        packet.setBody(buffer);

        packet.build();

        return [packet]
    }

    const packets = [];

    for (let begin = 0, end = bodySize; ; begin = end, end = end + bodySize) {
        const flag = end < buffer.byteLength;
        const packet = new PocoProtocolPacket();

        packet.setBody(buffer.slice(
            begin,
            Math.min(end, buffer.byteLength)
        ));

        if (flag) {
            packet.header().setMoreSegmentFlag();
        } else {
            packet.header().setNoMoreSegmentFlag();
        }

        packet.build()

        packets.push(packet);

        if (!flag) {
            break;
        }
    }

    return packets;
}