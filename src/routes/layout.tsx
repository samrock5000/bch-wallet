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
};
// import { getAddrFromHd } from "~/components/utils/utils";
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
  const mnemonicExist = useSignal(false as undefined | boolean);
  const contextUpdated = useSignal(false);
  const contextSet = useStore<ContextRdy>({
    rdy: false,
  });
  const subscriptionHasUpdated = useSignal(false);
  const urlCtxStore = useStore<NetworkUrlUpdate>({
    url: "",
    urls: [""],
  });
  const store = useStore<WalletData>({
    masterKey: "",
    activeAddr: "",
    balance: 0,
    tokenUtxoBalance: 0,
    mnemonic: "",
    networkUrl: "localhost",
    network: "test",
    networkConnection: false,
    bip44Path: derivationPath,
    utxos: [],
    tokenUtxos: [],
  });

  const updateUtxoStore = $((address: string, networkUrl: string) => {
    invoke("update_utxo_store", { address, networkUrl })
      .then(() => {
        contextUpdated.value = false;
        contextSet.rdy = false;
        console.log("store updated");
      })
      .catch((e) => console.error("update utxo store", e))
      .finally(() => {
        getUtxos(store.activeAddr)
          .then((utxos) => {
            // @ts-ignore
            store.utxos = utxos.filter((e) => !e.token_data);
            // @ts-ignore
            store.tokenUtxos = utxos.filter((e) => e.token_data);
            store.balance = utxoSum(store.utxos);
            console.log("tokenUtxos", store.tokenUtxos);
            contextSet.rdy = true;
            //TODO rm contextmenu
            // tauriWindow.getCurrent().listen(TauriEvent.WINDOW_CLOSE_REQUESTED, async () => {
            //   // await confirm('Are you sure?', 'Tauri');
            //    confirm('This action cannot be reverted. Are you sure?', { title: 'Tauri', type: 'warning' })
            //     .then( async(res)=> res == true ? await exit(1) : console.log("STAY OPEN") )
          })
          .catch((e) => console.error(e));
      });
  });
  const subscription = useStore({
    type: "",
    hash: "",
    webSocketID: 0,
  });
  useVisibleTask$(async ({ track }) => {
    const networkUrlUpdated = track(() => store.networkUrl);
    const walletExist = track(() => mnemonicExist.value);
    track(() => store.networkConnection);
    mnemonicExist.value = walletExist;
    // const urls = window.localStorage.getItem("networkUrls");
    urlCtxStore.url = networkUrlUpdated;

    /* const unlistenNetworkUrlUpdate = */ await listen<NetworkUrlUpdate>(
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
      },
    );

    // const networkUrl = window.localStorage.getItem("networkUrl");
    // if (networkUrl == null) {
    //   window.localStorage.setItem("networkUrl", "localhost");
    //   store.networkUrl = "localhost"; //window.localStorage.getItem("networkUrl")!;
    // }
    console.log("store.networkUrl.concat", store.networkUrl);
    networkPing(store.networkUrl.concat(":50001"))
      .then(() => {
        store.networkConnection = true;
      })
      .catch((e) => {
        store.networkConnection = false;
        console.error("networkPing", e);
      });

    doesWalletExist().then(async (res) => {
      mnemonicExist.value = res as boolean;
    });

    if (!walletExist) {
      /* const unlisten =  */ await listen<WalletInit>(
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
          `ws://${store.networkUrl.concat(":50003")}`,
        );
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
        wsClient.send(req).catch((e) => console.error("wsClient.send", e));
        wsClient.addListener((res) => {
          // console.log("Lisetener ready", r);
          subscription.type = res.type;
          if (subscription.type == "Text") {
            const data = JSON.parse(res.data as string);
            subscription.hash =
              data.result == undefined ? data.params[1] : data.result;
            window.localStorage.setItem("subscription", subscription.hash);
            if (subscription.hash != latestSubscription) {
              // updateUtxoStore(store.activeAddr, store.networkUrl)
              /* .then(() => */
              wsClient
                .disconnect()
                .then(() => {
                  console.log("WS DISCONNECTED", subscription.webSocketID);
                  store.networkConnection = false;
                })
                .then(() => {
                  updateUtxoStore(
                    store.activeAddr,
                    store.networkUrl.concat(":50001")!,
                  );
                })
                .catch((e) => {
                  console.error("wsClient err", e);
                })
                .finally(() => {
                  setTimeout(() => {
                    subscriptionHasUpdated.value = true;
                    contextUpdated.value = true;
                  }, 2000);
                });
            }
          }
          console.log("RESPONSE", res);

          if (typeof res === "string") {
            store.networkConnection = false;
          }
          // console.log("subscription hash", subscription.hash);
        });
      };

      addressFromHdPath(store.bip44Path, store.network)
        .then((addr: unknown) => {
          store.activeAddr = addr as string;
        })
        .catch((e) => console.error(e))
        .finally(() => {
          updateUtxoStore(store.activeAddr, store.networkUrl.concat(":50001")!)
            // .then(() => {})
            .catch((e) => console.error("updateUtxoStore err", e))
            .finally(() => {
              setListener();
            });
        });
    }
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
          {store.networkConnection
            ? `${store.networkUrl} ✓`
            : "no connection ✗"}
          {""}
        </span>

        <Header />
        <Slot />
        {/* <ManualUtxoCheck /> */}
        {/* <Footer/> */}
      </div>
    </>
  );
});

/* const ManualUtxoCheck = component$(() => {
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

  const address = walletData.activeAddr;
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
    </>
  );
});
 */
