import { component$, useStylesScoped$ } from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";
import {  use_client_state } from "~/utils/client_state";

export default component$(() => {
    useStylesScoped$(`
.root-index {
    width: 100%;
    height: 100%; 
}
.root-index > div {
    margin-top: 30px;
    text-align: center
}
    `);

    let state = use_client_state();


    return (
        <div class="root-index">
            <div>
                Welcome to CMS
                {
                    state.value.state === "loading" ? <div style={{ opacity: 0 }}>loading</div> :
                        state.value.state === "authenticated" ?
                            <div>go to <a href="/panel">panel</a></div>
                            : <div>you are not authinticated, go to <a href="/auth/login">login</a></div>
                }
            </div>
        </div>
    );
});

export const head: DocumentHead = {
    title: "Welcome to CMS",
    meta: [],
};
