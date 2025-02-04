import { component$, createContextId, Signal, Slot, useContext, useContextProvider, useSignal, useVisibleTask$ } from "@builder.io/qwik";
import { useContent, type RequestHandler } from "@builder.io/qwik-city";
import Client, { user_auth_state_event_target } from "../utils/client";

export const onGet: RequestHandler = async ({ cacheControl }) => {
    // Control caching for this request for best performance and to reduce hosting costs:
    // https://qwik.dev/docs/caching/
    cacheControl({
        // Always serve a cached response by default, up to a week stale
        staleWhileRevalidate: 60 * 60 * 24 * 7,
        // Max once every 5 seconds, revalidate on the server to get a fresh version of this page
        maxAge: 5,
    });
};


export const client = createContextId<Signal<null | Client>>("client");

export const use_auth_client = () => {
    let client_ = useContext(client);
    if (client_.value === null) {
        throw new Error("do not use use_auth_client outside of authiticated pages")
    }

    return client_.value.fetch_auth;
}

export default component$(() => {
    let client_sig = useSignal<null | Client>(null);
    useContextProvider(client, client_sig);

    useVisibleTask$(() => {
        user_auth_state_event_target.addEventListener("logout", () => {
            client_sig.value = null;
        });
    }, { strategy: "document-ready" });


    useVisibleTask$(() => {

    }, { strategy: "document-ready" });


    return client_sig.value === null ? <div>auth</div> : <Slot />;
});
