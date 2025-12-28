import {
    MintakaProviderBase,
    MintakaProviderMessage,
    MintakaProviderResponse,
    MintakaProviderRuntimeMessage,
    MintakaProviderState
} from "./mintaka.provider";
import {Config, GameState} from "../wasm/pkg/mintaka_wasm_worker";

export type MintakaWorkerMessage =
    | MintakaProviderMessage
    | { type: "init", payload: { config: Config, state: GameState } }

export type MintakaLoadingResponse =
    | { step: "download", size: number }
    | { step: "downloading", size: number, loaded: number }
    | { step: "compile" }

export type MintakaWorkerResponse =
    | MintakaProviderResponse
    | { type: "Load", content: MintakaLoadingResponse }
    | { type: "Ready", control: MintakaWorkerControl }

export class MintakaWorkerControl {
    readonly sab: SharedArrayBuffer
    readonly control_ptr: number

    constructor(sab: SharedArrayBuffer, control_ptr: number) {
        this.sab = sab
        this.control_ptr = control_ptr
    }

    abort() {
        const mem = new Uint8Array(this.sab)
        Atomics.store(mem, this.control_ptr, 1)
    }
}

export class MintakaWorkerProvider extends MintakaProviderBase {
    private readonly worker: Worker
    private workerControl?: MintakaWorkerControl

    onResponse?: (message: MintakaProviderResponse) => void
    onError?: (error: any) => void

    state: MintakaProviderState

    constructor(config: Config, gameState: GameState) {
        super()

        this.state = {
            type: "idle",
            message: this.idleMessage,
        }

        this.worker = new Worker(
            new URL("mintaka.worker.ts", import.meta.url),
            {type: "module"},
        )

        this.worker.onmessage = (event: MessageEvent<MintakaWorkerResponse>) => {
            switch (event.data.type) {
                case "Load": {
                    console.log(event.data.content)
                    return
                }
                case "Ready": {
                    this.workerControl = event.data.control
                    return
                }
                case "BestMove": {
                    this.state = {
                        type: "idle",
                        message: this.idleMessage,
                    }
                }
            }

            this.onResponse && this.onResponse(event.data)
        }

        this.worker.onerror = (event) => {
            this.onError && this.onError(event)
        }

        this.worker.postMessage({type: "init", payload: { config: config, state: gameState }})
    }

    private idleMessage = (message: MintakaProviderMessage) => {
        if (message.type == "launch") {
            this.state = {
                type: "in_computing",
                message: this.runtimeMessage,
            }
        }

        this.worker.postMessage(message)
    }

    private runtimeMessage = (_: MintakaProviderRuntimeMessage) => {
        this.workerControl?.abort()
    }

}
