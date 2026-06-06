import type { Command, CommandResult, Config, GameState, HashKey, SearchObjective, Timer } from "../wasm/pkg/rusty_renju_wasm"
import type { MintakaLaunchResponse, MintakaProvider, MintakaProviderResponse, MintakaProviderRuntimeCommand } from "./mintaka.provider"
import { duration, InfiniteDuration } from "./mintaka"
import { Mutex } from "../utils/mutex"

export type MintakaWorkerMessage =
    | { type: "config", config: Config }
    | { type: "command", id: number, command: Command }
    | { type: "launch", id: number, expectedHash: HashKey, timer: Timer, objective: SearchObjective }
    | { type: "init", config: Config, state: GameState }

export type MintakaWorkerResponse =
    | MintakaProviderResponse
    | { type: "CommandResult", id: number, content: CommandResult }
    | { type: "LaunchResult", id: number, content: MintakaLaunchResponse }
    | { type: "Load" }
    | { type: "Ready", version: string, sab: SharedArrayBuffer, counterPtr: number, abortPtr: number }

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
    max_nodes_in_1k: undefined,
    max_depth: undefined,
    max_vcf_depth: undefined,
    tt_size: 1024 * 1024 * 128,
    workers: Math.max(1, navigator.hardwareConcurrency - 1),
    pondering: false,
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
    readonly version = "0.0.0"

    private static readonly NodesPollingIntervalMs = 100

    private readonly worker: Worker
    private workerControl?: MintakaWorkerControl
    private nodesPollingInterval?: ReturnType<typeof setInterval>

    private readonly workerMutex = new Mutex()
    private commandId = 0
    private pendingRequest?: PendingRequest

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
                    this.resolvePending(event.data)
                    return
                }
                case "LaunchResult": {
                    this.resolvePending(event.data)
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
                    this.rejectPending(event.data.content)
                    break
                }
            }

            this.onResponse && this.onResponse(event.data)
        }

        this.worker.onerror = (event) => {
            const error = event.error ?? event.message
            this.stopNodesPolling()
            this.rejectPending(error)
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
        this.pendingRequest = undefined
        this.worker.terminate()
    }

    async config(config: Config) {
        return // TODO
    }

    async command(command: Command) {
        return await this.workerMutex.run(async () => {
            const id = this.nextId()

            return await this.postMessageForResponse("CommandResult", id, { type: "command", id, command })
        })
    }

    async launch(expectedHash: HashKey, timer: Timer, objective: SearchObjective): Promise<MintakaLaunchResponse> {
        return await this.workerMutex.run(async () => {
            const id = this.nextId()

            return await this.postMessageForResponse("LaunchResult", id, { type: "launch", timer, expectedHash, objective, id })
        })
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

    private postMessageForResponse(responseType: "CommandResult", id: number, message: MintakaWorkerMessage): Promise<CommandResult>
    private postMessageForResponse(responseType: "LaunchResult", id: number, message: MintakaWorkerMessage): Promise<MintakaLaunchResponse>
    private postMessageForResponse(responseType: WorkerResultResponseType, id: number, message: MintakaWorkerMessage) {
        return new Promise<CommandResult | MintakaLaunchResponse>((resolve, reject) => {
            this.pendingRequest = { id, responseType, resolve, reject }

            try {
                this.postMessage(message)
            } catch (error: unknown) {
                if (this.pendingRequest?.id === id)
                    this.pendingRequest = undefined

                reject(error)
            }
        })
    }

    private resolvePending(response: WorkerResultResponse) {
        const request = this.pendingRequest
        if (request === undefined)
            return

        if (request.id !== response.id || request.responseType !== response.type)
            return

        this.pendingRequest = undefined
        request.resolve(response.content)
    }

    private rejectPending(error: unknown) {
        const request = this.pendingRequest
        if (request === undefined)
            return

        this.pendingRequest = undefined
        request.reject(error)
    }

}

type WorkerResultResponse =
    | { type: "CommandResult", id: number, content: CommandResult }
    | { type: "LaunchResult", id: number, content: MintakaLaunchResponse }

type WorkerResultResponseType = WorkerResultResponse["type"]

type PendingRequest = {
    readonly id: number,
    readonly responseType: WorkerResultResponseType,
    readonly resolve: (value: CommandResult | MintakaLaunchResponse) => void,
    readonly reject: (reason?: unknown) => void,
}
