export type Theme = "system" | "dark" | "light"

export type UiStore = {
    readonly theme: Theme,
    readonly historyOpen: boolean,
    readonly configOpen: boolean,
}
