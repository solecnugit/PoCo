// Import the ChainUtil class used for hashing and verification
const ChainUtil = require("./util");

class Transaction {
  // the wallet instance will be passed as a parameter to the constructor
  // along with the data to be stored.
  constructor(data, wallet) {
    // unique id from uuidV1
    this.id = ChainUtil.id();

    // the from address is the public key of the wallet.
    this.from = wallet.publicKey;

    // the input is the data and timestamp.
    this.input = { data: data, timestamp: Date.now() };

    this.hash = ChainUtil.hash(this.input);

    // used to verify the transaction by other wallet.
    this.signature = wallet.sign(this.hash);
  }

  // this method verifies wether the transaction is valid
  static verifyTransaction(transaction) {
    return ChainUtil.verifySignature(
      transaction.from,
      transaction.signature,
      ChainUtil.hash(transaction.input),
    );
  }
}

module.exports = Transaction;
