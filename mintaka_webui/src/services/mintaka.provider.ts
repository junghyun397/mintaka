import {BestMove, Command, SearchObjective} from "../wasm/pkg";

export type MintakaProviderMessage =
    | { type: "command", payload: Command }
    | { type: "launch", payload: { objective: SearchObjective }}

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
    onResponse?: (message: MintakaProviderResponse) => void
    onError?: (error: any) => void
    state: MintakaProviderState
    idleState?: MintakaProviderIdleState
    inComputingState?: MintakaProviderInComputingState
}

export abstract class MintakaProviderBase implements MintakaProvider {
    abstract state: MintakaProviderState

    get idleState(): MintakaProviderIdleState | undefined {
        return this.state.type == "idle" ? this.state : undefined
    }

    get inComputingState(): MintakaProviderInComputingState | undefined {
        return this.state.type == "in_computing" ? this.state : undefined
    }

}
