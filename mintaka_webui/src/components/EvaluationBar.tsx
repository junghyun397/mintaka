import { useContext } from "solid-js"
import { AppContext } from "../context"
import { flatmap } from "../utils/undefined"
import { HashKey } from "../wasm/pkg/rusty_renju_wasm"
import {useStableSignal} from "../utils/signal";

export function RootEvaluationBar() {
    const { gameSelectors } = useContext(AppContext)!

    return <div class="mx-auto w-full max-w-90">
        <EvaluationBar hash={gameSelectors.boardDescribe.hash_key} />
    </div>
}

export function EvaluationBar(props: { hash: HashKey }) {
    const { appSelectors } = useContext(AppContext)!

    const winRate = () => appSelectors.winRateTable[props.hash]

    const whitePercent = useStableSignal(50.0, () =>
        flatmap(winRate(), valid => (-valid + 1) / 2 * 100)
    )

    return <div
        class="h-4 overflow-hidden rounded-full border-3 border-base-300 bg-black transition-opacity duration-200 ease-out"
        classList={{
            "opacity-0": winRate() === undefined,
        }}
    >
        <div
            class="ml-auto h-full bg-white transition-[width] duration-300 ease-out"
            style={{ width: `${whitePercent() ?? 50}%` }}
        />
    </div>
}
