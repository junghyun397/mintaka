import type { DurationSchema, Response } from "../wasm/pkg/mintaka_wasm"

export function duration(secs: number, nanos?: number): DurationSchema {
    return {
        secs,
        nanos: nanos ?? 0,
    }
}

export const InfiniteDuration = duration(60 * 60 * 24 * 365 * 750 * 10000)

export type ResponseBody = Extract<Response, { type: "Status" }>["content"]
