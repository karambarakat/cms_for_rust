import { Fragment, Slot, component$, useComputed$, useSignal, useVisibleTask$ } from "@builder.io/qwik"
import css from "./styles.module.scss"
import { use_schema } from "~/utils/schema";
import { Link, useLocation } from "@builder.io/qwik-city";



export default component$(() => {
    const schema = use_schema();
    const loc = useLocation();
    const pass = useSignal(false);

    useVisibleTask$(({ track }) => {
        track(loc);
        if (loc.isNavigating) {
            pass.value = false;
        }
        else {
            pass.value = true;
        }
    });

    return (<Fragment>
        <aside class={[css.page_sidebar, /* css.collection_sidebar */]}>
            <div class={[css.sidebar_content]}>
                {schema.value?.map((collection) => (
                    <Link key={collection.name} href={`/panel/collections/${collection.name}`}
                        class={css.sidebar_list_item}
                    >{collection.name}</Link>
                ))}
            </div>
        </aside >

        {!pass.value ? <div /> :
            <div class="page-wrapper flex-content" >
                <main class="page-content">
                    <Slot />
                </main>
            </div>
        }
    </Fragment>)
});
