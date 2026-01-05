import { HistoryTree } from "./domain/HistoryTree"
import { History } from "./wasm/pkg/mintaka_wasm"

type UrlParams = {
    readonly history?:
        | { type: "history", history: History }
        | { type: "history-tree", historyTree: HistoryTree },
    readonly viewer?: true,
}

function parserUrlParams(): UrlParams {
    const params = new URLSearchParams(window.location.search)

    const moves = params.get("moves")
    const history = params.get("history")
    const viewer = params.get("viewer")

    return {
        history: history ? JSON.parse(history) : undefined,
        viewer: viewer ? true : undefined,
    }
}

function pushUrlParams(params: UrlParams) {
    const url = new URL(window.location.href)

    if (params?.history?.type === "history") {
        url.searchParams.set("moves", params.history.history.join(","))
    }

    if (params?.history?.type === "history-tree") {
        url.searchParams.set("history", JSON.stringify(params.history.historyTree.toHistory()))
    }

    if (params.viewer) {
        url.searchParams.set("viewer", "true")
    }

    window.history.pushState({}, "", url.toString())
}
