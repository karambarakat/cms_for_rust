const user_auth_state_event_target = new EventTarget();

export const auth_event_target = {
    add_event_listener: (event: events, lis: () => void): any => {
        return user_auth_state_event_target.addEventListener(event, lis);
    },
    remove_event_listener: (event: events, lis: any) => {
        user_auth_state_event_target.removeEventListener(event, lis);
    },
    dispatch_event: (event: events) => {
        user_auth_state_event_target.dispatchEvent(new Event(event));
    }
};

export type events = "logout";

type auth_state_can_fetch = { backend_url: string, auth_token: string };

export type auth_state =
    | auth_state_can_fetch // can be at /panel/*
    | { backend_url: string, auth_token: null } // should be at /auth/login
    | { backend_url: null, auth_token: null }; // should be at /auth/init

export const global_client: { auth: auth_state } = {
    auth: {
        backend_url: null,
        auth_token: null,
    }
};

type schema = {
    action: string
    input: unknown
    output: unknown
    error: unknown
}


import * as v from "valibot";

const server_error_schema =
    v.object({
        error:
            v.object({
                hint: v.string(),
                user_error: v.object({
                    code: v.string(),
                    user_hint: v.string(),
                    structured_hit: v.optional(v.map(v.string(), v.string())),
                    server_suggest: v.optional(v.array(
                        v.string()
                    ))
                })
            })
    })

function check_if_invalid(token: string): boolean {
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

export async function fetch_auth_client
    <S extends schema, A>
    (
        action: A,
        input: S extends { action: A, input: infer I } ? I : never
    ): Promise<
        | [S extends { action: A, output: infer O } ? O : never, null]
        | [null, v.InferOutput<typeof server_error_schema>["error"]]
    > {
    let backend_url = window.localStorage.getItem("backend_url");
    let auth_token = window.localStorage.getItem("token");

    if (!backend_url || !auth_token || check_if_invalid(auth_token)) {
        // the UI should redirect to /auth/login
        // this is no-op now
        return undefined as any;
    }

    let auth_state = {
        backend_url: window.localStorage.getItem("backend_url"),
        auth_token: window.localStorage.getItem("token"),
    };


    return fetch_client(action, input, auth_state as any);
}

export async function fetch_client
    <S extends schema, A>
    (
        action: A,
        input: S extends { action: A, input: infer I } ? I : never,
        auth_state: auth_state_can_fetch
    ): Promise<
        | [S extends { action: A, output: infer O } ? O : never, null]
        | [null, v.InferOutput<typeof server_error_schema>["error"]]
    > {
    let res = await fetch(`${auth_state.backend_url}/${action}`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${auth_state.auth_token}`
        },
        body: JSON.stringify(input),
    });

    if (res.status.toString() === "500") {
        // server paniced
        alert("ops, some server error occured")
    }

    // any other error should be json
    let json = await res.json();

    if (res.status.toString().startsWith("4")) {
        let body = v.parse(server_error_schema, await res.json());
        if (body.error.user_error) {
            // user can handle this error
            return [null, body.error];
        }
        console.error("network error: hint:", body.error.hint);
        // all non user-error are bugs that should be handled
        // before sending the request
        throw new Error("build-time error: error is frontend's fault")
    }

    // everything else is a success
    // if need_super_user is added then there should be an X-token header
    // but I will add if statement just in case
    let token = res.headers.get("X-Cms-Token");
    if (token) {
        window.localStorage.setItem("token", token);
    }

    return [json, null];
}
