import type { Command, CommandResult, Config, GameState, HashKey, SearchObjective } from "../wasm/pkg/rusty_renju_wasm"
import type { MintakaLaunchResponse, MintakaProvider, MintakaProviderResponse, MintakaProviderRuntimeCommand } from "./mintaka.provider"
import { duration, InfiniteDuration } from "./mintaka"

export type MintakaWorkerMessage =
    | { type: "command", id: number, command: Command }
    | { type: "launch", id: number, expectedHash: HashKey, objective: SearchObjective }
    | { type: "init", config: Config, state: GameState }

export type MintakaWorkerResponse =
    | MintakaProviderResponse
    | { type: "CommandResult", id: number, content: CommandResult }
    | { type: "LaunchResult", id: number, content: MintakaLaunchResponse }
    | { type: "Load" }
    | { type: "Ready", sab: SharedArrayBuffer, counterPtr: number, abortPtr: number }

export class MintakaWorkerControl {
    readonly sab: SharedArrayBuffer
    readonly counterPtr: number
    readonly abortPtr: number

    constructor(sab: SharedArrayBuffer, counterPtr: number, abortPtr: number) {
        this.sab = sab
        this.counterPtr = counterPtr
        this.abortPtr = abortPtr
    }

    get global_nodes_in_1k() {
        const mem = new Uint32Array(this.sab)
        return Atomics.load(mem, this.counterPtr / Uint32Array.BYTES_PER_ELEMENT)
    }

    abort() {
        const mem = new Uint8Array(this.sab)
        Atomics.store(mem, this.abortPtr, 1)
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
    readonly type: "worker" = "worker"

    private static readonly NodesPollingIntervalMs = 100

    private readonly worker: Worker
    private workerControl?: MintakaWorkerControl
    private nodesPollingInterval?: ReturnType<typeof setInterval>

    private commandId = 0
    private readonly pendingCommands = new Map<number, PendingRequest<CommandResult>>()
    private readonly pendingLaunches = new Map<number, PendingRequest<MintakaLaunchResponse>>()

    private onResponse?: (message: MintakaProviderResponse) => void

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
                    this.workerControl = new MintakaWorkerControl(event.data.sab, event.data.counterPtr, event.data.abortPtr)
                    return
                }
                case "CommandResult": {
                    this.resolvePending(this.pendingCommands, event.data.id, event.data.content)
                    return
                }
                case "LaunchResult": {
                    this.resolvePending(this.pendingLaunches, event.data.id, event.data.content)
                    return
                }
                case "Begins": {
                    this.startNodesPolling()
                    break
                }
                case "BestMove": {
                    this.stopNodesPolling()
                    break
                }
                case "Error": {
                    this.rejectAllPending(event.data.content)
                    break
                }
            }

            this.onResponse && this.onResponse(event.data)
        }

        this.worker.onerror = (event) => {
            const error = event.error ?? event.message
            this.stopNodesPolling()
            this.rejectAllPending(error)
            this.onResponse && this.onResponse({ type: "Error", content: error })
        }

        this.postMessage({ type: "init", config, state })
    }

    subscribeResponse(handler: (response: MintakaProviderResponse) => void) {
        this.onResponse = handler
    }

    dispose() {
        this.onResponse = undefined
        this.stopNodesPolling()
        this.rejectAllPending(new Error("worker provider disposed"))
        this.worker.terminate()
    }

    async command(command: Command) {
        const id = this.nextId()

        return this.postMessageForResponse(this.pendingCommands, id, { type: "command", id, command })
    }

    async launch(expectedHash: HashKey, objective: SearchObjective): Promise<MintakaLaunchResponse> {
        const id = this.nextId()

        return this.postMessageForResponse(this.pendingLaunches, id, { type: "launch", expectedHash, objective, id })
    }

    control(command: MintakaProviderRuntimeCommand) {
        switch (command.type) {
            case "abort": {
                this.workerControl?.abort()
                break
            }
        }
    }

    private nextId() {
        this.commandId += 1
        return this.commandId
    }

    private postMessage(message: MintakaWorkerMessage) {
        this.worker.postMessage(message)
    }

    private startNodesPolling() {
        this.stopNodesPolling()
        this.nodesPollingInterval = setInterval(() => {
            if (this.workerControl === undefined)
                return

            this.onResponse && this.onResponse({ type: "Nodes", content: this.workerControl.global_nodes_in_1k })
        }, MintakaWorkerProvider.NodesPollingIntervalMs)
    }

    private stopNodesPolling() {
        if (this.nodesPollingInterval === undefined)
            return

        clearInterval(this.nodesPollingInterval)
        this.nodesPollingInterval = undefined
    }

    private postMessageForResponse<T>(pending: Map<number, PendingRequest<T>>, id: number, message: MintakaWorkerMessage) {
        return new Promise<T>((resolve, reject) => {
            pending.set(id, { resolve, reject })

            try {
                this.postMessage(message)
            } catch (error: unknown) {
                pending.delete(id)
                reject(error)
            }
        })
    }

    private resolvePending<T>(pending: Map<number, PendingRequest<T>>, id: number, content: T) {
        const request = pending.get(id)
        if (request === undefined)
            return

        pending.delete(id)
        request.resolve(content)
    }

    private rejectAllPending(error: unknown) {
        this.pendingCommands.forEach(request => request.reject(error))
        this.pendingCommands.clear()

        this.pendingLaunches.forEach(request => request.reject(error))
        this.pendingLaunches.clear()
    }

}

type PendingRequest<T> = {
    readonly resolve: (value: T) => void,
    readonly reject: (reason?: unknown) => void,
}
