import { createMemo, createSignal, Match, Show, Switch, useContext } from "solid-js"
import { AppContext } from "../context"
import {
    IconArrowUturnRight, IconChevronDoubleLeft, IconChevronDoubleRight, IconChevronLeft, IconChevronRight,
    IconCog8Tooth, IconDocument, IconGitBranch, IconInformationCircle, IconMagnifyingGlassMinus, IconMagnifyingGlassPlus, IconBars3,
    IconMoon, IconPause, IconPlay, IconStop, IconSun, IconThemeAuto,
} from "./icons"
import { nextHistoryDisplay, nextTheme } from "../stores/persist.config"
import { About } from "./About"
import { Portal } from "solid-js/web"
import { History } from "./History"
import { Dashboard } from "./Dashboard"

export function Control() {
    return <div class="flex gap-2 rounded-box bg-base-100 p-2 max-xs:gap-1 max-xs:p-1">
        <ConfigButton />
        <ControlButtons />
        <DashboardButton />
    </div>
}

function ControlButtons() {
    const { gameActions, appSettings, gameSelectors, runtimeSelectors } = useContext(AppContext)!

    const inBranchHead = createMemo(() => gameSelectors.gameState().historyTree.inBranchHead)
    const forwardable = createMemo(() => gameSelectors.gameState().historyTree.forwardable)
    const backwardable = createMemo(() => gameSelectors.gameState().historyTree.backwardable)

    return <>
        <Show when={gameSelectors.history().length === 0 && forwardable()} fallback={
            <button
                title="Undo all moves"
                class="btn btn-square"
                classList={{ "btn-disabled": !backwardable() }}
                onClick={gameActions.bulkBackward}
            >
                <IconChevronDoubleLeft />
            </button>
        }>
            <button
                title="Clear"
                class="btn btn-square"
                onClick={gameActions.clear}
            >
                <IconDocument />
            </button>
        </Show>
        <button
            title="Undo"
            class="btn btn-square"
            classList={{ "btn-disabled": !backwardable() }}
            onClick={gameActions.backward}
        >
            <IconChevronLeft />
        </button>
        <Switch fallback={
            <button
                title="Paused"
                class="btn btn-disabled btn-square"
            >
                <IconPlay />
            </button>
        }>
            <Match when={runtimeSelectors.inComputing()}>
                <button
                    title="Abort"
                    class="btn btn-square animate-pulse"
                    onClick={gameActions.abort}
                >
                    <IconStop />
                </button>
            </Match>
            <Match when={appSettings.launch && runtimeSelectors.isReady()}>
                <button
                    title="Pause"
                    class="btn btn-square"
                    onClick={gameActions.pause}
                >
                    <IconPause />
                </button>
            </Match>
            <Match when={!appSettings.launch && runtimeSelectors.isReady()}>
                <button
                    title="Start"
                    class="btn btn-square"
                    onClick={gameActions.start}
                >
                    <IconPlay />
                </button>
            </Match>
        </Switch>
        <Show when={inBranchHead()} fallback={
            <button
                title="Redo"
                class="btn btn-square"
                classList={{ "btn-disabled": !forwardable() }}
                onClick={[gameActions.forward, "continue"]}
            >
                <IconChevronRight />
            </button>
        }>
            <div class="dropdown dropdown-center dropdown-top">
                <div
                    title="Redo options"
                    tabindex="0" role="button" class="btn btn-square"
                >
                    <IconGitBranch />
                </div>
                <ul tabindex="-1" class="dropdown-content menu z-1 gap-2 rounded-box bg-base-100 max-xs:gap-1 max-xs:p-1">
                    <li>
                        <button
                            title="Continue as previous move"
                            class="btn btn-square"
                            onClick={[gameActions.forward, "return"]}
                        >
                            <IconArrowUturnRight />
                        </button>
                    </li>
                    <li>
                        <button
                            title="Continue as recent move"
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
            title="Redo all moves"
            class="btn btn-square"
            classList={{ "btn-disabled": !forwardable() }}
            onClick={[gameActions.bulkForward, "continue"]}
        >
            <IconChevronDoubleRight />
        </button>
    </>
}

