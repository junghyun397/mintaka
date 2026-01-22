import { createMemo, useContext } from "solid-js"
import { AppContext } from "../context"
import { durationSeconds, formatNodes } from "../domain/mintaka"
import { flatmap } from "../utils/undefined"

export function StatusMessage() {
    const { appSettings, runtimeSelectors } = useContext(AppContext)!

    const nodes = () =>
        flatmap(runtimeSelectors.statics()?.totalNodesIn1k, valid => formatNodes(valid))

    const remainingTime = createMemo(() => {
        const state = runtimeSelectors.runtimeState()

        const time = state?.type !== undefined && state.type !== "idle" && state.type !== "launched"
            ? state.resource !== undefined
                ? state.type === "streaming"
                    ? durationSeconds(state.resource.time) - durationSeconds(state.lastStatus.time_elapsed)
                    : durationSeconds(state.resource.time)
                :undefined
            : undefined

        return flatmap(time, valid => Math.round(valid))
    })

    const statusMessage = () => {
        const status = runtimeSelectors.runtimeState()?.type

        if (status === undefined)
            return ""

        if (status === "idle")
            if (appSettings.launch)
                return "Mintaka engine is waiting for your move."
            else
                return ""

        if (status === "launched" || status === "begins")
            return "Engine is started thinking..."

        if (status === "streaming")
            return `Thinking: ${nodes()} nodes visited, up to ${remainingTime()}s remaining...`

        if (status === "aborting")
            return "Engine is stopping the current analysis..."

        return ""
    }

    return <p class="text-sm leading-tight text-base-content/70">
        {statusMessage()}
    </p>
}
