// @ts-nocheck
import { defineConfig, definePreset } from "unocss";
import transformerVariantGroup from "@unocss/transformer-variant-group";
import presetMini from "@unocss/preset-mini";
import { variantParentMatcher } from "@unocss/preset-mini/utils";
import { animatedUno } from "animated-unocss";
import css_keywords from "./uno_css_keywords";
import css_colors, { default_base } from "./uno_css_colors";

export default defineConfig({
    content: {
        filesystem: ["src/**/*.{ts,tsx}"],
    },
    shortcuts: {
        base: "dark:bg-slate-8 dark:text-white",
        "base-invert": "dark:bg-white dark:text-black",
        "bg-primary":
            "text-white bg-blue-700 hover:bg-blue-800 dark:bg-blue-600 dark:hover:bg-blue-700",
        "ring-primary": "ring-4 ring-blue-300 dark:ring-blue-800",
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
        btn_al: `
    y-2.5 px-5 me-2 mb-2 rounded-lg

    text-sm font-medium text-gray-900
    hover:text-blue-700
    dark:text-gray-400
    dark:hover:text-white

    focus:outline-none

    bg-white dark:bg-gray-800
    hover:bg-gray-100 dark:hover:bg-gray-700
    focus:bg-gray-100 dark:focus:bg-gray-700

    focus:ring-4 focus:ring-gray-100 dark:focus:ring-gray-700

    border border-gray-200 dark:border-gray-600
    `,
        btn: `
    mb-2 me-2 px-5 py-2.5 rounded-lg

    text-sm font-medium text-white

    focus:outline-none
    focus:ring-4 focus:ring-blue-300
    dark:focus:ring-blue-800

    bg-blue-700 hover:bg-blue-800
    dark:bg-blue-600 dark:hover:bg-blue-700
    `,
    },
    rules: [
        ["animated-paused", { "animation-play-state": "paused" }],
        ["animate-spin", { animation: "spin 2s linear infinite" }],
        ["animated-running", { "animation-play-state": "running" }],
        ["isolate", { isolation: "isolate" }],
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
        colors: {
            base: Object.entries(default_base).reduce(
                (acc, [key, value]) => {
                    acc[key] = `light-dark(rgb(${value.light.join(",")}), rgb(${value.dark.join(",")}))`;
                    return acc;
                },
                {} as Record<string, string>
            ),
        },
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
        css_colors({
            base: default_base
        }),
        presetMini({
            dark: "media",
        }),
        animatedUno(),
    ],
});
