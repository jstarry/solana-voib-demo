import {
  Account,
  Connection,
  PublicKey,
  Transaction,
  SystemProgram,
  sendAndConfirmTransaction,
} from '@solana/web3.js';

const LOCAL_NODE = 'http://127.0.0.1:8899';
const PAYER_AIRDROP = 10000;

const PREPAY_PROGRAM_ID = new PublicKey(
  '9ecPa9EqqwcjzPTCNLisYaGskkc3j5b12xdcBZNP7sxK',
);
const PREPAY_CONTRACT_BALANCE = 1000;
const PREPAY_CONTRACT_SPACE = 96;
const GATEKEEPER_KEY = new PublicKey(
  'CwZ9PMZxESLNqdCS6tsyv2uaKCkE6fUW7Y2g7TiPzN1C',
);
const PROVIDER_KEY = new PublicKey(
  'G4pukKXJy7ysEZRwoHUGETNn7X5LXtBNGRGrtZruHGnS',
);

export default class PrepayService {
  connection: any;
  payerAccount: any;

  constructor() {
    this.connection = new Connection(LOCAL_NODE);
    this.payerAccount = new Account();
  }

  async init(): Promise<void> {
    console.log('request airdrop');
    await this.connection.requestAirdrop(
      this.payerAccount.publicKey,
      PAYER_AIRDROP,
    );
  }

  getPayerKey(): any {
    return this.payerAccount.publicKey;
  }

  async newContract(): Promise<any> {
    const contractAccount = new Account();
    const transaction = new Transaction();
    transaction.add(
      SystemProgram.createAccount(
        this.payerAccount.publicKey,
        contractAccount.publicKey,
        PREPAY_CONTRACT_BALANCE,
        PREPAY_CONTRACT_SPACE,
        PREPAY_PROGRAM_ID,
      ),
    );

    transaction.add({
      keys: [
        {
          pubkey: this.payerAccount.publicKey,
          isSigner: true,
          isDebitable: true,
        },
        {
          pubkey: contractAccount.publicKey,
          isSigner: true,
          isDebitable: true,
        },
        {pubkey: GATEKEEPER_KEY, isSigner: false, isDebitable: false},
        {pubkey: PROVIDER_KEY, isSigner: false, isDebitable: false},
      ],
      programId: PREPAY_PROGRAM_ID,
      data: Buffer.from([0x0, 0x0, 0x0, 0x0]),
    });

    console.log('sendAndConfirmTransaction');
    const signature = await sendAndConfirmTransaction(
      this.connection,
      transaction,
      this.payerAccount,
      contractAccount,
    );

    console.log('signature', {signature});
    return contractAccount.publicKey;
  }
}
