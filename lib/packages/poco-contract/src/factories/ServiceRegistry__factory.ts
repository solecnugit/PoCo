/* Autogenerated file. Do not edit manually. */
/* tslint:disable */
/* eslint-disable */
import { Signer, utils, Contract, ContractFactory, Overrides } from "ethers";
import type { Provider, TransactionRequest } from "@ethersproject/providers";
import type { PromiseOrValue } from "../common";
import type {
  ServiceRegistry,
  ServiceRegistryInterface,
} from "../ServiceRegistry";

const _abi = [
  {
    anonymous: false,
    inputs: [
      {
        indexed: false,
        internalType: "uint8",
        name: "version",
        type: "uint8",
      },
    ],
    name: "Initialized",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: "enum ServiceRegistry.Role",
        name: "role",
        type: "uint8",
      },
      {
        indexed: true,
        internalType: "address",
        name: "provider",
        type: "address",
      },
      {
        indexed: false,
        internalType: "string",
        name: "endpoint",
        type: "string",
      },
    ],
    name: "NewService",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: "enum ServiceRegistry.Role",
        name: "role",
        type: "uint8",
      },
      {
        indexed: true,
        internalType: "address",
        name: "provider",
        type: "address",
      },
      {
        indexed: false,
        internalType: "string",
        name: "endpoint",
        type: "string",
      },
    ],
    name: "ServiceUpdate",
    type: "event",
  },
  {
    inputs: [],
    name: "initialize",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        components: [
          {
            internalType: "string",
            name: "endpoint",
            type: "string",
          },
          {
            internalType: "enum ServiceRegistry.Role",
            name: "role",
            type: "uint8",
          },
        ],
        internalType: "struct ServiceRegistry.Record",
        name: "record",
        type: "tuple",
      },
    ],
    name: "setRecord",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "user",
        type: "address",
      },
    ],
    name: "getRecord",
    outputs: [
      {
        components: [
          {
            internalType: "string",
            name: "endpoint",
            type: "string",
          },
          {
            internalType: "enum ServiceRegistry.Role",
            name: "role",
            type: "uint8",
          },
        ],
        internalType: "struct ServiceRegistry.Record",
        name: "",
        type: "tuple",
      },
    ],
    stateMutability: "view",
    type: "function",
    constant: true,
  },
  {
    inputs: [],
    name: "getUsers",
    outputs: [
      {
        internalType: "address[]",
        name: "",
        type: "address[]",
      },
    ],
    stateMutability: "view",
    type: "function",
    constant: true,
  },
];

