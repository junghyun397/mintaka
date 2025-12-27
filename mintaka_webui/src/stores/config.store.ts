import {MintakaServerConfig} from "../domain/mintaka.server.provider";
import {Config} from "../wasm/pkg/mintaka_wasm";
import {createStore, SetStoreFunction} from "solid-js/store";

export type Theme = "system" | "dark" | "light"

export type AppConfig = {
    readonly serverConfig?: MintakaServerConfig,
    readonly config: Config,
    readonly theme: Theme,
    readonly openHistory: boolean,
}

export function createAppConfigStore(appConfig: AppConfig): [AppConfig, SetStoreFunction<AppConfig>] {
    return createStore(appConfig)
}
