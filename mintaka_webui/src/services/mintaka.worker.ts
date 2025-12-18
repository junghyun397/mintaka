import init, {BestMove, GameAgent, initThreadPool, JsAbortHandle, SearchObjective} from "../wasm/pkg";
import {MintakaWorkerMessage, MintakaWorkerResponse} from "./mintaka.worker.protocol";

let readyPromise: Promise<void> | undefined

export function initWorker() {
    if (!readyPromise) {
        readyPromise = (async () => {
            await init()
            await initThreadPool(navigator.hardwareConcurrency)
        })();
    }

    return readyPromise
}

const ctx = {
    agent: null as GameAgent | null,
    post: (data: MintakaWorkerResponse) => self.postMessage(data),
    postError: (error: any) => ctx.post({type: "Error", error: error})
}

self.addEventListener("message", async (event: MessageEvent<MintakaWorkerMessage>) => {
    try {
        await initWorker()

        switch (event.data.type) {
            case "init": {
                ctx.agent = new GameAgent(event.data.payload.config, event.data.payload.state)
                break
            }
            case "command": {
                ctx.agent!.command(event.data.payload)
                break
            }
            case "launch": {
                const bestMove: BestMove = ctx.agent!.launch(SearchObjective.Best, new JsAbortHandle())

                ctx.post({type: "BestMove", content: bestMove })
                break
            }
        }
    } catch (error: any) {
        ctx.postError(error)
    }
})
