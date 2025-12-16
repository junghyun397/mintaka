import init, {GameAgent, GameState, JsAbortHandle, Pos, SearchObjective} from "../wasm/pkg/mintaka_wasm";

await init();

type ConsoleDemoStartMessage = { type: "console_demo_start" };

const ctx: any = self;

ctx.addEventListener("message", (event: MessageEvent<ConsoleDemoStartMessage>) => {
    if (event.data?.type !== "console_demo_start") return;

    try {
        let state = new GameState();
        state = state.play(new Pos("h8"));
        state = state.play(new Pos("h9"));

        const config = {
            rule_kind: "Renju",
            draw_condition: 225,
            max_nodes_in_1k: 100,
            max_depth: 20,
            max_vcf_depth: 12,
            tt_size: 32 * 1024 * 1024,
            workers: 1,
            pondering: false,
            dynamic_time: false,
            initial_timer: {
                total_remaining: { secs: 5, nanos: 0 },
                increment: { secs: 0, nanos: 0 },
                turn: { secs: 1, nanos: 0 },
            },
            spawn_depth_specialist: false,
        };

        const agent = GameAgent.fromState(config, state);
        const abort_handle = new JsAbortHandle();

        const best_move = agent.launch(SearchObjective.Best, abort_handle);
        ctx.postMessage({ type: "console_demo_result", best_move });
    } catch (error) {
        ctx.postMessage({ type: "console_demo_error", error });
    }
});
