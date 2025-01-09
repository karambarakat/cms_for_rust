import { component$, useContextProvider, useVisibleTask$ } from "@builder.io/qwik";
import { useNavigate, type DocumentHead } from "@builder.io/qwik-city";

export default component$(() => {
    const nav = useNavigate();
    useVisibleTask$(() => {
        nav("/entity/todo")
    });

    return <div>
        redirect
        <button class ="block underline text-blue-500" onClick$={() => {
            nav("/entity/todo")
        }}>if not redireced, click here</button>
    </div>
});

export const head: DocumentHead = {
    title: "Admin Panel",
    meta: [
        {
            name: "description",
            content: "Content Management System Admin Panel",
        },
    ],
};
