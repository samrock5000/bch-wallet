import {
  $,
  component$,
  useTask$,
  useStore,
  useVisibleTask$,
  useSignal,
  createContextId,
  useContextProvider,
} from "@builder.io/qwik";
import { Link } from "@builder.io/qwik-city";
import { invoke } from "@tauri-apps/api/tauri";
import Login from "../login/login";
import Mnemonic from "../mnemonic/mnemonic";
// export const network = "tcp://electrum.imaginary.cash:50001";
// export const network = "tcp://chipnet.imaginary.cash:50001";

export const network = "tcp://localhost:50001";

interface WalletData {
  masterKey: string;
  balance: number;
  activeAddr: string;
  // purpose: string[];
}

export const WalletContext = createContextId<WalletData>("Wallet");

export default component$(() => {
  const balance = useSignal(0);
  const dbAddress = useSignal("");

  /*   const setContext = $(async () => {
    await invoke("address_from_hdkey")
      .then((addr) => {
        //@ts-ignore
        dbAddress.value = addr;
      })
      .catch((e) => console.log("failed to get address from db:", e))
      .then(() => {
        getAdressBalance(dbAddress.value)
          .then((b) => {
            balance.value = b;
          })
          .catch((e) => console.log("get balance fail: ", e));
      });
  });
 */

  /*   const getAdressBalance = $(async (address: string) => {
    await invoke("non_token_sum_from_db", {
      address,
    });
  }); */

  // getAdressBalance("bchtest:qptnz3u8atavszhaqk037v0fjrtahxmsl5mm45u3pf");

  useContextProvider(
    WalletContext,
    useStore<WalletData>({
      masterKey: "secretkey",
      balance: balance.value,
      activeAddr: dbAddress.value,
    }),
  );

  const store = useStore({
    account: "",
    coinType: 0,
    accountIndex: 0,
    balance: 0,
    // address: "bchtest:qptnz3u8atavszhaqk037v0fjrtahxmsl5mm45u3pf",
    address: "",
    network: network,
    network_utxos: 0,
    masterKeyExist: false,
    seedExist: false,
    history: "",
  });
  // const addressFromHdKey = $(async () => {
  //   store.address = await invoke("address_from_hdkey");
  // });

  // const keyExist = useSignal(false);
  // useTask$(({ track }) => {
  //   const masterKeyExist = track(() => store.masterKeyExist);
  //   store.masterKeyExist = masterKeyExist;
  // });
  // const updateStoreForAddress = $(async (address: string, network: string) => {
  //   await invoke("update_utxo_store", {
  //     address,
  //     network,
  //   });
  //   // .then((v) => console.log(v))
  //   // .catch((e) => console.log("ERR", e));
  // });

  // const getAddressHistory = $(
  //   async (address: string, network: string /*  fromHeight: number */) => {
  //     store.history = await invoke("address_history", {
  //       address,
  //       network,
  //       // fromHeight,
  //     })
  //       .then((v) => console.log(v))
  //       .catch((e) => console.log("ERR", e));
  //   }
  // );

  // const updateStore = $(() =>
  //   updateStoreForAddress(store.address, store.network)
  // );

  // const subcribe = $(async (address: string, network: string) => {
  //   await invoke("subscribe_to_address", {
  //     address,
  //     network,
  //   });
  // });
  // const masterKeyExist = $(async () => {
  //   await invoke("does_master_key_exist")
  //     .then((v) => {
  //       console.log("masterKeyExist", v);
  //       // keyExist.value = v;
  //       store.masterKeyExist = v as boolean;
  //     })
  //     .catch((e) => {
  //       console.log("Master key exist ERROR", e);
  //     });
  // });

  // //does_seed_exist
  // const seedExist = $(async () => {
  //   await invoke("does_seed_exist")
  //     .then((v) => {
  //       console.log("seed exist", v);
  //       // keyExist.value = v;
  //       store.seedExist = v as boolean;
  //     })
  //     .catch((e) => {
  //       console.log("Master key exist ERROR", e);
  //     });
  // });

  return (
    <>
      <div
      // window:onLoad$={() => {
      //   addressFromHdKey()
      //     .then(() =>
      //       updateStore()
      //         .then(() => {
      //           console.log("UTXO STORE UPDATED address", store.address);
      //         })
      //         .catch((e) => {
      //           console.log(e);
      //         })
      //     )
      //     .catch(() => console.log("will crash if fail"));
      // seedExist();
      // masterKeyExist();
      // getAddressHistory(store.address, store.network);
      // console.log("store.history", store.history);
      // }}
      ></div>
      <Mnemonic />
      {/* {store.masterKeyExist ? "KEY exist component?" : <KeySave />} */}
      {/* {store.masterKeyExist ? "KEY exist component?" : } */}
    </>
  );
});
