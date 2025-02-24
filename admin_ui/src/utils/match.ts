import * as v from "valibot";

export default function match<S>(s: v.GenericSchema<S>, obj: unknown) {

}


function test() {
    let vari = v.variant("state", [
        v.object({
            state: v.literal("case_1"),
            a: v.string(),
        }),
        v.object({
            state: v.literal("case_2"),
            b: v.number(),
        }),

    ]);
    let value = v.parse(vari, null);

    switch (value.state) {
        case "case_1":
            return `case_1: ${value.a}`;
        case "case_2":
            return `case_2: ${value.b}`;
    }

    // if (value.state === "case_1") {
    //     return `case_1: ${value.a}`;
    // } else if (value.state === "case_2") {
    //     return `case_2: ${value.b}`;
    // }

}
