import { component$, useId, useSignal, useVisibleTask$ } from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";

export default component$(() => {
    return (
        <div class="block">
            Welcome to CMS
        </div>
    );
});

export const head: DocumentHead = {
    title: "Welcome to CMS",
    meta: [],
};
