import {useContext} from "solid-js";
import {AppContext} from "../context";
import {
    IconChevronDoubleLeft,
    IconChevronDoubleRight,
    IconChevronLeft,
    IconChevronRight,
    IconCog8Tooth,
    IconCpuChip,
    IconPlay
} from "./icons";

export function Control() {
    const { actions, appConfigStore, setAppConfigStore, gameStore } = useContext(AppContext)!

    return <div class="card bg-base-100 p-2">
        <div class="flex gap-2">
            <button
                class="btn btn-square"
            >
                {IconCog8Tooth}
            </button>
            <button
                class="btn btn-square"
                classList={{
                    "btn-error": gameStore.inBranchHead,
                    "btn-disabled": !gameStore.backwardable,
                }}
                onClick={actions.bulkBackward}
            >
                {IconChevronDoubleLeft}
            </button>
            <button
                class="btn btn-square"
                classList={{
                    "btn-error": gameStore.inBranchHead,
                    "btn-disabled": !gameStore.backwardable,
                }}
                onClick={actions.backward}
            >
                {IconChevronLeft}
            </button>
            <button
                class="btn btn-square"
                classList={{
                    "btn-disabled": true
                }}
            >
                {IconPlay}
            </button>
            <button
                class="btn btn-square"
                classList={{
                    "btn-warning": gameStore.inBranchHead,
                    "btn-disabled": !gameStore.forwardable,
                }}
                onClick={[actions.forward, "continue"]}
            >
                {IconChevronRight}
            </button>
            <button
                class="btn btn-square"
                classList={{
                    "btn-warning": gameStore.inBranchHead,
                    "btn-disabled": !gameStore.forwardable,
                }}
                onClick={[actions.bulkForward, "continue"]}
            >
                {IconChevronDoubleRight}
            </button>
            <button
                class="btn btn-square btn-active"
                classList={{
                    "btn-active": appConfigStore.openDashboard
                }}
            >
                {IconCpuChip}
            </button>
        </div>
    </div>
}
