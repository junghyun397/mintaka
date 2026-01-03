import { useContext } from "solid-js"
import { AppContext } from "../context"

export function StatusMessage() {
    const { workerStore } = useContext(AppContext)!

    const statusMessage = () => {
        if (workerStore.inComputing)
            return "Mintaka engine is thinking now..."

        if (workerStore.loadedProviderType !== undefined && workerStore.autoLaunch)
            return "Mintaka engine is waiting for your move."

        // "Downloading and compiling mintaka engine..."

        return ""
    }

    return <p class="text-sm leading-tight text-base-content/70">
        {statusMessage()}
    </p>
}
