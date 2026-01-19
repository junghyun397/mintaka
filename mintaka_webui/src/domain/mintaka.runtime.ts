import type { BestMove, CommandResult, ComputingResource, HashKey } from "../wasm/pkg/rusty_renju_wasm"
import { HistoryTree } from "./HistoryTree"
import { MintakaProvider } from "./mintaka.provider"
import { StatusResponseBody } from "./mintaka"

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

    abort(): AbortingComputingState {
        return new AbortingComputingState(this.snapshot, this.historySnapshot)
    }
}

abstract class StreamableComputingState extends BaseComputingState {
    status(response: StatusResponseBody): StreamingComputingState {
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

    status(response: StatusResponseBody): StreamingComputingState {
        return new StreamingComputingState(this.snapshot, this.historySnapshot, response)
    }
}

class StreamingComputingState extends StreamableComputingState {
    readonly type: "streaming" = "streaming"

    constructor(
        snapshot: HashKey, historySnapshot: HistoryTree,
        readonly lastStatus: StatusResponseBody,
    ) { super(snapshot, historySnapshot) }

    status(response: StatusResponseBody): StreamingComputingState {
        return new StreamingComputingState(this.snapshot, this.historySnapshot, response)
    }
}

class AbortingComputingState extends BaseComputingState {
    readonly type: "aborting" = "aborting"

    constructor(
        snapshot: HashKey, historySnapshot: HistoryTree,
        readonly resource?: ComputingResource, readonly lastStatus?: StatusResponseBody,
    ) { super(snapshot, historySnapshot) }
}

export type MintakaRuntimeState = IdleState | LaunchedComputingState | BeginsComputingState | StreamingComputingState | AbortingComputingState

export function buildMintakaRuntime(snapshot: HashKey) {
    return new IdleState(snapshot)
}
