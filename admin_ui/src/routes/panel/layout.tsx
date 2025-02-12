import { Fragment, Slot, component$, useSignal, useStylesScoped$, useVisibleTask$ } from "@builder.io/qwik";
import { useNavigate } from "@builder.io/qwik-city";
import { use_client_state, use_gaurd_or_redirect } from "~/utils/client_state";

export default component$(() => {
    let gaurd = use_gaurd_or_redirect("authenticated");

    useStylesScoped$(`
        .panel_layout {
            height: 100%;
            width: 100%;
        }
    `);


    return (
        <Fragment>
            {gaurd.value ?
                <div class="panel_layout" >
                    <Slot />
                </div> : <div class="loader" />
            }
        </Fragment>
    );
}
);
