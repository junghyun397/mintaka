import {MintakaServerConfig} from "./mintaka.server.provider";

export type Config = {
    readonly serverConfig?: MintakaServerConfig,
    readonly providerKind: "device" | "server",
}
