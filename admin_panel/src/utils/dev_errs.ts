import { $, createContextId, Signal, useContext } from "@builder.io/qwik";
import { invoke } from "@tauri-apps/api/core";

export const developer_errors_ctx = createContextId<
  Signal<
    {
      msg: string;
      command: string;
      args: any;
    }[]
  >
>("developer_errors");

export const useInvoke$ = (command: string, args: any) => {
  const ctx = useContext(developer_errors_ctx);
  $(async () => {
    const result = await invoke(command, args).catch((err) => {
      // either handler paniced, or I provided a json that couldn't be serialized
      if (err.startsWith("Command ") && err.endsWith(" not found")) {
        ctx.value.push({ msg: err, command, args });
      } else if (
        err.startsWith("invalid args `") &&
        err.includes("` for command `")
      ) {
        ctx.value.push({ msg: err, command, args });
      } else if (err === "Panic") {
        ctx.value.push({ msg: err, command, args });
      } else {
        throw err;
      }
    });
  });
};
