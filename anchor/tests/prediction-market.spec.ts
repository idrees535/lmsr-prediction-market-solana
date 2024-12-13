import * as anchor from '@coral-xyz/anchor';
import { BankrunProvider, startAnchor } from "anchor-bankrun";
import { Program } from '@coral-xyz/anchor';
import { SystemProgram, Keypair, Transaction, PublicKey, TransactionResponse, GetVersionedTransactionConfig } from '@solana/web3.js';
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
  let outcomeMints: Keypair[] = [];

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

    // Create outcome mints
    const outcomes = ["Outcome1", "Outcome2"];
    outcomeMints = await Promise.all(
      outcomes.map(async () => {
        const outcomeMint = Keypair.generate();
        const outcomeMintTransaction = new Transaction().add(
          SystemProgram.createAccount({
            fromPubkey: user.publicKey,
            newAccountPubkey: outcomeMint.publicKey,
            space: splToken.MINT_SIZE,
            lamports: await provider.connection.getMinimumBalanceForRentExemption(splToken.MINT_SIZE),
            programId: splToken.TOKEN_PROGRAM_ID,
          }),
          splToken.createInitializeMintInstruction(
            outcomeMint.publicKey,
            0, // Decimals
            user.publicKey, // Mint authority
            null // Freeze authority
          )
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

    const remainingAccounts = outcomeMints.map((mint) => ({
      pubkey: mint.publicKey,
      isWritable: true,
      isSigner: false,
    }));


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
        new anchor.BN(700),               // b
        new anchor.BN(3600),            // duration (1 hour)
        new anchor.BN(2),               // fee_percent
        user.publicKey,                 // fee_recipient dummy
        new anchor.BN(1000)             // initial_funds
      )
      .accounts(marketAccounts)
      .remainingAccounts(remainingAccounts)
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
    const transferTx = new anchor.web3.Transaction().add(
      splToken.createTransferInstruction(
        userTokenAccount,
        marketTokenAccount,
        user.publicKey,
        500
      )
    );
    if (provider.sendAndConfirm) {
      await provider.sendAndConfirm(transferTx, [user]);
      console.log("Transfer tokens to market's ATT tx:", { transferTx });
    }

    console.log("Transfer tokens to market's ATT");
    const marketAccountInfo_after = await splToken.getAccount(provider.connection, marketTokenAccount);
    console.log("market Token Account Info:", marketAccountInfo_after);

    const userBalanceBefore = ((await splToken.getAccount(provider.connection, userTokenAccount)).amount);
    const marketBalanceBefore = ((await splToken.getAccount(provider.connection, marketTokenAccount)).amount);

    // Derive the user's associated token account for Outcome 0 shares
    const buy_outcome_index = 0;
    const outcomeMint = outcomeMints[buy_outcome_index];
    const userShareAccount = await splToken.getAssociatedTokenAddress(
      outcomeMint.publicKey,
      user.publicKey
    );

    // Create the user's share ATA if it doesn't exist
   
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
      console.log('User share ATA creation transaction sent');
    }
    console.log("User's ATA created:", userShareAccount.toBase58())


    const userSahreAccountInfo_before = await splToken.getAccount(provider.connection, userShareAccount);
    console.log("User Sahre Token Account Info:", userSahreAccountInfo_before);


    console.log("finally let's try fucking do what we are here to do, buy shares");
    // Prepare the BuyShares instruction
    const buySharesAccounts = {
      market: marketPDA,
      buyerTokenAccount: userTokenAccount,
      marketTokenAccount: marketTokenAccount,
      outcomeMint: outcomeMint.publicKey,
      userShareAccount: userShareAccount,
      baseTokenMint: baseTokenMint.publicKey,
      buyer: user.publicKey,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
    };

    // Call the buy_shares function
    const buySharesTx = await marketProgram.methods
      .buyShares(new anchor.BN(buy_outcome_index), new anchor.BN(10)) // Buying 10 shares of Outcome 0
      .accounts(buySharesAccounts)
      .signers([user])
      .rpc();

    console.log("Buy Shares Transaction Signature:", buySharesTx);


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

    const marketAccount = await marketProgram.account.market.fetch(marketPDA);
    // Access and log specific fields
    console.log("Market Maker Funds:", marketAccount.marketMakerFunds.toNumber());
    console.log("Collected Fees:", marketAccount.collectedFees.toNumber());
    console.log("Total Shares for Outcome 0:", marketAccount.outcomes[0].totalShares.toNumber());


    // Add assertions to verify updates
    expect(marketAccount.marketMakerFunds.toNumber()).toBeGreaterThan(0);
    expect(marketAccount.collectedFees.toNumber()).toBeGreaterThan(0);
    expect(marketAccount.outcomes[0].totalShares.toNumber()).toBe(10);
    
  });
});
/*
  // Test 3: Sell Shares
  it("Can sell shares and update state correctly", async () => {
    // Ensure the user has shares to sell
    const sellerShareAccount = await splToken.getAssociatedTokenAddress(
      outcomeMints[0],
      user.publicKey
    );
    const sellerShareAccountInfoBefore = await splToken.getAccount(provider.connection, sellerShareAccount);
    console.log("Seller's Share Token Account Info Before Sell:", sellerShareAccountInfoBefore);
    expect(sellerShareAccountInfoBefore.amount.toNumber()).toBeGreaterThanOrEqual(10);

    // Capture user and market balances before selling
    const userBalanceBefore = (await splToken.getAccount(provider.connection, userTokenAccount)).amount;
    const marketBalanceBefore = (await splToken.getAccount(provider.connection, marketTokenAccount)).amount;

    console.log(`User balance before selling: ${userBalanceBefore}`);
    console.log(`Market balance before selling: ${marketBalanceBefore}`);

    // Prepare the SellShares instruction
    const sellSharesAccounts = {
      market: marketPDA,
      seller_token_account: userTokenAccount,
      market_token_account: marketTokenAccount,
      seller_share_account: sellerShareAccount,
      base_token_mint: baseTokenMint.publicKey,
      outcome_mint: outcomeMints[0],
      seller: user.publicKey,
      token_program: splToken.TOKEN_PROGRAM_ID,
      associated_token_program: splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
      system_program: SystemProgram.programId,
    };

    // Call the sell_shares function
    const sellSharesTx = await marketProgram.methods
      .sellShares(new anchor.BN(0), new anchor.BN(5)) // Selling 5 shares of Outcome 0
      .accounts(sellSharesAccounts)
      .signers([user])
      .rpc();

    console.log("Sell Shares Transaction Signature:", sellSharesTx);

    // Fetch the transaction to capture logs
    const tx = await provider.connection.getTransaction(sellSharesTx, { commitment: "confirmed" });
    if (tx?.meta?.logMessages) {
      tx.meta.logMessages.forEach(log => {
        if (log.includes("Cost After Selling:")) {
          console.log("Cost After Selling (Log):", log);
        }
        if (log.includes("Cost Before Selling:")) {
          console.log("Cost Before Selling (Log):", log);
        }
        if (log.includes("Fee Amount:")) {
          console.log("Fee Amount (Log):", log);
        }
        if (log.includes("Shares Sold:")) {
          console.log("Shares Sold (Log):", log);
        }
      });
    }

    // Fetch updated market account
    const marketAccountAfter = await marketProgram.account.market.fetch(marketPDA);
    console.log("Market Account After Sell:", marketAccountAfter);

    // Calculate expected cost using LMSR formula
    const q_after_sell = marketAccountAfter.outcomes.map(o => o.total_shares.toNumber());
    const b = marketAccountAfter.b.toNumber();
    const sum_exp_after_sell = q_after_sell.reduce((acc, cur) => acc + Math.exp(cur / b), 0);
    const expected_cost_after_sell = b * Math.log(sum_exp_after_sell);
    const expected_cost_after_sell_scaled = Math.round(expected_cost_after_sell * 1_000_000); // SCALE factor

    console.log("Expected Cost After Selling (Scaled):", expected_cost_after_sell_scaled);

    // Fetch user share account to verify shares burned
    const sellerShareAccountInfoAfter = await splToken.getAccount(provider.connection, sellerShareAccount);
    console.log("Seller's Share Token Account Info After Sell:", sellerShareAccountInfoAfter);

    // Calculate the actual cost difference (refund)
    const userBalanceAfter = (await splToken.getAccount(provider.connection, userTokenAccount)).amount;
    const marketBalanceAfter = (await splToken.getAccount(provider.connection, marketTokenAccount)).amount;

    console.log(`User balance before selling: ${userBalanceBefore}`);
    console.log(`User balance after selling: ${userBalanceAfter}`);
    console.log(`Market balance before selling: ${marketBalanceBefore}`);
    console.log(`Market balance after selling: ${marketBalanceAfter}`);

    const userBalanceDiff = userBalanceAfter - userBalanceBefore;
    const marketBalanceDiff = marketBalanceAfter - marketBalanceBefore;
    console.log(`User balance diff (refund received): ${userBalanceDiff}`);
    console.log(`Market balance diff (refund paid): ${marketBalanceDiff}`);

    // Calculate expected net refund (cost_difference - fee)
    const cost_before_sell = calculate_cost(q_after_sell, b).unwrap();
    const cost_before_sell_refund = cost_before_sell; // Already scaled
    const fee_amount = calculate_fee(cost_before_sell_refund, 200).unwrap();
    const reinvest_amount = Math.floor(fee_amount / 2);
    const fee_recipient_amount = fee_amount - reinvest_amount;
    const expected_net_refund = cost_before_sell_refund - fee_amount;

    console.log("Expected Net Refund After Selling (Scaled):", expected_net_refund);
    console.log("Actual Market Balance Diff (Refund):", marketBalanceDiff);

    // Verify that shares were burned correctly
    expect(sellerShareAccountInfoAfter.amount.toNumber()).toBe(sellerShareAccountInfoBefore.amount.toNumber() - 5);
    console.log(`User's share account balance after selling: ${sellerShareAccountInfoAfter.amount.toNumber()}`);

    // Verify user balance increased by net refund
    expect(userBalanceAfter).toBeGreaterThan(userBalanceBefore);
    console.log(`User balance before selling: ${userBalanceBefore}, after selling: ${userBalanceAfter}`);

    // Verify market balance decreased by net refund
    expect(marketBalanceAfter).toBeLessThan(marketBalanceBefore);
    console.log(`Market balance before selling: ${marketBalanceBefore}, after selling: ${marketBalanceAfter}`);

    // Assert that the marketMakerFunds and collectedFees have decreased/increased appropriately
    expect(marketAccountAfter.market_maker_funds.toNumber()).toBeLessThan(marketAccountAfter.market_maker_funds.toNumber());
    expect(marketAccountAfter.collected_fees.toNumber()).toBeGreaterThan(marketAccountAfter.collected_fees.toNumber());
    expect(marketAccountAfter.outcomes[0].total_shares.toNumber()).toBe(5); // After selling 5 shares
  });
});

});
 */