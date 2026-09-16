import type { BestMove, Config, Duration, Response, Timer, TimeUnit, TimeValue } from "../wasm/pkg/rusty_renju_wasm"

export type StatusResponseBody = Extract<Response, { type: "Status" }>["content"]

function timeValueScale(unit: TimeUnit): number {
    return unit === "Clock" ? 1_000_000_000 : 1_000
}

export function timeValue(value: number, unit: TimeUnit): TimeValue {
    const scale = timeValueScale(unit)
    const whole = Math.trunc(value)
    return BigInt(whole) * BigInt(scale) + BigInt(Math.round((value - whole) * scale))
}

export function timeValueInUnit(value: TimeValue, unit: TimeUnit): number {
    return Number(value) / timeValueScale(unit)
}

export function updateTimeUnit(timer: Timer, unit: TimeUnit): Timer {
    if (timer.time_unit === unit)
        return timer

    const nanosecondsPerNode = 100n
    const convert = (value: TimeValue): TimeValue => unit === "Clock"
        ? value.valueOf() * nanosecondsPerNode
        : value.valueOf() / nanosecondsPerNode

    return {
        time_unit: unit,
        total_remaining: timer.total_remaining === undefined ? undefined : convert(timer.total_remaining),
        increment: convert(timer.increment),
        turn: timer.turn === undefined ? undefined : convert(timer.turn),
    }
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
