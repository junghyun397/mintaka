import type { Command, Config, GameState, HashKey, SearchObjective } from "../wasm/pkg/rusty_renju_wasm"
import type { MintakaProvider, MintakaProviderResponse, MintakaProviderRuntimeCommand, MintakaProviderType } from "./mintaka.provider"
import { duration, InfiniteDuration } from "./mintaka"

export type MintakaWorkerMessage =
    | { type: "command", command: Command }
    | { type: "launch", positionHash: HashKey, objective: SearchObjective }
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

export const DefaultWorkerConfig: Config = {
    rule_kind: "Renju",
    draw_condition: 225,
    max_nodes_in_1k: undefined,
    max_depth: undefined,
    max_vcf_depth: undefined,
    tt_size: 1024 * 1024 * 128,
    workers: Math.max(1, navigator.hardwareConcurrency - 1),
    pondering: false,
    dynamic_time: false,
    initial_timer: {
        total_remaining: undefined,
        increment: duration(0),
        turn: duration(5),
    },
    spawn_depth_specialist: false,
}

export const MaxWorkerConfig: Config = {
    ...DefaultWorkerConfig,
    max_vcf_depth: 225,
    workers: 256,
    tt_size: 1024 * 1024 * 2048, // 2 GiB
    pondering: true,
    initial_timer: {
        total_remaining: undefined,
        increment: InfiniteDuration,
        turn: undefined,
    },
    spawn_depth_specialist: true,
}

export class MintakaWorkerProvider implements MintakaProvider {
    readonly type: MintakaProviderType = "worker"

    private readonly worker: Worker
    private workerControl?: MintakaWorkerControl

    private onResponse?: (message: MintakaProviderResponse) => void
    private onError?: (error: any) => void

    constructor(state: GameState, config: Config) {
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

        this.postMessage({ type: "init", config, state })
    }

    subscribeResponse(handler: (response: MintakaProviderResponse) => void) {
        this.onResponse = handler
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
