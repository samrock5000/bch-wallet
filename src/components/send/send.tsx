import {
  $,
  component$,
  // useTask$,
  useStore,
  useSignal,
  useContext,
  useContextProvider,
  useVisibleTask$,
  createContextId,
} from "@builder.io/qwik";
import { invoke } from "@tauri-apps/api/tauri";
import type { Utxo, Transaction, WalletData } from "../utils/utils";
import { writeText, readText } from '@tauri-apps/api/clipboard';
import {
  broadcast_transaction,
  build_p2pkh_transaction,
  validateAddr,
} from "../utils/utils";
import {
  // networkUrl,
  derivationPath,
  WalletContext,
  ContextSuccess,
} from "../../routes/layout";

export const satToBch = (x: number) => x / 100_000_000;
const TransactionDetails = createContextId<Transaction>("TxDetails");

  const badgeState = {
    empty: "indicator-item badge badge-neutral badge-xs opacity-50",
    warning: "indicator-item badge badge-warning badge-xs opacity-50",
    invalid: "indicator-item badge badge-error gap-2 opacity-50",
    valid: "indicator-item badge badge-success gap-2 opacity-50",
  };
  const borderClassStates = {
    empty:'border-2 border-nuetral-500 input-xs m-1 w-full max-w-xs ',
    warning:'border-2 border-warning-500 input-xs m-1 w-full max-w-xs',
    invalid:' border-2 border-rose-500 input-xs m-1 w-full max-w-xs',
    valid:' border-2 border-green-500  input-xs m-1 w-full max-w-xs '
  }
  const broadcastNotif = {
    success:
      "border-t-1 rounded-b bg-success px-4 py-3 text-teal-900 shadow-md",
    error: "alert alert-error flex ",
  };
  const tooltip = {
    noCoins: "no coins available",
    destinationParamsNotSet: "Provide destination address and amount",
  };
