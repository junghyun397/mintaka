import { MintakaLaunchResponse, MintakaProvider, MintakaProviderResponse, MintakaProviderRuntimeCommand } from "./mintaka.provider"
import type {
    BestMove, Command, CommandResult, Config,
    CreateSessionRequest, CreateSessionResponse, GameState, HashKey,
    Health, LaunchSessionRequest, Response as MintakaResponse, SearchObjective,
} from "../wasm/pkg/rusty_renju_wasm"
import { SERVER_PROTOCOL } from "../config"
import type { Configs } from "./mintaka"

export type MintakaServerConfig = {
    readonly address: string,
    readonly apiPassword?: string,
}

function serverUrl(config: MintakaServerConfig) {
    return `${SERVER_PROTOCOL}://${config.address}`
}

const SESSION_TOKEN_HEADER_NAME = "mintaka-session-token"
const SESSION_TOKEN_QUERY_NAME = "token"

export type MintakaServerSession = CreateSessionResponse

function sessionHeaders(session: MintakaServerSession, extra?: HeadersInit) {
    const headers = new Headers(extra)
    headers.set(SESSION_TOKEN_HEADER_NAME, session.token)
    return headers
}

export async function checkHealth(serverConfig: MintakaServerConfig): Promise<boolean> {
    const response = await fetch(`${serverUrl(serverConfig)}/status`)

    if (!response.ok)
        return false

    const health = await response.json() as Health

    return Number.isFinite(health.available_workers)
}

export async function createSession(serverConfig: MintakaServerConfig, state: GameState, config: Config | undefined): Promise<MintakaServerSession> {
    const payload: CreateSessionRequest = {
        api_password: serverConfig.apiPassword,
        config: config,
        state,
    }

    const response = await fetch(`${serverUrl(serverConfig)}/sessions`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(payload),
    })

    await assertResponseOk(response, "Failed to create session")

    return await response.json() as CreateSessionResponse
}

export class MintakaServerProvider implements MintakaProvider {
    private stream?: EventSource

    private currentHash: HashKey

    private onResponse?: (message: MintakaProviderResponse) => void

    readonly type: "server" = "server"

    get version() {
        return this.session.version
    }

    constructor(readonly serverConfig: MintakaServerConfig, readonly session: MintakaServerSession) {
        this.currentHash = session.hash
    }

    async configs(): Promise<Configs> {
        const response = await fetch(`${serverUrl(this.serverConfig)}/sessions/${this.session.sid}/configs`, {
            headers: sessionHeaders(this.session),
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

        this.startStream()

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
            headers: sessionHeaders(this.session, {
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
        const payload: LaunchSessionRequest = {
            position_hash: positionHash,
        }

        const response = await fetch(`${serverUrl(this.serverConfig)}/sessions/${this.session.sid}/launch`, {
            method: "POST",
            headers: sessionHeaders(this.session, {
                "Content-Type": "application/json",
            }),
            body: JSON.stringify(payload),
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
        void fetch(`${serverUrl(this.serverConfig)}/sessions/${this.session.sid}/abort`, {
            method: "POST",
            headers: sessionHeaders(this.session),
        })
    }

    private closeStream() {
        if (this.stream) {
            this.stream.close()
            this.stream = undefined
        }
    }

    private startStream() {
        this.closeStream()

        const streamUrl = new URL(`${serverUrl(this.serverConfig)}/sessions/${this.session.sid}/stream`)
        streamUrl.searchParams.set(SESSION_TOKEN_QUERY_NAME, this.session.token)

        const eventSource = new EventSource(streamUrl.toString())
        this.stream = eventSource

        eventSource.addEventListener("Response", (event) => {
            this.onResponse && this.onResponse(JSON.parse(event.data) as MintakaResponse)
        })

        eventSource.addEventListener("BestMove", (event) => {
            eventSource.close()
            this.onResponse && this.onResponse({ type: "BestMove", content: JSON.parse(event.data) as BestMove })
            this.stream = undefined
        })

        eventSource.onerror = (error) => {
            eventSource.close()
            this.stream = undefined
            this.onError(error)
        }
    }

    private async disconnect() {
        void fetch(`${serverUrl(this.serverConfig)}/sessions/${this.session.sid}`, {
            method: "DELETE",
            headers: sessionHeaders(this.session),
        })
    }

    private onError = (error: unknown) => {
        this.onResponse && this.onResponse({ type: "Error", content: error })
    }
}

async function assertResponseOk(response: Response, message: string) {
    if (response.ok)
        return

    const text = await response.text()
    throw new Error(text || `${message}: ${response.status}`)
}
