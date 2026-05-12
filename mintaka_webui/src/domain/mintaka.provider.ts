import type { BestMove, Command, CommandResult, HashKey, Response, SearchObjective } from "../wasm/pkg/rusty_renju_wasm"

export type MintakaProviderType = "server" | "worker"

export type MintakaProviderResponse =
    | Response
    | { type: "Nodes", content: number }
    | { type: "BestMove", content: BestMove }
    | { type: "Error", content: unknown }

export type MintakaProviderRuntimeCommand =
    { type: "abort" }

export type MintakaLaunchResponse = "launched" | "snapshot-mismatch"

export interface MintakaProvider {
    readonly type: MintakaProviderType,
    readonly version: string,
    subscribeResponse(handler: (response: MintakaProviderResponse) => void): void,
    command(command: Command): Promise<CommandResult>,
    launch(expectedHash: HashKey, objective: SearchObjective): Promise<MintakaLaunchResponse>,
    control(command: MintakaProviderRuntimeCommand): void,
    dispose(): void,
}
