import { Slot, component$ } from "@builder.io/qwik";

export default component$(() => {
    return (
        <div class="page-wrapper center center-content" >
            <main class="page-content">
                <div class="wrapper wrapper-sm m-b-xl panel-wrapper">
                    <Slot />
                </div>
            </main>
            <footer class="page-footer">
                <Slot name="footer" />
            </footer>
        </div>
    );
});
