import init, { GameAgent, initThreadPool, JsAbortHandle } from "../wasm/pkg/mintaka_wasm_worker"
import { MintakaWorkerMessage, MintakaWorkerResponse } from "./mintaka.worker.provider"

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
        })()
    }

    return readyPromise
}

const ctx = {
    state: undefined as {
        agent: GameAgent,
        abort: JsAbortHandle
    } | undefined,
    post: (data: MintakaWorkerResponse) => self.postMessage(data),
    postError: (error: any) => self.postMessage({ type: "Error", error: error }),
}

self.addEventListener("message", async (event: MessageEvent<MintakaWorkerMessage>) => {
    try {
        await initWorker()

        switch (event.data.type) {
            case "init": {
                const { config, state } = event.data

                const abortHandle = new JsAbortHandle()
                const ptr = abortHandle.ptr()

                ctx.state = {
                    agent: new GameAgent(config, state),
                    abort: abortHandle,
                }

                ctx.post({ type: "Ready", sab: memory!, controlPtr: ptr })
                break
            }
            case "command": {
                const result = ctx.state!.agent.command(event.data.command)

                ctx.post({ type: "CommandResult", content: result })
                break
            }
            case "launch": {
                if (ctx.state?.agent === undefined) {
                    ctx.postError("agent not ready")
                    break
                }

                if (ctx.state?.agent?.hashKey() !== event.data.hash) {
                    ctx.postError("snapshot missmatch")
                    break
                }

                const bestMove = ctx.state.agent.launch(event.data.objective, ctx.state.abort)

                ctx.post({ type: "BestMove", content: bestMove })
                break
            }
        }
    } catch (error: any) {
        ctx.postError(error)
    }
})
