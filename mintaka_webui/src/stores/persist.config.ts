import type { MintakaServerConfig } from "../domain/mintaka.server.provider"
import type { Config } from "../wasm/pkg/rusty_renju_wasm"
import type { MintakaProviderType } from "../domain/mintaka.provider"
import { createStore } from "solid-js/store"
import { makePersisted } from "@solid-primitives/storage"

const Themes = ["system", "dark", "light"] as const

export type Theme = typeof Themes[number]

export function nextTheme(theme: Theme): Theme {
    return Themes[(Themes.indexOf(theme) + 1) % Themes.length]
}

const HistoryDisplays = ["none", "last", "pair", "sequence"] as const

export type HistoryDisplay = typeof HistoryDisplays[number]

export function nextHistoryDisplay(historyDisplay: HistoryDisplay): HistoryDisplay {
    return HistoryDisplays[(HistoryDisplays.indexOf(historyDisplay) + 1) % HistoryDisplays.length]
}

export type PersistConfig = {
    selectedProviderType: MintakaProviderType,
    serverConfig?: MintakaServerConfig,
    theme: Theme,
    zoomBoard: boolean,
    historyDisplay: HistoryDisplay,
}

export function defaultPersistConfig(): PersistConfig {
    return {
        selectedProviderType: "worker",
        serverConfig: undefined,
        theme: "system",
        zoomBoard: false,
        historyDisplay: "pair",
    }
}

export function createPersistConfigStore() {
    return makePersisted(createStore(defaultPersistConfig()))
}
