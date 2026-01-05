/* @refresh reload */
import './index.css'
import { render } from 'solid-js/web'
import 'solid-devtools'
import { App } from "./App"

import init from "./wasm/pkg/mintaka_wasm"

await init()

const root = document.getElementById('root')

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
    throw new Error('Root element not found.')
}

render(() => <App />, root!)
