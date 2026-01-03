import { createContext, createEffect, onCleanup, onMount, ParentProps } from "solid-js";
import { MintakaProvider } from "./domain/mintaka.provider";
import { createStore, reconcile, SetStoreFunction, unwrap } from "solid-js/store";
import { buildGameStore, GameStore } from "./stores/game.store";
import { EmptyHistoryTree, ForwardMethod, HistoryTree } from "./domain/HistoryTree";
import {
    BoardWorker,
    calculateNormEval,
    Color, Config,
    defaultBoard,
    defaultConfig,
    defaultGameState,
    HashKey,
    History,
    MaybePos,
    Pos,
} from "./wasm/pkg/mintaka_wasm";
import { AppConfig, defaultAppConfig, Theme } from "./stores/app.config.store";
import { flip } from "./domain/rusty-renju";
import { makePersisted } from "@solid-primitives/storage";
import { MintakaWorkerProvider } from "./domain/mintaka.worker.provider";
import { createWorkerStore, WorkerStore } from "./stores/worker.store";

type AppState = {
    mintakaProvider?: MintakaProvider,
    boardWorker: BoardWorker,
    historyTree: HistoryTree,
    evalTable: Map<HashKey, number>,
}

interface AppActions {
    play: (pos: Pos) => void,
    forward: (method: ForwardMethod) => void,
    bulkForward: (method: ForwardMethod) => void,
    backward: () => void,
    bulkBackward: () => void,
    start: () => void,
    pause: () => void,
    abort: () => void,
}

type AppContext = {
    readonly actions: AppActions,

    readonly appConfigStore: AppConfig,
    readonly setAppConfigStore: SetStoreFunction<AppConfig>,

    readonly gameStore: GameStore,
    readonly workerStore: WorkerStore,
}

export const AppContext = createContext<AppContext>()

