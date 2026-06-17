import { AppContext } from "../context"
import { createEffect, createMemo, createResource, createSignal, Match, on, Show, Switch, useContext } from "solid-js"
import { unwrap } from "solid-js/store"
import type { Config } from "rusty-renju-web/rusty-renju"
import { flatmap } from "../utils/undefined"
import { SERVER_PROTOCOL, SERVER_URL, WEB_WORKER_READY } from "rusty-renju-web/config"
import { duration, formatNodes, nps } from "rusty-renju-web/mintaka"
import { checkHealth, type MintakaServerConfig } from "rusty-renju-web/provider/mintaka.server.provider"
import { Modal, type ModalControlProps } from "./Modal"

export function Dashboard(props: ModalControlProps) {
    const { runtimeSelectors } = useContext(AppContext)!

    return <Modal
        id="dashboard_modal"
        title="mintaka WebUI"
        open={props.open}
        onClose={props.onClose}
    >
        <div class="flex flex-col gap-4">
            <Overview />
            <RuntimeConfig />
            <Show when={runtimeSelectors.configs()}>{configs =>
                <ConfigSections config={configs().config} maxConfig={configs().max_config} />
            }</Show>
            <DataSections />
        </div>
    </Modal>
}

function Overview() {
    const { runtimeSelectors } = useContext(AppContext)!

    return <Show when={runtimeSelectors.statics()}>{statics =>
        <div class="flex flex-col gap-4">
            <h3 class="text font-bold">Overview</h3>
                <div class="flex-col gap-2">
                    <p class="text"><b>Total</b> {formatNodes(statics().totalNodesIn1k)} <span class="text-sm">nodes</span></p>
                    <p class="text"><b>Runtime</b> {statics().totalRuntime.secs} <span class="text-sm">seconds</span></p>
                    <p class="text"><b>NPS</b> {formatNodes(nps(statics()))} <span class="text-sm">nodes/s</span></p>
                </div>
        </div>
    }</Show>
}

function RuntimeConfig() {
    const { appActions, persistConfig } = useContext(AppContext)!

    return <div class="flex flex-col gap-2">
        <h3 class="text font-bold">Runtime</h3>
        <Show when={WEB_WORKER_READY}>
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
        </Show>
        <Show when={persistConfig.selectedProviderType === "server"}>
            <ServerConfigSections />
        </Show>
    </div>
}

function ServerConfigSections() {
    const { runtimeSelectors, appActions, persistConfig, setPersistConfig } = useContext(AppContext)!

    const [address, setAddress] = createSignal(persistConfig.serverConfig?.address ?? SERVER_URL)
    const [apiPassword, setApiPassword] = createSignal(persistConfig.serverConfig?.apiPassword ?? "")

    const candidateServerConfig = createMemo<MintakaServerConfig | undefined>(() => {
        if (!/^((([a-zA-Z0-9-]+\.)*[a-zA-Z0-9-]+)|(\d{1,3}(\.\d{1,3}){3})):[0-9]+$/.test(address()))
            return undefined

        return {
            address: address(),
            apiPassword: apiPassword() || undefined,
        }
    })

    const [serverConfig, { mutate }] = createResource(candidateServerConfig, async (currentConfig) => {
        try {
            if (await checkHealth(currentConfig))
                return currentConfig
        } catch (e) {}

        return undefined
    })

    createEffect(on(candidateServerConfig, (config) => {
        if (config === undefined) mutate(undefined)
    }))

    createEffect(on(serverConfig, config => {
        setPersistConfig("serverConfig", config)
    }))

    return <>
        <fieldset class="fieldset">
            <legend class="fieldset-legend">Server Address</legend>
            <label class="input" classList={{ "input-error": candidateServerConfig() === undefined }}>
                <span class="label">{SERVER_PROTOCOL}://</span>
                <input
                    type="text"
                    placeholder="localhost:8085"
                    disabled={runtimeSelectors.runtimeType() === "ready"}
                    value={address()}
                    onChange={event => {
                        if (!(event.target instanceof HTMLInputElement)) return

                        setAddress(event.target.value)
                    }}
                />
            </label>
            <p class="label text-wrap">Self-hosted mintaka-server</p>
        </fieldset>
        <fieldset class="fieldset">
            <legend class="fieldset-legend">API Password</legend>
            <label class="input">
                <input
                    type="password"
                    autocomplete="current-password"
                    disabled={runtimeSelectors.runtimeType() === "ready"}
                    value={apiPassword()}
                    placeholder="No password required"
                    onChange={event => {
                        if (!(event.target instanceof HTMLInputElement)) return

                        setApiPassword(event.target.value)
                    }}
                />
            </label>
        </fieldset>
        <Switch>
            <Match when={runtimeSelectors.runtimeType() === "loading"}>
                <button
                    class="btn btn-disabled btn-success"
                >
                    <span class="loading loading-spinner" />Connecting
                </button>
            </Match>
            <Match when={runtimeSelectors.runtimeType() === "ready"}>
                <button
                    class="btn btn-error"
                    onClick={appActions.switchServerRuntime}
                >
                    Disconnect
                </button>
            </Match>
            <Match when={false}>
                <button
                    class="btn"
                    classList={{
                        "btn-success": serverConfig() !== undefined,
                        "btn-disabled": serverConfig() === undefined,
                    }}
                    onClick={appActions.loadServerRuntime}
                >
                    Connect
                </button>
            </Match>
        </Switch>
    </>
}

