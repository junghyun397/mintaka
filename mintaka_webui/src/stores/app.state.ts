import { BoardWorker, defaultBoard, HashKey, History } from "../wasm/pkg/mintaka_wasm"
import { EmptyHistoryTree, HistoryTree } from "../domain/HistoryTree"
import { Accessor, createSignal, Setter } from "solid-js"
import { MintakaRuntime } from "../domain/mintaka.runtime"

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

export function restoreAppStateFromHistory(history: History): AppState {
    return createAppState({
        boardWorker: BoardWorker.fromHistory(history),
        historyTree: EmptyHistoryTree,
    })
}
