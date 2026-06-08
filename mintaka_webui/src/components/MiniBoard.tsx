import { Index, Match, Show, Switch } from "solid-js"
import type { BoardDescribe, MaybePos } from "../wasm/pkg/rusty_renju_wasm"
import { range } from "../utils/array"
import { INDEX_TO_POS } from "../domain/rusty-renju"
import { filter } from "../utils/undefined"

export function MiniBoard(props: {
    describe: BoardDescribe,
    lastPos: MaybePos,
}) {
    return <div class="aspect-square size-full rounded-sm bg-[#efb072] p-1">
        <svg
            viewBox="0 0 30 30"
            class="
            size-full
            [&_.black]:fill-black
            [&_.black+.recent]:fill-white
            [&_.forbidden]:fill-[#ff0000]
            [&_.stone]:stroke-gray-500
            [&_.stone]:stroke-[0.15]
            [&_.white]:fill-white
            [&_.white+.recent]:fill-black
            "
        >
            <g stroke="black" stroke-linecap="butt" stroke-width="0.08">
                <Index each={range(1, 14)}>{sequence => {
                    const position = sequence() * 2 + 1

                    return <>
                        <line x1={position} y1="1" x2={position} y2="29" />
                        <line x1="1" y1={position} x2="29" y2={position} />
                    </>
                }}</Index>
                <rect x="1" y="1" width="28" height="28" fill="none" />
            </g>
            <Index each={range(0, 15 * 15)}>{position => {
                const cx = position() % 15 * 2 + 1
                const cy = (15 - Math.trunc(position() / 15)) * 2 - 1
                const pos = INDEX_TO_POS[position()]

                return <Switch>
                    <Match when={filter(props.describe.field[position()], valid => valid.type === "Stone")}>{stone =>
                        <>
                            <circle cx={cx} cy={cy} r="0.95" class="stone" classList={{
                                "black": stone().content === "Black",
                                "white": stone().content === "White",
                            }} />
                            <Show when={pos === props.lastPos}>
                                <circle cx={cx} cy={cy} r="0.3" class="recent"/>
                            </Show>
                        </>
                    }</Match>
                    <Match when={props.describe.field[position()].type === "Forbidden"}>
                        <circle cx={cx} cy={cy} r="0.3" class="forbidden"/>
                    </Match>
                </Switch>
            }}</Index>
        </svg>
    </div>
}
