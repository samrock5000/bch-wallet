import { $, component$, /* useTask$, */ useStore } from "@builder.io/qwik";
import { invoke } from "@tauri-apps/api/tauri";

export default component$(() => {
  const store = useStore({
    addressFromPath: "",
    derivation: "",
    seedExist: "",
    HdkeyNetwork: "test",
    xprivBase58: "",
  });

  const loadSeed = $(async (password: string | undefined) => {
    store.seedExist = await invoke("load_seed", { password });
  });

  const generateHdKey = $(async (network: string, seed: []) => {
    await invoke("create_hd_node", { network, seed })
      .then((key) => {
        store.xprivBase58 = key as string;
        console.log("generate_mnemonic sucess: ", key);
      })
      .catch((e) => console.log(e));
  });

  const addressFromHdKey = $(async () => {
    store.addressFromPath = await invoke("address_from_hdkey");
  });

  return (
    <div>
      <button
        preventdefault:click
        type="submit"
        class="btn btn-primary btn-accent btn-xs"
        onClick$={() => {
          loadSeed(undefined).then(() => {
            console.log("seed exist", store.seedExist);
          });
        }}
      >
        Load Seed
      </button>

      {/* <input
        placeholder="derivarion path"
        value={store.addressFromPath}
        onKeyDown$={(ev) => {
          store.derivation = (ev.target as HTMLInputElement).value;
          const validDerivationPath = /^[m](?:\/[0-9]+'?)*$/u;
          if (validDerivationPath.test(store.derivation)) {
            addressFromHdPath(store.derivation);
          } else {
            <p>invalid path :{store.addressFromPath}</p>;
          }
        }}
      ></input> */}
      <button
        onClick$={() =>
          addressFromHdKey()
            .then((val) => {
              // store.addressFromPath = val as string;
              console.log(store.addressFromPath);
            })
            .catch((e) => {
              console.log(e);
            })
        }
      >
        Get addr
      </button>
      <p>{store.addressFromPath}</p>
      <select
        onInput$={(ev) => {
          store.HdkeyNetwork = (ev.target as HTMLInputElement).value;
          console.log(store.HdkeyNetwork);
        }}
        class="select select-bordered select-xs w-full max-w-xs opacity-60"
      >
        <option disabled selected>
          Choose Network (default testnet)
        </option>
        <option value="main">Main</option>
        <option value="test">Test</option>
      </select>
    </div>
  );
});
