/* @refresh reload */
import './index.css';
import {render} from 'solid-js/web';
import 'solid-devtools';
import {App} from "./App";

import {ready} from "./services/rusty-renju";
import {MintakaWorkerMessage, MintakaWorkerResponse} from "./services/mintaka.worker.protocol";

await ready

const root = document.getElementById('root')

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
    throw new Error('Root element not found.')
}

render(() => <App />, root!)

void console_demo()

async function console_demo() {
    const worker = new Worker(
        new URL("./services/mintaka.worker.ts", import.meta.url),
        {type: "module"},
    )

    worker.onmessage = (event: MessageEvent<MintakaWorkerResponse>) => {
        console.log("[mintaka-worker]", event.data)
    }

    worker.onerror = (event) => {
        console.error("[mintaka-worker] error", event)
        worker.terminate()
    }

    function post(worker: Worker, data: MintakaWorkerMessage) {
        worker.postMessage(data)
    }

    post(worker, {type: "init", payload: { config: undefined, state: undefined }})
    post(worker, {type: "launch", payload: {}})
}
