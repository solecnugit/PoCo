import { UserRegistryInstance } from "../types/truffle-contracts"
import web3 from "web3";
import chai from "chai";
import chainPromise from "chai-as-promised";
import { BN } from "bn.js";
import chaiBN from "chai-bn";

const { expect } = chai.use(chainPromise).use(chaiBN(BN));

const UserRegistry = artifacts.require("UserRegistry");
const ContractName = "UserRegistry";

contract(ContractName, (accounts) => {
    let instance: UserRegistryInstance;
    let owner: string;
    let user1: string;
    let user2: string;
    let randomString: string;

    before(async () => {
        instance = await UserRegistry.deployed();

        [owner, user1, user2] = accounts;

        randomString = (Math.random() * 1000).toString()
    });

    it("expect user can set their registry", async () => {
        const { logs } = await instance.setRecord({ endpoint: randomString }, {
            from: owner
        });

        {
            const { event, args } = logs[0];

            expect(event).to.be.equal("UserJoin");
            expect(args[0]).to.be.equal(owner);
            expect((args as any)[1][0] as string).to.be.equal(randomString);
        }

        {
            const { event, args } = logs[1];

            expect(event).to.be.equal("UserRegistryUpdate");
            expect(args[0]).to.be.equal(owner);
            expect((args as any)[1][0] as string).to.be.equal(randomString);
        }
    })

    it("expect user can get other's registry", async () => {
        const { endpoint } = await instance.getRecord(owner, {
            from: user1
        })

        expect(endpoint).to.be.equal(randomString);
    });
});