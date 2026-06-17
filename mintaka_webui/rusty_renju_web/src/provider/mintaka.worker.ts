import init, { GameAgent, initThreadPool, JsAbortHandle, JsCounterHandle, mintakaVersion } from "../../wasm/pkg/mintaka_wasm"
import type { MintakaWorkerMessage, MintakaWorkerResponse } from "./mintaka.worker.provider"

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
        counter: JsCounterHandle,
        abort: JsAbortHandle,
    } | undefined,
    post: (data: MintakaWorkerResponse) => self.postMessage(data),
    postError: (error: unknown) => self.postMessage({ type: "Error", content: error } as MintakaWorkerResponse),
}

self.addEventListener("message", async (event: MessageEvent<MintakaWorkerMessage>) => {
    try {
        await initWorker()

        switch (event.data.type) {
            case "init": {
                const { config, state } = event.data

                const abortHandle = new JsAbortHandle()
                const abortPtr = abortHandle.ptr()

                const counterHandle = new JsCounterHandle()
                const counterPtr = counterHandle.ptr()

                ctx.state = {
                    agent: new GameAgent(config, state),
                    counter: counterHandle,
                    abort: abortHandle,
                }

                ctx.post({ type: "Ready", version: mintakaVersion(), sab: memory!, counterPtr: counterPtr, abortPtr: abortPtr })
                break
            }
            case "config": {
                ctx.state!.agent.config(event.data.config)
                break
            }
            case "command": {
                const result = ctx.state!.agent.command(event.data.command)

                ctx.post({ type: "CommandResult", id: event.data.id, content: result })
                break
            }
            case "launch": {
                if (ctx.state?.agent === undefined) {
                    ctx.postError("agent not ready")
                    break
                }

                if (ctx.state?.agent?.hashKey() !== event.data.expectedHash) {
                    ctx.post({ type: "LaunchResult", id: event.data.id, content: "snapshot-mismatch" })
                    break
                }

                ctx.post({ type: "LaunchResult", id: event.data.id, content: "launched" })

                const bestMove = ctx.state.agent.launch(event.data.timer, event.data.objective, ctx.state.counter, ctx.state.abort)

                ctx.post({ type: "BestMove", content: bestMove })
                break
            }
        }
    } catch (error: unknown) {
        ctx.postError(error)
    }
})
