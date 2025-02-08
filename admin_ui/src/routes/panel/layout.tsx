import { Slot, component$, useSignal, useVisibleTask$ } from "@builder.io/qwik";
import { global_client, auth_event_target } from "../../utils/client";

export default component$(() => {
    const pass = useSignal(false);
    useVisibleTask$(({ cleanup }) => {
        if (
            global_client.auth.auth_token === null
            || global_client.auth.backend_url === null
        ) {
            pass.value = false;
        } else {
            pass.value = true;
        }

        let cb = auth_event_target.add_event_listener("logout", () => {
            pass.value = false;
        });

        cleanup(() => {
            auth_event_target.remove_event_listener("logout", cb);
        })


    }, { strategy: "document-ready" });

    const href = useSignal("/auth/login");
    useVisibleTask$(({ track }) => {
        track(pass);

        if (
            global_client.auth.backend_url !== null
            && global_client.auth.auth_token === null
        ) {
            href.value = "/auth/login";
        } else if (
            global_client.auth.backend_url === null
            && global_client.auth.auth_token === null
        ) {
            href.value = "/auth/init";
        }
        else {
            throw new Error("build-time error: invalid state");
        }


    });


    if (pass) {
        return <Slot />
    } else {
        return <div> you are not authorized to view this page, you will be redirected to <a href={href.value}>authinticate</a></div>
    }
});
