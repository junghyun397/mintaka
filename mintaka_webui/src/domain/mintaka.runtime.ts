import { BestMove, CommandResult, ComputingResource, HashKey, Response } from "../wasm/pkg/mintaka_wasm"
import { HistoryTree } from "./HistoryTree"
import { MintakaProvider } from "./mintaka.provider"

type IdleState = {
    readonly type: "idle",
    readonly snapshot: HashKey,
    readonly lastBestMove?: BestMove,

    commandResult(result: CommandResult): IdleState,
    launch(historySnapshot: HistoryTree): LaunchedComputingState,
}

type BaseComputingState = {
    readonly snapshot: HashKey,
    readonly historySnapshot: HistoryTree,

    bestMove(bestMove: BestMove): IdleState,
    abort(): AbortedComputingState,
}

type Streamable = BaseComputingState & {
    status(response: Extract<Response, { type: "Status" }>["content"]): StreamingComputingState,
}

type LaunchedComputingState = BaseComputingState & {
    readonly type: "launched",

    begins(response: ComputingResource): BeginsComputingState,
}

type BeginsComputingState = BaseComputingState & Streamable & {
    readonly type: "begins",
    readonly resource: ComputingResource,
}

type StreamingComputingState = BaseComputingState & Streamable & {
    readonly type: "streaming",
    readonly lastStatus: Extract<Response, { type: "Status" }>["content"],
}

type AbortedComputingState = BaseComputingState & {
    readonly type: "aborted",
    readonly resource?: BeginsComputingState["resource"],
    readonly lastStatus?: StreamingComputingState["lastStatus"],
}

export type MintakaRuntimeState = IdleState | LaunchedComputingState | BeginsComputingState | StreamingComputingState | AbortedComputingState

export type MintakaRuntime = {
    readonly provider: MintakaProvider,
    readonly state: MintakaRuntimeState,
}

export function buildMintakaRuntimeState(initialSnapshot: HashKey): MintakaRuntimeState {
    const createIdleState = (snapshot: HashKey, lastBestMove?: BestMove): IdleState => ({
        type: "idle",
        snapshot,
        lastBestMove,

        commandResult(result: CommandResult): IdleState {
            return createIdleState(result.hash_key)
        },

        launch(historySnapshot: HistoryTree): LaunchedComputingState {
            return createLaunchedState(this.snapshot, historySnapshot)
        },
    })

    const createLaunchedState = (snapshot: HashKey, historySnapshot: HistoryTree): LaunchedComputingState => ({
        type: "launched",
        snapshot,
        historySnapshot,

        begins(response: ComputingResource): BeginsComputingState {
            return createBeginsState(this.snapshot, this.historySnapshot, response)
        },

        bestMove(bestMove: BestMove): IdleState {
            return createIdleState(this.snapshot, bestMove)
        },

        abort(): AbortedComputingState {
            return createAbortedState(this.snapshot, this.historySnapshot)
        },
    })

    const createBeginsState = (snapshot: HashKey, historySnapshot: HistoryTree, resource: ComputingResource): BeginsComputingState => ({
        type: "begins",
        snapshot,
        historySnapshot,
        resource,

        status(response: Extract<Response, { type: "Status" }>["content"]): StreamingComputingState {
            return createStreamingState(this.snapshot, this.historySnapshot, response)
        },

        bestMove(bestMove: BestMove): IdleState {
            return createIdleState(this.snapshot, bestMove)
        },

        abort(): AbortedComputingState {
            return createAbortedState(this.snapshot, this.historySnapshot, this.resource)
        },
    })

    const createStreamingState = (snapshot: HashKey, historySnapshot: HistoryTree, lastStatus: Extract<Response, { type: "Status" }>["content"]): StreamingComputingState => ({
        type: "streaming",
        snapshot,
        historySnapshot,
        lastStatus,

        status(response: Extract<Response, { type: "Status" }>["content"]): StreamingComputingState {
            return createStreamingState(this.snapshot, this.historySnapshot, response)
        },

        bestMove(bestMove: BestMove): IdleState {
            return createIdleState(this.snapshot, bestMove)
        },

        abort(): AbortedComputingState {
            return createAbortedState(this.snapshot, this.historySnapshot, undefined, this.lastStatus)
        },
    })

    const createAbortedState = (
        snapshot: HashKey,
        historySnapshot: HistoryTree,
        resource?: ComputingResource,
        lastStatus?: Extract<Response, { type: "Status" }>["content"],
    ): AbortedComputingState => ({
        type: "aborted",
        snapshot,
        historySnapshot,
        resource,
        lastStatus,

        bestMove(bestMove: BestMove): IdleState {
            return createIdleState(this.snapshot, bestMove)
        },

        abort(): AbortedComputingState {
            return this
        },
    })

    return createIdleState(initialSnapshot)
}
