import * as v from 'valibot';

export function Result<Ok, Err>(ok: v.GenericSchema<Ok>, err: v.GenericSchema<Err>) {
    return v.variant("success", [
        v.object({ success: v.literal(true), ok }),
        v.object({ success: v.literal(false), err }),
    ]);
}

export type ResultTy<O, E> = v.InferOutput<ReturnType<typeof Result<O, E>>>;
