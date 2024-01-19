import {component$, useContext, useVisibleTask$} from "@builder.io/qwik";
import {hexToBin} from "@bitauth/libauth"
import {type Output} from "@bitauth/libauth"
import { ContextSuccess, WalletContext } from "~/routes/layout";
import { Utxo } from "../utils/utils";

const utxoToOut = (output:Utxo[]) => {
}



export default component$(()=> {
 const walletData =  useContext(WalletContext)
 const ctxRdy =  useContext(ContextSuccess)
  useVisibleTask$(({track})=>{

    const ctxLoaded = track(() => ctxRdy.rdy);
    if (ctxLoaded) {

  console.log("CASH ASSEMBLY",walletData.utxos)
    }
  })
  return <></>
})
