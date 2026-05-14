import { createContext, createEffect, createMemo, type Accessor, type ParentProps } from "solid-js"
import { createStore, reconcile, unwrap, type SetStoreFunction, type Store } from "solid-js/store"
import type { ForwardMethod } from "./domain/HistoryTree"
import type { BoardDescribe, Color, Config, HashKey, History, Pos } from "./wasm/pkg/rusty_renju_wasm"
import { createPersistConfigStore, defaultPersistConfig, type PersistConfig } from "./stores/persist.config"
import { createAppSettingsStore, type AppSettings } from "./stores/app.settings"
import { createGameController } from "./controllers/game.controller"
import { createRuntimeController, type MintakaRuntimeType } from "./controllers/runtime.controller"
import { createAppState } from "./stores/app.state"
import type { MintakaRuntimeState } from "./domain/mintaka.runtime"
import { filter, flatmap } from "./utils/undefined"
import { parseUrlParams, setupUrlSync } from "./url"
import { buildGameStateFromHistorySource, emptyAppGameState, type AppGameState } from "./domain/rusty-renju"
import { setupThemeSync } from "./theme"
import { assertOk } from "./utils/response"
import { assertNever } from "./utils/never"
import type { Configs, MintakaStatics } from "./domain/mintaka"
import { WEB_WORKER_READY } from "./config"

interface AppActions {
    readonly loadWorkerRuntime: () => void,
    readonly switchServerRuntime: () => void,
    readonly loadServerRuntime: () => void,
    readonly updateConfig: (config: Config) => void,
    readonly restoreDefaultConfig: () => void,
    readonly resetAppData: () => void,
}

interface AppSelectors {
    readonly winRateTable: Store<Record<HashKey, number>>,
}

interface RuntimeSelectors {
    readonly runtimeType: Accessor<MintakaRuntimeType | undefined>,
    readonly runtimeState: Accessor<MintakaRuntimeState | undefined>
    readonly isReady: Accessor<boolean>,
    readonly inComputing: Accessor<boolean>,
    readonly configs: Accessor<Configs | undefined>,
    readonly statics: Accessor<MintakaStatics | undefined>,
    readonly version: Accessor<string | undefined>,
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
    readonly finished: Accessor<boolean>,
}

type AppContext = {
    readonly appActions: AppActions,
    readonly appSelectors: AppSelectors,
    readonly runtimeSelectors: RuntimeSelectors,

    readonly gameActions: GameActions,
    readonly gameSelectors: GameSelectors,

    readonly persistConfig: PersistConfig,
    readonly setPersistConfig: SetStoreFunction<PersistConfig>,

    readonly appSettings: AppSettings,
    readonly setAppSettings: SetStoreFunction<AppSettings>,
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
        ) ?? emptyAppGameState(),
    )

    const [persistConfig, setPersistConfig] = createPersistConfigStore()

    const gameController = createGameController(appState.gameState, appState.setGameState)

    const setWinRateTable = (hash: HashKey, color: Color, winRate: number) => {
        appState.setWinRateTable(hash, color === "Black" ? winRate : -winRate)
    }

    const runtimeController = createRuntimeController(
        appState.mintakaRuntime, appState.setMintakaRuntime,
        setWinRateTable,
        gameController.applyBestMove,
    )

    const [appSettings, setAppSettings] = createAppSettingsStore(initialUrlParam)

    const history = createMemo(() => appState.gameState().historyTree.toHistory())

    const [boardDescribe, setBoardDescribe] = createStore<BoardDescribe>(
        appState.gameState().boardWorker.describe(appState.gameState().historyTree.toHistory()),
    )

    const finished = createMemo(() =>
        boardDescribe.winner !== undefined,
    )

    createEffect(() => {
        setBoardDescribe(reconcile(
            appState.gameState().boardWorker.describe(appState.gameState().historyTree.toHistory()),
        ))
    })

    setupThemeSync(persistConfig)

    setupUrlSync(history, () => appSettings.viewer)

    const continuePlay = async (beforeHash: HashKey, pos: Pos) => {
        await runtimeController.syncPlay(beforeHash, pos)

        if (finished() || !appSettings.launch) return

        runtimeController.launch(appState.gameState())
    }

    const gameActions: GameActions = {
        clear: gameController.clear,
        play: (pos) => {
            if (gameSelectors.boardDescribe.winner !== undefined)
                return

            if (!appState.gameState().boardWorker.isLegalMove(pos))
                return

            const beforeHash = appState.gameState().boardWorker.hashKey()

            const playResponse = gameController.play(pos)

            assertOk(playResponse)

            void continuePlay(beforeHash, pos)
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
            runtimeController.launch(appState.gameState())

            setAppSettings("launch", true)
            setAppSettings("viewer", false)
        },
        pause: () => {
            setAppSettings("launch", false)
        },
        abort: () => {
            runtimeController.abort()

            setAppSettings("launch", false)
        },
    }

    const appActions: AppActions = {
        loadWorkerRuntime: () => {
            if (!WEB_WORKER_READY) return

            setPersistConfig("selectedProviderType", "worker")

            runtimeController.loadWorkerRuntime()
        },
        switchServerRuntime: () => {
            setPersistConfig("selectedProviderType", "server")

            runtimeController.unloadRuntime()
        },
        loadServerRuntime: () => {
            if (persistConfig.selectedProviderType !== "server" || persistConfig.serverConfig === undefined) return

            runtimeController.tryLoadServerRuntime(unwrap(persistConfig.serverConfig))
        },
        updateConfig: runtimeController.updateConfig,
        restoreDefaultConfig: runtimeController.restoreDefaultConfig,
        resetAppData: () => {
            setPersistConfig(defaultPersistConfig())
        },
    }

    const appSelectors: AppSelectors = {
        winRateTable: appState.winRateTable,
    }

    const runtimeSelectors: RuntimeSelectors = {
        runtimeType: createMemo(() => appState.mintakaRuntime()?.type),
        runtimeState: createMemo(() =>
            flatmap(appState.mintakaRuntime(), runtime => runtime.type === "ready" ? runtime.state : undefined),
        ),
        isReady: createMemo(() =>
            flatmap(appState.mintakaRuntime(), runtime => runtime.type === "ready" && runtime.state.type === "idle" && !finished()) ?? false,
        ),
        inComputing: createMemo(() =>
            flatmap(appState.mintakaRuntime(), runtime => runtime.type === "ready" && runtime.state.type !== "idle") ?? false,
        ),
        configs: createMemo(() =>
            filter(appState.mintakaRuntime(), runtime => runtime.type === "ready")?.configs,
        ),
        statics: createMemo(() =>
            filter(appState.mintakaRuntime(), runtime => runtime.type === "ready")?.statics,
        ),
        version: createMemo(() =>
            filter(appState.mintakaRuntime(), runtime => runtime.type === "ready")?.provider.version,
        ),
    }

    const gameSelectors: GameSelectors = {
        gameState: appState.gameState,
        history,
        boardDescribe,
        finished,
    }

    if (!appSettings.viewer) {
        switch (persistConfig.selectedProviderType) {
            case "worker": {
                runtimeController.loadWorkerRuntime()
                break
            }
            case "server": {
                if (persistConfig.serverConfig !== undefined)
                    runtimeController.tryLoadServerRuntime(persistConfig.serverConfig)
                break
            }
            default: assertNever(persistConfig.selectedProviderType)
        }
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

            appSettings,
            setAppSettings,
        } as AppContext}
        children={props.children}
    />
}
