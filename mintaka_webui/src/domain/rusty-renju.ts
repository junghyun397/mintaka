import type { BoardExportItem, Color, DurationSchema, Pos, Response } from "../wasm/pkg/mintaka_wasm"
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

export type ResponseBody = Extract<Response, { type: "Status" }>["content"]
