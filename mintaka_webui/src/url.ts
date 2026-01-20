import { History } from "./wasm/pkg/mintaka_wasm"
import { historyToString, parseHistory } from "./domain/rusty-renju"
import { Accessor, createEffect } from "solid-js"

export type UrlParams = {
    readonly moves: History | undefined,
    readonly viewer: boolean,
}

export function parseUrlParams(): UrlParams {
    const params = new URLSearchParams(window.location.search)

    const movesSource = params.get("moves")

    return {
        moves: movesSource === null ? undefined : parseHistory(movesSource),
        viewer: params.has("viewer"),
    }
}

export function pushUrlParams(params: UrlParams) {
    const url = new URL(window.location.href)

    if (params.viewer)
        url.searchParams.set("viewer", "")
    else
        url.searchParams.delete("viewer")

    const stringHistory = params.moves === undefined ? undefined : historyToString(params.moves)
    if (stringHistory !== undefined)
        url.searchParams.set("moves", stringHistory)
    else
        url.searchParams.delete("moves")

    window.history.replaceState({}, "", url)
}

export function setupUrlSync(history: Accessor<History>, viewer: Accessor<boolean>) {
    createEffect(() => {
        pushUrlParams({
            moves: history(),
            viewer: viewer(),
        })
    })
}
