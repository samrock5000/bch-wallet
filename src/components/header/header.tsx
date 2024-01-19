import {
  component$,
  useStore,
  useVisibleTask$,
  useContext,
  useTask$,
} from "@builder.io/qwik";
import { WalletContext, ContextSuccess } from "../../routes/layout";
import { BchLogo } from "../svg/bch-logo";
import { invoke } from "@tauri-apps/api";

export default component$(() => {
  const storeBalance = useStore({
    confirmed: 0,
    unconfirmed: 0,
  });

  const walletData = useContext(WalletContext);
  const contextSet = useContext(ContextSuccess);

  useVisibleTask$(({ track }) => {
    const ctxUpdated = track(() => contextSet.rdy);
    if (ctxUpdated) {
      const address = walletData.address;
      const networkUrl = walletData.networkUrl?.concat(":50001");
      invoke("network_unspent_balance_include_tokens", {
        address,
        networkUrl,
      }).then((r) => {
        //@ts-ignore
        const res = JSON.parse(r);
        storeBalance.confirmed = res.confirmed;
        storeBalance.unconfirmed = res.unconfirmed;
        // const x = res.confirmed;
        // console.log(" NETWORK RES", r);
        // console.log(" NETWORK RES X", x);
      });
      // storeBalance.confirmed = walletData.balance;
    }
  });

  return (
    <div class="navbar bg-base-100">
      <div class="navbar-start">
        {/* <h1 class=" text-xl normal-case">BCH-Wallet</h1> */}
        <BchLogo width={130} height={44} />
      </div>
      <div class="navbar-end">
        <div>
          {/* <div class="badge badge-primary badge-outline badge-sm"></div> */}
        </div>
        <div class="stats stats-vertical shadow">
          <div class="stat ">
            <div class="stat-title">Bitcoin Cash</div>
            <div class="stat-value text-secondary">
              {" "}
              {
                //@ts-ignore
                storeBalance.confirmed / 100_000_000
              }
            </div>
            <div class="stat-desc">
              unconfirmed :{" "}
              {
                //@ts-ignore
                storeBalance.unconfirmed / 100_000_000
              }
            </div>
          </div>
        </div>
      </div>
    </div>
  );
});
