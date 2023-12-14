import {
  component$,
  useContext,
  useContextProvider,
  $,
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
    url: "chipnet.imaginary.cash", /* undefined as string | null | undefined */
    //Currenly useless
    urls: ["localhost"],
    isValidUrl: false,
    networkErr:"",
    chain: "",
    connection: false,
  });

  useVisibleTask$(({ track }) => {
    // track(() => store.urls);
    // store.urls = networkCtxSet.urls;
  });

  return (
    <>
      <div class="mx-auto flex max-w-sm items-center space-x-4 rounded-xl  p-6 shadow-lg">
        <button
          class={
            store.isValidUrl ? "btn btn-primary btn-xs shadow-lg" : "hidden"
          }
          preventdefault:click
          onClick$={() => {

            // store.urls.push(store.url!);
            window.localStorage.setItem("networkUrl",store.url);
            // window.localStorage.setItem(
            //   "networkUrls",
            //   JSON.stringify(store.urls),
            // );
            emit("networkUrlupdate", { url: store.url, urls: store.urls });
          }}
        >
          Connect
        </button>
      
        <input
          autoCorrect="off" 
          placeholder="my.electrum.server"
          required={true}
          onInput$={(ev) => {
            store.url = (ev.target as HTMLInputElement).value.trim();
            const url = "tcp://".concat(store.url);
            invoke("check_url", { url })
              .then(() => {
                networkPing(url.concat(":50001"))
                  .then(() => {
                    store.networkErr = "";
                    store.connection = true;
                    store.isValidUrl = true;
                    console.log("CONNECTION VALID");
                    console.log("VALID URL");
                  })
                  .catch((e) => {
                    console.error(e);
                    store.networkErr = e;
                    store.isValidUrl = false;
                    console.log("No connection with URL");
                    store.url == "" ? store.networkErr = "" : {};
                  });
              })
              .catch((e) => {
                store.networkErr = e;
                store.isValidUrl = false;
                console.error(e);
                // store.url == "" ? store.networkErr = "" : {};
              });
          }}
          type="text"
        />

        <div class="text-xs text-stone-50">{store.networkErr}</div>
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
