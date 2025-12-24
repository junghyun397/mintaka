import {HistoryEntry, HistoryTree} from "../domain/history";
import {createStore, SetStoreFunction} from "solid-js/store";

export type HistoryStore = {
    readonly history: HistoryEntry[]
    readonly inBranchHead: boolean
}

export function createHistoryStore(historyTree: HistoryTree): [HistoryStore, SetStoreFunction<HistoryStore>] {
    return createStore({
        history: historyTree.linear,
        inBranchHead: historyTree.inBranchHead
    })
}
