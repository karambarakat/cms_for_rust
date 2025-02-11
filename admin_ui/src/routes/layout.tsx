import { Slot, component$, useStylesScoped$ } from "@builder.io/qwik";
import { client_state_provider } from "~/utils/client2";

export default component$(() => {
    useStylesScoped$(`
        .whole {
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            width: 100vw;
        }
    `);

    return (
        <div class="whole">
            <Slot />
        </div>
    )
});
