import { Fragment, Slot, component$, useId, useSignal, useStylesScoped$, } from "@builder.io/qwik";
import { routeLoader$ } from "@builder.io/qwik-city";
import type { InitialValues, SubmitHandler } from '@modular-forms/qwik';
import { formAction$, useForm, valiForm$ } from '@modular-forms/qwik';

type FormState = {
    identity: string;
    password: string;
};

export const useFormLoader = routeLoader$<InitialValues<FormState>>(() => ({
    identity: "",
    password: "",
}));

export default component$(() => {
    const id = useId();
    const [form, { Form, Field }] = useForm<FormState>({
        loader: useFormLoader(),
    });

    return <Fragment>
        <div class="content txt-center m-b-base">
            <h4>
                Superuser login
            </h4>
        </div>

        <Form>
            <form class="block">
                <Field name="identity">{(field, props) => (
                    <div class="form-field required">
                        <label for={id + "_identity"}>
                            Email
                        </label>
                        <input
                            id={id + "_identity"}
                            {...props}
                            required
                            type="email"
                        />
                        {field.error && <div class="invalid-feedback">{field.error}</div>}
                    </div>
                )}</Field>
                <Field name="password">{(field, props) => (
                    <div class="form-field required">
                        <label for={id + "_password"}>
                            Password
                        </label>
                        <input
                            id={id + "_password"}
                            {...props}
                            required
                            type="password"
                        />
                        {field.error && <div class="invalid-feedback">{field.error}</div>}
                    </div>
                )}</Field>
                <button type="submit" class={["btn btn-lg btn-block btn-next"]}>
                    <span class="txt">Login</span>
                    <i class="ri-arrow-right-line" />
                </button>
            </form>
        </Form>
    </Fragment>
});
