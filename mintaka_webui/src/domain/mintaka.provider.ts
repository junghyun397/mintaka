import type { BestMove, Command, CommandResult, HashKey, Response, SearchObjective } from "../wasm/pkg/rusty_renju_wasm"

export type MintakaProviderType = "server" | "worker"

export type MintakaProviderResponse =
    | Response
    | { type: "BestMove", content: BestMove }
    | { type: "Error", content: unknown }

export type MintakaProviderRuntimeCommand =
    { type: "abort" }

export interface MintakaProvider {
    readonly type: MintakaProviderType,
    subscribeResponse(handler: (response: MintakaProviderResponse) => void): void,
    dispose(): void,
    command(command: Command): Promise<CommandResult>,
    launch(positionHash: HashKey, objective: SearchObjective): Promise<void>,
    control(command: MintakaProviderRuntimeCommand): void,
}
