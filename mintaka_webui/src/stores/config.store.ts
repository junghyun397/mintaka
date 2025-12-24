import {MintakaServerConfig} from "../domain/mintaka.server.provider";
import {createStore, SetStoreFunction} from "solid-js/store";

export type ConfigStore = {
    readonly serverConfig?: MintakaServerConfig,
}

export function createConfigStore(serverConfig: MintakaServerConfig): [ConfigStore, SetStoreFunction<ConfigStore>] {
    const [configStore, setConfigStore] = createStore<ConfigStore>({
        serverConfig,
    })

    return [configStore, setConfigStore]
}
