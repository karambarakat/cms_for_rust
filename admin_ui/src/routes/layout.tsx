import { component$, createContextId, Signal, Slot, useContext, useContextProvider, useSignal, useVisibleTask$ } from "@builder.io/qwik";
import { useContent, useNavigate, type RequestHandler } from "@builder.io/qwik-city";
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
    let client_sig = useContext(client);
    if (client_sig.value === null) {
        throw new Error("do not use use_auth_client outside of authiticated pages")
    }

    return client_sig.value.fetch_auth;
}

export const global_client = new Client({
    backend_url: null,
    auth_token: null,
} as unknown as any);

const event_target = new EventTarget();
export const dispatch_event = (name: "logout" | "backend_url_set" | "auth_token_set") => {
    event_target.dispatchEvent(new Event(name));
}


// visiting /auth/init?backend_url=...&init_token=... 
// will set global_client.backend_url
//
// successful op in any /auth/* will set global_client.auth_token

export default component$(() => {
    let client_sig = useSignal<null | Client>(null);
    useContextProvider(client, client_sig);
    let nav = useNavigate();

    useVisibleTask$(() => {
        user_auth_state_event_target.addEventListener("logout", () => {
            client_sig.value = null;
            nav("/auth/login");
        });
    }, { strategy: "document-ready" });


    useVisibleTask$(() => {

    }, { strategy: "document-ready" });


    return client_sig.value === null ? <div>
        you don't have access to the current page, you will be redirected to <a href="/auth/login">login</a>
    </div> : <Slot />;
});
