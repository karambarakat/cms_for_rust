import { component$, useId, useSignal, useStylesScoped$, useVisibleTask$ } from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";

export default component$(() => {
    useStylesScoped$(`
.root-index {
    width: 100%;
    height: 100%; 
}
.root-index > div {
    margin-top: 30px;
    text-align: center
}
    `);
    return (
        <div class="root-index">
            <div>
                Welcome to CMS
            </div>
        </div>
    );
});

export const head: DocumentHead = {
    title: "Welcome to CMS",
    meta: [],
};
