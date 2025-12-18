import {BestMove, Command, Config, GameState} from "../wasm/pkg";

export type MintakaWorkerMessage =
    | { type: "init", payload: { config: Config, state: GameState } }
    | { type: "command", payload: Command }
    | { type: "launch", payload: { }}

export type MintakaWorkerResponse =
    | { type: "Ready" }
    | { type: "BestMove", content: BestMove }
    | { type: "Error", error: any }
