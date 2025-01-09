import { $ } from "@builder.io/qwik";
import { invoke } from "@tauri-apps/api/core";
import { Endpoints } from "./endpoints";

type EndpointMap = {
    [E in Endpoints as E["name"]]: {
        input: E["input"];
        output: E["output"];
    };
};

export type InputOf<Key extends Endpoints["name"]> = EndpointMap[Key]["input"];
export type OutputOf<Key extends Endpoints["name"]> =
    EndpointMap[Key]["output"];

type Handler = <Key extends keyof EndpointMap>(
    key: Key,
    opt: EndpointMap[Key]["input"],
) => Promise<EndpointMap[Key]["output"] | undefined>;

// type TauriError =
//   | {
//       kind: "UnkownOk";
//       content: unknown;
//     }
//   | {
//       kind: "ErrNotString";
//       content: unknown;
//     }
//   | {
//       kind: "CommandNotFound";
//       content: {
//         command: string;
//       };
//     }
//   | {
//       kind: "InvalidArgs";
//       content: {
//         args: unknown;
//       };
//     }
//   | {
//       kind: "Panic";
//       content: string;
//     };
//
// const DevErrorsGlobal = new EventTarget();
// export const DEV_ERRORS: Array<TauriError> = [];
// function dispatch(err: TauriError) {
//   DEV_ERRORS.push(err);
//   DevErrorsGlobal.dispatchEvent(new CustomEvent("error"));
// }
// export const dev_err = {
//   add_listener(fn: EventListenerOrEventListenerObject | null) {
//     DevErrorsGlobal.addEventListener("error", fn);
//   },
//   remove_listenr(fn: EventListenerOrEventListenerObject | null) {
//     DevErrorsGlobal.removeEventListener("error", fn);
//   },
// };

const backend$: Handler = $(async (endpoint: string, opt: any) => {
    if (typeof window === "undefined") {
        throw new Error("no window");
    }
    const result = await invoke(endpoint, { arg: opt })
        .catch((_) => {
            return undefined;
        });

    return result;
}) as any;

export default backend$;
