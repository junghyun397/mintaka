import { useContext } from "solid-js"
import { AppContext, AppContextProvider } from "./context"
import { Board } from "./components/Board"
import { Control } from "./components/Control"
import { RootEvaluationBar } from "./components/EvaluationBar"
import { StatusMessage } from "./components/StatusMessage"
import { Dashboard } from "./components/Dashboard"

export function App() {
    return <AppContextProvider>
        <AppLayout />
    </AppContextProvider>
}

function AppLayout() {
    const { appConfigStore } = useContext(AppContext)!

    const boardHeight = () => {
        if (appConfigStore.zoomBoard)
            return "min(48rem,max(var(--breakpoint-zoom),100dvw),var(--board-max-height))"
        else
            return "min(48rem,100dvw,var(--board-max-height))"
    }

    return <div
        class="flex justify-center"
        style={{
            "--vertical-ui-space": "9rem",
            "--board-max-height": "calc(100dvh - var(--vertical-ui-space)",
            "--board-height": boardHeight(),
        }}
    >
        <div class="flex flex-col items-center gap-4 py-2 max-xs:gap-2">
            <RootEvaluationBar />
            <div class="overflow-x-hidden">
                <div class="max-w-dvw overflow-x-auto">
                    <div class="aspect-square h-(--board-height)">
                        <Board />
                    </div>
                </div>
            </div>
            <div class="flex flex-col items-center gap-2">
                <Control />
                <StatusMessage />
            </div>
        </div>
        <Dashboard />
    </div>
}