const _bytecode =
  "0x608060405234801561001057600080fd5b50611033806100206000396000f3fe608060405234801561001057600080fd5b506004361061004b5760003560e01c8062ce8e3e14610050578063617fba041461006e57806377bcb7bc1461009e5780638129fc1c146100ba575b600080fd5b6100586100c4565b6040516100659190610714565b60405180910390f35b61008860048036038101906100839190610776565b6100d5565b60405161009591906108e7565b60405180910390f35b6100b860048036038101906100b39190610ad9565b6101fb565b005b6100c2610377565b005b60606100d060026104b5565b905090565b6100dd6105f9565b600160008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060405180604001604052908160008201805461013790610b51565b80601f016020809104026020016040519081016040528092919081815260200182805461016390610b51565b80156101b05780601f10610185576101008083540402835291602001916101b0565b820191906000526020600020905b81548152906001019060200180831161019357829003601f168201915b505050505081526020016001820160009054906101000a900460ff1660008111156101de576101dd610833565b5b60008111156101f0576101ef610833565b5b815250509050919050565b80600160003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008201518160000190816102509190610d38565b5060208201518160010160006101000a81548160ff0219169083600081111561027c5761027b610833565b5b02179055509050506102983360026104d690919063ffffffff16565b61030a573373ffffffffffffffffffffffffffffffffffffffff16816020015160008111156102ca576102c9610833565b5b7f20309915d5388e7ffbdf90aebe947775baed9026a9752d6ab873db9dac5f243d83600001516040516102fd9190610e54565b60405180910390a3610374565b3373ffffffffffffffffffffffffffffffffffffffff168160200151600081111561033857610337610833565b5b7f733a86b4159ceed337ac7cf47e99320d72e0e635c05e825aecff4e919037eccb836000015160405161036b9190610e54565b60405180910390a35b50565b60008060019054906101000a900460ff161590508080156103a85750600160008054906101000a900460ff1660ff16105b806103d557506103b730610506565b1580156103d45750600160008054906101000a900460ff1660ff16145b5b610414576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161040b90610ee8565b60405180910390fd5b60016000806101000a81548160ff021916908360ff1602179055508015610451576001600060016101000a81548160ff0219169083151502179055505b610459610529565b80156104b25760008060016101000a81548160ff0219169083151502179055507f7f26b83ff96e1f2b6a682f133852f6798a09c465da95921460cefb384740249860016040516104a99190610f50565b60405180910390a15b50565b606060006104c58360000161057a565b905060608190508092505050919050565b60006104fe836000018373ffffffffffffffffffffffffffffffffffffffff1660001b6105d6565b905092915050565b6000808273ffffffffffffffffffffffffffffffffffffffff163b119050919050565b600060019054906101000a900460ff16610578576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161056f90610fdd565b60405180910390fd5b565b6060816000018054806020026020016040519081016040528092919081815260200182805480156105ca57602002820191906000526020600020905b8154815260200190600101908083116105b6575b50505050509050919050565b600080836001016000848152602001908152602001600020541415905092915050565b60405180604001604052806060815260200160008081111561061e5761061d610833565b5b81525090565b600081519050919050565b600082825260208201905092915050565b6000819050602082019050919050565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b600061067b82610650565b9050919050565b61068b81610670565b82525050565b600061069d8383610682565b60208301905092915050565b6000602082019050919050565b60006106c182610624565b6106cb818561062f565b93506106d683610640565b8060005b838110156107075781516106ee8882610691565b97506106f9836106a9565b9250506001810190506106da565b5085935050505092915050565b6000602082019050818103600083015261072e81846106b6565b905092915050565b6000604051905090565b600080fd5b600080fd5b61075381610670565b811461075e57600080fd5b50565b6000813590506107708161074a565b92915050565b60006020828403121561078c5761078b610740565b5b600061079a84828501610761565b91505092915050565b600081519050919050565b600082825260208201905092915050565b60005b838110156107dd5780820151818401526020810190506107c2565b60008484015250505050565b6000601f19601f8301169050919050565b6000610805826107a3565b61080f81856107ae565b935061081f8185602086016107bf565b610828816107e9565b840191505092915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602160045260246000fd5b6001811061087357610872610833565b5b50565b600081905061088482610862565b919050565b600061089482610876565b9050919050565b6108a481610889565b82525050565b600060408301600083015184820360008601526108c782826107fa565b91505060208301516108dc602086018261089b565b508091505092915050565b6000602082019050818103600083015261090181846108aa565b905092915050565b600080fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b610946826107e9565b810181811067ffffffffffffffff821117156109655761096461090e565b5b80604052505050565b6000610978610736565b9050610984828261093d565b919050565b600080fd5b600080fd5b600080fd5b600067ffffffffffffffff8211156109b3576109b261090e565b5b6109bc826107e9565b9050602081019050919050565b82818337600083830152505050565b60006109eb6109e684610998565b61096e565b905082815260208101848484011115610a0757610a06610993565b5b610a128482856109c9565b509392505050565b600082601f830112610a2f57610a2e61098e565b5b8135610a3f8482602086016109d8565b91505092915050565b60018110610a5557600080fd5b50565b600081359050610a6781610a48565b92915050565b600060408284031215610a8357610a82610909565b5b610a8d604061096e565b9050600082013567ffffffffffffffff811115610aad57610aac610989565b5b610ab984828501610a1a565b6000830152506020610acd84828501610a58565b60208301525092915050565b600060208284031215610aef57610aee610740565b5b600082013567ffffffffffffffff811115610b0d57610b0c610745565b5b610b1984828501610a6d565b91505092915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602260045260246000fd5b60006002820490506001821680610b6957607f821691505b602082108103610b7c57610b7b610b22565b5b50919050565b60008190508160005260206000209050919050565b60006020601f8301049050919050565b600082821b905092915050565b600060088302610be47fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff82610ba7565b610bee8683610ba7565b95508019841693508086168417925050509392505050565b6000819050919050565b6000819050919050565b6000610c35610c30610c2b84610c06565b610c10565b610c06565b9050919050565b6000819050919050565b610c4f83610c1a565b610c63610c5b82610c3c565b848454610bb4565b825550505050565b600090565b610c78610c6b565b610c83818484610c46565b505050565b5b81811015610ca757610c9c600082610c70565b600181019050610c89565b5050565b601f821115610cec57610cbd81610b82565b610cc684610b97565b81016020851015610cd5578190505b610ce9610ce185610b97565b830182610c88565b50505b505050565b600082821c905092915050565b6000610d0f60001984600802610cf1565b1980831691505092915050565b6000610d288383610cfe565b9150826002028217905092915050565b610d41826107a3565b67ffffffffffffffff811115610d5a57610d5961090e565b5b610d648254610b51565b610d6f828285610cab565b600060209050601f831160018114610da25760008415610d90578287015190505b610d9a8582610d1c565b865550610e02565b601f198416610db086610b82565b60005b82811015610dd857848901518255600182019150602085019450602081019050610db3565b86831015610df55784890151610df1601f891682610cfe565b8355505b6001600288020188555050505b505050505050565b600082825260208201905092915050565b6000610e26826107a3565b610e308185610e0a565b9350610e408185602086016107bf565b610e49816107e9565b840191505092915050565b60006020820190508181036000830152610e6e8184610e1b565b905092915050565b7f496e697469616c697a61626c653a20636f6e747261637420697320616c72656160008201527f647920696e697469616c697a6564000000000000000000000000000000000000602082015250565b6000610ed2602e83610e0a565b9150610edd82610e76565b604082019050919050565b60006020820190508181036000830152610f0181610ec5565b9050919050565b6000819050919050565b600060ff82169050919050565b6000610f3a610f35610f3084610f08565b610c10565b610f12565b9050919050565b610f4a81610f1f565b82525050565b6000602082019050610f656000830184610f41565b92915050565b7f496e697469616c697a61626c653a20636f6e7472616374206973206e6f74206960008201527f6e697469616c697a696e67000000000000000000000000000000000000000000602082015250565b6000610fc7602b83610e0a565b9150610fd282610f6b565b604082019050919050565b60006020820190508181036000830152610ff681610fba565b905091905056fea26469706673582212208fc81b9d8304124994273716c92a5eea9d065272ac701d9709d36aaad3714ac164736f6c63430008110033";

