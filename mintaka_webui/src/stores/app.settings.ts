import { createStore } from "solid-js/store"
import { UrlParams } from "../url"

export type AppSettings = {
    viewer: boolean,
    openDashboard: boolean,
    launch: boolean,
}

export function createAppSettingsStore(urlParams: UrlParams) {
    return createStore<AppSettings>({
        viewer: urlParams.viewer,
        openDashboard: false,
        launch: false,
    })
}
