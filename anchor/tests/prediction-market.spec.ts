import * as anchor from '@coral-xyz/anchor';
import { BankrunProvider, startAnchor } from "anchor-bankrun";
import { Program} from '@coral-xyz/anchor';
import { SystemProgram, Keypair, Transaction, PublicKey, sendAndConfirmTransaction } from '@solana/web3.js';
import { PredictionMarket } from '../target/types/prediction_market';
import * as splToken from '@solana/spl-token';


const IDL = require('../target/idl/prediction_market.json');
const marketAddress = new PublicKey("AsjZ3kWAUSQRNt2pZVeJkywhZ6gpLpHZmJjduPmKZDZZ");

describe("Prediction Market", () => {
it("Can create a market", async () => {
  // Generate a new keypair for the user
  const user = Keypair.generate();

  const context = await startAnchor("", [
    {
      name: 'prediction_market',
      programId: marketAddress
    }],
    [
      {
        address: user.publicKey,
        info: {
          lamports: 1_000_000_000, // 1 SOL equivalent
          data: Buffer.alloc(0),
          owner: SystemProgram.programId,
          executable: false,
        },
      },
    ]);

  const provider = new BankrunProvider(context);

  const marketProgram = new Program<PredictionMarket>(IDL, provider);
  //const connection = new Connection(clusterApiUrl('devnet'), 'confirmed');
  //const connection = provider.connection as Connection;
  //const connection = provider.connection;

  
  // Create a dummy mint as base token mint
  //const baseTokenMint = Keypair.generate();
  // const baseTokenMint = await splToken.createMint(
  //   connection, // Solana connection
  //   user,                // Payer (fee payer)
  //   user.publicKey,      // Mint authority
  //   null,                // Freeze authority (optional, null for none)
  //   0                    // Decimals (e.g., 0 for integers)
  // );


  // const userTokenAccount = await splToken.createAccount(
  //   connection,
  //   user,                  // Fee payer
  //   baseTokenMint.publicKey,         // Mint address
  //   user.publicKey         // Owner of the token account
  // );

  // Mint some tokens to the user
  // await splToken.mintTo(
  //   connection,
  //   user,
  //   baseTokenMint.publicKey,
  //   userTokenAccount,
  //   payer,
  //   1_000_000_000 // 1 token (assuming 9 decimals)
  // );


  // Create base token mint manually as the above code is not working and it is fucking up with bankrun connection
  const baseTokenMint = Keypair.generate();
  const transaction = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: user.publicKey,
      newAccountPubkey: baseTokenMint.publicKey,
      space: splToken.MINT_SIZE,
      lamports: await provider.connection.getMinimumBalanceForRentExemption(splToken.MINT_SIZE),
      programId: splToken.TOKEN_PROGRAM_ID,
    }),
    splToken.createInitializeMintInstruction(
      baseTokenMint.publicKey,
      0, // Decimals
      user.publicKey, // Mint authority
      null // Freeze authority
    )
  );

  //await provider.sendAndConfirm(transaction, [user, baseTokenMint]);
  //await sendAndConfirmTransaction(provider.connection, transaction, [user, baseTokenMint]);
  // Use BankrunProvider's sendAndConfirm method
  if (provider.sendAndConfirm) {
    await provider.sendAndConfirm(transaction, [user, baseTokenMint]);
  } else {
    throw new Error("sendAndConfirm method is not available on BankrunProvider");
  }

  console.log("Base Token Mint Address:", baseTokenMint.publicKey.toBase58());

  // PDAs often determined by seeds, but here we rely on init account only.
  let [marketPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("market"), new anchor.BN(12345).toArrayLike(Buffer, "le", 8)],
    marketProgram.programId
  );

  console.log("Market PDA:", marketPDA.toBase58());
  console.log("User Public Key:", user.publicKey.toBase58());
  console.log("Base Token Mint Public Key:", baseTokenMint.publicKey.toBase58());

  const accounts: any = {
    market: marketPDA,
    user: user.publicKey,
    baseTokenMint: baseTokenMint.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
    tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  };

  const tx = await marketProgram.methods
    .createMarket(
      new anchor.BN(12345),           // market_id
      "My Test Market",               // title
      ["Outcome1", "Outcome2"],       // outcomes
      user.publicKey, // dummy oracle
      new anchor.BN(5),               // b
      new anchor.BN(3600),            // duration (1 hour)
      new anchor.BN(2),               // fee_percent
      user.publicKey, // fee_recipient dummy
      new anchor.BN(1000)             // initial_funds
    )
    .accounts(accounts)
    .signers([user])
    .rpc();

  console.log("Your transaction signature", tx);

  const marketAccount = await marketProgram.account.market.fetch(marketPDA);
  console.log("Market Account:", marketAccount);
});
});