type ServiceRegistryConstructorParams =
  | [signer?: Signer]
  | ConstructorParameters<typeof ContractFactory>;

const isSuperArgs = (
  xs: ServiceRegistryConstructorParams
): xs is ConstructorParameters<typeof ContractFactory> => xs.length > 1;

export class ServiceRegistry__factory extends ContractFactory {
  constructor(...args: ServiceRegistryConstructorParams) {
    if (isSuperArgs(args)) {
      super(...args);
    } else {
      super(_abi, _bytecode, args[0]);
    }
  }

  override deploy(
    overrides?: Overrides & { from?: PromiseOrValue<string> }
  ): Promise<ServiceRegistry> {
    return super.deploy(overrides || {}) as Promise<ServiceRegistry>;
  }
  override getDeployTransaction(
    overrides?: Overrides & { from?: PromiseOrValue<string> }
  ): TransactionRequest {
    return super.getDeployTransaction(overrides || {});
  }
  override attach(address: string): ServiceRegistry {
    return super.attach(address) as ServiceRegistry;
  }
  override connect(signer: Signer): ServiceRegistry__factory {
    return super.connect(signer) as ServiceRegistry__factory;
  }

  static readonly bytecode = _bytecode;
  static readonly abi = _abi;
  static createInterface(): ServiceRegistryInterface {
    return new utils.Interface(_abi) as ServiceRegistryInterface;
  }
  static connect(
    address: string,
    signerOrProvider: Signer | Provider
  ): ServiceRegistry {
    return new Contract(address, _abi, signerOrProvider) as ServiceRegistry;
  }
}
