import { Fragment, Slot, component$, useSignal, useStylesScoped$, useVisibleTask$ } from "@builder.io/qwik";
import { InferOutput } from "valibot";
import { fetch_client, schema_endpoint_schema } from "~/utils/client_fetch";
import { get_auth_state, use_gaurd_or_redirect } from "~/utils/client_state";
import { use_schema_provider } from "~/utils/schema";

export default component$(() => {
    let gaurd = use_gaurd_or_redirect("authenticated");

    useStylesScoped$(`
        .panel_layout {
            height: 100%;
            width: 100%;
        }
    `);

    let schema = useSignal<null | InferOutput<typeof schema_endpoint_schema>>(null);
    let pass = useSignal<boolean | { "error": string }>(false);
    // type-safty: <Slot /> is only rended when shcmea is not null
    use_schema_provider(schema.value as any);
    useVisibleTask$(async () => {
        let from_loc = localStorage.getItem("schema");
        if (from_loc) {
            let from_loc_js = JSON.parse(from_loc) as any;
            schema.value = from_loc_js;
            pass.value = true
            return
        }

        let state = get_auth_state();
        if (!state.success) {
            pass.value = { error: "you are not authorized to perform this action" }
            return
        }

        let act = await fetch_client("schema", null, state.ok);

        if (!act.success) {
            pass.value = { error: "failed to fetch schema" }
            return
        }

        let res = act.ok;

        console.log(res);

        let as_js = JSON.stringify(res);
        localStorage.setItem("schema", as_js);

        schema.value = res;

        pass.value = true
    }, { strategy: "document-ready" });

    return (
        <Fragment>
            {
                gaurd.value && pass.value ?
                    <div class="panel_layout" >
                        {
                            (pass.value as any).error ? <div>failed: {(pass.value as any).error}</div> :
                                schema.value ? <Slot /> : <div>built-time error</div>
                        }
                    </div> : <div class="loader" />
            }
        </Fragment>
    );
}
);
