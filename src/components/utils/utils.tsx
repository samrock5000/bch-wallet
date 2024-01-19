import { $ } from "@builder.io/qwik";
import { invoke } from "@tauri-apps/api/tauri";

export const validateAddr = $(async (address: string) => {
  await invoke("validate_cash_address", {
    address,
  });
});
export const updateNeworkUrl = $((networkUrl: string) => {
  invoke("update_network_url", { networkUrl });
});
export const validTokenAmount = $(
  async (amount: string) => await invoke("valid_token_amount", { amount }),
);
export const addressFromHdPath = $(
  async (path: string, network: string) =>
    await invoke("address_from_hdpath", {
      path,
      network,
    }),
);

export const build_p2pkh_transaction = $(
  async (
    derivationPath: string,
    destinationAddress: string,
    sourceAddress: string,
    amount: number,
    category: string | undefined,
    tokenAmount: string | undefined,
    commitment: string | undefined,
    capability: string | undefined,
    utxos: Utxo[],
    requiredUtxos: Utxo[] | undefined,
  ) =>
    await invoke("build_p2pkh_transaction", {
      derivationPath,
      destinationAddress,
      sourceAddress,
      amount,
      category,
      tokenAmount,
      commitment,
      capability,
      utxos,
      requiredUtxos,
    }),
);
export type MemStore = {
  db: WalletData;
};
export type WalletConfig = {
  network: string;
  networkUrl: string;
};

export const doesWalletExist = $(() => invoke("wallet_exist"));
export const decodeTransaction = $(
  async (transaction: string): Promise<Transaction> =>
    await invoke("decode_transaction", { transaction }),
);
export const loadConfig = $((address: string) => {
  invoke("get_store_config", { address });
});
export const saveConfig = $((address: string, walletConf: WalletConfig) => {
  invoke("save_config", { address, walletConf });
});
export const walletCache = $((): Promise<MemStore> => invoke("wallet_cache"));
export const getUtxos = $((address: string) =>
  invoke("utxo_cache", { address }),
);
export const networkPing = $((networkUrl: string) =>
  invoke("network_ping", { networkUrl }),
);

export const broadcast_transaction = $(
  async (transaction: string, networkUrl: string) => {
    return await invoke("broadcast_transaction", {
      transaction,
      networkUrl,
    });
  },
);

// export const utxoSum = (utxo:Utxo[]) => utxo.reduce((sum, outputs)=> sum + outputs.value ,0);
export type WalletInit = {
  mnemonic: string;
};
export type NetworkUrlUpdate = {
  url: string;
  urls: string[];
};
export type KeySetUp = {
  isSet: boolean;
};
export type WalletData = {
  masterKey: string;
  balance: number;
  tokenSatoshiBalance: number;
  address: string;
  network: string;
  networkUrl: string | null;
  // networkConnection: boolean;
  mnemonic: string;
  bip44Path: string;
  utxos: Utxo[];
  tokenUtxos: Utxo[];
  // walletExist: boolean;
};

/* export type TokenUtxo {
  amount:number,
  category:string,
  nft
}

 */

export type Utxo = {
  tx_hash: string;
  height: number;
  token_data: Token | null;
  tx_pos: number;
  value: number;
};

export type Input = {
  index: number;
  prevout: string;
};

export type Output = {
  amount: number;
  script: string;
  token: Token | null;
};

export type Token = {
  category: string;
  amount: number;
  nft: NFT | undefined;
};
export type ElectrumToken = {
  amount: string;
  category: string;
};
export type History = {
  height: number;
  tx_hash: string;
};

export type Transaction = {
  inputs: Input[];
  outputs: Output[];
  txid: string;
};
export type NFT = {
  capability: string;
  commitment: string;
};
