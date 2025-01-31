import { $, Fragment, Signal, component$, useId, useSignal, useVisibleTask$ } from "@builder.io/qwik";
import { routeLoader$, useNavigate } from "@builder.io/qwik-city";
import { InitialValues, SubmitHandler, formAction$, useForm } from "@modular-forms/qwik";

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

type Params = { token: string, backend_url: string };

export default component$(() => {
    let params = useSignal<Params | null | "invalid">(null);
    let loc = useNavigate();
    useVisibleTask$(({ track }) => {
        track(params);
        const url = new URL(window.location.href);
        const token = url.searchParams.get('token');
        const backend_url = url.searchParams.get('backend_url');
        if (token && backend_url) {
            params.value = { token, backend_url };
        } else {
            loc('/auth/login');
        }
    },
        { strategy: "document-ready" }
    );



    return params.value === null ?
        <div class="block txt-center">
            <span class="loader" />
        </div>
        : params.value === "invalid" ?
            <div class="block txt-center">
                you don't have access to this page
                redirect to <a href="/auth/login">login</a>
            </div>
            : <ThisForm data={params as any} />


})

const ThisForm = component$(({ data }: { data: Signal<Params> }) => {
    const id = useId();
    const [form, { Form, Field }] = useForm<FormState>({
        loader: useFormLoader(),
        validate: $((values) => {
            if (values.password !== values.confirm_password) {
                return { confirm_password: "Passwords do not match" };
            }
            return { }
        })
    });

    const handle_submit = $<SubmitHandler<FormState>>(async (values, eve) => {
        let res = await fetch(data.value.backend_url + '/auth/init_set_up_first', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${data.value.token}`
            },
            body: JSON.stringify({
                token: data.value.token,
                email: values.email,
                password: values.password,
            }),
        });
    });

    return <Fragment>
        <div class="content txt-center m-b-base">
            <h4>
                Create superuser account
            </h4>
        </div>
        <Form onSubmit$={handle_submit}>
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
});
