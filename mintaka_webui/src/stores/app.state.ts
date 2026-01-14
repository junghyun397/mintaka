import type { HashKey } from "../wasm/pkg/mintaka_wasm"
import { defaultBoard, BoardWorker } from "../wasm/pkg/mintaka_wasm"
import { EmptyHistoryTree } from "../domain/HistoryTree"
import { Accessor, createSignal, Setter } from "solid-js"
import { AppGameState } from "../domain/rusty-renju"
import { MintakaRuntime } from "../controllers/runtime.controller"

export type AppState = {
    readonly mintakaRuntime: Accessor<MintakaRuntime>,
    readonly setMintakaRuntime: Setter<MintakaRuntime>,

    readonly gameState: Accessor<AppGameState>,
    readonly setGameState: Setter<AppGameState>,

    readonly normEvalTable: Map<HashKey, number>,
}

export function createAppState(initial?: AppGameState): AppState {
    const appState = initial
        ?? { boardWorker: new BoardWorker(defaultBoard()), historyTree: EmptyHistoryTree }

    const [mintakaRuntime, setMintakaRuntime] = createSignal<MintakaRuntime>({ type: "none" })
    const [gameState, setGameState] = createSignal(appState)
    const normEvalTable = new Map()

    return {
        mintakaRuntime, setMintakaRuntime,
        gameState, setGameState,
        normEvalTable,
    }
}
