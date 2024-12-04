'use client'

import {getStatiadappsolanaProgram, getStatiadappsolanaProgramId} from '@project/anchor'
import {useConnection} from '@solana/wallet-adapter-react'
import {Cluster, Keypair, PublicKey} from '@solana/web3.js'
import {useMutation, useQuery} from '@tanstack/react-query'
import {useMemo} from 'react'
import toast from 'react-hot-toast'
import {useCluster} from '../cluster/cluster-data-access'
import {useAnchorProvider} from '../solana/solana-provider'
import {useTransactionToast} from '../ui/ui-layout'

export function useStatiadappsolanaProgram() {
  const { connection } = useConnection()
  const { cluster } = useCluster()
  const transactionToast = useTransactionToast()
  const provider = useAnchorProvider()
  const programId = useMemo(() => getStatiadappsolanaProgramId(cluster.network as Cluster), [cluster])
  const program = getStatiadappsolanaProgram(provider)

  const accounts = useQuery({
    queryKey: ['statiadappsolana', 'all', { cluster }],
    queryFn: () => program.account.statiadappsolana.all(),
  })

  const getProgramAccount = useQuery({
    queryKey: ['get-program-account', { cluster }],
    queryFn: () => connection.getParsedAccountInfo(programId),
  })

  const initialize = useMutation({
    mutationKey: ['statiadappsolana', 'initialize', { cluster }],
    mutationFn: (keypair: Keypair) =>
      program.methods.initialize().accounts({ statiadappsolana: keypair.publicKey }).signers([keypair]).rpc(),
    onSuccess: (signature) => {
      transactionToast(signature)
      return accounts.refetch()
    },
    onError: () => toast.error('Failed to initialize account'),
  })

  return {
    program,
    programId,
    accounts,
    getProgramAccount,
    initialize,
  }
}

export function useStatiadappsolanaProgramAccount({ account }: { account: PublicKey }) {
  const { cluster } = useCluster()
  const transactionToast = useTransactionToast()
  const { program, accounts } = useStatiadappsolanaProgram()

  const accountQuery = useQuery({
    queryKey: ['statiadappsolana', 'fetch', { cluster, account }],
    queryFn: () => program.account.statiadappsolana.fetch(account),
  })

  const closeMutation = useMutation({
    mutationKey: ['statiadappsolana', 'close', { cluster, account }],
    mutationFn: () => program.methods.close().accounts({ statiadappsolana: account }).rpc(),
    onSuccess: (tx) => {
      transactionToast(tx)
      return accounts.refetch()
    },
  })

  const decrementMutation = useMutation({
    mutationKey: ['statiadappsolana', 'decrement', { cluster, account }],
    mutationFn: () => program.methods.decrement().accounts({ statiadappsolana: account }).rpc(),
    onSuccess: (tx) => {
      transactionToast(tx)
      return accountQuery.refetch()
    },
  })

  const incrementMutation = useMutation({
    mutationKey: ['statiadappsolana', 'increment', { cluster, account }],
    mutationFn: () => program.methods.increment().accounts({ statiadappsolana: account }).rpc(),
    onSuccess: (tx) => {
      transactionToast(tx)
      return accountQuery.refetch()
    },
  })

  const setMutation = useMutation({
    mutationKey: ['statiadappsolana', 'set', { cluster, account }],
    mutationFn: (value: number) => program.methods.set(value).accounts({ statiadappsolana: account }).rpc(),
    onSuccess: (tx) => {
      transactionToast(tx)
      return accountQuery.refetch()
    },
  })

  return {
    accountQuery,
    closeMutation,
    decrementMutation,
    incrementMutation,
    setMutation,
  }
}
