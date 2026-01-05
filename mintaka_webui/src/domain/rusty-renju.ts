import { BoardExportItem, Color, DurationSchema, Pos } from "../wasm/pkg/mintaka_wasm"
import { HistoryEntry } from "./HistoryTree"

export const NUMS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15] as const

export const LETTERS = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o'] as const

export const INDEX_TO_POS: Pos[] =
    NUMS.flatMap((num) =>
        LETTERS.map((letter) =>
            `${letter}${num}` as const,
        ),
    )

export function flip(color: Color) {
    return color === "Black" ? "White" : "Black"
}

export function duration(secs: number, nanos?: number): DurationSchema {
    return {
        secs,
        nanos: nanos ?? 0,
    }
}

export const InfiniteDuration = duration(9271584000)

type StoneItem = Extract<BoardExportItem, { type: "Stone" }>
type NonStoneItem = Exclude<BoardExportItem, { type: "Stone" }>

export type BoardCellView =
    (
        | (StoneItem & { sequence: number })
        | NonStoneItem
        )
    & { pos: Pos }

export function buildBoardCellView(items: BoardExportItem[], linearHistory: HistoryEntry[]): BoardCellView[] {
    const sequenceMap = buildSequenceMap(linearHistory)

    return items.map((item, index) => {
        const pos = INDEX_TO_POS[index]

        return {
            pos,
            sequence: sequenceMap.get(pos)!,
            ...item,
        }
    })
}

function buildSequenceMap(linearHistory: HistoryEntry[]): Map<Pos, number> {
    const map = new Map<Pos, number>()

    for (const [index, entry] of linearHistory.entries()) {
        if (entry.pos === undefined) continue

        map.set(entry.pos, index + 1)
    }

    return map
}
