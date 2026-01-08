import { AppContext } from "../context"
import { createEffect, createSignal, useContext } from "solid-js"
import { unwrap } from "solid-js/store"
import { compareConfig } from "../wasm/pkg/mintaka_wasm"
import type { Config } from "../wasm/pkg/mintaka_wasm"
import { Keys } from "../utils/types"
import { flatmap } from "../utils/undefined"

export function Dashboard() {
    const { persistConfig, setPersistConfig } = useContext(AppContext)!

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
                    <MintakaConfig />
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

function ConfigSection<T extends Keys<number | undefined, Config>>(props: {
    configKey: T,
    min: number,
    scale: number,
    legend: string,
    label: string,
    description: string,
    placeholder: string,
}) {

    return <fieldset class="fieldset">
        <legend class="fieldset-legend">{props.legend}</legend>
        <label class="input">
            <input
                type="number"
                min={props.min}
                max={maxValue()}
                value={inputValue()}
                onInput={event => {
                }}
            />
            <span class="label">{props.label}</span>
        </label>
        <p class="label text-wrap">{props.description}</p>
    </fieldset>
}

function MintakaConfig() {
    const { persistConfig, maxMintakaConfig } = useContext(AppContext)!

    return <div class="flex flex-col gap-4">
        <div>
            <h3 class="text-lg">Resources</h3>
            <ConfigSection
                configKey="workers" min={1} scale={1}
                legend="Workers" label="Cores"
                description="CPU core allocation. Must be less than the number of physical CPUs."
            />
            <ConfigSection
                configKey="tt_size" min={16} scale={1}
                legend="TT Size" label="MiB"
                description="Shared memory size. Should be properly sized relative to computational volume."
            />
        </div>
        <div>
            <h3 class="text-lg">Time Controls</h3>
            <fieldset class="fieldset">
                <legend class="fieldset-legend">Total Time</legend>
                <label class="input">
                    <input
                        type="number"
                        placeholder="unlimited"
                        min={1}
                        max={maxMintakaConfig()?.initial_timer.total_remaining?.secs}
                        value={persistConfig.config.initial_timer.total_remaining?.secs}
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
                        max={maxMintakaConfig()?.initial_timer.increment.secs}
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
                        placeholder="unlimited"
                        min={1}
                        max={maxMintakaConfig()?.initial_timer.turn?.secs}
                        value={persistConfig.config.initial_timer.turn?.secs}
                    />
                    <span class="label text-wrap">seconds</span>
                </label>
                <p class="label">Max Turn</p>
            </fieldset>
        </div>
        <div>
            <h3 class="text-lg">Search Limits</h3>
            <ConfigSection
                configKey="max_nodes_in_1k" min={1} scale={1}
                placeholder="unlimited"
                legend="Node Limit" label="×1000"
                description="Maximum reachable nodes. Specify when maintaining a constant level regardless of time or hardware."
            />
            <ConfigSection
                configKey="max_depth" min={4} scale={1}
                placeholder="unlimited"
                legend="Depth Limit" label="moves"
                description="Maximum reachable selective depth."
            />
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
            class="btn btn-accent"
            onClick={appActions.resetConfig}
        >
            Reset Config
        </button>
        <button
            class="btn btn-error"
            onClick={appActions.clearAppData}
        >
            Reset App Data
        </button>
    </div>
}