function DashboardButton() {
    const [openDashboard, setOpenDashboard] = createSignal(false)

    return <>
        <button
            title="Settings"
            class="btn btn-square"
            onClick={() => setOpenDashboard(true)}
        >
            <IconCog8Tooth />
        </button>
        <Portal>
            <Dashboard
                open={openDashboard()}
                onClose={() => setOpenDashboard(false)}
            />
        </Portal>
    </>
}

function ConfigButton() {
    const { persistConfig, setPersistConfig } = useContext(AppContext)!

    const cycleTheme = () =>
        setPersistConfig("theme", nextTheme(persistConfig.theme))

    const cycleHistoryDisplay = () =>
        setPersistConfig("historyDisplay", nextHistoryDisplay(persistConfig.historyDisplay))

    const toggleZoomBoard = () => {
        setPersistConfig("zoomBoard", !persistConfig.zoomBoard)
    }

    const [openBranch, setOpenBranch] = createSignal(false)

    const [openAbout, setOpenAbout] = createSignal(false)

    return <div class="dropdown dropdown-center dropdown-top">
        <div
            title="Options"
            tabindex="0" role="button" class="btn btn-square"
        >
            <IconBars3 />
        </div>
        <ul tabindex="-1" class="dropdown-content menu z-1 gap-2 rounded-box bg-base-100 p-2 max-xs:gap-1 max-xs:p-1">
            <li>
                <button
                    title="About"
                    class="btn btn-square"
                    onClick={() => setOpenAbout(true)}
                >
                    <IconInformationCircle />
                </button>
                <Portal>
                    <About open={openAbout()} onClose={() => setOpenAbout(false)} />
                </Portal>
            </li>
            <li>
                <button
                    title="History"
                    class="btn btn-square"
                    onClick={() => setOpenBranch(true)}
                >
                    <IconGitBranch />
                </button>
                <Portal>
                    <History open={openBranch()} onClose={() => setOpenBranch(false)} />
                </Portal>
            </li>
            <li>
                <button
                    title="Theme"
                    class="btn btn-square"
                    onClick={cycleTheme}
                >
                    <Switch>
                        <Match when={persistConfig.theme === "system"}><IconThemeAuto /></Match>
                        <Match when={persistConfig.theme === "dark"}><IconMoon /></Match>
                        <Match when={persistConfig.theme === "light"}><IconSun /></Match>
                    </Switch>
                </button>
            </li>
            <li>
                <button
                    title="Markers"
                    class="btn btn-square"
                    onClick={cycleHistoryDisplay}
                >
                    <svg viewBox="0 0 100 100">
                        <defs><mask id="historyDisplayMask">
                            <rect x="0" y="0" width="100" height="100" fill="white"/>
                            <Switch>
                                <Match when={persistConfig.historyDisplay === "last"}>
                                    <circle fill="black" cx="50" cy="50" r="8"/>
                                </Match>
                                <Match when={persistConfig.historyDisplay === "pair"}>
                                    <g stroke="black" stroke-width="4">
                                        <line x1="35" y1="50" x2="65" y2="50"/>
                                        <line x1="50" y1="35" x2="50" y2="65"/>
                                    </g>
                                </Match>
                                <Match when={persistConfig.historyDisplay === "sequence"}>
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
                    title="Zoom board"
                    class="btn btn-square"
                    onClick={toggleZoomBoard}
                >
                    <Switch>
                        <Match when={!persistConfig.zoomBoard}>
                            <IconMagnifyingGlassPlus />
                        </Match>
                        <Match when={persistConfig.zoomBoard}>
                            <IconMagnifyingGlassMinus />
                        </Match>
                    </Switch>
                </button>
            </li>
        </ul>
    </div>
}
