import { describe, it } from "mocha";
import { expect } from "chai";
import { EventDispatcher } from "../src/index"

type Events = {
    "hello": (this: ThisType<EventDispatcher>, message: string) => void
}

describe("dispatcher", () => {
    let dispatcher: EventDispatcher<Events>;

    beforeEach(() => {
        dispatcher = new EventDispatcher();
    })

    it("#on() #emit()", async () => {
        let value = "world";

        dispatcher.on("hello", (data) => {
            expect(data).equal(value)
        })

        dispatcher.emit("hello", [value])
    })

    it("#on() #off()", async () => {
        const callback = () => { };

        expect(dispatcher.on("hello", callback)).to.be.true;
        expect(dispatcher.off("hello", callback)).to.be.true;
        expect(dispatcher.listeners("hello").length).equal(0)
    })

    it("#once()", async () => {
        const value = "world";

        setImmediate(() => {
            dispatcher.emit("hello", [value])
        })

        const [args] = await dispatcher.once("hello");

        expect(args).equal(value)
    })
})