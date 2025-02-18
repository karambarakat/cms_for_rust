import { Fragment, Slot, component$, useStylesScoped$ } from "@builder.io/qwik";
import { use_gaurd_or_redirect } from "~/utils/client_state";
import { use_schema_provider } from "~/utils/schema";
import css from "./styles.module.scss";

export default component$(() => {
    let gaurd = use_gaurd_or_redirect("authenticated");

    useStylesScoped$(`
        .other {
            height: 100%;
            width: 100%;
            display: flex;
justify-content: center;
align-items: center;
        }

    `);

    const pass = use_schema_provider();

    return (
        <div class={[css.panel_layout]}>
            <div class={[css.panel_body, "body"]}>
                {
                    (gaurd.value &&
                        pass.value.ty === "pass") ?
                        <Slot />
                        :
                        <div class="other">{
                            pass.value.ty === "error" ?
                                <div>fail to fetch schema: {pass.value.err}</div>
                                :
                                /* pass.value.ty === "loading" || !gaurd.value */
                                <div class="loader" />
                        }</div>
                }
            </div>
        </div>
    );
}
);
