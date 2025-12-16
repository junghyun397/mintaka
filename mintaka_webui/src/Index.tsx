/* @refresh reload */
import './index.css';
import {render} from 'solid-js/web';
import 'solid-devtools';
import {App} from "./App";

const root = document.getElementById('root')

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
    throw new Error('Root element not found.')
}

render(() => <App />, root!)

void console_demo()

async function console_demo() {
    const worker = new Worker(
        new URL("./services/mintaka-worker.ts", import.meta.url),
        {type: "module"},
    )

    worker.onmessage = (event) => {
        console.log("[mintaka-worker]", event.data)

        const data = event.data as unknown
        if (data && typeof data === "object" && "type" in data && (data as any).type === "console_demo_result") {
            worker.terminate()
        }
    }

    worker.onerror = (event) => {
        console.error("[mintaka-worker] error", event)
        worker.terminate()
    }

    worker.postMessage({type: "console_demo_start"})
}
