import * as anchor from '@coral-xyz/anchor';
import { Clock } from "solana-bankrun";
import { BankrunProvider, startAnchor } from "anchor-bankrun";
import { Program } from '@coral-xyz/anchor';
import { SystemProgram, Keypair, Transaction, PublicKey, TransactionResponse, GetVersionedTransactionConfig } from '@solana/web3.js';
import { PredictionMarket } from '../target/types/prediction_market';
import * as splToken from '@solana/spl-token';

const IDL = require('../target/idl/prediction_market.json');
const marketAddress = new PublicKey("AsjZ3kWAUSQRNt2pZVeJkywhZ6gpLpHZmJjduPmKZDZZ");
//let's import token decimals from constant.rs
const TOKEN_DECIMALS = 9;
const SHARE_DECIMALS = 0;

describe("Prediction Market", () => {
  let provider: BankrunProvider;
  let marketProgram: Program<PredictionMarket>;
  let user: Keypair;
  let baseTokenMint: Keypair;
  let marketPDA: PublicKey;
  let userTokenAccount: PublicKey;
  let userShareAccount: PublicKey;
  let marketTokenAccount: PublicKey;
  let outcomeMints: Keypair[] = [];
  let oracle: Keypair;
  let context: any;
  let client: any;
  let feeRecipient:Keypair;
  let feeRecipientTokenAccount: PublicKey;
  //;lets scale sahrees bought sold to to decimals precison 500*10^9
  let shares_bought = 5000 * Math.pow(10, SHARE_DECIMALS);
  let shares_sold = 1000* Math.pow(10, SHARE_DECIMALS);
  let fee_percent = 100; // 1% fee in basis point


  // Setup: Run once before all tests
  beforeAll(async () => {


    // Ensure a fresh context is set
    console.log("Bankrun environment reset successfully.");

    user = Keypair.generate();
    oracle= Keypair.generate();
    baseTokenMint = Keypair.generate();
    feeRecipient = Keypair.generate();

    context = await startAnchor("", [
      {
        name: 'prediction_market',
        programId: marketAddress,
      }],
      [
        {
          address: user.publicKey,
          info: {
            lamports: 1_000_000_00000000, // 1 SOL equivalent
            data: Buffer.alloc(0),
            owner: SystemProgram.programId,
            executable: false,
          },
        },
      ]);

    client = context.banksClient;
    provider = new BankrunProvider(context);
    marketProgram = new Program<PredictionMarket>(IDL, provider);

    // Create base token mint for the market
    
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
        TOKEN_DECIMALS, // Decimals
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

    // Create outcome mints
    const outcomes = ["Outcome1", "Outcome2"];
    outcomeMints = await Promise.all(
      outcomes.map(async () => {
        const outcomeMint = Keypair.generate();
        const outcomeMintTransaction = new Transaction().add(
          SystemProgram.createAccount({
            fromPubkey: user.publicKey, // Payer
            newAccountPubkey: outcomeMint.publicKey, // New account
            space: splToken.MINT_SIZE, // Space
            lamports: await provider.connection.getMinimumBalanceForRentExemption(splToken.MINT_SIZE), // Rent
            programId: splToken.TOKEN_PROGRAM_ID, // Program ID
          }),
          
          // splToken.createInitializeMintInstruction(
          //   outcomeMint.publicKey,
          //   0, // Decimals
          //   marketPDA, // Mint authority
          //   null // Freeze authority
          // )
        );

        if (provider.sendAndConfirm) {
          await provider.sendAndConfirm(outcomeMintTransaction, [user, outcomeMint]);
        } else {
          throw new Error("sendAndConfirm method is not available on BankrunProvider");
        }
        return outcomeMint;
      })
    );

    console.log("Outcome Mint Addresses:", outcomeMints.map(mint => mint.publicKey.toBase58()));
    const OutcomemintInfo_before = await splToken.getMint(provider.connection, outcomeMints[0].publicKey);
    console.log("Outcome Mint Info before:", OutcomemintInfo_before);

    userTokenAccount = await splToken.getAssociatedTokenAddress(
      baseTokenMint.publicKey,
      user.publicKey
    );

    console.log("Derived User Token Account Address:", userTokenAccount.toBase58());
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
    }
    console.log("Craeted User's ATA:", userTokenAccount.toBase58())


    marketTokenAccount = await splToken.getAssociatedTokenAddress(
      baseTokenMint.publicKey,
      marketPDA,
      true // Allow off-curve PDA
    );
    console.log("Derived Market Token Account Address:", marketTokenAccount.toBase58());

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
    }

    const mintTx = new anchor.web3.Transaction().add(
      splToken.createMintToInstruction(
        baseTokenMint.publicKey,
        userTokenAccount,
        user.publicKey,
        100000 * Math.pow(10, TOKEN_DECIMALS) // Mint 0.1M tokens
      )
    );
    if (provider.sendAndConfirm) {
      await provider.sendAndConfirm(mintTx, [user]);
    }

    console.log("Minted tokens to buyer's ATT");
    const userAccountInfo_after = await splToken.getAccount(provider.connection, userTokenAccount);
    console.log("User Token Account Info:", userAccountInfo_after);



    const remainingAccounts = outcomeMints.map((mint) => ({
      pubkey: mint.publicKey,
      isWritable: true,
      isSigner: false,
    }));


    const marketAccounts: any = {
      market: marketPDA,
      user: user.publicKey,
      baseTokenMint: baseTokenMint.publicKey,
      userTokenAccount: userTokenAccount,
      marketTokenAccount: marketTokenAccount,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    };

    await marketProgram.methods
      .createMarket(
        new anchor.BN(12345),           // market_id
        "My Test Market",               // title
        ["Outcome1", "Outcome2"],       // outcomes
        oracle.publicKey,                 
        new anchor.BN(1000),               // b
        new anchor.BN(3600),            // duration (1 hour)
        new anchor.BN(fee_percent),               // fee_percent
        feeRecipient.publicKey,                 
        new anchor.BN(694*Math.pow(10,TOKEN_DECIMALS))             // b.ln (n)
      )
      .accounts(marketAccounts)
      .remainingAccounts(remainingAccounts)
      .signers([user])
      .rpc();

    const OutcomemintInfo_after = await splToken.getMint(provider.connection, outcomeMints[0].publicKey);
    console.log("Outcome Mint Info after:", OutcomemintInfo_after);


  });
  

