import { AppContext } from "../context"
import { createMemo, useContext } from "solid-js"

export function Dashboard() {
    const { persistConfig, setPersistConfig, runtimeState } = useContext(AppContext)!

    const closeDashboard = () => {
        setPersistConfig("openDashboard", false)
    }

    return (
        <div
            class="fixed inset-0 z-999"
            classList={{ "pointer-events-none": !persistConfig.openDashboard }}
        >
            <button
                class="absolute inset-0 bg-black/40 transition-opacity"
                classList={{
                    "opacity-0": !persistConfig.openDashboard,
                    "opacity-100": persistConfig.openDashboard,
                }}
                onClick={closeDashboard}
            />
            <aside
                class="absolute top-0 left-0 h-full w-80 max-w-[85vw] overflow-y-auto bg-base-200 shadow-lg transition-transform"
                classList={{
                    "-translate-x-full": !persistConfig.openDashboard,
                    "translate-x-0": persistConfig.openDashboard,
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
    const { persistConfig } = useContext(AppContext)!

    const ttSizeInMib = () =>
        Math.floor(persistConfig.config.tt_size / 1024 / 1024)

    return <div class="flex flex-col gap-4">
        <div>
            <h3 class="text-lg">Resources</h3>
            <fieldset class="fieldset">
                <legend class="fieldset-legend">Workers</legend>
                <label class="input">
                    <input
                        type="number"
                        min={1}
                        value={persistConfig.config.workers}
                        onChange={event => {
                            event.target.valueAsNumber
                        }}
                    />
                    <span class="label">Cores</span>
                </label>
                <p class="label text-wrap">CPU core allocation. Must be less than the number of physical CPUs.</p>
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
                <p class="label text-wrap">Shared memory size. Should be properly sized relative to computational volume.</p>
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
                        value={persistConfig.config.initial_timer.total_remaining.secs}
                    />
                    <span class="label">seconds</span>
                </label>
                <p class="label text-wrap">Max Turn</p>
            </fieldset>
            <fieldset class="fieldset">
                <legend class="fieldset-legend">Increment Time</legend>
                <label class="input">
                    <input
                        type="number"
                        min={0}
                        value={persistConfig.config.initial_timer.increment.secs}
                    />
                    <span class="label text-wrap">seconds</span>
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
                        value={persistConfig.config.initial_timer.turn.secs}
                    />
                    <span class="label text-wrap">seconds</span>
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
                        value={persistConfig.config.max_nodes_in_1k}
                    />
                    <span class="label">×1000</span>
                </label>
                <p class="label text-wrap">Maximum reachable nodes. Specify when maintaining a constant level regardless of time or hardware.</p>
            </fieldset>
            <fieldset class="fieldset">
                <legend class="fieldset-legend">Depth Limit</legend>
                <label class="input">
                    <input
                        type="number"
                        min={1}
                        placeholder="unlimited"
                        value={persistConfig.config.max_depth}
                    />
                    <span class="label">moves</span>
                </label>
                <p class="label text-wrap">Maximum reachable selective depth.</p>
            </fieldset>
        </div>
    </div>
}

function ProviderConfig() {
    const { appActions } = useContext(AppContext)!

    return <div class="flex flex-col gap-2">
    </div>
}

function DangerZone() {
    const { appActions } = useContext(AppContext)!

    return <div class="flex flex-col gap-2">
        <h3 class="text-lg">Data Controls</h3>
        <button
            class="btn btn-error"
            onClick={appActions.cleatAppData}
        >
            Reset App Data
        </button>
    </div>
}
