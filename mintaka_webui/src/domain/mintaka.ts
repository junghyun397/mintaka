import type { Config, DurationSchema, Response } from "../wasm/pkg/rusty_renju_wasm"

export type StatusResponseBody = Extract<Response, { type: "Status" }>["content"]

export function duration(secs: number, nanos?: number): DurationSchema {
    return {
        secs,
        nanos: nanos ?? 0,
    }
}

export const InfiniteDuration = duration(9271584000)

export type Configs = {
    default_config: Config,
    max_config: Config,
    config: Config,
}
