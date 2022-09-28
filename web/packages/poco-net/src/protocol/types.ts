export type PocoObjectElement =
    null
    | number
    | string
    | PocoObjectElement[]
    | ArrayBuffer
    | ArrayBufferView
    | PocoObject;

export type PocoObject = {
    [key: string]: PocoObjectElement
}

export type PocoMessage = PocoObject;

