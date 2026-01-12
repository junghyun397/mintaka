import { createMemo, useContext } from "solid-js"
import { AppContext } from "../context"

export function StatusMessage() {
    const { runtimeState, appConfig } = useContext(AppContext)!

    const statusType = createMemo(() => runtimeState()?.type)

    const nodes = createMemo(() => 0)

    const remainingTime = createMemo(() => 0)

    const statusMessage = () => {
        const status = statusType()

        if (status === undefined)
            return ""

        if (status === "idle")
            if (appConfig.autoLaunch)
                return "Mintaka engine is waiting for your move."
            else
                return ""

        if (status === "launched" || status === "begins")
            return "Engine is started thinking..."

        if (status === "streaming")
            return `Thinking: ${nodes()}K nodes visited, up to ${remainingTime()}s remaining...`

        if (status === "aborting")
            return "Engine is stopping the current analysis..."

        return ""
    }

    return <p
        class="text-sm leading-tight text-base-content/70"
        classList={{ "animate-pulse": statusType() !== "idle" }}
    >
        {statusMessage()}
    </p>
}
