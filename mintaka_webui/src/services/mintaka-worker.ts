import init, {GameAgent, GameState, initThreadPool, JsAbortHandle, Pos, SearchObjective} from "../wasm/pkg";

await init()

await initThreadPool(navigator.hardwareConcurrency)

const ctx = self

ctx.addEventListener("message", async (_: MessageEvent<boolean>) => {
    try {
        let state = new GameState()
        state = state.play(new Pos("h8"))
        state = state.play(new Pos("h9"))

        const config = {
            rule_kind: "Renju",
            draw_condition: 225,
            max_nodes_in_1k: 100000000,
            max_depth: 20,
            max_vcf_depth: 12,
            tt_size: 512 * 1024 * 1024,
            workers: 8,
            pondering: false,
            dynamic_time: false,
            initial_timer: {
                total_remaining: { secs: 600, nanos: 0 },
                increment: { secs: 0, nanos: 0 },
                turn: { secs: 1, nanos: 0 },
            },
            spawn_depth_specialist: false,
        }

        const agent = GameAgent.fromState(config, state)
        const abort_handle = new JsAbortHandle()

        const best_move = agent.launch(SearchObjective.Best, abort_handle)
        ctx.postMessage({BestMove: best_move })
    } catch (error) {
        ctx.postMessage({Error: {
                error: String(error),
                stack: error instanceof Error ? error.stack : undefined,
            }
        })
    }
})
