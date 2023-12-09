import {
  $,
  component$,
  useTask$,
  useStore,
  useSignal,
  useVisibleTask$,
  useContextProvider,
  createContextId,
  useContext,
} from "@builder.io/qwik";
import { useNavigate } from "@builder.io/qwik-city";

import { invoke } from "@tauri-apps/api/tauri";

import { emit } from "@tauri-apps/api/event";
// import { WalletInit } from "../utils/utils";

import { type KeySetUp } from "../utils/utils";
import { WebviewWindow } from "@tauri-apps/api/window";
export const keyIsSet = createContextId<KeySetUp>("keyIsSet");

export default component$(() => {
  const store = useStore({
    xprivBase58: "",
    password: "", // optional salt for bip39 seed generation
    masterKeyExist: false,
    importSuccess: false,
    createSuccess: false,
    validKey: false,
    base58err: "",
    mnemonic: "",
    dbWords: "",
    HdkeyNetwork: "test",
    salt: "",
    seed: [],
    key: { isSet: false } as KeySetUp,
  });
  // const createKeys = useSignal()

  useTask$(({ track }) => {
    const data = track(() => store);
    const keySet = track(() => store.key.isSet);
    store.key.isSet = keySet;
    store.xprivBase58 = data.xprivBase58;
    store.password = data.password;
    store.masterKeyExist = data.masterKeyExist;
    store.base58err = data.base58err;
    store.HdkeyNetwork = data.HdkeyNetwork;
    store.seed = data.seed;
  });
  useContextProvider(keyIsSet, store.key);
  return (
    <>
      <div>
        <div class="mx-auto flex max-w-sm items-center  space-x-4 rounded-xl p-6 shadow-lg">
          <div class="">
            <div
              hidden={store.mnemonic == "" ? true : false}
              class="my-1 rounded-xl bg-neutral-900 p-2"
            >
              <p class="font-medium text-slate-500">{store.mnemonic}</p>
            </div>

            <ImportMnemonic />
          </div>
        </div>

        <div class="mx-auto flex max-w-sm items-center space-x-4 rounded-xl  p-6 shadow-lg">
          <div class="">
            <GenerateMnemonic />
          </div>
        </div>

        <div class="mx-auto flex max-w-sm items-center space-x-4 rounded-xl p-6 shadow-lg">
          <div class="">
            <LoadMnemonic />
          </div>
        </div>
      </div>
    </>
  );
});

export const GenerateMnemonic = component$(() => {
  const keyIsCreated = useContext(keyIsSet);
  const store = useStore({
    words: "",
    seed: [],
  });

  //TODO use modal for extra security
  // or create a separate file/account
  const generateMnemonic = $(() => {
    invoke("generate_mnemonic")
      .then((words) => {
        store.words = words as string;
        console.log("generate_mnemonic sucess: ", words);
        //TODO add password option
        const password = undefined;
        invoke("save_mnemonic", { words, password })
          .then(() => {
            console.log("mnemonic saved", store.words);
          })
          .catch((e) => console.log(e));
        //TODO add salt option
        const salt = undefined;
        invoke("generate_seed", { words, salt })
          .then((s) => {
            const seed = s as [];
            console.log("generate_seed_from_mnemonic sucess: ", s);
            invoke("save_seed", { seed })
              .then(() => {
                console.log("seed saved");
                keyIsCreated.isSet = true;
              })
              .catch((e) => console.error(e));
          })
          .catch((e) => console.error(e));
      })
      .catch((e) => console.error(e));
  });
  useContextProvider(keyIsSet, keyIsCreated);
  return (
    <>
      {" "}
      <h1>WARNING</h1>
      <button
        preventdefault:click
        class="btn btn-primary btn-xs shadow-lg"
        onClick$={() => generateMnemonic()}
      >
        Generate New Wallet
      </button>
      <p>WILL DESTROY CURRENT WALLET</p>
      <div>{store.words}</div>
    </>
  );
});

export const ImportMnemonic = component$(() => {
  // const nav = useNavigate();
  const keyIsCreated = useContext(keyIsSet);
  const store = useStore({
    words: "",
    validWords: false,
    isErr: false,
    saveErr: false,
    validationErr: "",
    inputCleared: false,
  });

  const validMnemonic = $((words: string) => {
    invoke("valid_mnemonic", { words })
      .then(() => {
        store.validWords = true;
        store.validationErr = "";
        store.words = words;
      })
      .catch((e) => {
        store.validationErr = e;
        store.isErr = true;
        store.validWords = false;
        console.error(e);
      });
  });
  const saveMnemonic = $((words: string, password: string | undefined) => {
    invoke("save_mnemonic", { words, password })
      .then(() => {
        console.log("mnemonic saved", store.words);
        //TODO add salt option for extra security
        const salt = password;
        const words = store.words;
        invoke("generate_seed", { words, salt }).then((seed) => {
          console.log("generate_seed_from_mnemonic sucess: ", seed);
          invoke("save_seed", { seed, password })
            .then(() => {
              console.log("seed saved");
              keyIsCreated.isSet = true;
            })
            .catch((e) => {
              store.isErr = true;
              console.error(e);
            });
        });
      })
      .catch((e) => {
        store.saveErr = true;
        store.validationErr = e;
        console.log("store.words", store.words);
        console.log(e);
      });
  });
  useContextProvider(keyIsSet, keyIsCreated);
  return (
    <>
      <div>
        <p>
          {" "}
          {store.inputCleared ? "" : store.isErr ? store.validationErr : ""}
        </p>
        <p class="py-2"> Import Wallet from Secret Phrase</p>
        <textarea
          placeholder="12 word mnemonic"
          onInput$={(e) => {
            validMnemonic((e.target as HTMLInputElement).value.trim());
            store.inputCleared =
              (e.target as HTMLInputElement).value == "" ? true : false;
          }}
          onPaste$={(e) => {
            validMnemonic((e.target as HTMLInputElement).value.trim());
            store.inputCleared =
              (e.target as HTMLInputElement).value == "" ? true : false;
          }}
        ></textarea>

        <button
          disabled={store.validWords ? false : true}
          preventdefault:click
          type="submit"
          class="btn btn-primary btn-xs"
          onClick$={() => {
            saveMnemonic(store.words, undefined)
              .catch((e) => console.error(e))
              .then(async () => {
                emit("mnemonicLoaded", { mnemonic: store.words });
                // invoke("close_wallet_create").then(() => {
                //   console.log("CLOSE WALLET CREEATE");
                // });
              })
              .finally(async () => {
                const walletCreateWindow =
                  WebviewWindow.getByLabel("create-wallet");
                // nav("/");
                // create-wallet
                await walletCreateWindow?.close();
              });
          }}
        >
          Import Wallet from Mnemonic
        </button>
      </div>
    </>
  );
});

export const LoadMnemonic = component$(() => {
  const showPhrase = useSignal(false);
  const store = useStore({
    words: "",
  });
  const loadWords = $((password: string) => invoke("load_mnemonic"));
  return (
    <>
      <button
        class="btn btn-primary btn-xs"
        onClick$={() => {
          loadWords("")
            .then((words) => {
              store.words = words as string;
              showPhrase.value = showPhrase.value == false ? true : false;
              console.log("MEMONIC LOADED", words);
            })
            .catch((e) => console.error(e));
        }}
      >
        Show seed phrase
      </button>
      <p>{showPhrase.value ? store.words : ""}</p>
    </>
  );
});
