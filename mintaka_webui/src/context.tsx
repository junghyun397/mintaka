import { Accessor, createContext, createEffect, createMemo, ParentProps } from "solid-js"
import { createStore, reconcile, SetStoreFunction, unwrap } from "solid-js/store"
import { ForwardMethod } from "./domain/HistoryTree"
import { BoardDescribe, Config, HashKey, History, Pos } from "./wasm/pkg/mintaka_wasm"
import { createPersistConfigStore, defaultPersistConfig, PersistConfig } from "./stores/persist.config"
import { AppConfig } from "./stores/app.config"
import { createGameController } from "./controllers/game.controller"
import { createRuntimeController, MintakaRuntime, RequireProviderReady } from "./controllers/runtime.controller"
import { createAppState } from "./stores/app.state"
import { MintakaRuntimeState } from "./domain/mintaka.runtime"
import { flatmap } from "./utils/undefined"
import { parseUrlParams, setupUrlSync } from "./url"
import { AppGameState, buildGameStateFromHistorySource } from "./domain/rusty-renju"
import { setupThemeSync } from "./theme"
import { assertOk } from "./utils/response"
import { assertNever } from "./utils/never"

interface AppActions {
    readonly loadWorkerRuntime: () => void,
    readonly switchServerRuntime: () => void,
    readonly loadServerRuntime: () => void,
    readonly updateConfig: (config: Config) => RequireProviderReady,
    readonly restoreDefaultConfig: () => RequireProviderReady,
    readonly resetAppData: () => void,
}

interface AppSelectors {
    readonly selectNormEval: (hash: HashKey) => number | undefined,
}

interface RuntimeSelectors {
    readonly runtimeType: Accessor<MintakaRuntime["type"]>,
    readonly runtimeState: Accessor<MintakaRuntimeState | undefined>
    readonly isReady: Accessor<boolean>,
    readonly inComputing: Accessor<boolean>,
    readonly config: Accessor<Config | undefined>,
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

    const [persistConfig, setPersistConfig] = createPersistConfigStore()

    const gameController = createGameController(appState.gameState, appState.setGameState)

    const runtimeController = createRuntimeController(
        appState.mintakaRuntime, appState.setMintakaRuntime, gameController.applyBestMove,
        appState.config, appState.setConfig, appState.maxConfig, appState.setMaxConfig,
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

            assertOk(playResponse)

            runtimeController.syncPlay(appState.gameState().boardWorker.hashKey(), pos)

            if (!appConfig.autoLaunch) return

            runtimeController.launch(appState.gameState())
        },
        forward: (method) => {
            const response = gameController.forward(method)

            assertOk(response)
        },
        bulkForward: (method) => {
            const response = gameController.bulkForward(method)

            assertOk(response)
        },
        backward: () => {
            const response = gameController.backward()

            assertOk(response)
        },
        bulkBackward: () => {
            const response = gameController.bulkBackward()

            assertOk(response)
        },
        start: () => {
            const response = runtimeController.launch(appState.gameState())

            assertOk(response)

            setAppConfig("autoLaunch", true)
        },
        pause: () => {
            setAppConfig("autoLaunch", false)
        },
        abort: () => {
            const result = runtimeController.abort()

            assertOk(result)

            setAppConfig("autoLaunch", false)
        },
    }

    const appActions: AppActions = {
        loadWorkerRuntime: () => {
            setPersistConfig("providerType", "worker")

            runtimeController.loadWorkerRuntime()
        },
        switchServerRuntime: () => {
            setPersistConfig("providerType", "server")

            runtimeController.unloadRuntime()
        },
        loadServerRuntime: () => {
            if (persistConfig.providerType === "server" || persistConfig.serverConfig === undefined) return

            runtimeController.tryLoadServerRuntime(unwrap(persistConfig.serverConfig))
        },
        updateConfig: runtimeController.updateConfig,
        restoreDefaultConfig: runtimeController.restoreDefaultConfig,
        resetAppData: () => {
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
        isReady: createMemo(() => {
            const runtime = appState.mintakaRuntime()

            return runtime.type === "ready" && runtime.state.type === "idle"
        }),
        inComputing: createMemo(() => {
            const runtime = appState.mintakaRuntime()

            return runtime.type === "ready" && runtime.state.type !== "idle"
        }),
        config: appState.config,
        maxConfig: appState.maxConfig,
    }

    const gameSelectors: GameSelectors = {
        gameState: appState.gameState,
        history,
        boardDescribe,
    }

    switch (persistConfig.providerType) {
        case "worker": {
            runtimeController.loadWorkerRuntime()
            break
        }
        case "server": {
            if (persistConfig.serverConfig === undefined) break
            runtimeController.tryLoadServerRuntime(persistConfig.serverConfig)
            break
        }
        default: assertNever(persistConfig.providerType)
    }

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
