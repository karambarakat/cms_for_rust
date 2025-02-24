import * as v from "valibot";

export type Action<O = unknown, E = unknown> = {
    input: (i: unknown) => [unknown, unknown],
    output: v.GenericSchema<O>,
    error: v.GenericSchema<E>,
}

export const api_is_up = {
    input: () => ["api_is_up", {}],
    output: v.object({ ok: v.literal(true) }),
} as const;

// export async function fetch_client_2
//     <S extends Action>
//     (
//         action: S,
//         input: S["input"] extends (i: infer I) => unknown ? I : never,
//         auth_state: { backend_url: string, token: string },
//         abort_signal?: AbortSignal,
//     ): Promise<
//         ResultTy<
//             S["output"] extends v.GenericSchema<infer O> ? O : never,
//             S["error"] extends v.GenericSchema<infer E> ? E : never>
//     > {
//     // type-safe check
//     v.parse(client_state_schema, auth_state)
//     let [endpoint, body] = action.input(input);
//     let res =
//         await fetch(`${auth_state.backend_url}/${endpoint}`, {
//             method: 'POST',
//             signal: abort_signal,
//             headers: {
//                 'Content-Type': 'application/json',
//                 'Authorization': `Bearer ${auth_state.token}`
//             },
//             body: JSON.stringify(body),
//         }).catch((e) => {
//             return { catched: e };
//         });
//
//
//     return { success: true, ok: v.parse(action.output, json) as any };
// }
