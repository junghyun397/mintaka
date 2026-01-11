import { AppGameState } from "../stores/app.state"
import { BestMove, Board, Config, defaultGameState, HashKey, History, MaybePos } from "../wasm/pkg/mintaka_wasm"
import { HistoryTree } from "../domain/HistoryTree"
import { PersistConfig } from "../stores/persist.config"
import { MintakaWorkerProvider } from "../domain/mintaka.worker.provider"
import { buildMintakaRuntime, MintakaRuntime } from "../domain/mintaka.runtime"
import { assertNever } from "../utils/never"

export type RequireProviderReady = "ok" | "provider-not-ready"
export type RequireProviderComputing = "ok" | "provider-not-launched"

interface RuntimeController {
    loadRuntime: (persistConfig: PersistConfig) => void,
    launch: (snapshot: HashKey, historyTreeSnapshot: HistoryTree) => RequireProviderReady,
    abort: () => RequireProviderComputing,
    syncPlay: (pos: MaybePos) => RequireProviderReady | "desynced",
    syncConfig: (config: Config) => RequireProviderReady,
}

export function createProviderController(
    gameState: () => AppGameState,
    mintakaRuntime: () => MintakaRuntime | undefined,
    setMintakaRuntime: (runtime: MintakaRuntime | undefined) => void,
    onBestMove: (bestMove: BestMove, historySnapShot: HistoryTree) => void,
): RuntimeController {
    const syncAll = (board: Board, history: History) => {
        const runtime = mintakaRuntime()
        if (runtime === undefined || runtime.state.type !== "idle")
            return "provider-not-ready"

        runtime.provider.command({
            type: "Sync",
            content: { board, history },
        })

        return "ok"
    }

    const subscribeRuntime = (initialRuntime: MintakaRuntime) => {
        initialRuntime.provider.subscribeResponse(response => {
            const runtime = mintakaRuntime()

            if (runtime === undefined) {
                initialRuntime.provider.dispose()
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

    return {
        loadRuntime: (persistConfig: PersistConfig) => {
            const previousRuntime = mintakaRuntime()

            if (previousRuntime !== undefined) {
                previousRuntime.provider.dispose()
            }

            const gameState = defaultGameState()

            const provider = new MintakaWorkerProvider(gameState, persistConfig.config)

            const runtimeState = buildMintakaRuntime(gameState.board.hash_key)

            const runtime: MintakaRuntime = { provider, state: runtimeState }

            subscribeRuntime(runtime)

            setMintakaRuntime(runtime)
        },
        launch: (snapshot: HashKey, historyTreeSnapshot: HistoryTree) => {
            const runtime = mintakaRuntime()
            if (runtime === undefined || runtime.state.type !== "idle")
                return "provider-not-ready"

            if (runtime.state.snapshot !== snapshot)
                syncAll(gameState().boardWorker.value(), gameState().historyTree.toHistory())

            runtime.provider.launch(snapshot, "Best")

            setMintakaRuntime({ ...runtime, state: runtime.state.launch(historyTreeSnapshot) })

            return "ok"
        },
        abort: () => {
            const runtime = mintakaRuntime()
            if (runtime === undefined || runtime.state.type === "idle")
                return "provider-not-launched"

            runtime.provider.control({ type: "abort" })

            setMintakaRuntime({ ...runtime, state: runtime.state.abort() })

            return "ok"
        },
        syncPlay: (pos: MaybePos) => {
            const runtime = mintakaRuntime()
            if (runtime === undefined || runtime.state.type !== "idle")
                return "provider-not-ready"

            if (runtime.state.snapshot !== gameState().boardWorker.hashKey())
                return "desynced"

            runtime.provider.command({ type: "Play", content: pos })

            return "ok"
        },
        syncConfig: (config: Config) => {
            const runtime = mintakaRuntime()
            if (runtime === undefined || runtime.state.type !== "idle")
                return "provider-not-ready"

            runtime.provider.command({ type: "Config", content: config })

            return "ok"
        },
    }
}
