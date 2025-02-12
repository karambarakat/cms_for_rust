import { Slot, component$, useId, useStylesScoped$, useVisibleTask$ } from "@builder.io/qwik";
import { client_state_provider, use_gaurd_provider } from "../utils/client_state";

export default component$(() => {
    client_state_provider();
    // let sig = use_gaurd_provider();
    useStylesScoped$(`
        .whole {
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            width: 100vw;
        }
    `);

    // const id = useId();
    // useVisibleTask$(({ track }) => {
    //     track(sig);
    //     console.log(sig.value);
    //     if (sig.value) {
    //         document.getElementById(id)?.classList.remove("loading");
    //     } else {
    //         document.getElementById(id)?.classList.add("loading");
    //     }
    // });

    return (
        <div class="whole">
            <Slot />
        </div>
    )
});
