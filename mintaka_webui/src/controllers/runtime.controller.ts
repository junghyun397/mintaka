import type { BestMove, Board, Color, CommandResult, Config, GameState, HashKey, History, MaybePos } from "../wasm/pkg/rusty_renju_wasm"
import { defaultBoard, calculateWinRate } from "../wasm/pkg/rusty_renju_wasm"
import { HistoryTree } from "../domain/HistoryTree"
import { DefaultWorkerConfig, MaxWorkerConfig, MintakaWorkerProvider } from "../domain/mintaka.worker.provider"
import { buildMintakaRuntime, IdleState, MintakaRuntimeState } from "../domain/mintaka.runtime"
import { assertNever } from "../utils/never"
import { createSession, MintakaServerConfig, MintakaServerProvider } from "../domain/mintaka.server.provider"
import { MintakaProvider } from "../domain/mintaka.provider"
import { AppGameState } from "../domain/rusty-renju"
import { Configs, extractStatics, MintakaStatics } from "../domain/mintaka"
import { MINTAKA_CONFIG_VERSION } from "../config"
import { Mutex } from "../utils/mutex"

interface RuntimeController {
    unloadRuntime: () => void,
    loadWorkerRuntime: () => void,
    tryLoadServerRuntime: (serverConfig: MintakaServerConfig) => void,
    launch: (snapshot: AppGameState) => void,
    abort: () => void,
    syncPlay: (snapshot: HashKey, pos: MaybePos) => void,
    updateConfig: (config: Config) => void,
    restoreDefaultConfig: () => void,
}

type MintakaProviderInstance = MintakaWorkerProvider | MintakaServerProvider

export type MintakaReadyRuntime = {
    readonly type: "ready",
    readonly provider: MintakaProviderInstance,
    readonly configs: Configs,
    readonly state: MintakaRuntimeState,
    readonly statics?: MintakaStatics,
}

export type MintakaLoadingRuntime = {
    type: "loading",
    progress:
        | {
            type: "worker",
            completionRate: number,
        }
        | { type: "server" }
}

export type MintakaRuntime =
    | MintakaLoadingRuntime
    | MintakaReadyRuntime
    | undefined

