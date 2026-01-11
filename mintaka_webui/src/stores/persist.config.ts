import { MintakaServerConfig } from "../domain/mintaka.server.provider"
import type { Config } from "../wasm/pkg/mintaka_wasm"
import { defaultConfig } from "../wasm/pkg/mintaka_wasm"
import { MintakaProviderType } from "../domain/mintaka.provider"
import { duration } from "../domain/mintaka"

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
    providerType: MintakaProviderType,
    serverConfig?: MintakaServerConfig,
    config: Config,
    theme: Theme,
    zoomBoard: boolean,
    historyDisplay: HistoryDisplay,
}

export function defaultPersistConfig(): PersistConfig {
    return {
        providerType: "worker",
        serverConfig: undefined,
        config: {
            ...defaultConfig(),
            "initial_timer": {
                total_remaining: duration(3000),
                increment: duration(0),
                turn: duration(1),
            },
        },
        theme: "system",
        zoomBoard: false,
        historyDisplay: "pair",
    }
}
