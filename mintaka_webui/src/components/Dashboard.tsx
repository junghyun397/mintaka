import { AppContext } from "../context"
import { createSignal, useContext } from "solid-js"
import { unwrap } from "solid-js/store"
import type { Config } from "../wasm/pkg/mintaka_wasm"
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

type MinValue =
    | { type: "finite", value: number, }
    | { type: "optional", value: undefined, placeholder: string }

function ConfigSection<M extends MinValue, V = number | M["value"]>(props: {
    produce: (config: Config, value: V) => Config,
    value: V,
    min: M,
    max: number,
    scale: number,
    legend: string,
    label: string,
    description: string,
}) {
    const { appActions, persistConfig } = useContext(AppContext)!

    const [isValid, setIsValid] = createSignal(true)

    return <fieldset class="fieldset">
        <legend class="fieldset-legend">{props.legend}</legend>
        <label
            class="input"
            classList={{
                "input-error": !isValid(),
            }}
        >
            <input
                type="number"
                value={flatmap(props.value as number | undefined, valid => Math.trunc(valid / props.scale))}
                max={props.max}
                placeholder={props.min.type === "optional" ? props.min.placeholder : undefined}
                onChange={event => {
                    const newValue = event.target instanceof HTMLInputElement
                        ? event.target.valueAsNumber * props.scale
                        : undefined

                    if (newValue === undefined && props.min.type === "optional") {
                        setIsValid(true)
                        appActions.syncConfig(props.produce(unwrap(persistConfig.config), newValue as V))
                        return
                    }

                    if (newValue === undefined) {
                        setIsValid(false)
                        return
                    }

                    if ((props.min.value ?? -1) <= newValue && newValue <= props.max) {
                        setIsValid(true)
                        appActions.syncConfig(props.produce(unwrap(persistConfig.config), newValue as V))
                        return
                    }
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
                produce={(config, value) => ({ ...config, workers: value })}
                value={persistConfig.config.workers}
                min={{
                    type: "finite",
                    value: 1,
                }}
                max={maxMintakaConfig()?.workers ?? 1}
                scale={1}
                legend="Workers" label="Cores"
                description="CPU core allocation. Must be less than the number of logical CPUs."
            />
            <ConfigSection
                produce={(config, value) => ({ ...config, tt_size: value })}
                value={persistConfig.config.tt_size}
                min={{
                    type: "finite",
                    value: 1024 * 1024 * 16,
                }}
                max={maxMintakaConfig()?.tt_size ?? 1024 * 1024 * 1024 * 8}
                scale={1024 * 1024}
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
                produce={(config, value) => ({ ...config, max_nodes_in_1k: value })}
                value={persistConfig.config.max_nodes_in_1k}
                min={{
                    type: "optional",
                    value: undefined,
                    placeholder: "unlimited",
                }}
                scale={1000}
                max={maxMintakaConfig()?.max_nodes_in_1k ?? 1000000000000}
                legend="Node Limit" label="×1000 nodes"
                description="Maximum reachable nodes. Specify when maintaining a constant level regardless of time or hardware."
            />
            <ConfigSection
                produce={(config, value) => ({ ...config, max_depth: value })}
                value={persistConfig.config.max_depth}
                min={{
                    type: "optional",
                    value: undefined,
                    placeholder: "unlimited",
                }}
                max={maxMintakaConfig()?.max_depth ?? 225}
                scale={1}
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
