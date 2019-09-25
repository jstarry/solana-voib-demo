import fs from 'mz/fs';
import path from 'path';
import {Account, BpfLoader, Connection, PublicKey, testnetChannelEndpoint} from '@solana/web3.js';

(async function() {
  const url = process.env.LIVE
  ? testnetChannelEndpoint(process.env.CHANNEL || 'beta', false)
  : 'http://localhost:8899';

  const distPath = path.join(
    __dirname,
    '..',
    '..',
    'dist',
  );

  const elfFile = path.join(distPath, 'programs', 'bandwidth_prepay.so');
  console.log(`Reading ${elfFile}...`);
  const elfData = await fs.readFile(elfFile);

  console.log('Loading program...');
  const loaderAccount = new Account();
  const connection = new Connection(url);
  await connection.requestAirdrop(loaderAccount.publicKey, 100000);
  await BpfLoader.load(connection, loaderAccount, elfData);

  const idFile = path.join(distPath, 'program_id.json');
  const programId = loaderAccount.publicKey.toBase58();
  await fs.writeFile(idFile, JSON.stringify(programId.toBase58()), 'utf8');
  console.log(`Saved program id to ${idFile}`);
})()
