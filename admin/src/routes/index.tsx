import {
  $,
  component$,
  Signal,
  useSignal,
  useStore,
  useTask$,
  useVisibleTask$,
} from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";
import { invoke } from "@tauri-apps/api/core";

type RustType = "String" | "bool";

type FieldTy = {
  name: string;
  type: RustType;
  optional?: boolean;
};

type EntityTy = {
  name: string;
  fields: Array<FieldTy>;
};

export default component$(() => {
  let shown_entity = useSignal<string | null>("Project");
  let entities = useStore<EntityTy[]>([
    {
      name: "Project",
      fields: [
        {
          name: "title",
          type: "String",
        },
        {
          name: "Done",
          type: "bool",
        },
        {
          name: "Description",
          type: "String",
          optional: true,
        },
      ],
    },
  ]);
  return (
    <div>
      <div class="min-h-50px" />
      <div class="px-7">
        <h1 class="text-2xl mb3"> Entity Builder </h1>
        <div class="flex flex-col select-none">
          {entities.map((item) => (
            <Entity data={item} shown={shown_entity} />
          ))}
        </div>
      </div>
    </div>
  );
});

const Entity = component$(
  ({ data, shown }: { data: EntityTy; shown: Signal<string | null> }) => {
    const toggleShown = $(() => {
      if (shown.value === data.name) {
        shown.value = null;
      } else {
        shown.value = data.name;
      }
    });

    return (
      <div>
        <div
          class="-mx3 p-3 user-select-none cursor-pointer rounded bg-slate/10 hover:bg-slate/20 flex item-strech"
          onClick$={toggleShown}
        >
          <span class="flex-1">{data.name}</span>
          <span>{shown.value === data.name ? "▼" : "▶"}</span>
        </div>
        {shown.value === data.name && (
          <div class="py4 user-select-none cursor-default">
            <div class="opacity-50">Fields:</div>
            {data.fields.map((field) => (
              <Field data={field} />
            ))}
          </div>
        )}
      </div>
    );
  },
);

const Field = component$(({ data }: { data: FieldTy }) => {
  const editing = useSignal(false);
  const data_local = useStore({ ...data });
  const update = $(() => {
    editing.value = false;
    data = data_local;
  });

  return (
    <div
      class=""
      onClick$={() => {
        if (editing.value === false) {
          editing.value = true;
        }
      }}
    >
      {editing.value ? (
        <div class="group -mx3 pl3 py3 my3 rounded bg-slate/20">
          <div class="pt3">
            name:
            <input
              type="text"
              value={data_local.name}
              onInput$={(_, target) => {
                data_local.name = target.value;
              }}
              class="ml2"
            />
          </div>
          <div class="pt3">
            type:
            <select
              value={data_local.type}
              onChange$={(_, target) => {
                // @ts-expect-error
                data_local.type = target.value;
              }}
              class="ml2"
            >
              <option value="String">String</option>
              <option value="bool">bool</option>
            </select>
          </div>
          <div class="py3">
            optional:
            <label>
              <input
                type="checkbox"
                class="ml2"
                checked={data_local.optional}
                onChange$={(_, target) => {
                  data_local.optional = target.checked;
                }}
              />
              Optional
            </label>
          </div>
          <div class="py3">
            <botton class="p-2 rounded bg-slate" onClick$={update}>
              Update
            </botton>
          </div>
        </div>
      ) : (
        <div class="flex group -mx3 pl3 py1 my1 flex rounded hover:bg-slate/20">
          <div class="flex-1">
            {data_local.name} ({data_local.type}){" "}
            {data_local.optional ? "(optional)" : ""}
          </div>
          <span
            class="group-hover:block hidden text-blue px2 cursor-pointer"
            onClick$={() => {
              editing.value = true;
            }}
          >
            ✎ edit
          </span>
        </div>
      )}
    </div>
  );
});

export const head: DocumentHead = {
  title: "Welcome to Qwik",
  meta: [
    {
      name: "description",
      content: "Qwik site description",
    },
  ],
};
