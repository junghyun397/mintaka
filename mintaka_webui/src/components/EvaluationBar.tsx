import { createMemo, useContext } from "solid-js"
import { AppContext } from "../context"

export function RootEvaluationBar() {
    const { workerStore } = useContext(AppContext)!

    return <div class="mx-auto w-full max-w-90">
        <EvaluationBar normEval={0} />
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
