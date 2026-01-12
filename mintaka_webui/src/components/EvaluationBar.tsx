import { createMemo, useContext } from "solid-js"
import { AppContext } from "../context"
import { calculateNormEval } from "../wasm/pkg/mintaka_wasm"

export function RootEvaluationBar() {
    const { appSelectors, boardDescribe, runtimeState } = useContext(AppContext)!

    const currentNormEval = createMemo(() => {
        const runtime = runtimeState()

        if (runtime != undefined && runtime.type === "streaming")
            return calculateNormEval(runtime.lastStatus.score, boardDescribe.player_color)

        return appSelectors.selectNormEval(boardDescribe.hash_key)
    })

    return <div class="mx-auto w-full max-w-90">
        <EvaluationBar normEval={currentNormEval()} />
    </div>
}

export function EvaluationBar(props: { normEval: number | undefined }) {
    const whitePercent = createMemo(() =>
        props.normEval !== undefined ? (-props.normEval + 1) / 2 * 100 : undefined,
    )

    return <div
        class="h-4 overflow-hidden rounded-full border-3 border-base-300 bg-black transition-opacity duration-200 ease-out"
        classList={{
            "opacity-0": whitePercent() === undefined,
        }}
    >
        <div
            class="ml-auto h-full bg-white transition-[width] duration-300 ease-out"
            style={{
                width: `${whitePercent() ?? 50}%`,
            }}
        />
    </div>
}
