import { MintakaServerConfig } from "../domain/mintaka.server.provider"
import { Config, defaultConfig } from "../wasm/pkg/mintaka_wasm"
import { MintakaProviderType } from "../domain/mintaka.provider"
import { duration } from "../domain/rusty-renju"

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

export type AppConfig = {
    readonly providerType: MintakaProviderType,
    readonly serverConfig?: MintakaServerConfig,
    readonly config: Config,
    readonly theme: Theme,
    readonly zoomBoard: boolean,
    readonly historyDisplay: HistoryDisplay,
    readonly openHistory: boolean,
    readonly openDashboard: boolean,
}

export function defaultAppConfig(): AppConfig {
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
        openHistory: false,
        openDashboard: false,
    }
}
