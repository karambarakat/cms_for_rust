// @ts-nocheck
import { defineConfig, definePreset } from "unocss";
import transformerVariantGroup from "@unocss/transformer-variant-group";
import presetMini from "@unocss/preset-mini";
import { variantParentMatcher } from "@unocss/preset-mini/utils";
import { animatedUno } from "animated-unocss";

type Option = {
    disable: undefined | true,
    start_with: undefined | string,
}

const css_keywords = definePreset((opt: Record<String, Option>) => {
    let rules = [];

    const specs = [
        {
            css_keyword: "mix-blend-mode",
            opt_keyword: "mix_blend_mode",
            default_shortcut: "mix-blend",
            values: ["normal", "multiply", "screen", "overlay", "darken", "lighten", "color-dodge", "color-burn", "hard-light", "soft-light", "difference", "exclusion", "hue", "saturation", "color", "luminosity", "plus-darker", "plus-lighter",]
        }
    ];

    for (const spec in specs) {
        if (!opt?.[spec.opt_keyword]?.disable) {
            let start_with = opt?.[spec.opt_keyword]?.start_with;
            let arr = spec.values;

            if (start_with) {
                for (const item in arr) {
                    rules.push([`${start_with}-${item}`, { [spec.css_keyword]: item }]);
                }
            }
            else {
                for (const item in arr) {
                    rules.push([`${spec.default_shortcut}-${item}`, { [spec.css_keyword]: item }]);
                }
            }
        }
    }

    return {
        name: "css-keywords",
        rules
    }
})

export default defineConfig({
    content: {
        filesystem: ["src/**/*.{ts,tsx}"],
    },
    shortcuts: {
        base: "dark:bg-slate-8 dark:text-white",
        "base-invert": "dark:bg-white dark:text-black",
        typo: "dark:text-white",
        "fill-as-text": "fill-black dark:fill-white",
        "stroke-as-text": "stroke-black dark:stroke-white",
        "typo-h1": "text-5xl pb-3",
        "typo-h2": "font-light text-3xl pb-3",
        "typo-lg": "text-2xl font-300",
        "typo-dim": " text-slate-700/80 dark:text-slate-200/80",

        separator: "min-w-1px min-h-1px self-stretch bg-black/80 dark:bg-white/80",
        "separator-inverted": "self-center flex-1",

        a: `
          cursor-pointer active:translate-y-1px
        text-blue-500 hover:text-blue-600 
        dark:text-blue dark:hover:text-blue-500`,
        "fill-as-a": `
          active:translate-y-1px
          fill-blue-500 hover:fill-blue-600 
          dark:fill-blue dark:hover:fill-blue-500 
    `,
    },
    rules: [
        // rule for user-select
        [
            /^select-(.+)$/,
            ([, d]) => {
                if (d !== "none" || d !== "text" || d !== "all") {
                    throw new Error("Invalid value for user-select recieved: " + d);
                }
                return {
                    "user-select": d,
                }
            },
        ],

        ["animated-paused", { "animation-play-state": "paused" }],
        ["animated-running", { "animation-play-state": "running" }],

        ["isolate", { isolation: "isolate" }],
        ["mix-blend-color", { "mix-blend-mode": "color" }],
        ["mix-blend-exclusion", { "mix-blend-mode": "exclusion" }],
        ["mix-blend-overlay", { "mix-blend-mode": "overlay" }],
        ["mix-blend-normal", { "mix-blend-mode": "normal" }],
        ["mix-blend-difference", { "mix-blend-mode": "difference" }],
        ["mix-blend-darken", { "mix-blend-mode": "darken" }],
        ["mix-blend-hue", { "mix-blend-mode": "hue" }],
        ["mix-blend-lighten", { "mix-blend-mode": "lighten" }],
        ["mix-blend-multiply", { "mix-blend-mode": "multiply" }],

        ["w-fit", { width: "fit-content" }],
        ["h-fit", { height: "fit-content" }],
        ["w-min", { width: "min-content" }],
        ["h-min", { height: "min-content" }],
        ["w-max", { width: "max-content" }],
        ["h-max", { height: "max-content" }],

        ["invert", { filter: "invert(1)" }],

        [
            /^container-(.+)$/,
            ([, d]) => ({
                width: `calc(800px + 2 * ${d})`,
                "margin-left": `calc(-${d})`,
            }),
        ],

        [
            /^container-shift-(.+)$/,
            ([, d]) => ({
                width: `calc(800px + 2 * ${d})`,
                "margin-left": `calc(-${d})`,
            }),
        ],
    ],
    theme: {
        breakpoints: {
            sm: "640px",
            md: "768px",
            lg: "1024px",
            xl: "1280px",
        },
    },
    variants: [

        variantParentMatcher("height", "@media (max-height: 700px)"),
        variantParentMatcher("motion", "#main-page.motion"),
        variantParentMatcher("no-motion", "#main-page.no-motion"),
    ],
    transformers: [
        // @ts-expect-error Type 'RegExp' is not assignable to type 'string'.ts(2322)
        transformerVariantGroup(),
    ],
    presets: [
        css_keywords(),
        presetMini({
            dark: "media",
        }),
        animatedUno(),
    ],
});


