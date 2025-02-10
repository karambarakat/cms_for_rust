import { Fragment, Slot, component$, useSignal, useVisibleTask$ } from "@builder.io/qwik";
import { useNavigate } from "@builder.io/qwik-city";

export default component$(() => {

    let ready = useSignal(false);
    let nav = useNavigate();
    useVisibleTask$(() => {
        let base_url = window.localStorage.getItem("backend_url");
        let token = window.localStorage.getItem("token");
        if (base_url && token) {
            nav("/panel");
        } else {
            ready.value = true;
        }
    }, { strategy: "document-ready" });

    return (

        <Fragment>
            {ready.value ?
                <div class={["page-wrapper full-page center center-content"]} >
                    <main class="page-content">
                        <div class="wrapper wrapper-sm m-b-xl panel-wrapper">
                            <Slot />
                        </div>
                    </main>
                    <footer class="page-footer">
                        <Slot name="footer" />
                    </footer>
                </div> : <div class="block txt-center">
                    <div class="loader" />
                </div>}
        </Fragment>
    );
}
);
