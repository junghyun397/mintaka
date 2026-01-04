import { createMemo, For, Match, Switch, useContext } from "solid-js"
import { AppContext } from "../context"
import { chunk, range } from "../utils/array"
import { LETTERS, NUMS } from "../domain/rusty-renju"
import { BoardCellView } from "../stores/game.store"

export function Board() {
    const { gameStore, workerStore } = useContext(AppContext)!

    const boardViewTopDown = createMemo(() =>
        chunk(gameStore.boardView, 15).toReversed(),
    )

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
            <g font-family="serif" font-size="0.8" fill="black" text-anchor="middle" dominant-baseline="middle">
                <For each={range(0, 15)}>{index => {
                    const num = index + 1
                    const letter = LETTERS[index].toUpperCase()

                    const numPosition = 30 - index * 2
                    const letterPosition = (index + 1) * 2
                    return <>
                        <text x="1" y={numPosition} text-anchor="end">{num}</text>
                        <text x="31" y={numPosition} text-anchor="start">{num}</text>
                        <text x={letterPosition} y="0.5">{letter}</text>
                        <text x={letterPosition} y="31.5">{letter}</text>
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
                [&_button.forbidden]:cursor-not-allowed [&_button.stone.black]:fill-black [&_button.stone.white]:fill-white
                "
                classList={{
                    "[&_button]:cursor-wait": workerStore.inComputing,
                    "[&_button.stone]:cursor-auto [&_button]:cursor-crosshair": workerStore.inComputing,
                }}
            >
                <For each={boardViewTopDown()}>{(row) =>
                    <For each={row}>{(cell) =>
                        <Cell cell={cell} />
                    }</For>
                }</For>
            </div>
        </div>
    </div>
}

function Cell(props: { cell: BoardCellView }) {
    const { gameActions, appConfigStore, gameStore } = useContext(AppContext)!

    const reversedFill = () =>
        props.cell.content === "Black" ? "white" : "black"

    return <button
        id={props.cell.pos}
        title={props.cell.pos}
        classList={{
            "stone": props.cell.type === "Stone",
            "black": props.cell.content === "Black",
            "white": props.cell.content === "White",
            "forbidden": props.cell.type === "Forbidden" && gameStore.playerColor === "Black",
        }}
        onClick={[gameActions.play, props.cell.pos]}
    >
        <Switch>
            <Match when={props.cell.type === "Stone" ? props.cell : undefined}>{cell =>
                <svg viewBox="0 0 100 100">
                    <circle
                        stroke-width="4"
                        cx="50" cy="50" r="45"
                    />
                    <Switch>
                        <Match when={appConfigStore.historyDisplay === "sequence"}>
                            <text
                                font-family="serif"
                                font-size="50"
                                fill={reversedFill()}
                                text-anchor="middle" dominant-baseline="middle"
                                x="50" y="54"
                            >
                                {cell().sequence}
                            </text>
                        </Match>
                        <Match when={
                            (appConfigStore.historyDisplay === "pair" || appConfigStore.historyDisplay === "last")
                            && cell().sequence === gameStore.history.length
                        }>
                            <circle
                                fill={reversedFill()}
                                cx="50" cy="50" r="10"
                            />
                        </Match>
                        <Match when={appConfigStore.historyDisplay === "pair" && (cell().sequence + 1) === gameStore.history.length}>
                            <g
                                stroke={reversedFill()}
                                stroke-width="4"
                            >
                                <line x1="35" y1="50" x2="65" y2="50"/>
                                <line x1="50" y1="35" x2="50" y2="65"/>
                            </g>
                        </Match>
                    </Switch>
                </svg>
            }</Match>
            <Match when={props.cell.type === "Forbidden"}>
                <svg class="fill-error stroke-0" viewBox="0 0 100 100">
                    <circle cx="50" cy="50" r="10"/>
                </svg>
            </Match>
        </Switch>
    </button>
}
