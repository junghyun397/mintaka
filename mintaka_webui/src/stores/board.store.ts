import {BoardExportItem, BoardWorker, Color, Pos} from "../wasm/pkg/mintaka_wasm";
import {createStore, SetStoreFunction} from "solid-js/store";
import {INDEX_TO_POS} from "../domain/rusty-renju";

export type BoardCellView = BoardExportItem & {
    readonly pos: Pos,
}

export type BoardStore = {
    readonly boardView: BoardCellView[],
    readonly userColor: Color,
    readonly playerColor: Color,
}

export function createBoardStore(board: BoardWorker): [BoardStore, SetStoreFunction<BoardStore>] {
    const [matchStore, setMatchStore] = createStore<BoardStore>({
        boardView: intoBoardView(board.field()),
        userColor: board.playerColor(),
        playerColor: board.playerColor(),
    })

    return [matchStore, setMatchStore]
}

export function intoBoardView(items: BoardExportItem[]): BoardCellView[] {
    return items.map((item, index) => ({
        pos: INDEX_TO_POS[index],
        ...item,
    }))
}
