import { BestMove, Board, Config, defaultBoard, HashKey, History, MaybePos } from "../wasm/pkg/mintaka_wasm"
import { HistoryTree } from "../domain/HistoryTree"
import { MintakaWorkerProvider } from "../domain/mintaka.worker.provider"
import { buildMintakaRuntime, MintakaRuntimeState } from "../domain/mintaka.runtime"
import { assertNever } from "../utils/never"
import { createSession, MintakaServerConfig, MintakaServerProvider } from "../domain/mintaka.server.provider"
import { MintakaProvider, MintakaProviderType } from "../domain/mintaka.provider"
import { AppGameState } from "../domain/rusty-renju"

export type RequireProviderReady = "ok" | "provider-not-ready"
export type RequireProviderComputing = "ok" | "provider-not-launched"

interface RuntimeController {
    unloadRuntime: () => void,
    loadWorkerRuntime: (config: Config) => void,
    loadServerRuntime: (config: Config, serverConfig: MintakaServerConfig) => void,
    launch: (snapshot: AppGameState) => RequireProviderReady,
    abort: () => RequireProviderComputing,
    syncPlay: (snapshot: HashKey, pos: MaybePos) => RequireProviderReady | "desynced",
    syncConfig: (config: Config) => RequireProviderReady,
}

export type MintakaReadyRuntime = {
    type: "ready",
    provider: MintakaProvider,
    state: MintakaRuntimeState,
}


export type MintakaRuntime =
    | { type: "none" }
    | {
        type: "loading",
        providerType: MintakaProviderType,
    }
    | MintakaReadyRuntime

export function createRuntimeController(
    mintakaRuntime: () => MintakaRuntime,
    setMintakaRuntime: (runtime: MintakaRuntime) => void,
    onBestMove: (bestMove: BestMove, historySnapShot: HistoryTree) => void,
): RuntimeController {
    const syncAll = (board: Board, history: History) => {
        const runtime = mintakaRuntime()
        if (runtime.type !== "ready")
            return "provider-not-ready"

        runtime.provider.command({
            type: "Sync",
            content: { board, history },
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
                    if (!(runtime!.state.type === "begins" || runtime!.state.type === "streaming")) return

                    setMintakaRuntime({ ...runtime, state: runtime.state.status(response.content) })
                    break
                }
                case "BestMove": {
                    if (runtime.state.type === "idle") return

                    onBestMove(response.content, runtime.state.historySnapshot)

                    setMintakaRuntime({ ...runtime, state: runtime.state.bestMove(response.content) })
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

    return {
        unloadRuntime,
        loadWorkerRuntime: (config: Config) => {
            unloadRuntime()

            const board: Board = defaultBoard()
            const history: History = []

            const provider = new MintakaWorkerProvider({ board, history }, config)

            const runtimeState = buildMintakaRuntime(board.hash_key)

            const runtime: MintakaRuntime = { type: "ready", provider, state: runtimeState }

            subscribeRuntime(provider)

            setMintakaRuntime(runtime)
        },
        loadServerRuntime: (config: Config, serverConfig: MintakaServerConfig) => {
            unloadRuntime()

            const board: Board = defaultBoard()
            const history: History = []

            setMintakaRuntime({ type: "loading", providerType: "server" })

            createSession(serverConfig, config, { board, history })
                .then(session => {
                    const provider = new MintakaServerProvider(serverConfig, session)

                    const runtimeState = buildMintakaRuntime(board.hash_key)

                    const runtime: MintakaRuntime = { type: "ready", provider, state: runtimeState }

                    subscribeRuntime(provider)

                    setMintakaRuntime(runtime)
                })
        },
        launch: (snapshot: AppGameState) => {
            const runtime = mintakaRuntime()
            if (runtime.type !== "ready" || runtime.state.type !== "idle")
                return "provider-not-ready"

            if (runtime.state.snapshot !== snapshot.boardWorker.hashKey())
                syncAll(snapshot.boardWorker.value(), snapshot.historyTree.toHistory())

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
        syncConfig: (config: Config) => {
            const runtime = mintakaRuntime()
            if (runtime.type === "none")
                return "ok"

            if (runtime.type !== "ready" || runtime.state.type !== "idle")
                return "provider-not-ready"

            runtime.provider.command({ type: "Config", content: config })

            return "ok"
        },
    }
}
