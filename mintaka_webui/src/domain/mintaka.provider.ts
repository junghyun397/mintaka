import type { BestMove, Command, CommandResult, Config, HashKey, Response, SearchObjective } from "../wasm/pkg/mintaka_wasm"
import { MintakaServerConfig } from "./mintaka.server.provider"

export type MintakaProviderType = "server" | "worker"

export type MintakaProviderResponse =
    | { type: "CommandResult", content: CommandResult }
    | Response
    | { type: "BestMove", content: BestMove }
    | { type: "Error", content: any }

export type MintakaProviderRuntimeCommand =
    { type: "abort" }

export interface MintakaProvider {
    readonly type: MintakaProviderType,
    readonly defaultConfig: Config,
    readonly maxConfig: Config,
    subscribeResponse(handler: (response: MintakaProviderResponse) => void): void,
    dispose(): void,
    command(command: Command): void,
    launch(positionHash: HashKey, objective: SearchObjective): void
    control(command: MintakaProviderRuntimeCommand): void,
}
