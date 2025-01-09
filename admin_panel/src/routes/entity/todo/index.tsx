import { Resource, Signal, Slot, component$, createContextId, useContext, useContextProvider, useResource$, useSignal, useTask$ } from "@builder.io/qwik";

const DataTable = component$((
    { columns, data }: {
        columns: { field: string, headerName: string }[],
        data: Signal<Record<string, string>[]>
    },
) => {
    let names = useSignal<string[]>([]);
    let t_data = useSignal<string[][]>([]);
    useTask$(({ track }) => {
        track(() => data.value);
        track(() => columns);
        let i_t_data: string[][] = [];
        for (const row of data.value) {
            let i_row = [];
            for (const name of columns) {
                let row_item = row[name.field];
                if (!row_item) {
                    throw new Error("all rows should have keys")
                }
                i_row.push(row_item);
            }
            i_t_data.push(i_row);
        }
        names.value = columns.map((e) => e.headerName);
        t_data.value = i_t_data;
    });

    return <div class="border border-base-300 relative overflow-x-auto sm:rounded-lg">
        <table class={["w-full text-left rtl:text-right"]}>
            <thead class="uppercase font-sans">
                {names.value.map((col) =>
                    <th scope="col" class="bg-base-120 px-6 py-3 border-b border-b-base-300">{col}</th>
                )}
                <th scope="col" class="bg-base-120 px-6 py-3 border-b border-b-base-300 ">action</th>
            </thead>
            <tbody>
                {
                    t_data.value.map((row) =>
                        <tr class="hover:bg-base-150">
                            {row.map((col) => <td class="px-6 py-4">{col}</td>)}
                            <td class="px-6 py-4 min-w-140px" >
                                <button class="text-red pr2">Delete</button>
                                <button class="text-blue">Edit</button>
                            </td>
                        </tr>
                    )
                }
            </tbody>
        </table>
    </div>;
});

type Entity = {
    name: string,
    fields: {
        name: string,
        type: string,
    }[]
};

let ctx = createContextId<Signal<undefined | Record<string, Entity>>>("dsf");

export const ctx_provider = () => {
    let val = useSignal<undefined | Record<string, Entity>>(undefined);
    useTask$(() => {
        val.value = {
            todo: {
                name: "Todo",
                fields: [
                    { name: "done", type: "bool" },
                    { name: "title", type: "string" },
                    { name: "description", type: "string" },
                    {
                        name: "new_title",
                        type: "string"
                    }
                ]
            }
        };

    })
    useContextProvider(ctx, val);
    if (val === undefined) {
        return <div>pending</div>
    } else {
        return <Slot />
    }
}

export const ctx_cons = () => {
    let val = useContext(ctx);
    if (val.value === undefined) {
        throw new Error("values are not initialized")
    } else {
        return val.value
    }
}

export default component$(() => {
    const todo = ctx_cons().todo;

    const res = useResource$(async ({ track }) => {
        return [
            { new_title: "hello", new_ttile_3: "hi", title: "hi", done: "d", description: "sd" },
            { new_title: "hello", new_ttile_3: "hi", title: "hi", done: "d", description: "sd" },
            { new_title: "hello", new_ttile_3: "hi", title: "hi", done: "d", description: "sd" },
            { new_title: "hello", new_ttile_3: "hi", title: "hi", done: "d", description: "sd" }
        ]
    });

    if (todo === undefined) {
        throw new Error("entry with that name does not exist")
    }

    return <div>
        <h1 class="mb4 text-3xl">{todo.name}</h1>
        <Resource
            value={res}
            onRejected={() =>
                <div>
                    <DataTable
                        columns={
                            todo.fields.map((e) => {
                                return {
                                    field: e.name,
                                    headerName: e.name.toUpperCase()
                                }
                            })
                        }
                        data={{
                            value: []
                        }}
                    />
                    rejected
                </div>
            }
            onPending={() =>
                <div>
                    <DataTable
                        columns={
                            todo.fields.map((e) => {
                                return {
                                    field: e.name,
                                    headerName: e.name.toUpperCase()
                                }
                            })
                        }
                        data={{
                            value: []
                        }}
                    />
                    pending
                </div>
            }
            onResolved={(data) =>
                <DataTable
                    columns={
                        todo.fields.map((e) => {
                            return {
                                field: e.name,
                                headerName: e.name.toUpperCase()
                            }
                        })
                    }
                    data={{
                        value: data
                    }}
                />
            }
        />

    </div>
});
