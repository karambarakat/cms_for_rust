import {
    $,
    Fragment,
    NoSerialize,
    QRL,
    Slot,
    component$, useContext, useId, useSignal, useStylesScoped$, useVisibleTask$,
    
} from "@builder.io/qwik";

import { Modal } from "@qwik-ui/headless";
import modal_s from "./modal.module.scss";
import { snakeCase } from "change-case";
import { useContent, useLocation } from "@builder.io/qwik-city";
import * as v from "valibot";
import { one_schema, use_schema } from "~/utils/schema";
import breadcrumbs from "./breadcrumbs.module.scss";
import { fetch_client } from "~/utils/client_fetch";
import { get_auth_state } from "~/utils/client_state";
import { Result } from "~/utils/valibot_ext";
import { SubmitHandler, useForm, valiForm$ } from "@modular-forms/qwik";

const pass_schema = v.nullable(one_schema);

const table_data_ready = v.array(v.array(v.string()));
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
        data: table_data_ready,
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
            // @ts-expect-error
            console.error("invalid schema", res2.issues, " data: ", res.output);
            return
        }

        // @ts-expect-error
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

    // modal
    let modal = useSignal(false);
    useVisibleTask$(({ cleanup }) => {
        let s = () => modal.value = true;
        let ss = setTimeout(s, 1000);
        cleanup(() => clearTimeout(ss));
    }, { strategy: "document-ready" });

    return (
        schema_this.value !== null &&
        <Fragment>
            <Modal.Root bind:show={modal}>
                <header class="page-header this_header">
                    <nav class={breadcrumbs.breadcrumbs}>
                        <div class={breadcrumbs.breadcrumb_item}>Collections</div>
                        <div class={breadcrumbs.breadcrumb_item}>{schema_this.value.name}</div>
                    </nav>
                    <button class="btn" onClick$={() => { modal.value = true }}>
                        <i class="ri-add-line" />
                        <span>Add</span>
                    </button>
                </header>
                <div class="table-wrapper scroller">
                    {table.value.state === "loading" && <Loading />}
                    {table.value.state === "error" && <Error error={table.value.err} />}
                    {table.value.state === "empty" && <div>
                        <table class="table">
                            <thead>
                                <tr>
                                    {schema_this.value.fields.map((field) => (
                                        <th>{field.name}</th>
                                    ))}
                                </tr>
                            </thead>
                        </table>
                        <div class="empty">
                            <div>No data</div>
                            <div>consider adding</div>
                        </div>
                    </div>}
                    {table.value.state === "ready" &&
                        <TableReady
                            schema={schema_this.value}
                            data={table.value.data}
                        />
                    }
                </div>
                <Modal.Panel class={modal_s.sheet}>
                    <AddNewEntry
                        schema={schema_this.value}
                    />
                </Modal.Panel>
            </Modal.Root>

        </Fragment >
    );
});

const TableReady = component$(({ schema, data }: { schema: v.InferOutput<typeof one_schema>, data: v.InferOutput<typeof table_data_ready> }) => {
    return <div>dsf</div>
});

function valibot_schema(schema: v.InferOutput<typeof one_schema>) {
    const s = {
        __no_serialize__: v.literal(true),
    };

    return v.pipe(v.intersect(
        [v.record(
            v.intersect([v.string(), v.number(), v.boolean()]),
            v.string(), "record is requierd"),
        v.object({
            __no_serialize__: v.literal(true),
        })]), v.transform((input) => {
            input.__no_serialize__ = true;
            return input;
        }))
}

type JSInput = string | number | boolean;

function empty(s: v.InferOutput<typeof one_schema>) {
    const default_: NoSerialize<Record<string, JSInput>> = {
        ss: "",
        __no_serialize__: true,
    };
    return default_
}


const AddNewEntry = component$(({ schema }: { schema: v.InferOutput<typeof one_schema>, }) => {
    const schema_s = valibot_schema(schema);
    const init = useSignal(empty(schema));
    const handle_submit: QRL<SubmitHandler<v.InferOutput<typeof schema_s>>> =
        $((vals, event) => { });
    const [store, { Form, Field }] = useForm(
        {
            loader: init,
        }
    );
    // form styles
    useStylesScoped$(`
        .add_action {
            display: flex;
            gap: 10px;
            justify-content: flex-end;
        }`);

    let ctx = useContext(Modal.context);
    let id = useId();

    return <MiddleOverflow>
        <div q:slot="header">
            <h1>Add new entry</h1>
        </div>
        <div q:slot="content">
            <Form onSubmit$={handle_submit}>
                {schema.fields.map((field) => (
                    <Field name={field.name} type={field.js_type}>
                        {(field, props) => (
                            <div>
                                <label for={id + field.name}>{field.name}</label>
                                <input id={id + field.name} type="text" />
                                {field.error && <div>{field.error}</div>}
                            </div>
                        )}
                    </Field>
                ))}
            </Form>
        </div>
        <div q:slot="footer" class="add_action">
            <button class="btn btn-secondary"
                onClick$={() => { ctx.showSig.value = false }}
            >
                Cancel
            </button>
            <button class="btn">Add</button>
        </div>
    </MiddleOverflow >;
});


const MiddleOverflow = component$(() => {
    // basic behavior
    useStylesScoped$(`
        .overflow-root {
            display: flex;
            flex-direction: column;
            height: 100%;
        }
        .overflow-header {
        }
        .overflow-content {
flex: 1;
overflow: auto;
        }
        .overflow-footer {
        }
        `);

    // spacing
    useStylesScoped$(`
    .overflow-header, .overflow-footer {
        padding: 20px;
    }
    .overflow-content {
        padding: 0 20px;
    `);
    return <div class="overflow-root">
        <div class="overflow-header"><Slot name="header" /></div>
        <div class="overflow-content"><Slot name="content" /></div>
        <div class="overflow-footer"><Slot name="footer" /></div>
    </div>
});


const Error = component$((props: { error: string }) => {
    return <div class="error">{props.error}</div>
});

const Loading = component$(() => {
    return <div class="loading" />
});
