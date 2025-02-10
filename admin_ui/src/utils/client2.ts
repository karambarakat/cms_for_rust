import { createContextId, useContextProvider, useSignal, useVisibleTask$ } from "@builder.io/qwik";
import { useLocation, useNavigate } from "@builder.io/qwik-city";
import { $ } from "@builder.io/qwik";
import { SubmitHandler } from "@modular-forms/qwik";
import * as v from "valibot";

const server_error_schema =
    v.object({
        error:
            v.object({
                hint: v.string(),
                user_error: v.object({
                    code: v.string(),
                    user_hint: v.string(),
                    structured_hint: v.optional(v.map(v.string(), v.string())),
                    server_suggest: v.optional(v.array(
                        v.string()
                    ))
                })
            })
    })

type Schema =
    | {
        action: "auth/init/sign_in_first",
        input: {
            user_name: string,
            email: string,
            password: string,
        },
        output: {},
        error: string
    } | {
        action: "auth/login",
        input: {
            email_: string,
            password: string,
        },
        output: {},
        error: string
    }

const ctx = createContextId("client_context");
const event = new EventTarget();

type client_state =
    | "loading"
    | { base_url: string, token: string }
    | { base_url: string, token: null }
    | { base_url: null, token: null };

export function token_is_invalid(token: string): boolean {
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

export async function fetch_client
    <S extends Schema, A extends Schema["action"]>
    (
        action: A,
        input: S extends { action: A, input: infer I } ? I : never,
        auth_state: { backend_url: string, auth_token: string },
        abort_signal?: AbortSignal,
    ): Promise<
        | [S extends { action: A, output: infer O } ? O : never, null]
        | [null, v.InferOutput<typeof server_error_schema>["error"]["user_error"]]
    > {
    let res =
        await fetch(`${auth_state.backend_url}/${action}`, {
            method: 'POST',
            signal: abort_signal,
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${auth_state.auth_token}`
            },
            body: JSON.stringify(input),
        }).catch((e) => {
            return { catched: e };
        });

    if ("catched" in res) {
        // network error
        if (res.catched instanceof TypeError) {
            // this means either:
            // 1. the internet has disconnected
            // 2. auth_state.backend_url is not accepting requests
            //
            // I need to know which one is which
            // I have no idea how to handle error in a good way in javascript
            //
            // this client should be able to connect to localhost too, so it doesn't matter if we are connected to the internet or not
            //
            // for now let just redirect to authentification page
            event.dispatchEvent(new Event("base_url_invalid"));
            return [null, { code: "base_url_invalid", user_hint: "some error occured" }];
        }

        alert("catch unkown error in fetch");
        console.log(res.catched);
        throw res.catched
    }



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
            return [null, body.error.user_error];
        }
        alert("non-user error occured");
        return [null, { code: "unkown", user_hint: "unkown" }];
    }

    if (json === null) {
        alert("json is null");
        throw new Error("json is null");
    }

    // everything else is a success
    // if need_super_user is added then there should be an X-token header
    // but I will add if statement just in case
    let token = res.headers.get("x-cms-token");
    if (token) {
        window.localStorage.setItem("token", token);
    }

    return [json, null];
}

export const use_client_init$ = (params: {
    backend_url: string,
    auth_token: string
}) => {
    let nav = useNavigate();
    return $<SubmitHandler<{
        user_name: string,
        email: string,
        password: string,
    }>>(async (input) => {
        let res = await fetch_client("auth/init/sign_in_first", input, params);
        if (res[1]) {
            return;
        }
        nav("/panel");
    });
}
