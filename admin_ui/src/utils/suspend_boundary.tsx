import { $, ContextId, QRL, Signal, Slot, TaskCtx, TaskFn, component$, createContextId, useComputed$, useContext, useContextProvider, useId, useSignal, useStylesScoped$, useTask$, useVisibleTask$ } from "@builder.io/qwik";
import * as v from "valibot";

// todos: 
// 1. use_suspend return some data
// 2. handle errors
// 3. what if component is at a later point (test the behavior is as expected)
// 4. different layout for loading and pass

const boundary_ty = v.variant("state", [
    v.object({
        state: v.literal("loading"),
    }),
    v.object({
        state: v.literal("pass"),
    })]);

type BoundaryTy = v.InferOutput<typeof boundary_ty>;

type CtxTy = {
    sig: Signal<BoundaryTy>,
    add_listener: QRL<(inp: any) => void>,
    remove_listener: QRL<(inp: any) => void>,
}

const boundary_ctx = createContextId<CtxTy>("boundary");
const ctx2 = createContextId<Signal<BoundaryTy>>("ctx2");

export const use_boundary = () => {
    let sig = useSignal<BoundaryTy>({ state: "loading" });
    let all_listeners = useSignal(0);
    let all_listeners_2 = useSignal(new Set<string>());

    useVisibleTask$(({ track }) => {
        track(all_listeners);
        console.log("all_listeners", all_listeners.value);
        if (all_listeners.value !== 0) {
            sig.value = { state: "loading" };
        } else {
            sig.value = { state: "pass" };
        }
    }, { strategy: "document-ready" });

    useContextProvider(ctx2, sig);
    useContextProvider(boundary_ctx, {
        sig,
        add_listener: $((inp: string) => {
            all_listeners.value = all_listeners.value + 1;
            all_listeners_2.value.add(inp);
        }),
        remove_listener: $((inp: any) => {
            if (all_listeners_2.value.delete(inp)) {
                all_listeners.value = all_listeners.value - 1;
            }
        })
    })

    return component$(() => {
        let sig = useContext(ctx2);
        useStylesScoped$(`
        [data-suspend-boundary="pass"] > .loading {
            opacity: 0;
            max-height: 0;
            height: 0;
        }
        [data-suspend-boundary="loading"] > .content {
            opacity: 0;
        }
    `);

        return <div data-suspend-boundary={sig.value.state}>
            <div class="loading">loading</div>
            <div class="content"><Slot /></div>
        </div>;
    });
}

export function use_computed_suspend<Ret>(fn: QRL<(tf: TaskCtx) => Promise<Ret>>) {
    let ctx = useContext(boundary_ctx)
    useTask$(() => {
        ctx.add_listener(fn);
    });
    let ret = useSignal<Ret | undefined>(undefined);
    useVisibleTask$(async (task_ctx) => {
        try {
            ret.value = await fn(task_ctx)
        }
        catch (e: unknown) {
            console.warn("handling errors is not (correctly) implemented yet");
        }

        ctx.remove_listener(fn);
    }, { strategy: "document-ready" });

    return ret;
}

export const use_suspend = (fn: QRL<(tf: TaskCtx) => Promise<void>>) => {
    let ctx = useContext(boundary_ctx)
    useTask$(() => {
        ctx.add_listener(fn);
    });
    useVisibleTask$(async (task_ctx) => {
        try {
            await fn(task_ctx)
        }
        catch (e: unknown) {
            console.warn("handling errors is not (correctly) implemented yet");
        }

        ctx.remove_listener(fn);
    }, { strategy: "document-ready" });
}


// mod test {
//
// import { $, component$, useStylesScoped$ } from "@builder.io/qwik";
// import { use_boundary, use_suspend } from "~/utils/suspend_boundary";
//
// export default component$(() => {
//     let Bound = use_boundary();
//
//     return <Bound>
//         <div class="content">root<Sub1 /></div>
//     </Bound>;
// });
//
// const Sub1 = component$(() => {
//     use_suspend($(async () => {
//         await new Promise((resolve) => {
//             setTimeout(resolve, 2001);
//         });
//         console.log("Sub1");
//     }));
//
//
//
//     return <div>Sub1
//        <Sub2 />
//     </div>;
// });
//
//
// const Sub2 = component$(() => {
//     let Bound = use_boundary();
//
//     return <Bound>
//         <div class="content">sub2<Sub3 /></div>
//     </Bound>;
// });
//
// const Sub3 = component$(() => {
//     use_suspend($(async () => {
//         await new Promise((resolve) => {
//             setTimeout(resolve, 4001);
//         });
//         console.log("Sub3");
//     }));
//
//     return <div>Sub3 </div>;
// });
// }
