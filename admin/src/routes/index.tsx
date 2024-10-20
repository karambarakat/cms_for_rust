import { $, component$, useSignal, useTask$, useVisibleTask$ } from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";
import { invoke } from "@tauri-apps/api/core";



export default component$(() => {
    const state = useSignal("");
    useVisibleTask$(() => {
        invoke("greet", { name: "Qwik" }).then((response) => {
            state.value = response as string;
        });
    });

    return (
        <>
            <h1>Hi there, 👋 </h1>
            <h2>{state}</h2>
        </>
    );
});

export const head: DocumentHead = {
    title: "Welcome to Qwik",
    meta: [
        {
            name: "description",
            content: "Qwik site description",
        },
    ],
};
