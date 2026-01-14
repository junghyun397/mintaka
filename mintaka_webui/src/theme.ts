import { createEffect, onCleanup, onMount } from "solid-js"
import { PersistConfig, Theme } from "./stores/persist.config"

const removeTheme = () =>
    document.documentElement.removeAttribute("data-theme")

const applyTheme = (theme: Exclude<Theme, "system">) => {
    document.documentElement.setAttribute("data-theme", theme)
}

export function setupThemeSync(persistConfig: PersistConfig) {
    onMount(() => {
        const mediaQueryList = window.matchMedia("(prefers-color-scheme: dark)")

        const handler = () => {
            if (persistConfig.theme === "system")
                removeTheme()
        }

        mediaQueryList.addEventListener?.("change", handler)

        onCleanup(() => {
            mediaQueryList.removeEventListener?.("change", handler)
        })
    })

    createEffect(() => {
        if (persistConfig.theme === "system")
            removeTheme()
        else
            applyTheme(persistConfig.theme)
    })
}
