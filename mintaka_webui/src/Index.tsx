/* @refresh reload */
import './index.css';
import { render } from 'solid-js/web';
import 'solid-devtools';
import { App } from "./App";

import init, { defaultConfig, defaultGameState } from "./wasm/pkg/mintaka_wasm";
import { MintakaWorkerProvider } from "./domain/mintaka.worker.provider";
import { MintakaProvider } from "./domain/mintaka.provider";

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
