import { $, Fragment, component$, useComputed$, useSignal, useStylesScoped$, useVisibleTask$ } from "@builder.io/qwik";
import { routeLoader$ } from "@builder.io/qwik-city";
import { setValue, useForm } from "@modular-forms/qwik";
import { use_client_state } from "~/utils/client_state";
import * as v from "valibot";
import { fetch_client } from "~/utils/client_fetch";

const use_loader = routeLoader$(() => {
    return {
        backend_url: null
    };
});

const loading_boundary = v.variant("state", [
    v.object({
        state: v.literal("ready"),
        data: v.object({
            response: v.string()
        })
    }),
    v.object({
        state: v.literal("loading")
    })
]);

type LoadingBoundary = v.InferOutput<typeof loading_boundary>;

function use_online_status() {
    const sig = useSignal<null | boolean>(null);
    useVisibleTask$(() => {
        sig.value = window.navigator.onLine
        window.addEventListener("online", () => sig.value = true)
        window.addEventListener("offline", () => sig.value = false)
    });
    return sig
}


export default component$(() => {
    let state = use_client_state();


    // let st = useComputed$(() => {
    //     console.log("state.value", state.value);
    //     return {
    //         backend_url: "backend_url"
    //     };
    //     return {
    //         backend_url: "backend_url" in state.value ? state.value.backend_url : null
    //     }
    // }
    // );
    const [form, { Form, Field }] = useForm<{ backend_url: string | null }>({
        loader: use_loader(),
    });

    let loading_boundary = useSignal<LoadingBoundary>({
        state: "loading",
    });

    let online = use_online_status();

    const test = $(async () => {
        let backend_url = "backend_url" in state.value ? state.value.backend_url : null;
        if (!backend_url) {
            return
        }
        let res = await fetch(`${backend_url}`).catch((e) => {
            return { catched: `${e.name}: ${e.message}` };
        });

        if (res instanceof Response) {
            if (res.headers.get("content-type")?.includes("text/plain")) {
                res = { catched: await res.text() };
            }
            else {
                res = { catched: "response is not parsable" };
            }
        }

        loading_boundary.value = {
            state: "ready",
            data: {
                response: res.catched
            }
        };
    })

    useVisibleTask$(async ({ track }) => {
        track(state);
        // let online = window.navigator.onLine;
        // window.navigator.on
        test()

        let backend_url = "backend_url" in state.value ? state.value.backend_url : null;
        setValue(form, "backend_url", backend_url);
    });

    useStylesScoped$(`
        .m-b-base {
            margin-bottom: 20px;
        }
        .loader {
            height: 100%;
            width: 100%;
            justify-self: center;
        }

.prompt {
    margin-top: 30px;
    margin-bottom: 10px;
}
.response {
    padding: 10px;
    background: var(--baseAlt1Color);
}
.response .oneline {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}
        `);

    return (
        <Fragment>
            <div class="content txt-center m-b-base">
                <h4>
                    Server is not responding
                </h4>
            </div>

            <div class="block txt-center">
                it seems like the server is not responding, this can be due to a network issue or the server is down.
            </div>
            <Form class="form">
                {loading_boundary.value.state === "loading" ? <div style={{ height: "292px" }} class="block txt-center">
                    <div class="loader" />
                </div> : <Fragment>
                    <div class="txt-center prompt">
                        check the server url manually
                    </div>
                    <Field name="backend_url">{(field, props) => (
                        <div class="form-field required">
                            <label for="backend_url">
                                Server URL
                            </label>
                            <input
                                id="backend_url"
                                {...props}
                                value={field.value}
                                required
                                type="text"
                            />
                        </div>

                    )}</Field>
                    <div class="response form-field">
                        Response{online.value === false && " (Offline)"}:
                        <div class="oneline">{loading_boundary.value.data.response}</div>
                    </div>
                    <button onClick$={test} type="submit" class={["btn btn-lg btn-block btn-next"]}>
                        <span class="txt">Test Again</span>
                    </button>
                </Fragment>}
            </Form>
        </Fragment >
    );
});
