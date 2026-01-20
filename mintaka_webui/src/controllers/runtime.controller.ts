import type { BestMove, Board, Config, GameState, HashKey, History, MaybePos } from "../wasm/pkg/rusty_renju_wasm"
import { defaultBoard } from "../wasm/pkg/rusty_renju_wasm"
import { HistoryTree } from "../domain/HistoryTree"
import { DefaultWorkerConfig, MaxWorkerConfig, MintakaWorkerProvider } from "../domain/mintaka.worker.provider"
import { buildMintakaRuntime, MintakaRuntimeState } from "../domain/mintaka.runtime"
import { assertNever } from "../utils/never"
import { createSession, fetchConfigs, MintakaServerConfig, MintakaServerProvider } from "../domain/mintaka.server.provider"
import { MintakaProvider, MintakaProviderType } from "../domain/mintaka.provider"
import { AppGameState } from "../domain/rusty-renju"
import { Configs, extractStatics, MintakaStatics } from "../domain/mintaka"

export type RequireProviderReady = "ok" | "provider-not-ready"
export type RequireProviderComputing = "ok" | "provider-not-launched"

interface RuntimeController {
    unloadRuntime: () => void,
    loadWorkerRuntime: () => void,
    tryLoadServerRuntime: (serverConfig: MintakaServerConfig) => void,
    launch: (snapshot: AppGameState) => RequireProviderReady,
    abort: () => RequireProviderComputing,
    syncPlay: (snapshot: HashKey, pos: MaybePos) => RequireProviderReady | "desynced",
    updateConfig: (config: Config) => RequireProviderReady,
    restoreDefaultConfig: () => RequireProviderReady,
}

