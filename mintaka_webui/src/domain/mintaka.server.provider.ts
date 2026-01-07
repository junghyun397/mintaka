import { MintakaProvider, MintakaProviderResponse, MintakaProviderRuntimeCommand, MintakaProviderType } from "./mintaka.provider"
import { defaultConfig } from "../wasm/pkg/mintaka_wasm"
import type { Command, CommandResult, Config, GameState, HashKey, SearchObjective } from "../wasm/pkg/mintaka_wasm"

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

export async function createSession(serverConfig: MintakaServerConfig, config: Config, state: GameState) {
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
    private eventSource?: EventSource

    private onResponse?: (message: MintakaProviderResponse) => void
    private onError?: (error: any) => void

    readonly type: MintakaProviderType = "server"
    readonly maxConfig: Config

    constructor(config: MintakaServerConfig, session: MintakaServerSession, maxConfig?: Config) {
        this.serverConfig = config
        this.session = session
        this.maxConfig = maxConfig ?? defaultConfig()
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
        this.closeStream()
    }

    command(command: Command) {
        this.chain = this.chain
            .then(async () => {
                await this.sendCommand(command)
            })
            .catch((error) => {
                this.onError && this.onError(error)
            })
    }

    launch(positionHash: HashKey, objective: SearchObjective) {
        this.chain = this.chain
            .then(async () => {
                await this.sendLaunch(positionHash, objective)
            })
            .catch((error) => {
                this.onError && this.onError(error)
            })
    }

    private sendCommand = async (command: Command) => {
        const response = await fetch(`${this.serverConfig.url}/sessions/${this.session.sid}/command`, {
            method: "POST",
            headers: this.serverConfig.headers({
                "Content-Type": "application/json",
            }),
            body: JSON.stringify(command),
        })

        await assertOk(response)

        const result = await response.json() as CommandResult

        this.onResponse && this.onResponse({ type: "CommandResult", content: result })
    }

    private sendLaunch = async (hash: HashKey, objective: SearchObjective) => {
        const response = await fetch(`${this.serverConfig.url}/sessions/${this.session.sid}/launch`, {
            method: "POST",
            headers: this.serverConfig.headers(),
            body: JSON.stringify({ hash, objective }),
        })

        await assertOk(response)

        this.startStream()
    }

    control(command: MintakaProviderRuntimeCommand) {
        switch (command.type) {
            case "abort": {
                void this.sendAbort()
                break
            }
        }
    }

    private sendAbort = async () => {
        const response = await fetch(`${this.serverConfig.url}/sessions/${this.session.sid}/abort`, {
            method: "POST",
            headers: this.serverConfig.headers(),
        })

        await assertOk(response)
    }

    private closeStream() {
        if (this.eventSource) {
            this.eventSource.close()
            this.eventSource = undefined
        }
    }

    private startStream = () => {
        this.closeStream()

        const streamUrl = new URL(`${this.serverConfig.url}/sessions/${this.session.sid}/stream`)

        if (this.serverConfig.apiPassword) {
            streamUrl.searchParams.set("api_password", this.serverConfig.apiPassword)
        }

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
            this.onError && this.onError(error)
            this.eventSource = undefined
        }
    }
}
