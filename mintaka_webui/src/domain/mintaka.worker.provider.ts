import { Config, GameState } from "../wasm/pkg/mintaka_wasm_worker"
import { Command, CommandResult, defaultConfig, HashKey, SearchObjective } from "../wasm/pkg/mintaka_wasm"
import { InfiniteDuration } from "./rusty-renju"
import { MintakaProvider, MintakaProviderResponse, MintakaProviderRuntimeCommand, MintakaProviderType } from "./mintaka.provider"

export type MintakaWorkerMessage =
    | { type: "command", command: Command }
    | { type: "launch", positionHash: HashKey, objective: SearchObjective }
    | { type: "init", config: Config, state: GameState }

export type MintakaWorkerResponse =
    | MintakaProviderResponse
    | { type: "Load" }
    | { type: "CommandResult", content: CommandResult }
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

    private readonly worker: Worker
    private workerControl?: MintakaWorkerControl

    private onResponse?: (message: MintakaProviderResponse) => void
    private onError?: (error: any) => void

    constructor(config: Config, gameState: GameState) {
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
            }

            this.onResponse && this.onResponse(event.data)
        }

        this.worker.onerror = (event) => {
            this.onError && this.onError(event)
        }

        this.postMessage({ type: "init", config: config, state: gameState })
    }

    subscribeResponse(handler: (response: MintakaProviderResponse) => void) {
        this.onResponse = handler
    }

    subscribeError(handler: (error: any) => void) {
        this.onError = handler
    }

    dispose() {
        this.onResponse = undefined
        this.onError = undefined
        this.worker.terminate()
    }

    command(command: Command) {
        this.postMessage({ type: "command", command: command })
    }

    launch(positionHash: HashKey, objective: SearchObjective) {
        this.postMessage({ type: "launch", positionHash, objective })
    }

    control(command: MintakaProviderRuntimeCommand) {
        switch (command.type) {
            case "abort": {
                this.workerControl?.abort()
                break
            }
        }
    }

    private postMessage(message: MintakaWorkerMessage) {
        this.worker.postMessage(message)
    }
}
