import { serialize, deserialize } from "bson"
import { PocoObject } from "./types"

export function serializePocoObject(object: PocoObject): ArrayBuffer {
    return serialize(object)
}

export function deserializePocoObject(buffer: ArrayBuffer): PocoObject {
    return deserialize(buffer)
}
