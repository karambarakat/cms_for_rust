import { Meta, StoryObj } from "storybook-framework-qwik";
import { Input, ScrollAreaRoot } from "./scroll_area";
import * as All from "./scroll_area";
import { useStylesScoped$ } from "@builder.io/qwik";

type Args = {
    context_args: Input;
    story_args?: {}
}

const META: Meta = {
    decorators: [
        (Story: StoryObj, {
            context_args,
            story_args,
        }: Args) => {
            ScrollAreaRoot({});
            useStylesScoped$("");

            return (
                <div>
                    <All.Main />
                </div >
            )
        },
    ],
};

export default META;

export const small = {} as StoryObj;
