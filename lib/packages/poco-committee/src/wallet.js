// Import the ChainUtil class used for hashing and verification
const ChainUtil = require("./util");
// Import transaction class  used for creating transactions
const Transaction = require("./transaction");

// wallet represents an account in the oracle
class Wallet {
  // The secret phase is passed an argument when creating a wallet
  // The keypair generated for a secret phrase is always the same
  constructor(secret) {
    this.keyPair = ChainUtil.genKeyPair(secret);
    this.publicKey = this.keyPair.getPublic("hex");
  }

  // Used for prining the wallet details
  toString() {
    return `Wallet - 
            publicKey: ${this.publicKey.toString()}`;
  }

  // Used for signing data hashes
  // sign的作用是使得拥有它的公钥的其他节点能够验证这个消息是来自它发送的
  sign(dataHash) {
    return this.keyPair.sign(dataHash).toHex();
  }

  // Creates and returns transactions
  // entry of createANewTransaction
  createTransaction(data) {
    return new Transaction(data, this);
  }

  // Return public key
  getPublicKey() {
    return this.publicKey;
  }
}

module.exports = Wallet;
