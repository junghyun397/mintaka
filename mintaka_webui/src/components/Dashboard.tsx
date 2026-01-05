import { AppContext } from "../context"
import { useContext } from "solid-js"

export function Dashboard() {
    const { appConfigStore, setAppConfigStore } = useContext(AppContext)!

    const closeDashboard = () => {
        // @ts-ignore
        setAppConfigStore("openDashboard", false)
    }

    return (
        <div
            class="fixed inset-0 z-999"
            classList={{ "pointer-events-none": !appConfigStore.openDashboard }}
        >
            <button
                class="absolute inset-0 bg-black/40 transition-opacity"
                classList={{
                    "opacity-0": !appConfigStore.openDashboard,
                    "opacity-100": appConfigStore.openDashboard,
                }}
                onClick={closeDashboard}
            />
            <aside
                class="absolute top-0 left-0 h-full w-80 max-w-[85vw] overflow-y-auto bg-base-200 shadow-lg transition-transform"
                classList={{
                    "-translate-x-full": !appConfigStore.openDashboard,
                    "translate-x-0": appConfigStore.openDashboard,
                }}
            >
                <button
                    class="btn absolute top-2 right-2 btn-sm btn-primary"
                    onClick={closeDashboard}
                >X</button>
                <div class="flex flex-col gap-4 p-4">
                    <Overview />
                    <Config />
                    <DangerZone />
                </div>
            </aside>
        </div>
    )
}

function Overview() {
    return <div class="flex flex-col gap-4">
        <div>
            <h3 class="text-lg">Overview</h3>
        </div>
    </div>
}

function Config() {
    const { appConfigStore } = useContext(AppContext)!

    const ttSizeInMib = () =>
        Math.floor(appConfigStore.config.tt_size / 1024 / 1024)

    return <div class="flex flex-col gap-4">
        <div>
            <h3 class="text-lg">Resources</h3>
            <fieldset class="fieldset">
                <legend class="fieldset-legend">Workers</legend>
                <label class="input">
                    <input
                        type="number"
                        min={1}
                        value={appConfigStore.config.workers}
                    />
                    <span class="label">Cores</span>
                </label>
                <p class="label">Number of CPU cores used for computation.</p>
            </fieldset>
            <fieldset class="fieldset">
                <legend class="fieldset-legend">TT Size</legend>
                <label class="input">
                    <input
                        type="number"
                        min={16}
                        max={2048}
                        value={ttSizeInMib()}
                    />
                    <span class="label">MiB</span>
                </label>
                <p class="label">RAM capacity for use in computation.</p>
            </fieldset>
        </div>
        <div>
            <h3 class="text-lg">Time Controls</h3>
            <fieldset class="fieldset">
                <legend class="fieldset-legend">Total Time</legend>
                <label class="input">
                    <input
                        type="number"
                        min={1}
                        placeholder="unlimited"
                        value={appConfigStore.config.initial_timer.total_remaining.secs}
                    />
                    <span class="label">seconds</span>
                </label>
                <p class="label">Max Turn</p>
            </fieldset>
            <fieldset class="fieldset">
                <legend class="fieldset-legend">Increment Time</legend>
                <label class="input">
                    <input
                        type="number"
                        min={0}
                        value={appConfigStore.config.initial_timer.increment.secs}
                    />
                    <span class="label">seconds</span>
                </label>
                <p class="label">Max Turn</p>
            </fieldset>
            <fieldset class="fieldset">
                <legend class="fieldset-legend">Max Turn Time</legend>
                <label class="input">
                    <input
                        type="number"
                        min={1}
                        placeholder="unlimited"
                        value={appConfigStore.config.initial_timer.turn.secs}
                    />
                    <span class="label">seconds</span>
                </label>
                <p class="label">Max Turn</p>
            </fieldset>
        </div>
        <div>
            <h3 class="text-lg">Search Limits</h3>
            <fieldset class="fieldset">
                <legend class="fieldset-legend">Node Limit</legend>
                <label class="input">
                    <input
                        type="number"
                        min={1}
                        placeholder="unlimited"
                        value={appConfigStore.config.max_nodes_in_1k}
                    />
                    <span class="label">×1000</span>
                </label>
                <p class="label">Max Nodes</p>
            </fieldset>
            <fieldset class="fieldset">
                <legend class="fieldset-legend">Depth Limit</legend>
                <label class="input">
                    <input
                        type="number"
                        min={1}
                        placeholder="unlimited"
                        value={appConfigStore.config.max_depth}
                    />
                    <span class="label">moves</span>
                </label>
                <p class="label">Max Depth</p>
            </fieldset>
        </div>
    </div>
}

function ProviderConfig() {
    const { appActions, appStore } = useContext(AppContext)!

    return <div class="flex flex-col gap-2">
    </div>
}

function DangerZone() {
    const { appActions, appStore } = useContext(AppContext)!

    return <div class="flex flex-col gap-2">
        <h3 class="text-lg">Data Controls</h3>
        <button
            class="btn btn-error"
            onClick={appActions.clearAppConfigStore}
        >
            Reset App Data
        </button>
    </div>
}
