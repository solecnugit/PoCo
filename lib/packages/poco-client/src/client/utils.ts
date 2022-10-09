import { ethers } from "ethers";
import { Networks } from "../eth";
import { PocoClient } from "./client";

export async function createPocoClient(
  network?: Networks
): Promise<PocoClient> {
  // const provider = await detectEthereumProvider({
  //     mustBeMetaMask: true
  // }) as any;

  const web3Provider = new ethers.providers.Web3Provider(
    (window as any).ethereum
  );
  const [account] = await web3Provider.send("eth_requestAccounts", []);

  return new PocoClient(
    web3Provider,
    ethers.utils.getAddress(account),
    network
  );
}
