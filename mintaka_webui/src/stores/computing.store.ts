import {MaybePos} from "../wasm/pkg/mintaka_wasm";

export type ComputingState = {
    readonly bestMove: MaybePos
    readonly evaluation: number
}

export type ComputingStore = {
    readonly state?: ComputingState
}
