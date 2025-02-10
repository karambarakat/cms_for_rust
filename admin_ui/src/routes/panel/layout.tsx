import { Fragment, Slot, component$, useSignal, useStylesScoped$, useVisibleTask$ } from "@builder.io/qwik";
import { useNavigate } from "@builder.io/qwik-city";

export default component$(() => {
    let ready = useSignal(false);
    let nav = useNavigate();
    useVisibleTask$(() => {
        let base_url = window.localStorage.getItem("backend_url");
        let token = window.localStorage.getItem("token");
        if (!base_url || !token) {
            nav("/auth");
        } else {
            ready.value = true;
        }
    }, { strategy: "document-ready" });

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
