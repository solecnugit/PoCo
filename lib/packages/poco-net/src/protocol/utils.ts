import { serialize, deserialize } from "bson"
import _ from "lodash"
import { PocoMessagePayload, PocoObject } from "./types"
import { Buffer } from "buffer";

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
    const payload = Object.values(deserialize(buffer));

    for (let i = 0; i < payload.length; i++) {
        if (isBinary2Type(payload[i])) {
            payload[i] = payload[i].buffer;
        }
    }

    return payload;
}