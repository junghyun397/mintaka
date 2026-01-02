import { Match, Show, Switch, useContext } from "solid-js";
import { AppContext } from "../context";
import {
    IconArrowUturnRight, IconChevronDoubleLeft, IconChevronDoubleRight, IconChevronLeft, IconChevronRight, IconCog8Tooth,
    IconCpuChip, IconGitBranch, IconInformationCircle, IconMagnifyingGlassMinus, IconMagnifyingGlassPlus, IconMoon,
    IconPlay, IconStop, IconSun, IconThemeAuto,
} from "./icons";
import { nextHistoryDisplay, nextTheme } from "../stores/app.config.store";

export function Control() {
    return <div class="flex gap-2 rounded-box bg-base-100 p-2">
        <ConfigButton />
        <ControlButtons />
        <DashboardButton />
    </div>
}

function ConfigButton() {
    const { appConfigStore, setAppConfigStore } = useContext(AppContext)!

    const cycleTheme = () =>
        // @ts-ignore
        setAppConfigStore("theme", nextTheme(appConfigStore.theme))

    const cycleHistoryDisplay = () =>
        // @ts-ignore
        setAppConfigStore("historyDisplay", nextHistoryDisplay(appConfigStore.historyDisplay))

    const toggleZoomBoard = () => {
        // @ts-ignore
        setAppConfigStore("zoomBoard", !appConfigStore.zoomBoard)
    }

    return <div class="dropdown dropdown-center dropdown-top">
        <div tabindex="0" role="button" class="btn btn-square">
            <IconCog8Tooth />
        </div>
        <ul tabindex="-1" class="dropdown-content menu z-1 gap-2 rounded-box bg-base-100">
            <li>
                <button
                    class="btn btn-square"
                >
                    <IconInformationCircle />
                </button>
            </li>
            <li>
                <button
                    class="btn btn-square"
                    onClick={cycleTheme}
                >
                    <Switch>
                        <Match when={appConfigStore.theme === "system"}><IconThemeAuto /></Match>
                        <Match when={appConfigStore.theme === "dark"}><IconMoon /></Match>
                        <Match when={appConfigStore.theme === "light"}><IconSun /></Match>
                    </Switch>
                </button>
            </li>
            <li>
                <button
                    class="btn btn-square"
                    onClick={cycleHistoryDisplay}
                >
                    <svg viewBox="0 0 100 100">
                        <defs><mask id="historyDisplayMask">
                            <rect x="0" y="0" width="100" height="100" fill="white"/>
                            <Switch>
                                <Match when={appConfigStore.historyDisplay === "last"}>
                                    <circle fill="black" cx="50" cy="50" r="8"/>
                                </Match>
                                <Match when={appConfigStore.historyDisplay === "pair"}>
                                    <g stroke="black" stroke-width="4">
                                        <line x1="35" y1="50" x2="65" y2="50"/>
                                        <line x1="50" y1="35" x2="50" y2="65"/>
                                    </g>
                                </Match>
                                <Match when={appConfigStore.historyDisplay === "sequence"}>
                                    <text
                                        font-family="serif"
                                        font-size="42"
                                        text-anchor="middle" dominant-baseline="middle"
                                        fill="black"
                                        x="50" y="54"
                                    >
                                        42
                                    </text>
                                </Match>
                            </Switch>
                        </mask></defs>
                        <circle
                            fill="currentColor"
                            cx="50" cy="50" r="32"
                            mask="url(#historyDisplayMask)"
                        />
                    </svg>
                </button>
            </li>
            <li class="block md:hidden">
                <button
                    class="btn btn-square"
                    onClick={toggleZoomBoard}
                >
                    <Switch>
                        <Match when={!appConfigStore.zoomBoard}><IconMagnifyingGlassPlus /></Match>
                        <Match when={appConfigStore.zoomBoard}><IconMagnifyingGlassMinus /></Match>
                    </Switch>
                </button>
            </li>
        </ul>
    </div>
}

function ControlButtons() {
    const { actions, workerStore, gameStore } = useContext(AppContext)!

    const backwardDisabled = () =>
        !gameStore.backwardable || workerStore.inComputing

    return <>
        <button
            class="btn btn-square max-xs:hidden"
            classList={{
                "btn-disabled": backwardDisabled(),
            }}
            onClick={actions.bulkBackward}
        >
            <IconChevronDoubleLeft />
        </button>
        <button
            class="btn btn-square"
            classList={{
                "btn-disabled": backwardDisabled(),
            }}
            onClick={actions.backward}
        >
            <IconChevronLeft />
        </button>
        <Show when={workerStore.inComputing} fallback={
            <button
                class="btn btn-square"
                classList={{
                    "btn-disabled": workerStore.loadedProviderType === undefined,
                }}
                onClick={actions.launch}
            >
                <IconPlay />
            </button>
        }>
            <button
                class="btn btn-square animate-pulse"
                classList={{
                    "btn-disabled": false,
                }}
                onClick={actions.abort}
            >
                <IconStop />
            </button>
        </Show>
        <Show when={gameStore.inBranchHead} fallback={
            <button
                class="btn btn-square"
                classList={{
                    "btn-disabled": !gameStore.forwardable,
                }}
                onClick={[actions.forward, "continue"]}
            >
                <IconChevronRight />
            </button>
        }>
            <div class="dropdown dropdown-center dropdown-top">
                <div tabindex="0" role="button" class="btn btn-square">
                    <IconGitBranch />
                </div>
                <ul tabindex="-1" class="dropdown-content menu z-1 gap-2 rounded-box bg-base-100">
                    <li>
                        <button
                            class="btn btn-square"
                            onClick={[actions.forward, "return"]}
                        >
                            <IconArrowUturnRight />
                        </button>
                    </li>
                    <li>
                        <button
                            class="btn btn-square"
                            onClick={[actions.forward, "continue"]}
                        >
                            <IconChevronRight />
                        </button>
                    </li>
                </ul>
            </div>
        </Show>
        <button
            class="btn btn-square max-xs:hidden"
            classList={{
                "btn-disabled": !gameStore.forwardable,
            }}
            onClick={[actions.bulkForward, "continue"]}
        >
            <IconChevronDoubleRight />
        </button>
    </>
}

function DashboardButton() {
    const { actions, appConfigStore } = useContext(AppContext)!

    return <button
        class="btn btn-active btn-square"
        classList={{
            "btn-active": appConfigStore.openDashboard,
        }}
    >
        <IconCpuChip />
    </button>
}
