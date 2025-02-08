import { $, Fragment, Signal, component$, useId, useSignal, useVisibleTask$ } from "@builder.io/qwik";
import { routeLoader$, useNavigate } from "@builder.io/qwik-city";
import { InitialValues, SubmitHandler, formAction$, useForm } from "@modular-forms/qwik";
import { fetch_client } from "~/utils/client";

type FormState = {
    user_name: string;
    email: string;
    password: string;
    confirm_password: string;
};

type Params = { init_token: string, backend_url: string };

export default component$(() => {
    let params = useSignal<Params | null>(null);

    const loading = useSignal(true);

    useVisibleTask$(() => {
        const url = new URL(window.location.href);
        const init_token = url.searchParams.get('init_token');
        const backend_url = url.searchParams.get('backend_url');
        if (init_token && backend_url) {
            // uncode backend_url from base64
            const backend_url2 = atob(backend_url);
            // for some reason ToBase64 trait in rust encodes strings
            // with the quote
            //
            // here I take them out
            const backend_url3 = backend_url2.replace(/^"/, "").replace(/"$/, "");
            params.value = { init_token, backend_url: backend_url3 };
        } else {
            // should I throw an error here?
        }

        loading.value = false;
    },
        { strategy: "document-ready" }
    );

    // because document-ready is not as eager as I want
    // and to prevent layout-shift,
    //
    // I will display loading until the the task has completed

    if (loading.value) {
        return <div class="block txt-center">
            <div class="loader" />
        </div>
    }

    return params.value === null ?
        <div class="block txt-center">
            <ManualSetUp signal={params} />
        </div>
        :
        <ParamsNotNull params={params.value} />
})

const ManualSetUp = component$(({ signal }: { signal: Signal<null | Params | "invalid"> }) => {
    const loader = useSignal<Params>({
        backend_url: "",
        init_token: "",
    });

    const [_, { Form, Field }] = useForm<Params>({
        loader,
        validate: $((values) => {
            if (!values.backend_url || !values.init_token) {
                return { backend_url: "required", token: "required" };
            }
            return {};
        })
    });

    const handle_submit = $<SubmitHandler<Params>>(async (values) => {
        signal.value = values;
    });

    return <div>
        <Form onSubmit$={handle_submit}>
            <div class="block">
                <div class="content txt-center m-b-base">
                    <h4>
                        Could not find your backend, set it up manually
                    </h4>
                </div>
                <Field name="backend_url">{(field, props) => (
                    <div class="form-field">
                        <label>
                            Backend URL
                        </label>
                        <input
                            {...props}
                            required
                            type="string"
                        />
                        {field.error && <div class="invalid-feedback">{field.error}</div>}
                    </div>
                )}</Field>
                <Field name="init_token">{(field, props) => (
                    <div class="form-field">
                        <label>
                            Initiation Token
                        </label>
                        <input
                            {...props}
                            required
                            type="password"
                        />
                        {field.error && <div class="invalid-feedback">{field.error}</div>}
                    </div>
                )}</Field>
                <button type="submit" class={["btn btn-lg btn-block btn-next"]}>
                    <span class="txt">Set up</span>
                    <i class="ri-arrow-right-line" />
                </button>
            </div>
        </Form>
    </div>
});


const ParamsNotNull = component$(({ params }: { params: Params }) => {
    const id = useId();
    const loader = useSignal({
        user_name: "",
        email: "",
        password: "",
        confirm_password: "",
    });

    const [form, { Form, Field }] = useForm<FormState>({
        loader,
        validate: $((values) => {
            if (!values.user_name || !values.email || !values.password || !values.confirm_password) {
                return { user_name: "required", email: "required", password: "required", confirm_password: "required" };
            }

            if (values.password !== values.confirm_password) {
                return { confirm_password: "Passwords do not match" };
            }
            return {}
        })
    });

    type Schema =
        | {
            action: "auth/init/sign_in_first", input: {
                user_name: string,
                email: string,
                password: string,
            }, output: {}, error: string
        }

    const handle_submit = $<SubmitHandler<FormState>>(async (values, eve) => {
        let res = await fetch_client<Schema, "auth/init/sign_in_first">(
            "auth/init/sign_in_first",
            {
                user_name: values.user_name,
                email: values.email,
                password: values.password,
            },
            { backend_url: params.backend_url, auth_token: params.init_token }
        );

        if (res[0]) {
            // I should see localStoreage has token
            let token = window.localStorage.getItem("token");
            if (!token) {
                throw new Error("build-time error: auth/init/sign_in_first should set localStorage token")
            }

            // found unvalid
            window.localStorage.setItem("backend_url", params.backend_url);
        }

    });
    return (
        <Fragment>
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
    );
});
