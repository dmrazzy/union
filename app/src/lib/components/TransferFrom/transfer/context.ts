import { derived, type Readable } from "svelte/store"
import type { Chain, UserAddresses } from "$lib/types"
import type { Address } from "$lib/wallet/types"
import { balanceStore, userAddress } from "./balances.ts"
import type { BalanceResult } from "$lib/queries/balance"
import type { QueryObserverResult } from "@tanstack/query-core"

export type BalanceRecord = {
  balance: bigint
  gasToken: boolean
  address: Address
  symbol: string
}

export type ChainBalances = {
  chainId: string
  balances: Array<BalanceRecord>
}

export type BalancesList = Array<ChainBalances>

export interface ContextStore {
  chains: Array<Chain>
  userAddress: UserAddresses
  balances: BalancesList
}

export function createContextStore(chains: Array<Chain>): Readable<ContextStore> {
  const balances = derived(
    balanceStore as Readable<Array<QueryObserverResult<Array<BalanceResult>, Error>>>,
    $rawBalances => {
      if ($rawBalances?.length === 0) {
        return chains.map(chain => ({
          chainId: chain.chain_id,
          balances: []
        }))
      }

      return chains.map((chain, chainIndex) => {
        const balanceResult = $rawBalances[chainIndex]

        if (!(balanceResult?.isSuccess && balanceResult.data)) {
          console.log(`No balances fetched yet for chain ${chain.chain_id}`)
          return {
            chainId: chain.chain_id,
            balances: []
          }
        }

        return {
          chainId: chain.chain_id,
          balances: balanceResult.data.map((balance: BalanceResult) => ({
            ...balance,
            balance: BigInt(balance.balance),
            gasToken: "gasToken" in balance ? (balance.gasToken ?? false) : false,
            address: balance.address as Address,
            symbol: balance.symbol || balance.address
          }))
        }
      })
    }
  ) as Readable<BalancesList>

  return derived([userAddress, balances], ([$userAddress, $balances]) => ({
    chains,
    userAddress: $userAddress,
    balances: $balances
  }))
}