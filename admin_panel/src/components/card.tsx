
const Card = component$(() => {
    // ctx_provider();
    return (
        <div><Slot /></div>
        // <div class="pt14 px3 flex jc-center ac-center">
        //     <div class="max-w700px w-full p3 rounded border border-base-300 shadow-lg">
        //         <Slot />
        //     </div>
        // </div>
    );
});
