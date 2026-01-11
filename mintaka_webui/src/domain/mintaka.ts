import type { DurationSchema, Response } from "../wasm/pkg/mintaka_wasm"

export type StatusResponseBody = Extract<Response, { type: "Status" }>["content"]

export function duration(secs: number, nanos?: number): DurationSchema {
    return {
        secs,
        nanos: nanos ?? 0,
    }
}

export const InfiniteDuration = duration(9271584000)
