import {Match, Show, Switch, useContext} from "solid-js";
import {AppContext} from "../context";
import {
    IconArrowUturnRight,
    IconChevronDoubleLeft,
    IconChevronDoubleRight,
    IconChevronLeft,
    IconChevronRight,
    IconCog8Tooth,
    IconCpuChip,
    IconGitBranch,
    IconMagnifyingGlassMinus,
    IconMagnifyingGlassPlus,
    IconMoon,
    IconPlay,
    IconSun,
    IconThemeAuto
} from "./icons";
import {nextHistoryDisplay, nextTheme} from "../stores/app.config.store";

export function Control() {
    const { actions, appConfigStore, setAppConfigStore, gameStore } = useContext(AppContext)!

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

    return <div class="flex gap-2 bg-base-100 p-2 rounded-box">
        <div class="dropdown dropdown-top dropdown-center">
            <div tabindex="0" role="button" class="btn btn-square">
                <IconCog8Tooth />
            </div>
            <ul tabindex="-1" class="dropdown-content menu rounded-box gap-2 bg-base-100 z-1">
                <li>
                    <button
                        class="btn btn-square"
                        onClick={cycleTheme}
                    >
                        <Switch>
                            <Match when={appConfigStore.theme == "system"}><IconThemeAuto /></Match>
                            <Match when={appConfigStore.theme == "dark"}><IconMoon /></Match>
                            <Match when={appConfigStore.theme == "light"}><IconSun /></Match>
                        </Switch>
                    </button>
                </li>
                <li>
                    <button
                        class="btn btn-square"
                        onClick={cycleHistoryDisplay}
                    >
                        <Switch>
                            <Match when={appConfigStore.historyDisplay == "none"}>None</Match>
                            <Match when={appConfigStore.historyDisplay == "last"}>Last</Match>
                            <Match when={appConfigStore.historyDisplay == "pair"}>Pair</Match>
                            <Match when={appConfigStore.historyDisplay == "sequence"}>Seq</Match>
                        </Switch>
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
        <button
            class="btn btn-square"
            classList={{
                "btn-disabled": !gameStore.backwardable,
            }}
            onClick={actions.bulkBackward}
        >
            <IconChevronDoubleLeft />
        </button>
        <button
            class="btn btn-square"
            classList={{
                "btn-disabled": !gameStore.backwardable,
            }}
            onClick={actions.backward}
        >
            <IconChevronLeft />
        </button>
        <button
            class="btn btn-square"
            classList={{
                "btn-disabled": true
            }}
        >
            <IconPlay />
        </button>
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
            <div class="dropdown dropdown-top dropdown-center">
                <div tabindex="0" role="button" class="btn btn-square">
                      <IconGitBranch />
                </div>
                <ul tabindex="-1" class="dropdown-content menu rounded-box gap-2 bg-base-100 z-1">
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
            class="btn btn-square"
            classList={{
                "btn-disabled": !gameStore.forwardable,
            }}
            onClick={[actions.bulkForward, "continue"]}
        >
            <IconChevronDoubleRight />
        </button>
        <button
            class="btn btn-square btn-active"
            classList={{
                "btn-active": appConfigStore.openDashboard
            }}
        >
            <IconCpuChip />
        </button>
    </div>
}
