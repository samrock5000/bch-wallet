import { $, component$, useSignal, useStore, useTask$ } from "@builder.io/qwik";
import { invoke } from "@tauri-apps/api/tauri";

// export const network = "tcp://localhost:50001";
export default component$(() => {
  const store = useStore({
    xpriv: "",
    account: "",
    password: "",
    wrong_password_error: false,
    io_error: false,
    other_error: false,
    passwordValid: false,
    error: "",
  });
  const error = useSignal("");

  const masterKeyExist = $(async () => {
    await invoke("does_master_key_exist");
  });

  useTask$(({ track, cleanup }) => {
    const pass = track(() => store.password);
    // const err = track(() => store.error);
    store.password = pass;
    // store.error = err;
    // cleanup(() => (store.error = ""));
  });

  const keyExist = $(() => {
    masterKeyExist()
      .then(() => {
        console.log("key good");
      })
      .catch((err: boolean) => {
        console.log(err);
      });
  });

  const CreateChangeKeys = $(async () => {
    await invoke("create_change_pubkeyhash_store")
      .then(() => {
        console.log("create_change_pubkeyhash_store OK");
      })
      .catch((e) => console.log("error: ", e));
  });

  return (
    <div>
      <h1 class="text-2xl font-bold"></h1>
      <div class="form-control py-6">
        <input
          // minLength="0"
          // maxlength="16"
          onKeyDown$={(ev) => {
            if (ev.key == "Enter") {
              // keyExist();
              // CreateChangeKeys();
              // validate();
              console.log(ev.key);
            }
          }}
          type="password"
          placeholder="wallet password"
          class="input input-bordered"
          value={store.password}
          autoFocus
          onInput$={(ev) => {
            store.password = (ev.target as HTMLInputElement).value;
            error.value = "";
          }}
        />

        {store.passwordValid === false ? (
          <> </>
        ) : (
          <div class="pt-4">
            <p class="text-slate-500">
              {store.other_error ? (
                ""
              ) : (
                <div class="alert" color="red">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                    class="stroke-info shrink-0 w-6 h-6"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
                    ></path>
                  </svg>
                  <span>{error.value}</span>
                </div>
              )}
            </p>
            <button
              disabled={store.password === ""}
              class="btn btn-primary"
              //@ts-ignore
              onClick$={() => validate()}
            >
              Continue
            </button>
          </div>
        )}
      </div>
    </div>
  );
});
