import {
    MintakaProvider, MintakaProviderLaunchResult,
    MintakaProviderResponse,
    MintakaProviderRuntimeMessage,
    MintakaProviderState, MintakaProviderType,
} from "./mintaka.provider"
import { Config, GameState } from "../wasm/pkg/mintaka_wasm_worker"
import { Command, defaultConfig, emptyHash, HashKey, SearchObjective } from "../wasm/pkg/mintaka_wasm"
import { InfiniteDuration } from "./rusty-renju"

export type MintakaWorkerMessage =
    | { type: "command", command: Command }
    | { type: "launch", hash: HashKey, objective: SearchObjective }
    | { type: "init", config: Config, state: GameState }

export type MintakaWorkerResponse =
    | MintakaProviderResponse
    | { type: "Load" }
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
    readonly maxConfig: Config = {
        ...defaultConfig(),
        max_vcf_depth: 225,
        workers: 256,
        tt_size: 1024 * 1024 * 2048, // 2 GiB
        pondering: true,
        initial_timer: {
            total_remaining: InfiniteDuration,
            turn: InfiniteDuration,
            increment: InfiniteDuration,
        },
    }

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

        this.snapshot = gameState.board.hash_key

        this.worker = new Worker(
            new URL("mintaka.worker.ts", import.meta.url),
            { type: "module" },
        )

        this.worker.onmessage = (event: MessageEvent<MintakaWorkerResponse>) => {
            switch (event.data.type) {
                case "Load": {
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

        this.worker.postMessage({ type: "init", config: config, state: gameState })
    }

    private command = (command: Command) => {
        this.worker.postMessage({ type: "command", command: command } as MintakaWorkerMessage)
    }

    private launch = (hash: HashKey, objective: SearchObjective): MintakaProviderLaunchResult => {
        if (this.snapshot !== hash) return "snapshot-mismatch"

        this.state = {
            type: "in_computing",
            message: this.runtimeMessage,
        }

        this.worker.postMessage({ type: "launch", hash, objective })

        return "ok"
    }

    private runtimeMessage = (_: MintakaProviderRuntimeMessage) => {
        this.workerControl?.abort()
    }

}
