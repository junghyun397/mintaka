import { BestMove, Command, CommandResult, Config, HashKey, Response, SearchObjective } from "../wasm/pkg/mintaka_wasm"

export type MintakaProviderType = "server" | "worker"

export type MintakaProviderRuntimeMessage =
    { type: "abort" }

export type MintakaProviderLaunchResult = "ok" | "snapshot-mismatch"

export type MintakaProviderIdleState = {
    type: "idle",
    command: (command: Command) => void,
    launch: (hash: HashKey, objective: SearchObjective) => MintakaProviderLaunchResult
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
    readonly maxConfig: Config
    snapshot: HashKey
    state: MintakaProviderState
    onResponse?: (message: MintakaProviderResponse) => void
    onError?: (error: any) => void
}

export abstract class BaseMintakaProvider implements MintakaProvider {
    abstract readonly type: MintakaProviderType
    abstract maxConfig: Config
    abstract snapshot: HashKey
    abstract state: MintakaProviderState

    private chain: Promise<void> = Promise.resolve()
}
