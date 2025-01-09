import { definePreset } from "unocss";

const specs: { css_keyword: string, default_shortcut: string, values: string[] }[] = [
    {
        css_keyword: "mix-blend-mode",
        default_shortcut: "blend",
        values: [
            "normal",
            "multiply",
            "screen",
            "overlay",
            "darken",
            "lighten",
            "color-dodge",
            "color-burn",
            "hard-light",
            "soft-light",
            "difference",
            "exclusion",
            "hue",
            "saturation",
            "color",
            "luminosity",
            "plus-darker",
            "plus-lighter",
        ],
    },
    {
        css_keyword: "width",
        default_shortcut: "w",
        values: ["auto", "min-content", "max-content", "fit-content", "stretch"],
    },
    {
        css_keyword: "height",
        default_shortcut: "h",
        values: ["auto", "min-content", "max-content", "fit-content", "stretch"],
    },
    {
        css_keyword: "min-width",
        default_shortcut: "min-w",
        values: ["auto", "min-content", "max-content", "fit-content", "stretch"],
    },
    {
        css_keyword: "min-height",
        default_shortcut: "min-h",
        values: ["auto", "min-content", "max-content", "fit-content", "stretch"],
    },
    {
        css_keyword: "justify-content",
        default_shortcut: "jc",
        values: [
            "center",
            "start",
            "end",
            "flex-start",
            "flex-end",
            "left",
            "right",
            "normal",
            "space-between",
            "space-around",
            "space-evenly",
            "stretch",
        ],
    },
    {
        css_keyword: "justify-items",
        default_shortcut: "ji",
        values: [
            "normal",
            "stretch",
            "center",
            "start",
            "end",
            "flex-start",
            "flex-end",
            "self-start",
            "self-end",
            "left",
            "right",
            "anchor-center",
            "baseline",
        ],
    },
    {
        css_keyword: "justify-self",
        default_shortcut: "js",
        values: [
            "auto",
            "normal",
            "stretch",
            "center",
            "start",
            "end",
            "flex-start",
            "flex-end",
            "self-start",
            "self-end",
            "left",
            "right",
            "anchor-center",
            "baseline",
        ],
    },
    {
        css_keyword: "align-content",
        default_shortcut: "ac",
        values: [
            "normal",
            "start",
            "center",
            "end",
            "flex-start",
            "flex-end",
            "baseline",
            "first baseline",
            "last baseline",
            "space-between",
            "space-around",
            "space-evenly",
            "stretch",
        ],
    },
    {
        css_keyword: "align-items",
        default_shortcut: "ai",
        values: [
            "normal",
            "stretch",
            "center",
            "start",
            "end",
            "flex-start",
            "flex-end",
            "self-start",
            "self-end",
            "anchor-center",
            "baseline",
        ],
    },
    {
        css_keyword: "align-self",
        default_shortcut: "as",
        values: [
            "auto",
            "normal",
            "center",
            "start",
            "end",
            "self-start",
            "self-end",
            "flex-start",
            "flex-end",
            "anchor-center",
            "baseline",
            "stretch",
        ],
    },
];

const css_keywords = definePreset(() => {
    let rules = [];

    for (const spec of specs) {
        const start_with = spec.default_shortcut;

        for (const value of spec.values) {
            rules.push([`${start_with}-${value}`, { [spec.css_keyword]: value }]);
        }
    }

    // special cases
    rules.push(["uppercase", { ['text-transform']: 'uppercase' }]);
    rules.push(["lowercase", { ['text-transform']: 'lowercase' }]);
    rules.push(["capitalize", { ['text-transform']: 'capitalize' }]);
    rules.push(["normal-case", { ['text-transform']: 'none' }]);


    return {
        name: "css-keywords",
        rules,
    };
});

export default css_keywords;
