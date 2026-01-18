import type { Config, HashKey } from "../wasm/pkg/mintaka_wasm"
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

    readonly config: Accessor<Config | undefined>,
    readonly setConfig: Setter<Config | undefined>,
    readonly maxConfig: Accessor<Config | undefined>,
    readonly setMaxConfig: Setter<Config | undefined>,

    readonly normEvalTable: Map<HashKey, number>,
}

export function createAppState(initial?: AppGameState): AppState {
    const appState = initial
        ?? { boardWorker: new BoardWorker(defaultBoard()), historyTree: EmptyHistoryTree }

    const [mintakaRuntime, setMintakaRuntime] = createSignal<MintakaRuntime>({ type: "none" })
    const [gameState, setGameState] = createSignal(appState)

    const [config, setConfig] = createSignal<Config | undefined>(undefined)
    const [maxConfig, setMaxConfig] = createSignal<Config | undefined>(undefined)

    const normEvalTable = new Map()

    return {
        mintakaRuntime, setMintakaRuntime,
        gameState, setGameState,
        config, setConfig, maxConfig, setMaxConfig,
        normEvalTable,
    }
}
