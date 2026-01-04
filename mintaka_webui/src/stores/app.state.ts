import { MintakaProvider } from "../domain/mintaka.provider"
import { BoardWorker, defaultBoard, HashKey, History } from "../wasm/pkg/mintaka_wasm"
import { EmptyHistoryTree, HistoryTree } from "../domain/HistoryTree"
import { Accessor, createSignal, Setter } from "solid-js"

export type AppGameState = {
    readonly boardWorker: BoardWorker,
    readonly historyTree: HistoryTree,
}

export type AppState = {
    mintakaProvider: Accessor<MintakaProvider | undefined>,
    setMintakaProvider: Setter<MintakaProvider>,

    gameState: Accessor<AppGameState>,
    setGameState: Setter<AppGameState>,

    normEvalTable: Map<HashKey, number>,
}

export function createAppState(initial: {
    boardWorker?: BoardWorker,
    historyTree?: HistoryTree,
}): AppState {
    const boardWorker = initial.boardWorker ?? new BoardWorker(defaultBoard())
    const historyTree = initial.historyTree ?? EmptyHistoryTree

    const [mintakaProvider, setMintakaProvider] = createSignal<MintakaProvider | undefined>(undefined)
    const [gameState, setGameState] = createSignal({ boardWorker, historyTree })
    const normEvalTable = new Map()

    return {
        mintakaProvider, setMintakaProvider: setMintakaProvider as Setter<MintakaProvider>,
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
