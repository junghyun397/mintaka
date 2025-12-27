import {createContext, ParentProps} from "solid-js";
import {MintakaProvider} from "./domain/mintaka.provider";
import {createStore, SetStoreFunction} from "solid-js/store";
import {BoardStore, createBoardStore} from "./stores/board.store";
import {createHistoryStore, HistoryStore} from "./stores/history.store";
import {HistoryTree} from "./domain/history";
import {BoardWorker, defaultBoard, defaultConfig} from "./wasm/pkg/mintaka_wasm";
import {AppConfig} from "./stores/config.store";

type AppContext = {
    mintakaProvider: MintakaProvider | undefined,

    boardWorker: BoardWorker,
    historyTree: HistoryTree,

    readonly appConfigStore: AppConfig,
    readonly setAppConfigStore: SetStoreFunction<AppConfig>,

    readonly boardStore: BoardStore,
    readonly setBoardStore: SetStoreFunction<BoardStore>,

    readonly historyStore: HistoryStore,
    readonly setHistoryStore: SetStoreFunction<HistoryStore>,
}

export const AppContext = createContext<AppContext>()

export function AppContextProvider(props: ParentProps) {
    const board = defaultBoard()
    const boardWorker = new BoardWorker(board)

    const historyTree = new HistoryTree(undefined, [])

    const appConfig: AppConfig = {
        serverConfig: undefined,
        config: defaultConfig(),
        theme: "system",
        openHistory: false
    }

    const [appConfigStore, setAppConfigStore] = createStore(appConfig)

    const [boardStore, setBoardStore] = createBoardStore(boardWorker)

    const [historyStore, setHistoryStore] = createHistoryStore(historyTree)

    return <AppContext.Provider
        value={{
            mintakaProvider: undefined,

            boardWorker,
            historyTree,

            appConfigStore,
            setAppConfigStore,

            boardStore,
            setBoardStore,

            historyStore,
            setHistoryStore,
        }}
        children={props.children}
    />
}
