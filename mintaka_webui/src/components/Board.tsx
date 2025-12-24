import {createMemo, For, Match, Switch, useContext} from "solid-js";
import {Pos} from "../wasm/pkg";
import {AppContext} from "../context";
import {chunk, range} from "../utils/array";
import {LETTERS, NUMS} from "../domain/rusty-renju";
import {BoardCellView} from "../stores/board.store";

export function Board() {
    const { boardWorker, boardStore } = useContext(AppContext)!

    const reversedBoardView = createMemo(() =>
        chunk(boardStore.boardView, 15).toReversed()
    )

    const onClick = (pos: Pos) => {
        if (!boardWorker.isLegalMove(pos)) {
            return
        }
    }

    // 1+2x15+1 = 32
    return <div class="relative h-full w-full @container-[size] grid place-items-center">
        <div class="relative aspect-square w-[min(100cqw,100cqh)] bg-orange-300">
            <svg class="absolute w-full h-full" viewBox="0 0 32 32" pointer-events="none">
                <g stroke="black" stroke-width="0.08" stroke-linecap="butt">
                    <For each={NUMS.slice(1, -1)}>{(sequence) =>
                        <>
                            <line x1={sequence * 2} y1={2} x2={sequence * 2} y2={30} />
                            <line x1={2} y1={sequence * 2} x2={30} y2={sequence * 2} />
                        </>
                    }</For>
                    <rect x={2} y={2} width={28} height={28} fill="none" />
                    <circle cx="16" cy="16" r="0.15"/>
                </g>
                <g font-family="serif" font-size="0.8" fill="black" text-anchor="middle" dominant-baseline="middle">
                    <For each={range(0, 15)}>{(index) => {
                        const num = NUMS[index]
                        const letter = LETTERS[index].toUpperCase()

                        const position = (index + 1) * 2
                        const numPosition = 30 - index * 2
                        return <>
                            <text x={1} y={numPosition} text-anchor="end">{num}</text>
                            <text x={31} y={numPosition} text-anchor="start">{num}</text>
                            <text x={position} y={0.5}>{letter}</text>
                            <text x={position} y={31.5}>{letter}</text>
                        </>
                    }}</For>
                </g>
            </svg>
            <div
                class="absolute w-full h-full grid grid-rows-15 grid-cols-15"
                classList={{"cursor-progress": false}}
                style={{ padding: "3.125%" }} // 1/32
            >
                <For each={reversedBoardView()}>{(row) =>
                    <For each={row}>{(cell) =>
                        <Cell onClick={onClick} cell={cell}/>
                    }</For>
                }</For>
            </div>
        </div>
    </div>
}

function Cell(props: {
    onClick: (pos: Pos) => void,
    cell: BoardCellView
}) {
    const { boardStore } = useContext(AppContext)!

    const cellClass = createMemo(() => {
        switch (props.cell.type) {
            case "Stone": return {
                "stroke-gray-500": true,
                "fill-black": props.cell.content === "Black",
                "fill-white": props.cell.content === "White",
            }
            case "Forbidden": return {
                "cursor-not-allowed": boardStore.userColor == "White",
            }
            case "Empty": return {}
        }
    })

    return <button
        id={props.cell.pos}
        title={props.cell.pos}
        class="relative"
        classList={cellClass()}
        onClick={[props.onClick, props.cell.pos]}
    >
        <Switch>
            <Match when={props.cell.type === "Stone"}>
                <svg class="" viewBox="0 0 100 100">
                    <circle cx="50" cy="50" r="45" stroke-width="4"/>
                </svg>
            </Match>
            <Match when={props.cell.type === "Forbidden"}>
                <svg class="fill-red-500" viewBox="0 0 100 100">
                    <circle cx="50" cy="50" r="10"/>
                </svg>
            </Match>
        </Switch>
</button>
}
