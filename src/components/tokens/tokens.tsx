import {
  $,
  component$,
  createContextId,
  useContext,
  useContextProvider,
  useSignal,
  useStore,
  useTask$,
  useComputed$,
  useVisibleTask$,
  useOnDocument,
} from "@builder.io/qwik";
import { invoke } from "@tauri-apps/api/tauri";
import { writeText, readText } from '@tauri-apps/api/clipboard';
import { WalletContext, TokenUtxos, ContextSuccess } from "../../routes/layout";
import {
  validateAddr,
  // type Token,
  type Utxo,
  validTokenAmount,
  build_p2pkh_transaction,
  type Transaction,
  decodeTransaction,
  broadcast_transaction,
  // binToUtf8,
  // hexToBytes,
  // Token,
} from "../../components/utils/utils";
// import { BchLogo } from "~/components/svg/bch-logo";

const TransactionDetails = createContextId<Transaction>("TxDetails");

export default component$(() => {
  const txDetailsStore = useStore<Transaction>({
    inputs: [],
    outputs: [],
    txid: "",
  });
  const store = useStore({
    destinationAddress: "",
    sourceAddress: "",
    tokenAddress: "",
    amount: 0,
    commitment: undefined,
    capability: undefined,
    tokenBalance: 0,
    tokenUtxoSatoshiBalance: 0,
    tokensUtxos: [] as Utxo[],
    utxos: [] as Utxo[],
    categorySend: "",
    tokenSendAmount: "",
    contextReady: false,
    derivationPath: "",
  });

  const sendToModalStore = useStore<Utxo>({
    height: 0,
    tx_hash: "",
    token_data: {
      amount: 0,
      category: "",
      nft: { capability: "", commitment: "" },
    },
    tx_pos: 0,
    value: 0,
  });
  const copyEvent = useSignal("copytext")

  // Debouncer task
  // useTask$(({ track }) => {
  //   const someInputEvent = track(() => store.doubleCount);
  //   const timer = setTimeout(() => {
  //     store.debounced = someInputEvent;
  //   }, 2000);
  //   return () => {
  //     clearTimeout(timer);
  //   };
  // });

  const sendToken = useSignal(false);
  const walletData = useContext(WalletContext);
  const contextSet = useContext(ContextSuccess);
  useVisibleTask$(({ track }) => {
    // const utxoloaded = setInterval(() => {
    store.utxos = walletData.utxos;
    store.tokensUtxos = walletData.tokenUtxos;
    store.derivationPath = walletData.bip44Path;
    store.tokenUtxoSatoshiBalance = walletData.tokenUtxoBalance;
    const storeUpdated = track(() => contextSet.rdy);
    if (storeUpdated) {
      const address = walletData.activeAddr;
      invoke("token_cash_address", { address })
        .then((addr) => {
          store.tokenAddress = addr as string;
        })
        .catch((e) => {
          console.error(e);
        });
    }
    /*
      if (contextSet.rdy) {
        const address = walletData.activeAddr;
        invoke("token_cash_address", { address })
          .then((addr) => {
            store.tokenAddress = addr as string;
          })
          .catch((e) => {
            console.error(e);
          });
        console.log("token data ready", store.tokensUtxos);
        clearInterval(utxoloaded);
      }
    }, 10);
    */
  });

  useContextProvider(TransactionDetails, txDetailsStore);
  useContextProvider(TokenUtxos, store.tokensUtxos);

  //Reset Values when modal open/close
  useOnDocument(
    "close",
    $((/* event */) => {
      sendToken.value = false;
      sendToModalStore.height = 0;
      sendToModalStore.token_data = {
        amount: 0,
        category: "",
        nft: { capability: "", commitment: "" },
      };
      sendToModalStore.tx_pos = 0;
      sendToModalStore.value = 0;
    }),
  );
  return (
    <div>
      <div class="max-[600px]: min-[320px]:text-center">
        {" "}
        <h1 class="text-sm text-accent">Token Address </h1>
        <p 
          onClick$={async()=> {
            copyEvent.value = "copytext copied"
            await writeText(store.tokenAddress);
            setTimeout(()=> {copyEvent.value = "copytext"},500)
        
          }}
          class={copyEvent.value + " " + "break-all"}>{store.tokenAddress}</p>
        {/* <h1 class="text-sm text-accent">Total BCH Token Value </h1> */}
        {/* {store.tokenUtxoSatoshiBalance} */}
        <br />
        <h1 class="m-1"> Cash Tokens</h1>
        <div class="">
          {store.tokensUtxos.map((utxo, index) =>
            // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
            utxo != null ? (
              <div key={index}>
                <div class=" flex justify-center">
                  <div class="card my-3 w-96 bg-gray-900 shadow-xl">
                    {/* <BchLogo height={100} width={100} /> */}
                    <div class="card-body">
                      <div class="grid h-10 grid-cols-2 place-content-evenly gap-2 ">
                        <p class="text-sm text-secondary">
                          Token Amount:{" "}
                          <span class="text-white">
                            {" "}
                            {utxo.token_data!.amount}
                          </span>
                        </p>

                        <p class="text-sm text-secondary">
                          Satoshi Value:{" "}
                          <span class="text-white">{utxo.value}</span>
                        </p>
                      </div>

                      <label class="text-sm font-medium text-secondary">
                        Category id
                      </label>
                      <p class="break-all">{utxo.token_data!.category} </p>
                      {utxo.token_data?.nft ? (
                        <div class="card-actions justify-end">
                          <label class="text-sm font-medium text-secondary">
                            Capability
                          </label>
                          <p class="break-all">
                            {utxo.token_data!.nft?.capability}{" "}
                          </p>

                          <label class="text-sm font-medium text-secondary">
                            Commitment
                          </label>
                          <p class="break-all">
                            {utxo.token_data!.nft?.commitment}{" "}
                          </p>
                        </div>
                      ) : (
                        <> </>
                      )}
                      <div class="card-actions justify-end">
                        <button
                          type="button"
                          onClick$={() => {
                            sendToModalStore.value = utxo.value;
                            sendToModalStore.tx_hash = utxo.tx_hash;
                            sendToModalStore.tx_pos = utxo.tx_pos;
                            sendToModalStore.token_data!.amount =
                              utxo.token_data!.amount;
                            sendToModalStore.token_data!.category =
                              utxo.token_data!.category;
                            if (utxo.token_data!.nft !== undefined) {
                              //@ts-ignore
                              sendToModalStore.token_data.nft.commitment =
                                // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
                                utxo.token_data!.nft == undefined
                                  ? undefined
                                  : utxo.token_data!.nft.commitment;
                              //@ts-ignore
                              sendToModalStore.token_data.nft.capability =
                                // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
                                utxo.token_data!.nft == undefined
                                  ? undefined
                                  : utxo.token_data!.nft.capability;
                              console.log("sendToModalStore", sendToModalStore);
                            }

                            if (utxo.token_data!.nft == undefined) {
                              sendToModalStore.token_data!.nft = undefined;
                            }

                            sendToken.value = true;
                            setTimeout(() => {
                              //@ts-ignore
                              document
                                .getElementById("tokensendcheck")
                                //@ts-ignore
                                .showModal();
                            }, 50);
                          }}
                          class="btn btn-outline text-emerald-300"
                        >
                          Send
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            ) : (
              <div key={index}></div>
            ),
          )}
        </div>
      </div>
      {!sendToken.value ? <></> : <SendTokenModal {...sendToModalStore} />}{" "}
    </div>
  );
});

