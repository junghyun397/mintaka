import { AppGameState } from "../stores/app.state"
import { MintakaProvider, MintakaProviderLaunchResult } from "../domain/mintaka.provider"
import { MaybePos } from "../wasm/pkg/mintaka_wasm"

interface ProviderController {
    launch: () => "provider-not-ready" | MintakaProviderLaunchResult,
    abort: () => "ok" | "provider-not-launched",
    syncSet: (pos: MaybePos) => "ok" | "provider-not-ready",
    syncAll: () => "ok" | "provider-not-ready",
}

export function createProviderController(
    gameState: () => AppGameState,
    mintakaProvider: () => MintakaProvider | undefined,
): ProviderController {
    return {
        launch: () => {
            const provider = mintakaProvider()
            if (provider === undefined) return "provider-not-ready"

            if (provider.state.type === "in_computing") return "provider-not-ready"

            return provider.state.launch(gameState().boardWorker.hashKey(), "Best")
        },
        abort: () => {
            const provider = mintakaProvider()
            if (provider === undefined || provider.state.type === "idle")
                return "provider-not-launched"

            provider.state.message({ type: "abort" })

            return "ok"
        },
        syncSet: (pos: MaybePos) => {
            const provider = mintakaProvider()
            if (provider === undefined || provider.state.type !== "idle")
                return "provider-not-ready"

            provider.state.command({ type: "Play", content: pos })

            return "ok"
        },
        syncAll: () => {
            const provider = mintakaProvider()
            if (provider === undefined || provider.state.type !== "idle")
                return "provider-not-ready"

            // todo: implement

            return "ok"
        },
    }
}
