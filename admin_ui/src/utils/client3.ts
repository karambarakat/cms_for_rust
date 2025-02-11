import { Signal, createContextId, useContext, useContextProvider, useSignal, useVisibleTask$ } from "@builder.io/qwik";
import * as v from "valibot";

const client_state_context = createContextId<Signal<client_state>>("client_state");

const events = new EventTarget();

export const logout = () => {
    events.dispatchEvent(new Event("logout"));
}

export const global_client_state: { value: Exclude<client_state, { state: "loading" }> } = { value: { state: "need_set_up", base_url: null, token: null } };

export const client_state_provider = () => {
    let sig = useSignal<client_state>({ state: "loading" });
    useContextProvider(client_state_context, sig);

    // load from local storage
    useVisibleTask$(() => {
        let state = localStorage.getItem("client_state");
        let parsed = v.safeParse(client_state_schema, state);
        if (parsed.success) {
            sig.value = parsed.output;
        } else {
            sig.value = { state: "need_set_up", base_url: null, token: null };
        }
    }, { strategy: "document-ready" });

    // keep global value in sync
    useVisibleTask$(({ track }) => {
        track(() => sig.value);
        if (sig.value.state === "loading") return;
        global_client_state.value = sig.value;
    });

    // listen for events
    useVisibleTask$(({ cleanup }) => {
        const logout = () => {
            sig.value = { state: "need_auth", base_url: sig.value.base_url, token: null };
        }
        events.addEventListener("logout", logout);
        cleanup(() => events.removeEventListener("logout", logout));
    });

};

export const use_client_state = () => {
    const ctx = useContext(client_state_context);
    return ctx;
}

export const client_state_schema = v.union([
    v.object({
        state: v.literal("authenticated"),
        base_url: v.string(),
        token: v.string(),
    }),
    v.object({
        state: v.literal("need_auth"),
        base_url: v.string(),
        token: v.null(),
    }),
    v.object({
        state: v.literal("loading"),
    }),
    v.object({
        state: v.literal("need_set_up"),
        base_url: v.null(),
        token: v.null(),
    }),
]);

export type client_state = v.InferOutput<typeof client_state_schema>;
