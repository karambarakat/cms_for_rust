import { $, component$, useSignal, useTask$, useVisibleTask$ } from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";
import { invoke } from "@tauri-apps/api/core";


const list = [
    "Todo",
    "Category",
    "Tag",
];

export default component$(() => {
    const state = useSignal("");
    useVisibleTask$(() => {
        invoke("greet", {}).then((response) => {
            state.value = response as string;
        });
    });

    return (
        <div>
            <div class="min-h-50px" />
            <div class="px-7">
                <h1 class="text-2xl pb-2"> Entity Builder </h1>
                <div class="flex flex-col select-none">
                    {list.map((item) => (
                        <div
                            class="user-select-none cursor-pointer hover:bg-slate/20"
                        >{item}</div>
                    ))}
                </div>

            </div>
        </div>
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
