import type { History, Pos } from "../wasm/pkg/rusty_renju_wasm"
import { BoardWorker } from "../wasm/pkg/rusty_renju_wasm"
import { EmptyHistoryTree, type HistoryEntry, HistoryTree } from "./HistoryTree"
import { assertNever } from "../utils/never"

export type HistorySource =
    | { type: "history", content: History }
    | { type: "history-tree", content: HistoryTree }

export type AppGameState = {
    readonly boardWorker: BoardWorker,
    readonly historyTree: HistoryTree,
}

export function emptyAppGameState(): AppGameState {
    return { boardWorker: BoardWorker.empty(), historyTree: EmptyHistoryTree }
}

export const NUMS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15] as const

export const LETTERS = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o'] as const

export const INDEX_TO_POS: Pos[] =
    NUMS.flatMap((num) =>
        LETTERS.map((letter) =>
            `${letter}${num}` as const,
        ),
    )

export function parseHistory(source: string): History | undefined {
    const sequence = source.match(/[A-Oa-o](?:1[0-5]|[1-9])/g)

    if (sequence === null || sequence.length === 0 || sequence.length !== new Set(sequence).size)
        return undefined

    return sequence as History
}

export function historyToString(history: History): string | undefined {
    if (history.length === 0) return undefined

    return history.join("")
}

export function buildGameStateFromHistorySource(historySource: HistorySource): AppGameState {
    switch (historySource.type) {
        case "history": {
            const historyEntries: HistoryEntry[] = []

            let boardWorker = BoardWorker.empty()
            for (const pos of historySource.content) {
                boardWorker = boardWorker.set(pos)
                historyEntries.push({ hashKey: boardWorker.hashKey(), pos })
            }

            return { boardWorker, historyTree: new HistoryTree(undefined, historyEntries) }
        }
        case "history-tree": {
            return {
                boardWorker: BoardWorker.fromHistory(historySource.content.toHistory()),
                historyTree: historySource.content,
            }
        }
        default: assertNever(historySource)
    }
}
