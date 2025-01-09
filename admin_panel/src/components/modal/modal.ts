import { useStyles$, useStylesScoped$ } from "@builder.io/qwik";

export function useModalStyle() {
  useStyles$(`
    .Modal-Panel::backdrop {
      background-color: unset;
    }
    `);

  const styling = {
    blur_backdrop: () => {
      useStyles$(`
        .Modal-Panel::backdrop {
          backdrop-filter: blur(5px);
        }`);
    },
  };

  return {
    Root: `Modal-Root`,
    Panel: `Modal-Panel`,
    Trigger: `Modal-Trigger`,
    Title: `Modal-Title`,
    Description: `Modal-Description`,
    Close: `Modal-Close`,
    styling,
    apply_all_styles: () => {
      for (const key in styling) {
        (styling as any)[key]();
      }
    },
  };
}
