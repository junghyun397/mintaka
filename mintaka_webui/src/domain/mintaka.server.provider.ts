import {
    MintakaLaunchResponse,
    MintakaProvider,
    MintakaProviderResponse,
    MintakaProviderRuntimeCommand,
} from "./mintaka.provider"
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
    readonly hash: HashKey,
    readonly version?: string,
}

function bitfieldToArray(bitfield: unknown): number[] {
    if (ArrayBuffer.isView(bitfield))
        return Array.from(new Uint8Array(bitfield.buffer, bitfield.byteOffset, bitfield.byteLength))
    else if (bitfield instanceof ArrayBuffer)
        return Array.from(new Uint8Array(bitfield))
    else if (Array.isArray(bitfield))
        return bitfield.map((value) => Number(value))
    else if (bitfield !== null && typeof bitfield === "object")
        return Object.values(bitfield).map((value) => Number(value))
    throw new Error("unsupported bitfield format")
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

    return parseSession(text, state)
}

export class MintakaServerProvider implements MintakaProvider {
    private stream?: MintakaServerStream

    private currentHash: HashKey

    private onResponse?: (message: MintakaProviderResponse) => void

    readonly type: "server" = "server"

    get version() {
        return this.session.version ?? "server"
    }

    constructor(readonly serverConfig: MintakaServerConfig, readonly session: MintakaServerSession) {
        this.currentHash = session.hash
    }

    async configs(): Promise<Configs> {
        const response = await fetch(`${serverUrl(this.serverConfig)}/sessions/${this.session.sid}/configs`, {
            headers: serverHeaders(this.serverConfig),
        })

        await assertResponseOk(response, "Failed to load session configs")

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

    async launch(expectedHash: HashKey, objective: SearchObjective): Promise<MintakaLaunchResponse> {
        if (this.currentHash !== expectedHash)
            return "snapshot-mismatch"

        await this.startStream()

        return await this.sendLaunch(expectedHash, objective)
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

        await assertResponseOk(response, "Failed to send command")

        const result = await response.json() as CommandResult
        this.currentHash = result.hash_key

        return result
    }

    private async sendLaunch(positionHash: HashKey, objective: SearchObjective) {
        const response = await fetch(`${serverUrl(this.serverConfig)}/sessions/${this.session.sid}/launch`, {
            method: "POST",
            headers: serverHeaders(this.serverConfig, {
                "Content-Type": "application/json",
            }),
            body: JSON.stringify({
                position_hash: positionHash,
                objective,
            }),
        })

        if (!response.ok) {
            const text = await response.text()
            if (text === "HASH_MISMATCH")
                return "snapshot-mismatch"

            throw new Error(text || `Failed to launch ${objective} search: ${response.status}`)
        }

        return "launched"
    }

    private sendAbort = async () => {
        const _ = await fetch(`${serverUrl(this.serverConfig)}/sessions/${this.session.sid}/abort`, {
            method: "POST",
            headers: serverHeaders(this.serverConfig),
        })
    }

    private closeStream() {
        if (this.stream) {
            this.stream.abortController.abort()
            this.stream = undefined
        }
    }

    private startStream = () => {
        if (this.stream)
            return this.stream.ready

        const abortController = new AbortController()
        const stream: MintakaServerStream = {
            abortController,
            ready: Promise.resolve(),
        }

        this.stream = stream

        stream.ready = (async () => {
            const response = await fetch(`${serverUrl(this.serverConfig)}/sessions/${this.session.sid}/stream`, {
                headers: serverHeaders(this.serverConfig),
                signal: abortController.signal,
            })

            await assertResponseOk(response, "Failed to subscribe session stream")

            if (response.body === null)
                throw new Error("session stream response has no body")

            void this.consumeStream(stream, response.body)
        })()

        stream.ready.catch(error => this.handleStreamError(stream, error))

        return stream.ready
    }

    private async consumeStream(stream: MintakaServerStream, body: ReadableStream<Uint8Array>) {
        const reader = body.getReader()
        const decoder = new TextDecoder()
        let buffer = ""

        try {
            while (true) {
                const { value, done } = await reader.read()
                if (done) break

                buffer += decoder.decode(value, { stream: true })
                buffer = this.consumeStreamBuffer(buffer)
            }

            buffer += decoder.decode()
            this.consumeStreamBuffer(buffer)
        } catch (error: unknown) {
            this.handleStreamError(stream, error)
        } finally {
            reader.releaseLock()
            if (this.stream === stream)
                this.stream = undefined
        }
    }

    private consumeStreamBuffer(buffer: string) {
        const chunks = buffer.split(/\r?\n\r?\n/)
        const rest = chunks.pop() ?? ""

        chunks
            .map(parseServerSentEvent)
            .forEach(event => event !== undefined && this.handleStreamEvent(event))

        return rest
    }

    private handleStreamEvent(event: ServerSentEvent) {
        switch (event.event) {
            case "Response": {
                this.onResponse && this.onResponse(JSON.parse(event.data))
                break
            }
            case "BestMove": {
                this.onResponse && this.onResponse({ type: "BestMove", content: JSON.parse(event.data) })
                break
            }
        }
    }

    private handleStreamError(stream: MintakaServerStream, error: unknown) {
        if (stream.abortController.signal.aborted)
            return

        if (this.stream === stream)
            this.stream = undefined

        this.onError(error)
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

type MintakaServerStream = {
    readonly abortController: AbortController,
    ready: Promise<void>,
}

type RawMintakaServerSession = string | {
    readonly sid: string,
    readonly hash?: HashKey,
    readonly version?: string,
}

function parseSession(text: string, state: GameState): MintakaServerSession {
    const raw = JSON.parse(text) as RawMintakaServerSession

    if (typeof raw === "string")
        return { sid: raw, hash: state.board.hash_key }

    return {
        sid: raw.sid,
        hash: raw.hash ?? state.board.hash_key,
        version: raw.version,
    }
}

type ServerSentEvent = {
    readonly event: string,
    readonly data: string,
}

function parseServerSentEvent(chunk: string): ServerSentEvent | undefined {
    let event = "message"
    const data: string[] = []

    chunk.split(/\r?\n/).forEach(line => {
        if (line.startsWith(":"))
            return

        const delimiter = line.indexOf(":")
        const field = delimiter === -1 ? line : line.slice(0, delimiter)
        const value = delimiter === -1 ? "" : line.slice(delimiter + 1).replace(/^ /, "")

        switch (field) {
            case "event": {
                event = value
                break
            }
            case "data": {
                data.push(value)
                break
            }
        }
    })

    if (data.length === 0)
        return undefined

    return { event, data: data.join("\n") }
}

async function assertResponseOk(response: Response, message: string) {
    if (response.ok)
        return

    const text = await response.text()
    throw new Error(text || `${message}: ${response.status}`)
}
