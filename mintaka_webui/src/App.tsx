import { useContext } from "solid-js";
import { AppContext, AppContextProvider } from "./context";
import { Board } from "./components/Board";
import { Control } from "./components/Control";
import { RootEvaluationBar } from "./components/EvaluationBar";
import { Dashboard } from "./components/Dashboard";
import { StatusMessage } from "./components/StatusMessage";
import { History } from "./components/History";

export function App() {
    return <AppContextProvider>
        <AppLayout />
    </AppContextProvider>
}

function AppLayout() {
    const { appConfigStore } = useContext(AppContext)!

    return <div class="flex w-full justify-center gap-4">
        <aside
            class="w-36 justify-end pt-10 pb-26 max-md:hidden"
            classList={{ "hidden": appConfigStore.openHistory }}
        >
            <History />
        </aside>
        <div class="flex flex-col items-center gap-4 overflow-x-hidden py-2">
            <RootEvaluationBar />
            <div class="w-full overflow-x-auto">
                <main
                    class="aspect-square"
                    classList={{
                        "h-[min(48rem,100dvw,calc(100dvh-10rem))]": !appConfigStore.zoomBoard,
                        "h-[min(48rem,max(36rem,100dvw),calc(100dvh-10rem))]": appConfigStore.zoomBoard,
                    }}
                >
                    <Board />
                </main>
            </div>
            <nav class="flex flex-col items-center gap-2">
                <Control />
                <StatusMessage />
            </nav>
        </div>
        <aside
            class="w-64 max-md:hidden"
            classList={{ "hidden": !appConfigStore.openDashboard }}
        >
            <Dashboard />
        </aside>
    </div>
}
