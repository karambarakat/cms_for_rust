import {
    component$,
    Slot,
} from "@builder.io/qwik";
import { ctx_provider } from "./entity/todo";

const Card = component$(() => {
    ctx_provider();
    return (
        <div class="pt14 px3 flex jc-center ac-center">
            <div class="max-w700px w-full p3 rounded border border-base-300 shadow-lg">
                <Slot />
            </div>
        </div>
    );
});

export default component$(() => {
    return (
        <Card>
            <Slot />
        </Card>
    );
});
