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
import { Result } from "~/utils/valibot_ext";

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
    useVisibleTask$(async ({ track }) => {
        track(schema_this);
        if (schema_this.value === null) {
            return
        }

        let state = get_auth_state();
        if (!state.success || state.ok.state !== "authenticated") {
            return
        }

        function get_many<S>(sc: v.GenericSchema<S>) {
            return v.object({
                page_count: v.number(),
                data: v.array(
                    v.object({
                        id: v.number(),
                        attr: sc,
                        relations: v.object({}),
                    })
                ),
            })
        }

        const say_this = Result(get_many(v.unknown()), v.null());

        let res = await fetch_client(
            `collection/${snakeCase(loc.params.collection)}/get_many` as any,
            {
                filters: {},
                relations: {},
                pagination: {
                    page: 1,
                    page_size: 10,
                },
            } as any,
            state.ok
        );

        let res2 = v.safeParse(say_this, res);

        if (!res.success) {
            console.error("invalid schema", res2.issues, " data: ", res.output);
            return
        }

        let ok2 = res2.output.ok.data;

        table.value = v.parse(table_data, { state: "ready", data: ok2 })
    });

    // .empty part
    useStylesScoped$(`
        .empty {
            padding: 20px;
            display: flex;
            flex-direction: column;
            gap: 10px;
            justify-content: center;
            align-items: center;
            }`);

    // add entries
    useStylesScoped$(`
        .this_header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 10px;
        }
    `);


    return (
        schema_this.value !== null &&
        <Fragment>
            <header class="page-header this_header">
                <nav class={breadcrumbs.breadcrumbs}>
                    <div class={breadcrumbs.breadcrumb_item}>Collections</div>
                    <div class={breadcrumbs.breadcrumb_item}>{schema_this.value.name}</div>
                </nav>
                <button class="btn">Add</button>
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
                                table.value.state === "empty" ?
                                    <div class="empty">
                                        <div>No data</div>
                                        <div>consider adding</div>
                                    </div>
                                    :
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
