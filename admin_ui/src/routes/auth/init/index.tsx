import { $, Fragment, Signal, component$, useId, useSignal, useVisibleTask$ } from "@builder.io/qwik";
import { routeLoader$, useNavigate } from "@builder.io/qwik-city";
import { InitialValues, SubmitHandler, formAction$, useForm } from "@modular-forms/qwik";

type FormState = {
    user_name: string;
    email: string;
    password: string;
    confirm_password: string;
};



type Params = { token: string, backend_url: string };

export default component$(() => {
    let params = useSignal<Params | null | "invalid">(null);
    let loc = useNavigate();
    useVisibleTask$(({ track }) => {
        const url = new URL(window.location.href);
        const token = url.searchParams.get('token');
        const backend_url = url.searchParams.get('backend_url');
        if (token && backend_url) {
            params.value = { token, backend_url };
        } else {
            params.value = "invalid";
        }
    },
        { strategy: "document-ready" }
    );


    const id = useId();
    const [form, { Form, Field }] = useForm<FormState>({
        loader: useFormLoader(),
        validate: $((values) => {
            if (values.password !== values.confirm_password) {
                return { confirm_password: "Passwords do not match" };
            }
            return {}
        })
    });

    const handle_submit = $<SubmitHandler<FormState>>(async (values, eve) => {
        if (!(params.value as any).token) {
            throw new Error("build-time error: do not use handle_submit unless you fetched the params")
        }
        let params_ = params.value as any as Params;

        let res = await fetch('http://localhost:3000/auth/init/sign_in_first', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${params_.token}`
            },
            body: JSON.stringify({
                user_name: values.user_name,
                email: values.email,
                password: values.password,
            }),
        });

        let body = await res.json();

        console.log(body);

        if (res.status.toString() === "500") {
            alert("ops, some server error occured")
        }

        if (res.status.toString().startsWith("4")) {
            return [null, body.error];
        }

        let token = res.headers.get("X-token");
        if (!token) {
            throw new Error("build-time error: fetching that backend should always renew the token for one more day")
        }

        window.localStorage.setItem("token", token);
    });


    return params.value === null ?
        <div class="block txt-center">
            <span class="loader" />
        </div>
        : params.value === "invalid" ?
            <div class="block txt-center">
                you don't have access to this page
                redirect to <a href="/auth/login">login</a>
            </div>
            : <Fragment>
                <div class="content txt-center m-b-base">
                    <h4>
                        Create superuser account
                    </h4>
                </div>
                <Form onSubmit$={handle_submit}>
                    <form class="block">
                        <Field name="user_name">{(field, props) => (
                            <div class="form-field required">
                                <label for={id + "_user_name"}>
                                    User Name
                                </label>
                                <input
                                    id={id + "_user_name"}
                                    {...props}
                                    required
                                    type="string"
                                />
                                {field.error && <div class="invalid-feedback">{field.error}</div>}
                            </div>
                        )}</Field>
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

export const useFormLoader = routeLoader$<InitialValues<FormState>>(() => ({
    user_name: "",
    email: "",
    password: "",
    confirm_password: "",
}));