export type MintakaReadyRuntime = {
    readonly type: "ready",
    readonly provider: MintakaProvider,
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
    | { type: "none" }
    | MintakaLoadingRuntime
    | MintakaReadyRuntime

export function createRuntimeController(
    mintakaRuntime: () => MintakaRuntime,
    setMintakaRuntime: (runtime: MintakaRuntime) => void,
    onBestMove: (bestMove: BestMove, historySnapShot: HistoryTree) => void,
): RuntimeController {
    const persistProviderConfigController = createPersistProviderConfigController()

    const syncAll = (gameState: GameState) => {
        const runtime = mintakaRuntime()
        if (runtime.type !== "ready")
            return "provider-not-ready"

        runtime.provider.command({
            type: "Sync",
            content: gameState,
        })

        return "ok"
    }

    const subscribeRuntime = (provider: MintakaProvider) => {
        provider.subscribeResponse(response => {
            const runtime = mintakaRuntime()

            if (runtime.type !== "ready") {
                provider.dispose()
                return
            }

            switch (response.type) {
                case "CommandResult": {
                    if (runtime.state.type !== "idle") return

                    setMintakaRuntime({ ...runtime, state: runtime.state.commandResult(response.content) })
                    break
                }
                case "Begins": {
                    if (runtime.state.type !== "launched") return

                    setMintakaRuntime({ ...runtime, state: runtime.state.begins(response.content) })
                    break
                }
                case "Status": {
                    if (!(runtime.state.type === "begins" || runtime.state.type === "streaming")) return

                    const statics = extractStatics(response.content)

                    setMintakaRuntime({ ...runtime, state: runtime.state.status(response.content), statics })
                    break
                }
                case "BestMove": {
                    if (runtime.state.type === "idle") return

                    const statics = extractStatics(response.content)

                    onBestMove(response.content, runtime.state.historySnapshot)

                    setMintakaRuntime({ ...runtime, state: runtime.state.bestMove(response.content), statics })
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
        const previousRuntime = mintakaRuntime()

        if (previousRuntime.type === "ready") {
            previousRuntime.provider.dispose()
        }

        setMintakaRuntime({ type: "none" })
    }

    const loadServerRuntime = async (serverConfig: MintakaServerConfig) => {
        const board: Board = defaultBoard()
        const history: History = []

        let storedConfig = persistProviderConfigController.load({ type: "server", config: serverConfig })

        setMintakaRuntime({ type: "loading", progress: { type: "server" } })

        const session = await createSession(serverConfig, { board, history }, storedConfig)
        const configs = await fetchConfigs(serverConfig, session)

        const provider = new MintakaServerProvider(serverConfig, session)

        const runtimeState = buildMintakaRuntime(board.hash_key)
        const runtime: MintakaRuntime = { type: "ready", provider, state: runtimeState, configs }

        subscribeRuntime(provider)

        setMintakaRuntime(runtime)
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
            const runtime = mintakaRuntime()
            if (runtime.type !== "ready" || runtime.state.type !== "idle")
                return "provider-not-ready"

            if (runtime.state.snapshot !== snapshot.boardWorker.hashKey())
                syncAll({ board: snapshot.boardWorker.value(), history: snapshot.historyTree.toHistory() })

            runtime.provider.launch(snapshot.boardWorker.hashKey(), "Best")

            setMintakaRuntime({ ...runtime, state: runtime.state.launch(snapshot.historyTree) })

            return "ok"
        },
        abort: () => {
            const runtime = mintakaRuntime()
            if (runtime.type !== "ready" || runtime.state.type === "idle")
                return "provider-not-launched"

            runtime.provider.control({ type: "abort" })

            setMintakaRuntime({ ...runtime, state: runtime.state.abort() })

            return "ok"
        },
        syncPlay: (snapshot: HashKey, pos: MaybePos) => {
            const runtime = mintakaRuntime()
            if (runtime.type === "none")
                return "ok"

            if (runtime.type !== "ready" || runtime.state.type !== "idle")
                return "provider-not-ready"

            if (runtime.state.snapshot !== snapshot)
                return "desynced"

            runtime.provider.command({ type: "Play", content: pos })

            return "ok"
        },
        updateConfig: (config: Config) => {
            const runtime = mintakaRuntime()
            if (runtime.type === "none")
                return "ok"

            if (runtime.type !== "ready" || runtime.state.type !== "idle")
                return "provider-not-ready"

            runtime.provider.command({ type: "Config", content: config })

            setMintakaRuntime({ ...runtime, configs: { ...runtime.configs, config } })
            persistProviderConfigController.set(runtime.provider.type, runtime.provider.storageKey, config)

            return "ok"
        },
        restoreDefaultConfig: () => {
            const runtime = mintakaRuntime()
            if (runtime.type === "none")
                return "ok"

            if (runtime.type !== "ready" || runtime.state.type !== "idle")
                return "provider-not-ready"

            const config = runtime.configs.default_config

            runtime.provider.command({ type: "Config", content: config })

            setMintakaRuntime({ ...runtime, configs: { ...runtime.configs, config } })
            persistProviderConfigController.set(runtime.provider.type, runtime.provider.storageKey, config)

            return "ok"
        },
    }
}

function buildPersistProviderConfigLabel(providerType: MintakaProviderType, providerId: string): string {
    return "provider-config-" + providerType + "-" + providerId + "-v1"
}

function createPersistProviderConfigController(): {
    load: (source: { type: "worker" } | { type: "server", config: MintakaServerConfig }) => Config | undefined,
    set: (providerType: MintakaProviderType, providerId: string, config: Config) => void,
} {
    return {
        load: (source): Config | undefined => {
            const id = source.type === "worker" ? "local" : source.config.address

            const configString = localStorage.getItem(buildPersistProviderConfigLabel(source.type, id))

            return configString === null ? undefined : JSON.parse(configString)
        },
        set: (providerType: MintakaProviderType, providerId: string, config: Config) => {
            localStorage.setItem(buildPersistProviderConfigLabel(providerType, providerId), JSON.stringify(config))
        },
    }
}
