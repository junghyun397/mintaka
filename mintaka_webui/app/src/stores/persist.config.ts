import type { MintakaServerConfig } from "rusty-renju-web/provider/mintaka.server.provider"
import type { MintakaProviderType } from "rusty-renju-web/provider/mintaka.provider"
import { createStore, type SetStoreFunction } from "solid-js/store"
import { makePersisted } from "@solid-primitives/storage"
import { PERSIST_CONFIG_VERSION, WEB_WORKER_READY } from "rusty-renju-web/config"

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

export function createPersistConfigStore(): [PersistConfig, SetStoreFunction<PersistConfig>] {
    const [persistConfig, setPersistConfig] = makePersisted(createStore(defaultPersistConfig()),
        { name: "persist-config-" + PERSIST_CONFIG_VERSION },
    )

    if (!WEB_WORKER_READY && persistConfig.selectedProviderType === "worker") {
        setPersistConfig("selectedProviderType", "server")
    }

    return [persistConfig, setPersistConfig]
}
