import {createContext, createEffect, onCleanup, onMount, ParentProps} from "solid-js";
import {MintakaProvider} from "./domain/mintaka.provider";
import {createStore, reconcile, SetStoreFunction, unwrap} from "solid-js/store";
import {buildGameStore, GameStore} from "./stores/game.store";
import {EmptyHistoryTree, ForwardMethod, HistoryTree} from "./domain/history";
import {BoardWorker, defaultBoard, Pos} from "./wasm/pkg/mintaka_wasm";
import {AppConfig, defaultAppConfig, Theme} from "./stores/app.config.store";
import {flip} from "./domain/rusty-renju";
import {ComputingStore} from "./stores/computing.store";
import {makePersisted} from "@solid-primitives/storage";

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
    const appState = createAppState()

    const [appConfigStore, setAppConfigStore] = createAppConfigStore()

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

export function createAppConfigStore(): [AppConfig, SetStoreFunction<AppConfig>] {
    const [appConfigStore, setAppConfigStore] = makePersisted(
        createStore(defaultAppConfig())
    )

    const removeTheme = () =>
        document.documentElement.removeAttribute("data-theme")

    const applyTheme = (theme: Exclude<Theme, "system">) => {
        document.documentElement.setAttribute("data-theme", theme)
    }

    onMount(() => {
        const mediaQueryList = window.matchMedia("(prefers-color-scheme: dark)");

        const handler = () => {
            if (appConfigStore.theme === "system")
                removeTheme()
        }

        mediaQueryList.addEventListener?.("change", handler)

        onCleanup(() => {
            mediaQueryList.removeEventListener?.("change", handler)
        })
    })

    createEffect(() => {
        if (appConfigStore.theme === "system")
            removeTheme()
        else
            applyTheme(appConfigStore.theme)
    })

    return [appConfigStore, setAppConfigStore]
}

export function createAppState(): AppState {
    return {
        boardWorker: new BoardWorker(defaultBoard()),
        historyTree: EmptyHistoryTree,
    }
}