// Test 2: Buy Shares
  it("Can buy shares", async () => {

    const OutcomemintInfo_before_buy= await splToken.getMint(provider.connection, outcomeMints[0].publicKey);
    console.log("OutcomemintInfo_before_buy:", OutcomemintInfo_before_buy);

    console.log("Base Token Mint Address:", baseTokenMint.publicKey.toBase58());
    
    console.log("Craeted market's ATA:", marketTokenAccount.toBase58())

    // Mint tokens to the user's associated token account
    const mintInfo = await splToken.getMint(provider.connection, baseTokenMint.publicKey);
    console.log("Base Token Mint Info:", mintInfo);

    const userAccountInfo = await splToken.getAccount(provider.connection, userTokenAccount);
    console.log("User Token Account Info:", userAccountInfo);

    const marketAccountInfo = await splToken.getAccount(provider.connection, marketTokenAccount);
    console.log("Market Token Account Info:", marketAccountInfo);

    console.log("Transfer tokens to market's ATA");
    const marketAccountInfo_after = await splToken.getAccount(provider.connection, marketTokenAccount);
    console.log("Market Token Account Info:", marketAccountInfo_after);

    const userBalanceBefore = ((await splToken.getAccount(provider.connection, userTokenAccount)).amount);
    const marketBalanceBefore = ((await splToken.getAccount(provider.connection, marketTokenAccount)).amount);

    // Derive the user's associated token account for Outcome 0 shares
    const buy_outcome_index = 0;
    const outcomeMint = outcomeMints[buy_outcome_index];
    console.log("Outcome Mint Address at index 0:", outcomeMint.publicKey.toBase58());
    userShareAccount = await splToken.getAssociatedTokenAddress(
      outcomeMint.publicKey,
      user.publicKey
    
    );

    //Create the user's share ATA if it doesn't exist
   
      const createUserShareATATx = new Transaction().add(
        splToken.createAssociatedTokenAccountInstruction(
          user.publicKey,            // Payer
          userShareAccount,         // Associated Token Account to create
          user.publicKey,            // Owner of the account
          outcomeMint.publicKey                // Mint address
        )
      );

    if (provider.sendAndConfirm) {
      await provider.sendAndConfirm(createUserShareATATx, [user]);
    }
    console.log("User's Share ATA created:", userShareAccount.toBase58())


    console.log("finally let's try fucking do what we are here to do, buy shares");
    // Prepare the BuyShares instruction
    const buySharesAccounts = {
      market: marketPDA,
      buyerTokenAccount: userTokenAccount,
      marketTokenAccount: marketTokenAccount,
      outcomeMint: outcomeMint.publicKey,
      buyerShareAccount: userShareAccount,
      baseTokenMint: baseTokenMint.publicKey,
      buyer: user.publicKey,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
    };

    const OutcomemintInfo_2 = await splToken.getMint(provider.connection, outcomeMints[0].publicKey);
    console.log("OutcomemintInfo_2:", OutcomemintInfo_2);

    // Call the buy_shares function
    const buySharesTx = await marketProgram.methods
      .buyShares(new anchor.BN(buy_outcome_index), new anchor.BN(shares_bought)) // Buying 10 shares of Outcome 0
      .accounts(buySharesAccounts)
      .signers([user])
      .rpc();

    console.log("Buy Shares Transaction Signature:", buySharesTx);
    const OutcomemintInfo_3 = await splToken.getMint(provider.connection, outcomeMints[0].publicKey);
    console.log("OutcomemintInfo_3:", OutcomemintInfo_3);


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

    const marketAccount1 = await marketProgram.account.market.fetch(marketPDA);
    // Access and log specific fields
    console.log("Market Maker Funds:", marketAccount1.marketMakerFunds.toNumber());
    console.log("Collected Fees:", marketAccount1.collectedFees.toNumber());
    console.log("Total Shares for Outcome 0:", marketAccount1.outcomes[0].totalShares.toNumber());


    // Add assertions to verify updates
    expect(marketAccount1.marketMakerFunds.toNumber()).toBeGreaterThan(0);
    expect(marketAccount1.collectedFees.toNumber()).toBeGreaterThan(0);
    expect(marketAccount1.outcomes[0].totalShares.toNumber()).toBe(shares_bought);

    //let's check the user's share account
    const userShareAccountInfo_after = await splToken.getAccount(provider.connection, userShareAccount);
    console.log("User's Share Token Account Info After Buy:", userShareAccountInfo_after);
    expect(Number(userShareAccountInfo_after.amount)).toBe(shares_bought);
    //let's also first print and then check total outcome shares
    console.log("Total Shares for Outcome 0:", marketAccount1.outcomes[0].totalShares.toNumber());
    expect(marketAccount1.outcomes[0].totalShares.toNumber()).toBe(shares_bought);
    
  });

  // Test 2: Sell Shares
  it("Can sell shares", async () => {
    const sell_outcome_index = 0;
    const outcomeMint = outcomeMints[sell_outcome_index];
    const userShareAccount = await splToken.getAssociatedTokenAddress(outcomeMint.publicKey, user.publicKey);

    const userBalanceBefore = (await splToken.getAccount(provider.connection, userTokenAccount)).amount;
    const marketBalanceBefore = (await splToken.getAccount(provider.connection, marketTokenAccount)).amount;

    // Setup accounts for sell_shares
    const sellSharesAccounts = {
      market: marketPDA,
      buyerShareAccount: userShareAccount,
      outcomeMint: outcomeMint.publicKey,
      sellerTokenAccount: userTokenAccount,
      marketTokenAccount: marketTokenAccount,
      seller: user.publicKey,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
    };

    // Call the sell_shares function
    const sellSharesTx = await marketProgram.methods
      .sellShares(new anchor.BN(sell_outcome_index), new anchor.BN(shares_sold)) // Selling 5 shares of Outcome 0
      .accounts(sellSharesAccounts)
      .signers([user])
      .rpc();

    console.log("Sell Shares Transaction Signature:", sellSharesTx);

    const userBalanceAfter = (await splToken.getAccount(provider.connection, userTokenAccount)).amount;
    const marketBalanceAfter = (await splToken.getAccount(provider.connection, marketTokenAccount)).amount;

    // Assertions
    expect(userBalanceAfter).toBeGreaterThan(userBalanceBefore);
    expect(marketBalanceAfter).toBeLessThan(marketBalanceBefore);

    console.log(`User balance before: ${userBalanceBefore}, after: ${userBalanceAfter}`);
    console.log(`Market balance before: ${marketBalanceBefore}, after: ${marketBalanceAfter}`);
  });

  it("Can close the market after end time", async () => {
    const currentClock = await client.getClock();
    if (!context?.setClock) {
      throw new Error("Bankrun context is not properly initialized.");
    }


    // Warp time forward to after the market's end time
    const newTimestamp = currentClock.unixTimestamp + BigInt(4000);// Advance 4000 seconds
    context.setClock(
      new Clock(
        currentClock.slot,
        currentClock.epochStartTimestamp,
        currentClock.epoch,
        currentClock.leaderScheduleEpoch,
        newTimestamp 
      )
    );

    console.log("Time traveled to: ", newTimestamp);


    const closeMarketTx = await marketProgram.methods
      .closeMarket()
      .accounts({
        market: marketPDA,
        oracle: oracle.publicKey, // The oracle is the user for this test
      })
      .signers([oracle])
      .rpc();

    console.log("Close Market Transaction Signature:", closeMarketTx);

    // Fetch the updated market state
    const updatedMarketAccount = await marketProgram.account.market.fetch(marketPDA);
    console.log("Market Account After Closing:", updatedMarketAccount);

    // Assertions
    expect(updatedMarketAccount.marketClosed).toBe(true);
  });

  it("Can set the winning outcome after market is closed", async () => {
   

    // Set the winning outcome
    const winningOutcomeIndex = 0; // Assume Outcome 0 is the winner
    const setOutcomeTx = await marketProgram.methods
      .setOutcome(new anchor.BN(winningOutcomeIndex))
      .accounts({
        market: marketPDA,
        oracle: user.publicKey,
      })
      .signers([user])
      .rpc();

    console.log("Set Outcome Transaction Signature:", setOutcomeTx);

    // Fetch the updated market state
    const updatedMarketAccount = await marketProgram.account.market.fetch(marketPDA);
    console.log("Market Account After Setting Outcome:", updatedMarketAccount);

    // Assertions
    expect(updatedMarketAccount.marketSettled).toBe(true);
    expect(updatedMarketAccount.winningOutcome.toNumber()).toBe(winningOutcomeIndex);
  });

  it("Can claim payout for winning shares", async () => {
    const userSharesToClaim = shares_bought-shares_sold; // User holds 10 shares of the winning outcome
    const winningOutcomeIndex = 0; // Assume Outcome 0 is the winner
    const totalPayoutperShare = 100; // Total payout per share
    

    const userShareAccountInfoBefore = await splToken.getAccount(provider.connection, userShareAccount);
    const marketTokenAccountInfoBefore = await splToken.getAccount(provider.connection, marketTokenAccount);
    const userTokenAccountInfoBefore = await splToken.getAccount(provider.connection, userTokenAccount);

    console.log("User shares before claim:", userShareAccountInfoBefore.amount.toString());
    console.log("Market funds before claim:", marketTokenAccountInfoBefore.amount.toString());
    console.log("User tokens before claim:", userTokenAccountInfoBefore.amount.toString());

    // Call claimPayout
    const claimPayoutAccounts = {
      market: marketPDA,
      marketTokenAccount: marketTokenAccount,
      userTokenAccount: userTokenAccount,
      outcomeMint: outcomeMints[winningOutcomeIndex].publicKey,
      userShareAccount: userShareAccount,
      user: user.publicKey,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
    };

    await marketProgram.methods
      .claimPayout()
      .accounts(claimPayoutAccounts)
      .signers([user])
      .rpc();

    // Fetch updated account states
    const userShareAccountInfoAfter = await splToken.getAccount(provider.connection, userShareAccount);
    const marketTokenAccountInfoAfter = await splToken.getAccount(provider.connection, marketTokenAccount);
    const userTokenAccountInfoAfter = await splToken.getAccount(provider.connection, userTokenAccount);

    console.log("User shares after claim:", userShareAccountInfoAfter.amount.toString());
    console.log("Market funds after claim:", marketTokenAccountInfoAfter.amount.toString());
    console.log("User tokens after claim:", userTokenAccountInfoAfter.amount.toString());

    // Assertions
    expect(Number(userShareAccountInfoAfter.amount)).toBe(Number(userShareAccountInfoBefore.amount) - userSharesToClaim);
    //expect(Number(marketTokenAccountInfoAfter.amount)).toBe(Number(marketTokenAccountInfoBefore.amount) - totalPayout);
    //expect(Number(userTokenAccountInfoAfter.amount)).toBe(Number(userTokenAccountInfoBefore.amount) + totalPayout);
  });

  it("Can withdraw fees", async () => {

    feeRecipientTokenAccount = await splToken.getAssociatedTokenAddress(
      baseTokenMint.publicKey,
      feeRecipient.publicKey
    );

    // Create fee recipient's token account if it doesn't exist
    const createFeeRecipientATATx = new Transaction().add(
      splToken.createAssociatedTokenAccountInstruction(
        user.publicKey, // Payer
        feeRecipientTokenAccount, // Associated Token Account to create
        feeRecipient.publicKey, // Owner of the account
        baseTokenMint.publicKey // Mint address
      )
    );

    if (provider.sendAndConfirm) {
      await provider.sendAndConfirm(createFeeRecipientATATx, [user]);
    }
    console.log("Fee recipient's ATA created:", feeRecipientTokenAccount.toBase58());
    
    // Ensure the market has collected fees
    const marketAccountBefore = await marketProgram.account.market.fetch(marketPDA);
    console.log("Market Account Before Withdrawal:", marketAccountBefore);
    const feesToWithdraw = marketAccountBefore.collectedFees.toNumber();
    //expect(feesToWithdraw).toBeGreaterThan(0);

    const marketTokenAccountInfoBefore = await splToken.getAccount(
      provider.connection,
      marketTokenAccount
    );
    const feeRecipientTokenAccountInfoBefore = await splToken.getAccount(
      provider.connection,
      feeRecipientTokenAccount
    );

    console.log("Market Token Account Before Withdrawal:", marketTokenAccountInfoBefore.amount.toString());
    console.log("Fee Recipient Token Account Before Withdrawal:", feeRecipientTokenAccountInfoBefore.amount.toString());

    // Call withdrawFees
    const withdrawFeesAccounts = {
      market: marketPDA,
      marketTokenAccount: marketTokenAccount,
      feeRecipientTokenAccount: feeRecipientTokenAccount,
      feeRecipient: feeRecipient.publicKey,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
    };

    await marketProgram.methods
      .withdrawFees()
      .accounts(withdrawFeesAccounts)
      .signers([feeRecipient])
      .rpc();

    // Fetch updated account states
    const marketAccountAfter = await marketProgram.account.market.fetch(marketPDA);
    const marketTokenAccountInfoAfter = await splToken.getAccount(
      provider.connection,
      marketTokenAccount
    );
    const feeRecipientTokenAccountInfoAfter = await splToken.getAccount(
      provider.connection,
      feeRecipientTokenAccount
    );

    console.log("Market Account After Withdrawal:", marketAccountAfter);
    console.log("Market Token Account After Withdrawal:", marketTokenAccountInfoAfter.amount.toString());
    console.log("Fee Recipient Token Account After Withdrawal:", feeRecipientTokenAccountInfoAfter.amount.toString());

    // Assertions
    //expect(marketAccountAfter.collectedFees.toNumber()).toBe(0);
    expect(Number(marketTokenAccountInfoAfter.amount)).toBe(
      Number(marketTokenAccountInfoBefore.amount) - feesToWithdraw
    );
    expect(Number(feeRecipientTokenAccountInfoAfter.amount)).toBe(
      Number(feeRecipientTokenAccountInfoBefore.amount) + feesToWithdraw
    );
  });





});