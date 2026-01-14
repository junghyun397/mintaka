import { MintakaProvider, MintakaProviderResponse, MintakaProviderRuntimeCommand, MintakaProviderType } from "./mintaka.provider"
import type { Command, CommandResult, Config, GameState, HashKey, SearchObjective } from "../wasm/pkg/mintaka_wasm"

export class MintakaServerConfig {
    readonly address: string
    private readonly apiPassword?: string

    constructor(address: string, apiPassword?: string) {
        this.address = address
        this.apiPassword = apiPassword
    }

    get url() {
        return this.address
    }

    headers = (extra?: HeadersInit) => {
        const headers = new Headers(extra)

        if (this.apiPassword) headers.set("Api-Password", this.apiPassword)

        return headers
    }
}

export type MintakaServerSession = {
    readonly sid: string,
    readonly defaultConfig: Config,
    readonly maxConfig: Config,
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

    return await response.json()
}

export class MintakaServerProvider implements MintakaProvider {
    private readonly serverConfig: MintakaServerConfig
    private readonly session: MintakaServerSession
    private chain: Promise<void> = Promise.resolve()
    private eventSource?: EventSource

    private onResponse?: (message: MintakaProviderResponse) => void
    private onError?: (error: any) => void

    readonly type: MintakaProviderType = "server"

    get defaultConfig() {
        return this.session.defaultConfig
    }

    get maxConfig() {
        return this.session.maxConfig
    }

    constructor(config: MintakaServerConfig, session: MintakaServerSession) {
        this.serverConfig = config
        this.session = session
    }

    subscribeResponse(handler: (response: MintakaProviderResponse) => void) {
        this.onResponse = handler
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

        const result = await response.json() as CommandResult

        this.onResponse && this.onResponse({ type: "CommandResult", content: result })
    }

    private sendLaunch = async (hash: HashKey, objective: SearchObjective) => {
        const response = await fetch(`${this.serverConfig.url}/sessions/${this.session.sid}/launch`, {
            method: "POST",
            headers: this.serverConfig.headers(),
            body: JSON.stringify({ hash, objective }),
        })

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
