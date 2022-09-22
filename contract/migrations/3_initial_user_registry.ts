import { deployProxy } from "@openzeppelin/truffle-upgrades";
import { ContractClass, Deployer } from "@openzeppelin/truffle-upgrades/dist/utils";
import { UserRegistryInstance } from "../types/truffle-contracts";
import chalk from "chalk";

const UserRegistry = artifacts.require("UserRegistry");

module.exports = async function (deployer: Truffle.Deployer & Deployer) {
    const instance = (await deployProxy(
        UserRegistry as unknown as ContractClass,
        [],
        {
            deployer,
            unsafeAllow: [],
        }
    )) as UserRegistryInstance;

    console.log(
        "Contract",
        chalk.bold.bgCyan("UserRegistry"),
        "deployed to",
        chalk.bgGreen(instance.address),
        ".");
};