export default component$(() => {
  const showTxDetails = useSignal(false);
  const canCreateToken = useSignal(false);
  const borderStateAddress = useSignal(borderClassStates.empty);
  const borderStateAmount = useSignal(borderClassStates.empty);
  const store = useStore({
    destinationAddr: undefined as string | undefined,
    outgoingAmount: undefined as number | undefined,
    validAddr: false,
    amountValid: false,
    txReady: false,
    tokenAmountValid: false,
    validNFT: false,
    isTokenGenesisIndexAvailable: false,
    isTokenCreateChecked: false,
    buildTxErr: "",
    dustAmount: 0  as number,
  });

  const storeContext = useStore<WalletData>({
    masterKey: "",
    activeAddr: "",
    balance: 0,
    mnemonic: "",
    networkUrl: "",
    network: "",
    networkConnection: false,
    bip44Path: derivationPath,
    tokenUtxoBalance: 0,
    utxos: [],
    tokenUtxos: [],
  });
  const TxDetailsStore = useStore<Transaction>({
    inputs: [],
    outputs: [],
    txid: "",
  });

  const txStore = useStore({
    raw: undefined as string | undefined,
    txid: undefined as string | undefined,
    broadcastResponse: undefined as string | undefined,
    broadcastResponseisErr: false,
  });
  const tokenStore = useStore({
    amount: undefined as string | undefined,
    tokenChange: undefined as string | undefined,
    category: undefined as string | undefined,
    capability: undefined as string | undefined,
    commitment: undefined as string | undefined,
    tokenUtxos: undefined as Utxo[] | undefined,
  });
  const tokenGenesisStore = useStore({
    candidate: undefined as Utxo | undefined,
  });

  const decodeTransaction = $(
    async (transaction: string): Promise<Transaction> =>
      await invoke("decode_transaction", { transaction }),
  );

  const walletData = useContext(WalletContext);
  const contextSet = useContext(ContextSuccess);

  useVisibleTask$(({ track, cleanup }) => {
    const storeUpdated = track(() => contextSet.rdy);
    // const utxoloaded = setInterval(() => {
    if (storeUpdated) {
      storeContext.activeAddr = walletData.activeAddr;
      storeContext.networkConnection = walletData.networkConnection;
      storeContext.networkUrl = walletData.networkUrl;
      storeContext.balance = walletData.balance;
      storeContext.utxos = walletData.utxos;
      storeContext.bip44Path = walletData.bip44Path;
      storeContext.networkUrl!.concat(":50001");
      tokenGenesisStore.candidate = storeContext.utxos.filter(
        (e) => e.tx_pos == 0,
      )[0];
      if (tokenGenesisStore.candidate.tx_hash) {
        tokenStore.tokenUtxos = [tokenGenesisStore.candidate];

        tokenStore.category = tokenStore.tokenUtxos[0].tx_hash!;
      }

      store.isTokenGenesisIndexAvailable = tokenStore.tokenUtxos?.length != 0;
    }

    track(() => txStore.broadcastResponse);
    if (txStore.broadcastResponse) {
      const interval = setInterval(() => {
        txStore.broadcastResponse = undefined;
      }, 5000);
      cleanup(() => clearInterval(interval));
    }
  });
const validSatoshiAmount = $((amount:number)=>{
        return amount > store.dustAmount;
    });

  const build = $(() => {
    build_p2pkh_transaction(
      storeContext.bip44Path,
      store.destinationAddr!,
      storeContext.activeAddr,
      store.outgoingAmount!,
      //@ts-ignore
      tokenStore.category,
      //@ts-ignore
      tokenStore.amount,
      tokenStore.commitment,
      tokenStore.capability,
      storeContext.utxos,
      store.isTokenCreateChecked ? tokenStore.tokenUtxos : [],
    )
      .then((txbuildRes) => {
        const res = JSON.parse(txbuildRes as string);
        txStore.raw = res.rawTx;
        store.dustAmount = res.dust;
        // store.amountValid = true;
        store.buildTxErr = "";
        showTxDetails.value = true;
        decodeTransaction(txStore.raw as string).then((tx) => {
          TxDetailsStore.inputs = tx.inputs;
          TxDetailsStore.outputs = tx.outputs;
          TxDetailsStore.txid = tx.txid;

        });
      })
      .catch((error: string) => {
        store.buildTxErr = error;
        // store.amountValid = false;
        store.dustAmount = 0
        showTxDetails.value = false;
        console.error("BUILD P2PKH", error);
          if (store.buildTxErr.includes("Amount")) {
            const dustAmount = store.buildTxErr
              .split(" ")[4]
              .split(":")[1];
              store.dustAmount = parseInt(dustAmount,10);
              // store.amountValid = false;
              }
      });
  });

  const broadcast = $(async () =>
    broadcast_transaction(
      txStore.raw!,
      storeContext.networkUrl!.concat(":50001"),
    )
      .then(async (resp) => {
        txStore.broadcastResponse = (await resp) as any;

        console.log("SUCCESS", txStore.broadcastResponse);
      })
      .catch((error) => {
        txStore.broadcastResponse = error;
        txStore.broadcastResponseisErr = true;
        console.error(error);
      })
      .finally(() => {
        txStore.broadcastResponseisErr = false;
        contextSet.rdy = false;
        showTxDetails.value = false;
      }),
  );

  useContextProvider(TransactionDetails, TxDetailsStore);

  const validAddressInput = $((addr: string) => {
    validateAddr(addr)
      .then(() => {
        store.validAddr = true;
      })
      .catch((e) => {
        store.validAddr = false;
        console.error("store.outgoingAmountERRRR", e, store.destinationAddr);
      });
                   store.destinationAddr == undefined
                    ? borderStateAddress.value = borderClassStates.empty
                    : borderStateAddress.value = !store.validAddr
                    ? borderStateAddress.value = borderClassStates.invalid
                    : borderStateAddress.value =borderClassStates.valid

                    store.outgoingAmount == undefined
                    ? borderStateAmount.value = borderClassStates.empty
                    : borderStateAmount.value = !store.amountValid
                    ? borderStateAmount.value = borderClassStates.invalid
                    : borderStateAmount.value = borderClassStates.valid
  });
  return (
    <div class="max-[600px]: min-[320px]:text-center">
      <form>
        <label class="block">
          <span class="text-sm font-medium text-emerald-300">Send BCH</span>
          <div class="flex w-full flex-col border-opacity-50">
            <div>
              <input
                autoComplete={"off"}
                type="text"
                required
                class={
                  "input input-bordered"+ " " +
                    borderStateAddress.value
                }
                  onmouseOver$={async()=>{
                   let text = await readText() as string;
                     validateAddr(text).then(()=>{
                      store.destinationAddr = text as string
                      store.validAddr =  true
                      validAddressInput(store.destinationAddr!);
                    }).catch(()=>{
                      store.validAddr = false
                    })
                  
                  build();
                  }}
                oninput$={(ev) => {
                  store.destinationAddr = (
                    ev.target as HTMLInputElement
                  ).value.trim();
                  validAddressInput(store.destinationAddr);
                }}
                value={store.destinationAddr}
                placeholder="address"
              ></input>
            </div>
            <div>
              <div class="dropdown dropdown-hover ">
                <input
                  // maxlength={16}
                  // minlength={1}
                  class={"input input-bordered input-xs  " + 
                    borderStateAmount.value
                  }
                  type="number"
                  onInput$={async(ev) => {
                    store.outgoingAmount = 
                    (ev.target as HTMLInputElement).value == "" ?
                    undefined :
                      parseInt(
                      (ev.target as HTMLInputElement).value,
                      10) 
                  store.amountValid = await validSatoshiAmount(store.outgoingAmount!)!;
                    build();
                  }}
                  value={
                    !store.outgoingAmount ? undefined : store.outgoingAmount
                  }
                  //TODO use bch value
                  placeholder="satoshi value"
                ></input>
                <div class="text-xs text-primary">
                  {store.dustAmount ? <>Dust Minimum {store.dustAmount}</> : ""}
                </div>
                <ul
                  // @ts-ignore
                  tabindex="0"
                  class="menu dropdown-content z-[1] w-52 rounded-box bg-base-100 p-2 shadow"
                >
                  <li>
                    <button
                      // @ts-ignore
                      tabindex="0"
                      class="btn btn-outline btn-accent btn-xs  opacity-60"
                      preventdefault:click
                      onClick$={() => {
                        store.outgoingAmount = walletData.balance;
                        store.amountValid =
                          store.outgoingAmount <= walletData.balance &&
                          store.outgoingAmount >= 546
                            ? true
                            : false;
                        store.amountValid && store.validAddr
                          ? build()
                          : undefined;
                        // showTxDetails.value =
                        // store.amountValid && store.validAddr;
                      }}
                    >
                      MAX AMOUNT
                    </button>
                  </li>
                </ul>
              </div>
              <br />
              <div
                class={
                  ""
                }
              >
                <div class="grid justify-items-center">
                  <div class="form-control">
                    <div class="m-4 mt-10 rounded-lg bg-cyan-100/10 px-3 shadow-xl hover:bg-cyan-300/30">
                      <div
                        class={"tooltip"}
                        data-tip={
                          !store.isTokenGenesisIndexAvailable
                            ? tooltip.noCoins
                            : !store.amountValid && !store.validAddr
                            ? tooltip.destinationParamsNotSet
                            : "token options are available"
                        }
                      >
                        <label class="">
                          <span class="label-text text-secondary ">
                            Create CashToken
                          </span>

                          <input
                            disabled={!store.amountValid && !store.validAddr}
                            type="checkbox"
                            checked={store.isTokenCreateChecked} //{canCreateToken.value}
                            onClick$={() => {
                              store.isTokenCreateChecked =
                                store.isTokenCreateChecked == false
                                  ? true
                                  : false;
                              canCreateToken.value = store.isTokenCreateChecked;
                              tokenStore.amount = canCreateToken.value
                                ? tokenStore.amount
                                : undefined;
                              build();
                              if (!canCreateToken.value) {
                                tokenStore.capability = undefined;
                                tokenStore.commitment = undefined;
                                tokenStore.tokenUtxos = [];
                                store.isTokenCreateChecked = false;
                              } else {
                                tokenStore.tokenUtxos = [
                                  tokenGenesisStore.candidate!,
                                ];
                                tokenStore.category =
                                  tokenStore.tokenUtxos[0].tx_hash;
                              }
                            }}
                            class="checkbox-success checkbox checkbox-xs "
                          />
                        </label>
                      </div>
                    </div>
                  </div>
                </div>
                <div class=" justify-items-center">
                  {store.amountValid && store.validAddr ? (
                    !store.isTokenCreateChecked ? (
                      <></>
                    ) : (
                      <div class="">
                        <div>
                          <div>
                            <input
                              type="text"
                              required
                              class="input input-bordered input-xs w-full max-w-xs"
                              onInput$={(ev) => {
                                //@ts-ignore
                                tokenStore.amount = (
                                  ev.target as HTMLInputElement
                                ).value;
                                const amount = canCreateToken.value
                                  ? tokenStore.amount
                                  : undefined;

                                invoke("valid_token_amount", { amount })
                                  .then(() => {
                                    store.tokenAmountValid = true;
                                    tokenStore.amount = amount;
                                    // showTxDetails.value =
                                    // store.amountValid &&
                                    // store.validAddr &&
                                    // store.tokenAmountValid;
                                    build();
                                  })
                                  .catch((e) => {
                                    store.tokenAmountValid = false;
                                    tokenStore.amount = undefined;
                                    console.error(e);
                                  });
                              }}
                              value={tokenStore.amount}
                              placeholder="Token Amount"
                            ></input>
                            <div
                              class={
                                // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
                                tokenStore.amount == undefined
                                  ? badgeState.empty
                                  : !store.tokenAmountValid
                                  ? badgeState.invalid
                                  : badgeState.valid
                              }
                            ></div>
                          </div>
                          <select
                            class="select select-bordered select-xs w-full max-w-xs"
                            onInput$={(ev) => {
                              //commitment cant be undefined if capability is set
                              tokenStore.commitment = "";
                              //@ts-ignore
                              tokenStore.capability = (
                                ev.target as HTMLInputElement
                              ).value;
                              // build();
                              const commitment = tokenStore.commitment;
                              const capability = tokenStore.capability;
                              invoke("valid_nft", { commitment, capability })
                                .then(() => {
                                  store.validNFT = true;
                                  build();
                                })
                                .catch((e) => {
                                  store.validNFT = false;
                                  console.error(e);
                                });
                            }}
                          >
                            <option disabled selected>
                              Capability
                            </option>
                            <option value="none">None</option>
                            <option value="mutable">Mutable</option>
                            <option value="minting">Minting</option>
                          </select>
                          <div
                            class={
                              // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
                              tokenStore.capability == undefined
                                ? badgeState.empty
                                : !store.validNFT
                                ? badgeState.invalid
                                : badgeState.valid
                            }
                          ></div>
                          <div>
                            <input
                              disabled={
                                tokenStore.capability == undefined
                                  ? true
                                  : false
                              }
                              maxLength={40}
                              type="text"
                              required
                              class="input input-bordered input-xs w-full max-w-xs"
                              onInput$={(ev) => {
                                //@ts-ignore
                                tokenStore.commitment = (
                                  ev.target as HTMLInputElement
                                ).value;
                                const commitment =
                                  canCreateToken.value &&
                                  tokenStore.capability != undefined
                                    ? tokenStore.commitment
                                    : undefined;

                                const capability = canCreateToken.value
                                  ? tokenStore.capability
                                  : undefined;
                                invoke("valid_nft", { commitment, capability })
                                  .then(() => {
                                    store.validNFT = true;
                                    build();
                                  })
                                  .catch((e) => {
                                    store.validNFT = false;
                                    console.error(e);
                                  });
                              }}
                              value={tokenStore.commitment}
                              placeholder="commitment"
                            ></input>
                            <div
                              class={
                                // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
                                tokenStore.commitment == undefined
                                  ? badgeState.empty
                                  : !store.validNFT
                                  ? badgeState.invalid
                                  : badgeState.valid
                              }
                            ></div>
                          </div>
                        </div>
                      </div>
                    )
                  ) : (
                    <></>
                  )}
                </div>
              </div>

              <br />
            </div>
          </div>
        </label>
      </form>

      <div class="">
        <div class="break-words">
          {!showTxDetails.value ? (
            <></>
          ) : (
            <div class="container mx-auto my-5 outline-1  outline-white ">
              <div class="">
                {" "}
                <TxInfo />{" "}
              </div>
            </div>
          )}
        </div>

        <div></div>
        {/* TXID: */}
        {txStore.broadcastResponse && (
          <div class="toast ">
            <div
              class={
                txStore.broadcastResponseisErr
                  ? broadcastNotif.error
                  : broadcastNotif.success
              }
              // role="alert"
            >
              {/* <p class="font-bold">{Transacton}</p> */}
              <p class="text-sm">{txStore.broadcastResponse}.</p>
            </div>
          </div>
        )}
        {/* <p>{txStore.broadcastResponse}</p> */}
        <dialog id="txsendcheck" class="modal modal-bottom sm:modal-middle">
          <div class="modal-box">
            <p class="py-4">
              Are you sure you want to send {store.outgoingAmount} to{" "}
              {store.destinationAddr}
            </p>
            <div class="modal-action">
              <form method="dialog">
                {/*   <!-- if there is a button in form, it will close the modal --> */}
                <button class="btn  btn-outline btn-error relative">
                  Cancel
                </button>
                <button
                  onClick$={() =>
                    broadcast()
                      .then(() => {
                        store.destinationAddr = "";
                        store.outgoingAmount = 0;
                        tokenStore.amount = undefined;
                        tokenStore.capability = undefined;
                        tokenStore.commitment = undefined;
                        // showTxDetails.value = false;
                      })
                      .catch((e) => {
                        console.error(e);
                      })
                  }
                  class="btn pl-4"
                >
                  Send
                </button>
              </form>
            </div>
          </div>
        </dialog>
      </div>
    </div>
  );
});

