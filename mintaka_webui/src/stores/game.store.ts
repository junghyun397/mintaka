import { BoardExportItem, BoardWorker, Color, Pos } from "../wasm/pkg/mintaka_wasm"
import { INDEX_TO_POS } from "../domain/rusty-renju"
import { HistoryEntry, HistoryTree } from "../domain/HistoryTree"

type StoneItem = Extract<BoardExportItem, { type: "Stone" }>
type NonStoneItem = Exclude<BoardExportItem, { type: "Stone" }>

export type BoardCellView =
    (
        | (StoneItem & { sequence: number })
        | NonStoneItem
    )
    & { pos: Pos }

export type GameStore = {
    readonly boardView: BoardCellView[],
    readonly playerColor: Color,

    readonly history: HistoryEntry[],
    readonly backwardable: boolean,
    readonly forwardable: boolean,
    readonly inBranchHead: boolean,
}

export function buildGameStore(board: BoardWorker, historyTree: HistoryTree): GameStore {
    const history = historyTree.linear()

    return {
        boardView: buildBoardView(board.field(), history),
        playerColor: board.playerColor(),

        history,
        backwardable: historyTree.backwardable,
        forwardable: historyTree.forwardable,
        inBranchHead: historyTree.inBranchHead,
    }
}

function buildBoardView(items: BoardExportItem[], linearHistory: HistoryEntry[]): BoardCellView[] {
    return items.map((item, index) => {
        const pos = INDEX_TO_POS[index]

        return {
            pos,
            sequence: linearHistory.findIndex((entry) => entry.pos === pos) + 1,
            ...item,
        }
    })
}
