export type MintakaWorkerMessage =
    | { type: "init", payload: { config: any, state: any } }
    | { type: "command", payload: any }
    | { type: "launch", payload: { }}

export type MintakaWorkerResponse =
    | { type: "response", payload: any }
    | { type: "best-move", payload: any }
    | { type: "error", payload: any }