function ConfigSections(props: { config: Config, maxConfig: Config }) {
    return <div class="flex flex-col gap-4">
        <div>
            <h3 class="text font-bold">Resources</h3>
            <NumericConfigSection
                produce={value => ({ ...unwrap(props.config), workers: value })}
                value={props.config.workers}
                min={1}
                max={props.maxConfig.workers ?? 1}
                scale={1}
                legend="Workers" label="Cores"
                description="CPU cores. Must be less than the number of logical CPUs."
            />
            <NumericConfigSection
                produce={value => ({ ...unwrap(props.config), tt_size: value })}
                value={props.config.tt_size}
                min={1024 * 1024 * 16}
                max={props.maxConfig.tt_size ?? 1024 * 1024 * 1024 * 8}
                scale={1024 * 1024}
                legend="TT Size" label="MiB"
                description="Shared memory size. Should be properly sized relative to other resources."
            />
        </div>
        <div>
            <h3 class="text font-bold">Time Controls</h3>
            <NumericConfigSection
                produce={value => {
                    const config = unwrap(props.config)
                    return {
                        ...config,
                        initial_timer: {
                            ...config.initial_timer,
                            total_remaining: flatmap(value, valid => duration(valid)),
                        },
                    }
                }}
                value={props.config.initial_timer.total_remaining?.secs}
                optional
                placeholder="unlimited"
                max={props.maxConfig.initial_timer.total_remaining?.secs}
                scale={1}
                legend="Total Time" label="seconds"
                description="Default time limit."
            />
            <NumericConfigSection
                produce={value => {
                    const config = unwrap(props.config)
                    return {
                        ...config,
                        initial_timer: {
                            ...config.initial_timer,
                            increment: duration(value),
                        },
                    }
                }}
                value={props.config.initial_timer.increment.secs}
                min={0}
                max={props.maxConfig.initial_timer.increment.secs}
                scale={1}
                legend="Increment Time" label="seconds"
                description="Time added after each move."
            />
            <NumericConfigSection
                produce={value => {
                    const config = unwrap(props.config)
                    return {
                        ...config,
                        initial_timer: {
                            ...config.initial_timer,
                            turn: flatmap(value, valid => duration(valid)),
                        },
                    }
                }}
                value={props.config.initial_timer.turn?.secs}
                optional
                placeholder="unlimited"
                max={props.maxConfig.initial_timer.turn?.secs}
                scale={1}
                legend="Max Turn Time" label="seconds"
                description="Maximum time for each move."
            />
        </div>
        <div>
            <h3 class="text font-bold">Search Limits</h3>
            <NumericConfigSection
                produce={value => ({ ...unwrap(props.config), max_nodes_in_1k: value })}
                value={props.config.max_nodes_in_1k}
                optional
                placeholder="unlimited"
                scale={1000}
                max={props.maxConfig.max_nodes_in_1k}
                legend="Node Limit" label="×1000 nodes"
                description="Maximum reachable nodes. Specify when maintaining a constant level regardless of time or hardware."
            />
            <NumericConfigSection
                produce={value => ({ ...unwrap(props.config), max_depth: value })}
                value={props.config.max_depth}
                optional
                placeholder="unlimited"
                max={props.maxConfig.max_depth ?? 225}
                scale={1}
                legend="Depth Limit" label="moves"
                description="Maximum reachable selective depth."
            />
        </div>
    </div>
}

function DataSections() {
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

type NumericConfigSectionBaseProps = {
    max?: number,
    scale: number,
    legend: string,
    label: string,
    description: string,
}

type RequiredNumericConfigSectionProps = NumericConfigSectionBaseProps & {
    optional?: false,
    placeholder?: never,
    value: number,
    min: number,
    produce: (value: number) => Config,
}

type OptionalNumericConfigSectionProps = NumericConfigSectionBaseProps & {
    optional: true,
    value: number | undefined,
    min?: number,
    placeholder: string,
    produce: (value: number | undefined) => Config,
}

type NumericConfigSectionProps =
    | RequiredNumericConfigSectionProps
    | OptionalNumericConfigSectionProps

function NumericConfigSection(props: NumericConfigSectionProps) {
    const { appActions } = useContext(AppContext)!

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
                value={flatmap(props.value, valid => scaled(valid, props.scale)) ?? ""}
                min={flatmap(props.min, valid => scaled(valid, props.scale))}
                max={flatmap(props.max, valid => scaled(valid, props.scale))}
                placeholder={props.optional ? props.placeholder : undefined}
                onChange={event => {
                    const { value, valueAsNumber } = event.currentTarget
                    if (value === "") {
                        if (!props.optional) {
                            setIsValid(false)
                            return
                        }

                        setIsValid(true)
                        appActions.updateConfig(props.produce(undefined))
                        return
                    }

                    if (Number.isNaN(valueAsNumber)) {
                        setIsValid(false)
                        return
                    }

                    const newValue = valueAsNumber * props.scale
                    const valid = (props.min === undefined || props.min <= newValue)
                        && (props.max === undefined || newValue <= props.max)
                    setIsValid(valid)
                    if (valid)
                        appActions.updateConfig(props.produce(newValue))
                }}
            />
            <span class="label">{props.label}</span>
        </label>
        <p class="label text-wrap">{props.description}</p>
    </fieldset>
}

function scaled(value: number, scale: number) {
    return Math.trunc(value / scale)
}
