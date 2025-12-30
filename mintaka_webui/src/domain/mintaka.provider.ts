import { BestMove, Command, Response, SearchObjective } from "../wasm/pkg/mintaka_wasm";

export type MintakaProviderType = "server" | "worker"

export type MintakaProviderMessage =
    | { type: "command", payload: Command }
    | { type: "launch", payload: { objective: SearchObjective } }

export type MintakaProviderRuntimeMessage =
    { type: "abort" }

export type MintakaProviderIdleState = {
    type: "idle",
    message: (message: MintakaProviderMessage) => void,
}

export type MintakaProviderInComputingState = {
    type: "in_computing",
    message: (message: MintakaProviderRuntimeMessage) => void
}

export type MintakaProviderResponse =
    | Response
    | { type: "BestMove", content: BestMove }

export type MintakaProviderState = MintakaProviderIdleState | MintakaProviderInComputingState

export interface MintakaProvider {
    readonly type: string
    onResponse?: (message: MintakaProviderResponse) => void
    onError?: (error: any) => void
    state: MintakaProviderState
}
