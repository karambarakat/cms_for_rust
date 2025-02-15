import { component$ } from "@builder.io/qwik";
import { use_schema } from "~/utils/schema";

export default component$(() => {
    let s = use_schema();

    return (
        <div>
            <h1>collections</h1>
            {s?.length}
        </div>
    );
})
