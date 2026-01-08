import { HistorySource } from "./domain/HistoryTree"

type UrlParams = {
    readonly historySource?: HistorySource,
    readonly viewer?: true,
}

function parserUrlParams(): UrlParams {
    const params = new URLSearchParams(window.location.search)

    const moves = params.get("moves")
    const history = params.get("history")
    const historyTree = params.get("history-tree")
    const viewer = params.get("viewer")

    return { }
}

function pushUrlParams(params: UrlParams) {
    const url = new URL(window.location.href)

    if (params.viewer) {
        url.searchParams.set("viewer", "true")
    }

    window.history.replaceState({}, "", url.toString())
}
