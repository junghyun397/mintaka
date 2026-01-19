import { AppContext } from "../context"
import { createMemo, createSignal, Match, Show, Switch, useContext } from "solid-js"
import { unwrap } from "solid-js/store"
import type { Config } from "../wasm/pkg/rusty_renju_wasm"
import { flatmap } from "../utils/undefined"
import { SERVER_PROTOCOL, SERVER_URL } from "../config"

export function Dashboard() {
    const { appConfig, setAppConfig, runtimeSelectors } = useContext(AppContext)!

    const closeDashboard = () => {
        setAppConfig("openDashboard", false)
    }

    const configSet = createMemo(() => {
        const config = runtimeSelectors.config()
        const maxConfig = runtimeSelectors.maxConfig()

        if (config === undefined || maxConfig === undefined) return undefined

        return { config, maxConfig }
    })

    return (
        <div
            class="fixed inset-0 z-999"
            classList={{ "pointer-events-none": !appConfig.openDashboard }}
        >
            <button
                class="absolute inset-0 bg-black/40 transition-opacity"
                classList={{
                    "opacity-0": !appConfig.openDashboard,
                    "opacity-100": appConfig.openDashboard,
                }}
                onClick={closeDashboard}
            />
            <aside
                class="absolute top-0 left-0 h-full w-80 max-w-[85vw] overflow-y-auto bg-base-200 shadow-lg transition-transform"
                classList={{
                    "-translate-x-full": !appConfig.openDashboard,
                    "translate-x-0": appConfig.openDashboard,
                }}
            >
                <button
                    class="btn absolute top-4 right-4 btn-xs btn-primary"
                    onClick={closeDashboard}
                >X</button>
                <div class="flex flex-col gap-4 p-4">
                    <h1 class="text-lg">Mintaka WebUI</h1>
                    <Overview />
                    <RuntimeConfig />
                    <Show when={configSet()}>{validConfigs =>
                        <MintakaConfig config={validConfigs().config} maxConfig={validConfigs().maxConfig} />
                    }</Show>
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

function RuntimeConfig() {
    const { appActions, persistConfig, runtimeSelectors } = useContext(AppContext)!

    return <div class="flex flex-col gap-4">
        <h3 class="text-lg">Runtime</h3>
        <div class="btn-group flex gap-4">
            <div class="flex gap-2">
                <input
                    class="radio"
                    type="radio" name="options" id="worker"
                    checked={persistConfig.selectedProviderType === "worker"}
                    onChange={appActions.loadWorkerRuntime}
                />
                <label for="worker" class="text inline-flex items-center">Web Worker</label>
            </div>
            <div class="flex gap-2">
                <input
                    class="radio"
                    type="radio" name="options" id="server"
                    checked={persistConfig.selectedProviderType === "server"}
                    onChange={appActions.switchServerRuntime}
                />
                <label for="server" class="text inline-flex items-center">Server</label>
            </div>
        </div>
        <Show when={persistConfig.selectedProviderType === "server"}>
            <fieldset class="fieldset">
                <legend class="fieldset-legend">Server Address</legend>
                <label class="input" classList={{ "input-error": false }}>
                    <span class="label">{SERVER_PROTOCOL}://</span>
                    <input
                        type="text"
                        placeholder="localhost:8080"
                        value={SERVER_URL}
                        onChange={event => {
                        }}
                    />
                </label>
                <p class="label text-wrap">Self-hosted mintaka-server</p>
            </fieldset>
            <Switch>
                <Match when={runtimeSelectors.runtimeType() === "none"}>
                    <button
                        class="btn"
                        classList={{
                            "btn-success": true,
                            "btn-disabled": false,
                        }}
                        onClick={appActions.loadServerRuntime}
                    >
                        Connect
                    </button>
                </Match>
                <Match when={runtimeSelectors.runtimeType() === "loading"}>
                    <span class="flex gap-2">
                        <span class="loading loading-sm loading-ring"/>
                        <p class="text">Connecting...</p>
                    </span>
                </Match>
                <Match when={runtimeSelectors.runtimeType() === "ready"}>
                    <button
                        class="btn btn-error"
                        onClick={appActions.switchServerRuntime}
                    >
                        Disconnect
                    </button>
                </Match>
            </Switch>
        </Show>
    </div>
}

function MintakaConfig(props: { config: Config, maxConfig: Config }) {
    return <div class="flex flex-col gap-4">
        <div>
            <h3 class="text-lg">Resources</h3>
            <ConfigSection
                produce={value => ({ ...unwrap(props.config), workers: value })}
                value={props.config.workers}
                min={{
                    type: "finite",
                    value: 1,
                }}
                max={props.maxConfig.workers ?? 1}
                scale={1}
                legend="Workers" label="Cores"
                description="CPU core allocation. Must be less than the number of logical CPUs."
            />
            <ConfigSection
                produce={value => ({ ...unwrap(props.config), tt_size: value })}
                value={props.config.tt_size}
                min={{
                    type: "finite",
                    value: 1024 * 1024 * 16,
                }}
                max={props.maxConfig.tt_size ?? 1024 * 1024 * 1024 * 8}
                scale={1024 * 1024}
                legend="TT Size" label="MiB"
                description="Shared memory size. Should be properly sized relative to computational volume."
            />
        </div>
        <div>
            <h3 class="text-lg">Time Controls</h3>
            <ConfigSection
                produce={value => ({ ...{ initial_timer: { total: value } }, ...unwrap(props.config) })}
                value={props.config.initial_timer.total_remaining?.secs}
                min={{
                    type: "optional",
                    value: undefined,
                    placeholder: "undefined",
                }}
                max={props.maxConfig.initial_timer.total_remaining?.secs}
                scale={1}
                legend="Total Time" label="seconds" description="Total time"
            />
            <ConfigSection
                produce={value => ({ ...{ initial_timer: { increment: value } }, ...unwrap(props.config) })}
                value={props.config.initial_timer.increment.secs}
                min={{
                    type: "finite",
                    value: 0,
                    placeholder: "0",
                }}
                max={props.maxConfig.initial_timer.increment.secs}
                scale={1}
                legend="Increment Time" label="seconds" description="Increment time"
            />
            <ConfigSection
                produce={value => ({ ...{ initial_timer: { turn: value } }, ...unwrap(props.config) })}
                value={props.config.initial_timer.turn?.secs}
                min={{
                    type: "optional",
                    value: undefined,
                    placeholder: "unlimited",
                }}
                max={props.maxConfig.initial_timer.turn?.secs}
                scale={1}
                legend="Max Turn Time" label="seconds" description="Max turn"
            />
        </div>
        <div>
            <h3 class="text-lg">Search Limits</h3>
            <ConfigSection
                produce={value => ({ ...unwrap(props.config), max_nodes_in_1k: value })}
                value={props.config.max_nodes_in_1k}
                min={{
                    type: "optional",
                    value: undefined,
                    placeholder: "unlimited",
                }}
                scale={1000}
                max={props.maxConfig.max_nodes_in_1k}
                legend="Node Limit" label="×1000 nodes"
                description="Maximum reachable nodes. Specify when maintaining a constant level regardless of time or hardware."
            />
            <ConfigSection
                produce={value => ({ ...unwrap(props.config), max_depth: value })}
                value={props.config.max_depth}
                min={{
                    type: "optional",
                    value: undefined,
                    placeholder: "unlimited",
                }}
                max={props.maxConfig.max_depth ?? 225}
                scale={1}
                legend="Depth Limit" label="moves" description="Maximum reachable selective depth."
            />
        </div>
    </div>
}

function DangerZone() {
    const { appActions } = useContext(AppContext)!

    return <div class="flex flex-col gap-2">
        <h3 class="text-lg">Data Controls</h3>
        <button
            class="btn btn-accent"
            onClick={appActions.restoreDefaultConfig}
        >
            Restore Default Config
        </button>
        <button
            class="btn btn-error"
            onClick={appActions.resetAppData}
        >
            Reset App Data
        </button>
    </div>
}

type MinValue =
    | { type: "finite", value: number, }
    | { type: "optional", value: undefined, placeholder: string }

function ConfigSection<M extends MinValue, V = number | M["value"]>(props: {
    produce: (value: V) => Config,
    value: V,
    min: M,
    max: number | undefined,
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
                        appActions.updateConfig(props.produce(newValue as V))
                        return
                    }

                    if (newValue === undefined) {
                        setIsValid(false)
                        return
                    }

                    if ((props.min.value === undefined || props.min.value <= newValue)
                        && (props.max === undefined || newValue <= props.max)
                    ) {
                        setIsValid(true)
                        appActions.updateConfig(props.produce(newValue as V))
                        return
                    }
                }}
            />
            <span class="label">{props.label}</span>
        </label>
        <p class="label text-wrap">{props.description}</p>
    </fieldset>
}
