import type { BestMove, Command, CommandResult, HashKey, Response, SearchObjective } from "../wasm/pkg/rusty_renju_wasm"

export type MintakaProviderType = "server" | "worker"

export type MintakaProviderResponse =
    | { type: "CommandResult", id: number, content: CommandResult }
    | Response
    | { type: "BestMove", content: BestMove }
    | { type: "Error", content: unknown }

export type MintakaProviderRuntimeCommand =
    { type: "abort" }

export interface MintakaProvider {
    readonly type: MintakaProviderType,
    subscribeResponse(handler: (response: MintakaProviderResponse) => void): void,
    dispose(): void,
    command(command: Command): void,
    launch(positionHash: HashKey, objective: SearchObjective): void
    control(command: MintakaProviderRuntimeCommand): void,
}
