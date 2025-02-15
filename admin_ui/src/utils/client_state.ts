import { QRL, Signal, createContextId, useContext, useContextProvider, useSignal, useVisibleTask$ } from "@builder.io/qwik";
import { useNavigate } from "@builder.io/qwik-city";
import * as v from "valibot";

const client_state_context = createContextId<Signal<client_state>>("client_state");

const events = new EventTarget();

class SetData extends Event {
    data: client_state;
    constructor(cb: (old: client_state) => client_state) {
        super("set_data")
        let data = cb(global_client_state.value)
        this.data = data
    }
}

export const set_state =
    (cb: (old: client_state) => client_state) => {
        events.dispatchEvent(new SetData(cb));
    }

export const logout = () => {
    events.dispatchEvent(new Event("logout"));
}

export const no_connection = () => {
    events.dispatchEvent(new Event("no_connection"));
}


export const get_auth_state = () => {
    let state = localStorage.getItem("client_state");
    if (!state) {
        return { success: false as const, err: "no client_state" }
    }

    return { success: true as const, ok: { backend_url: "", auth_token: "" } }

}

export const authenticate = ({ base_url, token }: { base_url: string, token: string }) => {
    let store: client_state = { state: "authenticated", base_url, token };
    localStorage.setItem("client_state", JSON.stringify(store));
    set_state((_ol) => ({ state: "authenticated", base_url, token }));
    // events.dispatchEvent(new Event("login"));
}

export const global_client_state: { value: Exclude<client_state, { state: "loading" }> } = { value: { state: "need_set_up", base_url: null, token: null } };

export const client_state_provider = () => {
    let sig = useSignal<client_state>({ state: "loading" });
    useContextProvider(client_state_context, sig);

    // load from local storage
    useVisibleTask$(() => {
        let state = JSON.parse(localStorage.getItem("client_state") || "null");
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

    useVisibleTask$(({ cleanup }) => {
        function to(ev: SetData) {
            sig.value = ev.data
        }
        events.addEventListener("set_data", to as any)
        cleanup(() => events.removeEventListener("set_data", to as any));
    });

    // listen for events
    useVisibleTask$(({ cleanup }) => {
        const logout = () => {
            if (sig.value.state === "authenticated") {
                sig.value = {
                    state: "need_auth",
                    base_url: sig.value.base_url,
                    token: null
                };
            } else {
                throw new Error("should not logout while you are not authenticated")
            }
        }
        events.addEventListener("logout", logout);
        cleanup(() => events.removeEventListener("logout", logout));

        const auth = () => {
            const state = localStorage.getItem("client_state");
            if (!state) throw new Error()
            const pars = v.parse(client_state_schema, JSON.parse(state));
            sig.value = pars;
        }

        events.addEventListener("login", auth);
        cleanup(() => events.removeEventListener("login", auth));
    });
};

const gaurd_context = createContextId<Signal<boolean>>("gaurd_context");

export const use_gaurd_provider = () => {
    const gaurd = useSignal(false);
    useContextProvider(gaurd_context, gaurd);
    return gaurd
};

export const use_gaurd = () => {
    return useContext(gaurd_context);
}

export const use_gaurd_or_redirect = (input_state: v.InferOutput<typeof client_state_schema>["state"]) => {
    let ready = useSignal(false);
    let nav = useNavigate();

    const client_state = use_client_state();
    useVisibleTask$(({ track }) => {
        track(client_state);
        let state = client_state.value.state;
        if (state === input_state) {
            ready.value = true;
        }
        else {
            if (state === "need_auth") {
                nav("/auth/login");
            } else if (state === "authenticated") {
                nav("/panel")
            } else if (state === "need_set_up") {
                nav("/auth/init")
            } else if (state === "no_connection") {
                nav("/auth/no_connection")
            }
        }
    }, { strategy: "document-ready" });

    return ready;
}

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
        state: v.literal("no_connection"),
        base_url: v.union([v.string(), v.null()]),
        token: v.union([v.string(), v.null()]),
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
