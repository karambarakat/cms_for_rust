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

// export const global_client: { auth: auth_state } = {
//     auth: {
//         backend_url: null,
//         auth_token: null,
//     }
// };

type schema = {
    action: string
    input: unknown
    output: unknown
    error: unknown
}


import * as v from "valibot";





