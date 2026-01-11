import type { HashKey } from "../wasm/pkg/mintaka_wasm"
import { defaultBoard, BoardWorker } from "../wasm/pkg/mintaka_wasm"
import { EmptyHistoryTree, HistoryEntry, HistorySource, HistoryTree } from "../domain/HistoryTree"
import { Accessor, createSignal, Setter } from "solid-js"
import { MintakaRuntime } from "../domain/mintaka.runtime"
import { assertNever } from "../utils/never"

export type AppGameState = {
    readonly boardWorker: BoardWorker,
    readonly historyTree: HistoryTree,
}

export type AppState = {
    readonly mintakaRuntime: Accessor<MintakaRuntime | undefined>,
    readonly setMintakaRuntime: Setter<MintakaRuntime | undefined>,

    readonly gameState: Accessor<AppGameState>,
    readonly setGameState: Setter<AppGameState>,

    readonly normEvalTable: Map<HashKey, number>,
}

export function createAppState(initial: {
    boardWorker?: BoardWorker,
    historyTree?: HistoryTree,
}): AppState {
    const boardWorker = initial.boardWorker ?? new BoardWorker(defaultBoard())
    const historyTree = initial.historyTree ?? EmptyHistoryTree

    const [mintakaRuntime, setMintakaRuntime] = createSignal<MintakaRuntime | undefined>(undefined)
    const [gameState, setGameState] = createSignal({ boardWorker, historyTree })
    const normEvalTable = new Map()

    return {
        mintakaRuntime, setMintakaRuntime,
        gameState, setGameState,
        normEvalTable,
    }
}

function buildGameStateFromHistorySource(historySource: HistorySource): [BoardWorker, HistoryTree] {
    switch (historySource.type) {
        case "history": {
            const historyEntries: HistoryEntry[] = []

            let boardWorker = new BoardWorker(defaultBoard())
            for (const pos of historySource.content) {
                boardWorker = boardWorker.set(pos)
                historyEntries.push({ hashKey: boardWorker.hashKey(), pos })
            }

            return [boardWorker, new HistoryTree(undefined, historyEntries)]
        }
        case "history-tree": {
            return [BoardWorker.fromHistory(historySource.content.toHistory()), historySource.content]
        }
        default: assertNever(historySource)
    }
}
