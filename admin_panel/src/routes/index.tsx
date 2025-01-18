import { component$, useContextProvider, useVisibleTask$ } from "@builder.io/qwik";
import { useNavigate, type DocumentHead } from "@builder.io/qwik-city";

export default component$(() => {
    const nav = useNavigate();
    useVisibleTask$(() => {
        if (localStorage.getItem("token") === null) {
            nav("/auth/login")
        }
    });

    return <div>
        redirect
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
