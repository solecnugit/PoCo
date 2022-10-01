declare module "poco-contract-abi/*.json"

declare module '@truffle/contract' {
    export default function contract<T>(json: object): Truffle.Contract<T>
}