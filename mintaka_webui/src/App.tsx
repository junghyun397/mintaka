import {AppContextProvider} from "./context";
import {Board} from "./components/Board";
import {Control} from "./components/Control";
import {EvaluationBar} from "./components/EvaluationBar";
import {Preference} from "./components/Preference";
import {History} from "./components/History";

export function App() {
    return <AppContextProvider>
        <div class="h-dvh overflow-hidden flex">
            <aside class="shrink-0 overflow-auto">
                <History/>
            </aside>
            <main class="flex-1 flex flex-col">
                <EvaluationBar value={0} />
                <div class="flex-1 min-h-0 flex items-center justify-center">
                    <Board/>
                </div>
                <Control />
            </main>
            <aside class="shrink-0 overflow-auto">
                <Preference/>
            </aside>
        </div>
    </AppContextProvider>
}