export const TxInfo = component$(() => {
  const transaction = useContext(TransactionDetails);
  return (
    <>
      <div class=" break-words">
        <div class="rounded-lg bg-neutral text-neutral-content ">
          <div class="">
            {/* <figure></figure> */}
            <div>
              <h1 class="bg-auto text-xl text-accent">Transaction Details</h1>
              <label class="text-xs text-accent">Transaction ID: </label>
              <span class="text-xs">{transaction.txid}</span>
              <div>
                <p>Coin Selection</p>
                <span>Inputs (Unspent Coins):</span>
                <div class=" mx-1 my-1  ">
                  {transaction.inputs.map((input, index) => (
                    <div class="" key={index}>
                      <label class="text-xs text-accent"> Utxo Hash : </label>
                      <span class="text-xs">
                        <span>{input.prevout}</span>
                      </span>
                      <div>
                        <label class="text-xs text-accent "> Index : </label>
                        <span class="text-xs">
                          <span>{input.index}</span>
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
              <div>
                <span>Outputs:</span>
                {transaction.outputs.map((output, index) => (
                  <div key={index}>
                    <div>
                      <span class="text-xs">
                        <label class="text-xs text-accent"> Address: </label>
                        <span>{output.script}</span>
                      </span>
                    </div>
                    <div>
                      <span class="text-xs">
                        <label class="text-xs text-accent"> Amount: </label>
                        <span>{output.amount}</span>
                      </span>
                    </div>
                    {!output.token ? (
                      <></>
                    ) : (
                      <div>
                        <div class="text-xs">
                          <label class="text-xs text-secondary">
                            {" "}
                            Token Amount:{" "}
                          </label>
                          <span>{output.token.amount}</span>
                        </div>
                        <div class="text-xs">
                          <label class="text-xs text-secondary">
                            {" "}
                            Category:{" "}
                          </label>
                          <span>{output.token.category}</span>
                        </div>
                        {output.token.nft ? (
                          <div class="text-xs">
                            <h1 class="text-xs text-primary"> NFT: </h1>
                            <label class="text-xs text-secondary">
                              {" "}
                              Capability:{" "}
                            </label>
                            <span>{output.token.nft?.capability}</span>
                            <label class="text-xs text-secondary">
                              {" "}
                              Commitment:{" "}
                            </label>
                            <span>{output.token.nft?.commitment}</span>
                          </div>
                        ) : (
                          <></>
                        )}
                      </div>
                    )}
                  </div>
                ))}
              </div>

              <div class=" justify-center p-5">
                <button
                  onClick$={() => {
                    //@ts-ignore
                    document.getElementById("txsendcheck").showModal();
                  }}
                  type="button"
                  value={"BROADCAST"}
                  class="btn btn-primary"
                >
                  {" "}
                  Send Transaction
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  );
});
