import {
  $,
  createContextId,
  QRL,
  Signal,
  useContext,
  useContextProvider,
  useId,
  useSignal,
  useTask$,
} from "@builder.io/qwik";

const ctx_id =
  createContextId<Signal<{ id: string; alt_name?: string } | null>>("only_one");

export function useOnlyOneRoot(ctx: {
  init?: string;
  on_change?: QRL<() => void>;
}) {
  const one = useSignal<{ id: string; alt_name?: string } | null>(null);
  if (ctx.on_change) {
    useTask$(({ track }) => {
      track(one);
      if (ctx.on_change) {
        ctx.on_change();
      }
    });
  }
  useContextProvider(ctx_id, one);
  return one;
}

export function useOnlyOneChild(over: {
  alt_name?: string;
  on_deactivate?: QRL<() => void>;
}): QRL<() => void> {
  let id = useId();
  let ctx = useContext(ctx_id);
  useTask$(({ track }) => {
    track(ctx);
    if (ctx.value?.id !== id) {
      over.on_deactivate?.();
    }
  });
  return $(() => {
    ctx.value = { id, alt_name: over.alt_name };
  });
}
