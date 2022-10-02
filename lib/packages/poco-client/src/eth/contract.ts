import contract from "@truffle/contract";
import * as ContractAddress from "poco-contract/contracts.json";
import { EitherOr } from "../utils";
import { provider } from "web3-core";

export type Networks = "development";
export type ContractNames = "JobCenter" | "ServiceRegistry"
export type ContractOptions = EitherOr<{ network: Networks, address: string }, "network", "address">

export type PocoContractInstance<Contract extends Truffle.ContractInstance> = Contract & {

}

export async function getContractInstance
    <Contract extends Truffle.ContractInstance>(
        provider: provider,
        abi: object,
        name: ContractNames,
        { network, address }: ContractOptions)
    : Promise<PocoContractInstance<Contract>> {
    const func: any = (typeof TruffleContract !== undefined) ? TruffleContract : contract;

    const instance = func(abi);

    instance.setProvider(provider);

    if (address) {
        return instance.at(address)
    }

    if (network) {
        return instance.at(ContractAddress[network][name])
    }

    throw new Error("unreachable")
}