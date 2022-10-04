import { serialize, deserialize } from "bson"
import { PocoMessagePayload, PocoObject } from "./types"

export function serializePocoObject(object: PocoObject): ArrayBuffer {
    return serialize(object)
}

export function deserializePocoObject(buffer: ArrayBuffer): PocoObject {
    return deserialize(buffer)
}

export function serializePocoMessagePayload(payload: PocoMessagePayload): ArrayBuffer {
    return serialize(payload)
}

export function deserializeMessagePayload(buffer: ArrayBuffer): PocoMessagePayload {
    return Object.values(deserialize(buffer))
}