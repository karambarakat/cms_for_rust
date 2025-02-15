import { ContextId, QRL, Signal, createContextId, useComputed$, useContext, useContextProvider, useId, useSignal, useVisibleTask$ } from "@builder.io/qwik";

export const create_boundary = () => {

};

type CtxTy = {
    sig: Signal<Map<string, boolean>>
}

const boundary = createContextId<CtxTy>("boundary");

export const use_boundary_provider = () => {
    const suspend = useSignal<Map<string, boolean>>(new Map());
    useContextProvider(boundary, {
        sig: suspend
    });

    let sig = useComputed$(() => {
        if (Array.from(suspend.value.values()).every((e) => e)) {
            return true
        } else return false

    })

    return { suspend: sig };
}

// should I take action as QRL or return {pass, fail}
export const use_boundary = (ctx: ContextId<unknown>, action:
    QRL<(force_to_fail: () => void) => void>
) => {
    let self: CtxTy = useContext(ctx) as any;
    let id = useId();
    useVisibleTask$(() => {
        self.sig.value = { ...self.sig.value, [id]: false };
        let fail = false;
        let make_fail = () => fail = true;
        action(make_fail);
        if (fail) {
            throw new Error("todo");
        }
        else {
            let clone = self.sig.value;
            clone.set(id, true);
            self.sig.value = clone;
        }
    }, { strategy: "document-ready" })

    return {
        pass: () => {},
        fail: (msg: string) => {}
    }
}
