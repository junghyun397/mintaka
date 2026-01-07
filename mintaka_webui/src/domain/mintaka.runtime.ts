import type { BestMove, CommandResult, ComputingResource, HashKey } from "../wasm/pkg/mintaka_wasm"
import { HistoryTree } from "./HistoryTree"
import { MintakaProvider } from "./mintaka.provider"
import { ResponseBody } from "./rusty-renju"

class IdleState {
    readonly type: "idle" = "idle"

    constructor(readonly snapshot: HashKey, readonly lastBestMove?: BestMove) {}

    commandResult(result: CommandResult): IdleState {
        return new IdleState(result.hash_key, this.lastBestMove)
    }

    launch(historySnapshot: HistoryTree): LaunchedComputingState {
        return new LaunchedComputingState(this.snapshot, historySnapshot)
    }
}

abstract class BaseComputingState {
    protected constructor(readonly snapshot: HashKey, readonly historySnapshot: HistoryTree) {
        this.snapshot = snapshot
        this.historySnapshot = historySnapshot
    }

    bestMove(bestMove: BestMove): IdleState {
        return new IdleState(this.snapshot, bestMove)
    }

    abort(): AbortedComputingState {
        return new AbortedComputingState(this.snapshot, this.historySnapshot)
    }
}

abstract class StreamableComputingState extends BaseComputingState {
    status(response: ResponseBody): StreamingComputingState {
        return new StreamingComputingState(this.snapshot, this.historySnapshot, response)
    }
}

class LaunchedComputingState extends BaseComputingState {
    readonly type: "launched" = "launched"

    constructor(
        snapshot: HashKey, historySnapshot: HistoryTree,
    ) { super(snapshot, historySnapshot) }

    begins(response: ComputingResource): BeginsComputingState {
        return new BeginsComputingState(this.snapshot, this.historySnapshot, response)
    }
}

class BeginsComputingState extends StreamableComputingState {
    readonly type: "begins" = "begins"

    constructor(
        snapshot: HashKey, historySnapshot: HistoryTree,
        readonly resource: ComputingResource,
    ) { super(snapshot, historySnapshot) }

    status(response: ResponseBody): StreamingComputingState {
        return new StreamingComputingState(this.snapshot, this.historySnapshot, response)
    }
}

class StreamingComputingState extends StreamableComputingState {
    readonly type: "streaming" = "streaming"

    constructor(
        snapshot: HashKey, historySnapshot: HistoryTree,
        readonly lastStatus: ResponseBody,
    ) { super(snapshot, historySnapshot) }

    status(response: ResponseBody): StreamingComputingState {
        return new StreamingComputingState(this.snapshot, this.historySnapshot, response)
    }
}

class AbortedComputingState extends BaseComputingState {
    readonly type: "aborted" = "aborted"

    constructor(
        snapshot: HashKey, historySnapshot: HistoryTree,
        readonly resource?: ComputingResource, readonly lastStatus?: ResponseBody,
    ) { super(snapshot, historySnapshot) }
}

export type MintakaRuntimeState = IdleState | LaunchedComputingState | BeginsComputingState | StreamingComputingState | AbortedComputingState

export type MintakaRuntime = {
    readonly provider: MintakaProvider,
    readonly state: MintakaRuntimeState,
}

export function buildMintakaRuntime(snapshot: HashKey) {
    return new IdleState(snapshot)
}
