import type { HashKey } from "../wasm/pkg/rusty_renju_wasm"
import { Accessor, createSignal, Setter } from "solid-js"
import { createStore, SetStoreFunction } from "solid-js/store"
import type { AppGameState } from "../domain/rusty-renju"
import type { MintakaRuntime } from "../controllers/runtime.controller"

export type AppState = {
    readonly mintakaRuntime: Accessor<MintakaRuntime>,
    readonly setMintakaRuntime: Setter<MintakaRuntime>,

    readonly gameState: Accessor<AppGameState>,
    readonly setGameState: Setter<AppGameState>,

    readonly winRateTable: Record<HashKey, number>,
    readonly setWinRateTable: SetStoreFunction<Record<HashKey, number>>,
}

export function createAppState(initial: AppGameState): AppState {
    const [mintakaRuntime, setMintakaRuntime] = createSignal<MintakaRuntime>({ type: "none" })
    const [gameState, setGameState] = createSignal(initial)

    const [winRateTable, setWinRateTable] = createStore<Record<HashKey, number>>({})

    return {
        mintakaRuntime, setMintakaRuntime,
        gameState, setGameState,
        winRateTable, setWinRateTable,
    }
}
