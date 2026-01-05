import { Accessor, createContext, createEffect, onCleanup, onMount, ParentProps } from "solid-js"
import { createStore, SetStoreFunction, unwrap } from "solid-js/store"
import { ForwardMethod, HistoryTree } from "./domain/HistoryTree"
import { BestMove, Pos } from "./wasm/pkg/mintaka_wasm"
import { AppConfig, defaultAppConfig, Theme } from "./stores/app.config.store"
import { makePersisted } from "@solid-primitives/storage"
import { AppStore } from "./stores/appStore"
import { createGameController } from "./controllers/game.controller"
import { createProviderController } from "./controllers/runtime.controller"
import { AppGameState, createAppState } from "./stores/app.state"
import { MintakaRuntimeState } from "./domain/mintaka.runtime"

interface AppActions {
    readonly clearAppConfigStore: () => void,
}

interface GameActions {
    readonly play: (pos: Pos) => void,
    readonly forward: (method: ForwardMethod) => void,
    readonly bulkForward: (method: ForwardMethod) => void,
    readonly backward: () => void,
    readonly bulkBackward: () => void,
    readonly start: () => void,
    readonly pause: () => void,
    readonly abort: () => void,
}

type AppContext = {
    readonly appActions: AppActions,
    readonly gameActions: GameActions,

    readonly gameState: Accessor<AppGameState>,
    readonly runtimeState: Accessor<MintakaRuntimeState | undefined>

    readonly appConfigStore: AppConfig,
    readonly setAppConfigStore: SetStoreFunction<AppConfig>,

    readonly appStore: AppStore,
}

export const AppContext = createContext<AppContext>()

export function AppContextProvider(props: ParentProps) {
    const appState = createAppState({})

    const [appConfigStore, setAppConfigStore] = createAppConfigStore()

    const [appStore, setAppStore] = createStore<AppStore>({ autoLaunch: false })

    const gameController = createGameController(appState.gameState, appState.setGameState)

    const providerController = createProviderController(appState.gameState, appState.mintakaRuntime, appState.setMintakaRuntime, gameController.applyBestMove)

    const gameActions: GameActions = {
        play: (pos) => {
            if (!appState.gameState().boardWorker.isLegalMove(pos))
                return

            const playResponse = gameController.play(pos)

            if (playResponse !== "ok") return

            providerController.syncPlay(pos)

            if (!appStore.autoLaunch) return

            providerController.launch(appState.gameState().boardWorker.hashKey(), appState.gameState().historyTree)
        },
        forward: (method) => {
            const response = gameController.forward(method)

            if (response !== "ok")
                throw new Error(response)
        },
        bulkForward: (method) => {
            const response = gameController.bulkForward(method)

            if (response !== "ok")
                throw new Error(response)
        },
        backward: () => {
            const response = gameController.backward()

            if (response !== "ok")
                throw new Error(response)
        },
        bulkBackward: () => {
            const response = gameController.bulkBackward()

            if (response !== "ok")
                throw new Error(response)
        },
        start: () => {
            const response = providerController.launch(appState.gameState().boardWorker.hashKey(), appState.gameState().historyTree)

            if (response === "ok") {
                setAppStore("autoLaunch", true)
            } else {
                throw new Error(response)
            }
        },
        pause: () => {
            setAppStore("autoLaunch", false)
        },
        abort: () => {
            const result = providerController.abort()

            if (result === "ok") {
                setAppStore("autoLaunch", false)
            } else {
                throw new Error(result)
            }
        },
    }

    const appActions: AppActions = {
        clearAppConfigStore: () => {
            setAppConfigStore(defaultAppConfig())
        },
    }

    const runtimeState = () => appState.mintakaRuntime()?.state

    providerController.loadRuntime(unwrap(appConfigStore))

    return <AppContext.Provider
        value={{
            appActions,
            gameActions,

            gameState: appState.gameState,
            runtimeState,

            appConfigStore,
            setAppConfigStore,

            appStore,
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
