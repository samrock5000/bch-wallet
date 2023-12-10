import {
  component$,
  useContext,
  useContextProvider,
  useStore,
  useVisibleTask$,
} from "@builder.io/qwik";
import { invoke } from "@tauri-apps/api";
import { ContextSuccess, UrlContext, WalletContext } from "~/routes/layout";
import { type NetworkUrlUpdate, networkPing } from "../utils/utils";
import { emit } from "@tauri-apps/api/event";
//TODO create network chain select

export default component$(() => {
  // const walletData = useContext(ContextSuccess);
  const contextSet = useContext(ContextSuccess);
  const networkCtxSet = useContext(UrlContext);

  const store = useStore({
    url: "" /* undefined as string | null | undefined */,
    urls: ["localhost"],
    isValidUrl: false,
    chain: "",
    connection: false,
  });

  useVisibleTask$(({ track }) => {
    // const storeUpdated = track(() => contextSet.rdy);
    /*   const urlsUpdated =  */ track(() => store.urls);
    // store.urls = JSON.parse(window.localStorage.getItem("networkUrls")!);
    store.urls = networkCtxSet.urls;
    // if (storeUpdated) {
    //   // store.urls = props.urls;
    //   // console.log("VISIBLE STORE URLS ", store.urls);
    //   // console.log("VISIBLE URLS UPDATED", urlsUpdated);
    // }
  });
  // console.log("STORE URLS ", store.urls);

  return (
    <>
      <div class="mx-auto flex max-w-sm items-center space-x-4 rounded-xl  p-6 shadow-lg">
        <button
          class={
            store.isValidUrl ? "btn btn-primary btn-xs shadow-lg" : "hidden"
          }
          preventdefault:click
          onClick$={() => {
            console.log("CMON NOw", store.url);

            store.urls.push(store.url!);
            window.localStorage.setItem("networkUrl",store.url);
            window.localStorage.setItem(
              "networkUrls",
              JSON.stringify(store.urls),
            );
            // const urls = window.localStorage.getItem("networkUrls");
            // store.urls = JSON.parse(urls!);
            emit("networkUrlupdate", { url: store.url, urls: store.urls });
          }}
        >
          Connect
        </button>
        <input
          placeholder="my.electrum.server"
          required={true}
          onInput$={(ev) => {
            store.url = (ev.target as HTMLInputElement).value.trim();
            const url = "tcp://".concat(store.url);
            invoke("check_url", { url })
              .then(() => {
                networkPing(url.concat(":50001"))
                  .then(() => {
                    store.connection = true;
                    store.isValidUrl = true;
                    console.log("CONNECTION VALID");
                    console.log("VALID URL");
                  })
                  .catch((e) => {
                    console.error(e);
                    store.isValidUrl = false;
                    console.log("No connection with URL");
                  });
              })
              .catch((e) => {
                store.isValidUrl = false;
                console.error(e);
              });
          }}
          type="text"
        />

        <select
          class="select select-bordered select-xs w-full max-w-xs"
          onInput$={(ev) => {
            store.chain = (ev.target as HTMLInputElement).value.trim();
            window.localStorage.setItem("chain", store.chain);
          }}
        >
          <option value="test">test (chipnet)</option>
          <option disabled selected>
            CHAIN
          </option>
          {/* <option disabled selected value="main">
            main
          </option> */}
        </select>
      </div>
    </>
  );
});
