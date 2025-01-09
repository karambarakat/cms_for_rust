import { Signal, Slot, component$, createContextId, useContextProvider, useId } from "@builder.io/qwik";

export const root_ctx = createContextId<Context>("scroll-area");

export type Input = {
    /*
     * Delay in milliseconds before the scroll area hides.
     * has to be signal to mutate, if undefined or number it is unmutable
     * @default 1000
     **/
    scrollHideDelay?: Signal<number> | number;
};

type Context = {
    scrollHideDelay: number | Signal<number>;
}

export const ScrollAreaRoot = (input: Input) => {
    useContextProvider(root_ctx, {
        scrollHideDelay: input.scrollHideDelay || 1000,
    });
};

export const Main = component$(() => {
    return <div><Slot/></div>
})

export const ScrollBar = component$(() => {
    return <div></div>
})
