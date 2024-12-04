import * as anchor from '@coral-xyz/anchor'
import {Program} from '@coral-xyz/anchor'
import {Keypair} from '@solana/web3.js'
import {Statiadappsolana} from '../target/types/statiadappsolana'

describe('statiadappsolana', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)
  const payer = provider.wallet as anchor.Wallet

  const program = anchor.workspace.Statiadappsolana as Program<Statiadappsolana>

  const statiadappsolanaKeypair = Keypair.generate()

  it('Initialize Statiadappsolana', async () => {
    await program.methods
      .initialize()
      .accounts({
        statiadappsolana: statiadappsolanaKeypair.publicKey,
        payer: payer.publicKey,
      })
      .signers([statiadappsolanaKeypair])
      .rpc()

    const currentCount = await program.account.statiadappsolana.fetch(statiadappsolanaKeypair.publicKey)

    expect(currentCount.count).toEqual(0)
  })

  it('Increment Statiadappsolana', async () => {
    await program.methods.increment().accounts({ statiadappsolana: statiadappsolanaKeypair.publicKey }).rpc()

    const currentCount = await program.account.statiadappsolana.fetch(statiadappsolanaKeypair.publicKey)

    expect(currentCount.count).toEqual(1)
  })

  it('Increment Statiadappsolana Again', async () => {
    await program.methods.increment().accounts({ statiadappsolana: statiadappsolanaKeypair.publicKey }).rpc()

    const currentCount = await program.account.statiadappsolana.fetch(statiadappsolanaKeypair.publicKey)

    expect(currentCount.count).toEqual(2)
  })

  it('Decrement Statiadappsolana', async () => {
    await program.methods.decrement().accounts({ statiadappsolana: statiadappsolanaKeypair.publicKey }).rpc()

    const currentCount = await program.account.statiadappsolana.fetch(statiadappsolanaKeypair.publicKey)

    expect(currentCount.count).toEqual(1)
  })

  it('Set statiadappsolana value', async () => {
    await program.methods.set(42).accounts({ statiadappsolana: statiadappsolanaKeypair.publicKey }).rpc()

    const currentCount = await program.account.statiadappsolana.fetch(statiadappsolanaKeypair.publicKey)

    expect(currentCount.count).toEqual(42)
  })

  it('Set close the statiadappsolana account', async () => {
    await program.methods
      .close()
      .accounts({
        payer: payer.publicKey,
        statiadappsolana: statiadappsolanaKeypair.publicKey,
      })
      .rpc()

    // The account should no longer exist, returning null.
    const userAccount = await program.account.statiadappsolana.fetchNullable(statiadappsolanaKeypair.publicKey)
    expect(userAccount).toBeNull()
  })
})
