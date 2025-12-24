import {createContext, JSXElement, ParentProps} from "solid-js";
import {MintakaProvider} from "./domain/mintaka.provider";
import {createStore, SetStoreFunction} from "solid-js/store";
import {BoardStore, createBoardStore} from "./stores/board.store";
import {createHistoryStore, HistoryStore} from "./stores/history.store";
import {HistoryTree} from "./domain/history";
import {BoardWorker, defaultBoard} from "./wasm/pkg";
import {UiStore} from "./stores/ui.store";

type AppContext = {
    mintakaProvider: MintakaProvider | undefined,

    boardWorker: BoardWorker,
    historyTree: HistoryTree,

    readonly uiStore: UiStore,
    readonly setUiStore: SetStoreFunction<UiStore>,

    readonly boardStore: BoardStore,
    readonly setBoardStore: SetStoreFunction<BoardStore>,

    readonly historyStore: HistoryStore,
    readonly setHistoryStore: SetStoreFunction<HistoryStore>,
}

export const AppContext = createContext<AppContext>()

export function AppContextProvider(props: ParentProps): JSXElement {
    const board = defaultBoard()
    const boardWorker = new BoardWorker(board)

    const historyTree = new HistoryTree(undefined, [])

    const [uiStore, setUiStore] = createStore<UiStore>({
        theme: "system",
        historyOpen: false,
        configOpen: false,
    })

    const [boardStore, setBoardStore] = createBoardStore(boardWorker)

    const [historyStore, setHistoryStore] = createHistoryStore(historyTree)

    return <AppContext.Provider
        value={{
            mintakaProvider: undefined,

            boardWorker,
            historyTree,

            uiStore,
            setUiStore,

            boardStore,
            setBoardStore,

            historyStore,
            setHistoryStore,
        }}
        children={props.children}
    />
}
