import * as anchor from '@coral-xyz/anchor'
import {Program} from '@coral-xyz/anchor'
import {Keypair,PublicKey} from '@solana/web3.js'
import {PredictionMarket} from '../target/types/PredictionMarket'

const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);

  const program = anchor.workspace.PredictionMarket as Program<PredictionMarket>;

  it("Can create a market", async () => {
    // Generate a new keypair for the user
    const user = Keypair.generate();

    // Airdrop SOL to user
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(user.publicKey, 2_000_000_000),
    );

    // Create a dummy mint as base token mint
    // For testing, just use a random key. In production, you'd create or pass a real SPL token mint.
    const baseTokenMint = Keypair.generate();
    
    // PDAs often determined by seeds, but here we rely on init account only.
    let [marketPDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("market"), new anchor.BN(12345).toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const tx = await program.methods
      .createMarket(
        new anchor.BN(12345),           // market_id
        "My Test Market",               // title
        ["Outcome1", "Outcome2"],       // outcomes
        new PublicKey("11111111111111111111111111111111"), // dummy oracle
        new anchor.BN(5),               // b
        new anchor.BN(3600),            // duration (1 hour)
        new anchor.BN(2),               // fee_percent
        new PublicKey("11111111111111111111111111111111"), // fee_recipient dummy
        new anchor.BN(1000)             // initial_funds
      )
      .accounts({
        market: marketPDA,
        user: user.publicKey,
        baseTokenMint: baseTokenMint.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId as PublicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY
      })
      .signers([user])
      .rpc();

    console.log("Your transaction signature", tx);

    const marketAccount = await program.account.market.fetch(marketPDA);
    console.log("Market Account:", marketAccount);
  });
