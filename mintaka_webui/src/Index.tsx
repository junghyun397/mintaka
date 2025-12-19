/* @refresh reload */
import './index.css';
import {render} from 'solid-js/web';
import 'solid-devtools';
import {App} from "./App";

import init, {defaultConfig, GameStateWorker} from "./wasm/pkg";
import {MintakaWorkerControl, MintakaWorkerMessage, MintakaWorkerResponse} from "./services/mintaka.worker.protocol";

await init()

const root = document.getElementById('root')

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
    throw new Error('Root element not found.')
}

render(() => <App />, root!)

void consoleDemo()

async function consoleDemo() {
    let workerControl: MintakaWorkerControl | undefined = undefined

    const worker = new Worker(
        new URL("./services/mintaka.worker.ts", import.meta.url),
        {type: "module"},
    )

    worker.onmessage = (event: MessageEvent<MintakaWorkerResponse>) => {
        if (event.data.type == "Ready") {
            workerControl = event.data.content
        }

        console.log("[mintaka-worker]", event.data)
    }

    worker.onerror = (event) => {
        console.error("[mintaka-worker] error", event)
        worker.terminate()
    }

    function post(worker: Worker, data: MintakaWorkerMessage) {
        worker.postMessage(data)
    }

    const config = defaultConfig()
    const gameState = GameStateWorker.default().toJs()

    post(worker, {type: "init", payload: { config: config, state: gameState }})
    post(worker, {type: "command", payload: { type: "Workers", content: 4}})
    post(worker, {type: "command", payload: { type: "TurnTime", content: {"secs": 1, "nanos": 0}}})
    post(worker, {type: "launch", payload: {}})
}
