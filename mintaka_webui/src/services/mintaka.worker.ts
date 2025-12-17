import init, {GameAgent, initThreadPool, JsAbortHandle, SearchObjective} from "../wasm/pkg";
import {MintakaWorkerMessage, MintakaWorkerResponse} from "./mintaka.worker.protocol";

function serializeError(error: unknown) {
    if (error instanceof Error) {
        return {name: error.name, message: error.message, stack: error.stack}
    }
    return {name: "UnknownError", message: String(error)}
}

const runtimeReady = (async () => {
    await init()
    await initThreadPool(navigator.hardwareConcurrency)
})()

const handlers = {
    init: async (payload: { config: any, state: any }, ctx: Context) => {
        ctx.agent = new GameAgent(payload.config)
    },
    command: async (command: any, ctx: Context) => {
        if (ctx.agent == null) {
            throw new Error("agent is not initialized yet")
        }

        ctx.agent.command(command)
    },
    launch: async (_: {}, ctx: Context) => {
        if (ctx.agent == null) {
            throw new Error("agent is not initialized yet")
        }

        const best_move = ctx.agent.launch(SearchObjective.Best, new JsAbortHandle())

        ctx.post({type: "best-move", payload: best_move })
    },
}

const ctx = {
    agent: null as GameAgent | null,
    post: (data: MintakaWorkerResponse) => self.postMessage(data),
}

type Context = typeof ctx

self.addEventListener("message", async (event: MessageEvent<MintakaWorkerMessage>) => {
    try {
        await runtimeReady
        await handlers[event.data.type as keyof typeof handlers](event.data.payload, ctx)
    } catch (error: any) {
        ctx.post({type: "error", payload: serializeError(error)})
    }
})
