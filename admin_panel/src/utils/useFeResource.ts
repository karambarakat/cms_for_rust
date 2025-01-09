import { $, QRL, useSignal } from "@builder.io/qwik";

export type State<R> =
  | {
      state: "inactive";
      resolved: undefined;
      rejected: undefined;
    }
  | {
      state: "pending";
      resolved: undefined;
      rejected: undefined;
    }
  | {
      state: "resolved";
      resolved: R;
      rejected: undefined;
    }
  | {
      state: "rejected";
      resolved: undefined;
      rejected: unknown;
    };

export function useFeResource<R, A>(fn: QRL<(args: A) => Promise<R>>) {
  const state = useSignal<State<R>>({
    state: "inactive",
    resolved: undefined,
    rejected: undefined,
  });

  const action = $(async (all: A) => {
    state.value = {
      state: "pending",
      rejected: undefined,
      resolved: undefined,
    };
    // @ts-ignore
    await fn(all)
      .then((val) => {
        state.value = {
          state: "resolved",
          rejected: undefined,
          resolved: val,
        };
      })
      .catch((e) => {
        state.value = {
          state: "rejected",
          rejected: e,
          resolved: undefined,
        };
      });
  });

  const opt = {
    reset: $(() => {
      state.value = {
        state: "inactive",
        resolved: undefined,
        rejected: undefined,
      };
    }),
  };

  return [action, state, opt] as const;
}
