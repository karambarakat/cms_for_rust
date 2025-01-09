import { Rule, definePreset } from "unocss";

export const default_base: Option["base"] = {
    50: { light: [250, 249, 253], dark: [19, 17, 33] },
    100: { light: [245, 244, 248], dark: [30, 28, 43] },
    200: { light: [235, 234, 241], dark: [40, 39, 54] },
    300: { light: [205, 204, 216], dark: [63, 62, 76] },
    400: { light: [163, 162, 177], dark: [88, 86, 99] },

    500: { light: [163, 162, 177], dark: [88, 86, 99] },
    // 100 used to be [205, 204, 216]
    // 100 used to be [235, 234, 241]
    // 200: { light: [163, 162, 177], dark: [40, 39, 54] },
    // 200: { light: [235, 234, 241] },
    // 300: { light: [121, 120, 139], dark: [63, 62, 76] },
    // 400: { light: [205, 204, 216], dark: [0, 0, 0] },
    // 500: { light: [140, 139, 148], dark: [88, 86, 99] },
    // 500: { light: [113, 112, 123], dark: [113, 112, 123] },

    600: { light: [88, 86, 99], dark: [140, 139, 148] },
    700: { light: [63, 62, 76], dark: [121, 120, 139] },
    800: { light: [40, 39, 54], dark: [163, 162, 177] },
    900: { light: [30, 28, 43], dark: [245, 244, 248] },
    950: { light: [19, 17, 33], dark: [245, 244, 248] }
}

type Option = {
    base: {
        50: { light: [number, number, number], dark: [number, number, number] },
        100: { light: [number, number, number], dark: [number, number, number] },
        200: { light: [number, number, number], dark: [number, number, number] },
        300: { light: [number, number, number], dark: [number, number, number] },
        400: { light: [number, number, number], dark: [number, number, number] },
        500: { light: [number, number, number], dark: [number, number, number] },
        600: { light: [number, number, number], dark: [number, number, number] },
        700: { light: [number, number, number], dark: [number, number, number] },
        800: { light: [number, number, number], dark: [number, number, number] },
        900: { light: [number, number, number], dark: [number, number, number] },
        950: { light: [number, number, number], dark: [number, number, number] },
    }
}

// support dark/light and primary color
export default definePreset((option: Option) => {

    const entries = Object.entries(option.base).map(([key, value]) => [parseInt(key), value] as const);
    const base = option.base;

    function interpolate(a: string): string {
        let int = parseInt(a);
        if (int < 20 || int > 980) {
            throw new RangeError("Invalid color value, 20 <= value <= 980");
        }

        if (int === 950 || int === 50 || int % 100 === 0) {
            // @ts-ignore
            return `light-dark(rgb(${base[int].light.join(",")}), rgb(${base[int].dark.join(",")}))`
        }

        if (int < 50) {
            let ratio = (int - 50) / 50;
            let lcolor = base[50].light.map((v, i) => Math.round(v + (base[100].light[i] - v) * ratio));
            let dcolor = base[50].dark.map((v, i) => Math.round(v + (base[100].dark[i] - v) * ratio));
            return `light-dark(rgb(${lcolor.join(",")}), rgb(${dcolor.join(",")}))`
        }
        if (int > 950) {
            // interpolate between 900 and 950
            let ratio = (int - 900) / 50;
            let lcolor = base[900].light.map((v, i) => Math.round(v + (base[950].light[i] - v) * ratio));
            let dcolor = base[900].dark.map((v, i) => Math.round(v + (base[950].dark[i] - v) * ratio));
            return `light-dark(rgb(${lcolor.join(",")}), rgb(${dcolor.join(",")}))`

        }

        for (let i = 0; i < entries.length - 1; i++) {
            const [key1, value1] = entries[i];
            const [key2, value2] = entries[i + 1];

            if (int >= key1 && int <= key2) {
                let ratio = (int - key1) / (key2 - key1);
                let lcolor = value1.light.map((v, i) => Math.round(v + (value2.light[i] - v) * ratio));
                let dcolor = value1.dark.map((v, i) => Math.round(v + (value2.dark[i] - v) * ratio));
                return `light-dark(rgb(${lcolor.join(",")}), rgb(${dcolor.join(",")}))`

            }
        }

        throw new Error("Invalidiii color value");
    }

    let rules: Rule[] = [
        [/^bg-base-(\d+)$/, ([, d]) => {
            let int = parseInt(d);
            return {
                ['background-color']: interpolate(d),
                color: int > 500 ? 'light-dark(white, black)' : 'light-dark(black, white)'
            }
        }],
    ];
    return {
        name: "css-color",
        rules,
    };
});
