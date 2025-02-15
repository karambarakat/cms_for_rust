import { Signal, createContextId, useContext, useContextProvider } from "@builder.io/qwik";
import { InferOutput } from "valibot";
import * as v from "valibot"

export const ctx = createContextId<Schema>("schema");

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

export const use_schema_provider = (v: Schema) => {
    useContextProvider(ctx, v)
}

export const use_schema = () => {
    let sc = useContext(ctx);
    return sc
}
