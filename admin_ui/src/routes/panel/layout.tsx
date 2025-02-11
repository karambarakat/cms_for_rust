import { Fragment, Slot, component$, useSignal, useStylesScoped$, useVisibleTask$ } from "@builder.io/qwik";
import { useNavigate } from "@builder.io/qwik-city";
import * as v from "valibot";
import { client_state_schema, use_client_state } from "~/utils/client3";

export default component$(() => {
    let ready = useSignal(false);
    let nav = useNavigate();
    let state = use_client_state();

    useVisibleTask$(() => {
        let state_2 = state.value.state;

        if (state_2 === "authenticated") {
            ready.value = true;
        } else if (state_2 === "need_set_up" || state_2 === "need_auth") {
            nav("/auth");
        }
    });

    useStylesScoped$(`
        .panel_layout {
            height: 100%;
            width: 100%;
        }
    `);


    return (
        <Fragment>
            {ready.value ?
                <div class="panel_layout" >
                    <Slot />
                </div> : <div class="loader" />
            }
        </Fragment>
    );
}
);