export function AppContextProvider(props: ParentProps) {
    const appState = createAppState()

    const [appConfigStore, setAppConfigStore] = createAppConfigStore()

    const [gameStore, setGameStore] = createStore(buildGameStore(appState.boardWorker, appState.historyTree, "Black"))

    const [workerStore, setWorkerStore] = createWorkerStore()

    const reconcileGameStore = () =>
        setGameStore(reconcile(buildGameStore(appState.boardWorker, appState.historyTree, flip(unwrap(gameStore.playerColor)))))

    const syncSet = (pos: MaybePos) => {
        if (appState.mintakaProvider && appState.mintakaProvider.state.type === "idle")
            appState.mintakaProvider.state.command({ type: "Play", content: pos })
    }

    const syncUnset = (pos: MaybePos, color: Color) => {
        if (appState.mintakaProvider && appState.mintakaProvider.state.type === "idle")
            return
    }

    const resolveDesync = (stopAt: HashKey) => {
        const result = appState.historyTree.backwardTo(stopAt)

        if (!result) return
        const [historyTree, entries] = result

        appState.historyTree = historyTree
        for (const entry of entries.reverse()) {
            appState.boardWorker = appState.boardWorker.unset(entry.pos)
        }

        reconcileGameStore()
    }

    const play = (pos: MaybePos) => {
        if (pos ? !appState.boardWorker.isLegalMove(pos) : true) return

        appState.boardWorker = appState.boardWorker.set(pos)
        const hashKey = appState.boardWorker.hashKey()

        appState.historyTree = appState.historyTree.push({ hashKey, pos })

        syncSet(pos)

        reconcileGameStore()
    }

    const launch = () => {
        if (appState.mintakaProvider === undefined) return

        if (appState.mintakaProvider.state.type === "in_computing") return

        setWorkerStore("inComputing", true)

        const result = appState.mintakaProvider.state.launch(appState.boardWorker.hashKey(), "Best")

        console.log(result)
    }

    const abort = () => {
        if (appState.mintakaProvider === undefined) return

        if (appState.mintakaProvider.state.type === "idle") return

        appState.mintakaProvider.state.message({ type: "abort" })
    }

    const actions = {
        play: (pos: Pos) => {
            play(pos)

            if (workerStore.autoLaunch)
                launch()
        },
        forward: (method: ForwardMethod) => {
            const result = appState.historyTree.forward(method)
            if (!result) return
            const [historyTree, entry] = result

            appState.historyTree = historyTree
            appState.boardWorker = appState.boardWorker.set(entry.pos!)

            syncSet(entry.pos)

            reconcileGameStore()
        },
        bulkForward: (method: ForwardMethod) => {
            const result = appState.historyTree.bulkForward(method)
            if (!result) return
            const [historyTree, entries] = result

            appState.historyTree = historyTree
            for (const entry of entries) {
                appState.boardWorker = appState.boardWorker.set(entry.pos)
            }

            reconcileGameStore()
        },
        backward: () => {
            if (workerStore.inComputing) return

            const result = appState.historyTree.backward()
            if (!result) return
            const [historyTree, entry] = result

            appState.historyTree = historyTree
            appState.boardWorker = appState.boardWorker.unset(entry.pos!)

            reconcileGameStore()
        },
        bulkBackward: () => {
            if (workerStore.inComputing) return

            const result = appState.historyTree.bulkBackward()
            if (!result) return
            const [historyTree, entries] = result

            appState.historyTree = historyTree
            for (const entry of entries.reverse()) {
                appState.boardWorker = appState.boardWorker.unset(entry.pos)
            }

            reconcileGameStore()
        },
        start: () => {
            setWorkerStore("autoLaunch", true)

            launch()
        },
        pause: () => {
            setWorkerStore("autoLaunch", false)
        },
        abort: () => {
            setWorkerStore("autoLaunch", false)
            abort()
        },
    }

    const loadProvider = () => {
        const providerType = unwrap(appConfigStore.providerType)
        const config = unwrap(appConfigStore.config)

        const gameState = defaultGameState()

        appState.mintakaProvider = new MintakaWorkerProvider(config, gameState)

        setWorkerStore("loadedProviderType", providerType)

        connectProvider(appState.mintakaProvider)
    }

    const connectProvider = (provider: MintakaProvider) => {
        provider.onResponse = response => {
            switch (response.type) {
                case "Begins": {
                    setWorkerStore("state", {
                        type: "began",
                        content: response.content,
                    })
                    break
                }
                case "Status": {
                    setWorkerStore("state", reconcile({
                        type: "in-computing",
                        content: response.content,
                        normEval: calculateNormEval(response.content.score),
                    }))
                    break
                }
                case "BestMove": {
                    if (response.content.position_hash !== appState.boardWorker.hashKey())
                        resolveDesync(response.content.position_hash)

                    if (response.content.best_move !== undefined)
                        play(response.content.best_move)

                    setWorkerStore("state", reconcile({
                        type: "finished",
                        content: response.content,
                        normEval: calculateNormEval(response.content.score),
                    }))

                    break
                }
            }
        }

        provider.onError = error => {
            console.log(error)
        }
    }

    loadProvider()

    return <AppContext.Provider
        value={{
            actions,

            appConfigStore,
            setAppConfigStore,

            gameStore,
            workerStore,
        }}
        children={props.children}
    />
}

function createAppConfigStore(): [AppConfig, SetStoreFunction<AppConfig>] {
    const [appConfigStore, setAppConfigStore] = makePersisted(
        createStore(defaultAppConfig()),
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

function createAppState(): AppState {
    return {
        boardWorker: new BoardWorker(defaultBoard()),
        historyTree: EmptyHistoryTree,
        evalTable: new Map(),
    }
}

function loadMintakaProvider(): MintakaProvider {
    const config = defaultConfig()
    const gameState = defaultGameState()

    return new MintakaWorkerProvider(config, gameState)
}

type UrlParams = {
    readonly history?: History | HistoryTree,
    readonly viewer?: true,
}

function parserUrlParams(): UrlParams {
    const params = new URLSearchParams(window.location.search)

    const moves = params.get("moves")
    const history = params.get("history")
    const viewer = params.get("viewer")

    return {
        history: history ? JSON.parse(history) : undefined,
        viewer: viewer === "true" ? true : undefined,
    }
}

function pushUrlParams(params: UrlParams) {
    const url = new URL(window.location.href)

    url.searchParams.set("history", JSON.stringify(params.history))

    if (params.viewer) {
        url.searchParams.set("viewer", "true")
    }

    window.history.pushState({}, "", url.toString())
}
