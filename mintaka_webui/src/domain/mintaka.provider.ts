import type { BestMove, Command, CommandResult, Config, HashKey, Response, SearchObjective } from "../wasm/pkg/mintaka_wasm"

export type MintakaProviderType = "server" | "worker"

export type MintakaProviderResponse =
    | { type: "CommandResult", content: CommandResult }
    | Response
    | { type: "BestMove", content: BestMove }

export type MintakaProviderRuntimeCommand =
    { type: "abort" }

export interface MintakaProvider {
    readonly type: MintakaProviderType,
    readonly maxConfig: Config,
    subscribeResponse(handler: (response: MintakaProviderResponse) => void): void,
    subscribeError(handler: (error: any) => void): void,
    dispose(): void,
    command(command: Command): void,
    launch(positionHash: HashKey, objective: SearchObjective): void
    control(command: MintakaProviderRuntimeCommand): void,
}
