import {BestMove, Command, Config, GameState} from "../wasm/pkg";

export type MintakaWorkerMessage =
    | { type: "init", payload: { config: Config, state: GameState } }
    | { type: "command", payload: Command }
    | { type: "launch", payload: { }}

export type MintakaWorkerResponse =
    | { type: "Ready", content: MintakaWorkerControl }
    | { type: "BestMove", content: BestMove }
    | { type: "Error", error: any }

export type MintakaWorkerControl = {
    sab: SharedArrayBuffer,
    control_ptr: number,
}

export function abortWorker(control: MintakaWorkerControl) {
    const mem = new Uint8Array(control.sab)
    Atomics.store(mem, control.control_ptr, 1)
}
