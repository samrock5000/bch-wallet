import {
  $,
  Resource,
  component$,
  useContext,
  useResource$,
  useStore,
  useVisibleTask$,
} from "@builder.io/qwik";
import { invoke } from "@tauri-apps/api";
import { WalletContext } from "~/routes/layout";
import { History } from "../utils/utils";

export default component$(() => {
  const store = useStore({
    address: "",
    history: [] as History[],
    historyReady: false,
  });

  const walletData = useContext(WalletContext);

  useVisibleTask$(() => {
    store.address = walletData.activeAddr;
    // setTimeout(() => (store.address = walletData.activeAddr), 1000);

    // const address = walletData.activeAddr;

    setTimeout(() => {
      const address = walletData.activeAddr;
      const networkUrl = walletData.networkUrl.concat(":50001");
      invoke("address_history", { address, networkUrl })
        .then((data) => {
          store.history = JSON.parse(data as string);
          console.log(store.history);
          store.historyReady = true;
        })
        .catch((e) => console.log(e));
    }, 50);
  });

  return (
    <>
      <div class=" grid justify-items-center">
        <h1>History</h1>
        <p>{walletData.activeAddr}</p>
        <p>{store.historyReady ? store.history[1].height : "loading"}</p>
      </div>
    </>
  );
});
