import { ethers } from 'ethers';
import { Networks } from '../eth';
import { PocoClient } from './client';

export async function createPocoClient(network?: Networks): Promise<PocoClient> {
    // const provider = await detectEthereumProvider({
    //     mustBeMetaMask: true
    // }) as any;

    const web3Provider = new ethers.providers.Web3Provider((window as any).ethereum);
    const [account] = await web3Provider.send("eth_requestAccounts", []);

    return new PocoClient(web3Provider, ethers.utils.getAddress(account), network);
}

export async function sha256Digest(buffer: ArrayBuffer): Promise<string> {
    const hashBuffer = await crypto.subtle.digest('SHA-256', buffer);

    const hashArray = Array.from(new Uint8Array(hashBuffer));

    const hashHex = hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');

    return hashHex;
}