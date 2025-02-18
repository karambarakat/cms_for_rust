import { Slot, component$, useSignal, useVisibleTask$ } from "@builder.io/qwik";
import { useNavigate } from "@builder.io/qwik-city";
import { use_schema } from "~/utils/schema";

export default component$(() => {
    const navigate = useNavigate();

    useVisibleTask$(() => {
        navigate("/panel/collections");
    }, { strategy: "document-ready" });

    return <div class="block txt-center">
        <div class="loader" />
    </div>
});
