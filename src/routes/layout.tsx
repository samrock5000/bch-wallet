import {
  component$,
  Slot,
  useStore,
  $,
  useContext,
  createContextId,
  useVisibleTask$,
  useContextProvider,
  // useTask$,
  useSignal,
  // useComputed$,
  // useOnWindow,
  // useOnDocument,
  // useOn,
} from "@builder.io/qwik";
// import { Link } from "@builder.io/qwik-city";
// import { exit } from "@tauri-apps/api/process";
// import { WebviewWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/tauri";
// import { window as tauriWindow } from "@tauri-apps/api";
// import { exists, BaseDirectory } from "@tauri-apps/api/fs";
// import { appWindow } from "@tauri-apps/api/window";
import { listen /* TauriEvent*/ } from "@tauri-apps/api/event";
// import { confirm } from "@tauri-apps/api/dialog";
import { type RequestHandler } from "@builder.io/qwik-city";
import Header from "../components/header/header";

import WebSocket from "tauri-plugin-websocket-api";
import {
  addressFromHdPath,
  doesWalletExist,
  getUtxos,
  networkPing,
  type WalletInit,
  type Utxo,
  type WalletData,
  type NetworkUrlUpdate,
  walletCache,
} from "~/components/utils/utils";

export const onGet: RequestHandler = async ({ cacheControl }) => {
  // Control caching for this request for best performance and to reduce hosting costs:
  // https://qwik.builder.io/docs/caching/
  cacheControl({
    // Always serve a cached response by default, up to a week stale
    staleWhileRevalidate: 60 * 60 * 24 * 7,
    // Max once every 5 seconds, revalidate on the server to get a fresh version of this page
    maxAge: 5,
  });
};
export type ContextRdy = {
  rdy: boolean;
  walletExist: boolean;
};
// import { getAddrFromHd } from "~/components/utils/utils";
// export const networkUrl = "tcp://chipnet.imaginary.cash:50001";
// export const networkUrl = "tcp://localhost:50001";
export const networkChain = "test";
// export const derivationPath = "m/44'/1'/0'/0/0";
// export const derivationPath = "m/44'/0'/0'/0/0";
export const derivationPath = "m/44'/145'/0'/0/0";
export const WalletContext = createContextId<WalletData>("Wallet");
export const TokenUtxos = createContextId<Utxo[]>("tokenUtxos");
export const ContextSuccess = createContextId<ContextRdy>("contextset");
export const UrlContext = createContextId<NetworkUrlUpdate>("url");

export const utxoSum = (utxo: Utxo[]) =>
  utxo.reduce((sum, outputs) => sum + outputs.value, 0);

