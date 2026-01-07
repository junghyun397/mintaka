import { createMemo, useContext } from "solid-js"
import { AppContext } from "../context"

export function StatusMessage() {
    const { runtimeState, appConfig } = useContext(AppContext)!

    const inComputing = createMemo(() => runtimeState()?.type !== "idle")

    const statusMessage = () => {
        if (inComputing())
            return "Mintaka engine is thinking now..."

        if (runtimeState() !== undefined && appConfig.autoLaunch)
            return "Mintaka engine is waiting for your move."

        // "Downloading and compiling mintaka engine..."

        return ""
    }

    return <p
        class="text-sm leading-tight text-base-content/70"
        classList={{ "animate-pulse": inComputing() }}
    >
        {statusMessage()}
    </p>
}
