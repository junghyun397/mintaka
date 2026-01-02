import { BestMove, Command, CommandResult, HashKey, Response, SearchObjective } from "../wasm/pkg/mintaka_wasm";

export type MintakaProviderType = "server" | "worker"

export type MintakaProviderRuntimeMessage =
    { type: "abort" }

export type MintakaProviderLaunchError = "snapshot-mismatch"

export type MintakaProviderIdleState = {
    type: "idle",
    command: (command: Command) => void,
    launch: (hash: HashKey, objective: SearchObjective) => MintakaProviderLaunchError | undefined
}

export type MintakaProviderInComputingState = {
    type: "in_computing",
    message: (message: MintakaProviderRuntimeMessage) => void,
}

export type MintakaProviderResponse =
    | Response
    | { type: "CommandResult", content: CommandResult }
    | { type: "BestMove", content: BestMove }

export type MintakaProviderState = MintakaProviderIdleState | MintakaProviderInComputingState

export interface MintakaProvider {
    readonly type: MintakaProviderType
    onResponse?: (message: MintakaProviderResponse) => void
    onError?: (error: any) => void
    snapshot: HashKey
    state: MintakaProviderState
}