export function createRuntimeController(
    mintakaRuntime: () => MintakaRuntime,
    setMintakaRuntime: (runtime: MintakaRuntime) => void,
    upsertWinRate: (hash: HashKey, color: Color, winRate: number) => void,
    handleBestMove: (bestMove: BestMove, historySnapShot: HistoryTree) => HashKey,
): RuntimeController {
    let mutex = new Mutex()

    const persistProviderConfigController = createPersistProviderConfigController()

    const syncAll = (gameState: GameState) => {
        console.log("syncall")
        const runtime = mintakaRuntime()
        if (runtime?.type !== "ready")
            return "provider-not-ready"

        runtime.provider.command({
            type: "Sync",
            content: gameState,
        })

        return "ok"
    }

    const handleCommandResult = (runtime: MintakaReadyRuntime, state: IdleState, commandResult: CommandResult) => {
        setMintakaRuntime({ ...runtime, state: state.commandResult(commandResult) })
    }

    const subscribeRuntime = (provider: MintakaProvider) => {
        provider.subscribeResponse(response => {
            const runtime = mintakaRuntime()

            if (runtime?.type !== "ready") {
                provider.dispose()
                return
            }

            switch (response.type) {
                case "Begins": {
                    if (runtime.state.type !== "launched") return

                    setMintakaRuntime({ ...runtime, state: runtime.state.begins(response.content) })
                    break
                }
                case "Status": {
                    if (!(runtime.state.type === "begins" || runtime.state.type === "streaming")) return

                    const statics = extractStatics(response.content)

                    console.log(response.content.hash, runtime.state.snapshot)

                    setMintakaRuntime({ ...runtime, state: runtime.state.status(response.content), statics })

                    upsertWinRate(response.content.hash, runtime.state.historySnapshot.playerColor, calculateWinRate(response.content.score))
                    break
                }
                case "BestMove": {
                    if (runtime.state.type === "idle") return

                    const statics = extractStatics(response.content)

                    const afterHash = handleBestMove(response.content, runtime.state.historySnapshot)

                    console.log(response.content.position_hash, runtime.state.snapshot)

                    setMintakaRuntime({ ...runtime, state: runtime.state.bestMove(response.content), statics })

                    const snapshotColor = runtime.state.historySnapshot.playerColor

                    upsertWinRate(response.content.position_hash, snapshotColor, calculateWinRate(response.content.score))
                    upsertWinRate(afterHash, snapshotColor, calculateWinRate(response.content.score))
                    break
                }
                case "Error": {
                    console.error(response.content)
                    break
                }
                default: assertNever(response)
            }
        })
    }

    const unloadRuntime = () => {
        const runtime = mintakaRuntime()

        if (runtime?.type === "ready") {
            runtime.provider.dispose()
        }

        setMintakaRuntime(undefined)
    }

    const loadServerRuntime = async (serverConfig: MintakaServerConfig) => {
        const board: Board = defaultBoard()
        const history: History = []

        let storedConfig = persistProviderConfigController.load({ type: "server", config: serverConfig })

        setMintakaRuntime({ type: "loading", progress: { type: "server" } })

        const session = await createSession(serverConfig, { board, history }, storedConfig)

        const provider = new MintakaServerProvider(serverConfig, session)

        const configs = await provider.configs()

        const runtimeState = buildMintakaRuntime(board.hash_key)
        const runtime: MintakaRuntime = { type: "ready", provider, state: runtimeState, configs }

        subscribeRuntime(provider)

        setMintakaRuntime(runtime)
    }

    const isReadyIdleRuntime = (runtime: MintakaRuntime): runtime is MintakaReadyRuntime & { state: IdleState } =>
        runtime !== undefined && runtime.type === "ready" && runtime.state.type === "idle"

    const storeConfig = async (runtime: MintakaReadyRuntime & { state: IdleState }, config: Config) => {
        const commandResult = await runtime.provider.command({ type: "Config", content: config })

        handleCommandResult(runtime, runtime.state, commandResult)

        setMintakaRuntime({ ...runtime, configs: { ...runtime.configs, config } })

        switch (runtime.provider.type) {
            case "worker": {
                persistProviderConfigController.set({ type: "worker" }, config)
                break
            }
            case "server": {
                persistProviderConfigController.set({ type: "server", config: runtime.provider.serverConfig }, config)
                break
            }
            default: assertNever(runtime.provider)
        }
    }

    return {
        unloadRuntime,
        loadWorkerRuntime: () => {
            unloadRuntime()

            const board: Board = defaultBoard()
            const history: History = []

            let config = persistProviderConfigController.load({ type: "worker" }) ?? DefaultWorkerConfig
            const configs = { default_config: DefaultWorkerConfig, max_config: MaxWorkerConfig, config: config }

            const provider = new MintakaWorkerProvider({ board, history }, config)

            const runtimeState = buildMintakaRuntime(board.hash_key)
            const runtime: MintakaRuntime = { type: "ready", provider, state: runtimeState, configs }

            subscribeRuntime(provider)

            setMintakaRuntime(runtime)
        },
        tryLoadServerRuntime: (serverConfig: MintakaServerConfig) => {
            unloadRuntime()

            void loadServerRuntime(serverConfig)
        },
        launch: (snapshot: AppGameState) => {
            void mutex.run(async () => {
                const runtime = mintakaRuntime()

                if (runtime?.type !== "ready" || runtime?.state.type !== "idle")
                    return

                if (runtime.state.snapshot !== snapshot.boardWorker.hashKey())
                    syncAll({ board: snapshot.boardWorker.value(), history: snapshot.historyTree.toHistory() })

                setMintakaRuntime({ ...runtime, state: runtime.state.launch(snapshot.historyTree) })

                await runtime.provider.launch(snapshot.boardWorker.hashKey(), "Best")
            })
        },
        abort: () => {
            const runtime = mintakaRuntime()

            if (runtime?.type !== "ready" || runtime?.state.type === "idle")
                return

            runtime.provider.control({ type: "abort" })

            setMintakaRuntime({ ...runtime, state: runtime.state.abort() })
        },
        syncPlay: (snapshot: HashKey, pos: MaybePos) => {
            void mutex.run(async () => {
                const runtime = mintakaRuntime()

                if (runtime === undefined)
                    return

                if (runtime?.type !== "ready" || runtime?.state.type !== "idle")
                    return

                if (runtime.state.snapshot !== snapshot)
                    return

                const commandResult = await runtime.provider.command({ type: "Play", content: pos })

                handleCommandResult(runtime, runtime.state, commandResult)
            })
        },
        updateConfig: (config: Config) => {
            void mutex.run(async () => {
                const runtime = mintakaRuntime()

                if (!isReadyIdleRuntime(runtime))
                    return

                await storeConfig(runtime, config)
            })
        },
        restoreDefaultConfig: () => {
            void mutex.run(async () => {
                const runtime = mintakaRuntime()

                if (!isReadyIdleRuntime(runtime))
                    return

                await storeConfig(runtime, runtime.configs.default_config)
            })
        },
    }
}

type PersistProviderConfigSource = { type: "worker" } | { type: "server", config: MintakaServerConfig }

function buildPersistProviderConfigLabel(source: PersistProviderConfigSource): string {
    const id = source.type === "worker" ? "local" : source.config.address

    return "provider-config-" + source.type + "-" + id + MINTAKA_CONFIG_VERSION
}

function createPersistProviderConfigController(): {
    load: (source: PersistProviderConfigSource) => Config | undefined,
    set: (source: PersistProviderConfigSource, config: Config) => void,
} {
    return {
        load: (source): Config | undefined => {
            const configString = localStorage.getItem(buildPersistProviderConfigLabel(source))

            return configString === null ? undefined : JSON.parse(configString)
        },
        set: (source, config) => {
            localStorage.setItem(buildPersistProviderConfigLabel(source), JSON.stringify(config))
        },
    }
}
