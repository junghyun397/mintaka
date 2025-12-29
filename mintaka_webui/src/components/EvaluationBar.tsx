import {Show} from "solid-js";

export function EvaluationBar(props: { value: number | undefined }) {
    const blackPercent = () =>
        props.value ? ((props.value + 1) / 2) * 100 : undefined

    return <Show when={blackPercent()} fallback={
        <div class="h-4" />
    }>{percent =>
        <div
            class="h-4 border-3 border-base-300 rounded-full bg-white"
            role="meter"
        >
            <div
                class="h-full bg-black transition-all duration-200 ease-out rounded-l-full"
                style={{ width: `${percent}%` }}
            />
        </div>
    }</Show>
}
