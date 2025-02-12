import { $ } from "@builder.io/qwik";
import { SubmitHandler } from "@modular-forms/qwik";
import * as v from "valibot";
import { authenticate, logout, no_connection } from "./client_state";

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

function token_is_invalid(token: string): boolean {
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
        | { success: true, ok: S extends { action: A, output: infer O } ? O : never }
        | { success: false, err: v.InferOutput<typeof server_error_schema>["error"]["user_error"] }
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
            no_connection();
            return {
                success: false,
                err: {
                    code: "network_error",
                    user_hint: "network error",
                }
            }
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
            return { success: false, err: body.error.user_error }
        }
        alert("non-user error occured");
        return { success: false, err: { code: "unkown", user_hint: "unkown" } };
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
        authenticate({ base_url: auth_state.backend_url, token: token });
    }

    return { success: true, ok: json };
}

