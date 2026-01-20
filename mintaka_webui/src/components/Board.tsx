import { createMemo, createSelector, For, Index, Match, Switch, useContext } from "solid-js"
import { AppContext } from "../context"
import { INDEX_TO_POS, LETTERS, NUMS } from "../domain/rusty-renju"
import { range } from "../utils/array"
import { filter } from "../utils/undefined"

export function Board() {
    const { gameSelectors, runtimeSelectors } = useContext(AppContext)!

    const lastSequence = createMemo(() => gameSelectors.history().length)
    const prevSequence = createMemo(() => lastSequence() - 1)
    const isLastSequence = createSelector(lastSequence)
    const isPrevSequence = createSelector(prevSequence)

    // 1+2x15+1 = 32
    return <div class="relative h-full w-full rounded-box bg-[#efb072]">
        <svg class="absolute inset-0" viewBox="0 0 32 32">
            <g stroke="black" stroke-width="0.08" stroke-linecap="butt">
                <For each={NUMS.slice(1, -1)}>{sequence =>
                    <>
                        <line x1={sequence * 2} y1="2" x2={sequence * 2} y2="30" />
                        <line x1="2" y1={sequence * 2} x2="30" y2={sequence * 2} />
                    </>
                }</For>
                <rect x="2" y="2" width="28" height="28" fill="none" />
                <circle cx="16" cy="16" r="0.15"/>
            </g>
            <g font-family="serif" font-size="0.8" fill="black" text-anchor="middle">
                <For each={NUMS}>{num => {
                    const letter = LETTERS[num - 1].toUpperCase()

                    const numPosition = 32 - num * 2
                    const letterPosition = num * 2
                    return <>
                        <text x="1" y={numPosition} dy="0.32em" text-anchor="end">{num}</text>
                        <text x="31" y={numPosition} dy="0.32em" text-anchor="start">{num}</text>
                        <text x={letterPosition} y="0.5" dy="0.32em">{letter}</text>
                        <text x={letterPosition} y="31.5" dy="0.32em">{letter}</text>
                    </>
                }}</For>
            </g>
        </svg>
        <div
            class="absolute inset-0 p-[3.125%]" // 1/32
        >
            <div
                class="
                grid h-full w-full
                grid-cols-15 grid-rows-15
                stroke-gray-500
                [&_button.stone]:cursor-auto
                [&_button.stone.black]:fill-black [&_button.stone.black_.glyph]:fill-white
                [&_button.stone.white]:fill-white [&_button.stone.white_.glyph]:fill-black
                "
                classList={{
                    "[&_button.forbidden]:cursor-not-allowed": gameSelectors.boardDescribe.player_color === "Black",
                    "[&_button]:cursor-wait": runtimeSelectors.inComputing(),
                    "[&_button]:cursor-crosshair": !runtimeSelectors.inComputing(),
                }}
            >
                <Index each={range(0, 15 * 15)}>{position =>
                    <Cell
                        position={position()}
                        isLastSequence={isLastSequence}
                        isPrevSequence={isPrevSequence}
                    />
                }</Index>
            </div>
        </div>
    </div>
}

function Cell(props: {
    position: number,
    isLastSequence: (sequence: number) => boolean,
    isPrevSequence: (sequence: number) => boolean,
}) {
    const { gameActions, persistConfig, gameSelectors } = useContext(AppContext)!

    const pos = INDEX_TO_POS[props.position]

    const cell = createMemo(() => gameSelectors.boardDescribe.field[props.position])

    const stone = createMemo(() => filter(cell(), valid => valid.type === "Stone"))

    return <button
        id={pos}
        title={pos}
        classList={{
            "stone": cell().type === "Stone",
            "black": stone()?.content.color === "Black",
            "white": stone()?.content.color === "White",
            "forbidden": cell().type === "Forbidden",
        }}
        style={{
            "grid-row": 15 - Math.trunc(props.position / 15),
            "grid-column": (props.position % 15) + 1,
        }}
        onClick={[gameActions.play, pos]}
    >
        <Switch>
            <Match when={stone()}>{stone =>
                <svg viewBox="0 0 100 100">
                    <circle
                        stroke-width="4"
                        cx="50" cy="50" r="45"
                    />
                    <Switch>
                        <Match when={persistConfig.historyDisplay === "sequence"}>
                            <text
                                class="glyph"
                                font-family="serif"
                                font-size="50"
                                text-anchor="middle"
                                x="50" y="50" dy="0.32em"
                            >
                                {stone().content.sequence}
                            </text>
                        </Match>
                        <Match when={
                            (persistConfig.historyDisplay === "pair" || persistConfig.historyDisplay === "last")
                            && props.isLastSequence(stone().content.sequence)
                        }>
                            <circle
                                class="glyph"
                                cx="50" cy="50" r="10"
                            />
                        </Match>
                        <Match when={
                            persistConfig.historyDisplay === "pair"
                            && props.isPrevSequence(stone().content.sequence)
                        }>
                            <g class="glyph">
                                <rect x="35" y="48" width="30" height="4"/>
                                <rect x="48" y="35" width="4" height="30"/>
                            </g>
                        </Match>
                    </Switch>
                </svg>
            }</Match>
            <Match when={cell().type === "Forbidden"}>
                <svg
                    class="forbidden fill-error stroke-0"
                    viewBox="0 0 100 100"
                >
                    <circle cx="50" cy="50" r="10"/>
                </svg>
            </Match>
        </Switch>
    </button>
}
