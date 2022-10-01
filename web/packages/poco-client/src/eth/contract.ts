import { JobCenterInstance, ServiceRegistryInstance } from "poco-contract";
import contract from "@truffle/contract";

import JobCenterABI from "poco-contract-abi/JobCenter.json";
import ServiceRegistryABI from "poco-contract-abi/ServiceRegistry.json"

export async function getJobCenterContractInstance(address: string): Promise<JobCenterInstance> {
    return contract<JobCenterInstance>(JobCenterABI).at(address)
}

export async function getServiceRegistryInstance(address: string): Promise<ServiceRegistryInstance> {
    return contract<ServiceRegistryInstance>(ServiceRegistryABI).at(address)
}