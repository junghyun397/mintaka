import { MintakaProvider } from "../domain/mintaka.provider"
import {BoardWorker, defaultBoard, HashKey, History} from "../wasm/pkg/mintaka_wasm"
import { EmptyHistoryTree, HistoryTree } from "../domain/HistoryTree"

export type AppState = {
    mintakaProvider?: MintakaProvider,
    boardWorker: BoardWorker,
    historyTree: HistoryTree,
    evalTable: Map<HashKey, number>,
}

export function restoreAppStateFromHistory(history: History): AppState {
    return {
        boardWorker: BoardWorker.fromHistory(history),
        historyTree: EmptyHistoryTree,
        evalTable: new Map(),
    }

}

export function createDefaultAppState(): AppState {
    return {
        boardWorker: new BoardWorker(defaultBoard()),
        historyTree: EmptyHistoryTree,
        evalTable: new Map(),
    }
}
