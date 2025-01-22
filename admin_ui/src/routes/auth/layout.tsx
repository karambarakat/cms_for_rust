import { Fragment, Slot, component$, useSignal } from "@builder.io/qwik";

export default component$(() => {
    const isLoading = useSignal(false);
    return (
        <Fragment>
            {isLoading.value &&
                <div class="block txt-center">
                    <span class="loader" />
                </div>
            }

            <div class={["page-wrapper full-page center center-content"]} >
                <main class="page-content">
                    <div class="wrapper wrapper-sm m-b-xl panel-wrapper">
                        <Slot />
                    </div>
                </main>
                <footer class="page-footer">
                    <Slot name="footer" />
                </footer>
            </div>
        </Fragment>
    );
}
);
