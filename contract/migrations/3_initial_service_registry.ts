import { deployProxy } from "@openzeppelin/truffle-upgrades";
import {
  ContractClass,
  Deployer,
} from "@openzeppelin/truffle-upgrades/dist/utils";
import { ServiceRegistryInstance } from "../types/truffle-contracts";
import chalk from "chalk";

const ServiceRegistry = artifacts.require("ServiceRegistry");

module.exports = async function (deployer: Truffle.Deployer & Deployer) {
  const instance = (await deployProxy(
    ServiceRegistry as unknown as ContractClass,
    [],
    {
      deployer,
      unsafeAllow: [],
    }
  )) as ServiceRegistryInstance;

  console.log(
    "Contract",
    chalk.bold.bgCyan("ServiceRegistry"),
    "deployed to",
    chalk.bgGreen(instance.address),
    "."
  );
};
