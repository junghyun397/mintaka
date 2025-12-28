import {MintakaServerConfig} from "../domain/mintaka.server.provider";
import {Config, defaultConfig} from "../wasm/pkg/mintaka_wasm";

export type Theme = "system" | "dark" | "light"

export type HistoryDisplay = "none" | "last" | "pair" | "sequence"

export type AppConfig = {
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
        serverConfig: undefined,
        config: defaultConfig(),
        theme: "system",
        zoomBoard: true,
        historyDisplay: "last",
        openHistory: false,
        openDashboard: false,
    }
}