export default component$(() => {
  const walletExist = useSignal(false);
  const networkConnection = useSignal(false);
  const contextUpdated = useSignal(false);
  const contextSet = useStore<ContextRdy>({
    rdy: false,
    walletExist: false,
  });
  const subscriptionHasUpdated = useSignal(false);
  const wsClientInstanceCount = useSignal(0);
  const urlCtxStore = useStore<NetworkUrlUpdate>({
    url: "",
    urls: [""],
  });
  const store = useStore<WalletData>({
    masterKey: "",
    address: "",
    balance: 0,
    tokenSatoshiBalance: 0,
    mnemonic: "",
    networkUrl: "",
    network: "test",
    // networkConnection: false,
    bip44Path: derivationPath,
    utxos: [],
    tokenUtxos: [],
    // walletExist: false,
  });

  const subscription = useStore({
    type: "",
    hash: "",
    webSocketID: 0,
  });
  useVisibleTask$(async ({ track }) => {
    doesWalletExist().then((res) => {
      walletExist.value = res as boolean;
      contextSet.walletExist = res as boolean;
    });
    if (!walletExist.value) {
      console.log("walletExist.value", walletExist.value);
      await listen<WalletInit>("mnemonicLoaded", async (event) => {
        walletExist.value = true;
        contextSet.walletExist = true;
        console.log(
          `new wallet created ${event.windowLabel}, payload: ${event.payload.mnemonic}`,
        );
      });
    }
    walletCache()
      .then(async (cache) => {
        console.log("CACHE", cache);

        store.address = cache.db.address;
        store.balance = cache.db.balance;
        store.bip44Path = cache.db.bip44Path;
        store.mnemonic = cache.db.mnemonic;
        store.network = cache.db.network;
        store.networkUrl = cache.db.networkUrl;
        store.tokenSatoshiBalance = cache.db.tokenSatoshiBalance;
        store.tokenUtxos = cache.db.tokenUtxos;
        store.utxos = cache.db.utxos;
        // store.walletExist = cache.walletExist;

        contextSet.rdy = true;
        networkPing(store.networkUrl!.concat(":50001"))
          .then(() => {
            networkConnection.value = true;
          })
          .catch((e) => {
            networkConnection.value = false;
            console.error("networkPing", e);
          });
      })
      .catch((e) => console.error("walletCache", e))
      .then(() => {
        const val = "m/0";
        invoke("update_bip44_path", { val });
        // networkPing(store.networkUrl!.concat(":50001"))
        //   .then(() => {
        //     networkConnection.value = true;
        //   })
        //   .catch((e) => {
        //     networkConnection.value = false;
        //     console.error("networkPing", e);
        //   });
      });
    track(() => walletExist.value);
    track(() => contextSet.rdy);
    track(() => contextSet.walletExist);
    // console.log("STORE", store);
  });

  useContextProvider(WalletContext, store);
  useContextProvider(ContextSuccess, contextSet);
  useContextProvider(UrlContext, urlCtxStore);

  return (
    <>
      <div>
        <label class="text-xs"> Network: </label>
        <span class="text-xs text-secondary">
          {" "}
          {networkConnection.value
            ? `${store.networkUrl} ✓`
            : "no connection ✗"}
          {""}
        </span>
        {!walletExist.value ? (
          <>
            <Slot />
          </>
        ) : (
          <>
            <Header />
            <Slot />
            <ManualUtxoCheck />
          </>
        )}
      </div>
    </>
  );
});

const ManualUtxoCheck = component$(() => {
  const walletData = useContext(WalletContext);
  const store = useStore({
    utxos: [] as Utxo[],
  });
  const updateUtxoStore = $((address: string, networkUrl: string) => {
    invoke("update_utxo_store", { address, networkUrl })
      .then(() => {
        console.log("store updated");
      })
      .catch((e) => console.error(e));
  });
  const getUtxos = $((address: string, networkUrl: string) => {
    invoke("network_unspent_utxos", { address, networkUrl })
      .then((utxos) => {
        // @ts-ignore
        const x = JSON.parse(utxos);
        // @ts-ignore
        store.utxos = x.filter((e) => !e.token_data);
        console.log("utxos", store.utxos);
      })
      .catch((e) => console.error(e));
  });

  // const address = walletData.address;
  const address = "bchtest:qq68a6ucj6my5jzdzqv6zcr4cx22zlnqsy9k4ash3q";
  const networkUrl = "localhost:50001";
  return (
    <>
      <h1>CONSOLE DEBUG</h1>
      <button
        class="btn btn-outline btn-accent btn-xs  opacity-60"
        onClick$={() =>
          invoke("get_mempool_address", { address, networkUrl }).then((data) =>
            // @ts-ignore
            console.log(JSON.parse(data)),
          )
        }
      >
        MEMPOOL CHECK
      </button>
      <br></br>
      <button
        class="btn btn-outline btn-accent btn-xs  opacity-60"
        onClick$={() => updateUtxoStore(address, networkUrl)}
      >
        UPDATE LOCAL STORE
      </button>

      <br></br>
      <button
        class="btn btn-outline btn-accent btn-xs  opacity-60"
        onClick$={() => getUtxos(address, networkUrl)}
      >
        FETCH DB UTXOS
      </button>

      <br></br>
      <button
        class="btn btn-outline btn-accent btn-xs  opacity-60"
        onClick$={() => {
          invoke("utxo_cache", { address })
            .then((d) => {
              console.log("UTXOS CASHE", d);
            })
            .catch((e) => console.error(e));
          invoke("address_cache", { address })
            .then((d) => {
              console.log("ADDRESS CACHE", d);
            })
            .catch((e) => console.error(e));
        }}
      >
        CHECK COMMAND
      </button>
    </>
  );
});

