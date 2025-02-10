import { Slot, component$, useStylesScoped$ } from "@builder.io/qwik";

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
        <div
            class="whole"
        >
            <Slot />
        </div>
    );
});