export const SendTokenModal = component$((props: Utxo) => {
  const createTokenDetails = useSignal(false);
  const txDetails = useContext(TransactionDetails);
  const walletData = useContext(WalletContext);

  const errStore = useStore({
    build: "",
  });

  const store = useStore({
    derivationPath: walletData.bip44Path,
    srcAddress: walletData.activeAddr,
    availableTokenSatoshiAmount: props.value,
    destinationAddress: "",
    tokenSendAmount: undefined as string | undefined,
    availableTokenAmount: 0,
    category: undefined as string | undefined,
    commitment: undefined as string | undefined,
    capability: undefined as string | undefined,
    satoshiSendAmount: 0,
    tokenAmountValid: false,
    amountValid: false,
    validAddr: false,
    utxos: [] as Utxo[],
    tokenUtxos: walletData.tokenUtxos,
    tokenRequiredUtxos: [] as Utxo[],
    broadcastRes: "",
    balance: walletData.balance,
    rawTx: "",
    buildIsOk: false,
    broadcastErr: false,
  });

  useVisibleTask$(({ track }) => {
    store.availableTokenAmount = props.token_data!.amount;
    store.category = props.token_data!.category;
    store.srcAddress = walletData.activeAddr;
    store.derivationPath = walletData.bip44Path;
    store.commitment =
      props.token_data!.nft == undefined
        ? undefined
        : props.token_data!.nft.commitment;
    // : binToUtf8(hexToBytes(props.token_data!.nft.commitment));
    store.capability =
      props.token_data!.nft == undefined
        ? undefined
        : props.token_data!.nft.capability;

    store.utxos = walletData.utxos;
    store.tokenUtxos = walletData.tokenUtxos;
    store.availableTokenSatoshiAmount = props.value;
    store.tokenRequiredUtxos = [props];
    console.log("REQUIRED UTXO", store.tokenRequiredUtxos);
    const shouldClose = track(() => store.broadcastRes);
    if (shouldClose) {
      //@ts-ignore
      document
        .getElementById("tokensendcheck")
        //@ts-ignore
        .close();
    }
  });

  const build = $(() => {
    build_p2pkh_transaction(
      store.derivationPath,
      store.destinationAddress,
      store.srcAddress,
      store.satoshiSendAmount,
      store.category,
      store.tokenSendAmount,
      store.commitment,
      store.capability,
      store.utxos as [],
      store.tokenRequiredUtxos,
    )
      .then((txBuild) => {
        const res = JSON.parse(txBuild as string);
        store.buildIsOk = true;
        store.rawTx = res.rawTx as string;
        decodeTransaction(store.rawTx as string)
          .then((tx) => {
            txDetails.inputs = tx.inputs;
            txDetails.outputs = tx.outputs;
            txDetails.txid = tx.txid;
            createTokenDetails.value = true;
          })
          .catch((e) => {
            console.log(e);
          });
      })
      .catch((e) => {
        store.buildIsOk = false;
        errStore.build = e;
        console.error("errStore.build:", errStore.build);
        createTokenDetails.value = false;
      });
  });

  const badgeState = {
    empty: "badge badge-neutral badge-xs opacity-50",
    invalid: "badge badge-error  gap-2 opacity-50",
    valid: "badge badge-success gap-2 opacity-50",
  };
  const broadcastNotif = {
    success:
      "border-t-1 rounded-b bg-success px-4 py-3 text-teal-900 shadow-md",
    error: "alert alert-error flex ",
  };

  // const build =
  return (
    <div>
      {store.broadcastRes && (
        <div class="toast ">
          <div
            class={
              store.broadcastErr ? broadcastNotif.error : broadcastNotif.success
            }
            // role="alert"
          >
            {/* <p class="font-bold">{Transacton}</p> */}
            <p class="text-sm">{store.broadcastRes}.</p>
          </div>
        </div>
      )}
      <dialog id="tokensendcheck" class="modal modal-bottom sm:modal-middle">
        <div class="modal-box">
          <input
            // preventdefault:paste
            autoCapitalize="off"
            autoCorrect="off"
            type="text"
            required
            class="input input-bordered input-xs m-1 w-full max-w-xs"
            onInput$={(ev) => {
              store.destinationAddress = (
                ev.target as HTMLInputElement
              ).value.trim();
              validateAddr(store.destinationAddress)
                .then(() => {
                  store.validAddr = true;
                })
                .catch((e) => {
                  store.validAddr = false;
                  console.error(e);
                });
              //Using decoders shift props.nft.commitment to address props somehow
              build();
            }}
            value={store.destinationAddress}
            placeholder="address"
          ></input>

          <div
            class={
              store.destinationAddress == ""
                ? badgeState.empty
                : !store.validAddr
                ? badgeState.invalid
                : badgeState.valid
            }
          ></div>
          <h2>
            Available Tokens: <span>{store.availableTokenAmount}</span>
          </h2>
          {props.token_data?.nft != undefined ? (
            <div>
              <label>Fungible</label>
              <input
                onClick$={() => {
                  store.capability = undefined;
                  store.commitment = undefined;
                  build();
                }}
                type="checkbox"
              ></input>
            </div>
          ) : (
            <></>
          )}
          <input
            type="text"
            required
            class="input input-bordered input-xs w-full max-w-xs"
            onInput$={(ev) => {
              //@ts-ignore
              store.tokenSendAmount = (ev.target as HTMLInputElement).value;
              validTokenAmount(store.tokenSendAmount)
                .then(() => {
                  store.tokenAmountValid = true;
                })
                .catch((e) => {
                  store.tokenAmountValid = false;

                  console.error(e);
                });
              store.tokenAmountValid =
                store.availableTokenAmount >=
                parseInt(store.tokenSendAmount, 10)
                  ? true
                  : false;
              store.tokenSendAmount = store.tokenAmountValid
                ? store.tokenSendAmount
                : undefined;

              build();
            }}
            value={store.tokenSendAmount}
            placeholder="TokenAmount"
          ></input>
          <div
            class={
              // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
              store.tokenSendAmount == undefined
                ? badgeState.empty
                : !store.tokenAmountValid
                ? badgeState.invalid
                : badgeState.valid
            }
          ></div>
          <div class="dropdown dropdown-hover ">
            <div>
              <h1>Satoshi Amount</h1>
              <input
                // maxLength={16}
                // minLength={0}
                class="input input-bordered input-xs  "
                type="number"
                onInput$={(ev) => {
                  store.satoshiSendAmount = parseInt(
                    (ev.target as HTMLInputElement).value,
                    10,
                  );
                  console.log("AMOUNT SAT SEND", store.satoshiSendAmount);
                  store.amountValid =
                    store.satoshiSendAmount <= walletData.balance &&
                    store.satoshiSendAmount >= 546
                      ? true
                      : false;
                  console.log("store.amountValid", store.amountValid);
                  createTokenDetails.value =
                    store.amountValid && store.validAddr ? true : false;

                  build();
                }}
                value={
                  !store.satoshiSendAmount ? undefined : store.satoshiSendAmount
                }
                placeholder="value"
              ></input>
              <div
                class={
                  ""
                  //TODO
                  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
                  // store.satoshiSendAmount == 0
                  //   ? badgeState.empty
                  //   : !store.amountValid
                  //   ? badgeState.invalid
                  //     ? !store.buildIsOk
                  //     : badgeState.invalid
                  //   : badgeState.valid
                }
              ></div>

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
                      store.satoshiSendAmount =
                        store.availableTokenSatoshiAmount;
                      store.tokenSendAmount = store.availableTokenAmount.toString()

                      build();
                    }}
                  >
                    Max Amount
                  </button>
                </li>
              </ul>
            </div>
          </div>

          <div class="modal-action">
            <form method="dialog">
              {/*   <!-- if there is a button in form, it will close the modal --> */}
              <button
                onClick$={() => {
                  store.tokenSendAmount = "";
                  store.satoshiSendAmount = 0;
                  store.availableTokenAmount = props.token_data!.amount;
                  store.category = props.token_data!.category;
                  // isOpen.value = false;
                }}
                class="btn  btn-outline btn-error m-2"
              >
                Close
              </button>

              <div
                onClick$={() => {
                  build();
                  if (store.buildIsOk) {
                    const transaction = store.rawTx;
                    const networkUrl = walletData.networkUrl!.concat(":50001");

                    // invoke("broadcast_transaction", { transaction, networkUrl })
                    broadcast_transaction(transaction, networkUrl)
                      .then((res: unknown) => {
                        store.broadcastRes = res as string;
                        console.log("broadcastRes", store.broadcastRes);
                      })
                      .catch((e) => {
                        store.broadcastRes = e as string;
                        store.broadcastErr = true;
                        console.log(e);
                      });
                  }
                }}
                class="btn btn-primary m-2"
              >
                Send Token
              </div>
            </form>
          </div>
        </div>
        <div class="overflow-auto">
          {!createTokenDetails.value ? <></> : <TxDetails {...txDetails} />}
        </div>
      </dialog>
    </div>
  );
});

export const TxDetails = component$((tx: Transaction) => {
  const transaction = tx; //useContext(TransactionDetails);
  // const transaction = useContext(TransactionDetails);
  return (
    <>
      <div class="overflow-auto bg-slate-900">
        <div class="">
          <div class="">
            {/* <figure></figure> */}
            <div class="">
              <h1 class="bg-auto text-xl text-accent">Transaction Details</h1>
              <label class="text-xs text-accent">Transaction ID: </label>
              <span class="text-xs">{transaction.txid}</span>
              <p>
                <h2>Coin Selection</h2>
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
              </p>
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

                            <div class="truncate">
                              <p>
                                {output.token.nft?.commitment !== undefined
                                  ? output.token.nft!.commitment
                                  : ""}
                              </p>
                            </div>
                          </div>
                        ) : (
                          <></>
                        )}
                      </div>
                    )}
                  </div>
                ))}
              </div>

              {/* <div class=" justify-center p-5">
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
              </div> */}
            </div>
          </div>
        </div>
      </div>
    </>
  );
});
