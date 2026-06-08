import { Modal, type ModalControlProps } from "./Modal"
import { createMemo, Match, Switch, useContext } from "solid-js"
import { BoardWorker, type History, type RuleKind } from "../wasm/pkg/rusty_renju_wasm"
import { filter } from "../utils/undefined"
import { AppContext } from "../context"
import type { HistoryTreeNode } from "../domain/HistoryTree"
import { MiniBoard } from "./MiniBoard"

export function History(props: ModalControlProps) {
    const { gameSelectors } = useContext(AppContext)!

    const tree = createMemo(() => gameSelectors.gameState().historyTree.tree())

    return <Modal
        id="history_modal"
        title="History"
        open={props.open}
        onClose={props.onClose}
    >
        <div class="flex w-full flex-col items-start">
            <HistoryTreeBoard node={tree()} />
        </div>
    </Modal>
}

function HistoryTreeBoard(props: { node: HistoryTreeNode }) {
    const history = createMemo(() => filter(props.node, valid => valid.type === "history"))
    const branch = createMemo(() => filter(props.node, valid => valid.type === "branch"))

    return <Switch>
        <Match when={history()}>{node =>
            <div class="w-50 max-w-full">
                <HistoryBoard history={node().history} />
            </div>
        }</Match>
        <Match when={branch()}>{node =>
            <div class="flex w-full flex-col items-start">
                <ShowCurrentHistory history={node().current} />
                <div class="grid w-full grid-cols-2 gap-3">
                    <HistoryBoard history={node().currentBranchHead} />
                    <HistoryBoard history={node().branchHead} />
                </div>
                <TimelineConnector />
                <HistoryTreeBoard node={node().before} />
            </div>
        }</Match>
    </Switch>
}

function ShowCurrentHistory(props: { history: History | undefined }) {
    return <Switch>
        <Match when={props.history}>{history =>
            <>
                <div class="w-50 max-w-full">
                    <HistoryBoard history={history()} />
                </div>
                <TimelineConnector />
            </>
        }</Match>
    </Switch>
}

function TimelineConnector() {
    return <div class="flex h-5 w-50 max-w-full justify-center">
        <div class="w-px bg-base-content/20" />
    </div>
}

function HistoryBoard(props: { history: History }) {
    const { runtimeSelectors } = useContext(AppContext)!

    const describe = createMemo(() => BoardWorker.fromHistory(props.history, "Renju").describe())
    const last = createMemo(() => props.history.at(-1))

    return <MiniBoard
        describe={describe()}
        lastPos={last()}
    />
}
