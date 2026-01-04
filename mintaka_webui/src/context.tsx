import { createContext, createEffect, createMemo, onCleanup, onMount, ParentProps } from "solid-js"
import { MintakaProvider } from "./domain/mintaka.provider"
import { createStore, reconcile, SetStoreFunction, unwrap } from "solid-js/store"
import { buildGameStore, GameStore } from "./stores/game.store"
import { ForwardMethod, HistoryTree } from "./domain/HistoryTree"
import { calculateNormEval, defaultGameState, History, Pos } from "./wasm/pkg/mintaka_wasm"
import { AppConfig, defaultAppConfig, Theme } from "./stores/app.config.store"
import { makePersisted } from "@solid-primitives/storage"
import { MintakaWorkerProvider } from "./domain/mintaka.worker.provider"
import { createWorkerStore, WorkerStore } from "./stores/worker.store"
import { createGameController } from "./controllers/game.controller"
import { createProviderController } from "./controllers/provider.controller"
import { createAppState } from "./stores/app.state"

interface AppActions {
    clearAppConfigStore: () => void,
}

interface GameActions {
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
    readonly appActions: AppActions,
    readonly gameActions: GameActions,

    readonly appConfigStore: AppConfig,
    readonly setAppConfigStore: SetStoreFunction<AppConfig>,

    readonly gameStore: GameStore,
    readonly workerStore: WorkerStore,
}

export const AppContext = createContext<AppContext>()

export function AppContextProvider(props: ParentProps) {
    const appState = createAppState({})

    const [appConfigStore, setAppConfigStore] = createAppConfigStore()

    const [gameStore, setGameStore] = createStore(buildGameStore(appState.gameState().boardWorker, appState.gameState().historyTree))

    const [workerStore, setWorkerStore] = createWorkerStore()

    const gameController = createGameController(appState.gameState, appState.setGameState)

    const providerController = createProviderController(appState.gameState, appState.mintakaProvider)

    const gameStateMemo = createMemo(() => {
        const gameState = appState.gameState()

        return buildGameStore(gameState.boardWorker, gameState.historyTree)
    })

    createEffect(() => setGameStore(reconcile(gameStateMemo())))

    const gameActions: GameActions = {
        play: (pos) => {
            gameController.play(pos)

            if (!workerStore.inComputing && workerStore.autoLaunch)
                providerController.launch()
        },
        forward: (method) => {
            gameController.forward(method)
        },
        bulkForward: (method) => {
            gameController.bulkForward(method)
        },
        backward: () => {
            gameController.backward()
        },
        bulkBackward: () => {
            gameController.bulkBackward()
        },
        start: () => {
            const result = providerController.launch()

            if (result === "ok") {
                setWorkerStore("inComputing", true)
                setWorkerStore("autoLaunch", true)
            }
        },
        pause: () => {
            setWorkerStore("autoLaunch", false)
        },
        abort: () => {
            const result = providerController.abort()

            if (result === "ok")
                setWorkerStore("autoLaunch", false)
        },
    }

    const appActions: AppActions = {
        clearAppConfigStore: () => {
            setAppConfigStore(reconcile(defaultAppConfig()))
        },
    }

    const loadProvider = () => {
        const providerType = unwrap(appConfigStore.providerType)
        const config = unwrap(appConfigStore.config)

        const gameState = defaultGameState()

        const provider = new MintakaWorkerProvider(config, gameState)
        appState.setMintakaProvider(provider)

        setWorkerStore("loadedProviderType", providerType)

        connectProvider(provider)
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
                    if (response.content.position_hash !== appState.gameState().boardWorker.hashKey())
                        providerController.syncAll()

                    gameController.play(response.content.best_move!)

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
            appActions,
            gameActions,

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
        const mediaQueryList = window.matchMedia("(prefers-color-scheme: dark)")

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
