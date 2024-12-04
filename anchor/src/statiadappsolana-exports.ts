// Here we export some useful types and functions for interacting with the Anchor program.
import { AnchorProvider, Program } from '@coral-xyz/anchor'
import { Cluster, PublicKey } from '@solana/web3.js'
import StatiadappsolanaIDL from '../target/idl/statiadappsolana.json'
import type { Statiadappsolana } from '../target/types/statiadappsolana'

// Re-export the generated IDL and type
export { Statiadappsolana, StatiadappsolanaIDL }

// The programId is imported from the program IDL.
export const STATIADAPPSOLANA_PROGRAM_ID = new PublicKey(StatiadappsolanaIDL.address)

// This is a helper function to get the Statiadappsolana Anchor program.
export function getStatiadappsolanaProgram(provider: AnchorProvider) {
  return new Program(StatiadappsolanaIDL as Statiadappsolana, provider)
}

// This is a helper function to get the program ID for the Statiadappsolana program depending on the cluster.
export function getStatiadappsolanaProgramId(cluster: Cluster) {
  switch (cluster) {
    case 'devnet':
    case 'testnet':
      // This is the program ID for the Statiadappsolana program on devnet and testnet.
      return new PublicKey('CounNZdmsQmWh7uVngV9FXW2dZ6zAgbJyYsvBpqbykg')
    case 'mainnet-beta':
    default:
      return STATIADAPPSOLANA_PROGRAM_ID
  }
}
