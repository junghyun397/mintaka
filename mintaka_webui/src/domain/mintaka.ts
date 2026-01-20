import type { Config, DurationSchema, Response } from "../wasm/pkg/rusty_renju_wasm"

export type StatusResponseBody = Extract<Response, { type: "Status" }>["content"]

export const InfiniteDuration = duration(9271584000)

export function duration(secs: number, nanos?: number): DurationSchema {
    return {
        secs,
        nanos: nanos ?? 0,
    }
}

export function durationSeconds(duration: DurationSchema): number {
    return duration.secs + duration.nanos / 1_000_000_000
}

export function formatNodes(nodesIn1k: number) {
    if (nodesIn1k >= 1_000)
        return (nodesIn1k / 1_000).toFixed(2) + "M"
    else
        return nodesIn1k + "K"
}

export type Configs = {
    readonly default_config: Config,
    readonly max_config: Config,
    readonly config: Config,
}

export type MintakaStatics = {
    readonly totalRuntime: DurationSchema,
    readonly totalNodesIn1k: number,
}

export function extractStatics(response: { total_nodes_in_1k: number, time_elapsed: DurationSchema }): MintakaStatics {
    return { totalNodesIn1k: response.total_nodes_in_1k, totalRuntime: response.time_elapsed }
}

export function nps(statics: MintakaStatics): number {
    return Math.trunc(statics.totalNodesIn1k / durationSeconds(statics.totalRuntime))
}
