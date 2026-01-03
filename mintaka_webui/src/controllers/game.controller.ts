import { reconcile, SetStoreFunction, unwrap } from "solid-js/store"
import { ForwardMethod } from "../domain/HistoryTree"
import { Color, HashKey, MaybePos, Pos } from "../wasm/pkg/mintaka_wasm"
import { WorkerStore } from "../stores/worker.store"
import { buildGameStore, GameStore } from "../stores/game.store"
import { flip } from "../domain/rusty-renju"
import { AppState } from "../stores/app.state"

export interface GameActions {
    play: (pos: Pos) => void,
    forward: (method: ForwardMethod) => void,
    bulkForward: (method: ForwardMethod) => void,
    backward: () => void,
    bulkBackward: () => void,
    start: () => void,
    pause: () => void,
    abort: () => void,
}

export function createGameController(props: {
    appState: AppState,
    workerStore: WorkerStore,
    setWorkerStore: SetStoreFunction<WorkerStore>,
    gameStore: GameStore,
    setGameStore: SetStoreFunction<GameStore>,
}): {
    play: (pos: MaybePos) => void,
    resolveDesync: (hash: HashKey) => void,
    gameActions: GameActions,
} {
    const reconcileGameStore = () =>
        props.setGameStore(reconcile(buildGameStore(props.appState.boardWorker, props.appState.historyTree, flip(unwrap(props.gameStore.playerColor)))))

    const syncSet = (pos: MaybePos) => {
        if (props.appState.mintakaProvider && props.appState.mintakaProvider.state.type === "idle")
            props.appState.mintakaProvider.state.command({ type: "Play", content: pos })
    }

    const syncUnset = (pos: MaybePos, color: Color) => {
        if (props.appState.mintakaProvider && props.appState.mintakaProvider.state.type === "idle")
            return
    }

    const play = (pos: MaybePos) => {
        if (pos ? !props.appState.boardWorker.isLegalMove(pos) : true) return

        props.appState.boardWorker = props.appState.boardWorker.set(pos)
        const hashKey = props.appState.boardWorker.hashKey()

        props.appState.historyTree = props.appState.historyTree.push({ hashKey, pos })

        syncSet(pos)

        reconcileGameStore()
    }

    const launch = () => {
        if (props.appState.mintakaProvider === undefined) return

        if (props.appState.mintakaProvider.state.type === "in_computing") return

        props.setWorkerStore("inComputing", true)

        const _ = props.appState.mintakaProvider.state.launch(props.appState.boardWorker.hashKey(), "Best")
    }

    const abort = () => {
        if (props.appState.mintakaProvider === undefined) return

        if (props.appState.mintakaProvider.state.type === "idle") return

        props.appState.mintakaProvider.state.message({ type: "abort" })
    }

    const resolveDesync = (stopAt: HashKey) => {
        const result = props.appState.historyTree.backwardTo(stopAt)

        if (!result) return
        const [historyTree, entries] = result

        props.appState.historyTree = historyTree
        for (const entry of entries.reverse()) {
            props.appState.boardWorker = props.appState.boardWorker.unset(entry.pos)
        }

        reconcileGameStore()
    }

    const gameActions: GameActions = {
        play: (pos: Pos) => {
            play(pos)

            if (unwrap(props.workerStore.autoLaunch))
                launch()
        },
        forward: (method: ForwardMethod) => {
            const result = props.appState.historyTree.forward(method)
            if (!result) return
            const [historyTree, entry] = result

            props.appState.historyTree = historyTree
            props.appState.boardWorker = props.appState.boardWorker.set(entry.pos!)

            syncSet(entry.pos)

            reconcileGameStore()
        },
        bulkForward: (method: ForwardMethod) => {
            const result = props.appState.historyTree.bulkForward(method)
            if (!result) return
            const [historyTree, entries] = result

            props.appState.historyTree = historyTree
            for (const entry of entries) {
                props.appState.boardWorker = props.appState.boardWorker.set(entry.pos)
            }

            reconcileGameStore()
        },
        backward: () => {
            if (unwrap(props.workerStore.inComputing)) return

            const result = props.appState.historyTree.backward()
            if (!result) return
            const [historyTree, entry] = result

            props.appState.historyTree = historyTree
            props.appState.boardWorker = props.appState.boardWorker.unset(entry.pos!)

            reconcileGameStore()
        },
        bulkBackward: () => {
            if (unwrap(props.workerStore.inComputing)) return

            const result = props.appState.historyTree.bulkBackward()
            if (!result) return
            const [historyTree, entries] = result

            props.appState.historyTree = historyTree
            for (const entry of entries.reverse()) {
                props.appState.boardWorker = props.appState.boardWorker.unset(entry.pos)
            }

            reconcileGameStore()
        },
        start: () => {
            props.setWorkerStore("autoLaunch", true)

            launch()
        },
        pause: () => {
            props.setWorkerStore("autoLaunch", false)
        },
        abort: () => {
            props.setWorkerStore("autoLaunch", false)

            abort()
        },
    }

    return { play, resolveDesync, gameActions }
}
