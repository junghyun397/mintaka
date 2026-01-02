import { useContext } from "solid-js";
import { AppContext, AppContextProvider } from "./context";
import { Board } from "./components/Board";
import { Control } from "./components/Control";
import { RootEvaluationBar } from "./components/EvaluationBar";
import { StatusMessage } from "./components/StatusMessage";

export function App() {
    return <AppContextProvider>
        <AppLayout />
    </AppContextProvider>
}

function AppLayout() {
    const { appConfigStore } = useContext(AppContext)!

    const boardHeight = () => {
        if (appConfigStore.zoomBoard)
            return "min(48rem,max(36rem,100dvw),calc(100dvh - var(--vertical-ui-space)))"
        else
            return "min(48rem,100dvw,calc(100dvh - var(--vertical-ui-space))"
    }

    return <div
        class="flex justify-center"
        style={{
            "--vertical-ui-space": "10rem",
            "--board-height": boardHeight(),
        }}
    >
        <div class="flex flex-col items-center gap-4 py-2">
            <RootEvaluationBar />
            <div class="overflow-x-hidden">
                <div class="max-w-dvw overflow-x-auto">
                    <div
                        class="aspect-square"
                        style={{
                            height: "var(--board-height)",
                        }}
                    >
                        <Board />
                    </div>
                </div>
            </div>
            <div class="flex flex-col items-center gap-2">
                <Control />
                <StatusMessage />
            </div>
        </div>
    </div>
}
