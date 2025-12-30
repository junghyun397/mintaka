/* @refresh reload */
import './index.css';
import { render } from 'solid-js/web';
import 'solid-devtools';
import { App } from "./App";

import init, { defaultConfig, defaultGameState } from "./wasm/pkg/mintaka_wasm";
import { MintakaWorkerProvider } from "./domain/mintaka.worker.provider";
import { MintakaProvider } from "./domain/mintaka.provider";
import {
    checkHealth,
    createSession,
    LocalHostServerConfig,
    MintakaServerConfig,
    MintakaServerProvider,
} from "./domain/mintaka.server.provider";

await init()

const root = document.getElementById('root')

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
    throw new Error('Root element not found.')
}

render(() => <App />, root!)

async function worker(): Promise<MintakaProvider> {
    const config = defaultConfig()
    const gameState = defaultGameState()

    return new MintakaWorkerProvider(config, gameState)
}

async function server(): Promise<MintakaProvider> {
    const config = defaultConfig()
    const gameState = defaultGameState()

    const serverConfig = LocalHostServerConfig

    const healthy = await checkHealth(serverConfig)

    console.log("healthy: ", healthy)

    const session = await createSession(serverConfig, config, gameState)

    console.log("session: ", session)

    return new MintakaServerProvider(serverConfig, session)
}

async function consoleDemo() {
    const mintakaProvider = await worker()

    mintakaProvider.onResponse = (message) => {
        console.log("response: ", message)
    }

    mintakaProvider.onError = (error) => {
        console.error("worker error: ", error)
    }

    mintakaProvider.idleState!.message({ type: "command", payload: { type: "TurnTime", content: { secs: 1, nanos: 0 } } })
    mintakaProvider.idleState!.message({ type: "command", payload: { type: "Workers", content: 1 } })
    mintakaProvider.idleState!.message({ type: "launch", payload: { objective: "Best" } })
}
