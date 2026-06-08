import { useContext } from "solid-js"
import { AppContext } from "../context"
import { formatNodes } from "../domain/mintaka"
import { flatmap } from "../utils/undefined"

export function StatusMessage() {
    const { appSettings, runtimeSelectors, gameSelectors } = useContext(AppContext)!

    const nodes = () =>
        flatmap(runtimeSelectors.statics()?.totalNodesIn1k, valid => formatNodes(valid))

    const statusMessage = () => {
        const status = runtimeSelectors.runtimeState()?.type

        if (gameSelectors.finished()) {
            if (gameSelectors.boardDescribe.winner !== undefined)
                return `${gameSelectors.boardDescribe.winner.color} wins.`

            return "Game finished."
        }

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
            return `Thinking: ${nodes()} nodes visited...`

        if (status === "aborting")
            return "Engine is stopping the current analysis..."

        return ""
    }

    return <p class="text-sm leading-tight text-base-content/70">
        {statusMessage()}
    </p>
}
