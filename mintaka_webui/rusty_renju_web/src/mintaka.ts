import type { BestMove, Config, Duration, Response, TimeValue } from "../wasm/pkg/rusty_renju_wasm"

export type StatusResponseBody = Extract<Response, { type: "Status" }>["content"]

export function timeValue(secs: number): TimeValue {
    const wholeSecs = Math.trunc(secs)
    return BigInt(wholeSecs) * 1_000_000_000n + BigInt(Math.round((secs - wholeSecs) * 1_000_000_000))
}

export function timeValueSeconds(value: TimeValue): number {
    return Number(value) / 1_000_000_000
}

export function stringifyMintakaJson(value: unknown): string {
    const json = JSON as JSON & { rawJSON: (text: string) => unknown }
    return JSON.stringify(value, (_, value) => typeof value === "bigint" ? json.rawJSON(value.toString()) : value)
}

export function parseMintakaJson<T>(text: string): T {
    return JSON.parse(text, (key, value, context?: { source: string }) => {
        if (["total_remaining", "increment", "turn", "time_limit"].includes(key) && typeof value === "number")
            return BigInt(context!.source)

        return value
    })
}

export function durationSeconds(duration: Duration): number {
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
    readonly totalRuntime: Duration,
    readonly totalNodesIn1k: number,
}

export function extractStatics(response: BestMove | StatusResponseBody): MintakaStatics {
    return { totalNodesIn1k: response.total_nodes.in_1k, totalRuntime: response.time_elapsed }
}

export function nps(statics: MintakaStatics): number {
    return Math.trunc(statics.totalNodesIn1k / durationSeconds(statics.totalRuntime))
}
