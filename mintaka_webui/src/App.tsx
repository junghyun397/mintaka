import {AppContextProvider} from "./context";
import {Board} from "./components/Board";
import {Control} from "./components/Control";
import {EvaluationBar} from "./components/EvaluationBar";
import {Dashboard} from "./components/Dashboard";
import {StatusMessage} from "./components/StatusMessage";
import {History} from "./components/History";

export function App() {
    return <AppContextProvider>
        <div
            class="h-dvh bg-base-200 overflow-hidden grid content-start justify-center grid-rows-[auto_minmax(0,48rem)_auto] grid-cols-[auto_minmax(0,48rem)_auto] lg:gap-4"
        >
            <div class="row-start-1 col-start-2 flex justify-center max-lg:mb-4 pt-4">
                <div class="w-90">
                    <EvaluationBar value={0} />
                </div>
            </div>
            <aside class="fixed lg:static row-start-2 col-start-1">
                <History />
            </aside>
            <main class="row-start-2 col-start-2 @container-[size] flex items-center">
                <Board />
            </main>
            <aside class="fixed lg:static row-start-2 col-start-3">
                <Dashboard/>
            </aside>
            <div class="row-start-3 col-start-2 flex flex-col gap-1 items-center max-lg:mt-4">
                <Control />
                <StatusMessage />
            </div>
        </div>
    </AppContextProvider>
}
