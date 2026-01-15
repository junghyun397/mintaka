import { Accessor, createContext, createEffect, createMemo, ParentProps } from "solid-js"
import { createStore, reconcile, SetStoreFunction, unwrap } from "solid-js/store"
import { ForwardMethod } from "./domain/HistoryTree"
import { BoardDescribe, Config, HashKey, History, Pos } from "./wasm/pkg/mintaka_wasm"
import { PersistConfig, defaultPersistConfig } from "./stores/persist.config"
import { makePersisted } from "@solid-primitives/storage"
import { AppConfig } from "./stores/app.config"
import { createGameController } from "./controllers/game.controller"
import { createRuntimeController, MintakaRuntime, RequireProviderReady } from "./controllers/runtime.controller"
import { createAppState } from "./stores/app.state"
import { MintakaRuntimeState } from "./domain/mintaka.runtime"
import { flatmap } from "./utils/undefined"
import { parseUrlParams, setupUrlSync } from "./url"
import { AppGameState, buildGameStateFromHistorySource } from "./domain/rusty-renju"
import { setupThemeSync } from "./theme"

interface AppActions {
    readonly loadWorkerRuntime: () => void,
    readonly switchServerRuntime: () => void,
    readonly loadServerRuntime: () => void,
    readonly syncConfig: (config: Config) => RequireProviderReady,
    readonly resetConfig: () => RequireProviderReady,
    readonly clearAppData: () => void,
}

interface AppSelectors {
    readonly selectNormEval: (hash: HashKey) => number | undefined,
}

interface RuntimeSelectors {
    readonly runtimeType: Accessor<MintakaRuntime["type"]>,
    readonly runtimeState: Accessor<MintakaRuntimeState | undefined>
    readonly inComputing: Accessor<boolean>,
    readonly maxConfig: Accessor<Config | undefined>,
}

interface GameActions {
    readonly clear: () => void,
    readonly play: (pos: Pos) => void,
    readonly forward: (method: ForwardMethod) => void,
    readonly bulkForward: (method: ForwardMethod) => void,
    readonly backward: () => void,
    readonly bulkBackward: () => void,
    readonly start: () => void,
    readonly pause: () => void,
    readonly abort: () => void,
}

interface GameSelectors {
    readonly gameState: Accessor<AppGameState>,
    readonly history: Accessor<History>,
    readonly boardDescribe: BoardDescribe,
}

type AppContext = {
    readonly appActions: AppActions,
    readonly appSelectors: AppSelectors,
    readonly runtimeSelectors: RuntimeSelectors,

    readonly gameActions: GameActions,
    readonly gameSelectors: GameSelectors,

    readonly persistConfig: PersistConfig,
    readonly setPersistConfig: SetStoreFunction<PersistConfig>,

    readonly appConfig: AppConfig,
    readonly setAppConfig: SetStoreFunction<AppConfig>,
}

export const AppContext = createContext<AppContext>()

export function AppContextProvider(props: ParentProps) {
    const initialUrlParam = parseUrlParams()

    const appState = createAppState(
        flatmap(initialUrlParam.moves, history =>
            buildGameStateFromHistorySource({
                type: "history",
                content: history,
            }),
        ),
    )

    const [persistConfig, setPersistConfig] = makePersisted(createStore(defaultPersistConfig()))

    const gameController = createGameController(appState.gameState, appState.setGameState)

    const runtimeController = createRuntimeController(
        appState.mintakaRuntime, appState.setMintakaRuntime, gameController.applyBestMove,
    )

    const [appConfig, setAppConfig] = createStore<AppConfig>({ autoLaunch: false, openDashboard: false, viewer: initialUrlParam.viewer })

    const history = createMemo(() => appState.gameState().historyTree.toHistory())

    const [boardDescribe, setBoardDescribe] = createStore<BoardDescribe>(
        appState.gameState().boardWorker.describe(appState.gameState().historyTree.toHistory()),
    )

    createEffect(() => {
        setBoardDescribe(reconcile(
            appState.gameState().boardWorker.describe(appState.gameState().historyTree.toHistory()),
        ))
    })

    setupThemeSync(persistConfig)

    setupUrlSync(history, () => appConfig.viewer)

    const gameActions: GameActions = {
        clear: gameController.clear,
        play: (pos) => {
            if (!appState.gameState().boardWorker.isLegalMove(pos))
                return

            const playResponse = gameController.play(pos)

            if (playResponse !== "ok") return

            runtimeController.syncPlay(appState.gameState().boardWorker.hashKey(), pos)

            if (!appConfig.autoLaunch) return

            runtimeController.launch(appState.gameState().boardWorker.value(), appState.gameState().historyTree)
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
            const response =
                runtimeController.launch(appState.gameState().boardWorker.value(), appState.gameState().historyTree)

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
            const result = runtimeController.abort()

            if (result === "ok") {
                setAppConfig("autoLaunch", false)
            } else {
                throw new Error(result)
            }
        },
    }

    const appActions: AppActions = {
        loadWorkerRuntime: () => {
            setPersistConfig("providerType", "worker")

            runtimeController.loadWorkerRuntime(unwrap(persistConfig.config))
        },
        switchServerRuntime: () => {
            setPersistConfig("providerType", "server")

            runtimeController.unloadRuntime()
        },
        loadServerRuntime: () => {
            if (persistConfig.providerType === "server" || persistConfig.serverConfig === undefined) return

            runtimeController.loadServerRuntime(unwrap(persistConfig.config), unwrap(persistConfig.serverConfig))
        },
        syncConfig: (config: Config) => {
            setPersistConfig("config", reconcile(config))

            return runtimeController.syncConfig(config)
        },
        resetConfig: () => {
            const runtime = appState.mintakaRuntime()

            if (runtime.type !== "ready") return "provider-not-ready"

            setPersistConfig("config", reconcile(runtime.provider.defaultConfig))

            return runtimeController.syncConfig(runtime.provider.defaultConfig)
        },
        clearAppData: () => {
            setPersistConfig(defaultPersistConfig())
        },
    }

    const appSelectors: AppSelectors = {
        selectNormEval: (hash) => appState.normEvalTable.get(hash),
    }

    const runtimeSelectors: RuntimeSelectors = {
        runtimeType: () => appState.mintakaRuntime().type,
        runtimeState: createMemo(() => {
            const runtime = appState.mintakaRuntime()

            return runtime.type === "ready" ? runtime.state : undefined
        }),
        inComputing: createMemo(() => {
            const runtime = appState.mintakaRuntime()

            return runtime.type === "ready" && runtime.state.type !== "idle"
        }),
        maxConfig: () => {
            const runtime = appState.mintakaRuntime()

            return runtime.type === "ready" ? runtime.provider.maxConfig : undefined
        },
    }

    const gameSelectors: GameSelectors = {
        gameState: appState.gameState,
        history,
        boardDescribe,
    }

    runtimeController.loadWorkerRuntime(unwrap(persistConfig.config))

    return <AppContext.Provider
        value={{
            appActions,
            appSelectors,
            runtimeSelectors,

            gameActions,
            gameSelectors,

            persistConfig,
            setPersistConfig,

            appConfig,
            setAppConfig,
        } as AppContext}
        children={props.children}
    />
}
