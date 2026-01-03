import {
    MintakaProvider,
    MintakaProviderResponse,
    MintakaProviderRuntimeMessage,
    MintakaProviderState,
    MintakaProviderType,
} from "./mintaka.provider"
import { Command, Config, emptyHash, GameState, HashKey, SearchObjective } from "../wasm/pkg/mintaka_wasm"

export class MintakaServerConfig {
    readonly address: string
    readonly port: number
    readonly apiPassword?: string

    constructor(address: string, port: number, apiPassword?: string) {
        this.address = address
        this.port = port
        this.apiPassword = apiPassword
    }

    get url() {
        return `${this.address}:${this.port}`
    }

    headers = (extra?: HeadersInit) => {
        const headers = new Headers(extra)

        if (this.apiPassword) headers.set("Api-Password", this.apiPassword)

        return headers
    }
}

export const LocalHostServerConfig = new MintakaServerConfig("http://localhost", 8080, "test")

export type MintakaServerSession = {
    readonly sid: string
}

async function assertOk(response: Response) {
    if (response.ok) {
        return
    }

    const message = await response.text()
    throw new Error(message || `request failed: ${response.status}`)
}

export async function checkHealth(serverConfig: MintakaServerConfig): Promise<boolean> {
    try {
        const response = await fetch(serverConfig.url + "/status", {
            headers: serverConfig.headers(),
        })

        return response.ok
    } catch {
        return false
    }
}

export async function createSession(serverConfig: MintakaServerConfig, config: Config, state: GameState): Promise<MintakaServerSession> {
    const response = await fetch(serverConfig.url + "/sessions", {
        method: "POST",
        headers: serverConfig.headers({
            "Content-Type": "application/json",
        }),
        body: JSON.stringify({
            config: config,
            state: state,
        }),
    })

    await assertOk(response)

    return { sid: await response.json() as string }
}

export class MintakaServerProvider implements MintakaProvider {
    private readonly serverConfig: MintakaServerConfig
    private readonly session: MintakaServerSession
    private chain: Promise<void> = Promise.resolve()

    onResponse?: (message: MintakaProviderResponse) => void
    onError?: (error: any) => void

    readonly type: MintakaProviderType = "server"
    snapshot: HashKey = emptyHash()

    state: MintakaProviderState

    constructor(config: MintakaServerConfig, session: MintakaServerSession) {
        this.serverConfig = config
        this.session = session
        this.state = {
            type: "idle",
            command: this.command,
            launch: () => void this.launch,
        }
    }

    private command = (command: Command) => {
        this.chain = this.chain
            .then(async () => {
                await this.sendCommand(command)
            })
    }

    private runtimeMessage = (_: MintakaProviderRuntimeMessage) => {
        void this.sendAbort()
    }

    private sendCommand = async (command: Command) => {
        const response = await fetch(this.serverConfig.url + `/sessions/${this.session.sid}/commands`, {
            method: "POST",
            headers: this.serverConfig.headers({
                "Content-Type": "application/json",
            }),
            body: JSON.stringify(command),
        })

        await assertOk(response)
    }

    private launch = async (hash: HashKey, objective: SearchObjective) => {
        const response = await fetch(this.serverConfig.url + `/sessions/${this.session.sid}/launch`, {
            method: "POST",
            headers: this.serverConfig.headers(),
            body: JSON.stringify({ hash, objective }),
        })

        await assertOk(response)

        this.state = {
            type: "in_computing",
            message: this.runtimeMessage,
        }

        void this.startStream()
    }

    private sendAbort = async () => {
        const response = await fetch(this.serverConfig.url + `/sessions/${this.session.sid}/abort`, {
            method: "POST",
            headers: this.serverConfig.headers(),
        })

        await assertOk(response)
    }

    private startStream = () => {
        const streamUrl = new URL(this.serverConfig.url + `/sessions/${this.session.sid}/stream`)

        if (this.serverConfig.apiPassword) {
            streamUrl.searchParams.set("api_password", this.serverConfig.apiPassword)
        }

        const eventSource = new EventSource(streamUrl.toString())

        eventSource.addEventListener("Response", (event) => {
            this.onResponse && this.onResponse(JSON.parse(event.data))
        })

        eventSource.addEventListener("BestMove", (event) => {
            eventSource.close()
            this.onResponse && this.onResponse({ type: "BestMove", content: JSON.parse(event.data) })

            this.state = {
                type: "idle",
                command: this.command,
                launch: () => void this.launch,
            }
        })

        eventSource.onerror = (error) => {
            eventSource.close()
            this.onError && this.onError(error)
        }
    }
}
