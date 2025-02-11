import { Fragment, Slot, component$, useId, useSignal, useStyles$, useStylesScoped$, useVisibleTask$ } from "@builder.io/qwik";
import { useNavigate } from "@builder.io/qwik-city";
import { useLocalStorage } from "@ditadi/qwik-hooks";
import { client_state, client_state_schema } from "~/utils/client3";
import * as v from "valibot";
import { use_client_state } from "~/utils/client2";

export default component$(() => {
    let ready = useSignal(false);
    let nav = useNavigate();

    const client_state = use_client_state(); 
    useVisibleTask$(({track}) => {
        track(client_state);
        let state = client_state.value.state;
        if (state === "auth") {
            nav("/panel");
        }
        else if (state === "need_set_up" || state === "need_auth") {
            ready.value = true;
        } else {
        }
    }, { strategy: "document-ready" });

    return (
        <Fragment>
            {ready.value ?
                <div class="page-wrapper center center-content" >
                    <main class="page-content">
                        <div class="wrapper wrapper-sm m-b-xl panel-wrapper">
                            <Slot />
                        </div>
                    </main>
                    <footer class="page-footer">
                        <Slot name="footer" />
                    </footer>
                </div> : <div class="loader" />
            }
        </Fragment>
    );
}
);

const EagerExecution = component$(() => {
    let id = useId();
    return (
        <Fragment>
            <div class={[id, "not_ready"]} >
                <style hidden dangerouslySetInnerHTML={`
.${id}:not(.ready) .${id}-content { display: none; }
.${id}.ready .${id}-loader { display: none; }
                `} />
                <script dangerouslySetInnerHTML={`
let token = localStorage.getItem("token");
let base_url = localStorage.getItem("base_url");
if (token && base_url) {
    document.querySelector(".${id}").classList.add("ready");
}
                `} />
                <div class={id + '-content'}><Slot /></div>

                <div class={[id + '-loader', "qwik-loading block txt-center"]}>
                    <div class="loader" />
                </div>
            </div>
        </Fragment>
    )
});
