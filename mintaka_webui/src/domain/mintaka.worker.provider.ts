import {
    MintakaProvider, MintakaProviderLaunchError,
    MintakaProviderResponse,
    MintakaProviderRuntimeMessage,
    MintakaProviderState, MintakaProviderType,
} from "./mintaka.provider";
import { Config, GameState } from "../wasm/pkg/mintaka_wasm_worker";
import { Command, emptyHash, HashKey, SearchObjective } from "../wasm/pkg/mintaka_wasm";

export type MintakaWorkerMessage =
    | { type: "command", payload: Command }
    | { type: "launch", payload: { hash: HashKey, objective: SearchObjective } }
    | { type: "init", payload: { config: Config, state: GameState } }

export type MintakaLoadingResponse =
    | { step: "download", size: number }
    | { step: "downloading", size: number, loaded: number }
    | { step: "compile" }

export type MintakaWorkerResponse =
    | MintakaProviderResponse
    | { type: "Load", content: MintakaLoadingResponse }
    | { type: "Ready", sab: SharedArrayBuffer, controlPtr: number }

export class MintakaWorkerControl {
    readonly sab: SharedArrayBuffer
    readonly controlPtr: number

    constructor(sab: SharedArrayBuffer, control_ptr: number) {
        this.sab = sab
        this.controlPtr = control_ptr
    }

    abort() {
        const mem = new Uint8Array(this.sab)
        Atomics.store(mem, this.controlPtr, 1)
    }
}

export class MintakaWorkerProvider implements MintakaProvider {
    readonly type: MintakaProviderType = "worker"
    snapshot: HashKey = emptyHash()

    private readonly worker: Worker
    private workerControl?: MintakaWorkerControl

    onResponse?: (message: MintakaProviderResponse) => void
    onError?: (error: any) => void

    state: MintakaProviderState

    constructor(config: Config, gameState: GameState) {
        this.state = {
            type: "idle",
            command: this.command,
            launch: this.launch,
        }

        this.worker = new Worker(
            new URL("mintaka.worker.ts", import.meta.url),
            { type: "module" },
        )

        this.worker.onmessage = (event: MessageEvent<MintakaWorkerResponse>) => {
            switch (event.data.type) {
                case "Load": {
                    console.log(event.data.content)
                    return
                }
                case "Ready": {
                    this.workerControl = new MintakaWorkerControl(event.data.sab, event.data.controlPtr)
                    return
                }
                case "CommandResult": {
                    this.snapshot = event.data.content.hash_key
                    break
                }
                case "BestMove": {
                    this.state = {
                        type: "idle",
                        command: this.command,
                        launch: this.launch,
                    }
                    break
                }
            }

            this.onResponse && this.onResponse(event.data)
        }

        this.worker.onerror = (event) => {
            this.onError && this.onError(event)
        }

        this.worker.postMessage({ type: "init", payload: { config: config, state: gameState } })
    }

    private command = (command: Command) => {
        this.worker.postMessage({ type: "command", payload: command } as MintakaWorkerMessage)
    }

    private launch = (hash: HashKey, objective: SearchObjective): MintakaProviderLaunchError | undefined => {
        if (this.snapshot !== hash) return "snapshot-mismatch"

        this.state = {
            type: "in_computing",
            message: this.runtimeMessage,
        }

        this.worker.postMessage({ type: "launch", payload: { hash, objective } })

        return undefined
    }

    private runtimeMessage = (_: MintakaProviderRuntimeMessage) => {
        this.workerControl?.abort()
    }

}
