import { createMemo, Match, Show, Switch, useContext } from "solid-js"
import { AppContext } from "../context"
import {
    IconArrowUturnRight, IconChevronDoubleLeft, IconChevronDoubleRight, IconChevronLeft, IconChevronRight,
    IconCog8Tooth, IconCpuChip, IconGitBranch, IconInformationCircle, IconMagnifyingGlassMinus, IconMagnifyingGlassPlus,
    IconMoon, IconPause, IconPlay, IconStop, IconSun, IconThemeAuto,
} from "./icons"
import { nextHistoryDisplay, nextTheme } from "../stores/app.config.store"
import { Portal } from "solid-js/web"

export function Control() {
    return <div class="flex gap-2 rounded-box bg-base-100 p-2 max-xs:gap-1 max-xs:p-1">
        <DashboardButton />
        <ControlButtons />
        <ConfigButton />
    </div>
}

function ControlButtons() {
    const { gameActions, appStore, gameState, runtimeState } = useContext(AppContext)!

    const inComputing = createMemo(() => runtimeState()?.type !== "idle")

    const inBranchHead = createMemo(() => gameState().historyTree.inBranchHead)
    const forwardable = createMemo(() => gameState().historyTree.forwardable)
    const backwardable = createMemo(() => gameState().historyTree.backwardable)

    return <>
        <button
            class="btn btn-square"
            classList={{ "btn-disabled": !backwardable() }}
            onClick={gameActions.bulkBackward}
        >
            <IconChevronDoubleLeft />
        </button>
        <button
            class="btn btn-square"
            classList={{ "btn-disabled": !backwardable() }}
            onClick={gameActions.backward}
        >
            <IconChevronLeft />
        </button>
        <Switch>
            <Match when={inComputing()}>
                <button
                    class="btn btn-square animate-pulse"
                    onClick={gameActions.abort}
                >
                    <IconStop />
                </button>
            </Match>
            <Match when={appStore.autoLaunch && !inComputing()}>
                <button
                    class="btn btn-square"
                    onClick={gameActions.pause}
                >
                    <IconPause />
                </button>
            </Match>
            <Match when={!appStore.autoLaunch && !inComputing()}>
                <button
                    class="btn btn-square"
                    classList={{ "btn-disabled": runtimeState() === undefined }}
                    onClick={gameActions.start}
                >
                    <IconPlay />
                </button>
            </Match>
        </Switch>
        <Show when={inBranchHead()} fallback={
            <button
                class="btn btn-square"
                classList={{ "btn-disabled": !forwardable() }}
                onClick={[gameActions.forward, "continue"]}
            >
                <IconChevronRight />
            </button>
        }>
            <div class="dropdown dropdown-center dropdown-top">
                <div tabindex="0" role="button" class="btn btn-square">
                    <IconGitBranch />
                </div>
                <ul tabindex="-1" class="dropdown-content menu z-1 gap-2 rounded-box bg-base-100 max-xs:gap-1 max-xs:p-1">
                    <li>
                        <button
                            class="btn btn-square"
                            onClick={[gameActions.forward, "return"]}
                        >
                            <IconArrowUturnRight />
                        </button>
                    </li>
                    <li>
                        <button
                            class="btn btn-square"
                            onClick={[gameActions.forward, "continue"]}
                        >
                            <IconChevronRight />
                        </button>
                    </li>
                </ul>
            </div>
        </Show>
        <button
            class="btn btn-square"
            classList={{ "btn-disabled": !forwardable() }}
            onClick={[gameActions.bulkForward, "continue"]}
        >
            <IconChevronDoubleRight />
        </button>
    </>
}

function DashboardButton() {
    const { setAppConfigStore, appConfigStore } = useContext(AppContext)!

    const toggleDashboard = () => {
        // @ts-ignore
        setAppConfigStore("openDashboard", !appConfigStore.openDashboard)
    }

    return <button
        class="btn btn-active btn-square"
        classList={{
            "btn-active": appConfigStore.openDashboard,
        }}
        onClick={toggleDashboard}
    >
        <IconCpuChip />
    </button>
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
        <ul tabindex="-1" class="dropdown-content menu z-1 gap-2 rounded-box bg-base-100 p-2 max-xs:gap-1 max-xs:p-1">
            <li>
                <AboutButton />
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
                            cx="50" cy="50" r="30"
                            mask="url(#historyDisplayMask)"
                        />
                    </svg>
                </button>
            </li>
            <li class="block zoom:hidden">
                <button
                    class="btn btn-square"
                    onClick={toggleZoomBoard}
                >
                    <Switch>
                        <Match when={!appConfigStore.zoomBoard}>
                            <IconMagnifyingGlassPlus />
                        </Match>
                        <Match when={appConfigStore.zoomBoard}>
                            <IconMagnifyingGlassMinus />
                        </Match>
                    </Switch>
                </button>
            </li>
        </ul>
    </div>
}

function AboutButton() {
    let dialogRef: HTMLDialogElement | undefined

    return <>
        <button
            class="btn btn-square"
            onClick={() => dialogRef?.showModal()}
        >
            <IconInformationCircle />
        </button>
        <Portal>
            <dialog ref={ref => dialogRef = ref} id="about_modal" class="modal">
                <div class="modal-box p-3">
                    <form method="dialog">
                        <button class="btn absolute top-2 right-2 btn-sm btn-primary">X</button>
                    </form>
                    <h3 class="text-lg">Mintaka WebUI</h3>
                    <a class="link" target="_blank" rel="noopener" href="https://github.com/junghyun397/mintaka">github.com/junghyun397/mintaka</a>
                </div>
                <form method="dialog" class="modal-backdrop">
                    <button aria-label="Close"></button>
                </form>
            </dialog>
        </Portal>
    </>
}
