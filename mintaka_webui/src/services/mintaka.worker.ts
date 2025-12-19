import init, {BestMove, GameAgent, initThreadPool, JsAbortHandle} from "../wasm/pkg";
import {MintakaWorkerMessage, MintakaWorkerResponse} from "./mintaka.worker.protocol";

let readyPromise: Promise<void> | undefined
let memory: SharedArrayBuffer | undefined

export function initWorker() {
    if (!readyPromise) {
        readyPromise = (async () => {
            const initOut = await init()
            await initThreadPool(navigator.hardwareConcurrency)

            const buffer = initOut.memory.buffer
            if (typeof SharedArrayBuffer === "undefined" || !(buffer instanceof SharedArrayBuffer)) {
                throw new Error("WASM memory is not SharedArrayBuffer; abort via Atomics is unavailable (check COOP/COEP + threads build).")
            }
            memory = buffer
        })();
    }

    return readyPromise
}

const ctx = {
    state: undefined as {
        agent: GameAgent,
        control: JsAbortHandle
    } | undefined,
    post: (data: MintakaWorkerResponse) => self.postMessage(data),
    postError: (error: any) => ctx.post({type: "Error", error: error})
}

self.addEventListener("message", async (event: MessageEvent<MintakaWorkerMessage>) => {
    try {
        await initWorker()

        switch (event.data.type) {
            case "init": {
                const { config, state } = event.data.payload

                const control = new JsAbortHandle()
                const ptr = control.ptr()

                ctx.state = {
                    control: control,
                    agent: new GameAgent(config, state)
                }

                ctx.post({type: "Ready", content: { sab: memory!, control_ptr: ptr }})
                break
            }
            case "command": {
                ctx.state!.agent.command(event.data.payload)
                break
            }
            case "launch": {
                const bestMove: BestMove = ctx.state!.agent.launch("Best", ctx.state!.control)

                ctx.post({type: "BestMove", content: bestMove })
                break
            }
        }
    } catch (error: any) {
        ctx.postError(error)
    }
})
