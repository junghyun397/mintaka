import {AppContextProvider} from "./context";
import {Board} from "./components/Board";
import {Control} from "./components/Control";
import {EvaluationBar} from "./components/EvaluationBar";
import {Dashboard} from "./components/Dashboard";
import {History} from "./components/History";
import {StatusMessage} from "./components/StatusMessage";

export function App() {
    return <AppContextProvider>
        <div class="h-dvh bg-base-200 overflow-hidden flex">
            <aside class="shrink-0 overflow-auto">
                <History/>
            </aside>
            <main class="flex-1 flex flex-col min-h-0 items-center gap-4 *:max-w-3xl">
                <EvaluationBar value={0} />
                <Board />
                <Control />
                <StatusMessage />
            </main>
            <aside class="shrink-0 overflow-auto">
                <Dashboard/>
            </aside>
        </div>
    </AppContextProvider>
}
