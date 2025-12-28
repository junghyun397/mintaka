import {createContext, ParentProps} from "solid-js";
import {MintakaProvider} from "./domain/mintaka.provider";
import {createStore, reconcile, SetStoreFunction, unwrap} from "solid-js/store";
import {buildGameStore, GameStore} from "./stores/game.store";
import {EmptyHistoryTree, ForwardMethod, HistoryTree} from "./domain/history";
import {BoardWorker, defaultBoard, Pos} from "./wasm/pkg/mintaka_wasm";
import {AppConfig, defaultAppConfig} from "./stores/config.store";
import {flip} from "./domain/rusty-renju";
import {ComputingStore} from "./stores/computing.store";

type AppState = {
    boardWorker: BoardWorker,
    historyTree: HistoryTree,
}

interface AppActions {
    play: (pos: Pos) => void
    forward: (method: ForwardMethod) => void
    bulkForward: (method: ForwardMethod) => void
    backward: () => void
    bulkBackward: () => void
}

type AppContext = {
    mintakaProvider: MintakaProvider | undefined,

    readonly actions: AppActions,

    readonly appState: AppState,

    readonly appConfigStore: AppConfig,
    readonly setAppConfigStore: SetStoreFunction<AppConfig>,

    readonly gameStore: GameStore,
    readonly computingStore: ComputingStore,
}

export const AppContext = createContext<AppContext>()

export function AppContextProvider(props: ParentProps) {
    const appState = {
        boardWorker: new BoardWorker(defaultBoard()),
        historyTree: EmptyHistoryTree,
    }

    const appConfig = defaultAppConfig()

    const [appConfigStore, setAppConfigStore] = createStore(appConfig)

    const [gameStore, setGameStore] = createStore(buildGameStore(appState.boardWorker, appState.historyTree, "Black"))

    const [computingStore, setComputingStore] = createStore<ComputingStore>({ state: undefined })

    const reconcileGameStore = () =>
        setGameStore(reconcile(buildGameStore(appState.boardWorker, appState.historyTree, flip(unwrap(gameStore.playerColor)))))

    const actions = {
        play: (pos: Pos) => {
            if (!appState.boardWorker.isLegalMove(pos)) return

            appState.boardWorker = appState.boardWorker.set(pos)
            appState.historyTree = appState.historyTree.push({ pos })

            reconcileGameStore()
        },
        forward: (method: ForwardMethod) => {
            const result = appState.historyTree.forward(method)
            if (!result) return
            const [historyTree, entry] = result

            appState.historyTree = historyTree
            appState.boardWorker = appState.boardWorker.set(entry.pos!)

            reconcileGameStore()
        },
        bulkForward: (method: ForwardMethod) => {
            const result = appState.historyTree.bulkForward(method)
            if (!result) return
            const [historyTree, entries] = result

            appState.historyTree = historyTree
            for (const entry of entries) {
                if (!entry.pos)
                    appState.boardWorker = appState.boardWorker.pass()
                else
                    appState.boardWorker = appState.boardWorker.set(entry.pos)
            }

            reconcileGameStore()
        },
        backward: () => {
            const result = appState.historyTree.backward()
            if (!result) return
            const [historyTree, entry] = result

            appState.historyTree = historyTree
            appState.boardWorker = appState.boardWorker.unset(entry.pos!)

            reconcileGameStore()
        },
        bulkBackward: () => {
            const result = appState.historyTree.bulkBackward()
            if (!result) return
            const [historyTree, entries] = result

            appState.historyTree = historyTree
            for (const entry of entries.reverse()) {
                if (!entry.pos)
                    appState.boardWorker = appState.boardWorker.pass()
                else
                    appState.boardWorker = appState.boardWorker.unset(entry.pos)
            }

            reconcileGameStore()
        }
    }

    return <AppContext.Provider
        value={{
            mintakaProvider: undefined,

            actions,
            appState,

            appConfigStore,
            setAppConfigStore,

            gameStore,
            computingStore,
        }}
        children={props.children}
    />
}
