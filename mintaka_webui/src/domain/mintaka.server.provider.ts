import { MintakaProvider, MintakaProviderResponse, MintakaProviderRuntimeCommand } from "./mintaka.provider"
import type { Command, CommandResult, Config, GameState, HashKey, SearchObjective } from "../wasm/pkg/rusty_renju_wasm"
import { SERVER_PROTOCOL } from "../config"
import { Configs } from "./mintaka"

export type MintakaServerConfig = {
    readonly address: string,
    readonly apiPassword?: string,
}

function serverUrl(config: MintakaServerConfig) {
    return `${SERVER_PROTOCOL}://${config.address}`
}

function serverHeaders(config: MintakaServerConfig, extra?: HeadersInit) {
    const headers = new Headers(extra)
    if (config.apiPassword) headers.set("Api-Password", config.apiPassword)
    return headers
}

export type MintakaServerSession = {
    readonly sid: string,
}

function bitfieldToArray(bitfield: unknown): number[] {
    if (bitfield instanceof Uint8Array)
        return Array.from(bitfield)
    else if (bitfield instanceof ArrayBuffer)
        return Array.from(new Uint8Array(bitfield))
    else if (Array.isArray(bitfield))
        return bitfield.map((value) => Number(value))
    return []
}

function serializeGameState(state: GameState) {
    const board = state.board as GameState["board"] & { bitfield: unknown }
    const encodedBitfield = board.bitfield.map(bitfieldToArray)

    return {
        ...state,
        board: {
            ...board,
            bitfield: encodedBitfield,
        },
    }
}

export async function checkHealth(serverConfig: MintakaServerConfig): Promise<boolean> {
    const response = await fetch(`${serverUrl(serverConfig)}/status`, {
        headers: serverHeaders(serverConfig),
    })

    return response.ok
}

export async function createSession(serverConfig: MintakaServerConfig, state: GameState, config: Config | undefined): Promise<MintakaServerSession> {
    const payloadState = serializeGameState(state)
    const response = await fetch(`${serverUrl(serverConfig)}/sessions`, {
        method: "POST",
        headers: serverHeaders(serverConfig, {
            "Content-Type": "application/json",
        }),
        body: JSON.stringify({
            config: config,
            state: payloadState,
        }),
    })

    const text = await response.text()
    if (!response.ok) {
        throw new Error(text || `Failed to create session: ${response.status}`)
    }

    return { sid: JSON.parse(text) }
}

export class MintakaServerProvider implements MintakaProvider {
    private eventSource?: EventSource

    private onResponse?: (message: MintakaProviderResponse) => void

    readonly type: "server" = "server"

    constructor(readonly serverConfig: MintakaServerConfig, readonly session: MintakaServerSession) {}

    async configs(): Promise<Configs> {
        const response = await fetch(`${serverUrl(this.serverConfig)}/sessions/${this.session.sid}/configs`, {
            headers: serverHeaders(this.serverConfig),
        })

        return response.json()
    }

    subscribeResponse(handler: (response: MintakaProviderResponse) => void) {
        this.onResponse = handler
    }

    dispose() {
        this.onResponse = undefined
        this.closeStream()
        void this.disconnect()
    }

    async command(command: Command) {
        return await this.sendCommand(command)
    }

    async launch(positionHash: HashKey, objective: SearchObjective) {
        return await this.sendLaunch(positionHash, objective)
    }

    control(command: MintakaProviderRuntimeCommand) {
        switch (command.type) {
            case "abort": {
                void this.sendAbort()
                break
            }
        }
    }

    private async sendCommand(command: Command) {
        const response = await fetch(`${serverUrl(this.serverConfig)}/sessions/${this.session.sid}/commands`, {
            method: "POST",
            headers: serverHeaders(this.serverConfig, {
                "Content-Type": "application/json",
            }),
            body: JSON.stringify(command),
        })

        return await response.json() as CommandResult
    }

    private async sendLaunch(hash: HashKey, objective: SearchObjective) {
        const response = await fetch(`${serverUrl(this.serverConfig)}/sessions/${this.session.sid}/launch`, {
            method: "POST",
            headers: serverHeaders(this.serverConfig),
            body: JSON.stringify({ hash, objective }),
        })

        this.startStream()
    }

    private sendAbort = async () => {
        const _ = await fetch(`${serverUrl(this.serverConfig)}/sessions/${this.session.sid}/abort`, {
            method: "POST",
            headers: serverHeaders(this.serverConfig),
        })
    }

    private closeStream() {
        if (this.eventSource) {
            this.eventSource.close()
            this.eventSource = undefined
        }
    }

    private startStream = () => {
        this.closeStream()

        const streamUrl = new URL(`${serverUrl(this.serverConfig)}/sessions/${this.session.sid}/stream`)

        const eventSource = new EventSource(streamUrl.toString())
        this.eventSource = eventSource

        eventSource.addEventListener("Response", (event) => {
            this.onResponse && this.onResponse(JSON.parse(event.data))
        })

        eventSource.addEventListener("BestMove", (event) => {
            eventSource.close()
            this.onResponse && this.onResponse({ type: "BestMove", content: JSON.parse(event.data) })
            this.eventSource = undefined
        })

        eventSource.onerror = (error) => {
            eventSource.close()
            this.onResponse && this.onResponse({ type: "Error", content: error })
            this.eventSource = undefined
        }
    }

    private async disconnect() {
        const _ = await fetch(`${serverUrl(this.serverConfig)}/sessions/${this.session.sid}`, {
            method: "DELETE",
            headers: serverHeaders(this.serverConfig),
        })
    }

    private onError = (error: unknown) => {
        this.onResponse && this.onResponse({ type: "Error", content: error })
    }
}
