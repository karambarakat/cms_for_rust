import { Fragment, Slot, component$, useStylesScoped$ } from "@builder.io/qwik";
import { use_gaurd_or_redirect } from "~/utils/client_state";
import { use_schema_provider } from "~/utils/schema";

export default component$(() => {
    let gaurd = use_gaurd_or_redirect("authenticated");

    useStylesScoped$(`
        .panel_layout {
            height: 100%;
            width: 100%;
        }
    `);

    const pass = use_schema_provider();

    return (
        <Fragment>
            {
                (gaurd.value &&
                    pass.value.ty === "pass") ? <Slot /> :
                    pass.value.ty === "error" ? <div>fail to fetch schema</div> :
                        /* pass.value.ty === "loading" || !gaurd.value */
                        <div class="loader" />
            }
        </Fragment>
    );
}
);
