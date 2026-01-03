import { AppContext } from "../context";
import { useContext } from "solid-js";

export function Dashboard() {
    const { workerStore } = useContext(AppContext)!

    return <div class="flex flex-col gap-4">
        <Config />
    </div>
}

function Config() {
    const { appConfigStore } = useContext(AppContext)!

    const ttSizeInMib = () =>
        Math.floor(appConfigStore.config.tt_size / 1024 / 1024)

    return <div class="flex flex-col gap-4">
        <fieldset class="fieldset">
            <legend class="fieldset-legend">Workers</legend>
            <input
                type="number" class="input"
                value={appConfigStore.config.workers}
            />
            <p class="label">CPU core usage</p>
        </fieldset>
        <fieldset class="fieldset">
            <legend class="fieldset-legend">TT Size</legend>
            <input
                type="number" class="input"
                value={ttSizeInMib()}
            />
            <p class="label">RAM usage</p>
        </fieldset>
        <fieldset class="fieldset">
            <legend class="fieldset-legend">Node Limit</legend>
            <input
                type="number" class="input"
                value={appConfigStore.config.max_nodes_in_1k}
            />
            <p class="label">Max Nodes</p>
        </fieldset>
        <fieldset class="fieldset">
            <legend class="fieldset-legend">Depth Limit</legend>
            <input
                type="number" class="input"
                value={appConfigStore.config.max_depth}
            />
            <p class="label">Max Depth</p>
        </fieldset>
        <fieldset class="fieldset">
            <legend class="fieldset-legend">Max Turn Time</legend>
            <input
                type="number" class="input"
                value={appConfigStore.config.initial_timer.turn.secs}
            />
            <p class="label">Max Depth</p>
        </fieldset>
    </div>
}
