import { QRL, Signal, createContextId, useContext, useContextProvider, useSignal, useVisibleTask$ } from "@builder.io/qwik";
import { useNavigate } from "@builder.io/qwik-city";
import * as v from "valibot";

const client_state_context = createContextId<Signal<client_state>>("client_state");

const events = new EventTarget();

function token_is_valie(token: string): boolean {
    // check if token is expired
    let payload = token.split(".")[1];
    let decoded = JSON.parse(atob(payload));
    let exp = decoded.exp;
    let now = Math.floor(Date.now() / 1000);
    if (now > exp) {
        return true;
    }
    return false;
}

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

// @deprecated
export const logout = () => {
    events.dispatchEvent(new Event("logout"));
}



// this is a function to be used outside the boundary of component
export const get_auth_state = () => {
    let state = localStorage.getItem("client_state");
    if (!state) {
        return { success: false as const, err: "no client_state" }
    }

    let parsed = v.safeParse(client_state_schema, JSON.parse(state));

    if (!parsed.success) {
        return { success: false as const, err: "client_state is invalid" }
    }

    return { success: true as const, ok: parsed.output }
}

export const authenticate = ({ base_url, token }: { base_url: string, token: string }) => {
    let store: client_state = { state: "authenticated", backend_url: base_url, token };
    localStorage.setItem("client_state", JSON.stringify(store));
    set_state((_ol) => ({ state: "authenticated", backend_url: base_url, token }));
    // events.dispatchEvent(new Event("login"));
}

export const global_client_state: { value: client_state } = { value: { state: "loading" } };

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
            sig.value = { state: "need_set_up" };
        }
    }, { strategy: "document-ready" });

    // keep global value in sync
    useVisibleTask$(({ track }) => {
        track(() => sig.value);
        global_client_state.value = sig.value;
    });

    // accept mutation globally
    useVisibleTask$(({ cleanup }) => {
        function to(ev: SetData) {
            sig.value = ev.data
        }
        events.addEventListener("set_data", to as any)
        cleanup(() => events.removeEventListener("set_data", to as any));
    });

    // // @deprecated
    // useVisibleTask$(({ cleanup }) => {
    //     const logout = () => {
    //         if (sig.value.state === "authenticated") {
    //             sig.value = {
    //                 state: "need_auth",
    //                 backend_url: sig.value.backend_url,
    //             };
    //         } else {
    //             throw new Error("should not logout while you are not authenticated")
    //         }
    //     }
    //     events.addEventListener("logout", logout);
    //     cleanup(() => events.removeEventListener("logout", logout));
    //
    //     const auth = () => {
    //         const state = localStorage.getItem("client_state");
    //         if (!state) throw new Error()
    //         const pars = v.parse(client_state_schema, JSON.parse(state));
    //         sig.value = pars;
    //     }
    //
    //     events.addEventListener("login", auth);
    //     cleanup(() => events.removeEventListener("login", auth));
    // });
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

const backend_url = v.pipe(v.string(), v.url());
const token = v.pipe(v.string(), v.minLength(1));

export const client_state_schema = v.variant("state", [
    v.object({
        state: v.literal("authenticated"),
        backend_url,
        token,
    }),
    v.object({
        state: v.literal("no_connection"),
        backend_url: v.nullable(backend_url),
        token: v.nullable(token),
    }),
    v.object({
        state: v.literal("need_auth"),
        backend_url: backend_url,
    }),
    v.object({
        state: v.literal("loading"),
    }),
    v.object({
        state: v.literal("need_set_up"),
    }),
]);

const client_state_schema2 = v.pipe(
    client_state_schema,
    v.transform((input) => {
        if (input.state === "authenticated" && !token_is_valie(input.token)) {
            return { state: "need_auth", backend_url: input.backend_url }
        }
        return input
    })
);

export type client_state = v.InferOutput<typeof client_state_schema>;
