const Transaction = require('../src/transaction'); 
const ChainUtil = require('../src/util'); 
const Wallet = require('../src/wallet');

describe('Transaction', () => {
  let data, transaction;

  // initialize wallet
  const wallet = new Wallet(process.env.SECRET); 

  beforeEach(() => {
    // setup data and wallet
    data = { amount: 10, address: 'test-address' };

    // create a new transaction
    transaction = new Transaction(data, wallet);
  });

  test('should create a transaction with correct data', () => {
    expect(transaction.from).toBe(wallet.getPublicKey());
    expect(transaction.input.data).toEqual(data);
    expect(typeof transaction.id).toBe('string');
    expect(typeof transaction.signature).toBe('string');
  });

  test('should verify a valid transaction', () => {
    expect(Transaction.verifyTransaction(transaction)).toBe(true);
  });

  test('should not verify an invalid transaction', () => {
    // tamper with the transaction data
    transaction.input.data.amount = 1000;
    expect(Transaction.verifyTransaction(transaction)).toBe(false);
  });
});