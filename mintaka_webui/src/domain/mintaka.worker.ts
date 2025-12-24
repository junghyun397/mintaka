import init, {BestMove, GameAgent, initThreadPool, JsAbortHandle} from "../wasm/pkg";
import {MintakaWorkerControl, MintakaWorkerMessage, MintakaWorkerResponse} from "./mintaka.worker.provider";

let readyPromise: Promise<void> | undefined
let memory: SharedArrayBuffer | undefined

export function initWorker() {
    if (!readyPromise) {
        readyPromise = (async () => {
            const initOut = await init()
            await initThreadPool(navigator.hardwareConcurrency)

            const buffer = initOut.memory.buffer
            if (!(buffer instanceof SharedArrayBuffer)) {
                throw new Error("check COOP/COEP")
            }
            memory = buffer
        })();
    }

    return readyPromise
}

const ctx = {
    state: undefined as {
        agent: GameAgent,
        abort: JsAbortHandle
    } | undefined,
    post: (data: MintakaWorkerResponse) => self.postMessage(data),
    postError: (error: any) => self.postMessage({type: "Error", error: error})
}

self.addEventListener("message", async (event: MessageEvent<MintakaWorkerMessage>) => {
    try {
        await initWorker()

        switch (event.data.type) {
            case "init": {
                const { config, state } = event.data.payload

                const abortHandle = new JsAbortHandle()
                const ptr = abortHandle.ptr()

                const control = new MintakaWorkerControl(memory!, ptr)

                ctx.state = {
                    abort: abortHandle,
                    agent: new GameAgent(config, state)
                }

                ctx.post({type: "Ready", control: control })
                break
            }
            case "command": {
                ctx.state!.agent.command(event.data.payload)
                break
            }
            case "launch": {
                const bestMove: BestMove = ctx.state!.agent.launch("Best", ctx.state!.abort)

                ctx.post({type: "BestMove", content: bestMove })
                break
            }
        }
    } catch (error: any) {
        ctx.postError(error)
    }
})
