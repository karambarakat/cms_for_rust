import { Fragment, component$, useId, useSignal, useVisibleTask$ } from "@builder.io/qwik";
import { routeLoader$, useNavigate } from "@builder.io/qwik-city";
import { InitialValues, useForm } from "@modular-forms/qwik";

type FormState = {
    email: string;
    password: string;
    confirm_password: string;
};

export const useFormLoader = routeLoader$<InitialValues<FormState>>(() => ({
    email: "",
    password: "",
    confirm_password: "",
}));

export default component$(() => {
    let token = useSignal<string | null>(null);
    let loc = useNavigate();
    useVisibleTask$(({ track }) => {
        track(token);
        const url = new URL(window.location.href);
        const found = url.searchParams.get('token');
        if (found) {
            token.value = found;
        } else {
            loc('/auth/login');
        }
    },
        { strategy: "document-ready" }
    );

    const id = useId();
    const [form, { Form, Field }] = useForm<FormState>({
        loader: useFormLoader(),
    });

    return token == null ?
        <div class="block txt-center">
            <span class="loader" />
        </div>
        :
        <Fragment>
            <div class="content txt-center m-b-base">
                <h4>
                    Create superuser account
                </h4>
            </div>
            <Form>
                <form class="block">
                    <Field name="email">{(field, props) => (
                        <div class="form-field required">
                            <label for={id + "_email"}>
                                Email
                            </label>
                            <input
                                id={id + "_email"}
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
                    <Field name="confirm_password">{(field, props) => (
                        <div class="form-field required">
                            <label for={id + "_confirm_password"}>
                                Confirm password
                            </label>
                            <input
                                id={id + "_confirm_password"}
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

})
