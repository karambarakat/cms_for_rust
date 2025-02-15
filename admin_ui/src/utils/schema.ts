import { Signal, createContextId, useContext, useContextProvider, useSignal, useVisibleTask$ } from "@builder.io/qwik";
import { InferOutput } from "valibot";
import * as v from "valibot"
import { get_auth_state } from "./client_state";
import { fetch_client } from "./client_fetch";

export const ctx = createContextId<Signal<Schema>>("schema");

export const schema_endpoint_schema = v.array(v.object({
    name: v.string(),
    fields: v.array(v.object({
        name: v.string(),
        s_type: v.union([
            v.literal("String"),
            v.literal("Todo"),
        ])
    }))
}));

type Schema = InferOutput<typeof schema_endpoint_schema>;

type Pass = { ty: "pass" } | { ty: "error", err: string } | { ty: "loading" };


export const use_schema_provider = () => {
    let schema = useSignal<null | InferOutput<typeof schema_endpoint_schema>>(null);
    useContextProvider(ctx, schema);
    let pass = useSignal<Pass>({ ty: "loading" });
    useVisibleTask$(async () => {
        let from_loc = localStorage.getItem("schema");
        if (from_loc) {
            let from_loc_js = JSON.parse(from_loc) as any;
            schema.value = from_loc_js;
            pass.value = { ty: "pass" }
            return
        }

        let state = get_auth_state();
        if (!state.success) {
            pass.value = { ty: "error", err: "you are not authorized to perform this action" }
            return
        }

        console.log(state);

        let act = await fetch_client("schema", null, state.ok);

        if (!act.success) {
            // pass.value = { error: "failed to fetch schema" }
            pass.value = { ty: "error", err: "failed to fetch schema" }
            return
        }

        let res = act.ok;

        let as_js = JSON.stringify(res);
        localStorage.setItem("schema", as_js);

        schema.value = res;

        pass.value = { ty: "pass" }
    }, { strategy: "document-ready" });


    return pass
}

export const use_schema = () => {
    let sc = useContext(ctx);
    useVisibleTask$(({ track }) => {
        track(() => sc.value);
        if (!sc.value) {
            console.error("you should not render this page until schema is loaded")
            throw new Error("schema not found in context")
        }
    });

    return sc.value as Schema | null
}
