import { Accessor, createContext, createEffect, createMemo, onCleanup, onMount, ParentProps } from "solid-js"
import { createStore, reconcile, SetStoreFunction, unwrap } from "solid-js/store"
import { ForwardMethod } from "./domain/HistoryTree"
import { BoardDescribe, Config, defaultConfig, Pos } from "./wasm/pkg/mintaka_wasm"
import { PersistConfig, defaultPersistConfig, Theme } from "./stores/persist.config"
import { makePersisted } from "@solid-primitives/storage"
import { AppConfig } from "./stores/app.config"
import { createGameController } from "./controllers/game.controller"
import { createProviderController, RequireProviderReady } from "./controllers/runtime.controller"
import { AppGameState, createAppState } from "./stores/app.state"
import { MintakaRuntimeState } from "./domain/mintaka.runtime"

interface AppActions {
    readonly syncConfig: (config: Config) => RequireProviderReady,
    readonly resetConfig: () => RequireProviderReady,
    readonly clearAppData: () => void,
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

    readonly boardDescribe: BoardDescribe,

    readonly persistConfig: PersistConfig,
    readonly setPersistConfig: SetStoreFunction<PersistConfig>,
    readonly maxMintakaConfig: Accessor<Config | undefined>,

    readonly appConfig: AppConfig,
    readonly setAppConfig: SetStoreFunction<AppConfig>,
}

export const AppContext = createContext<AppContext>()

export function AppContextProvider(props: ParentProps) {
    const appState = createAppState({})

    const [persistConfig, setPersistConfig] = createPersistConfigStore()

    const gameController = createGameController(appState.gameState, appState.setGameState)

    const providerController = createProviderController(
        appState.gameState, appState.mintakaRuntime, appState.setMintakaRuntime, gameController.applyBestMove,
    )

    const [appConfig, setAppConfig] = createStore<AppConfig>({ autoLaunch: false, openDashboard: false })

    const [boardDescribe, setBoardDescribe] = createStore<BoardDescribe>(
        appState.gameState().boardWorker.describe(appState.gameState().historyTree.toHistory()),
    )

    createEffect(() => {
        setBoardDescribe(reconcile(
            appState.gameState().boardWorker.describe(appState.gameState().historyTree.toHistory()),
        ))
    })

    const maxMintakaConfig = createMemo(() => appState.mintakaRuntime()?.provider.maxConfig)

    const gameActions: GameActions = {
        play: (pos) => {
            if (!appState.gameState().boardWorker.isLegalMove(pos))
                return

            const playResponse = gameController.play(pos)

            if (playResponse !== "ok") return

            providerController.syncPlay(pos)

            if (!appConfig.autoLaunch) return

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
                setAppConfig("autoLaunch", true)
            } else {
                throw new Error(response)
            }
        },
        pause: () => {
            setAppConfig("autoLaunch", false)
        },
        abort: () => {
            const result = providerController.abort()

            if (result === "ok") {
                setAppConfig("autoLaunch", false)
            } else {
                throw new Error(result)
            }
        },
    }

    const appActions: AppActions = {
        syncConfig: (config: Config) => {
            setPersistConfig("config", reconcile(config))

            return providerController.syncConfig(config)
        },
        resetConfig: () => {
            const config = appState.mintakaRuntime()?.provider.defaultConfig ?? defaultConfig()

            setPersistConfig("config", reconcile(config))

            return providerController.syncConfig(config)
        },
        clearAppData: () => {
            setPersistConfig(defaultPersistConfig())
        },
    }

    const runtimeState = () => appState.mintakaRuntime()?.state

    providerController.loadRuntime(unwrap(persistConfig))

    return <AppContext.Provider
        value={{
            appActions,
            gameActions,

            gameState: appState.gameState,
            runtimeState,

            boardDescribe,

            persistConfig,
            setPersistConfig,
            maxMintakaConfig,

            appConfig,
            setAppConfig,
        }}
        children={props.children}
    />
}

function createPersistConfigStore(): [PersistConfig, SetStoreFunction<PersistConfig>] {
    const [persistConfig, setPersistConfig] = makePersisted(
        createStore(defaultPersistConfig()),
    )

    const removeTheme = () =>
        document.documentElement.removeAttribute("data-theme")

    const applyTheme = (theme: Exclude<Theme, "system">) => {
        document.documentElement.setAttribute("data-theme", theme)
    }

    onMount(() => {
        const mediaQueryList = window.matchMedia("(prefers-color-scheme: dark)")

        const handler = () => {
            if (persistConfig.theme === "system")
                removeTheme()
        }

        mediaQueryList.addEventListener?.("change", handler)

        onCleanup(() => {
            mediaQueryList.removeEventListener?.("change", handler)
        })
    })

    createEffect(() => {
        if (persistConfig.theme === "system")
            removeTheme()
        else
            applyTheme(persistConfig.theme)
    })

    return [persistConfig, setPersistConfig]
}
