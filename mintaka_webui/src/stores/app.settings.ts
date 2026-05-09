import { createStore } from "solid-js/store"
import { UrlParams } from "../url"

export type AppSettings = {
    viewer: boolean,
    launch: boolean,
    openDashboard: boolean,
}

export function createAppSettingsStore(urlParams: UrlParams) {
    return createStore<AppSettings>({
        viewer: urlParams.viewer,
        launch: false,
        openDashboard: false,
    })
}
