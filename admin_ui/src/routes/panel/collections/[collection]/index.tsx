import {
    Fragment,
    component$, useSignal, useStylesScoped$, useVisibleTask$,
} from "@builder.io/qwik";
import { snakeCase } from "change-case";
import { useLocation } from "@builder.io/qwik-city";
import * as v from "valibot";
import { one_schema, use_schema } from "~/utils/schema";
import breadcrumbs from "./breadcrumbs.module.scss";
import { fetch_client } from "~/utils/client_fetch";
import { get_auth_state } from "~/utils/client_state";

const pass_schema = v.nullable(one_schema);

const table_data = v.pipe(v.variant("state", [
    v.object({
        state: v.literal("loading"),
    }),
    v.object({
        state: v.literal("error"),
        err: v.string(),
    }),
    v.object({
        state: v.literal("empty"),
    }),
    v.object({
        state: v.literal("ready"),
        data: v.array(v.array(v.string())),
    })
]), v.transform((input) => {
    if (input.state === "ready" && input.data.length === 0) {
        return { state: "empty" as const }
    }
    return input
}))

export default component$(() => {
    const loc = useLocation();
    const schema = use_schema();
    const schema_this = useSignal<v.InferOutput<typeof pass_schema>>(null);
    useVisibleTask$(({ track }) => {
        track(loc);
        track(schema);
        if (loc.isNavigating) {
            return
        }
        if (!schema.value) {
            return
        }
        let find = schema.value.find((x) => x.name === loc.params.collection);

        if (!find) {
            return
        }

        schema_this.value = find;
    });

    // custom styles
    useStylesScoped$(`
        .page-header {
            user-select: none;
            cursor: default;
         }
    `);

    // fetch data
    const table = useSignal<v.InferOutput<typeof table_data>>({ state: "loading" });
    useVisibleTask$(async ({ track, cleanup }) => {
        track(schema_this);
        if (schema_this.value === null) {
            return
        }

        let state = get_auth_state();
        if (!state.success || state.ok.state !== "authenticated") {
            return
        }

        let res = await fetch_client(
            `collection/${snakeCase(loc.params.collection)}/get_many` as any,
            ["input"] as any,
            state.ok
        );

        if (!res.success) {
            return
        }

        let s = v.array(v.object({
            id: v.string(),
        }));

        let ok = v.safeParse(s, res.ok);

        if (!ok.success) {
            console.error("invalid schema", ok.issues);
            return
        }

        let ok2 = ok.output;

        table.value = v.parse(table_data, { state: "ready", data: ok2 })
    });

    return (
        schema_this.value !== null &&
        <Fragment>
            <header class="page-header">
                <nav class={breadcrumbs.breadcrumbs}>
                    <div class={breadcrumbs.breadcrumb_item}>Collections</div>
                    <div class={breadcrumbs.breadcrumb_item}>{schema_this.value.name}</div>
                </nav>
            </header>
            <div class="table-wrapper scroller">
                <table class="table">
                    <thead>
                        <tr>
                            {schema_this.value.fields.map((field) => (
                                <th>{field.name}</th>
                            ))}
                        </tr>
                    </thead>
                    <tbody>
                        {table.value.state === "loading" ? <Loading /> :
                            table.value.state === "error" ? <Error error={table.value.err} /> :
                                table.value.state === "empty" ? <NoData /> :
                                    table.value.data.map((row) => (
                                        <tr>
                                            {row.map((cell) => (
                                                <td>{cell}</td>
                                            ))}
                                        </tr>
                                    ))
                        }
                    </tbody>
                </table>
            </div>
        </Fragment>
    );
});

const Error = component$((props: { error: string }) => {
    return <div class="error">{props.error}</div>
});

const Loading = component$(() => {
    return <div class="loading" />
});

const NoData = component$(() => {
    return <div class="empty">No data</div>
}
);
