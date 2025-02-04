export const user_auth_state_event_target = new EventTarget();



export default class Client {
    load_token(): string | null {
        let token = window.localStorage.getItem("token");

        let payload = token?.split(".")[1];
        if (!payload) {
            throw new Error("build-time error: token should have 3 parts")
        }

        let payload_decoded = JSON.parse(atob(payload));

        let exp = payload_decoded.exp;

        let now = Math.floor(Date.now() / 1000);

        if (exp < now) {
            window.localStorage.removeItem("token");
            user_auth_state_event_target.dispatchEvent(new Event("logout"));
            return null;
        }

        return token;
    }

    constructor() {
        this.load_token();
    }

    async fetch_auth(endpoint: string, values: any): Promise<unknown> {
        let token = this.load_token();
        if (!token) {
            throw new Error("build-time error: the UI should redirect to login");
        }
        return await this.fetch(endpoint, values, token);
    }

    async fetch(endpoint: string, body: any, token: string): Promise<unknown> {
        return undefined as any as unknown;
        // let params_ = params.value as any as Params;
        //
        // let res = await fetch('http://localhost:3000/auth/init/sign_in_first', {
        //     method: 'POST',
        //     headers: {
        //         'Content-Type': 'application/json',
        //         'Authorization': `Bearer ${params_.token}`
        //     },
        //     body: JSON.stringify({
        //         user_name: values.user_name,
        //         email: values.email,
        //         password: values.password,
        //     }),
        // });
        //
        // let body = await res.json();
        //
        // console.log(body);
        //
        // if (res.status.toString() === "500") {
        //     alert("ops, some server error occured")
        // }
        //
        // if (res.status.toString().startsWith("4")) {
        //     return [null, body.error];
        // }
        //
        // let token = res.headers.get("X-token");
        // if (!token) {
        //     throw new Error("build-time error: fetching that backend should always renew the token for one more day")
        // }
        //
        // window.localStorage.setItem("token", token);
    }
}