/* 
old init client work
    doesWalletExist().then(async (res) => {
      mnemonicExist.value = res as boolean;
    });

    const walletExist = track(() => mnemonicExist.value);
    mnemonicExist.value = walletExist;

    const isNetworkSet = window.localStorage.getItem("networkUrl") != null;
    if (!isNetworkSet) {
      window.localStorage.setItem("networkUrl", store.networkUrl!);
    } else {
      store.networkUrl = window.localStorage.getItem("networkUrl")!;
    }

    const networkUrlUpdated = track(() => store.networkUrl);

    urlCtxStore.url = networkUrlUpdated!;

     await listen<NetworkUrlUpdate>(
      "networkUrlupdate",
      async (event) => {
        store.networkUrl = event.payload.url;
        urlCtxStore.urls = event.payload.urls;
        store.networkConnection = true;
        window.localStorage.setItem(
          "networkUrls",
          JSON.stringify(urlCtxStore.urls),
        );
        console.log(
          `networkUrl updates ${event.windowLabel}, payload: ${event.payload}`,
        );
        // invoke("close_splash").catch((e) => console.error(e));
      },
    );

    // const networkUrl = window.localStorage.getItem("networkUrl");
    // if (networkUrl == null) {
    //   window.localStorage.setItem("networkUrl", "localhost");
    //   store.networkUrl = "localhost"; //window.localStorage.getItem("networkUrl")!;
    // }
    // console.log("store.networkUrl.concat", store.networkUrl);
    networkPing(store.networkUrl!.concat(":50001"))
      .then(() => {
        store.networkConnection = true;
      })
      .catch((e) => {
        store.networkConnection = false;
        console.error("networkPing", e);
      });

    if (!walletExist) {
      // const unlisten =  await listen<WalletInit>(
        "mnemonicLoaded",
        async (event) => {
          mnemonicExist.value = true;
          console.log(
            `new wallet created ${event.windowLabel}, payload: ${event.payload.mnemonic}`,
          );
        },
      );
    } else {
      const setListener = async () => {
        const wsClient = await WebSocket.connect(
          `ws://${store.networkUrl!.concat(":50003")}`,
        );
        wsClientInstanceCount.value += 1;
        if (wsClientInstanceCount.value > 1) {
          wsClient.disconnect().then(() => {
            console.log("WS DISCONNECTED");
          });
        }
        // const wsClient = await WebSocket.connect(
        // "ws://chipnet.imaginary.cash:50003",
        // );
        const ctxUpdate = track(() => contextUpdated.value);
        contextUpdated.value = ctxUpdate;
        const subscribeUpdate = track(() => subscriptionHasUpdated.value);
        subscriptionHasUpdated.value = subscribeUpdate;

        const latestSubscription = window.localStorage.getItem("subscription");
        const req = JSON.stringify({
          method: "blockchain.address.subscribe",
          params: [store.activeAddr],
          id: subscription.webSocketID,
        });
        wsClientInstanceCount.value != 1
          ? {}
          : wsClient.send(req).catch((e) => console.error("wsClient.send", e));
        wsClient.addListener((res) => {
          subscription.type = res.type;
          if (subscription.type == "Text") {
            const data = JSON.parse(res.data as string);
            subscription.hash =
              data.result == undefined ? data.params[1] : data.result;
            window.localStorage.setItem("subscription", subscription.hash);
            if (subscription.hash != latestSubscription) {
              wsClient
                .disconnect()
                .then(() => {
                  console.log("WS DISCONNECTED", subscription.webSocketID);
                  wsClientInstanceCount.value -= 1;
                  store.networkConnection = false;
                })
                .then(() => {
                  updateUtxoStore(
                    store.activeAddr,
                    store.networkUrl!.concat(":50001")!,
                  );
                })
                .catch((e) => {
                  console.error("wsClient err", e);
                })
                .finally(() => {
                  setTimeout(() => {
                    subscriptionHasUpdated.value = true;
                    contextUpdated.value = true;
                  }, 1000);
                });
            }
          }
          console.log("RESPONSE", res);

          if (typeof res === "string") {
            store.networkConnection = false;
          }
        });
      };
      setListener();
      // addressFromHdPath(store.bip44Path, store.network)
      //   .then((addr: unknown) => {
      //     store.activeAddr = addr as string;
      //   })
      //   .catch((e) => console.error(e))
      // .finally(() => {

       
        updateUtxoStore(store.activeAddr, store.networkUrl!.concat(":50001")!)
        .catch((e) => console.error("updateUtxoStore err", e))
        .finally(() => {
          setListener();
        });
      

      // });
    }
    */
