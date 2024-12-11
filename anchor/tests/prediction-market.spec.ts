import * as anchor from '@coral-xyz/anchor';
import { BankrunProvider, startAnchor } from "anchor-bankrun";
import { Program } from '@coral-xyz/anchor';
import { SystemProgram, Keypair, Transaction, PublicKey } from '@solana/web3.js';
import { PredictionMarket } from '../target/types/prediction_market';
import * as splToken from '@solana/spl-token';

const IDL = require('../target/idl/prediction_market.json');
const marketAddress = new PublicKey("AsjZ3kWAUSQRNt2pZVeJkywhZ6gpLpHZmJjduPmKZDZZ");

describe("Prediction Market", () => {
  let provider: BankrunProvider;
  let marketProgram: Program<PredictionMarket>;
  let user: Keypair;
  let baseTokenMint: Keypair;
  let marketPDA: PublicKey;
  let userTokenAccount: PublicKey;
  let marketTokenAccount: PublicKey;

  // Setup: Run once before all tests
  beforeAll(async () => {
    user = Keypair.generate();
    const context = await startAnchor("", [
      {
        name: 'prediction_market',
        programId: marketAddress,
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

    provider = new BankrunProvider(context);
    marketProgram = new Program<PredictionMarket>(IDL, provider);

    // Create base token mint for the market
    baseTokenMint = Keypair.generate();
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

    if (provider.sendAndConfirm) {
      await provider.sendAndConfirm(transaction, [user, baseTokenMint]);
    } else {
      throw new Error("sendAndConfirm method is not available on BankrunProvider");
    }

    // Create the market
    let [marketPDAGenerated] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("market"), new anchor.BN(12345).toArrayLike(Buffer, "le", 8)],
      marketProgram.programId
    );
    marketPDA = marketPDAGenerated;

    const marketAccounts: any = {
      market: marketPDA,
      user: user.publicKey,
      baseTokenMint: baseTokenMint.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    };

    await marketProgram.methods
      .createMarket(
        new anchor.BN(12345),           // market_id
        "My Test Market",               // title
        ["Outcome1", "Outcome2"],       // outcomes
        user.publicKey,                 // dummy oracle
        new anchor.BN(5),               // b
        new anchor.BN(3600),            // duration (1 hour)
        new anchor.BN(2),               // fee_percent
        user.publicKey,                 // fee_recipient dummy
        new anchor.BN(1000)             // initial_funds
      )
      .accounts(marketAccounts)
      .signers([user])
      .rpc();
  });

  // Test 1: Create Market
  it("Can create a market", async () => {
    console.log("Market Created with PDA:", marketPDA.toBase58());

    const marketAccount = await marketProgram.account.market.fetch(marketPDA);
    console.log("Market Account Data:", marketAccount);

    // Ensure the market creation was successful
    expect(marketAccount.title).toBe("My Test Market");
    expect(marketAccount.outcomes.length).toBe(2);
    expect(marketAccount.marketMakerFunds.toNumber()).toBe(1000);
  });

  // Test 2: Buy Shares
  it("Can buy shares", async () => {
    // Mint tokens to user
    userTokenAccount = await splToken.createAccount(
      provider.connection,
      user,
      baseTokenMint.publicKey,
      user.publicKey
    );

    // Mint tokens to the user's token account
    const mintTx = new Transaction().add(
      splToken.createMintToInstruction(
        baseTokenMint.publicKey,  // Mint address
        userTokenAccount,         // Target token account
        user.publicKey,           // Mint authority
        1000                      // Amount to mint
      )
    );

    if (provider.sendAndConfirm) {
      await provider.sendAndConfirm(mintTx, [user]);
    } else {
      throw new Error("sendAndConfirm method is not available on BankrunProvider");
    }

    // await splToken.mintTo(
    //   provider.connection,
    //   user,
    //   baseTokenMint.publicKey,
    //   userTokenAccount,
    //   user,
    //   1000 // Mint 1000 tokens to the user's token account
    // );

    marketTokenAccount = await splToken.getAssociatedTokenAddress(
      baseTokenMint.publicKey,  // Mint address
      marketPDA,                 // Owner's public key (market PDA)
      false,                     // Allow off-curve (usually false)
      splToken.TOKEN_PROGRAM_ID, // Token program ID
      splToken.ASSOCIATED_TOKEN_PROGRAM_ID // Associated token program ID
    );

    // Prepare the BuyShares instruction
    const buySharesAccounts = {
      market: marketPDA,
      buyerTokenAccount: userTokenAccount,
      marketTokenAccount: marketTokenAccount,
      buyer: user.publicKey,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
    };

    // Call the buy_shares function
    const buySharesTx = await marketProgram.methods
      .buyShares(new anchor.BN(0), new anchor.BN(10)) // Buying 10 shares of Outcome 0
      .accounts(buySharesAccounts)
      .signers([user])
      .rpc();

    console.log("Buy Shares Transaction Signature:", buySharesTx);

    // Verify the market state after the transaction
    const marketAccount = await marketProgram.account.market.fetch(marketPDA);
    console.log("Market Account After Buy:", marketAccount);

    // Check the buyer's token account balance
    const buyerTokenBalance = await splToken.getAccount(
      provider.connection,
      userTokenAccount
    );
    console.log("User Token Account Balance after Buy:", buyerTokenBalance.amount.toString());

    // Verify that the buyer's token balance decreased after buying shares
    expect(Number(buyerTokenBalance.amount)).toBeLessThan(1000);

  });
});




