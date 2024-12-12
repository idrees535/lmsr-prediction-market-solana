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
        2, // Decimals
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

    console.log("Base Token Mint Address:", baseTokenMint.publicKey.toBase58());

    const marketAccount = await marketProgram.account.market.fetch(marketPDA);
    console.log("Market Account Data:", marketAccount);

    // Ensure the market creation was successful
    expect(marketAccount.title).toBe("My Test Market");
    expect(marketAccount.outcomes.length).toBe(2);
    expect(marketAccount.marketMakerFunds.toNumber()).toBe(1000);
  });

// Test 2: Buy Shares
  it("Can buy shares", async () => {

    console.log ('Deriving users ATA')
    userTokenAccount = await splToken.getAssociatedTokenAddress(
      baseTokenMint.publicKey,
      user.publicKey
    );

    console.log("User Token Account Address Derived:", userTokenAccount.toBase58());
   
    console.log("Creating user's Associated Token Account...");
    const createUserATATx = new Transaction().add(
      splToken.createAssociatedTokenAccountInstruction(
        user.publicKey,         // Payer
        userTokenAccount,       // Associated Token Account to create
        user.publicKey,         // Owner of the account
        baseTokenMint.publicKey // Mint address
      )
    );

      if (provider.sendAndConfirm) {
        await provider.sendAndConfirm(createUserATATx, [user]);
        console.log('User ATA creation transaction sent');
      }
      console.log("User's ATA created:", userTokenAccount.toBase58())
    

    
    console.log("Base Token Mint Address:", baseTokenMint.publicKey.toBase58());    
    marketTokenAccount = await splToken.getAssociatedTokenAddress(
      baseTokenMint.publicKey,
      marketPDA,
      true // Allow off-curve PDA
    );
    console.log('Market token account Derived');
    console.log("Market Token Account Address:", marketTokenAccount.toBase58());

    console.log("Creating market's Associated Token Account...");
    const createMarketATATx = new Transaction().add(
      splToken.createAssociatedTokenAccountInstruction(
        user.publicKey,         // Payer
        marketTokenAccount,       // Associated Token Account to create
        marketPDA,        // Owner of the account
        baseTokenMint.publicKey // Mint address
      )
    );

    if (provider.sendAndConfirm) {
      await provider.sendAndConfirm(createMarketATATx, [user]);
      console.log('market ATA creation transaction sent');
    }
    console.log("market's ATA created:", marketTokenAccount.toBase58())

    // Mint tokens to the user's associated token account
    const mintInfo = await splToken.getMint(provider.connection, baseTokenMint.publicKey);
    console.log("Mint Info:", mintInfo);

    const userAccountInfo = await splToken.getAccount(provider.connection, userTokenAccount);
    console.log("User Token Account Info:", userAccountInfo);

    const marketAccountInfo = await splToken.getAccount(provider.connection, marketTokenAccount);
    console.log("market Token Account Info:", marketAccountInfo);

    console.log("Minting tokens to buyer's ATA...");
     const mintTx = new anchor.web3.Transaction().add(
       splToken.createMintToInstruction(
         baseTokenMint.publicKey,
         userTokenAccount,
         user.publicKey,
         1000000000000000
       )
     );
     if (provider.sendAndConfirm) {
       await provider.sendAndConfirm(mintTx, [user]);
       console.log("Minted tokens to buyer's ATT tx:",{mintTx});
     }

    console.log("Minted tokens to buyer's ATT");
    const userAccountInfo_after = await splToken.getAccount(provider.connection, userTokenAccount);
    console.log("User Token Account Info:", userAccountInfo_after);


    console.log("transfer tokens to market's ATA...");
    const mintTx_1 = new anchor.web3.Transaction().add(
      splToken.createTransferInstruction(
        userTokenAccount,
        marketTokenAccount,
        user.publicKey,
        500
      )
    );
    if (provider.sendAndConfirm) {
      await provider.sendAndConfirm(mintTx_1, [user]);
      console.log("Transfer tokens to market's ATT tx:", { mintTx_1 });
    }

    console.log("Transfer tokens to market's ATT");
    const marketAccountInfo_after = await splToken.getAccount(provider.connection, marketTokenAccount);
    console.log("market Token Account Info:", marketAccountInfo_after);

    const userBalanceBefore = ((await splToken.getAccount(provider.connection, userTokenAccount)).amount);
    const marketBalanceBefore = ((await splToken.getAccount(provider.connection, marketTokenAccount)).amount);

    console.log("finally let's try fucking do what we are here to do, buy shares");
    // Prepare the BuyShares instruction
    const buySharesAccounts = {
      market: marketPDA,
      buyerTokenAccount: userTokenAccount,
      marketTokenAccount: marketTokenAccount,
      baseTokenMint: baseTokenMint.publicKey,
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

    const userBalanceAfter = (await splToken.getAccount(provider.connection, userTokenAccount)).amount;
    const marketBalanceAfter = (await splToken.getAccount(provider.connection, marketTokenAccount)).amount;

    expect(userBalanceAfter).toBeLessThan(userBalanceBefore);
    console.log(`User balance before: ${userBalanceBefore}, after: ${userBalanceAfter}`);

    // Check market token account balance
    expect(marketBalanceAfter).toBeGreaterThan(marketBalanceBefore );
    console.log(`Market balance before: ${marketBalanceBefore}, after: ${marketBalanceAfter}`);

    console.log(`User balance diff: ${userBalanceBefore-userBalanceAfter}`);
    console.log(`market balance diff: ${marketBalanceAfter - marketBalanceBefore}`);

    
  });

});
 