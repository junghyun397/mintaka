import { createContext, createEffect, onCleanup, onMount, ParentProps } from "solid-js"
import { MintakaProvider } from "./domain/mintaka.provider"
import { createStore, reconcile, SetStoreFunction, unwrap } from "solid-js/store"
import { buildGameStore, GameStore } from "./stores/game.store"
import { HistoryTree } from "./domain/HistoryTree"
import {
    calculateNormEval,
    defaultGameState,
    History,

} from "./wasm/pkg/mintaka_wasm"
import { AppConfig, defaultAppConfig, Theme } from "./stores/app.config.store"
import { makePersisted } from "@solid-primitives/storage"
import { MintakaWorkerProvider } from "./domain/mintaka.worker.provider"
import { createWorkerStore, WorkerStore } from "./stores/worker.store"
import { createGameController, type GameActions } from "./controllers/game.controller"
import { createDefaultAppState } from "./stores/app.state"

interface AppActions {
    clearAppConfigStore: () => void,
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
    const appState = createDefaultAppState()

    const [appConfigStore, setAppConfigStore] = createAppConfigStore()

    const [gameStore, setGameStore] = createStore(buildGameStore(appState.boardWorker, appState.historyTree, "Black"))

    const [workerStore, setWorkerStore] = createWorkerStore()

    const { play, resolveDesync, gameActions } = createGameController({
        appState,
        workerStore,
        setWorkerStore,
        gameStore,
        setGameStore,
    })

    const appActions: AppActions = {
        clearAppConfigStore: () => {
            setAppConfigStore(reconcile(defaultAppConfig()))
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
